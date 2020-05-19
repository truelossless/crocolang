extern crate unicode_segmentation;

use crate::token::{
    KeywordEnum::*, LiteralEnum::*, OperatorEnum::*, SeparatorEnum::*, Token, Token::*,
};

use unicode_segmentation::UnicodeSegmentation;

fn starts_ascii(el: &str) -> bool {
    ('a'..'z').contains(&el.chars().next().unwrap_or('0'))
}

fn is_number(el: &str) -> Option<f32> {
    el.parse().ok()
}

#[derive(Default)]
pub struct Lexer {}

impl Lexer {
    pub fn new() -> Self {
        Lexer {}
    }

    // TODO: loop through a get_token() to remove complexity and indentation
    /// returns an array of tokens
    pub fn process(&self, code: &str) -> Result<Vec<Token>, String> {
        // let words: Vec<&str> = code.split_word_bounds().collect();
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
                _ if num != None => Literal(Number(num)),

                // string literal
                "\"" => {
                    let mut words_in_quotes: Vec<&str> = Vec::new();

                    loop {
                        if let Some(el) = iter.next() {
                            // escape quotes
                            match el {
                                "\\" => {
                                    if iter.peek() == Some(&"\"") {
                                        iter.next();
                                        words_in_quotes.push("\"");
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
                            return Err("unclosed quotes".to_string());
                        }
                    }

                    Literal(Text(Some(words_in_quotes.join(""))))
                }

                // boolean literal
                "true" => Literal(Boolean(Some(true))),
                "false" => Literal(Boolean(Some(false))),

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
                                ret = Literal(Number(Some(-y)));
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
                            continue;
                        }
                        _ => {
                            Operator(Divide)
                        }
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

                // keywords
                "bool" => Keyword(Bool),
                "elif" => Keyword(Elif),
                "else" => Keyword(Else),
                "fn" => Keyword(Function),
                "for" => Keyword(For),
                "if" => Keyword(If),
                "import" => Keyword(Import),
                "let" => Keyword(Let),
                "match" => Keyword(Match),
                "num" => Keyword(Num),
                "while" => Keyword(While),
                "str" => Keyword(Str),
                "struct" => Keyword(Struct),
                "test" => Keyword(Test),
                "return" => Keyword(Return),

                // variables
                _ if starts_ascii(el) => Identifier(el.to_string()),

                // ignore whitespaces
                " " => Discard,
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
