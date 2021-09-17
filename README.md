[![Rust](https://github.com/danrspencer/assert_fn/actions/workflows/rust.yml/badge.svg)](https://github.com/danrspencer/assert_fn/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/assert_fn)](https://crates.io/crates/assert_fn)
[![docs.rs](https://img.shields.io/docsrs/assert_fn?label=docs.io)](https://docs.rs/assert_fn/)
# assert_fn

## Why?

The `assert_fn` library supplies a proc macro which can be used to turn test helper functions into `assert!` style macros. It is designed to be used where your test helper is performing an assert for you, for example: 

```rust
fn check_eq_if_doubled(a: usize, b: usize) {
  assert_eq!(a * 2, b)
}

check_eq_if_doubled(1, 2);
check_eq_if_doubled(2, 4);
check_eq_if_doubled(4, 8);
```

There are two reasons you'd want to do this:

1. Readability - `assert!` style macros are easy to spot in a test. They help others reading your test understand that this is where you are asserting correctness.
2. Traceability - In the above example, if you got a failure in one of your calls to `check_eq_if_doubled` the panic would originate on line 2, rather than the line which triggered the failure. In more complex tests this can make it hard to track down where the test is broken.

Using `assert_fn` the above can be written as:

```rust
use assert_fn::assert_fn;

#[assert_fn]
fn eq_if_doubled(a: usize, b: usize) -> bool {
    a * 2 == b
}

assert_eq_if_doubled!(1, 2);
assert_eq_if_doubled!(2, 4);
assert_eq_if_doubled!(4, 8);
```

Or if you want to use `assert_eq!` instead of `assert!` so you can see what the values were, just return a tuple instead of a bool:

```rust
use assert_fn::assert_fn;

#[assert_fn]
fn eq_if_doubled(a: usize, b: usize) -> (usize, usize) {
    (a * 2, b)
}

assert_eq_if_doubled!(1, 2);
assert_eq_if_doubled!(2, 4);
assert_eq_if_doubled!(4, 8);
```

In both of these examples the failure will be logged against the line in your test on which the error originated instead of inside a line inside the `eq_if_doubled`.

See the [Rust docs](https://docs.rs/assert_fn/latest/assert_fn/attr.assert_fn.html) for lots more examples.

## API

Supported fields on the `#[assert_fn]` macro.

### `message`

Only supported on functions returning a tuple, or `Result` of a tuple.

Provides a default message when the assert fails. The first item in the list must be a string literal. Additional items in the list are destructured from the returned tuple and can be substituted into the message via named properties. Any unused tuple fields prior to one you wish to use must be noted as a `_`.

Example consuming the third property on a returned tuple.

```rust
use assert_fn::assert_fn;

#[assert_fn(message("Nope! But {panic}", _, _, panic))]
fn meaning_of_life(a: usize) -> (usize, usize, String) {
    (a, 42, "don't panic!".to_string())
}
```

### `export`

If the `export` field is supplied to `#[assert_fn]` then `#[macro_export]` is added to your generated macro so it can be used elsewhere.

```rust
use assert_fn::assert_fn;

#[assert_fn(export)]
fn is_ten(num: usize) -> (usize, usize) {
    (num, 10)
}
```

## Trouble Shooting

Because macros have to be defined before they can be used, your helper functions must be declared above the tests you want to use them in. 

If you get a `cannot find macro` error check that your functions are in the correct order.

