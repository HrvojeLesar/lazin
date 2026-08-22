use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Signature, parse_macro_input, punctuated::Punctuated};

use crate::test_args::LazinTestArgs;

#[doc(hidden)]
mod asserts;
#[doc(hidden)]
mod test_args;

#[proc_macro_attribute]
pub fn lazin_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as LazinTestArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let visibility = &input_fn.vis;
    let signature = &input_fn.sig;
    let block = &input_fn.block;
    let attributes = &input_fn.attrs;
    let fn_args = &signature.inputs;

    let before_calls = args.before.iter().map(|call| {
        let path = &call.path;
        let call_args = call.args.iter().map(|arg| match arg {
            test_args::CallArg::Ident { mutability, ident } => {
                if *mutability {
                    quote! { &mut #ident }
                } else {
                    quote! { &#ident }
                }
            }

            test_args::CallArg::Literal(lit) => quote! { #lit },
            test_args::CallArg::Some(lit) => quote! { Some(#lit) },
            test_args::CallArg::None => quote! { None },
        });

        quote! { #path(#(#call_args),*); }
    });

    let after_calls = args.after.iter().map(|call| {
        let function = &call.path;
        quote! { #function(); }
    });

    let mut context_setup = Vec::new();

    for arg in fn_args {
        match arg {
            syn::FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;

                context_setup.push(quote! {
                    let #pat = ::lazin_test_utils::context::TestContextDropGuard::new(
                        <#ty as ::lazin_test_utils::context::TestContext>::setup()
                    );
                });
            }
            syn::FnArg::Receiver(receiver) => {
                return syn::Error::new_spanned(
                    receiver,
                    "lazin_test cannot be used on methods that take `self`",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    let teardown = quote! {
        struct __LazinTeardownGuard;
        impl ::core::ops::Drop for __LazinTeardownGuard {
            fn drop(&mut self) {
                #(#after_calls)*
            }
        }
        let _lazin_teardown_guard = __LazinTeardownGuard{};
    };

    let signature = strip_signature(signature);

    let assert_guard = quote! {
        let __lazin_assert_guard = ::lazin_test_utils::asserts_guard::AssertGuard::new();
    };

    quote! {
        #(#attributes)*
        #[test]
        #visibility #signature {
            #assert_guard
            #(#context_setup)*
            #(#before_calls)*
            #teardown
            #block
        }
    }
    .into()
}

fn strip_signature(signature: &Signature) -> Signature {
    let mut signature = signature.clone();
    signature.inputs = Punctuated::new();

    signature
}

/// Expected to be used in #\[lazin_test\], will silenty do nothing if not used.
/// If used in helper functions, the whole function chain should be annotated with #\[track_caller\].
///
/// # Example
///
/// ```
/// # use lazin_test_macros::lazin_assert_eq;
/// #[track_caller]
/// fn helper_fn() {
///     lazin_assert_eq!(true, false, "Glorious comparison");
/// }
/// ```
#[proc_macro]
pub fn lazin_assert_eq(input: TokenStream) -> TokenStream {
    asserts::lazin_assert_eq(input)
}

/// Expected to be used in #\[lazin_test\], will silenty do nothing if not used.
/// If used in helper functions, the whole function chain should be annotated with #\[track_caller\].
///
/// # Example
///
/// ```
/// # use lazin_test_macros::lazin_assert_ne;
/// #[track_caller]
/// fn helper_fn() {
///     lazin_assert_ne!(true, true, "Glorious comparison");
/// }
/// ```
#[proc_macro]
pub fn lazin_assert_ne(input: TokenStream) -> TokenStream {
    asserts::lazin_assert_ne(input)
}

/// Expected to be used in #\[lazin_test\], will silenty do nothing if not used.
/// If used in helper functions, the whole function chain should be annotated with #\[track_caller\].
///
/// # Example
///
/// ```
/// # use lazin_test_macros::lazin_assert;
/// #[track_caller]
/// fn helper_fn() {
///     lazin_assert!(true == false, "Glorious comparison");
/// }
/// ```
#[proc_macro]
pub fn lazin_assert(input: TokenStream) -> TokenStream {
    asserts::lazin_assert(input)
}
