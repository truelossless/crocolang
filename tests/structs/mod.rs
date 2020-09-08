// Struct tests

use croco::Crocoi;

#[test]
fn it_is_declared_correctly() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/structs/it_is_declared_correctly/declaration.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/structs/it_is_declared_correctly/declaration_err.croco")
        .is_err());
}

#[test]
fn it_is_instanciated_correctly() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/structs/it_is_instanciated_correctly/instanciation.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/structs/it_is_instanciated_correctly/nested_instanciation.croco")
        .is_ok());
}

#[test]
fn it_returns_correct_field_values() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/structs/it_returns_correct_field_values/field_value.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/structs/it_returns_correct_field_values/default_value.croco")
        .is_ok());
}

#[test]
fn it_sets_correct_field_values() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/structs/it_sets_correct_field_values/set_field_value.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/structs/it_sets_correct_field_values/set_multiple_field_values.croco")
        .is_ok());
}

#[test]
fn it_does_not_change_field_type() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/structs/it_does_not_change_field_type/num_to_str_err.croco")
        .is_err());
}

#[test]
fn it_calls_methods_correctly() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/structs/it_calls_methods_correctly/basic_method.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/structs/it_calls_methods_correctly/self_method.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/structs/it_calls_methods_correctly/self_mut_method.croco")
        .is_ok());
}
