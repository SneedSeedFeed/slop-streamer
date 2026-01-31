//! Module for the OAI Compatible responses endpoint

use std::sync::LazyLock;

use url::Url;

pub const OPENROUTER_RESPONSES_ENDPOINT: &str = "https://openrouter.ai/api/v1/responses";
pub static OPENROUTER_RESPONSES_URL: LazyLock<Url> =
    LazyLock::new(|| OPENROUTER_RESPONSES_ENDPOINT.parse().unwrap());

pub mod contract_macro;
pub mod request;
pub mod stream;
