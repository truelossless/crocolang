use crate::ast::{AstNode, AstNodeType, INodeResult};
use crate::error::CrocoError;
use crate::token::CodePos;

#[cfg(feature = "crocoi")]
use crate::crocoi::symbol::{ISymTable, ISymbol};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult, LSymbol};

/// a node dereferencing a symbol reference in the symtable to a value
#[derive(Clone)]
pub struct DerefNode {
    symbol: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl DerefNode {
    pub fn new(code_pos: CodePos) -> Self {
        DerefNode {
            symbol: None,
            code_pos,
        }
    }
}

impl AstNode for DerefNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let symbol = self
            .symbol
            .as_mut()
            .unwrap()
            .crocoi(symtable)?
            .into_var(&self.code_pos)?;

        let deref_symbol = match symbol.borrow().clone() {
            ISymbol::Ref(r) => r,
            _ => {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "cannot dereference this variable",
                ))
            }
        };

        Ok(INodeResult::Variable(deref_symbol))
    }
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.symbol.is_none() {
            self.symbol = Some(node);
        } else {
            unreachable!()
        }
    }
    fn get_type(&self) -> AstNodeType {
        AstNodeType::UnaryNode
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let symbol = self
            .symbol
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_value(&self.code_pos)?;

        let symbol = LSymbol {
            value: codegen
                .builder
                .build_load(symbol.value.into_pointer_value(), "deref"),
            symbol_type: symbol.symbol_type.deref(),
        };

        Ok(LNodeResult::Value(symbol))
    }
}
