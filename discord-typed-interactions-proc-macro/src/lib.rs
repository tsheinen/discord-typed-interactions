extern crate proc_macro;
use proc_macro::TokenStream;

use proc_macro::TokenTree;
use quote::{quote, ToTokens};
use syn::LitStr;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro]
pub fn generate_structs(input: TokenStream) -> TokenStream {
    discord_typed_interactions::structify(
        &std::fs::read_to_string(parse_macro_input!(input as LitStr).value())
            .expect("provided file should be readable"),
    )
    .into()
}
