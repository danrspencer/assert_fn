use std::panic::{catch_unwind, UnwindSafe};

#[derive(Debug, PartialEq)]
pub enum PanicMessage {
    Message(String),
    CouldNotGetMessage,
    DidNotPanic,
}

pub fn catch_panic_message<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> PanicMessage {
    match catch_unwind(f) {
        Err(panic) => match (panic.downcast_ref::<String>(), panic.downcast_ref::<&str>()) {
            (Some(panic_message), _) => PanicMessage::Message(panic_message.to_string()),
            (_, Some(panic_message)) => PanicMessage::Message(panic_message.to_string()),
            _ => PanicMessage::CouldNotGetMessage,
        },
        _ => PanicMessage::DidNotPanic,
    }
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn it_gets_the_panic_message() {
    // Assert raises a panic with either a &str
    assert_eq!(
        catch_panic_message(|| assert!(false, "oh no!")),
        PanicMessage::Message("oh no!".to_string())
    );

    // Or a String, depending on if you're formatting or not
    assert_eq!(
        catch_panic_message(|| assert!(false, "oh no! {}", "with arg")),
        PanicMessage::Message("oh no! with arg".to_string())
    );

    // A panic can actually have any value in it
    assert_eq!(
        catch_panic_message(|| std::panic::panic_any(5)),
        PanicMessage::CouldNotGetMessage
    );

    // And sometimes we "Don't panic"
    assert_eq!(
        catch_panic_message(|| assert!(true)),
        PanicMessage::DidNotPanic
    )
}
