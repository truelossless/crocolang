use crate::ast::{AstNode, BlockScope};
use crate::error::CrocoError;

#[cfg(feature = "crocoi")]
use crate::crocoi::{INodeResult, ISymTable};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult};

/// node containing multiple instructions
/// creates a new scope, or not
/// e.g: if body, function body, etc.
#[derive(Clone)]
pub struct BlockNode {
    // all instructions of the block node
    body: Vec<Box<dyn AstNode>>,
    scope: BlockScope,
    // instructions that get prepended, e.g variables in fn calls
    // prepended: Vec<Box<dyn AstNode>>,
    // same as previous, useful for future defer calls
    // appended: Vec<Box<dyn AstNode>>
}

impl BlockNode {
    pub fn new(scope: BlockScope) -> Self {
        BlockNode {
            body: Vec::new(),
            scope
            // prepended: Vec::new(),
            // appended: Vec::new(),
        }
    }
}

impl AstNode for BlockNode {
    fn prepend_child(&mut self, node: Box<dyn AstNode>) {
        self.body.insert(0, node);
    }

    fn add_child(&mut self, node: Box<dyn AstNode>) {
        self.body.push(node);
    }

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        // push a new scope if needed
        match self.scope {
            BlockScope::New => symtable.add_scope(),
            BlockScope::Keep => (),
        }

        // early return from the block
        let mut value = INodeResult::Void;
        // iterate over all nodes in the body
        for node in &mut self.body
        // .chain(self.prepended.iter_mut())
        // .chain(self.appended.iter_mut())
        {
            value = node.crocoi(symtable)?;

            match value {
                // propagate the early-returns until something catches it
                INodeResult::Return(_) | INodeResult::Break | INodeResult::Continue => break,
                _ => (),
            }
        }

        // clean up the injected statements
        // self.prepended.clear();
        // self.appended.clear();

        // return void if there is no return value
        if let INodeResult::Value(_) = value {
            return Ok(INodeResult::Void);
        };

        // we're done with this scope, drop it
        match self.scope {
            BlockScope::New => symtable.drop_scope(),
            BlockScope::Keep => (),
        }

        Ok(value)
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        if let BlockScope::New = self.scope {
            let block = codegen
                .context
                .append_basic_block(codegen.current_fn, "entry");

            let mut early_return = false;

            for node in &mut self.body {
                match node.crocol(codegen)? {
                    LNodeResult::Return(ret) => {
                        // TODO: void returns
                        codegen.builder.position_at_end(block);
                        if let Some(ret_val) = ret {
                            codegen.builder.build_return(Some(&ret_val.value));
                        } else {
                            codegen.builder.build_return(None);
                        }
                        early_return = true;
                        break;
                    }
                    LNodeResult::Variable(_) | LNodeResult::Value(_) | LNodeResult::Void => (),
                    _ => unimplemented!(),
                }
            }

            // if there is no early return the function returns void
            if !early_return {
                codegen.builder.position_at_end(block);
                codegen.builder.build_return(None);
            }

            // we're done with this scope, drop it
            match self.scope {
                BlockScope::New => codegen.symtable.drop_scope(),
                BlockScope::Keep => (),
            }
        }

        Ok(LNodeResult::Void)
    }
}
