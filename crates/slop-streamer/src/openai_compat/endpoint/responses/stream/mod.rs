use std::{
    borrow::Cow,
    pin::Pin,
    str::Utf8Error,
    task::{Context, Poll},
};

use futures::Stream;

use sseer::errors::EventStreamError;

use crate::openai_compat::endpoint::responses::stream::stream_item::{ConvertToOwned, StreamEvent};

pub mod stream_item;

pin_project_lite::pin_project! {

    #[project = OAICompatResponsesStreamProjection]
    #[derive(Debug)]
    pub struct OAICompatResponsesStream<S> {
        #[pin]
        state: OAICompatResponsesStreamState<S>
    }
}

impl<S> OAICompatResponsesStream<S> {
    pub fn new(stream: S) -> Self
    where
        S: Stream,
    {
        Self {
            state: OAICompatResponsesStreamState::Active {
                stream: sseer::EventStream::new(stream),
            },
        }
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    #[project = OAICompatResponsesStreamStateProjection]
    enum OAICompatResponsesStreamState<S> {
        Active {
            #[pin]
            stream: sseer::EventStream<S>
        },
        Terminated
    }
}

#[derive(Debug)]
pub enum OAICompatResponsesStreamError<E> {
    Transport(E),
    Utf8Error(Utf8Error),
    Deserialize(serde_json::Error),
}

impl<E> From<serde_json::Error> for OAICompatResponsesStreamError<E> {
    fn from(value: serde_json::Error) -> Self {
        Self::Deserialize(value)
    }
}

impl<E> From<EventStreamError<E>> for OAICompatResponsesStreamError<E> {
    fn from(value: EventStreamError<E>) -> Self {
        match value {
            EventStreamError::Transport(e) => Self::Transport(e),
            EventStreamError::Utf8Error(e) => Self::Utf8Error(e),
        }
    }
}

impl<E> std::fmt::Display for OAICompatResponsesStreamError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAICompatResponsesStreamError::Transport(e) => e.fmt(f),
            OAICompatResponsesStreamError::Utf8Error(utf8_error) => utf8_error.fmt(f),
            OAICompatResponsesStreamError::Deserialize(error) => error.fmt(f),
        }
    }
}

impl<E> std::error::Error for OAICompatResponsesStreamError<E> where E: std::error::Error {}

impl<S, B, E> Stream for OAICompatResponsesStream<S>
where
    S: Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
{
    type Item = Result<stream_item::StreamEvent, OAICompatResponsesStreamError<E>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let stream = match this.state.as_mut().project() {
            OAICompatResponsesStreamStateProjection::Active { stream } => stream,
            OAICompatResponsesStreamStateProjection::Terminated => return Poll::Ready(None),
        };

        let ev = match futures::ready!(stream.poll_next(cx)) {
            Some(Ok(ev)) => ev,
            Some(Err(err)) => {
                return Poll::Ready(Some(Err(err.into())));
            }
            None => {
                this.state.set(OAICompatResponsesStreamState::Terminated);
                return Poll::Ready(None);
            }
        };

        if ev.data == "[DONE]" {
            this.state.set(OAICompatResponsesStreamState::Terminated);
            return Poll::Ready(None);
        }

        Poll::Ready(Some(
            serde_json::from_str::<StreamEvent<Cow<'_, str>>>(&ev.data)
                .map(|borrowed| borrowed.convert_to_owned(&ev.data))
                .map_err(From::from),
        ))
    }
}

