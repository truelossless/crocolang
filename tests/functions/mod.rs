// Function tests

use croco::Crocoi;
#[test]
fn it_is_called() {
    let mut interpreter = Crocoi::new();

    // function call
    assert!(interpreter
        .exec_file("tests/functions/it_is_called/call.croco")
        .is_ok());

    // function call with multiple arguments
    assert!(interpreter
        .exec_file("tests/functions/it_is_called/multiple_arguments.croco")
        .is_ok());
}

#[test]
fn it_returns_values() {
    let mut interpreter = Crocoi::new();

    // num value
    assert!(interpreter
        .exec_file("tests/functions/it_returns_values/num_value.croco")
        .is_ok());

    // str value
    assert!(interpreter
        .exec_file("tests/functions/it_returns_values/str_value.croco")
        .is_ok());

    // bool value
    assert!(interpreter
        .exec_file("tests/functions/it_returns_values/bool_value.croco")
        .is_ok());

    // void value and early return
    assert!(interpreter
        .exec_file("tests/functions/it_returns_values/void_value.croco")
        .is_ok());

    // struct value
    assert!(interpreter
        .exec_file("tests/functions/it_returns_values/struct_value.croco")
        .is_ok());
}
