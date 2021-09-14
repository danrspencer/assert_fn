extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, ReturnType, Type};

#[proc_macro_attribute]
pub fn assert_eq(_: TokenStream, raw_item: TokenStream) -> TokenStream {
    create_assert_macro(AssertType::Eq, raw_item)
}

#[proc_macro_attribute]
pub fn assert(_: TokenStream, raw_item: TokenStream) -> TokenStream {
    create_assert_macro(AssertType::True, raw_item)
}

enum AssertType {
    True,
    Eq,
}

fn create_assert_macro(assert_type: AssertType, raw_item: TokenStream) -> TokenStream {
    let item = raw_item.clone();
    let item = parse_macro_input!(item as ItemFn);

    let fn_name = item.sig.ident.to_string();
    let (params, values) = item.sig.inputs.into_iter().enumerate().fold(
        ("".to_string(), "".to_string()),
        |(params, values), (n, _)| {
            (
                format!("{}$arg_{}:expr,", params, n),
                format!("{}$arg_{},", values, n),
            )
        },
    );

    let (async_block, dot_await) = if item.sig.asyncness.is_some() {
        ("async", ".await")
    } else {
        ("", "")
    };

    let (if_result_open, if_result_close) = if match item.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, type_) => match *type_ {
            Type::Path(path) => path.path.segments.last().unwrap().ident == "Result",
            _ => false,
        },
    } {
        ("if let Ok(result) = result {", "}")
    } else {
        ("", "")
    };

    let assert_call = match assert_type {
        AssertType::True => "assert!(result",
        AssertType::Eq => "assert_eq!(result.0, result.1",
    };

    format!(
        r#"
        #[macro_export]
        macro_rules! assert_{fn_name} {{
            ({params_trimmed}$(,)?) => {{ {async_block} {{
                let result = {fn_name}({values}){dot_await};

                {if_result_open}
                {assert_call});
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
        async_block = async_block,
        dot_await = dot_await,
        if_result_open = if_result_open,
        if_result_close = if_result_close,
        assert_call = assert_call,
        params = params,
        params_trimmed = params.trim_end_matches(|c| c == ','),
        values = values.trim_end_matches(|c| c == ','),
        original_fn = raw_item.to_string()
    )
    .parse()
    .expect("Generated invalid tokens")
}
