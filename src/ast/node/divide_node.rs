use crate::ast::{AstNode, AstNodeType, INodeResult};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::token::{CodePos, LiteralEnum::*};

#[cfg(feature = "crocoi")]
use crate::crocoi::{symbol::SymbolContent, utils::get_number_value, ISymbol};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult};

#[derive(Clone)]
pub struct DivideNode {
    left: Option<Box<dyn AstNode>>,
    right: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl DivideNode {
    pub fn new(code_pos: CodePos) -> Self {
        DivideNode {
            left: None,
            right: None,
            code_pos,
        }
    }
}

impl AstNode for DivideNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let value = Num(get_number_value(&mut self.left, symtable, &self.code_pos)?
            / get_number_value(&mut self.right, symtable, &self.code_pos)?);
        Ok(INodeResult::construct_symbol(SymbolContent::Primitive(
            value,
        )))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(&mut self, codegen: &Codegen<'ctx>) -> Result<LNodeResult<'ctx>, CrocoError> {
        // a value may either be a pointer from a variable, or directly a float.
        // TODO: distinguish variables and pointer values ?

        let left_val = self.left.as_mut().unwrap().crocol(codegen)?.into_symbol();
        let right_val = self.right.as_mut().unwrap().crocol(codegen)?.into_symbol();

        let left_float = codegen.auto_deref(left_val).into_float_value();
        let right_float = codegen.auto_deref(right_val).into_float_value();

        let res = codegen
            .builder
            .build_float_div(left_float, right_float, "tmpdiv");
        Ok(LNodeResult::Symbol(res.into()))
    }

    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.left.is_none() {
            self.left = Some(node);
        } else if self.right.is_none() {
            self.right = Some(node);
        } else {
            unreachable!()
        }
    }
    fn get_type(&self) -> AstNodeType {
        AstNodeType::BinaryNode
    }
}
