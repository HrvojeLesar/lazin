use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use crate::test_args::LazinTestArgs;

mod test_args;

#[proc_macro_attribute]
pub fn lazin_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as LazinTestArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let visibility = &input_fn.vis;
    let signature = &input_fn.sig;
    let block = &input_fn.block;

    let before_calls = args.before.iter().map(|f| quote! { #f(); });
    let after_calls = args.after.iter().map(|f| quote! { #f(); });

    let teardown = if args.after.is_empty() {
        quote! {}
    } else {
        quote! {
            struct __LazinTeardownGuard;
            impl ::core::ops::Drop for __LazinTeardownGuard {
                fn drop(&mut self) {
                    #(#after_calls)*
                }
            }
            let _lazin_teardown_guard = __LazinTeardownGuard;
        }
    };

    quote! {
        #[test]
        #visibility #signature {
            #(#before_calls)*
            #teardown
            #block
        }
    }
    .into()
}
