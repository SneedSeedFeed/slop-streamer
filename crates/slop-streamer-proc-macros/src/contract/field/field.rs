use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Expr, Ident, LitStr, Meta, MetaList, Token, Type,
    parse::{Parse, ParseStream},
};

#[derive(Default)]
pub(crate) struct FieldAttributes {
    pub(crate) skip_serializing_if: Option<Expr>,
    pub(crate) rename: Option<LitStr>,
}

impl FieldAttributes {
    pub(crate) fn from_attributes(attrs: Vec<Attribute>) -> syn::Result<Self> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident("skip_serializing_if") {
                if result.skip_serializing_if.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "skip_serializing_if specified multiple times",
                    ));
                }
                result.skip_serializing_if = Some(parse_expr_attribute(&attr)?);
            } else if attr.path().is_ident("rename") {
                if result.rename.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "rename specified multiple times",
                    ));
                }
                result.rename = Some(parse_lit_str_attribute(&attr)?);
            }
        }

        Ok(result)
    }
}

pub(crate) fn parse_expr_attribute(attr: &Attribute) -> syn::Result<Expr> {
    let meta = &attr.meta;
    match meta {
        Meta::List(MetaList { tokens, .. }) => syn::parse2(tokens.clone()),
        _ => Err(syn::Error::new_spanned(
            attr,
            "expected #[skip_serializing_if(expr)]",
        )),
    }
}

pub(crate) fn parse_lit_str_attribute(attr: &Attribute) -> syn::Result<LitStr> {
    let meta = &attr.meta;
    match meta {
        Meta::List(MetaList { tokens, .. }) => syn::parse2(tokens.clone()),
        _ => Err(syn::Error::new_spanned(
            attr,
            "expected #[rename(\"name\")]",
        )),
    }
}

pub(crate) struct Field {
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) name: Ident,
    pub(crate) ty: Type,
    pub(crate) default: Option<Expr>,
    pub(crate) parsed_attrs: Option<FieldAttributes>,
}

impl Field {
    pub(crate) fn generate_contract_fn(&self) -> TokenStream {
        let Self {
            name, ty, default, ..
        } = &self;

        match default {
            Some(default) => quote! {
                fn #name(&self) -> #ty {
                    #default
                }
            },
            None => quote! {
                fn #name(&self) -> #ty;
            },
        }
    }

    pub(crate) fn is_ref_type(&self) -> bool {
        matches!(self.ty, Type::Reference(_))
    }

    pub(crate) fn generate_delegate_method(&self, trait_name: &Ident) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        quote! {
            fn #name(&self) -> #ty {
                <T as #trait_name>::#name(self)
            }
        }
    }

    pub(crate) fn generate_serialize(&self) -> TokenStream {
        let field_name = &self.name;

        let attrs = self.parsed_attrs.as_ref();

        let key = attrs
            .and_then(|a| a.rename.as_ref())
            .cloned()
            .unwrap_or_else(|| LitStr::new(&field_name.to_string(), field_name.span()));

        let val = if self.is_ref_type() {
            quote! {let val = self.0.#field_name();}
        } else {
            quote! {let val = &self.0.#field_name();}
        };

        match attrs.and_then(|a| a.skip_serializing_if.as_ref()) {
            Some(skip_if) => {
                quote! {
                    #val
                    if !#skip_if(val) {
                        serde::ser::SerializeMap::serialize_entry(&mut map, #key, val)?;
                    }
                }
            }
            None => quote! {
                #val
                serde::ser::SerializeMap::serialize_entry(&mut map, #key, val)?;
            },
        }
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let default = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            attrs,
            name,
            ty,
            default,
            parsed_attrs: None,
        })
    }
}
