use crate::ast::AstNode;
use crate::crocoi::symbol::{Array, FunctionDecl, ISymbol, Struct, ISymTable};
use crate::error::CrocoError;
use crate::{
    symbol_type::SymbolType,
    token::{CodePos, LiteralEnum, LiteralEnum::*},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// returns the LiteralEnum associated to a node
pub fn get_value(
    opt_node: &mut Option<Box<dyn AstNode>>,
    symtable: &mut ISymTable,
    code_pos: &CodePos,
) -> Result<LiteralEnum, CrocoError> {
    Ok(opt_node
        .as_mut()
        .ok_or_else(|| CrocoError::new(code_pos, "one variable hasn't been initialized !"))?
        .crocoi(symtable)?
        .into_symbol(code_pos)?
        .into_primitive()
        .map_err(|_| CrocoError::new(code_pos, "cannot use this type in an expression"))?)
}

/// returns the number value of a node
pub fn get_number_value(
    opt_node: &mut Option<Box<dyn AstNode>>,
    symtable: &mut ISymTable,
    code_pos: &CodePos,
) -> Result<f32, CrocoError> {
    let node = get_value(opt_node, symtable, &code_pos)?;
    match node {
        Num(x) => Ok(x),
        _ => Err(CrocoError::new(
            code_pos,
            "performing an operation on a wrong variable type !",
        )),
    }
}

/// auto deref if we have a Ref
/// e.g (&a).foo -> a.foo
pub fn auto_deref(mut symbol: ISymbol) -> ISymbol {
    loop {
        let reference;

        if let ISymbol::Ref(r) = symbol {
            reference = r.borrow().clone();
        } else {
            break;
        }

        symbol = reference;
    }

    symbol
}

/// initializes recursively a symbol to its default value
pub fn init_default(
    symbol_type: &SymbolType,
    symtable: &mut ISymTable,
    code_pos: &CodePos,
) -> Result<ISymbol, CrocoError> {
    Ok(match symbol_type {
        SymbolType::Num => ISymbol::Primitive(Num(0.)),
        SymbolType::Bool => ISymbol::Primitive(LiteralEnum::Bool(false)),
        SymbolType::Str => ISymbol::Primitive(Str(String::new())),
        SymbolType::Array(array_type) => ISymbol::Array(Array {
            array_type: array_type.clone(),
            contents: Vec::new(),
        }),
        SymbolType::Ref(_) => return Err(CrocoError::new(code_pos, "dangling reference")),
        SymbolType::Struct(struct_type) => {
            let struct_decl = symtable
                .get_struct_decl(struct_type)
                .map_err(|e| CrocoError::new(&code_pos, e))?
                .clone();

            let rc_fields = struct_decl.fields.into_iter();

            let mut fields = HashMap::new();
            for (k, x) in rc_fields {
                fields.insert(
                    k,
                    Rc::new(RefCell::new(init_default(&x, symtable, code_pos)?)),
                );
            }

            ISymbol::Struct(Struct {
                fields,
                struct_type: struct_type.clone(),
            })
        }
        SymbolType::Map(_, _) => todo!(),
        SymbolType::Function(fn_type) => ISymbol::Function(Box::new(FunctionDecl {
            args: fn_type.args.clone(),
            return_type: *fn_type.return_type.clone(),
            body: None,
        })),
        SymbolType::CrocoType => ISymbol::CrocoType(SymbolType::CrocoType),
    })
}
