use crate::{ast::node::BlockNode, error::CrocoError};
use crate::{ast::BlockScope, crocoi::CrocoiNode};

use crate::crocoi::{ICodegen, INodeResult};

impl CrocoiNode for BlockNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        // push a new scope if needed
        match self.scope {
            BlockScope::New | BlockScope::Function => codegen.symtable.add_scope(),
            BlockScope::Keep => (),
        }

        // early return from the block
        let mut value = INodeResult::Void;
        // iterate over all nodes in the body
        for node in &mut self.body
        // .chain(self.prepended.iter_mut())
        // .chain(self.appended.iter_mut())
        {
            value = node.crocoi(codegen)?;

            match value {
                // propagate the early-returns until something catches it
                INodeResult::Return(_) | INodeResult::Break | INodeResult::Continue => break,
                _ => (),
            }
        }

        // return void if there is no return value
        if let INodeResult::Value(_) = value {
            return Ok(INodeResult::Void);
        };

        // we're done with this scope, drop it
        match self.scope {
            BlockScope::New | BlockScope::Function => codegen.symtable.drop_scope(),
            BlockScope::Keep => (),
        }

        Ok(value)
    }
}
