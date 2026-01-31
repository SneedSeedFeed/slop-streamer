use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Ident, Meta, MetaList, Token, Visibility,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::contract::field::field::FieldAttributes;

pub(crate) mod field;

pub(crate) struct ContractTraitInput {
    pub(crate) vis: Visibility,
    pub(crate) trait_name: Ident,
    pub(crate) fields: Vec<field::Field>,
    pub(crate) const_fields: Vec<field::ConstField>,
    pub(crate) wrapper_name: Ident,
    pub(crate) super_trait: Ident,
    pub(crate) public_trait: Ident,
}

impl ContractTraitInput {
    pub(crate) fn size_hint(&self) -> Option<usize> {
        if self
            .fields
            .iter()
            .flat_map(|field| field.parsed_attrs.iter())
            .any(|attr| attr.skip_serializing_if.is_some())
        {
            None
        } else {
            Some(self.const_fields.len() + self.fields.len())
        }
    }

    pub(crate) fn generate_contract_trait(&self) -> TokenStream {
        let Self {
            vis,
            trait_name,
            fields,
            wrapper_name,
            ..
        } = self;
        let field_contracts = fields.iter().map(field::Field::generate_contract_fn);
        let field_delegates = fields
            .iter()
            .map(|field| field.generate_delegate_method(trait_name))
            .collect::<Vec<_>>();
        quote! {
            #vis trait #trait_name {
                #(
                    #field_contracts
                )*

                fn into_wrapped(self) -> #wrapper_name<Self>
                where
                    Self: Sized,
                {
                    #wrapper_name(self)
                }

                fn as_wrapped(&self) -> #wrapper_name<&Self> {
                    #wrapper_name(self)
                }

                /// Get a reference to this value as its wrapper type.
                /// This is safe because the wrapper is #[repr(transparent)].
                fn as_wrapper_ref(&self) -> &#wrapper_name<Self>
                where
                    Self: Sized,
                {
                    // Safety: #wrapper is #[repr(transparent)], so &Self and &#wrapper<Self>
                    // have identical layout and can be safely transmuted
                    unsafe {&*(std::ptr::from_ref(self) as *const #wrapper_name<Self>)}
                }
            }

            impl<T: #trait_name + ?Sized> #trait_name for &T {
                #(
                    #field_delegates
                )*
            }
            impl<T: #trait_name + ?Sized> #trait_name for &mut T {
                #(
                    #field_delegates
                )*
            }
            impl<T: #trait_name + ?Sized> #trait_name for Box<T> {
                #(
                    #field_delegates
                )*
            }
            impl<T: #trait_name + ?Sized> #trait_name for std::rc::Rc<T> {
                #(
                    #field_delegates
                )*
            }
            impl<T: #trait_name + ?Sized> #trait_name for std::sync::Arc<T> {
                #(
                    #field_delegates
                )*
            }
        }
    }

    pub(crate) fn generate_wrapper(&self) -> TokenStream {
        let Self {
            vis,
            trait_name,
            fields,
            const_fields,
            wrapper_name,
            super_trait,
            public_trait,
        } = self;

        let size_hint = self
            .size_hint()
            .map(|hint| quote! {Some(#hint)})
            .unwrap_or_else(|| quote! {None});

        let field_serialize = fields.iter().map(field::Field::generate_serialize);
        let const_serialize = const_fields
            .iter()
            .map(field::ConstField::generate_serialize);
        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[repr(transparent)]
            #vis struct #wrapper_name<T>(pub T);

            impl<T> #wrapper_name<T> {
                pub fn new(inner: T) -> Self {
                    Self(inner)
                }

                pub fn into_inner(self) -> T {
                    self.0
                }

                pub fn as_inner(&self) -> &T {
                    &self.0
                }
            }

            impl<T> AsRef<T> for #wrapper_name<T> {
                fn as_ref(&self) -> &T {
                    &self.0
                }
            }

            impl<T> std::ops::Deref for #wrapper_name<T> {
                type Target = T;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl<T> serde::Serialize for #wrapper_name<T>
                where T: #trait_name
            {
                fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    let mut map = serializer.serialize_map(#size_hint)?;
                    #(#field_serialize)*
                    #(#const_serialize)*
                    serde::ser::SerializeMap::end(map)
                }
            }

            impl<T> private::Sealed for #wrapper_name<T> where T: #trait_name {}

            impl<T: #trait_name> #super_trait for #wrapper_name<T> {
                fn as_erased(&self) -> &dyn erased_serde::Serialize {
                    self
                }
            }

            impl<T: #trait_name> #public_trait for #wrapper_name<T> {
                fn erase_variant(&self) -> &dyn #super_trait {
                    self
                }
            }
        }
    }

    pub(crate) fn generate_contract(&self) -> TokenStream {
        let trait_tokens = self.generate_contract_trait();
        let wrapper_tokens = self.generate_wrapper();
        quote! {
            #trait_tokens
            #wrapper_tokens
        }
    }
}

