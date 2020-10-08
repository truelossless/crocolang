#[cfg(feature = "crocoi")]
use crate::crocoi::{INodeResult, ISymTable, ISymbol};

#[cfg(feature = "crocol")]
use {
    crate::crocol::{utils::set_str_text, Codegen, LNodeResult, LSymbol},
    crate::symbol_type::SymbolType,
};

use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::token::{CodePos, LiteralEnum};

/// a node holding a literal value
#[derive(Clone)]
pub struct ConstantNode {
    value: LiteralEnum,
    code_pos: CodePos,
}

impl ConstantNode {
    pub fn new(value: LiteralEnum, code_pos: CodePos) -> Self {
        ConstantNode { value, code_pos }
    }
}

impl AstNode for ConstantNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, _symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::Value(ISymbol::Primitive(self.value.clone())))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let constant_symbol = match &self.value {
            LiteralEnum::Bool(b) => LSymbol {
                value: codegen
                    .context
                    .bool_type()
                    .const_int(*b as u64, false)
                    .into(),
                symbol_type: SymbolType::Bool,
            },

            LiteralEnum::Num(n) => LSymbol {
                value: codegen.context.f32_type().const_float(*n as f64).into(),
                symbol_type: SymbolType::Num,
            },

            // TODO: wacky. we need to initialize right away our string because we then loose the information about
            // the text content.
            // Maybe introduce a strconst type for compiled backends ?
            LiteralEnum::Str(s) => {
                let alloca =
                    codegen.create_entry_block_alloca(codegen.str_type.into(), "allocastr");
                set_str_text(alloca, &s, codegen);

                LSymbol {
                    value: alloca.into(),
                    symbol_type: SymbolType::Str,
                }
            }
        };

        Ok(LNodeResult::Value(constant_symbol))
    }
}
