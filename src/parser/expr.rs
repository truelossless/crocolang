use super::{ExprParsingType, Parser};

use crate::ast::{node::*, BackendNode};
use crate::error::CrocoError;
use crate::token::{CodePos, KeywordEnum::*, OperatorEnum::*, SeparatorEnum::*, Token, Token::*};

impl Parser {
    /// Parses an expression using the shunting-yard algorithm.
    // https://brilliant.org/wiki/shunting-yard-algorithm
    // https://en.wikipedia.org/wiki/Shunting-yard_algorithm
    // https://www.klittlepage.com/2013/12/22/twelve-days-2013-shunting-yard-algorithm/
    pub fn parse_expr(
        &mut self,
        iter: &mut std::iter::Peekable<std::vec::IntoIter<(Token, CodePos)>>,
        parse_type: ExprParsingType,
    ) -> Result<Box<dyn BackendNode>, CrocoError> {
        // and an expression to finish.
        let mut stack: Vec<Token> = Vec::new(); // == operand stack
        let mut output: Vec<Box<dyn BackendNode>> = Vec::new(); // == operator stack

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
                Operator(UnaryMinus) => 7,
                Operator(Power) => 8,
                _ => unreachable!(),
            }
        };

        // util to know if an operator can be right-associative
        // e.g 3+4 == 4+3
        // but 3^4 != 4^3
        let right_associative = |op: &Token| -> bool {
            !matches!(
                op,
                Operator(Divide)
                    | Operator(Minus)
                    | Operator(Power)
                    | Operator(GreaterOrEqual)
                    | Operator(GreaterThan)
                    | Operator(LowerOrEqual)
                    | Operator(LowerThan)
            )
        };

        // if we encouter a right parenthesis while there's no left parenthesis opened in our expression that
        // means that we're in this situation :
        // call_my_fn(3 + 4) <- this right parenthesis is the end of the function
        let mut parenthesis_opened = 0;

        // sometimes minus can behave as an unary operator, e.g
        // let a = --6
        // let a = -(6*4)
        let mut is_unary = true;

        loop {
            let mut expr_token = self.peek_token(iter);
            // make sure that this token belongs to the expression
            // println!("{:?}", self.peek_token(iter));
            match expr_token {
                // the right parenthesis is the end of a function
                Separator(RightParenthesis) if parenthesis_opened == 0 => break,

                // end of an expr
                Separator(NewLine)
                | Separator(Comma)
                | Separator(LeftCurlyBracket)
                | EOF
                | Separator(RightSquareBracket) => break,
                _ => (),
            }

            let mut is_next_token_unary =
                matches!(expr_token, Operator(_) | Separator(LeftParenthesis));

            match expr_token {
                Identifier(_) | Literal(_) | Separator(LeftSquareBracket) => {
                    output.push(self.parse_identifier(iter, parse_type)?);
                }

                Operator(BitwiseAnd) | Operator(Multiplicate) if is_unary => {
                    output.push(self.parse_identifier(iter, parse_type)?);
                    is_next_token_unary = false;
                }

                Keyword(Num) | Keyword(Str) | Keyword(Bool) => {
                    self.next_token(iter);
                    output.push(self.get_node(expr_token)?)
                }

                Operator(_) => {
                    self.next_token(iter);
                    // if we have an unary operator flag it accordingly
                    // https://github.com/MacTee/Shunting-Yard-Algorithm/blob/master/ShuntingYard/InfixToPostfixConverter.cs
                    match expr_token {
                        Operator(Minus) if is_unary => {
                            expr_token = Operator(UnaryMinus);
                        }
                        Operator(Bang) if !is_unary => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                "misuse of the bang operator",
                            ))
                        }
                        // do nothing as "!" is always unary
                        Operator(Bang) => (),
                        _ if is_unary => {
                            return Err(CrocoError::new(
                                &self.token_pos,
                                "not a valid unary operator",
                            ));
                        }
                        _ => (),
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
                    self.next_token(iter);
                    stack.push(expr_token);
                    parenthesis_opened += 1;
                }
                Separator(RightParenthesis) => {
                    self.next_token(iter);
                    parenthesis_opened -= 1;

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
                                            "missing parenthesis in expression",
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
                        "unexpected token in expression",
                    ))
                }
            }

            is_unary = is_next_token_unary;
        }

        while let Some(popped) = stack.pop() {
            match popped {
                Separator(LeftParenthesis) => {
                    return Err(CrocoError::new(
                        &self.token_pos,
                        "missing parenthesis in expression",
                    ))
                }
                _ => self.add_node(&mut output, popped)?,
            }
        }

        if output.is_empty() {
            return Ok(Box::new(VoidNode::new()));
        }

        Ok(output.pop().unwrap())
    }
}
