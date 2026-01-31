use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, LitStr, Token,
    parse::{Parse, ParseStream},
};

pub(crate) struct ConstField {
    pub(crate) key: LitStr,
    pub(crate) value: Expr,
}

impl ConstField {
    pub(crate) fn generate_serialize(&self) -> TokenStream {
        let const_key = &self.key;
        let const_value = &self.value;
        quote! {
            serde::ser::SerializeMap::serialize_entry(&mut map, #const_key, &#const_value)?;
        }
    }
}

impl Parse for ConstField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![const]>()?;
        let key = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;

        Ok(Self { key, value })
    }
}
