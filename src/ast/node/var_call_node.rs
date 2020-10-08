use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::token::CodePos;

#[cfg(feature = "crocoi")]
use crate::crocoi::{INodeResult, ISymTable};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult};

/// a node holding a variable reference
#[derive(Clone)]
pub struct VarCallNode {
    name: String,
    code_pos: CodePos,
}

impl VarCallNode {
    pub fn new(name: String, code_pos: CodePos) -> Self {
        VarCallNode { name, code_pos }
    }
}

impl AstNode for VarCallNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        let symbol = symtable
            .get_symbol(&self.name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?;

        Ok(INodeResult::Variable(symbol.clone()))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let symbol = codegen.symtable.get_symbol(&self.name).unwrap();
        Ok(LNodeResult::Variable(symbol.clone()))
    }
}
