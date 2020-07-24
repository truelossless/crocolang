use super::{ExprParsingType::*, Parser, TypedArg};

use crate::ast::node::*;
use crate::ast::{AstNode, BlockScope};
use crate::error::CrocoError;
use crate::symbol::{Struct, SymbolContent, Symbol};
use crate::token::{
    CodePos, KeywordEnum::*, LiteralEnum, OperatorEnum::*, SeparatorEnum::*, Token, Token::*,
};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

impl Parser {
    /// Parses a code block e.g for loop body, function body, etc.
    /// warning: it consumes the closing right bracket but not the opening one
    pub fn parse_block(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        scope: BlockScope,
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        let mut block = BlockNode::new(scope);
        // loop until we have no token remaining
        loop {
            let token = self.next_token(iter);
            if let EOF = token {
                break;
            }

            match token {
                // ending the block
                Separator(RightCurlyBracket) => break,

                // declaring a new number variable
                Keyword(Let) => {
                    let mut out_node: Option<Box<dyn AstNode>> = None;

                    // we're expecting a variable name
                    let identifier = self.expect_identifier(
                        iter,
                        "expected a variable name after the let keyword",
                    )?;

                    let mut assign_type: SymbolContent = SymbolContent::Primitive(LiteralEnum::Void);

                    match self.peek_token(iter) {
                        // we're giving a value to our variable with type inference
                        Operator(Assign) => {
                            self.next_token(iter);
                            out_node = Some(self.parse_expr(iter, AllowStructDeclaration)?);
                        }

                        // we're giving a type annotation
                        Keyword(Num) |
                        Keyword(Str) |
                        Keyword(Bool) |
                        Identifier(_) |
                        Separator(LeftSquareBracket) => {
                            assign_type = self.parse_var_type(iter)?;
                        }

                        // newline: we're declaring a variable without value or type
                        // for now we're not able to infer the variable type.
                        Separator(NewLine) | EOF => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                format!("cannot infer the variable type of {}", identifier.name),
                            ))
                        }
                        _ => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                format!("expected an equals sign after {}", identifier.name),
                            ))
                        }
                    }

                    // if we had a type annotation we need to check again for the variable value
                    if !assign_type.is_void() {
                        match self.next_token(iter) {
                            Operator(Assign) => {
                                out_node = Some(self.parse_expr(iter, AllowStructDeclaration)?);
                            }
                            Separator(NewLine) | EOF => (),
                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    format!("expected an equals sign after {}", identifier.name),
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
                Identifier(identifier) => {

                    let lvalue_node = self.parse_identifier(iter, identifier.clone(), DenyStructDeclaration)?;

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

                                let expr_node = self.parse_expr(iter, DenyStructDeclaration)?;

                                // add to the root function this statement
                                if op_token == Assign {
                                    block.add_child(Box::new(AssignmentNode::new(lvalue_node, expr_node, self.token_pos.clone())));
                                } else {
                                    let mut dyn_op_node: Box<dyn AstNode> = match op_token {
                                        PlusEquals => Box::new(PlusNode::new(self.token_pos.clone())),
                                        MinusEquals => Box::new(MinusNode::new(self.token_pos.clone())),
                                        MultiplicateEquals => {
                                            Box::new(MultiplicateNode::new(self.token_pos.clone()))
                                        }
                                        DivideEquals => Box::new(DivideNode::new(self.token_pos.clone())),
                                        PowerEquals => Box::new(PowerNode::new(self.token_pos.clone())),
                                        _ => unreachable!(),
                                    };
                                    
                                    let var_node = Box::new(VarCopyNode::new(identifier.name, self.token_pos.clone()));
                                    dyn_op_node.add_child(var_node);
                                    dyn_op_node.add_child(expr_node);

                                    self.expect_token(iter, Separator(NewLine), "expected a new line after assignation")?;

                                    block.add_child(Box::new(AssignmentNode::new(lvalue_node, dyn_op_node, self.token_pos.clone())));
                                }
                            }

                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    format!("expected an assignation sign or a function call after the identifier {}", identifier.name)
                                ))
                            }
                        }
                    } else {
                        block.add_child(lvalue_node);
                    }
                }

                // declaring a struct
                Keyword(Struct) => {
                    let identifier = self.expect_identifier(
                        iter,
                        "expected the struct name after struct declaration",
                    )?;

                    self.expect_token(
                        iter,
                        Separator(LeftCurlyBracket),
                        "expected a left bracket after the struct name",
                    )?;

                    let mut fields: HashMap<String, Symbol> = HashMap::new();

                    loop {
                        self.discard_newlines(iter);

                        // TODO: check if this introduces new bugs with namespaces
                        let field_name = match self.next_token(iter) {
                            Separator(RightCurlyBracket) => break,
                            Identifier(identifier) => identifier,
                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    "expected a field name".to_owned(),
                                ))
                            }
                        };

                        let field_type = match self.next_token(iter) {
                            Keyword(Num) => SymbolContent::Primitive(LiteralEnum::Num(None)),
                            Keyword(Str) => SymbolContent::Primitive(LiteralEnum::Str(None)),
                            Keyword(Bool) => SymbolContent::Primitive(LiteralEnum::Bool(None)),
                            Identifier(struct_type) => {
                                SymbolContent::Struct(Struct::new(struct_type.name))
                            }
                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    "expected a field type".to_owned(),
                                ))
                            }
                        };

                        if fields.insert(field_name.name, Rc::new(RefCell::new(field_type))).is_some() {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                "duplicate field in struct".to_owned(),
                            ));
                        }
                    }

                    block.add_child(Box::new(StructDeclNode::new(
                        identifier.name,
                        fields,
                        self.token_pos.clone(),
                    )));
                }

                // declaring a function
                Keyword(Function) => {
                    let identifier = self.expect_identifier(
                        iter,
                        "expected the function name after function declaration",
                    )?;

                    self.expect_token(
                        iter,
                        Separator(LeftParenthesis),
                        "expected a left parenthensis after the function name",
                    )?;

                    let mut typed_args: Vec<TypedArg> = Vec::new();

                    let mut first_arg = false;

                    loop {
                        match self.peek_token(iter) {
                            Separator(RightParenthesis) => {
                                self.next_token(iter);
                                break;
                            }
                            Separator(Comma) if first_arg => {
                                self.next_token(iter);
                            }

                            Separator(Comma) =>
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    "no argument before comma".to_owned(),
                                )),

                            _ if !first_arg => (),

                            _ =>
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    format!(
                                    "expected a comma or a right parenthesis in {} function declaration",
                                    identifier.name),
                                ))
                        }

                        first_arg = true;

                        self.discard_newlines(iter);

                        // we're expecting an argument variable name here
                        let arg_name = self.expect_identifier(
                            iter,
                            &format!(
                                "expected an argument name in {} function declaration",
                                identifier.name
                            ),
                        )?;

                        // here this should be the argument type
                        let arg_type = match self.next_token(iter) {
                            Keyword(Num) => SymbolContent::Primitive(LiteralEnum::Num(None)),
                            Keyword(Str) => SymbolContent::Primitive(LiteralEnum::Str(None)),
                            Keyword(Bool) => SymbolContent::Primitive(LiteralEnum::Bool(None)),
                            Identifier(identifier) => SymbolContent::Struct(Struct {
                                fields: None,
                                struct_type: identifier.name
                            }),
                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    format!("expected an argument type for {}", arg_name.name),
                                ))
                            }
                        };
                        typed_args.push(TypedArg{
                            arg_name: arg_name.name,
                            arg_type
                        });
                    }

                    // Might allow weird parsing: does it matter ?
                    // fn bla()
                    // Void
                    // { ...
                    self.discard_newlines(iter);

                    // if the return type isn't specified the function is Void
                    let mut return_type = SymbolContent::Primitive(LiteralEnum::Void);

                    match self.next_token(iter) {
                        Keyword(Num) => return_type = SymbolContent::Primitive(LiteralEnum::Num(None)),
                        Keyword(Str) => return_type = SymbolContent::Primitive(LiteralEnum::Str(None)),
                        Keyword(Bool) => return_type = SymbolContent::Primitive(LiteralEnum::Bool(None)),
                        Separator(LeftCurlyBracket) => (),
                        _ => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                format!(
                                    "expected left bracket after {} function declaration",
                                    identifier.name
                                ),
                            ))
                        }
                    }

                    self.discard_newlines(iter);

                    if !return_type.is_void() {
                        self.expect_token(
                            iter,
                            Separator(LeftCurlyBracket),
                            &format!(
                                "expected left bracket after {} function declaration",
                                identifier.name
                            ),
                        )?;
                    }

                    // we can't declare a function with a dot in its name
                    if identifier.name.contains('.') {
                        return Err(CrocoError::new(
                            &self.token_pos,
                            "a function cannot have a dot in its name".to_owned(),
                        ));
                    }

                    // get the namespaced name of the function
                    let fn_name = identifier.get_namespaced_name();

                    let mut func_decl = FunctionDeclNode::new(
                        fn_name,
                        return_type,
                        typed_args,
                        self.token_pos.clone(),
                    );
                    func_decl.add_child(self.parse_block(iter, BlockScope::New)?);

                    block.add_child(Box::new(func_decl));
                }

                // returning a value
                Keyword(Return) => {
                    let return_node = self.parse_expr(iter, AllowStructDeclaration)?;
                    // TODO: correct CodePos
                    block.add_child(Box::new(ReturnNode::new(
                        return_node,
                        self.token_pos.clone(),
                    )));
                }

                // if block
                Keyword(If) => {
                    // we can have multiple conditions in an if  elif construct, we use an array to keep track of all of them
                    let mut conditions: Vec<Box<dyn AstNode>> = Vec::new();
                    // same for the statements inside the if / elif / else
                    let mut bodies: Vec<Box<dyn AstNode>> = Vec::new();

                    conditions.push(self.parse_expr(iter, DenyStructDeclaration)?);

                    self.expect_token(
                        iter,
                        Separator(LeftCurlyBracket),
                        "expected left bracket after if expression",
                    )?;

                    bodies.push(self.parse_block(iter, BlockScope::New)?);

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

                                bodies.push(self.parse_block(iter, BlockScope::New)?);
                            }

                            Keyword(Else) => {
                                self.next_token(iter);

                                self.expect_token(
                                    iter,
                                    Separator(LeftCurlyBracket),
                                    "expected left bracket after else expression",
                                )?;

                                bodies.push(self.parse_block(iter, BlockScope::New)?);

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
                    let cond = self.parse_expr(iter, DenyStructDeclaration)?;

                    self.expect_token(
                        iter,
                        Separator(LeftCurlyBracket),
                        "expected a left bracket after while expression",
                    )?;

                    let body = self.parse_block(iter, BlockScope::New)?;
                    block.add_child(Box::new(WhileNode::new(cond, body, self.token_pos.clone())))
                }

                // break from a loop
                Keyword(Break) => block.add_child(Box::new(BreakNode::new())),

                // continue from a loop
                Keyword(Continue) => block.add_child(Box::new(ContinueNode::new())),

                // dynamically importing a package
                Keyword(Import) => {
                    let import_name =
                        self.expect_str(iter, "expected a str after the import keyword")?;
                    let import_node =
                        Box::new(ImportNode::new(import_name, self.token_pos.clone()));
                    block.add_child(import_node);
                }

                Separator(NewLine) => continue,
                // TODO: impl line numbers / rows
                el => {
                    return Err(CrocoError::new(
                        &self.token_pos,
                        format!("unexpected token: {:?}", el),
                    ))
                }
            }
        }

        Ok(Box::new(block))
    }
}
