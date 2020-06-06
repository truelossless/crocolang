use croco::interpreter::Interpreter;
#[test]
fn it_is_called() {
    let mut interpreter = Interpreter::new();

    // function call
    assert!(interpreter.exec("fn a() {\n}\na()").is_ok());
}

#[test]
fn it_returns_values() {
    let mut interpreter = Interpreter::new();

    // num value
    assert!(interpreter.exec("fn a() num {\nreturn 42\n}\nassert(a() == 42)").is_ok());

    // str value
    assert!(interpreter.exec("fn a() str {\nreturn \"42\"\n}\nassert(a() == \"42\")").is_ok());

    // bool value
    assert!(interpreter.exec("fn a() bool {\nreturn true\n}\nassert(a() == true)").is_ok());

    // void value
    assert!(interpreter.exec("fn a() {\nreturn\n}").is_ok());
}