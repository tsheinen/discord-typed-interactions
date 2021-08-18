use quote::{ToTokens, TokenStreamExt};
use proc_macro2::{TokenStream, Ident, Span};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy)]
pub(crate) struct Defer<T>(pub T);

impl<F: Fn() -> TokenStream> ToTokens for Defer<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(std::iter::once(self.0()))
    }
}

impl<F: Fn() -> TokenStream> ToTokens for Defer<(bool, F)> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.0 {
            tokens.extend(std::iter::once(self.0.1()))
        }
    }
}

impl ToTokens for Defer<&str> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new(self.0, Span::call_site()));
    }
}

impl<T: Display> Display for Defer<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
