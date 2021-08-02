use quote::{ToTokens, TokenStreamExt};
use proc_macro2::{TokenStream, Ident, Span};

pub(crate) struct Defer<F: Fn() -> TokenStream>(pub F);

impl<F: Fn() -> TokenStream> ToTokens for Defer<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(std::iter::once(self.0()))
    }
}

pub(crate) struct DeferredIdent<'a>(pub &'a str);

impl ToTokens for DeferredIdent<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new(self.0, Span::call_site()));
    }
}

