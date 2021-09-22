use assert_fn::assert_fn;
use test_helpers::{catch_panic_message, PanicMessage};

#[assert_fn(message("{num} wasn't quite right", num))]
fn is_ten(num: usize) -> (usize, usize, String) {
    (num, 10, "Some other useful value".to_string())
}

#[test]
fn it_doesnt_require_placeholders_for_every_tuple_value() {
    let result = catch_panic_message(|| assert_is_ten!(9));
    assert_eq!(
        result,
        PanicMessage::Message(
            "assertion failed: `(left == right)`\n  left: `9`,\n right: `10`: 9 wasn't quite right"
                .to_string()
        )
    )
}

#[assert_fn]
fn it_can_copy() -> Result<(String, String), ()> {
    Ok(("Test1".to_string(), "Test1".to_string()))
}

#[test]
fn it_can_return_a_result_of_something_that_doesnt_implement_copy() {
    assert_it_can_copy!("test");
}