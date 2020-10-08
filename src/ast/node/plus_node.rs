#[cfg(feature = "checker")]
use crate::{
    checker::{Checker, CheckerSymbol},
    symbol_type::SymbolType,
};

#[cfg(feature = "crocoi")]
use crate::crocoi::{symbol::ISymbol, utils::get_value, INodeResult, ISymTable};

#[cfg(feature = "crocol")]
use crate::crocol::{Codegen, LNodeResult, LSymbol};

use crate::ast::{AstNode, AstNodeType};
use crate::error::CrocoError;
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
    #[cfg(feature = "checker")]
    fn check(&mut self, checker: &mut Checker) -> Result<CheckerSymbol, CrocoError> {
        let mut left_val = self.left.as_mut().unwrap().check(checker)?;
        let mut right_val = self.right.as_mut().unwrap().check(checker)?;

        left_val.set_used(checker, &self.code_pos)?;
        right_val.set_used(checker, &self.code_pos)?;

        // both variables are unknown, we can't deduce anything
        let ret = if left_val.is_unknown() && right_val.is_unknown() {
            CheckerSymbol::new_unknown_value()

        // one variable is unknown
        } else if left_val.is_unknown() || right_val.is_unknown() {
            let unknown_symbol;
            let known_symbol;

            if left_val.is_unknown() {
                unknown_symbol = left_val;
                known_symbol = right_val;
            } else {
                unknown_symbol = right_val;
                known_symbol = left_val;
            }

            // TODO: this will obviously fail on structs
            if let Some(var_name) = unknown_symbol.var {
                let variable = checker.symtable.get_symbol(&var_name).unwrap();
                variable.borrow_mut().symbol_type = known_symbol.symbol_type.clone();
            }

            CheckerSymbol::new_value(known_symbol.symbol_type.unwrap())

        // both values are known, check if the types match
        } else {
            // we can have either an addition or a concatenation
            match (
                left_val.symbol_type.unwrap(),
                right_val.symbol_type.unwrap(),
            ) {
                (SymbolType::Num, SymbolType::Num) => CheckerSymbol::new_value(SymbolType::Num),
                (SymbolType::Str, SymbolType::Str) => CheckerSymbol::new_value(SymbolType::Str),
                _ => {
                    return Err(CrocoError::new(
                        &self.code_pos,
                        "cannot add these two types together",
                    ))
                }
            }
        };

        Ok(ret)
    }

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
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
        Ok(INodeResult::Value(ISymbol::Primitive(value)))
    }

    #[cfg(feature = "crocol")]
    fn crocol<'ctx>(
        &mut self,
        codegen: &mut Codegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        let left_val = self
            .left
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_value(&self.code_pos)?;
        let right_val = self
            .right
            .as_mut()
            .unwrap()
            .crocol(codegen)?
            .into_value(&self.code_pos)?;

        let left_float = left_val.value.into_float_value();
        let right_float = right_val.value.into_float_value();

        let res = codegen
            .builder
            .build_float_mul(left_float, right_float, "tmpdiv");

        let symbol = LSymbol {
            value: res.into(),
            symbol_type: SymbolType::Num,
        };

        Ok(LNodeResult::Value(symbol))
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
