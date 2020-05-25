use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralEnum {
    Bool(Option<bool>),
    Num(Option<f32>),
    Str(Option<String>),
    Void,
}

pub fn literal_eq(a: &LiteralEnum, b: &LiteralEnum) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

impl LiteralEnum {
    pub fn is_void(&self) -> bool {
        match self {
            LiteralEnum::Void => true,
            _ => false,
        }
    }

    pub fn is_num(&self) -> bool {
        match self {
            LiteralEnum::Num(_) => true,
            _ => false,
        }
    }

    pub fn into_bool(self) -> bool {
        match self {
            LiteralEnum::Bool(Some(b)) => b,
            _ => panic!("LiteralEnum is not a boolean !"),
        }
    }

    pub fn into_str(self) -> String {
        match self {
            LiteralEnum::Str(Some(t)) => t,
            _ => panic!("LiteralEnum is not a string !"),
        }
    }

    pub fn into_num(self) -> f32 {
        match self {
            LiteralEnum::Num(Some(n)) => n,
            _ => panic!("LiteralEnum is not a number !"),
        }
    }
}

impl PartialOrd for LiteralEnum {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            LiteralEnum::Num(n1) => {
                if let LiteralEnum::Num(n2) = other {
                    n1.partial_cmp(n2)
                } else {
                    unreachable!()
                }
            }

            _ => unreachable!(),
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
    Comma,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    // the name of the identifier, e.g foo
    pub name: String,

    // where the identifier came, e.g fs
    // this is used in the parser for functions imported from other packages:
    // you call them with fs.foo()
    pub namespace: String,
}

impl Identifier {
    pub fn new(name: String, namespace: String) -> Self {
        Identifier { name, namespace }
    }
    ///  returns the namespaced name of the identifer
    pub fn get_namespaced_name(self) -> String {
        if self.namespace.is_empty() {
            self.name
        } else {
            format!("{}.{}", self.namespace, self.name)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(LiteralEnum),
    Separator(SeparatorEnum),
    Operator(OperatorEnum),
    Identifier(Identifier),
    Keyword(KeywordEnum),
    Discard,
}
