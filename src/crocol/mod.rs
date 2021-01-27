pub mod node;
pub mod symbol;
pub mod utils;

pub use self::symbol::LCodegen;
pub use self::symbol::LNodeResult;
pub use self::symbol::LSymbol;
use self::utils::insert_builtin_functions;

use std::fs;

use crate::lexer::Lexer;
use crate::symbol::{Decl, SymTable};
use crate::{ast::AstNode, parser::Parser};
use crate::{
    error::{CrocoError, CrocoErrorKind},
    linker::Linker,
};
use inkwell::{
    context::Context,
    memory_buffer::MemoryBuffer,
    module::Module,
    passes::{PassManager, PassManagerBuilder},
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};
use utils::strip_ext;
#[derive(PartialEq)]
enum OutputFormat {
    LlvmIr,
    ObjectFile,
    Assembly,
    Executable,
}

/// crocol backend code generation, using llvm
// we could also return a Box<dyn AnyType>, but enum performance should be better
pub trait CrocolNode: AstNode {
    fn crocol<'ctx>(
        &mut self,
        _codegen: &mut LCodegen<'ctx>,
    ) -> Result<LNodeResult<'ctx>, CrocoError> {
        unimplemented!();
    }
}

pub struct Crocol {
    file_path: String,
    output_format: OutputFormat,
    output_flag: String,
    no_llvm_checks_flag: bool,
    verbose_flag: bool,
    optimization_flag: OptimizationLevel,
}

impl Default for Crocol {
    fn default() -> Self {
        Crocol::new()
    }
}

impl Crocol {
    pub fn new() -> Self {
        Crocol {
            file_path: String::new(),
            output_format: OutputFormat::Executable,
            output_flag: String::new(),
            no_llvm_checks_flag: false,
            verbose_flag: false,
            optimization_flag: OptimizationLevel::Default,
        }
    }

    pub fn emit_assembly(&mut self) {
        self.output_format = OutputFormat::Assembly;
    }

    pub fn emit_object_file(&mut self) {
        self.output_format = OutputFormat::ObjectFile;
    }

    pub fn emit_llvm(&mut self) {
        self.output_format = OutputFormat::LlvmIr;
    }

    pub fn set_no_llvm_checks(&mut self, no_llvm_checks: bool) {
        self.no_llvm_checks_flag = no_llvm_checks;
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose_flag = verbose;
    }

    pub fn set_output(&mut self, output: String) {
        self.output_flag = output;
    }

    pub fn set_optimization_level(&mut self, opt_level: OptimizationLevel) {
        self.optimization_flag = opt_level;
    }

