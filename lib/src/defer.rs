use quote::{ToTokens, TokenStreamExt};
use proc_macro2::{TokenStream, Ident, Span};

#[derive(Clone, Copy)]
pub(crate) struct Defer<T>(pub T);

impl<F: Fn() -> TokenStream> ToTokens for Defer<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(std::iter::once(self.0()))
    }
}

impl ToTokens for Defer<&str> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new(self.0, Span::call_site()));
    }
}
