use crate::ast::*;
use crate::error::CrocoError;
use crate::token;
use crate::token::{
    CodePos, KeywordEnum::*, LiteralEnum, OperatorEnum::*, SeparatorEnum::*, Token, Token::*,
};
use Token::EOF;

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
    content: Box<dyn AstNode>,
    is_fn_call: bool,
}

impl ParsedIdentifier {
    fn new(content: Box<dyn AstNode>, is_fn_call: bool) -> Self {
        ParsedIdentifier {
            content,
            is_fn_call,
        }
    }
}
#[derive(Default)]
pub struct Parser {
    scope: BlockScope,
    token_pos: CodePos,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            scope: BlockScope::New,
            token_pos: CodePos {
                file: String::new(),
                line: 0,
                word: 0,
            },
        }
    }

    pub fn next_token(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) -> Token {
        if let Some((token, pos)) = iter.next() {
            self.token_pos = pos;
            token
        } else {
            Token::EOF
        }
    }

    // i'm not really sure how I could return a reference created in this function (if it's even possible)
    // maybe something like peek_token<'a>(&mut self, iter: &'a mut iter....) -> &'a Token
    pub fn peek_token(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) -> Token {
        match iter.peek() {
            Some((token, pos)) => {
                self.token_pos = pos.clone();
                token.clone()
            }
            None => EOF,
        }
    }

    pub fn set_scope(&mut self, scope: BlockScope) {
        self.scope = scope;
    }

    /// util to build a node from a token
    fn get_node(&self, token: Token) -> Result<Box<dyn AstNode>, CrocoError> {
        // println!("got token {:?}", token);

        match token {
            Identifier(identifier) => Ok(Box::new(VarNode::new(
                identifier.name,
                self.token_pos.clone(),
            ))),
            Literal(literal) => Ok(Box::new(LiteralNode::new(literal))),
            Operator(Plus) => Ok(Box::new(PlusNode::new(self.token_pos.clone()))),
            Operator(Minus) => Ok(Box::new(MinusNode::new(self.token_pos.clone()))),
            Operator(UnaryMinus) => Ok(Box::new(UnaryMinusNode::new(self.token_pos.clone()))),
            Operator(Multiplicate) => Ok(Box::new(MultiplicateNode::new(self.token_pos.clone()))),
            Operator(Divide) => Ok(Box::new(DivideNode::new(self.token_pos.clone()))),
            Operator(Power) => Ok(Box::new(PowerNode::new(self.token_pos.clone()))),
            Operator(Equals) => Ok(Box::new(CompareNode::new(Equals, self.token_pos.clone()))),
            Operator(NotEquals) => Ok(Box::new(CompareNode::new(
                NotEquals,
                self.token_pos.clone(),
            ))),
            Operator(GreaterOrEqual) => Ok(Box::new(CompareNode::new(
                GreaterOrEqual,
                self.token_pos.clone(),
            ))),
            Operator(GreaterThan) => Ok(Box::new(CompareNode::new(
                GreaterThan,
                self.token_pos.clone(),
            ))),
            Operator(LowerOrEqual) => Ok(Box::new(CompareNode::new(
                LowerOrEqual,
                self.token_pos.clone(),
            ))),
            Operator(LowerThan) => Ok(Box::new(CompareNode::new(
                LowerThan,
                self.token_pos.clone(),
            ))),
            Operator(Bang) => Ok(Box::new(NotNode::new(self.token_pos.clone()))),
            _ => Err(CrocoError::new(
                &self.token_pos,
                format!("can't evaluate token in expression: {:?}", token),
            )),
        }
    }

    /// util to add a node to the output
    fn add_node(
        &mut self,
        output: &mut Vec<Box<dyn AstNode>>,
        token: Token,
    ) -> Result<(), CrocoError> {
        let pos = self.token_pos.clone();
        let mut root_node = self.get_node(token)?;

        let right = match output.pop() {
            Some(x) => x,
            None => {
                return Err(CrocoError::new(
                    &pos,
                    "missing element in expression".to_owned(),
                ))
            }
        };

        // if we have a binary node we must get two elements on the output
        if let AstNodeType::BinaryNode = root_node.get_type() {
            let left = match output.pop() {
                Some(x) => x,
                None => {
                    return Err(CrocoError::new(
                        &pos,
                        "missing element in expression".to_owned(),
                    ))
                }
            };

            root_node.add_child(left);
        }

        root_node.add_child(right);

        output.push(root_node);
        Ok(())
    }

    /// Parses an identifier into either a FunctionCallNode or a VariableNode
    fn parse_identifier(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        identifier: token::Identifier,
    ) -> Result<ParsedIdentifier, CrocoError> {
        let next_token = self.peek_token(iter);

        match next_token {
            Separator(LeftParenthesis) => (),
            _ => {
                return Ok(ParsedIdentifier::new(
                    Box::new(VarNode::new(identifier.name, self.token_pos.clone())),
                    false,
                ))
            }
        }
        iter.next();
        let mut fn_args: Vec<Box<dyn AstNode>> = Vec::new();

        loop {
            // TODO: avoid hello()) function calls
            if let Separator(RightParenthesis) = self.peek_token(iter) {
                self.next_token(iter);
                break;
            }

            fn_args.push(self.parse_expr(iter)?);
            self.discard_newlines(iter);

            match self.next_token(iter) {
                Separator(Comma) => (),
                Separator(RightParenthesis) => break,
                _ => {
                    return Err(CrocoError::new(
                        &self.token_pos,
                        format!("unexpected token in {} function call", identifier.name),
                    ))
                }
            }
        }

        Ok(ParsedIdentifier::new(
            Box::new(FunctionCallNode::new(
                identifier.name,
                fn_args,
                self.token_pos.clone(),
            )),
            true,
        ))
    }

    /// Parses an expression using the shunting-yard algorithm.
    // https://brilliant.org/wiki/shunting-yard-algorithm
    // https://en.wikipedia.org/wiki/Shunting-yard_algorithm
    // https://www.klittlepage.com/2013/12/22/twelve-days-2013-shunting-yard-algorithm/
    fn parse_expr(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        // and an expression to finish.
        let mut stack: Vec<Token> = Vec::new(); // == operand stack
        let mut output: Vec<Box<dyn AstNode>> = Vec::new(); // == operator stack

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
                Operator(UnaryMinus) => 8,
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

        // sometimes minus can behave as an unary operator, e.g
        // let a = --6
        // let a = -(6*4)
        // so we need to keep track of the last token
        let mut last_token = Discard;
        let is_unary = |last_token: &Token| -> bool {
            match last_token {
                Operator(_) | Discard => {
                    true
                }
                _ => false,
            }
        };

        loop {
            // make sure that this token belongs to the expression
            match self.peek_token(iter) {
                // the right parenthesis is the end of a function
                Separator(RightParenthesis) => {
                    if !parenthesis_opened {
                        break;
                    }
                }

                // end of an expr
                Separator(NewLine) | Separator(Comma) | Separator(LeftBracket) | EOF => break,
                _ => (),
            }

            // now that we know that the token is right, we can consume it
            let mut expr_token = self.next_token(iter);
            let mut expr_token_clone = expr_token.clone();

            match expr_token {
                Identifier(identifier) => {
                    output.push(self.parse_identifier(iter, identifier)?.content)
                }
                Literal(_) => output.push(self.get_node(expr_token)?),
                Operator(_) => {
                    
                    // if we have an unary operator flag it accordingly
                    // https://github.com/MacTee/Shunting-Yard-Algorithm/blob/master/ShuntingYard/InfixToPostfixConverter.cs
                    match expr_token {
                        Operator(Minus) if is_unary(&last_token) => {
                            expr_token = Operator(UnaryMinus);
                            expr_token_clone = Operator(UnaryMinus);
                        }
                        Operator(Bang) if !is_unary(&last_token) => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                "misuse of the bang operator".to_owned(),
                            ))
                        }
                        // do nothing as "!" is always unary
                        Operator(Bang) => (),
                        _ if is_unary(&last_token) => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                "not a valid unary operator".to_owned(),
                            ))
                        }
                        _ => ()
                    }

                    while let Some(top) = stack.last() {
                        match top {
                            Operator(_) => {
                                if (!right_associative(&top)
                                    && get_precedence(&top) == get_precedence(&expr_token))
                                    || get_precedence(&top) > get_precedence(&expr_token)
                                {
                                    let op = stack.pop().unwrap();
                                    match op {
                                        Operator(_) => self.add_node(&mut output, op)?,
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

                    stack.push(expr_token);
                }
                Separator(LeftParenthesis) => {
                    stack.push(expr_token);
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
                                        self.add_node(&mut output, popped.unwrap())?
                                    }
                                    _ => {
                                        return Err(CrocoError::new(
                                            &self.token_pos,
                                            "missing parenthesis in expression".to_owned(),
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Err(CrocoError::new(
                        &self.token_pos,
                        "unexpected symbol in math expression".to_owned(),
                    ))
                }
            }
            last_token = expr_token_clone;
            // println!("stack: {:?}", stack);
            // println!("output: {:?}", output);
        }

        while let Some(popped) = stack.pop() {
            match popped {
                Separator(LeftParenthesis) => {
                    return Err(CrocoError::new(
                        &self.token_pos,
                        "missing parenthesis in expression".to_owned(),
                    ))
                }
                _ => self.add_node(&mut output, popped)?,
            }
        }

        if output.is_empty() {
            return Ok(Box::new(LiteralNode::new(LiteralEnum::Void)));
        }

        Ok(output.pop().unwrap())
    }

    /// Parses a code block e.g for loop body, function body, etc.
    /// warning: it consumes the closing right bracket but not the opening one
    fn parse_block(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        scope: BlockScope,
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        let mut block = BlockNode::new(scope);
        // loop until we have no token remaining
        loop {
            // a discard token means that we have no token left
            let token = self.next_token(iter);
            if let EOF = token {
                break;
            }

            match token {
                // ending the block
                Separator(RightBracket) => break,

                // declaring a new number variable
                Keyword(Let) => {
                    // rust is too dumb to figure out that out_node is always initalized so we
                    // have to wrap out_node in an Option
                    let mut out_node: Option<Box<dyn AstNode>> = None;

                    // we're expecting a variable name
                    let identifier = self.expect_identifier(
                        iter,
                        "expected a variable name after the let keyword",
                    )?;

                    let mut assign_type: LiteralEnum = LiteralEnum::Void;

                    match self.next_token(iter) {
                        // we're giving a value to our variable with type inference
                        Operator(Assign) => {
                            out_node = Some(self.parse_expr(iter)?);
                        }

                        // we're giving a type annotation
                        Keyword(Num) => assign_type = LiteralEnum::Num(None),
                        Keyword(Str) => assign_type = LiteralEnum::Str(None),
                        Keyword(Bool) => assign_type = LiteralEnum::Bool(None),

                        // newline: we're declaring a variable without value or type
                        // for now we're not able to infer the variable type.
                        Separator(NewLine) => {
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
                                out_node = Some(self.parse_expr(iter)?);
                            }
                            // we can infer the default value since we have the type annotation
                            Separator(NewLine) => {
                                out_node = match assign_type {
                                    LiteralEnum::Bool(_) => Some(
                                        self.get_node(Literal(LiteralEnum::Bool(Some(false))))?,
                                    ),
                                    LiteralEnum::Num(_) => {
                                        Some(self.get_node(Literal(LiteralEnum::Num(Some(0.))))?)
                                    }
                                    LiteralEnum::Str(_) => Some(self.get_node(Literal(
                                        LiteralEnum::Str(Some(String::new())),
                                    ))?),
                                    _ => {
                                        return Err(CrocoError::new(
                                            &self.token_pos,
                                            format!(
                                                "cannot infer default value for {}",
                                                identifier.name
                                            ),
                                        ))
                                    }
                                }
                            }
                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    format!("expected an equals sign after {}", identifier.name),
                                ))
                            }
                        }
                    }

                    // add this statement to the block
                    block.add_child(Box::new(DeclNode::new(
                        identifier.get_namespaced_name(),
                        out_node.unwrap(),
                        assign_type,
                        self.token_pos.clone(),
                    )));
                }

                // assigning a new value to a variable, or calling a function
                Identifier(identifier) => {
                    let parsed_identifier = self.parse_identifier(iter, identifier.clone())?;

                    // we're calling a function
                    if parsed_identifier.is_fn_call {
                        self.expect_token(
                            iter,
                            Separator(NewLine),
                            "expected a new line after function call",
                        )?;

                        block.add_child(parsed_identifier.content);
                        continue;
                    }

                    let op_token = self.next_token(iter);
                    match op_token {

                        // assigning to a variable
                        Operator(Assign)
                        | Operator(PlusEquals)
                        | Operator(MinusEquals)
                        | Operator(MultiplicateEquals)
                        | Operator(DivideEquals)
                        | Operator(PowerEquals) => {

                            let out_node = self.parse_expr(iter)?;
                            // add to the root function this statement
                            if op_token == Operator(Assign) {
                                block.add_child(Box::new(AssignmentNode::new(identifier.name, out_node, self.token_pos.clone())));
                            } else {
                                let mut dyn_op_node: Box<dyn AstNode> = match op_token {
                                    Operator(PlusEquals) => Box::new(PlusNode::new(self.token_pos.clone())),
                                    Operator(MinusEquals) => Box::new(MinusNode::new(self.token_pos.clone())),
                                    Operator(MultiplicateEquals) => {
                                        Box::new(MultiplicateNode::new(self.token_pos.clone()))
                                    }
                                    Operator(DivideEquals) => Box::new(DivideNode::new(self.token_pos.clone())),
                                    Operator(PowerEquals) => Box::new(PowerNode::new(self.token_pos.clone())),
                                    _ => unreachable!(),
                                };
                                let var_node = Box::new(VarNode::new(identifier.name.clone(), self.token_pos.clone()));
                                dyn_op_node.add_child(var_node);
                                dyn_op_node.add_child(out_node);

                                self.expect_token(iter, Separator(NewLine), "expected a new line after assignation")?;

                                block.add_child(Box::new(AssignmentNode::new(identifier.name, dyn_op_node, self.token_pos.clone())));
                            }
                        }
                        _ => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                format!("expected an assignation sign or a function call after the identifier {}", identifier.name)
                            ))
                        }
                    }
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
                    // continue to look for args while we haven't found a right parenthesis
                    let mut parenthesis_consumed = false;

                    loop {
                        let peek_token = self.peek_token(iter);
                        if peek_token == EOF || peek_token == Separator(RightParenthesis) {
                            break;
                        }
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
                            Keyword(Num) => LiteralEnum::Num(None),
                            Keyword(Str) => LiteralEnum::Str(None),
                            Keyword(Bool) => LiteralEnum::Bool(None),
                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    format!("expected an argument type for {}", arg_name.name),
                                ))
                            }
                        };
                        typed_args.push(TypedArg::new(arg_name.name, arg_type));

                        self.discard_newlines(iter);

                        match self.next_token(iter) {
                            Separator(Comma) => (),
                            Separator(RightParenthesis) => {
                                parenthesis_consumed = true;
                                break;
                            }
                            _ => {
                                return Err(CrocoError::new(
                                    &self.token_pos,
                                    format!(
                                    "expected a comma or a right parenthesis in {} function call",
                                    identifier.name),
                                ))
                            }
                        }
                    }

                    // consume the right parenthesis
                    if !parenthesis_consumed {
                        iter.next();
                    }

                    // TMight allow weird parsing: does it matter ?
                    // fn bla()
                    // Void
                    // { ...
                    self.discard_newlines(iter);

                    // if the return type isn't specified the function is Void
                    let mut return_type = LiteralEnum::Void;

                    match self.next_token(iter) {
                        Keyword(Num) => return_type = LiteralEnum::Num(None),
                        Keyword(Str) => return_type = LiteralEnum::Str(None),
                        Keyword(Bool) => return_type = LiteralEnum::Bool(None),
                        Separator(LeftBracket) => (),
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

                    if return_type != LiteralEnum::Void {
                        self.expect_token(
                            iter,
                            Separator(LeftBracket),
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
                    let return_node = self.parse_expr(iter)?;
                    // TODO: correct CodePos
                    block.add_child(Box::new(ReturnNode::new(
                        return_node,
                        self.token_pos.clone(),
                    )));
                }

                // if block
                Keyword(If) => {
                    let cond = self.parse_expr(iter)?;

                    self.expect_token(
                        iter,
                        Separator(LeftBracket),
                        "expected left bracket after if expression",
                    )?;

                    let body = self.parse_block(iter, BlockScope::New)?;
                    block.add_child(Box::new(IfNode::new(cond, body, self.token_pos.clone())))
                }

                // while loop
                Keyword(While) => {
                    let cond = self.parse_expr(iter)?;

                    self.expect_token(
                        iter,
                        Separator(LeftBracket),
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
                        self.expect_str(iter, "expected an str after the import keyword")?;
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

    /// Expects a token
    fn expect_token(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        token: Token,
        error_msg: &str,
    ) -> Result<(), CrocoError> {
        // The EOF token is behaving like a newline in expect
        let mut next_token = self.next_token(iter);
        if next_token == EOF {
            next_token = Separator(NewLine);
        }

        if next_token == token {
            Ok(())
        } else {
            Err(CrocoError::new(&self.token_pos, error_msg.to_owned()))
        }
    }

    /// Expects an identifier and returns its name
    fn expect_identifier(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        error_msg: &str,
    ) -> Result<token::Identifier, CrocoError> {
        match self.next_token(iter) {
            Identifier(identifier) => Ok(identifier),
            _ => Err(CrocoError::new(&self.token_pos, error_msg.to_owned())),
        }
    }

    /// Expects a literal string a returns its value
    fn expect_str(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        error_msg: &str,
    ) -> Result<String, CrocoError> {
        match self.next_token(iter) {
            Literal(LiteralEnum::Str(Some(s))) => Ok(s),
            _ => Err(CrocoError::new(&self.token_pos, error_msg.to_owned())),
        }
    }

    /// Discards all next tokens that are newlines
    fn discard_newlines(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
    ) {
        while let Separator(NewLine) = self.peek_token(iter) {
            self.next_token(iter);
        }
    }

    /// builds an Abstact Syntax Tree (AST) with the results of the lexer.
    pub fn process(
        &mut self,
        tokens: Vec<(Token, CodePos)>,
    ) -> Result<Box<dyn AstNode>, CrocoError> {
        // iterator which returns a movable and peekable token iterator
        let mut iter = tokens.into_iter().peekable();
        let root = self.parse_block(&mut iter, self.scope.clone())?;
        Ok(root)
    }
}
