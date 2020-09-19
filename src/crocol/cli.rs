use croco::Crocol;
use gumdrop::Options;

#[derive(Options)]
struct MyOptions {
    #[options(free, help = "the .croco file to execute")]
    input: Vec<String>,

    #[options(no_short, help = "verbose output")]
    verbose: bool,

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
    let opts = MyOptions::parse_args_default_or_exit();

    if opts.version {
        println!(env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    if opts.input.len() > 1 {
        eprintln!("Expected at maximum one argument");
        std::process::exit(1);
    }

    let mut output_flat_count = 0;

    if opts.assembly {
        output_flat_count += 1;
    }

    if opts.object {
        output_flat_count += 1;
    }

    if opts.emit_llvm {
        output_flat_count += 1;
    }

    if output_flat_count > 1 {
        eprintln!("Conflicting output flags");
        std::process::exit(1);
    }

    let file_path = if opts.input.is_empty() {
        "main.croco"
    } else {
        &opts.input[0]
    };

    let mut crocol = Crocol::new();

    if opts.assembly {
        crocol.emit_assembly();
    }

    if opts.object {
        crocol.emit_object_file();
    }

    if opts.emit_llvm {
        crocol.emit_llvm();
    }

    if opts.verbose {
        crocol.set_verbose(true);
    }

    if !opts.output.is_empty() {
        crocol.set_output(opts.output);
    }

    if let Err(e) = crocol.exec_file(file_path) {
        println!("{}", e);
    }
}