#[cfg(test)]
#[cfg(not(feature = "miri"))] // These tests don't work with miri
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures::StreamExt;
    use std::{convert::Infallible, path::PathBuf};

    fn get_sample_path(model: &str, sample: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("samples");
        path.push(model);
        path.push(sample);
        path
    }

    fn create_test_stream(content: String) -> impl Stream<Item = Result<Bytes, Infallible>> {
        futures::stream::iter([Ok(Bytes::from(content.into_bytes()))])
    }

    async fn test_sample_file(model: &str, sample_file: &str) {
        let path = get_sample_path(model, sample_file);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

        let byte_stream = create_test_stream(content);
        let mut stream = OAICompatResponsesStream::new(byte_stream);

        let mut event_count = 0;
        while let Some(result) = stream.next().await {
            match result {
                Ok(_event) => {
                    event_count += 1;
                }
                Err(e) => {
                    panic!("Error processing {} ({}): {}", sample_file, model, e);
                }
            }
        }

        assert!(
            event_count > 0,
            "Expected at least one event from {} ({})",
            sample_file,
            model
        );
    }

    // Discover all model directories and generate tests
    fn get_model_directories() -> Vec<String> {
        let samples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples");

        if !samples_dir.exists() {
            return vec![];
        }

        std::fs::read_dir(samples_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_dir() {
                    path.file_name()?.to_str().map(String::from)
                } else {
                    None
                }
            })
            .collect()
    }

    macro_rules! generate_model_tests {
        ($(($model:ident, $dir_name:literal)),*) => {
            $(
                mod $model {
                    use super::*;

                    #[tokio::test]
                    async fn sample1_representative() {
                        test_sample_file($dir_name, "sample1_representative.txt").await;
                    }

                    #[tokio::test]
                    async fn sample2_web_search() {
                        test_sample_file($dir_name, "sample2_web_search.txt").await;
                    }

                    #[tokio::test]
                    async fn sample3_rejected() {
                        test_sample_file($dir_name, "sample3_rejected.txt").await;
                    }
                }
            )*
        };
    }

    generate_model_tests!(
        (sonnet_4_5, "anthropic_claude-sonnet-4.5"),
        (gemini_3_pro_preview, "google_gemini-3-pro-preview"),
        (gpt_5_2, "openai_gpt-5.2"),
        (glm_4_7_flash, "z-ai_glm-4.7-flash")
    );

    // Runtime discovery test (useful for CI or when models are added dynamically)
    #[tokio::test]
    async fn test_all_discovered_models() {
        let models = get_model_directories();

        if models.is_empty() {
            println!("No sample directories found, skipping discovery test");
            return;
        }

        let mut tested_count = 0;
        for model in models {
            for sample in &[
                "sample1_representative.txt",
                "sample2_web_search.txt",
                "sample3_rejected.txt",
            ] {
                let path = get_sample_path(&model, sample);
                if path.exists() {
                    println!("Testing {}/{}", model, sample);
                    test_sample_file(&model, sample).await;
                    tested_count += 1;
                }
            }
        }

        assert!(tested_count > 0, "No samples were tested in discovery test");
    }

    // Specific validation tests
    #[tokio::test]
    async fn test_stream_ends_with_done() {
        let content = r#"data: {"type":"response.created","response":{"id":"test","status":"in_progress"},"sequence_number":0}

data: [DONE]
"#;

        let byte_stream = create_test_stream(content.to_string());
        let mut stream = OAICompatResponsesStream::new(byte_stream);

        let mut count = 0;
        while let Some(result) = stream.next().await {
            result.expect("Should not error");
            count += 1;
        }

        assert_eq!(count, 1, "Should receive exactly one event before [DONE]");
    }

    #[tokio::test]
    async fn test_stream_handles_empty() {
        let content = "data: [DONE]\n";

        let byte_stream = create_test_stream(content.to_string());
        let mut stream = OAICompatResponsesStream::new(byte_stream);

        let result = stream.next().await;
        assert!(result.is_none(), "Should immediately terminate on [DONE]");
    }

    #[tokio::test]
    async fn test_stream_skips_processing_lines() {
        let content = r#": OPENROUTER PROCESSING

data: {"type":"response.created","response":{"id":"test","status":"in_progress"},"sequence_number":0}

: OPENROUTER PROCESSING

data: [DONE]
"#;

        let byte_stream = create_test_stream(content.to_string());
        let mut stream = OAICompatResponsesStream::new(byte_stream);

        let mut count = 0;
        while let Some(result) = stream.next().await {
            result.expect("Should not error on processing lines");
            count += 1;
        }

        assert_eq!(
            count, 1,
            "Should receive exactly one event, skipping processing lines"
        );
    }
}
