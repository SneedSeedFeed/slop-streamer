use std::borrow::Cow;

use bytes::Bytes;
use bytes_utils::Str;
use serde::{Deserialize, Serialize};

const_str!(pub struct ResponseStr("response"));
const_str!(pub struct MessageStr("message"));
const_str!(pub struct ReasoningStr("reasoning"));
const_str!(pub struct FunctionCallStr("function_call"));
const_str!(pub struct OutputTextStr("output_text"));
const_str!(pub struct ReasoningTextStr("reasoning_text"));
const_str!(pub struct SummaryTextStr("summary_text"));
const_str!(pub struct UrlCitationStr("url_citation"));

// Enums for fields with multiple discrete values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    InProgress,
    Completed,
}

// Main event types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent<T = Str> {
    #[serde(rename = "response.created")]
    ResponseCreated(ResponseCreatedData),

    #[serde(rename = "response.in_progress")]
    ResponseInProgress(ResponseInProgressData),

    #[serde(rename = "response.output_item.added")]
    ResponseOutputItemAdded(OutputItemAddedData<T>),

    #[serde(rename = "response.content_part.added")]
    ResponseContentPartAdded(ContentPartAddedData<T>),

    #[serde(rename = "response.output_text.delta")]
    ResponseOutputTextDelta(OutputTextDeltaData<T>),

    #[serde(rename = "response.output_text.annotation.added")]
    ResponseOutputTextAnnotationAdded(AnnotationAddedData<T>),

    #[serde(rename = "response.output_text.done")]
    ResponseOutputTextDone(OutputTextDoneData<T>),

    #[serde(rename = "response.content_part.done")]
    ResponseContentPartDone(ContentPartDoneData<T>),

    #[serde(rename = "response.output_item.done")]
    ResponseOutputItemDone(OutputItemDoneData<T>),

    #[serde(rename = "response.function_call_arguments.delta")]
    ResponseFunctionCallArgumentsDelta(FunctionCallArgumentsDeltaData<T>),

    #[serde(rename = "response.function_call_arguments.done")]
    ResponseFunctionCallArgumentsDone(FunctionCallArgumentsDoneData<T>),

    #[serde(rename = "response.reasoning_text.delta")]
    ResponseReasoningTextDelta(ReasoningTextDeltaData<T>),

    #[serde(rename = "response.reasoning_text.done")]
    ResponseReasoningTextDone(ReasoningTextDoneData<T>),

    #[serde(rename = "response.reasoning_summary_part.added")]
    ResponseReasoningSummaryPartAdded(ReasoningSummaryPartAddedData<T>),

    #[serde(rename = "response.reasoning_summary_text.delta")]
    ResponseReasoningSummaryTextDelta(ReasoningSummaryTextDeltaData<T>),

    #[serde(rename = "response.reasoning_summary_text.done")]
    ResponseReasoningSummaryTextDone(ReasoningSummaryTextDoneData<T>),

    #[serde(rename = "response.reasoning_summary_part.done")]
    ResponseReasoningSummaryPartDone(ReasoningSummaryPartDoneData<T>),

    #[serde(rename = "response.completed")]
    ResponseCompleted(ResponseCompletedData),
}

