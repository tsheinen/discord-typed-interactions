use quote::{ToTokens, TokenStreamExt};
use proc_macro2::{TokenStream, Ident, Span};

pub(crate) struct Defer<F: Fn() -> TokenStream>(pub F);

impl<F: Fn() -> TokenStream> ToTokens for Defer<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(std::iter::once(self.0()))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DeferredIdent<'a>(pub &'a str);

impl ToTokens for DeferredIdent<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new(self.0, Span::call_site()));
    }
}

pub(crate) struct DeferredConditional<F: Fn() -> TokenStream, G: Fn() -> TokenStream>(pub bool, pub F, pub G);

impl<F: Fn() -> TokenStream, G: Fn() -> TokenStream> ToTokens for DeferredConditional<F, G> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = if self.0 {
            self.1()
        } else {
            self.2()
        };
        tokens.extend(std::iter::once(stream))
    }
}
