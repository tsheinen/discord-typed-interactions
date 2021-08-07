use proc_macro::TokenStream;

use syn::LitStr;
use syn::parse_macro_input;

#[proc_macro]
pub fn typify(input: TokenStream) -> TokenStream {
    discord_typed_interactions_lib::typify_driver(
        &std::fs::read_to_string(parse_macro_input!(input as LitStr).value())
            .expect("provided file should be readable"),
        None
    )
    .into()
}
