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

    if opts.assembly && opts.object {
        eprintln!("Conflicting flags");
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
