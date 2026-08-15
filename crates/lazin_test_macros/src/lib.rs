use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Signature, parse_macro_input, punctuated::Punctuated};

use crate::test_args::LazinTestArgs;

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
        let call_args = call.args.iter().map(|arg| {
            let ident = &arg.ident;
            if arg.mutability {
                quote! { &mut #ident }
            } else {
                quote! { &#ident }
            }
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
                    let #pat = ::lazin_test_utils::context::TestContextDropGuard(
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

    quote! {
        #(#attributes)*
        #[test]
        #visibility #signature {
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
