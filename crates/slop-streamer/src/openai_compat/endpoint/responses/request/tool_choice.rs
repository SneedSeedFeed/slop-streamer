use serde::{Deserialize, Serialize};

use crate::define_contract;

mod private {
    pub trait Sealed {}
}

pub trait ToolChoice: private::Sealed + erased_serde::Serialize {
    fn as_erased(&self) -> &dyn erased_serde::Serialize
    where
        Self: Sized,
    {
        self
    }
}

pub trait AsToolChoice {
    fn erase_variant(&self) -> &dyn ToolChoice;
}

erased_serde::serialize_trait_object!(ToolChoice);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    None,
    Auto,
    Required,
}

impl std::fmt::Display for ToolChoiceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolChoiceMode::None => "none",
            ToolChoiceMode::Auto => "auto",
            ToolChoiceMode::Required => "required",
        }
        .fmt(f)
    }
}
impl private::Sealed for ToolChoiceMode {}

impl ToolChoice for ToolChoiceMode {
    fn as_erased(&self) -> &dyn erased_serde::Serialize
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceAllowedToolMode {
    Auto,
    Required,
}

impl std::fmt::Display for ToolChoiceAllowedToolMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolChoiceAllowedToolMode::Auto => "auto",
            ToolChoiceAllowedToolMode::Required => "required",
        }
        .fmt(f)
    }
}
impl private::Sealed for ToolChoiceAllowedToolMode {}

impl ToolChoice for ToolChoiceAllowedToolMode {
    fn as_erased(&self) -> &dyn erased_serde::Serialize
    where
        Self: Sized,
    {
        self
    }
}

define_contract!(
    pub trait ToolChoiceAllowedTools: ToolChoice + AsToolChoice => AllowedTools {
        [mode: ToolChoiceAllowedToolMode]
        const ["type": "allowed_tools"]
    }
);
