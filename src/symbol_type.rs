use crate::parser::TypedArg;

/// the type of a symbol, which is backend-agnostic
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
    Num,
    Void,
}

impl SymbolType {
    pub fn is_void(&self) -> bool {
        match self {
            SymbolType::Void => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionType {
    pub args: Vec<TypedArg>,
    pub return_type: Box<SymbolType>,
}

/// compare if two symbols are of the same type
pub fn type_eq(a: &SymbolType, b: &SymbolType) -> bool {
    match (a, b) {
        (SymbolType::Bool, SymbolType::Bool)
        | (SymbolType::Str, SymbolType::Str)
        | (SymbolType::Num, SymbolType::Num)
        | (SymbolType::Void, SymbolType::Void) => true,
        (SymbolType::Struct(a), SymbolType::Struct(b)) => a == b,
        (SymbolType::Map(a, b), SymbolType::Map(c, d)) => type_eq(a, b) && type_eq(c, d),
        (SymbolType::Array(a), SymbolType::Array(b)) => type_eq(a, b),
        (SymbolType::Ref(a), SymbolType::Ref(b)) => type_eq(a, b),
        (SymbolType::CrocoType, SymbolType::CrocoType) => true,
        (SymbolType::Function(a), SymbolType::Function(b)) => {
            if a.args.len() != b.args.len() {
                return false;
            }

            if !type_eq(&*a.return_type, &*b.return_type) {
                return false;
            }

            // we should not find any argument from a with a different type of b
            a.args
                .iter()
                .zip(b.args.iter())
                .find(|(c, d)| !type_eq(&c.arg_type, &d.arg_type))
                .is_none()
        }
        _ => false,
    }
}
