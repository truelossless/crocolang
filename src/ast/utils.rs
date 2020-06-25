use crate::ast::AstNode;
use crate::error::CrocoError;
use crate::symbol::{SymTable, Symbol, Struct};
use crate::token::{CodePos, LiteralEnum, LiteralEnum::*};

/// returns the LiteralEnum associated to a node
pub fn get_value(
    opt_node: &mut Option<Box<dyn AstNode>>,
    symtable: &mut SymTable,
    code_pos: &CodePos,
) -> Result<LiteralEnum, CrocoError> {
    match opt_node {
        Some(node) => {
            let visited = node
                .visit(symtable)?
                .into_symbol(code_pos)?
                .into_primitive()
                .map_err(|_| {
                    CrocoError::new(code_pos, "cannot use this type in an expr".to_owned())
                })?;

            if visited.is_void() {
                panic!("should have got a value there !!");
            }
            Ok(visited)
        }
        None => Err(CrocoError::new(
            code_pos,
            "One variable hasn't been initialized !".to_owned(),
        )),
    }
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
pub fn init_default(symbol: &mut Symbol, symtable: &mut SymTable, code_pos: &CodePos) -> Result<(), CrocoError> {

    *symbol = match symbol {
        Symbol::Primitive(Num(_)) => Symbol::Primitive(Num(Some(0.))),
        Symbol::Primitive(Str(_)) => Symbol::Primitive(Str(Some(String::new()))),
        Symbol::Primitive(Bool(_)) => Symbol::Primitive(Bool(Some(false))),

        Symbol::Struct(s) => {

            let mut struct_decl_default = symtable
                .get_struct_decl(&s.struct_type)
                .map_err(|e| CrocoError::new(&code_pos, e))?
                .clone();

            for (_, field) in struct_decl_default.iter_mut() {
                init_default(field, symtable, code_pos)?;
            }

            Symbol::Struct(
                Struct {
                    struct_type: s.struct_type.clone(),
                    fields: Some(struct_decl_default),
                }
            )
        }

        // we cannot have a struct with a void primitive
        _ => unreachable!(),
    };

    Ok(())
}