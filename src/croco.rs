use croco::interpreter::Interpreter;
use gumdrop::Options;

#[derive(Options)]
struct MyOptions {

    #[options(free, help = "the .croco file to execute")]
    input: Vec<String>,

    #[options(help = "show help message")]
    help: bool,

    #[options(help = "show croco version")]
    version: bool,
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

    let file_path = if opts.input.is_empty() {
        "main.croco"
    } else {
        &opts.input[0]
    };

    let mut croco = Interpreter::new();

    if let Err(e) = croco.exec_file(&file_path) {
        println!("{}", e);
    }
}
