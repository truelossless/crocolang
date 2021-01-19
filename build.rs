use std::{
    io::{self, Write},
    process::Command,
};

fn main() {
    #[cfg(feature = "crocol")]
    {
        #[cfg(windows)]
        {
            cc::Build::new()
                .define("MICROSOFT_CRAZINESS_IMPLEMENTATION", None)
                .file("src/microsoft_craziness.c")
                .compile("microsoft_craziness");
        }

        // build the crocol stdlib
        Command::new("clang")
            .current_dir("src/crocol/stdlib")
            .args(&["global.c", "-c", "-emit-llvm", "-o", "global.bc"])
            .output()
            .expect("Unable to start Clang. Check if it's correctly installed.");
    }
}
