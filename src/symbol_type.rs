use std::fmt;

/// The type of a symbol, which is backend-agnostic
#[derive(Clone, Debug)]
pub enum SymbolType {
    Ref(Box<SymbolType>),
    Array(Box<SymbolType>),
    Map(Box<SymbolType>, Box<SymbolType>),
    Struct(String),
    Function(FunctionType),
    CrocoType,
    // primitive types
    Bool,
    Str,
    Fnum,
    Num,
}

#[derive(Clone, Debug)]
pub struct FunctionType {
    pub args: Vec<SymbolType>,
    pub return_type: Box<SymbolType>,
}

impl SymbolType {
    /// Transforms a Ref(SymbolType) into a SymbolType
    pub fn deref(self) -> SymbolType {
        match self {
            SymbolType::Ref(s) => *s,
            _ => panic!("did not get a reference"),
        }
    }
}

impl PartialEq for SymbolType {
    /// compare if two symbols are of the same type
    fn eq(&self, other: &SymbolType) -> bool {
        match (self, other) {
            (SymbolType::Bool, SymbolType::Bool)
            | (SymbolType::Str, SymbolType::Str)
            | (SymbolType::Num, SymbolType::Num)
            | (SymbolType::Fnum, SymbolType::Fnum) => true,
            (SymbolType::Struct(a), SymbolType::Struct(b)) => a == b,
            (SymbolType::Map(a, b), SymbolType::Map(c, d)) => a == c && b == d,
            (SymbolType::Array(a), SymbolType::Array(b)) => a == b,
            (SymbolType::Ref(a), SymbolType::Ref(b)) => a == b,
            (SymbolType::Function(a), SymbolType::Function(b)) => {
                if a.args.len() != b.args.len() {
                    return false;
                }

                if a.return_type != b.return_type {
                    return false;
                }

                // we should not find any argument from a with a different type of b
                a.args
                    .iter()
                    .zip(b.args.iter())
                    .find(|(c, d)| c != d)
                    .is_none()
            }
            _ => false,
        }
    }
}

impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolType::Ref(r) => write!(f, "&{}", r),
            SymbolType::Array(a) => write!(f, "[{}]", a),
            SymbolType::Map(a, b) => write!(f, "{{{}, {}}}", a, b),
            SymbolType::Struct(s) => write!(f, "{}", s),
            SymbolType::Function(func) => {
                write!(f, "{} fn(", func.return_type)?;
                for i in 0..func.args.len() - 1 {
                    write!(f, "{}, ", func.args[i])?;
                }

                if let Some(last) = func.args.last() {
                    write!(f, "{})", last)
                } else {
                    write!(f, ")")
                }
            }
            SymbolType::CrocoType => write!(f, "type"),
            SymbolType::Bool => write!(f, "bool"),
            SymbolType::Str => write!(f, "str"),
            SymbolType::Fnum => write!(f, "fnum"),
            SymbolType::Num => write!(f, "num"),
        }
    }
}
