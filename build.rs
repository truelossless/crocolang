fn main() {
    
    #[cfg(windows)]
    {
        cc::Build::new()
            .define("MICROSOFT_CRAZINESS_IMPLEMENTATION", None)
            .file("src/microsoft_craziness.c")
            .compile("microsoft_craziness");
    }

}