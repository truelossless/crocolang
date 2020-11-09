use croco::Crocol;
use gumdrop::Options;
use inkwell::OptimizationLevel;

#[derive(Options)]
struct MyOptions {
    #[options(free, help = "the .croco file to execute")]
    input: Vec<String>,

    #[options(help = "build without optimization")]
    o0: bool,

    #[options(help = "build with few optimizations")]
    o1: bool,
    
    #[options(help = "build with optimizations")]
    o2: bool,

    #[options(help = "build with aggressive optimizations")]
    o3: bool,

    #[options(no_short, help = "verbose output")]
    verbose: bool,

    #[options(no_short, help = "ignore llvm ir checks")]
    no_llvm_checks: bool,

    #[options(help = "show help message")]
    help: bool,

    #[options(help = "show crocol version")]
    version: bool,

    #[options(short = "S", no_long, help = "emit assembly only")]
    assembly: bool,

    #[options(short = "c", no_long, help = "emit object files only")]
    object: bool,

    #[options(no_short, help = "emit llvm ir only")]
    emit_llvm: bool,

    #[options(short = "o", no_long, help = "output file path")]
    output: String,

}

pub fn main() {
    let mut crocol = Crocol::new();

    let opts = MyOptions::parse_args_default_or_exit();

    if opts.version {
        println!(env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    if opts.input.len() > 1 {
        eprintln!("Expected at maximum one argument");
        std::process::exit(1);
    }

    let mut output_count = 0;
    let mut opt_count = 0;

    if opts.o0 {
        opt_count+=1;
        crocol.set_optimization_level(OptimizationLevel::None);
    }

    if opts.o1 {
        opt_count+=1;
        crocol.set_optimization_level(OptimizationLevel::Less);
    }


    if opts.o2 {
        opt_count+=1;
        crocol.set_optimization_level(OptimizationLevel::Default);
    }


    if opts.o3 {
        opt_count+=1;
        crocol.set_optimization_level(OptimizationLevel::Aggressive);
    }

    if opts.assembly {
        output_count += 1;
        crocol.emit_assembly();
    }

    if opts.object {
        output_count += 1;
        crocol.emit_object_file();
    }

    if opts.emit_llvm {
        output_count += 1;
        crocol.emit_llvm()
    }

    if opt_count > 1 {
        eprintln!("Conflicting optimization flags");
        std::process::exit(1);
    }

    if output_count > 1 {
        eprintln!("Conflicting output flags");
        std::process::exit(1);
    }

    let file_path = if opts.input.is_empty() {
        "main.croco"
    } else {
        &opts.input[0]
    };


    if opts.assembly {
        crocol.emit_assembly();
    }

    if opts.object {
        crocol.emit_object_file();
    }

    if opts.emit_llvm {
        crocol.emit_llvm();
    }
    
    if !opts.output.is_empty() {
        crocol.set_output(opts.output);
    }

    crocol.set_verbose(opts.verbose);
    crocol.set_no_llvm_checks(opts.no_llvm_checks);

    if let Err(e) = crocol.exec_file(file_path) {
        println!("{}", e);
    }
}
