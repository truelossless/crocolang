use crate::ast::node::*;
use crate::ast::{AstNode, NodeResult};
use crate::symbol::{symbol_eq, FunctionKind, SymTable, Symbol, SymbolContent};
use crate::token::{CodePos, LiteralEnum};

use crate::error::CrocoError;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
/// a node to access symbol methods
pub struct DotMethodNode {
    // the struct to which belongs the method
    bottom: Box<dyn AstNode>,
    // the method name
    method_name: String,
    // the method args
    args: Vec<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl DotMethodNode {
    pub fn new(
        bottom: Box<dyn AstNode>,
        method_name: String,
        args: Vec<Box<dyn AstNode>>,
        code_pos: CodePos,
    ) -> Self {
        DotMethodNode {
            bottom,
            method_name,
            args,
            code_pos,
        }
    }
}

impl AstNode for DotMethodNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // resolve the method arguments
        let mut visited_args = Vec::new();
        for arg in &mut self.args {
            let value = arg.visit(symtable)?.into_symbol(&self.code_pos)?;
            visited_args.push(value);
        }

        let mut symbol = self.bottom.visit(symtable)?.into_symbol(&self.code_pos)?;

        // auto deref if we have a Ref
        loop {
            let reference;

            if let SymbolContent::Ref(r) = &*symbol.borrow() {
                reference = r.clone();
            } else {
                break;
            }

            symbol = reference;
        }

        let method_decl = match &*symbol.borrow() {
            // struct methods
            SymbolContent::Struct(s) => {
                // inject the "self" argument
                visited_args.insert(0, symbol.clone());

                // look for the method declaration
                let struct_decl = symtable
                    .get_struct_decl(&s.struct_type)
                    .map_err(|e| CrocoError::new(&self.code_pos, e))?;

                struct_decl
                    .methods
                    .get(&self.method_name)
                    .ok_or(CrocoError::new(
                        &self.code_pos,
                        format!("no method called {}", self.method_name),
                    ))?
                    .clone()
            }

            // str methods
            SymbolContent::Primitive(LiteralEnum::Str(Some(_s))) => {
                todo!();
            }

            // num methods
            SymbolContent::Primitive(LiteralEnum::Num(Some(_n))) => {
                todo!();
            }

            // bool methods
            SymbolContent::Primitive(LiteralEnum::Bool(Some(_b))) => {
                todo!();
            }

            // array methods
            SymbolContent::Array(_arr) => {
                todo!();
            }

            _ => unreachable!(),
        };

        // TODO: this code is quite redundant of FunctionCallNode. merge them together ?
        // check the number of parameters but don't forget to omit self
        if visited_args.len() != method_decl.args.len() + 1 {
            return Err(CrocoError::new(
                &self.code_pos,
                format!(
                "mismatched number of arguments in function {}\n expected {} parameters but got {}",
                self.method_name,
                method_decl.args.len(),
                visited_args.len() - 1
            ),
            ));
        }

        // skip self
        for (i, arg) in visited_args.iter().skip(1).enumerate() {
            if !symbol_eq(&*arg.borrow(), &method_decl.args[i].arg_type) {
                return Err(CrocoError::new(
                    &self.code_pos,
                    format!(
                        "parameter {} doesn't match {} method definition",
                        i + 1,
                        self.method_name
                    ),
                ));
            }
        }

        let return_value: Symbol;

        if !symbol_eq(&method_decl.return_type, &*return_value.borrow()) {
            return Err(CrocoError::new(
                &self.code_pos,
                format!("function {} returned a wrong type", self.method_name),
            ));
        }

        Ok(NodeResult::Symbol(return_value))
    }

    fn prepend_child(&mut self, node: Box<dyn AstNode>) {
        self.args.insert(0, node);
    }

    fn add_child(&mut self, node: Box<dyn AstNode>) {
        self.args.push(node);
    }
}