    pub fn exec_file(&mut self, file_path: &str) -> Result<(), CrocoError> {
        let contents = fs::read_to_string(file_path).map_err(|_| {
            CrocoError::from_type(format!("file not found: {}", file_path), CrocoErrorKind::Io)
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

        // import our standard library
        let std_contents =
            MemoryBuffer::create_from_memory_range(include_bytes!("stdlib/global.bc"), "std");
        let std = Module::parse_bitcode_from_buffer(&std_contents, &context).unwrap();

        module.link_in_module(std).map_err(|_| {
            CrocoError::from_type("cannot link croco std library", CrocoErrorKind::Linker)
        })?;

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
                self.optimization_flag,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| {
                CrocoError::from_type(
                    "cannot create target machine",
                    CrocoErrorKind::CompileTarget,
                )
            })?;

        let ptr_size = context.ptr_sized_int_type(&target_machine.get_target_data(), None);

        let mut codegen = LCodegen {
            str_type: module.get_struct_type("struct.CrocoStr").unwrap(),
            context: &context,
            module,
            builder: context.create_builder(),
            symtable: SymTable::new(),
            ptr_size,
            current_fn: None,
            sret_ptr: None,
        };

        // insert all the built-in functions from the std
        insert_builtin_functions(&mut codegen.symtable);

        // insert all the declarations found by the parser
        for (fn_name, fn_decl) in parser.take_fn_decls() {
            codegen
                .symtable
                .register_decl(fn_name, Decl::FunctionDecl(fn_decl))
                .map_err(|e| CrocoError::from_type(e, CrocoErrorKind::Compilation))?;
        }

        for (struct_name, struct_decl) in parser.take_struct_decls() {
            codegen
                .symtable
                .register_decl(struct_name, Decl::StructDecl(struct_decl))
                .unwrap();
        }

        if let Err(mut e) = tree.crocol(&mut codegen) {
            e.set_kind(CrocoErrorKind::Compilation);
            return Err(e);
        }

        // this should never fail if our nodes are right (but this fails everytime obviously)
        if !self.no_llvm_checks_flag {
            codegen.module.verify().map_err(|e| {
                CrocoError::from_type(
                    format!(
                        "An LLVM error has occured, this should never happen!\n{}",
                        e
                    ),
                    CrocoErrorKind::Compilation,
                )
            })?;
        }

        let pass_manager_builder = PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(self.optimization_flag);

        let mpm = PassManager::create(());
        pass_manager_builder.populate_module_pass_manager(&mpm);
        mpm.run_on(&codegen.module);

        let lpm = PassManager::create(());
        pass_manager_builder.populate_lto_pass_manager(&lpm, false, false);
        lpm.run_on(&codegen.module);

        // emit an executable if we don't specifically want to emit assembly, object files, llvm ir
        // get the llvm file output name
        let llvm_output_filename =
            if !self.output_flag.is_empty() && self.output_format != OutputFormat::Executable {
                self.output_flag.clone()
            } else {
                let ext = match self.output_format {
                    OutputFormat::Assembly => "s",
                    OutputFormat::ObjectFile | OutputFormat::Executable => "o",
                    OutputFormat::LlvmIr => "ll",
                };
                format!("{}.{}", strip_ext(&self.file_path), ext)
            };

        // here we want to save the llvm ir
        if self.output_format == OutputFormat::LlvmIr {
            codegen
                .module
                .print_to_file(llvm_output_filename)
                .map_err(|_| {
                    CrocoError::from_type("cannot write llvm ir to disk", CrocoErrorKind::Io)
                })?;
            return Ok(());
        }

        let emit_method = match self.output_format {
            OutputFormat::Assembly => FileType::Assembly,
            _ => FileType::Object,
        };

        target_machine
            .write_to_file(&codegen.module, emit_method, llvm_output_filename.as_ref())
            .map_err(|e| {
                CrocoError::from_type(e.to_str().unwrap(), CrocoErrorKind::CompileTarget)
            })?;

        // if the user specified -S or -c, we're done here
        match self.output_format {
            OutputFormat::Assembly | OutputFormat::ObjectFile => return Ok(()),
            _ => (),
        }

        // else we need to link the object file into an executable
        let mut linker = Linker::new();

        let linker_search = linker
            .find_linker()
            .map_err(|e| CrocoError::from_type(e, CrocoErrorKind::Linker))?;

        if self.verbose_flag {
            println!("{}", linker_search);
        }

        let exe_output_filename = if !self.output_flag.is_empty() {
            self.output_flag.clone()
        } else if cfg!(windows) {
            format!("{}.exe", strip_ext(&self.file_path))
        } else {
            strip_ext(&self.file_path).to_owned()
        };

        let link_stage = linker
            .link(&llvm_output_filename, &exe_output_filename)
            .map_err(|e| CrocoError::from_type(e, CrocoErrorKind::Linker))?;

        if self.verbose_flag {
            println!("{}", link_stage);
        }

        // we can now remove the unwanted object file
        fs::remove_file(llvm_output_filename).map_err(|_| {
            CrocoError::from_type("cannot remove temporary object file", CrocoErrorKind::Io)
        })?;
        Ok(())
    }
}
