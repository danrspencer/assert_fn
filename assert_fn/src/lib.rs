#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use syn::{
    parse_macro_input, AttributeArgs, GenericArgument, ItemFn, Lit, Meta, NestedMeta,
    PathArguments, PathSegment, ReturnType, Type,
};

#[proc_macro_attribute]
/// The basic usage of `#[assert_fn]` is to convert a function which returns a boolean into an `assert!` style macro.
/// ```
/// # use assert_fn::assert_fn;
/// # use std::panic::catch_unwind;
/// #[assert_fn]
/// fn is_ten(num: usize) -> bool {
///     num == 10
/// }
///
/// assert_is_ten!(10);
/// assert!(catch_unwind(|| assert_is_ten!(9)).is_err());
/// ```
///
/// A custom message can be specified on the `#[assert_fn]` macro, e.g.
/// ```
/// # use assert_fn::assert_fn;
/// # use test_helpers::{catch_panic_message, PanicMessage};
/// #[assert_fn(message("That wasn't ten"))]
/// fn is_ten(num: usize) -> bool {
///     num == 10
/// }
///
/// let result = catch_panic_message(|| assert_is_ten!(9));
/// assert_eq!(result, PanicMessage::Message("That wasn't ten".to_string()));
/// ```
///
/// The generated macros also support custom messages in the same manner as the standard `assert!` macro.
/// This overrides any message set in the `#[assert_fn]` macro.
/// ```
/// # use assert_fn::assert_fn;
/// # use test_helpers::{catch_panic_message, PanicMessage};
/// #[assert_fn(message("Default message"))]
/// fn is_ten(num: usize) -> bool {
///     num == 10
/// }
///
/// let result = catch_panic_message(|| assert_is_ten!(9, "Custom error {}", "example"));
/// assert_eq!(result, PanicMessage::Message("Custom error example".to_string()))
/// ```
///
/// If you'd like to retain more details in your error message you can return a tuple instead of a bool
/// which will use `assert_eq!` under the hood instead of `assert!`
/// ```
/// # use assert_fn::assert_fn;
/// # use test_helpers::{catch_panic_message, PanicMessage};
/// #[assert_fn(message("Doh!"))]
/// fn is_ten(num: usize) -> (usize, usize) {
///     (num, 10)
/// }
///
/// let result = catch_panic_message(|| assert_is_ten!(9));
/// assert_eq!(result, PanicMessage::Message("assertion failed: `(left == right)`\n  left: `9`,\n right: `10`: Doh!".to_string()))
/// ```
///
/// The returned tuple can be destructured into your message via named parameters. Only the first two values in the tuple are used
/// for the assert, so you can return extra values for your message. Use `_` for any params you want to skip.
/// ```
/// # use assert_fn::assert_fn;
/// # use test_helpers::{catch_panic_message, PanicMessage};
/// #[assert_fn(message("The difference was `{diff}`", _, _, diff))]
/// fn is_ten(num: usize) -> (usize, usize, isize) {
///     (num, 10, num as isize-10)
/// }
///
/// let result = catch_panic_message(|| assert_is_ten!(9));
/// assert_eq!(result, PanicMessage::Message("assertion failed: `(left == right)`\n  left: `9`,\n right: `10`: The difference was `-1`".to_string()))
/// ```
///
/// Async functions are supported and will return an async block for you to await in your test function
/// ```
/// # use assert_fn::assert_fn;
/// #[assert_fn]
/// async fn is_ten(num: usize) -> (usize, usize) {
///     (num, 10)
/// }
///
/// tokio_test::block_on( async { assert_is_ten!(10).await } );
/// ```
///
/// Functions that return a `Result` are also supported. The result is ignored inside the assert and
/// is instead returned to the test function to be handled.
/// ```
/// # use assert_fn::assert_fn;
/// #[assert_fn]
/// fn is_ten(num: usize) -> Result<(usize, usize), ()> {
///     Ok((num, 10))
/// }
///
/// fn main() -> Result<(), ()> {
///     assert_is_ten!(10)?;
///     Ok(())
/// }
/// ```
///
/// Finally, as demonstrated in the Result example, the return value of your assert function is returned
/// from the macro. This allows you to get back additional useful values from your assert to use elsewhere
/// in your test.
/// ```
/// # use assert_fn::assert_fn;
/// #[assert_fn]
/// fn is_ten(num: usize) -> (usize, usize, String) {
///     (num, 10, "Some other useful value".to_string())
/// }
///
/// let (_, _, value) = assert_is_ten!(10);
/// assert_eq!(&value, "Some other useful value")
/// ```
pub fn assert_fn(args: TokenStream, item: TokenStream) -> TokenStream {
    let raw_item = item.clone();
    let item = parse_macro_input!(item as ItemFn);
    let args = parse_macro_input!(args as AttributeArgs);

    let return_type = get_return_type(&item);
    let assert_message = get_message(&args);

    let fn_name = item.sig.ident.to_string();
    let (params, values) = get_values_and_params(&item);
    let (async_block, dot_await) = get_async(&item);
    let tuple_destructure = get_tuple_destructure(&assert_message, &return_type);
    let (if_result_open, if_result_close) = get_result_block(&return_type);
    let assert_call = get_assert_call(&return_type);
    let message = assert_message.map(|msg| msg.message).unwrap_or_default();

    format!(
        r#"
        #[macro_export]
        macro_rules! assert_{fn_name} {{
            ({params_trimmed}$(,)?) => {{ {async_block} {{
                let result = {fn_name}({values}){dot_await};

                {if_result_open}
                {tuple_destructure}
                {assert_call}{message});
                {if_result_close}

                result
            }}}};
            ({params}$($arg:tt)+) => {{ {async_block} {{
                let result = {fn_name}({values}){dot_await};

                {if_result_open}
                {assert_call}, $($arg)*);
                {if_result_close}

                result
            }}}};
        }}

        {original_fn}
    "#,
        fn_name = fn_name,
        params = params,
        params_trimmed = params.trim_end_matches(|c| c == ','),
        values = values.trim_end_matches(|c| c == ','),
        async_block = async_block,
        dot_await = dot_await,
        tuple_destructure = tuple_destructure,
        if_result_open = if_result_open,
        if_result_close = if_result_close,
        assert_call = assert_call,
        message = message,
        original_fn = raw_item.to_string()
    )
    .parse()
    .expect("Generated invalid tokens")
}

