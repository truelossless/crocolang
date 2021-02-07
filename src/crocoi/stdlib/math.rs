use crate::crocoi::stdlib::{BuiltinModule, BuiltinVar};
use crate::crocoi::symbol::ISymbol::*;
use crate::token::LiteralEnum::*;

// module definition
pub fn get_module() -> BuiltinModule {
    let functions = Vec::new();

    let vars = vec![
        BuiltinVar {
            name: "pi".to_owned(),
            value: Primitive(Fnum(std::f32::consts::PI)),
        },
        BuiltinVar {
            name: "e".to_owned(),
            value: Primitive(Fnum(std::f32::consts::E)),
        },
    ];

    BuiltinModule { functions, vars }
}
