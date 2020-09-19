#[cfg(feature = "crocoi")]
use crate::crocoi::{symbol::SymbolContent, utils::get_value, INodeResult, ISymbol};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult};

use crate::ast::{AstNode, AstNodeType};
use crate::error::CrocoError;
use crate::symbol::SymTable;
use crate::token::{CodePos, LiteralEnum::*};

/// a node used for addition and concatenation
#[derive(Clone)]
pub struct PlusNode {
    left: Option<Box<dyn AstNode>>,
    right: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl PlusNode {
    pub fn new(code_pos: CodePos) -> Self {
        PlusNode {
            left: None,
            right: None,
            code_pos,
        }
    }
}

/// node handling additions and concatenations
impl AstNode for PlusNode {
    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut SymTable<ISymbol>) -> Result<INodeResult, CrocoError> {
        let left_val = get_value(&mut self.left, symtable, &self.code_pos)?;
        let right_val = get_value(&mut self.right, symtable, &self.code_pos)?;

        // different kinds of additions can happen (concatenation or number addition)
        // the PlusNode also works for concatenation.
        let value = match (left_val, right_val) {
            (Num(n1), Num(n2)) => Num(n1 + n2),
            (Str(s1), Str(s2)) => Str(format!("{}{}", s1, s2)),
            _ => {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "cannot add these two types together",
                ))
            }
        };
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
            .build_float_add(left_float, right_float, "tmpadd");
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
