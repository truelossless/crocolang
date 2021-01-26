use crate::{test_file_err, test_file_ok, CROCOI};

// Array tests

#[test]
fn it_can_be_created() {
    test_file_ok(
        "tests/arrays/it_can_be_created/basic_creation.croco",
        CROCOI,
    );

    test_file_ok(
        "tests/arrays/it_can_be_created/struct_creation.croco",
        CROCOI,
    );
}

#[test]
fn it_does_not_change_type() {
    test_file_err(
        "tests/arrays/it_does_not_change_type/num_and_str_err.croco",
        CROCOI,
    );

    test_file_err(
        "tests/arrays/it_does_not_change_type/struct_err.croco",
        CROCOI,
    );
}

#[test]
fn it_is_indexable() {
    test_file_ok("tests/arrays/it_is_indexable/basic_index.croco", CROCOI);

    test_file_err(
        "tests/arrays/it_is_indexable/out_of_bounds_err.croco",
        CROCOI,
    );

    test_file_ok(
        "tests/arrays/it_is_indexable/multi_dimensional.croco",
        CROCOI,
    );
}
