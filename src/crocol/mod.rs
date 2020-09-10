pub mod symbol;

pub use self::symbol::Codegen;
pub use self::symbol::LNodeResult;
pub use self::symbol::LSymbol;

pub mod utils;

use std::rc::Rc;
use std::{cell::RefCell, fs};

use inkwell::{
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};

use crate::error::{CrocoError, CrocoErrorKind};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::{symbol::SymTable, token::CodePos};

#[derive(Default)]
pub struct Crocol {
    file_path: String,
}

impl Crocol {
    pub fn new() -> Self {
        Crocol {
            file_path: String::new(),
        }
    }

    pub fn exec_file(&mut self, file_path: &str) -> Result<(), CrocoError> {
        let contents = fs::read_to_string(file_path).map_err(|_| {
            let mut err = CrocoError::new(
                &CodePos {
                    file: Rc::from(file_path),
                    line: 0,
                    word: 0,
                },
                &format!("file not found: {}", file_path),
            );
            err.set_kind(CrocoErrorKind::IO);
            err
        })?;

        self.file_path = file_path.to_owned();
        self.exec(&contents)
    }

    pub fn exec(&mut self, code: &str) -> Result<(), CrocoError> {
        let tokens;
        let mut tree;

        let mut lexer = Lexer::new();
        lexer.set_file(&self.file_path);
        match lexer.process(code) {
            Ok(t) => tokens = t,
            Err(mut e) => {
                e.set_kind(CrocoErrorKind::Syntax);
                return Err(e);
            }
        }

        // println!("tokens: {:?}", &tokens);
        let mut parser = Parser::new();
        match parser.process(tokens) {
            Ok(root_node) => tree = root_node,
            Err(mut e) => {
                e.set_kind(CrocoErrorKind::Parse);
                return Err(e);
            }
        }

        // create the context and the default module
        let context = Context::create();
        let module = context.create_module("main");

        // initialize the target from this machine's specs
        // https://gitter.im/inkwell-rs/Lobby?at=5ef469846c06cd1bf452d3d3
        Target::initialize_all(&InitializationConfig::default());
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).map_err(|_| {
            CrocoError::from_type("cannot create target triple", CrocoErrorKind::CompileTarget)
        })?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| {
                CrocoError::from_type(
                    "cannot create target machine",
                    CrocoErrorKind::CompileTarget,
                )
            })?;

        // append the basic entry block
        let fn_return = context.void_type().fn_type(&[], false);
        let main_fn = module.add_function("main", fn_return, None);

        let codegen = Codegen {
            context: &context,
            module,
            builder: context.create_builder(),
            symtable: RefCell::new(SymTable::new()),
            ptr_size: context.ptr_sized_int_type(&target_machine.get_target_data(), None),
            current_fn: RefCell::new(main_fn),
        };

        if let Err(mut e) = tree.crocol(&codegen) {
            e.set_kind(CrocoErrorKind::Runtime);
            return Err(e);
        }

        codegen.module.verify().unwrap();

        // get the result with both formats
        let outputs = vec![(FileType::Assembly, "asm"), (FileType::Object, "o")];

        for output in outputs.into_iter() {
            let output_filename = format!("{}.{}", self.file_path, output.1);

            target_machine
                .write_to_file(&codegen.module, output.0, output_filename.as_ref())
                .map_err(|e| {
                    CrocoError::from_type(e.to_str().unwrap(), CrocoErrorKind::CompileTarget)
                })?;
        }

        Ok(())
    }
}
