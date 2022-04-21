extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[derive(Clone, Copy)]
enum EntryMode {
    DEFAULT,
    DEBUG,
}
impl Default for EntryMode {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[proc_macro_attribute]
pub fn fluffl(attr: TokenStream, input: TokenStream) -> TokenStream {
    // checks if debug is detected, the entry point will
    // initalize panic hooks for the wasm entry point
    let mode = attr
        .into_iter()
        .next()
        .map(|tok| tok.to_string().to_lowercase())
        .map(|val| match val.as_str() {
            "debug" => EntryMode::DEBUG,
            _ => EntryMode::DEFAULT,
        })
        .unwrap_or_default();

    // parse the function into an AST so to make manipulation easier
    let function = parse_macro_input!(input as ItemFn);
    let function_body = function.block;
    let return_type = function.sig.output;

    if function.sig.asyncness.is_none() {
        panic!("Function must be async");
    }

    let debug_quote_wasm = if let EntryMode::DEBUG = mode {
        quote! {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        }
    } else {
        quote! { /* STUB */  }
    };
    
    let expanded = quote! {
        pub fn main() #return_type {
            //wasm entry point
            #[cfg(all(target_family = "wasm", not(target_os = "wasi")))]{
                #debug_quote_wasm
                let r = wasm_bindgen_futures::spawn_local(async{
                    #function_body
                });
                return r;
            }
            //desktop entry point
            #[cfg(not(all(target_family = "wasm", not(target_os = "wasi"))))]{
                let r = tokio::runtime::Runtime::new().expect("failed to create tokio runtime").block_on(async {
                    #function_body
                });
                return r;
            };
        }
    };
    expanded.into()
}
