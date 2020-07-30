use crate::ast::{AstNode, AstNodeType, NodeResult};
use crate::error::CrocoError;
use crate::symbol::{SymTable, SymbolContent};
use crate::token::CodePos;

/// a node creating a reference to a symbol
#[derive(Clone)]
pub struct RefNode {
    symbol: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl RefNode {
    pub fn new(code_pos: CodePos) -> Self {
        RefNode {
            symbol: None,
            code_pos,
        }
    }
}

impl AstNode for RefNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        let symbol = self
            .symbol
            .as_mut()
            .unwrap()
            .visit(symtable)?
            .into_symbol(&self.code_pos)?;

        Ok(NodeResult::construct_symbol(SymbolContent::Ref(symbol)))
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
}
