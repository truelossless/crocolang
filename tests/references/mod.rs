use crate::{test_file_err, test_file_ok, CROCOI};

// References tests

#[test]
fn it_assigns_correctly() {
    test_file_ok(
        "tests/references/it_assigns_correctly/basic_reference.croco",
        CROCOI,
    );
    test_file_ok(
        "tests/references/it_assigns_correctly/mut_reference.croco",
        CROCOI,
    );
    test_file_ok(
        "tests/references/it_assigns_correctly/correct_mut.croco",
        CROCOI,
    );
    test_file_ok(
        "tests/references/it_assigns_correctly/deref_ref.croco",
        CROCOI,
    );

    test_file_err(
        "tests/references/it_assigns_correctly/deref_ref_err.croco",
        CROCOI,
    );
}

#[test]
fn it_auto_dereferences() {
    test_file_ok(
        "tests/references/it_auto_dereferences/auto_dereference.croco",
        CROCOI,
    );
}
