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
}

#[derive(Clone, Debug)]
pub struct FunctionType {
    pub args: Vec<TypedArg>,
    pub return_type: Box<SymbolType>,
}

impl SymbolType {
    /// compare if two symbols are of the same type
    pub fn eq(&self, b: &SymbolType) -> bool {
        match (self, b) {
            (SymbolType::Bool, SymbolType::Bool)
            | (SymbolType::Str, SymbolType::Str)
            | (SymbolType::Num, SymbolType::Num) => true,
            (SymbolType::Struct(a), SymbolType::Struct(b)) => a == b,
            (SymbolType::Map(a, b), SymbolType::Map(c, d)) => a.eq(b) && c.eq(d),
            (SymbolType::Array(a), SymbolType::Array(b)) => a.eq(b),
            (SymbolType::Ref(a), SymbolType::Ref(b)) => a.eq(b),
            (SymbolType::Function(a), SymbolType::Function(b)) => {
                if a.args.len() != b.args.len() {
                    return false;
                }

                if !a.return_type.eq(&*b.return_type) {
                    return false;
                }

                // we should not find any argument from a with a different type of b
                a.args
                    .iter()
                    .zip(b.args.iter())
                    .find(|(c, d)| !c.arg_type.eq(&d.arg_type))
                    .is_none()
            }
            _ => false,
        }
    }

    /// Transforms a Ref(SymbolType) into a SymbolType
    pub fn deref(self) -> SymbolType {
        match self {
            SymbolType::Ref(s) => *s,
            _ => panic!("did not get a reference"),
        }
    }
}
