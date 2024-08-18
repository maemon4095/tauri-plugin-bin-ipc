use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    macro_impl::command(attr.into(), item.into()).into()
}
