use crate::ast::{AstNode, AstNodeType, INodeResult};
use crate::error::CrocoError;
use crate::token::CodePos;

#[cfg(feature = "crocoi")]
use {crate::crocoi::ISymTable, std::cell::RefCell, std::rc::Rc};

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
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        // it only make sense to create a reference to a reference or a variable, everything else is just
        // dropping temporary values
        // e.g &12, &[3, 4] ...
        // references to variables are handled with VarRefNode, so we just care about
        // references of references.

        let symbol = self
            .symbol
            .as_mut()
            .unwrap()
            .crocoi(symtable)?
            .into_var_ref(&self.code_pos)
            .map_err(|_| CrocoError::new(&self.code_pos, "dropping a temporary value"))?;

        Ok(INodeResult::Variable(Rc::new(RefCell::new(symbol))))
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
