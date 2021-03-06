use crate::crocoi::CrocoiNode;
use crate::{ast::node::AsNode, error::CrocoError};
use crate::{symbol_type::SymbolType, token::LiteralEnum::*};

use crate::crocoi::{symbol::INodeResult, symbol::ISymbol, ICodegen};

impl CrocoiNode for AsNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let val = self
            .bottom
            .as_mut()
            .unwrap()
            .crocoi(codegen)?
            .into_symbol(&self.code_pos)?;

        // we can only cast primitive together
        let val_primitive = val
            .into_primitive()
            .map_err(|_| CrocoError::cast_non_primitive_error(&self.code_pos))?;

        let casted = match (val_primitive, &self.as_type) {
            // useless cast
            (Bool(_), SymbolType::Bool)
            | (Str(_), SymbolType::Str)
            | (Num(_), SymbolType::Num)
            | (Fnum(_), SymbolType::Fnum) => {
                return Err(CrocoError::cast_redundant_error(&self.code_pos))
            }
            (Bool(b), SymbolType::Fnum) => {
                if b {
                    Fnum(1.)
                } else {
                    Fnum(0.)
                }
            }

            (Bool(b), SymbolType::Num) => {
                if b {
                    Num(1)
                } else {
                    Num(0)
                }
            }

            (Bool(b), SymbolType::Str) => {
                if b {
                    Str("true".to_owned())
                } else {
                    Str("false".to_owned())
                }
            }

            (Num(n), SymbolType::Bool) => {
                if n == 0 {
                    Bool(false)
                } else {
                    Bool(true)
                }
            }

            (Num(n), SymbolType::Str) => Str(n.to_string()),

            (Num(n), SymbolType::Fnum) => Fnum(n as f32),

            (Fnum(n), SymbolType::Bool) => {
                if n == 0. {
                    Bool(false)
                } else {
                    Bool(true)
                }
            }
            (Fnum(n), SymbolType::Str) => Str(n.to_string()),

            (Fnum(n), SymbolType::Num) => Num(n as i32),

            (Str(s), SymbolType::Fnum) => {
                let n = s.parse().map_err(|_| {
                    CrocoError::new(&self.code_pos, "could not parse the str into a fnum")
                })?;
                Fnum(n)
            }
            (Str(s), SymbolType::Bool) => {
                if s.is_empty() {
                    Bool(false)
                } else {
                    Bool(true)
                }
            }

            (Str(s), SymbolType::Num) => {
                let n = s.parse().map_err(|_| {
                    CrocoError::new(&self.code_pos, "could not parse the str into a num")
                })?;
                Num(n)
            }

            _ => {
                return Err(CrocoError::cast_non_primitive_error(&self.code_pos));
            }
        };

        Ok(INodeResult::Value(ISymbol::Primitive(casted)))
    }
}
