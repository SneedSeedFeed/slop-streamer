//! Responses input has 24 variants as of writing (not all mapped here), expecting a user to think about all 24 at once in a single struct sucks and in a single enum sounds like 24 tuple variants and 24 new structs that all need to be nicely named.
//! This is an experiment in 'trait based' API writing instead of type based.
//! You bring your own types (meaning you're free to use references and slices or whatever) and implement only the request items you actually use

use std::borrow::Cow;

use serde::{
    Deserialize, Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};

use crate::define_contract;

mod private {
    pub trait Sealed {}
}

pub trait InputItem: private::Sealed + erased_serde::Serialize {
    fn as_erased(&self) -> &dyn erased_serde::Serialize
    where
        Self: Sized,
    {
        self
    }
}

// Provides a way for users to have enums where all the members can be a valid InputItem, without trusting them to uphold the contract of InputItem themselves
pub trait AsInputItem {
    fn erase_variant(&self) -> &dyn InputItem;
}
erased_serde::serialize_trait_object!(InputItem);

pub trait InputItemCollection {
    fn serialize_items<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;

    /// Get the number of items (if known)
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

macro_rules! impl_input_item_collection {
    ($($ty:ty),*) => {
        $(
            impl<T> InputItemCollection for $ty
            where
                T: AsInputItem,
            {
                fn serialize_items<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    let mut seq = serializer.serialize_seq(self.size_hint())?;
                    for item in self.iter() {
                        seq.serialize_element(item.erase_variant())?;
                    }
                    seq.end()
                }

                fn size_hint(&self) -> Option<usize> {
                    Some(self.len())
                }
            }
        )*
    };
}

impl_input_item_collection!(
    Vec<T>,
    &[T],
    Box<[T]>,
    std::sync::Arc<[T]>,
    std::rc::Rc<[T]>,
    [T]
);

impl<T> InputItemCollection for &T
where
    T: InputItemCollection,
{
    fn serialize_items<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        <T as InputItemCollection>::serialize_items(self, serializer)
    }

    fn size_hint(&self) -> Option<usize> {
        <T as InputItemCollection>::size_hint(self)
    }
}

impl<T, const LEN: usize> InputItemCollection for [T; LEN]
where
    T: AsInputItem,
{
    fn serialize_items<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(self.size_hint())?;
        for item in self.iter() {
            seq.serialize_element(item.erase_variant())?;
        }
        seq.end()
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Developer,
}

// Note: Doesn't cover images
define_contract!(
    pub trait InputMessage: InputItem + AsInputItem => Message {
        content: Cow<'_, str>,
        role: Role
    }
);

impl InputMessage for str {
    fn content(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }

    fn role(&self) -> Role {
        Role::User
    }
}

impl InputMessage for String {
    fn content(&self) -> Cow<'_, str> {
        <str as InputMessage>::content(self)
    }
    fn role(&self) -> Role {
        Role::User
    }
}

impl AsInputItem for String {
    fn erase_variant(&self) -> &dyn InputItem {
        self.as_wrapper_ref()
    }
}

impl private::Sealed for String {}

impl InputItem for String {
    fn as_erased(&self) -> &dyn erased_serde::Serialize
    where
        Self: Sized,
    {
        self.as_wrapper_ref()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    InProgress,
    Completed,
    Incomplete,
}

define_contract!(
    pub trait InputFunctionCall: InputItem + AsInputItem => FunctionCall {
        [arguments: Cow<'_, str>, call_id: Cow<'_, str>, name: Cow<'_, str>, id: Option<Cow<'_, str>> = None, status: Option<Status> = None]
        const ["type": "function_call"]
    }
);

define_contract!(
    pub trait InputFunctionCallOutput: InputItem + AsInputItem => FunctionCallOutput {
        [call_id: Cow<'_, str>, output: Cow<'_, str>, id: Option<Cow<'_, str>> = None, status: Option<Status> = None]
        const ["type": "function_call_output"]
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapper_ref_transmute() {
        struct MyMsg {
            content: String,
        }

        impl InputMessage for MyMsg {
            fn content(&self) -> Cow<'_, str> {
                Cow::Borrowed(&self.content)
            }
            fn role(&self) -> Role {
                Role::User
            }
        }

        let msg = MyMsg {
            content: "test".into(),
        };
        let wrapper_ref: &Message<MyMsg> = msg.as_wrapper_ref();

        assert_eq!(
            &msg as *const MyMsg as usize,
            wrapper_ref as *const Message<MyMsg> as usize
        );

        // Verify serialization works
        let json = serde_json::to_string(wrapper_ref).unwrap();
        assert_eq!(&json, r#"{"content":"test","role":"user"}"#);
    }
}