enum AssertReturnType {
    Bool,
    Tuple(u8),
    ResultBool,
    ResultTuple(u8),
}

fn get_return_type(item: &ItemFn) -> AssertReturnType {
    let fn_name = item.sig.ident.to_string();

    let return_type = match &item.sig.output {
        ReturnType::Default => panic!("{} does not return anything", fn_name),
        ReturnType::Type(_, return_type) => *return_type.clone(),
    };

    match return_type {
        Type::Path(path) => {
            let last_segment = path
                .path
                .segments
                .last()
                .expect("{} returned an unexpected return type");
            let path_ident = last_segment.ident.to_string();

            if path_ident == "bool" {
                AssertReturnType::Bool
            } else if path_ident == "Result" {
                get_return_result_type(&fn_name, last_segment)
            } else {
                panic!(
                    "{} must return a bool, tuple or a Result wrapping one of those types",
                    fn_name
                )
            }
        }
        Type::Tuple(tuple) => AssertReturnType::Tuple(tuple.elems.len() as u8),
        _ => panic!(
            "{} must return a bool, tuple or a Result wrapping one of those types",
            fn_name
        ),
    }
}

fn get_return_result_type(fn_name: &str, path_segment: &PathSegment) -> AssertReturnType {
    let args = match &path_segment.arguments {
        PathArguments::AngleBracketed(args) => args,
        _ => panic!("{} returned an invalid Result type", fn_name),
    };

    let arg_type = match args
        .args
        .first()
        .unwrap_or_else(|| panic!("{} returned an invalid Result type", fn_name))
    {
        GenericArgument::Type(arg_type) => arg_type,
        _ => panic!("{} returned an invalid Result type", fn_name),
    };

    match arg_type {
        Type::Path(arg_path) => {
            let arg_path_ident = arg_path
                .path
                .segments
                .last()
                .unwrap_or_else(|| panic!("{} returned an invalid Result type", fn_name))
                .ident
                .clone();
            if arg_path_ident == "bool" {
                AssertReturnType::ResultBool
            } else {
                panic!("{} must return a Result of type bool or tuple", fn_name)
            }
        }
        Type::Tuple(tuple) => AssertReturnType::ResultTuple(tuple.elems.len() as u8),
        _ => panic!("{} must return a Result of type bool or tuple", fn_name),
    }
}

