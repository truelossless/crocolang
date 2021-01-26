use crate::{test_file_err, test_file_ok, CROCOI};

// Struct tests

#[test]
fn it_is_declared_correctly() {
    test_file_ok(
        "tests/structs/it_is_declared_correctly/declaration.croco",
        CROCOI,
    );

    test_file_err(
        "tests/structs/it_is_declared_correctly/declaration_err.croco",
        CROCOI,
    );
}

#[test]
fn it_is_instanciated_correctly() {
    test_file_ok(
        "tests/structs/it_is_instanciated_correctly/instanciation.croco",
        CROCOI,
    );

    test_file_ok(
        "tests/structs/it_is_instanciated_correctly/nested_instanciation.croco",
        CROCOI,
    );
}

#[test]
fn it_returns_correct_field_values() {
    test_file_ok(
        "tests/structs/it_returns_correct_field_values/field_value.croco",
        CROCOI,
    );

    test_file_ok(
        "tests/structs/it_returns_correct_field_values/default_value.croco",
        CROCOI,
    );
}

#[test]
fn it_sets_correct_field_values() {
    test_file_ok(
        "tests/structs/it_sets_correct_field_values/set_field_value.croco",
        CROCOI,
    );

    test_file_ok(
        "tests/structs/it_sets_correct_field_values/set_multiple_field_values.croco",
        CROCOI,
    );
}

#[test]
fn it_does_not_change_field_type() {
    test_file_err(
        "tests/structs/it_does_not_change_field_type/num_to_str_err.croco",
        CROCOI,
    );
}

#[test]
fn it_calls_methods_correctly() {
    test_file_ok(
        "tests/structs/it_calls_methods_correctly/basic_method.croco",
        CROCOI,
    );

    test_file_ok(
        "tests/structs/it_calls_methods_correctly/self_method.croco",
        CROCOI,
    );

    test_file_ok(
        "tests/structs/it_calls_methods_correctly/self_mut_method.croco",
        CROCOI,
    );
}
