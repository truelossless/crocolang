// Function tests
use super::{test_file_err, test_file_ok, ALL_BACKENDS, CROCOI};
#[test]
fn it_is_called() {
    // function call
    test_file_ok("tests/functions/it_is_called/call.croco", CROCOI);

    // function call with multiple arguments
    test_file_ok(
        "tests/functions/it_is_called/multiple_arguments.croco",
        CROCOI,
    );
}

#[test]
fn it_returns_values() {
    // num value
    test_file_ok("tests/functions/it_returns_values/num_value.croco", CROCOI);

    // str value
    test_file_ok("tests/functions/it_returns_values/str_value.croco", CROCOI);

    // bool value
    test_file_ok("tests/functions/it_returns_values/bool_value.croco", CROCOI);

    // void value and early return
    test_file_ok("tests/functions/it_returns_values/void_value.croco", CROCOI);

    // struct value
    test_file_ok(
        "tests/functions/it_returns_values/struct_value.croco",
        CROCOI,
    );
}

#[test]
fn it_uses_correctly_variables() {
    // block leaking a variable to the function
    test_file_err(
        "tests/functions/it_uses_correcly_variables/outside_var_err.croco",
        CROCOI,
    );

    // using a global variable in a function
    test_file_ok(
        "tests/functions/it_uses_correctly_variables/global_var.croco",
        CROCOI,
    );

    // if the variables are correctly restored after function calls
    test_file_ok(
        "tests/functions/it_uses_correctly_variables/var_restored.croco",
        CROCOI,
    );
}
