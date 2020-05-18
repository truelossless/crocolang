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
        match &self {
            LiteralEnum::Void => true,
            _ => false
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

#[derive(Debug, PartialEq)]
pub enum OperatorEnum {
    Assign,
    PlusEquals,
    MinusEquals,
    MultiplicateEquals,
    DivideEquals,
    PowerEquals,

    Equals,
    NotEqualsTo,
    GreaterThan,
    GreaterOrEqualTo,
    LowerThan,
    LowerOrEqualTo,

    Plus,
    Minus,
    Multiplicate,
    Divide,
    Power,
}

#[derive(Debug, PartialEq)]
pub enum KeywordEnum {
    If,
    Elif,
    Else,
    Return,
    While,
    For,
    Match,
    Num,
    Str,
    Bool,
    Let,
    Struct,
    Test,
    Function,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(LiteralEnum),
    Separator(SeparatorEnum),
    Operator(OperatorEnum),
    Identifier(String),
    Keyword(KeywordEnum),
    Comment(String),
    Discard,
}
