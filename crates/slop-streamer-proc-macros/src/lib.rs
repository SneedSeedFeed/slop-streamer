use proc_macro::TokenStream;

pub(crate) mod contract;
pub(crate) mod contract_group;

/// Define a contract trait with an associated wrapper type
///
/// # Syntax
/// ```ignore
/// contract_trait! {
///     #[wrapper(Message)]
///     #[impl_traits(InputItem, AsInputItem)]
///     pub trait MyContract {
///         // Required fields
///         field1: Type1,
///         field2: Type2,
///         
///         // Optional fields with defaults
///         #[skip_serializing_if(is_none)]
///         field3: Option<Type3> = None,
///         
///         #[rename("custom_name")]
///         field4: Type4 = default_value(),
///         
///         // Constant fields (always present in serialization)
///         const "type" = "my_type",
///         const "version" = 1,
///     }
/// }
/// ```
#[proc_macro]
pub fn contract_trait(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as contract::ContractTraitInput);
    input.generate_contract().into()
}
