extern crate unicode_segmentation;

use crate::token::Identifier;
use crate::token::{
    KeywordEnum::*, LiteralEnum, OperatorEnum::*, SeparatorEnum::*, Token, Token::*,
};

use unicode_segmentation::UnicodeSegmentation;

fn starts_ascii(el: &str) -> bool {
    ('a'..'z').contains(&el.chars().next().unwrap_or('0'))
}

fn is_number(el: &str) -> Option<f32> {
    el.parse().ok()
}

#[derive(Default)]
pub struct Lexer {
    namespace: String
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            namespace: String::new()
        }
    }

    pub fn set_namespace(&mut self, namespace: String) {
        self.namespace = namespace;
    }

    // TODO: loop through a get_token() to remove complexity and indentation
    /// returns an array of tokens
    pub fn process(&mut self, code: &str) -> Result<Vec<Token>, String> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut iter = code.split_word_bounds().peekable();

        // iterate trough all the words
        while let Some(el) = iter.next() {
            let token: Token;

            // check if it's a number
            let num = is_number(&el);

            // tokenize
            token = match el {
                // number literal
                _ if num != None => Literal(LiteralEnum::Num(num)),

                // string literal
                "\"" => {
                    let mut words_in_quotes: Vec<&str> = Vec::new();

                    loop {
                        if let Some(el) = iter.next() {
                            // escape quotes
                            match el {
                                "\\" => {
                                    if iter.peek() == Some(&"\"") {
                                        words_in_quotes.push("\"");
                                        iter.next();
                                    }
                                }

                                "\"" => {
                                    break;
                                }

                                _ => {
                                    words_in_quotes.push(el);
                                }
                            }
                        } else {
                            return Err("unclosed quotes".to_owned());
                        }
                    }
                    Literal(LiteralEnum::Str(Some(words_in_quotes.join(""))))
                }

                // boolean literal
                "true" => Literal(LiteralEnum::Bool(Some(true))),
                "false" => Literal(LiteralEnum::Bool(Some(false))),

                // separators
                "(" => Separator(LeftParenthesis),
                ")" => Separator(RightParenthesis),
                "{" => Separator(LeftBracket),
                "}" => Separator(RightBracket),
                "," => Separator(Comma),
                "\r\n" => Separator(NewLine), // windows line endings
                "\n" => Separator(NewLine),   // unix line endings
                ";" => Separator(Semicolon),
                // operators
                "=" => {
                    let mut ret = Operator(Assign);

                    if let Some(x) = iter.peek() {
                        if x == &"=" {
                            iter.next();
                            ret = Operator(Equals);
                        }
                    }

                    ret
                }

                "+" => {
                    let mut ret = Operator(Plus);

                    if let Some(x) = iter.peek() {
                        if x == &"=" {
                            iter.next();
                            ret = Operator(PlusEquals);
                        }
                    }

                    ret
                }

                "-" => {
                    let mut ret = Operator(Minus);

                    let next = iter.peek();

                    if let Some(x) = next {
                        if x == &"=" {
                            iter.next();
                            ret = Operator(MinusEquals);
                        } else {
                            let num = is_number(&x);
                            if let Some(y) = num {
                                iter.next();
                                ret = Literal(LiteralEnum::Num(Some(-y)));
                            }
                        }
                    }

                    ret
                }

                "*" => {
                    let mut ret = Operator(Multiplicate);

                    if let Some(x) = iter.peek() {
                        if x == &"=" {
                            iter.next();
                            ret = Operator(MultiplicateEquals);
                        }
                    }

                    ret
                }

                "/" => {
                    match iter.peek() {
                        Some(&"=") => {
                            iter.next();
                            Operator(DivideEquals)
                        }
                        // this is a comment, discard until next line
                        Some(&"/") => {
                            iter.next();
                            loop {
                                let next = iter.next();
                                if next.is_none() || next == Some("\n") || next == Some("\r\n") {
                                    break;
                                }
                            }
                            Discard
                        }
                        _ => Operator(Divide),
                    }
                }

                "^" => {
                    let mut ret = Operator(Power);

                    if let Some(x) = iter.peek() {
                        if x == &"=" {
                            iter.next();
                            ret = Operator(PowerEquals);
                        }
                    }

                    ret
                }

                ">" => {
                    let mut ret = Operator(GreaterThan);

                    if let Some(x) = iter.peek() {
                        if x == &"=" {
                            iter.next();
                            ret = Operator(GreaterOrEqual);
                        }
                    }

                    ret
                }

                "<" => {
                    let mut ret = Operator(LowerThan);

                    if let Some(x) = iter.peek() {
                        if x == &"=" {
                            iter.next();
                            ret = Operator(LowerOrEqual);
                        }
                    }

                    ret
                }

                "&" => {
                    let mut ret = Operator(BitwiseAnd);

                    if let Some(x) = iter.peek() {
                        if x == &"&" {
                            iter.next();
                            ret = Operator(And);
                        }
                    }

                    ret
                }

                "|" => {
                    let mut ret = Operator(BitwiseOr);

                    if let Some(x) = iter.peek() {
                        if x == &"|" {
                            iter.next();
                            ret = Operator(Or);
                        }
                    }

                    ret
                }

                // keywords
                "bool" => Keyword(Bool),
                "break" => Keyword(Break),
                "continue" => Keyword(Continue),
                "elif" => Keyword(Elif),
                "else" => Keyword(Else),
                "fn" => Keyword(Function),
                "for" => Keyword(For),
                "if" => Keyword(If),
                "let" => Keyword(Let),
                "match" => Keyword(Match),
                "num" => Keyword(Num),
                "while" => Keyword(While),
                "str" => Keyword(Str),
                "struct" => Keyword(Struct),
                "test" => Keyword(Test),
                "return" => Keyword(Return),

                // imports are resolved directly by the lexer.
                // the imported file is then also processed by the lexer
                // and the tokens are prepended to the one contained in
                // the main file.
                "import" => {
                    Keyword(Import)
                }

                // variables
                _ if starts_ascii(el) => Identifier(Identifier::new(el.to_owned(), self.namespace.clone())),

                // ignore whitespaces
                " " => Discard,
                "\t" => Discard,
                // _ => Discard
                _ => return Err(format!("unrecognized symbol: \"{}\"", el)),
            };

            if token != Discard {
                tokens.push(token);
            }
        }

        Ok(tokens)
    }
}
