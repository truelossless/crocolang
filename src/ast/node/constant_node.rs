#[cfg(feature = "crocoi")]
use crate::crocoi::{symbol::SymbolContent, INodeResult, ISymbol};

#[cfg(feature = "crocol")]
use {
    crate::crocol::{utils::set_str_text, Codegen, LNodeResult},
    inkwell::values::AnyValueEnum,
};

use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::symbol::SymTable;
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
    fn crocoi(&mut self, _symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            self.value.clone(),
        )))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(&mut self, codegen: &Codegen<'ctx>) -> Result<LNodeResult<'ctx>, CrocoError> {
        let llvm_value: AnyValueEnum = match &self.value {
            LiteralEnum::Bool(b) => codegen
                .context
                .bool_type()
                .const_int(*b as u64, false)
                .into(),
            LiteralEnum::Num(n) => codegen.context.f32_type().const_float(*n as f64).into(),
            LiteralEnum::Str(s) => {
                let alloca =
                    codegen.create_entry_block_alloca(codegen.str_type.into(), "str alloc");
                set_str_text(alloca, &s, codegen);

                alloca.into()
            }
            _ => unreachable!(),
        };

        Ok(LNodeResult::Symbol(llvm_value))
    }
}