// Response lifecycle events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseCreatedData {
    pub response: ResponseMetadata,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseInProgressData {
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseCompletedData {
    pub response: ResponseMetadata,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseMetadata<T = Str> {
    pub id: T,
    pub status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageInfo {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
}

// Output item events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputItemAddedData<T = Str> {
    pub output_index: u32,
    pub item: OutputItem<T>,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputItemDoneData<T = Str> {
    pub output_index: u32,
    pub item: OutputItem<T>,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputItem<T = Str> {
    Message(MessageItem<T>),
    Reasoning(ReasoningItem<T>),
    FunctionCall(FunctionCallItem<T>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageItem<T = Str> {
    pub id: T,
    pub status: ItemStatus,
    #[serde(default = "Vec::new")]
    pub content: Vec<ContentPart<T>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningItem<T = Str> {
    pub id: T,
    #[serde(default = "Vec::new")]
    pub summary: Vec<SummaryPart<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallItem<T = Str> {
    pub call_id: T,
    pub name: T,
    pub arguments: T,
    pub status: ItemStatus,
}

// Content part events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentPartAddedData<T = Str> {
    pub item_id: T,
    pub output_index: u32,
    pub content_index: u32,
    pub part: ContentPart<T>,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentPartDoneData<T = Str> {
    pub item_id: T,
    pub output_index: u32,
    pub content_index: u32,
    pub part: ContentPart<T>,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart<T = Str> {
    OutputText(OutputTextPart<T>),
    ReasoningText(ReasoningTextPart<T>),
    SummaryText(SummaryTextPart<T>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputTextPart<T = Str> {
    pub text: T,
    #[serde(default = "Vec::new")]
    pub annotations: Vec<Annotation<T>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTextPart<T = Str> {
    pub text: T,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryPart<T = Str> {
    #[serde(flatten)]
    pub content: SummaryContent<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SummaryContent<T = Str> {
    SummaryText(SummaryTextPart<T>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryTextPart<T = Str> {
    pub text: T,
}

// Text delta events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputTextDeltaData<T = Str> {
    pub output_index: u32,
    pub item_id: T,
    pub content_index: u32,
    pub delta: T,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputTextDoneData<T = Str> {
    pub item_id: T,
    pub output_index: u32,
    pub content_index: u32,
    pub text: T,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTextDeltaData<T = Str> {
    pub output_index: u32,
    pub item_id: T,
    pub content_index: u32,
    pub delta: T,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTextDoneData<T = Str> {
    pub output_index: u32,
    pub item_id: T,
    pub content_index: u32,
    pub text: T,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningSummaryPartAddedData<T = Str> {
    pub output_index: u32,
    pub item_id: T,
    pub summary_index: u32,
    pub part: SummaryPart<T>,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningSummaryTextDeltaData<T = Str> {
    pub item_id: T,
    pub output_index: u32,
    pub summary_index: u32,
    pub delta: T,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningSummaryTextDoneData<T = Str> {
    pub output_index: u32,
    pub item_id: T,
    pub summary_index: u32,
    pub text: T,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningSummaryPartDoneData<T = Str> {
    pub output_index: u32,
    pub item_id: T,
    pub summary_index: u32,
    pub part: SummaryPart<T>,
    pub sequence_number: u64,
}

// Function call events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallArgumentsDeltaData<T = Str> {
    pub item_id: T,
    pub output_index: u32,
    pub delta: T,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallArgumentsDoneData<T = Str> {
    pub item_id: T,
    pub output_index: u32,
    pub name: T,
    pub arguments: T,
    pub sequence_number: u64,
}

// Annotation events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationAddedData<T = Str> {
    pub output_index: u32,
    pub item_id: T,
    pub content_index: u32,
    pub sequence_number: u64,
    pub annotation_index: u32,
    pub annotation: Annotation<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation<T = Str> {
    UrlCitation(UrlCitation<T>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlCitation<T = Str> {
    pub url: T,
    pub title: T,
    pub start_index: u32,
    pub end_index: u32,
}

pub(crate) trait ConvertToOwned {
    type Owned;
    fn convert_to_owned(self, buf: &Str) -> Self::Owned;
}

impl ConvertToOwned for Cow<'_, str> {
    type Owned = Str;

    fn convert_to_owned(self, buf: &Str) -> Str {
        match self {
            Cow::Borrowed(subslice) => buf.slice_ref(subslice),
            Cow::Owned(owned) => {
                // bytes::Bytes has an efficient implementation of From<Vec<u8>> that can avoid extra allocations that Str lacks an equivalent of for String
                let vec = owned.into_bytes();
                let bytes = Bytes::from(vec);
                // Safety: This Bytes was created from a vec that was unmodified since being taken from a String, thus its bytes are valid UTF-8
                unsafe { Str::from_inner_unchecked(bytes) }
            }
        }
    }
}

impl<T> ConvertToOwned for Vec<T>
where
    T: ConvertToOwned,
{
    type Owned = Vec<<T as ConvertToOwned>::Owned>;

    fn convert_to_owned(self, buf: &Str) -> Vec<<T as ConvertToOwned>::Owned> {
        self.into_iter()
            .map(|v| ConvertToOwned::convert_to_owned(v, buf))
            .collect()
    }
}

impl<T> ConvertToOwned for Option<T>
where
    T: ConvertToOwned,
{
    type Owned = Option<<T as ConvertToOwned>::Owned>;

    fn convert_to_owned(self, buf: &Str) -> Self::Owned {
        self.map(|v| v.convert_to_owned(buf))
    }
}

macro_rules! impl_conversion {
    ($target:ident struct [$($target_field:ident),*] [$($other_field:ident),*]) => {
        impl ConvertToOwned for $target<Cow<'_, str>> {
            type Owned = $target;
            fn convert_to_owned(self, buf: &Str) -> $target {
                $target {
                    $(
                        $target_field: self.$target_field.convert_to_owned(buf),
                    )*
                    $(
                        $other_field: self.$other_field,
                    )*
                }
            }
        }
    };
    ($target:ident enum [$($tuple_variant:ident),*] [$($unit_variant:ident),*]) => {
        impl ConvertToOwned for $target<Cow<'_, str>> {
            type Owned = $target;
            fn convert_to_owned(self, buf: &Str) -> $target {
                match self {
                    $(
                        Self::$tuple_variant(val) => $target::$tuple_variant(val.convert_to_owned(buf)),
                    )*
                    $(
                        Self::$unit_variant => $target::$unit_variant,
                    )*
                }
            }
        }
    };
    ($target:ident) => {
        impl ConvertToOwned for $target {
            type Owned = $target;
            fn convert_to_owned(self, _: &Str) -> $target {
                self
            }
        }
    }
}

// Enums for fields with multiple discrete values
impl_conversion!(ResponseStatus);
impl_conversion!(ItemStatus);

impl_conversion!(StreamEvent enum [
    ResponseCreated,
    ResponseInProgress,
    ResponseOutputItemAdded,
    ResponseContentPartAdded,
    ResponseOutputTextDelta,
    ResponseOutputTextAnnotationAdded,
    ResponseOutputTextDone,
    ResponseContentPartDone,
    ResponseOutputItemDone,
    ResponseFunctionCallArgumentsDelta,
    ResponseFunctionCallArgumentsDone,
    ResponseReasoningTextDelta,
    ResponseReasoningTextDone,
    ResponseReasoningSummaryPartAdded,
    ResponseReasoningSummaryTextDelta,
    ResponseReasoningSummaryTextDone,
    ResponseReasoningSummaryPartDone,
    ResponseCompleted
] []);

// Response lifecycle events
impl_conversion!(ResponseCreatedData);
impl_conversion!(ResponseInProgressData);
impl_conversion!(ResponseCompletedData);
impl_conversion!(ResponseMetadata);
impl_conversion!(UsageInfo);

// Output item events
impl_conversion!(OutputItemAddedData struct [item] [output_index, sequence_number]);
impl_conversion!(OutputItemDoneData struct [item] [output_index, sequence_number]);

impl_conversion!(OutputItem enum [Message, Reasoning, FunctionCall] []);
impl_conversion!(MessageItem struct [id, content] [status]);
impl_conversion!(ReasoningItem struct [id, summary, encrypted_content] []);
impl_conversion!(FunctionCallItem struct [call_id, name, arguments] [status]);

// Content part events
impl_conversion!(ContentPartAddedData struct [item_id, part] [output_index, content_index, sequence_number]);
impl_conversion!(ContentPartDoneData struct [item_id, part] [output_index, content_index, sequence_number]);

impl_conversion!(ContentPart enum [OutputText, ReasoningText, SummaryText] []);
impl_conversion!(OutputTextPart struct [text, annotations] []);
impl_conversion!(ReasoningTextPart struct [text] []);
impl_conversion!(SummaryPart struct [content] []);
impl_conversion!(SummaryContent enum [SummaryText] []);
impl_conversion!(SummaryTextPart struct [text] []);

// Text delta events
impl_conversion!(OutputTextDeltaData struct [item_id, delta] [output_index, content_index, sequence_number]);
impl_conversion!(OutputTextDoneData struct [item_id, text] [output_index, content_index, sequence_number]);
impl_conversion!(ReasoningTextDeltaData struct [item_id, delta] [output_index, content_index, sequence_number]);
impl_conversion!(ReasoningTextDoneData struct [item_id, text] [output_index, content_index, sequence_number]);
impl_conversion!(ReasoningSummaryPartAddedData struct [item_id, part] [output_index, summary_index, sequence_number]);
impl_conversion!(ReasoningSummaryTextDeltaData struct [item_id, delta] [output_index, summary_index, sequence_number]);
impl_conversion!(ReasoningSummaryTextDoneData struct [item_id, text] [output_index, summary_index, sequence_number]);
impl_conversion!(ReasoningSummaryPartDoneData struct [item_id, part] [output_index, summary_index, sequence_number]);

// Function call events
impl_conversion!(FunctionCallArgumentsDeltaData struct [item_id, delta] [output_index, sequence_number]);
impl_conversion!(FunctionCallArgumentsDoneData struct [item_id, name, arguments] [output_index, sequence_number]);

// Annotation events
impl_conversion!(AnnotationAddedData struct [item_id, annotation] [output_index, content_index, sequence_number, annotation_index]);
impl_conversion!(Annotation enum [UrlCitation] []);
impl_conversion!(UrlCitation struct [url, title] [start_index, end_index]);
