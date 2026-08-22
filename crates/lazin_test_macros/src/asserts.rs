use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct AssertEqualityInput {
    left: Expr,
    right: Expr,
    args: Option<syn::punctuated::Punctuated<Expr, Token![,]>>,
}

impl Parse for AssertEqualityInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let left: Expr = input.parse()?;
        input.parse::<Token![,]>()?;

        let right: Expr = input.parse()?;

        let args = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            Some(syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated(input)?)
        } else {
            None
        };

        Ok(Self { left, right, args })
    }
}

pub fn lazin_assert_eq(input: TokenStream) -> TokenStream {
    let AssertEqualityInput { left, right, args } =
        parse_macro_input!(input as AssertEqualityInput);

    let message = match args {
        Some(args) => quote! {
            format!(
                "eq assertion failed: `{}`\n  left: {:#?}\n right: {:#?}",
                format!(#args),
                left_val,
                right_val,
            )
        },
        None => quote! {
            format!(
                "eq assertion failed:\n  left: {:#?}\n right: {:#?}",
                left_val,
                right_val,
            )
        },
    };

    quote! {
        match (&#left, &#right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let __lazin_loc = ::std::panic::Location::caller();
                    ::lazin_test_utils::asserts_guard::assert_failed(
                        format!("{}:{}: {}", __lazin_loc.file(), __lazin_loc.line(), #message)
                    );
                }
            }
        }
    }
    .into()
}

pub fn lazin_assert_ne(input: TokenStream) -> TokenStream {
    let AssertEqualityInput { left, right, args } =
        parse_macro_input!(input as AssertEqualityInput);

    let message = match args {
        Some(args) => quote! {
            format!(
                "ne assertion failed: `{}`\n  left: {:#?}\n right: {:#?}",
                format!(#args),
                left_val,
                right_val,
            )
        },
        None => quote! {
            format!(
                "ne assertion failed:\n  left: {:#?}\n right: {:#?}",
                left_val,
                right_val,
            )
        },
    };

    quote! {
        match (&#left, &#right) {
            (left_val, right_val) => {
                if *left_val == *right_val {
                    let __lazin_loc = ::std::panic::Location::caller();
                    ::lazin_test_utils::asserts_guard::assert_failed(
                        format!("{}:{}: {}", __lazin_loc.file(), __lazin_loc.line(), #message)
                    );
                }
            }
        }
    }
    .into()
}

struct AssertInput {
    cond: Expr,
    args: Option<syn::punctuated::Punctuated<Expr, Token![,]>>,
}

impl Parse for AssertInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cond: Expr = input.parse()?;

        let args = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            Some(syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated(input)?)
        } else {
            None
        };

        Ok(Self { cond, args })
    }
}

pub fn lazin_assert(input: TokenStream) -> TokenStream {
    let AssertInput { cond, args } = parse_macro_input!(input as AssertInput);

    let message = match args {
        Some(args) => quote! {
            format!(
                "assertion failed: `{}`\n  condition: {:#?}",
                format!(#args),
                cond_val,
            )
        },
        None => quote! {
            format!(
                "assertion failed:\n  condition: {:#?}",
                cond_val,
            )
        },
    };

    quote! {
        match #cond {
            cond_val => {
                if !cond_val {
                    let __lazin_loc = ::std::panic::Location::caller();
                    ::lazin_test_utils::asserts_guard::assert_failed(
                        format!("{}:{}: {}", __lazin_loc.file(), __lazin_loc.line(), #message)
                    );
                }
            }
        }
    }
    .into()
}
