use crate::ast::{AstNode, BlockScope};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::{
    crocoi::{symbol::SymbolContent, INodeResult, ISymbol},
    crocol::{Codegen, LNodeResult},
    token::LiteralEnum::*,
};

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

    fn visit(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        // push a new scope if needed
        match self.scope {
            BlockScope::New => symtable.add_scope(),
            BlockScope::Keep => (),
        }

        // early return from the block
        let mut value = INodeResult::construct_symbol(SymbolContent::Primitive(Void));
        // iterate over all nodes in the body
        for node in &mut self.body
        // .chain(self.prepended.iter_mut())
        // .chain(self.appended.iter_mut())
        {
            value = node.visit(symtable)?;

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
        if let INodeResult::Symbol(_) = value {
            value = INodeResult::construct_symbol(SymbolContent::Primitive(Void))
        }

        // we're done with this scope, drop it
        match self.scope {
            BlockScope::New => symtable.drop_scope(),
            BlockScope::Keep => (),
        }

        Ok(value)
    }

    fn crocol<'ctx>(
        &mut self,
        codegen: &'ctx mut Codegen,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        if let BlockScope::New = self.scope {
            codegen
                .context
                .append_basic_block(codegen.current_fn, "entry");
        }

        Ok(LNodeResult::Void)
    }
}
