use crate::ast::{AstNode, AstNodeType, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{
    crocoi::ISymbol,
    crocol::{Codegen, LNodeResult},
    token::CodePos,
};

/// a node creating a reference to a symbol
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
    fn visit(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let symbol = self
            .symbol
            .as_mut()
            .unwrap()
            .visit(symtable)?
            .into_symbol(&self.code_pos)?
            .borrow()
            .clone()
            .into_ref()
            .map_err(|_| CrocoError::new(&self.code_pos, "cannot dereference this variable"))?;

        Ok(INodeResult::Symbol(symbol))
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

    fn crocol<'ctx>(
        &mut self,
        codegen: &'ctx mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let ptr = self
            .symbol
            .as_ref()
            .unwrap()
            .crocol(codegen)?
            .into_symbol()
            .into_pointer_value();

        Ok(LNodeResult::Symbol(
            codegen.builder.build_load(ptr, "deref").into(),
        ))
    }

    fn prepend_child(&mut self, _node: Box<dyn AstNode>) {
        unimplemented!();
    }
}
