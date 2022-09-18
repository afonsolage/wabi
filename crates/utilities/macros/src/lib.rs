use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{parse::Parse, parse_macro_input, Generics};

pub(crate) struct ProxyDef {
    pub type_name: Ident,
    pub generics: Generics,
}

impl Parse for ProxyDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let type_name = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>()?;

        Ok(Self {
            type_name,
            generics,
        })
    }
}

#[proc_macro]
pub fn proxy(input: TokenStream) -> TokenStream {
    println!("{:?}", input);

    TokenStream::default()
}
