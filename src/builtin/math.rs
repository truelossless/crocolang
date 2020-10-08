use crate::builtin::{BuiltinModule, BuiltinVar};
use crate::crocoi::symbol::ISymbol::*;
use crate::token::LiteralEnum::*;

// module definition
#[allow(clippy::unreadable_literal)]
pub fn get_module() -> BuiltinModule {
    let functions = Vec::new();

    let vars = vec![
        BuiltinVar {
            name: "pi".to_owned(),
            value: Primitive(Num(std::f32::consts::PI)),
        },
        BuiltinVar {
            name: "e".to_owned(),
            value: Primitive(Num(std::f32::consts::E)),
        },
    ];

    BuiltinModule { functions, vars }
}
