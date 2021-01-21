use super::{ExprParsingType::*, Parser};

use crate::ast::{AstNode, BlockScope};
use crate::error::CrocoError;
use crate::{
    ast::{node::*, BackendNode},
    symbol::StructDecl,
};
use crate::{
    symbol_type::SymbolType,
    token::{CodePos, KeywordEnum::*, OperatorEnum::*, SeparatorEnum::*, Token, Token::*},
};
use std::collections::{BTreeMap, HashMap};

impl Parser {
    /// Parses a code block e.g for loop body, function body, etc.
    /// warning: it consumes the closing right bracket but not the opening one
    pub fn parse_block(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        scope: BlockScope,
        is_top_level: bool,
    ) -> Result<Box<dyn BackendNode>, CrocoError> {
        let mut block = BlockNode::new(scope);
        // loop until we have no token remaining
        loop {
            let token = self.peek_token(iter);
            if let EOF = token {
                break;
            }

            match token {
                // ending the block
                Separator(RightCurlyBracket) => {
                    self.next_token(iter);
                    break;
                }

                // declaring a new number variable
                Keyword(Let) => {
                    self.next_token(iter);
                    let mut out_node: Option<Box<dyn BackendNode>> = None;

                    // we're expecting a variable name
                    let identifier = self.expect_identifier(
                        iter,
                        "expected a variable name after the let keyword",
                    )?;

                    let mut assign_type = None;

                    match self.peek_token(iter) {
                        // we're giving a value to our variable with type inference
                        Operator(Assign) => {
                            self.next_token(iter);
                            out_node = Some(self.parse_expr(iter, AllowStructDeclaration)?);
                        }

                        // we're giving a type annotation
                        Keyword(Num)
                        | Keyword(Str)
                        | Keyword(Bool)
                        | Identifier(_)
                        | Operator(BitwiseAnd)
                        | Separator(LeftSquareBracket) => {
                            assign_type = Some(self.parse_var_type(iter)?);
                        }

                        // newline: we're declaring a variable without value or type
                        // for now we're not able to infer the variable type.
                        Separator(NewLine) | EOF => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                &format!("cannot infer the variable type of {}", identifier.name),
                            ))
                        }
                        _ => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                &format!("expected an equals sign after {}", identifier.name),
                            ))
                        }
                    }

                    // if we had a type annotation we need to check again for the variable value
                    if assign_type.is_some() {
                        match self.next_token(iter) {
                            Operator(Assign) => {
                                out_node = Some(self.parse_expr(iter, AllowStructDeclaration)?);
                            }
                            Separator(NewLine) | EOF => (),
                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    &format!("expected an equals sign after {}", identifier.name),
                                ))
                            }
                        }
                    }

                    // add this statement to the block
                    block.add_child(Box::new(VarDeclNode::new(
                        identifier.get_namespaced_name(),
                        out_node,
                        assign_type,
                        self.token_pos.clone(),
                    )));
                }

                // assigning a new value to a variable / struct field, or calling a function
                Identifier(_) | Operator(Multiplicate) => {
                    let lvalue_node = self.parse_identifier(iter, AllowStructDeclaration)?;

                    if let Operator(op_token) = self.peek_token(iter) {
                        self.next_token(iter);

                        // assigning to a variable
                        match op_token  {

                            Assign
                            | PlusEquals
                            | MinusEquals
                            | MultiplicateEquals
                            | DivideEquals
                            | PowerEquals => {

                                let expr_node = self.parse_expr(iter, AllowStructDeclaration)?;

                                // add to the root function this statement
                                if op_token == Assign {
                                    block.add_child(Box::new(AssignmentNode::new(lvalue_node, expr_node, self.token_pos.clone())));
                                } else {
                                    let mut dyn_op_node: Box<dyn BackendNode> = match op_token {
                                        PlusEquals => Box::new(PlusNode::new(self.token_pos.clone())),
                                        MinusEquals => Box::new(MinusNode::new(self.token_pos.clone())),
                                        MultiplicateEquals => {
                                            Box::new(MultiplicateNode::new(self.token_pos.clone()))
                                        }
                                        DivideEquals => Box::new(DivideNode::new(self.token_pos.clone())),
                                        PowerEquals => Box::new(PowerNode::new(self.token_pos.clone())),
                                        _ => unreachable!(),
                                    };

                                    dyn_op_node.add_child(lvalue_node.clone());
                                    dyn_op_node.add_child(expr_node);

                                    self.expect_token(iter, Separator(NewLine), "expected a new line after assignation")?;

                                    block.add_child(Box::new(AssignmentNode::new(lvalue_node, dyn_op_node, self.token_pos.clone())));
                                }
                            }

                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    "expected an assignation sign or a function call after the identifier"
                                ))
                            }
                        }
                    } else {
                        block.add_child(lvalue_node);
                    }
                }

                // declaring a struct
                Keyword(Struct) => {
                    self.next_token(iter);

                    if !is_top_level {
                        return Err(CrocoError::new(
                            &self.token_pos,
                            "structs can only be declared at top level",
                        ));
                    }

                    let struct_name = self.expect_identifier(
                        iter,
                        "expected the struct name after struct declaration",
                    )?;

                    self.expect_token(
                        iter,
                        Separator(LeftCurlyBracket),
                        "expected a left bracket after the struct name",
                    )?;

                    let mut fields: BTreeMap<String, SymbolType> = BTreeMap::new();
                    let mut methods: HashMap<String, Box<dyn BackendNode>> = HashMap::new();

                    loop {
                        self.discard_newlines(iter);

                        match self.next_token(iter) {
                            // struct method
                            Keyword(Function) => {
                                let (method_name, method_body) =
                                    self.parse_function_decl(iter, Some(struct_name.name.clone()))?;

                                // check if the method name isn't already a field name
                                if methods.contains_key(&method_name) {
                                    return Err(CrocoError::new(
                                        &self.token_pos,
                                        &format!("method {} is already defined as a field in this struct", method_name),
                                    ));
                                }

                                if methods.insert(method_name.clone(), method_body).is_some() {
                                    return Err(CrocoError::new(
                                        &self.token_pos,
                                        &format!("duplicate field {} in struct", method_name),
                                    ));
                                }
                            }

                            // struct field
                            Identifier(field_name) => {
                                // check if the field name isn't already a method name
                                if methods.contains_key(&field_name.name) {
                                    return Err(CrocoError::new(
                                        &self.token_pos,
                                        &format!("field {} is already defined as a method in this struct", field_name.name),
                                    ));
                                }

                                let field_type = self.parse_var_type(iter)?;

                                if fields.insert(field_name.name.clone(), field_type).is_some() {
                                    return Err(CrocoError::new(
                                        &self.token_pos,
                                        &format!("duplicate field {} in struct", field_name.name),
                                    ));
                                }
                            }

                            Separator(RightCurlyBracket) => break,

                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    "expected a field or a method name",
                                ))
                            }
                        }
                    }

                    // register the struct declaration
                    self.register_struct_decl(&struct_name.name, StructDecl { fields })?;

                    block.add_child(Box::new(StructDeclNode::new(
                        struct_name.name,
                        methods,
                        self.token_pos.clone(),
                    )));
                }

                // declaring a function
                Keyword(Function) => {
                    self.next_token(iter);

                    if !is_top_level {
                        return Err(CrocoError::new(
                            &self.token_pos,
                            "functions can only be declared at top level",
                        ));
                    }

                    let (fn_name, fn_body) = self.parse_function_decl(iter, None)?;

                    let fn_decl_node =
                        FunctionDeclNode::new(fn_name, fn_body, self.token_pos.clone());

                    // we can't use `prepend_child()` for now because it is hard for crocol
                    // to use forward dedclarations.
                    block.add_child(Box::new(fn_decl_node));
                }

                // returning a value
                Keyword(Return) => {
                    self.next_token(iter);

                    if is_top_level {
                        return Err(CrocoError::new(
                            &self.token_pos,
                            "can't return a value outside of a function",
                        ));
                    }

                    let return_node = self.parse_expr(iter, AllowStructDeclaration)?;
                    // TODO: correct CodePos
                    block.add_child(Box::new(ReturnNode::new(
                        return_node,
                        self.token_pos.clone(),
                    )));
                }

                // if block
                Keyword(If) => {
                    self.next_token(iter);

                    if is_top_level {
                        return Err(CrocoError::new(
                            &self.token_pos,
                            "cannot use a if outside a function",
                        )
                        .hint("add a main function"));
                    }

                    // we can have multiple conditions in an if elif construct, we use an array to keep track of all of them
                    let mut conditions: Vec<Box<dyn BackendNode>> = Vec::new();
                    // same for the statements inside the if / elif / else
                    let mut bodies: Vec<Box<dyn BackendNode>> = Vec::new();

                    conditions.push(self.parse_expr(iter, DenyStructDeclaration)?);

                    self.expect_token(
                        iter,
                        Separator(LeftCurlyBracket),
                        "expected left bracket after if expression",
                    )?;

                    bodies.push(self.parse_block(iter, BlockScope::New, false)?);

                    // handle the elif conditions
                    loop {
                        match self.peek_token(iter) {
                            Keyword(Elif) => {
                                self.next_token(iter);

                                conditions.push(self.parse_expr(iter, DenyStructDeclaration)?);

                                self.expect_token(
                                    iter,
                                    Separator(LeftCurlyBracket),
                                    "expected left bracket after elif expression",
                                )?;

                                bodies.push(self.parse_block(iter, BlockScope::New, false)?);
                            }

                            Keyword(Else) => {
                                self.next_token(iter);

                                self.expect_token(
                                    iter,
                                    Separator(LeftCurlyBracket),
                                    "expected left bracket after else expression",
                                )?;

                                bodies.push(self.parse_block(iter, BlockScope::New, false)?);

                                break;
                            }

                            _ => break,
                        }
                    }

                    block.add_child(Box::new(IfNode::new(
                        conditions,
                        bodies,
                        self.token_pos.clone(),
                    )));
                }

                // while loop
                Keyword(While) => {
                    self.next_token(iter);

                    if is_top_level {
                        return Err(CrocoError::new(
                            &self.token_pos,
                            "cannot use a while outside a function",
                        )
                        .hint("add a main function"));
                    }

                    let cond = self.parse_expr(iter, DenyStructDeclaration)?;

                    self.expect_token(
                        iter,
                        Separator(LeftCurlyBracket),
                        "expected a left bracket after while expression",
                    )?;

                    let body = self.parse_block(iter, BlockScope::New, false)?;
                    block.add_child(Box::new(WhileNode::new(cond, body, self.token_pos.clone())))
                }

                // break from a loop
                Keyword(Break) => {
                    self.next_token(iter);

                    if is_top_level {
                        return Err(CrocoError::new(
                            &self.token_pos,
                            "cannot break outside a loop",
                        ));
                    }

                    block.add_child(Box::new(BreakNode::new()))
                }
                // continue from a loop
                Keyword(Continue) => {
                    self.next_token(iter);

                    if is_top_level {
                        return Err(CrocoError::new(
                            &self.token_pos,
                            "cannot continue outside a loop",
                        ));
                    }

                    block.add_child(Box::new(ContinueNode::new()));
                }
                // importing a package
                Keyword(Import) => {
                    self.next_token(iter);

                    if !is_top_level {
                        return Err(CrocoError::new(
                            &self.token_pos,
                            "imports can only be declared at top level",
                        ));
                    }

                    let import_name =
                        self.expect_str(iter, "expected a str after the import keyword")?;
                    let import_node =
                        Box::new(ImportNode::new(import_name, self.token_pos.clone()));
                    block.add_child(import_node);
                }

                Separator(NewLine) => {
                    self.next_token(iter);
                }
                // TODO: impl line numbers / rows
                el => {
                    return Err(CrocoError::new(
                        &self.token_pos,
                        &format!("unexpected token: {:?}", el),
                    ))
                }
            }
        }

        Ok(Box::new(block))
    }
}
