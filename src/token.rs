use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralEnum {
    Boolean(Option<bool>),
    Number(Option<f32>),
    Text(Option<String>),
    Void,
}

pub fn literal_eq(a: &LiteralEnum, b: &LiteralEnum) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

impl LiteralEnum {
    pub fn is_void(&self) -> bool {
        match self {
            LiteralEnum::Void => true,
            _ => false
        }
    }

    pub fn is_num(&self) -> bool {
        match self {
            LiteralEnum::Number(_) => true,
            _ => false
        }
    }

    pub fn into_bool(self) -> bool {
        match self {
            LiteralEnum::Boolean(Some(b)) => b,
            _ => panic!("LiteralEnum is not a boolean !")
        }
    }
}

impl PartialOrd for LiteralEnum {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {

        match self {
            
            LiteralEnum::Number(n1) => {

                if let LiteralEnum::Number(n2) = other {
                    n1.partial_cmp(n2)
                } else {
                    unreachable!()
                }
            }

            _ => unreachable!()
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum SeparatorEnum {
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    Semicolon,
    NewLine,
    Comma
}

#[derive(Clone, Debug, PartialEq)]
pub enum OperatorEnum {
    Assign,
    PlusEquals,
    MinusEquals,
    MultiplicateEquals,
    DivideEquals,
    PowerEquals,

    Or,
    And,
    BitwiseOr,
    BitwiseAnd,
    Equals,
    NotEquals,
    GreaterThan,
    GreaterOrEqual,
    LowerThan,
    LowerOrEqual,

    Plus,
    Minus,
    Multiplicate,
    Divide,
    Power,
}

#[derive(Debug, PartialEq)]
pub enum KeywordEnum {
    Bool,
    Break,
    Continue,
    Elif,
    Else,
    For,
    Function,
    If,
    Import,
    Let,
    Match,
    Num,
    Return,
    Str,
    While,
    Struct,
    Test,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(LiteralEnum),
    Separator(SeparatorEnum),
    Operator(OperatorEnum),
    Identifier(String),
    Keyword(KeywordEnum),
    Discard,
}
