use proc_macro::TokenStream;

use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::LitStr;
use syn::Token;
#[proc_macro]
pub fn typify(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<LitStr, Token![,]>::parse_separated_nonempty;

    discord_typed_interactions_lib::typify_driver(
        parser
            .parse(input.clone())
            .unwrap()
            .into_iter()
            .map(|x| std::fs::read_to_string(x.value()).expect("provided file should be readable")),
        None,
    )
    .into()
}
