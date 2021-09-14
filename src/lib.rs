extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta, ReturnType, Type};

#[proc_macro_attribute]
pub fn assert_eq_fn(args: TokenStream, item: TokenStream) -> TokenStream {
    create_assert_macro(AssertType::Eq, args, item)
}

#[proc_macro_attribute]
pub fn assert_fn(args: TokenStream, item: TokenStream) -> TokenStream {
    create_assert_macro(AssertType::True, args, item)
}

enum AssertType {
    True,
    Eq,
}

fn create_assert_macro(
    assert_type: AssertType,
    args: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let raw_item = item.clone();
    let item = parse_macro_input!(item as ItemFn);

    let args = parse_macro_input!(args as AttributeArgs);

    let fn_name = item.sig.ident.to_string();
    let (params, values) = get_values_and_params(&item);
    let (async_block, dot_await) = get_async(&item);
    let (if_result_open, if_result_close) = get_result_block(&item);
    let assert_call = get_assert_call(&assert_type);
    let message = get_message(&args);

    format!(
        r#"
        #[macro_export]
        macro_rules! assert_{fn_name} {{
            ({params_trimmed}$(,)?) => {{ {async_block} {{
                let result = {fn_name}({values}){dot_await};

                {if_result_open}
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
        if_result_open = if_result_open,
        if_result_close = if_result_close,
        assert_call = assert_call,
        message = message,
        original_fn = raw_item.to_string()
    )
    .parse()
    .expect("Generated invalid tokens")
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

fn get_result_block(item: &ItemFn) -> (String, String) {
    if match &item.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, type_) => match *type_.clone() {
            Type::Path(path) => path.path.segments.last().unwrap().ident == "Result",
            _ => false,
        },
    } {
        ("if let Ok(result) = result {".to_string(), "}".to_string())
    } else {
        ("".to_string(), "".to_string())
    }
}

fn get_assert_call(assert_type: &AssertType) -> String {
    match assert_type {
        AssertType::True => "assert!(result".to_string(),
        AssertType::Eq => "assert_eq!(result.0, result.1".to_string(),
    }
}

fn get_message(args: &AttributeArgs) -> String {
    let message = args
        .iter()
        .filter_map(|item| match item {
            NestedMeta::Meta(meta) => Some(meta),
            NestedMeta::Lit(_) => None,
        })
        .filter_map(|meta| match meta {
            Meta::NameValue(name_value) => Some(name_value),
            _ => None,
        })
        .filter_map(|name_value| {
            if let Some(seg) = name_value.path.segments.last() {
                if seg.ident.to_string() == "message" {
                    Some(name_value.clone().lit)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .find_map(|lit| match lit {
            Lit::Str(value) => Some(value.value()),
            _ => None
        });

    message
        .map(|str| format!(", \"{}\"", str))
        .unwrap_or_default()
}
