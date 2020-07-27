use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::symbol::{Array, Struct, SymTable, SymbolContent};
use crate::token::{CodePos, LiteralEnum, LiteralEnum::*};
use std::{cell::RefCell, rc::Rc};

/// returns the LiteralEnum associated to a node
pub fn get_value(
    opt_node: &mut Option<Box<dyn AstNode>>,
    symtable: &mut SymTable,
    code_pos: &CodePos,
) -> Result<LiteralEnum, CrocoError> {
    Ok(opt_node
        .as_mut()
        .ok_or_else(|| {
            CrocoError::new(
                code_pos,
                "one variable hasn't been initialized !".to_owned(),
            )
        })?
        .visit(symtable)?
        .into_symbol(code_pos)?
        .borrow()
        .clone()
        .into_primitive()
        .map_err(|_| {
            CrocoError::new(code_pos, "cannot use this type in an expression".to_owned())
        })?)
}

/// returns the number value of a node
pub fn get_number_value(
    opt_node: &mut Option<Box<dyn AstNode>>,
    symtable: &mut SymTable,
    code_pos: &CodePos,
) -> Result<f32, CrocoError> {
    let node = get_value(opt_node, symtable, &code_pos)?;
    match node {
        Num(x) => Ok(x.ok_or_else(|| {
            CrocoError::new(
                code_pos,
                "performing an operation with a type instead of a value".to_owned(),
            )
        }))?,
        _ => Err(CrocoError::new(
            code_pos,
            "performing an operation on a wrong variable type !".to_owned(),
        )),
    }
}

/// initializes recursively a symbol to its default value
pub fn init_default(
    symbol: &mut SymbolContent,
    symtable: &mut SymTable,
    code_pos: &CodePos,
) -> Result<(), CrocoError> {
    *symbol = match symbol {
        SymbolContent::Primitive(Num(_)) => SymbolContent::Primitive(Num(Some(0.))),
        SymbolContent::Primitive(Str(_)) => SymbolContent::Primitive(Str(Some(String::new()))),
        SymbolContent::Primitive(Bool(_)) => SymbolContent::Primitive(Bool(Some(false))),

        SymbolContent::Struct(s) => {
            let mut struct_decl_default = symtable
                .get_struct_decl(&s.struct_type)
                .map_err(|e| CrocoError::new(&code_pos, e))?
                .clone();

            for mut field in struct_decl_default.fields.values_mut() {
                init_default(&mut field, symtable, code_pos)?;
            }

            let rc_fields = struct_decl_default
                .fields
                .into_iter()
                .map(|(k, x)| (k, Rc::new(RefCell::new(x))))
                .collect();

            SymbolContent::Struct(Struct {
                fields: Some(rc_fields),
                struct_type: s.struct_type.clone(),
            })
        }

        SymbolContent::Array(arr) => SymbolContent::Array(Array {
            contents: Some(Vec::new()),
            array_type: arr.array_type.clone(),
        }),

        SymbolContent::Ref(_) => {
            return Err(CrocoError::new(&code_pos, "dangling reference".to_owned()))
        }

        // we cannot have a struct with a void primitive
        _ => unreachable!(),
    };

    Ok(())
}
