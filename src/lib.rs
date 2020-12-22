#[cfg(feature = "crocol")]
mod crocol;

#[cfg(feature = "crocol")]
pub use self::crocol::Crocol;

#[cfg(feature = "crocol")]
mod linker;

#[cfg(all(windows, feature = "crocol"))]
mod ms_craziness_bindings;

#[cfg(feature = "crocoi")]
mod crocoi;

#[cfg(feature = "crocoi")]
pub use self::crocoi::Crocoi;

pub use self::error::CrocoError;

mod ast;
mod error;
mod lexer;
mod parser;
mod symbol;
mod symbol_type;
mod token;
