use crocolib::interpreter::Interpreter;

pub fn main() {
    let mut croco = Interpreter::new();
    // if let Err(e) = croco.exec("let testvar = 5+18/(9-3)\nlet test4_var2 = 3+21") {
    //     panic!("{}", e);
    // }

    if let Err(e) = croco.exec_file("main.croco") {
        println!("{}", e);
    }
}
