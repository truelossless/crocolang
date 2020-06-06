use croco::interpreter::Interpreter;

#[test]
fn it_assign_correct_values() {
    let mut interpreter = Interpreter::new();

    // num assignment
    assert!(interpreter.exec("let a = 3\nassert(a == 3)").is_ok());
    assert!(interpreter.exec("let a num\nassert(a == 0)").is_ok());

    // str assignment
    assert!(interpreter
        .exec("let a = \"croco\"\nassert(a == \"croco\")")
        .is_ok());
    assert!(interpreter.exec("let a str\nassert(a == \"\")").is_ok());

    // bool assignment
    assert!(interpreter.exec("let a = true\nassert(a == true)").is_ok());
    assert!(interpreter.exec("let a bool\nassert(a == false)").is_ok());
}

#[test]
fn it_shadows_correctly() {
    let mut interpreter = Interpreter::new();

    assert!(interpreter.exec("let a = 4\nlet a = 0").is_err());
    assert!(interpreter
        .exec("let a = 4\nif true {\nlet a = 0\n}")
        .is_ok());
}

#[test]
fn it_does_not_change_type() {
    let mut interpreter = Interpreter::new();

    assert!(interpreter.exec("let a = 4\na = true").is_err());
    assert!(interpreter.exec("let a = \"croco\"\na = true").is_err());
}

#[test]
fn it_calculates_correctly() {
    let mut interpreter = Interpreter::new();

    assert!(interpreter.exec("assert(4*4/2+3 == 11)").is_ok());
    assert!(interpreter.exec("assert(2^2-11 == -7)").is_ok());
    assert!(interpreter.exec("assert(1.5*3 == 4.5)").is_ok());
}
