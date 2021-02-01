use crate::ALL_BACKENDS;

use super::test_file_ok;

#[test]
fn it_loops_correctly() {
    test_file_ok("tests/loops/it_loops_correctly/counter.croco", ALL_BACKENDS);

    test_file_ok("tests/loops/it_loops_correctly/shadow.croco", ALL_BACKENDS);
}

#[test]
fn it_returns_early() {
    test_file_ok("tests/loops/it_returns_early/break.croco", ALL_BACKENDS);

    test_file_ok("tests/loops/it_returns_early/return.croco", ALL_BACKENDS);

    test_file_ok("tests/loops/it_returns_early/continue.croco", ALL_BACKENDS);
}
