use quote::ToTokens;
use proc_macro2::TokenStream;

pub(crate) struct Defer<F: Fn() -> TokenStream>(pub F);

impl<F: Fn() -> TokenStream> ToTokens for Defer<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(std::iter::once(self.0()))
    }
}
