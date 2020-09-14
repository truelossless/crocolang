extern crate unicode_segmentation;

use std::{iter::Peekable, rc::Rc};
use unicode_segmentation::{UWordBounds, UnicodeSegmentation};

use crate::token::Identifier;
use crate::token::{
    CodePos, KeywordEnum::*, LiteralEnum, OperatorEnum::*, SeparatorEnum::*, Token, Token::*,
};

use crate::error::CrocoError;

fn is_number(el: &str) -> Option<f32> {
    el.parse().ok()
}

pub struct Lexer {
    namespace: String,

    // code location
    file: Rc<str>,
    line_index: u32,
    word_index: u16,
    new_line: bool,

    queue: Vec<Token>,
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            namespace: String::new(),
            file: Rc::from(""),
            queue: Vec::new(),
            new_line: false,
            line_index: 0,
            word_index: 0,
        }
    }

    pub fn set_namespace(&mut self, namespace: String) {
        self.namespace = namespace;
    }

    pub fn set_file(&mut self, file: &str) {
        self.file = Rc::from(file);
    }

    // TODO: remove complexity and indentation
    /// consume one loop of the iter and push to the queue the correspoonding tokens
    fn process_token(&mut self, iter: &mut Peekable<UWordBounds>) -> Result<bool, CrocoError> {
        // get the next iterator value
        let el_opt = iter.next();

        // we've reached the end of the file
        if el_opt.is_none() {
            return Ok(false);
        }

        let el = el_opt.unwrap();

        // check if it's a number
        let num = is_number(&el);

        // tokenize
        match el {
            // number literal
            _ if num != None => self.queue.push(Literal(LiteralEnum::Num(num.unwrap()))),

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
                        return Err(CrocoError::new(
                            &CodePos {
                                file: self.file.clone(),
                                line: self.line_index,
                                word: self.word_index,
                            },
                            "unclosed quotes",
                        ));
                    }
                }
                self.queue
                    .push(Literal(LiteralEnum::Str(words_in_quotes.join(""))));
            }

            // boolean literal
            "true" => self.queue.push(Literal(LiteralEnum::Bool(true))),
            "false" => self.queue.push(Literal(LiteralEnum::Bool(false))),

            // separators
            "(" => self.queue.push(Separator(LeftParenthesis)),
            ")" => self.queue.push(Separator(RightParenthesis)),
            "{" => self.queue.push(Separator(LeftCurlyBracket)),
            "}" => self.queue.push(Separator(RightCurlyBracket)),
            "[" => self.queue.push(Separator(LeftSquareBracket)),
            "]" => self.queue.push(Separator(RightSquareBracket)),
            "," => self.queue.push(Separator(Comma)),
            "." => self.queue.push(Separator(Dot)),
            ":" => self.queue.push(Separator(Colon)),
            "\r\n" | "\n" => {
                self.new_line = true;
                self.queue.push(Separator(NewLine))
            }
            ";" => self.queue.push(Separator(Semicolon)),
            // operators

            // as can be viewed as a keyword but it behaves as an unary operator
            "as" => self.queue.push(Operator(As)),
            "=" => {
                let mut ret = Operator(Assign);

                if let Some(x) = iter.peek() {
                    if x == &"=" {
                        iter.next();
                        ret = Operator(Equals);
                    }
                }

                self.queue.push(ret)
            }

            "+" => {
                let mut ret = Operator(Plus);

                if let Some(x) = iter.peek() {
                    if x == &"=" {
                        iter.next();
                        ret = Operator(PlusEquals);
                    }
                }

                self.queue.push(ret)
            }

            "-" => {
                let mut ret = Operator(Minus);

                let next = iter.peek();

                if let Some(x) = next {
                    if x == &"=" {
                        iter.next();
                        ret = Operator(MinusEquals);
                    }
                }

                self.queue.push(ret)
            }

            "*" => {
                let mut ret = Operator(Multiplicate);

                if let Some(x) = iter.peek() {
                    if x == &"=" {
                        iter.next();
                        ret = Operator(MultiplicateEquals);
                    }
                }

                self.queue.push(ret)
            }

            "/" => {
                match iter.peek() {
                    Some(&"=") => {
                        iter.next();
                        self.queue.push(Operator(DivideEquals));
                    }
                    // this is a comment, discard until next line
                    Some(&"/") => {
                        iter.next();
                        loop {
                            let next = iter.next();
                            if next.is_none() || next == Some("\n") || next == Some("\r\n") {
                                self.new_line = true;
                                break;
                            }
                        }
                    }
                    _ => self.queue.push(Operator(Divide)),
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

                self.queue.push(ret)
            }

            ">" => {
                let mut ret = Operator(GreaterThan);

                if let Some(x) = iter.peek() {
                    if x == &"=" {
                        iter.next();
                        ret = Operator(GreaterOrEqual);
                    }
                }

                self.queue.push(ret)
            }

            "<" => {
                let mut ret = Operator(LowerThan);

                if let Some(x) = iter.peek() {
                    if x == &"=" {
                        iter.next();
                        ret = Operator(LowerOrEqual);
                    }
                }

                self.queue.push(ret)
            }

            "&" => {
                let mut ret = Operator(BitwiseAnd);

                if let Some(x) = iter.peek() {
                    if x == &"&" {
                        iter.next();
                        ret = Operator(And);
                    }
                }

                self.queue.push(ret)
            }

            "|" => {
                let mut ret = Operator(BitwiseOr);

                if let Some(x) = iter.peek() {
                    if x == &"|" {
                        iter.next();
                        ret = Operator(Or);
                    }
                }

                self.queue.push(ret)
            }

            "!" => {
                let mut ret = Operator(Bang);

                if let Some(x) = iter.peek() {
                    if x == &"=" {
                        iter.next();
                        ret = Operator(NotEquals);
                    }
                }

                self.queue.push(ret)
            }

            // keywords
            "bool" => self.queue.push(Keyword(Bool)),
            "break" => self.queue.push(Keyword(Break)),
            "continue" => self.queue.push(Keyword(Continue)),
            "elif" => self.queue.push(Keyword(Elif)),
            "else" => self.queue.push(Keyword(Else)),
            "fn" => self.queue.push(Keyword(Function)),
            "for" => self.queue.push(Keyword(For)),
            "if" => self.queue.push(Keyword(If)),
            "let" => self.queue.push(Keyword(Let)),
            "match" => self.queue.push(Keyword(Match)),
            "num" => self.queue.push(Keyword(Num)),
            "while" => self.queue.push(Keyword(While)),
            "str" => self.queue.push(Keyword(Str)),
            "struct" => self.queue.push(Keyword(Struct)),
            "test" => self.queue.push(Keyword(Test)),
            "return" => self.queue.push(Keyword(Return)),
            "import" => self.queue.push(Keyword(Import)),

            // ignore whitespaces
            _ if el.trim().is_empty() => (),

            // identifiers
            _ if char::is_alphabetic(el.chars().next().unwrap()) => {
                // we have an identifier here, but it might be made of dots
                // e.g point.x is parsed as an unique identifier by unicode_segmentation
                // so we must split this into multiple tokens
                let split: Vec<_> = el.split('.').collect();
                let split_len = split.len();

                for (i, identifier) in split.iter().rev().enumerate() {
                    self.queue.push(Identifier(Identifier::new(
                        identifier.to_owned().to_owned(),
                        self.namespace.clone(),
                    )));

                    // i haven't found any idiomatic way of iterating but excluding the last element ...
                    if i != split_len - 1 {
                        self.queue.push(Separator(Dot));
                    }
                }
            }

            // error out on other characters
            _ => {
                return Err(CrocoError::new(
                    &CodePos {
                        file: self.file.clone(),
                        line: self.line_index,
                        word: self.word_index,
                    },
                    &format!("unknown character: \"{}\"", el),
                ));
            }
        };

        Ok(true)
    }

    // returns an array of tokens
    pub fn process(&mut self, code: &str) -> Result<Vec<(Token, CodePos)>, CrocoError> {
        let mut iter = code.split_word_bounds().peekable();

        let mut tokens: Vec<(Token, CodePos)> = Vec::new();

        // loop until we reach the end of the file
        while self.process_token(&mut iter)? {
            // a single process token call can add multiple tokens to the queue
            while let Some(token) = self.queue.pop() {
                tokens.push((
                    token,
                    CodePos {
                        file: self.file.clone(),
                        word: self.word_index,
                        line: self.line_index,
                    },
                ));
            }

            if self.new_line {
                self.line_index += 1;
                self.word_index = 0;
                self.new_line = false;
            } else {
                self.word_index += 1;
            }
        }

        Ok(tokens)
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}
