use crate::ast::*;
use crate::token;
use crate::token::{
    KeywordEnum::*, LiteralEnum, OperatorEnum::*, SeparatorEnum::*, Token, Token::*,
};

#[derive(Clone, Debug)]
pub struct TypedArg {
    pub arg_name: String,
    pub arg_type: LiteralEnum,
}

impl TypedArg {
    pub fn new(arg_name: String, arg_type: LiteralEnum) -> Self {
        TypedArg { arg_name, arg_type }
    }
}

struct ParsedIdentifier {
    content: AstNode,
    is_fn_call: bool,
}

impl ParsedIdentifier {
    fn new(content: AstNode, is_fn_call: bool) -> Self {
        ParsedIdentifier {
            content,
            is_fn_call,
        }
    }
}

#[derive(Default)]
pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    /// util to get the namespaced name of an identifier
    fn get_namespaced_name(identifier: &token::Identifier) -> String {

        if identifier.namespace.is_empty() {
            identifier.name.clone()
        } else {
            format!("{}.{}", identifier.namespace, identifier.name)
        }
    }

    /// util to build a node from a token
    fn get_node(token: Token) -> Result<AstNode, String> {
        // println!("got token {:?}", token);

        match token {
            Identifier(identifier) => {
                Ok(AstNode::LeafNode(Box::new(VarNode::new(identifier.name))))
            }
            Literal(literal) => Ok(AstNode::LeafNode(Box::new(LiteralNode::new(literal)))),
            Operator(Plus) => Ok(AstNode::BinaryNode(Box::new(PlusNode::new()))),
            Operator(Minus) => Ok(AstNode::BinaryNode(Box::new(MinusNode::new()))),
            Operator(Multiplicate) => Ok(AstNode::BinaryNode(Box::new(MultiplicateNode::new()))),
            Operator(Divide) => Ok(AstNode::BinaryNode(Box::new(DivideNode::new()))),
            Operator(Power) => Ok(AstNode::BinaryNode(Box::new(PowerNode::new()))),
            Operator(Equals) => Ok(AstNode::BinaryNode(Box::new(CompareNode::new(Equals)))),
            Operator(NotEquals) => Ok(AstNode::BinaryNode(Box::new(CompareNode::new(NotEquals)))),
            Operator(GreaterOrEqual) => Ok(AstNode::BinaryNode(Box::new(CompareNode::new(
                GreaterOrEqual,
            )))),
            Operator(GreaterThan) => {
                Ok(AstNode::BinaryNode(Box::new(CompareNode::new(GreaterThan))))
            }
            Operator(LowerOrEqual) => Ok(AstNode::BinaryNode(Box::new(CompareNode::new(
                LowerOrEqual,
            )))),
            Operator(LowerThan) => Ok(AstNode::BinaryNode(Box::new(CompareNode::new(LowerThan)))),
            _ => Err(format!("can't evaluate token in expression: {:?}", token)),
        }
    }

    /// util to add a node to the output
    fn add_node(output: &mut Vec<AstNode>, token: Token) -> Result<(), String> {
        let root_node_obj = Parser::get_node(token)?;

        let mut binary_root_node = match root_node_obj {
            AstNode::BinaryNode(node) => node,
            _ => panic!("Trying to add a node in an expr but the parent is not a binary node !"),
        };

        match output.pop() {
            Some(x) => binary_root_node.set_right(x),
            None => return Err("missing element in expression".to_string()),
        }

        match output.pop() {
            Some(x) => binary_root_node.set_left(x),
            None => return Err("missing element in expression".to_string()),
        }

        output.push(AstNode::BinaryNode(binary_root_node));
        Ok(())
    }

    /// Parses an identifier into either a FunctionCallNode or a VariableNode
    fn parse_identifier(
        iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
        identifier: token::Identifier,
    ) -> Result<ParsedIdentifier, String> {
        let next_token = iter.peek();

        match next_token {
            Some(Separator(LeftParenthesis)) => (),
            _ => {
                return Ok(ParsedIdentifier::new(
                    AstNode::LeafNode(Box::new(VarNode::new(identifier.name))),
                    false,
                ))
            }
        }
        iter.next();
        let mut fn_args: Vec<AstNode> = Vec::new();

        loop {
            // TODO: avoid hello()) function calls
            if let Some(Separator(RightParenthesis)) = iter.peek() {
                iter.next();
                break;
            }

            fn_args.push(Parser::parse_expr(iter)?);
            Parser::discard_newlines(iter);

            match iter.next() {
                Some(Separator(Comma)) => (),
                Some(Separator(RightParenthesis)) => break,
                _ => {
                    return Err(format!(
                        "unexpected token in {} function call",
                        identifier.name
                    ))
                }
            }
        }

        Ok(ParsedIdentifier::new(
            AstNode::NaryNode(Box::new(FunctionCallNode::new(identifier.name, fn_args))),
            true,
        ))
    }

    /// Parses an expression using the shunting-yard algorithm.
    // https://brilliant.org/wiki/shunting-yard-algorithm
    // https://en.wikipedia.org/wiki/Shunting-yard_algorithm
    // https://www.klittlepage.com/2013/12/22/twelve-days-2013-shunting-yard-algorithm/
    fn parse_expr(
        iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
    ) -> Result<AstNode, String> {
        // and an expression to finish.
        let mut stack: Vec<Token> = Vec::new(); // == operand stack
        let mut output: Vec<AstNode> = Vec::new(); // == operator stack

        // util to know which operator has the highest priority (higher value is higher priority)
        let get_precedence = |op: &Token| -> u8 {
            match op {
                Operator(Or) => 1,
                Operator(And) => 2,
                Operator(Equals) | Operator(NotEquals) => 3,
                Operator(GreaterOrEqual)
                | Operator(GreaterThan)
                | Operator(LowerOrEqual)
                | Operator(LowerThan) => 4,
                Operator(Plus) | Operator(Minus) => 5,
                Operator(Multiplicate) | Operator(Divide) => 6,
                Operator(Power) => 7,
                _ => unreachable!(),
            }
        };

        // util to know if an operator can be right-associative
        // e.g 3+4 == 4+3
        // but 3^4 != 4^3
        let right_associative = |op: &Token| -> bool {
            match op {
                Operator(Divide)
                | Operator(Minus)
                | Operator(Power)
                | Operator(GreaterOrEqual)
                | Operator(GreaterThan)
                | Operator(LowerOrEqual)
                | Operator(LowerThan) => false,
                _ => true,
            }
        };

        // if we encouter a right parenthesis while there's no left parenthesis in our expression that
        // means that we're in this situation :
        // call_my_fn(3 + 4) <- this right parenthesis is the end of the function
        let mut parenthesis_opened = false;

        while let Some(next_token) = iter.peek() {
            // make sure that this token belongs to the expression
            match next_token {
                // the right parenthesis is the end of a function
                Separator(RightParenthesis) => {
                    if !parenthesis_opened {
                        break;
                    }
                }

                // end of an expr
                Separator(NewLine) | Separator(Comma) | Separator(LeftBracket) => break,
                _ => (),
            }

            // now that we know that the token is right, we can consume it
            let token_expr = iter.next().unwrap(); // we can safely unwrap as we already have peeked

            match token_expr {
                Identifier(identifier) => {
                    output.push(Parser::parse_identifier(iter, identifier)?.content)
                }
                Literal(_) => output.push(Parser::get_node(token_expr)?),
                Operator(_) => {
                    while let Some(top) = stack.last() {
                        match top {
                            Operator(_) => {
                                if (!right_associative(&top)
                                    && get_precedence(&top) == get_precedence(&token_expr))
                                    || get_precedence(&top) > get_precedence(&token_expr)
                                {
                                    let op = stack.pop().unwrap();

                                    match op {
                                        Operator(_) => Parser::add_node(&mut output, op)?,
                                        _ => panic!("not an operator found in the stack"),
                                    }
                                } else {
                                    break;
                                }
                            }

                            Separator(_) => break,
                            _ => (unreachable!()),
                        }
                    }

                    stack.push(token_expr);
                }
                Separator(LeftParenthesis) => {
                    stack.push(token_expr);
                    parenthesis_opened = true;
                }
                Separator(RightParenthesis) => {
                    while let Some(top) = stack.last() {
                        match top {
                            Separator(LeftParenthesis) => {
                                stack.pop();
                                break;
                            }
                            _ => {
                                let popped = stack.pop();
                                match popped {
                                    Some(Operator(_)) => {
                                        Parser::add_node(&mut output, popped.unwrap())?
                                    }
                                    None => {
                                        return Err("missing parenthesis in expression".to_string())
                                    }
                                    _ => {
                                        return Err(format!(
                                            "expected an operator but got {:?}",
                                            popped
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
                _ => return Err("unexpected symbol in math expression".to_string()),
            }

            // println!("stack: {:?}", stack);
            // println!("output: {:?}", output);
        }

        while !stack.is_empty() {
            let popped = stack.pop().unwrap();

            match popped {
                Separator(LeftParenthesis) => {
                    return Err("missing parenthesis in expression".to_string())
                }
                _ => Parser::add_node(&mut output, popped)?,
            }
        }

        if output.is_empty() {
            return Ok(AstNode::LeafNode(Box::new(LiteralNode::new(
                LiteralEnum::Void,
            ))));
        }

        Ok(output.pop().unwrap())
    }

    /// Parses a code block e.g for loop body, function body, etc.
    /// warning: it consumes the closing right bracket but not the opening one
    fn parse_block(
        iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
    ) -> Result<AstNode, String> {
        let mut block = BlockNode::new();
        while let Some(token) = iter.next() {
            match token {
                // ending the block
                Separator(RightBracket) => break,

                // declaring a new number variable
                Keyword(Let) => {
                    let identifier;
                    // rust is too dumb to figure out that out_node is always initalized so we
                    // have to wrap out_node in an Option
                    let mut out_node: Option<AstNode> = None;

                    // we're expecting a variable name
                    match iter.next() {
                        Some(Identifier(x)) => identifier = x,
                        _ => {
                            return Err("expected a variable name after the let keyword".to_owned())
                        }
                    }

                    let mut assign_type: LiteralEnum = LiteralEnum::Void;

                    match iter.next().as_ref() {
                        // we're giving a value to our variable with type inference
                        Some(Operator(Assign)) => {
                            out_node = Some(Parser::parse_expr(iter)?);
                        }

                        // we're giving a type annotation
                        Some(Keyword(Num)) => assign_type = LiteralEnum::Number(None),
                        Some(Keyword(Str)) => assign_type = LiteralEnum::Text(None),
                        Some(Keyword(Bool)) => assign_type = LiteralEnum::Boolean(None),

                        // newline: we're declaring a variable without value or type
                        // for now we're not able to infer the variable type.
                        Some(Separator(NewLine)) => {
                            return Err(format!("cannot infer the variable type of {}", identifier.name))
                        }
                        _ => return Err(format!("expected an equals sign after {}", identifier.name)),
                    }

                    // if we had a type annotation we need to check again for the variable value
                    if !assign_type.is_void() {
                        match iter.next() {
                            Some(Operator(Assign)) => {
                                out_node = Some(Parser::parse_expr(iter)?);
                            }
                            // we can infer the default value since we have the type annotation
                            Some(Separator(NewLine)) => {
                                out_node = match assign_type {
                                    LiteralEnum::Boolean(_) => Some(Parser::get_node(Literal(
                                        LiteralEnum::Boolean(Some(false)),
                                    ))?),
                                    LiteralEnum::Number(_) => Some(Parser::get_node(Literal(
                                        LiteralEnum::Number(Some(0.)),
                                    ))?),
                                    LiteralEnum::Text(_) => Some(Parser::get_node(Literal(
                                        LiteralEnum::Text(Some("".to_owned())),
                                    ))?),
                                    _ => {
                                        return Err(format!(
                                            "cannot infer default value for {}",
                                            identifier.name
                                        ))
                                    }
                                }
                            }
                            _ => return Err(format!("expected an equals sign after {}", identifier.name)),
                        }
                    }

                    // add this statement to the block
                    block.add_child(AstNode::BinaryNode(Box::new(DeclNode::new(
                        Parser::get_namespaced_name(&identifier),
                        out_node.unwrap(),
                        assign_type,
                    ))));
                }

                // assigning a new value to a variable, or calling a function
                Identifier(identifier) => {
                    let parsed_identifier = Parser::parse_identifier(iter, identifier.clone())?;

                    // we're calling a function
                    if parsed_identifier.is_fn_call {
                        block.add_child(parsed_identifier.content);
                        continue;
                    }

                    let next_token = iter.next();

                    match next_token {

                        // assigning to a variable
                        Some(Operator(Assign))
                        | Some(Operator(PlusEquals))
                        | Some(Operator(MinusEquals))
                        | Some(Operator(MultiplicateEquals))
                        | Some(Operator(DivideEquals))
                        | Some(Operator(PowerEquals)) => {

                            let out_node = Parser::parse_expr(iter)?;

                            // add to the root function this statement
                            if next_token == Some(Operator(Assign)) {
                                block.add_child(AstNode::BinaryNode(Box::new(AssignmentNode::new(identifier.name, out_node))));
                            } else {
                                let mut dyn_op_node: Box<dyn BinaryNodeTrait> = match next_token {
                                    Some(Operator(PlusEquals)) => Box::new(PlusNode::new()),
                                    Some(Operator(MinusEquals)) => Box::new(MinusNode::new()),
                                    Some(Operator(MultiplicateEquals)) => {
                                        Box::new(MultiplicateNode::new())
                                    }
                                    Some(Operator(DivideEquals)) => Box::new(DivideNode::new()),
                                    Some(Operator(PowerEquals)) => Box::new(PowerNode::new()),
                                    _ => unreachable!(),
                                };
                                let var_node = AstNode::LeafNode(Box::new(VarNode::new(identifier.name.clone())));
                                dyn_op_node.set_left(var_node);
                                dyn_op_node.set_right(out_node);
                                let op_node = AstNode::BinaryNode(dyn_op_node);
                                block.add_child(AstNode::BinaryNode(Box::new(AssignmentNode::new(identifier.name, op_node))));
                            }
                        }
                        _ => {
                            return Err(format!(
                                "expected an assignation sign or a function call after the identifier {}",
                                identifier.name
                            ))
                        }
                    }
                }

                // declaring a function
                Keyword(Function) => {
                    let identifier = match iter.next() {
                        Some(Identifier(identifier)) => identifier,
                        _ => {
                            return Err(
                                "expecting the function name after function declaration".to_owned()
                            )
                        }
                    };

                    match iter.next() {
                        Some(Separator(LeftParenthesis)) => (),
                        _ => {
                            return Err(
                                "expecting a left parenthensis after the function name".to_owned()
                            )
                        }
                    }

                    let mut typed_args: Vec<TypedArg> = Vec::new();
                    // continue to look for args while we haven't found a right parenthesis
                    let mut parenthesis_consumed = false;
                    while Some(&Separator(RightParenthesis)) != iter.peek() {
                        Parser::discard_newlines(iter);

                        // we're expecting an argument variable name here
                        let arg_name = match iter.next() {
                            Some(Identifier(arg_identifier)) => arg_identifier,
                            _ => {
                                return Err(format!(
                                    "expected an argument name in {} function declaration",
                                    identifier.name
                                ))
                            }
                        };

                        // here this should be the argument type
                        let arg_type = match iter.next() {
                            Some(Keyword(Num)) => LiteralEnum::Number(None),
                            Some(Keyword(Str)) => LiteralEnum::Text(None),
                            Some(Keyword(Bool)) => LiteralEnum::Boolean(None),
                            _ => {
                                return Err(format!(
                                    "expected an argument type for {}",
                                    arg_name.name
                                ))
                            }
                        };
                        typed_args.push(TypedArg::new(arg_name.name, arg_type));

                        Parser::discard_newlines(iter);

                        match iter.next() {
                            Some(Separator(Comma)) => (),
                            Some(Separator(RightParenthesis)) => {
                                parenthesis_consumed = true;
                                break;
                            }
                            _ => {
                                return Err(format!(
                                    "expected a comma or a right parenthesis in {} function call",
                                    identifier.name
                                ))
                            }
                        }
                    }

                    // consume the right parenthesis
                    if !parenthesis_consumed {
                        iter.next();
                    }

                    // TODO: might allow weird parsing: does it matter ?
                    // fn bla()
                    // Void
                    // { ...
                    Parser::discard_newlines(iter);

                    // if the return type isn't specified the function is Void
                    let mut return_type = LiteralEnum::Void;

                    match iter.next() {
                        Some(Keyword(Num)) => return_type = LiteralEnum::Number(None),
                        Some(Keyword(Str)) => return_type = LiteralEnum::Text(None),
                        Some(Keyword(Bool)) => return_type = LiteralEnum::Boolean(None),
                        Some(Separator(LeftBracket)) => (),
                        _ => {
                            return Err(format!(
                                "expected left bracket after {} function declaration",
                                identifier.name
                            ))
                        }
                    }

                    Parser::discard_newlines(iter);

                    if return_type != LiteralEnum::Void {
                        match iter.next() {
                            Some(Separator(LeftBracket)) => (),
                            _ => {
                                return Err(format!(
                                    "expected left bracket after {} function declaration",
                                    identifier.name
                                ))
                            }
                        }
                    }

                    // we can't declare a function with a dot in its name
                    if identifier.name.contains('.') {
                        return Err("a function cannot have a dot in its name".to_owned());
                    }

                    // get the namespaced name of the function
                    let fn_name = Parser::get_namespaced_name(&identifier);
                    
                    let mut func_decl =
                        FunctionDeclNode::new(fn_name, return_type, typed_args);
                    func_decl.set_bottom(Parser::parse_block(iter)?);

                    block.add_child(AstNode::UnaryNode(Box::new(func_decl)));
                }

                // returning a value
                Keyword(Return) => {
                    let return_node = Parser::parse_expr(iter)?;
                    block.add_child(AstNode::UnaryNode(Box::new(ReturnNode::new(return_node))));
                }

                // if block
                Keyword(If) => {
                    let cond = Parser::parse_expr(iter)?;

                    Parser::expect(
                        iter,
                        Separator(LeftBracket),
                        "expected left bracket after if expression",
                    )?;

                    let body = Parser::parse_block(iter)?;
                    block.add_child(AstNode::BinaryNode(Box::new(IfNode::new(cond, body))))
                }

                // while loop
                Keyword(While) => {
                    let cond = Parser::parse_expr(iter)?;

                    Parser::expect(
                        iter,
                        Separator(LeftBracket),
                        "expected a left bracket after while expression",
                    )?;

                    let body = Parser::parse_block(iter)?;
                    block.add_child(AstNode::BinaryNode(Box::new(WhileNode::new(cond, body))))
                }

                // break from a loop
                Keyword(Break) => block.add_child(AstNode::LeafNode(Box::new(BreakNode::new()))),

                // continue from a loop
                Keyword(Continue) => {
                    block.add_child(AstNode::LeafNode(Box::new(ContinueNode::new())))
                }

                Separator(NewLine) => continue,
                // TODO: impl line numbers / rows
                el => return Err(format!("unexpected token: {:?}", el)),
            }
        }

        Ok(AstNode::NaryNode(Box::new(block)))
    }

    /// Expects a token
    fn expect(
        iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
        token: Token,
        error_msg: &str,
    ) -> Result<(), String> {
        if iter.next() == Some(token) {
            Ok(())
        } else {
            Err(error_msg.to_owned())
        }
    }

    /// Discards all next tokens that are newlines
    fn discard_newlines(iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>) {
        while let Some(token) = iter.peek() {
            match token {
                Separator(NewLine) => {
                    iter.next();
                }
                _ => break,
            }
        }
    }

    /// builds an Abstact Syntax Tree (AST) with the results of the lexer.
    pub fn process(&mut self, tokens: Vec<Token>) -> Result<AstNode, String> {
        // iterator which returns a movable and peekable token iterator
        let mut iter = tokens.into_iter().peekable();
        let root = Parser::parse_block(&mut iter)?;
        Ok(root)
    }
}