fn get_values_and_params(item: &ItemFn) -> (String, String) {
    item.sig.inputs.iter().enumerate().fold(
        ("".to_string(), "".to_string()),
        |(params, values), (n, _)| {
            (
                format!("{}$arg_{}:expr,", params, n),
                format!("{}$arg_{},", values, n),
            )
        },
    )
}

fn get_async(item: &ItemFn) -> (String, String) {
    if item.sig.asyncness.is_some() {
        ("async".to_string(), ".await".to_string())
    } else {
        ("".to_string(), "".to_string())
    }
}

fn get_tuple_destructure(
    assert_message: &Option<AssertMessage>,
    return_type: &AssertReturnType,
) -> String {
    if let Some(mut args) = assert_message
        .clone()
        .map(|msg| msg.args)
        .filter(|args| !args.is_empty())
    {
        let tuple_size = match return_type {
            AssertReturnType::Bool | AssertReturnType::ResultBool => {
                panic!("Tried to use message args on function with boolean return type")
            }
            AssertReturnType::Tuple(n) | AssertReturnType::ResultTuple(n) => *n,
        };

        // Make sure we have enough destructuring placeholders for the full tuple
        while (args.len() as u8) < tuple_size {
            args.push("_".to_string());
        }
        format!("let ({}) = result;", args.join(", "))
    } else {
        "".to_string()
    }
}

fn get_result_block(return_type: &AssertReturnType) -> (String, String) {
    if matches!(
        return_type,
        AssertReturnType::ResultBool | AssertReturnType::ResultTuple(_)
    ) {
        ("if let Ok(result) = result {".to_string(), "}".to_string())
    } else {
        ("".to_string(), "".to_string())
    }
}

fn get_assert_call(return_type: &AssertReturnType) -> String {
    match return_type {
        AssertReturnType::Bool | AssertReturnType::ResultBool => "assert!(result".to_string(),
        AssertReturnType::Tuple(_) | AssertReturnType::ResultTuple(_) => {
            "assert_eq!(result.0, result.1".to_string()
        }
    }
}

#[derive(Clone)]
struct AssertMessage {
    message: String,
    args: Vec<String>,
}

fn get_message(args: &[NestedMeta]) -> Option<AssertMessage> {
    args.iter()
        .filter_map(|item| match item {
            NestedMeta::Meta(Meta::List(list)) => Some(list),
            _ => None,
        })
        .filter_map(|list| {
            list.path
                .segments
                .last()
                .filter(|seg| seg.ident == "message")
                .map(|_| list.nested.clone())
        })
        .find_map(|params| {
            let mut iter = params.into_iter();
            match iter.next() {
                // The first item in our param list should be the message string literal
                Some(NestedMeta::Lit(Lit::Str(str))) => Some(str.value()),
                _ => None,
            }
            .map(|message| {
                // And the rest are message args
                let args = iter
                    .filter_map(|nested_meta| match nested_meta {
                        NestedMeta::Meta(Meta::Path(path)) => path.segments.last().cloned(),
                        _ => None,
                    })
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<_>>();

                let message = if args.is_empty() {
                    format!(", \"{}\"", message)
                } else {
                    let used_args = args
                        .iter()
                        .filter(|arg| *arg != "_")
                        .map(|arg| format!("{}={}", arg, arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(", \"{}\", {}", message, used_args)
                };

                AssertMessage { message, args }
            })
        })
}
