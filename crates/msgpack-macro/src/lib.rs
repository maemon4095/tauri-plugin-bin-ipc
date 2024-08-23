use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn bin_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    macro_impl::bin_command(attr.into(), item.into()).into()
}

#[proc_macro]
pub fn generate_handler(item: TokenStream) -> TokenStream {
    macro_impl::generate_handler(item.into()).into()
}