pub(crate) struct ContractAttributes {
    pub(crate) super_pub_trait_idents: Option<(Ident, Ident)>,
    pub(crate) wrapper_name: Option<Ident>,
}

/// Parse trait-level attributes to extract wrapper and impl_traits info
pub(crate) fn parse_trait_attributes(attrs: Vec<Attribute>) -> syn::Result<ContractAttributes> {
    let mut wrapper_name = None;
    let mut super_pub_trait_idents = None;

    for attr in attrs {
        if attr.path().is_ident("wrapper") {
            if wrapper_name.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "wrapper attribute specified multiple times",
                ));
            }
            wrapper_name = Some(parse_wrapper_attribute(&attr)?);
        } else if attr.path().is_ident("impl_traits") {
            if super_pub_trait_idents.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "impl_traits attribute specified multiple times",
                ));
            }
            super_pub_trait_idents = Some(parse_impl_traits_attribute(&attr)?);
        }
    }

    Ok(ContractAttributes {
        super_pub_trait_idents,
        wrapper_name,
    })
}

pub(crate) fn parse_wrapper_attribute(attr: &Attribute) -> syn::Result<Ident> {
    let meta = &attr.meta;
    match meta {
        Meta::List(MetaList { tokens, .. }) => syn::parse2(tokens.clone()),
        _ => Err(syn::Error::new_spanned(
            attr,
            "expected #[wrapper(WrapperName)]",
        )),
    }
}

pub(crate) fn parse_impl_traits_attribute(attr: &Attribute) -> syn::Result<(Ident, Ident)> {
    let meta = &attr.meta;
    match meta {
        Meta::List(MetaList { tokens, .. }) => {
            let parser = Punctuated::<Ident, Token![,]>::parse_separated_nonempty;
            let traits: Punctuated<Ident, Token![,]> =
                syn::parse::Parser::parse2(parser, tokens.clone())?;

            if traits.len() != 2 {
                return Err(syn::Error::new_spanned(
                    attr,
                    "impl_traits attribute requires exactly 2 traits: #[impl_traits(SuperTrait, PublicTrait)]",
                ));
            }

            let mut iter = traits.into_iter();
            let super_trait = iter.next().unwrap();
            let public_trait = iter.next().unwrap();

            Ok((super_trait, public_trait))
        }
        _ => Err(syn::Error::new_spanned(
            attr,
            "expected #[impl_traits(SuperTrait, PublicTrait)]",
        )),
    }
}

impl Parse for ContractTraitInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let vis: Visibility = input.parse()?;
        input.parse::<Token![trait]>()?;
        let trait_name: Ident = input.parse()?;

        let ContractAttributes {
            super_pub_trait_idents,
            wrapper_name,
        } = parse_trait_attributes(attrs)?;

        let content;
        syn::braced!(content in input);

        let mut fields = Vec::new();
        let mut const_fields = Vec::new();

        while !content.is_empty() {
            if content.peek(Token![const]) {
                const_fields.push(content.parse()?);
            } else {
                let mut field: field::Field = content.parse()?;
                field.parsed_attrs = Some(FieldAttributes::from_attributes(field.attrs.clone())?);
                fields.push(field);
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        let wrapper_name = wrapper_name.unwrap_or_else(|| format_ident!("{}Wrapper", trait_name));

        let (super_trait, public_trait) = super_pub_trait_idents.ok_or_else(|| {
            syn::Error::new(
                trait_name.span(),
                "Missing #[impl_traits(SuperTrait, PublicTrait)] attribute on trait",
            )
        })?;

        Ok(ContractTraitInput {
            vis,
            trait_name,
            fields,
            const_fields,
            wrapper_name,
            super_trait,
            public_trait,
        })
    }
}
