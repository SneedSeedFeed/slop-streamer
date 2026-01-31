use serde::{Serialize, Serializer};

pub mod input_type;
pub mod tool_choice;

#[derive(Debug, Clone, Serialize)]
pub struct Request<I, M> {
    #[serde(
        bound(serialize = "I: input_type::InputItemCollection"),
        serialize_with = "input_type::InputItemCollection::serialize_items"
    )]
    pub input: I,
    #[serde(
        bound(serialize = "M: AsRef<str>"),
        serialize_with = "serialize_as_ref_str"
    )]
    pub model: M,
    pub parallel_tool_calls: bool,
    pub stream: bool,
}

fn serialize_as_ref_str<T: AsRef<str>, S: Serializer>(t: &T, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(t.as_ref())
}
