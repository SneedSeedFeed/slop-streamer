use crate::openai_compat::endpoint::responses::request::Request;

#[macro_use]
pub(crate) mod const_str;
pub mod openai_compat;
pub mod tool;

fn test() {
    let request = Request {
        input: [String::from("")],
        model: String::from(""),
        parallel_tool_calls: true,
        stream: false,
    };

    let serialize = serde_json::to_string(&request);
}
