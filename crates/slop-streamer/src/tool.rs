use schemars::JsonSchema;

pub trait Tool: Send + Sync {
    type Input: JsonSchema;
}
