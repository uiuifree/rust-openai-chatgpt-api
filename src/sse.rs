use std::pin::Pin;

use futures_util::{Stream, StreamExt};
use serde::de::DeserializeOwned;

use crate::error::OpenAiError;

/// A stream of parsed server-sent events returned by the streaming endpoints.
pub type EventStream<T> = Pin<Box<dyn Stream<Item = Result<T, OpenAiError>> + Send>>;

/// Incremental parser for a server-sent-event byte stream.
///
/// Bytes are pushed in as they arrive and complete `data:` payloads are pulled
/// out. Multibyte UTF-8 sequences never span an event boundary (the delimiter
/// is ASCII), so decoding per event is safe.
#[derive(Default)]
pub(crate) struct SseBuffer {
    buf: Vec<u8>,
}

impl SseBuffer {
    pub(crate) fn push(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// Returns the next complete `data:` payload, or `None` until one is buffered.
    pub(crate) fn next_data(&mut self) -> Option<String> {
        loop {
            let (end, delimiter_len) = find_event_boundary(&self.buf)?;
            let event: Vec<u8> = self.buf.drain(..end + delimiter_len).collect();
            let text = String::from_utf8_lossy(&event[..end]);
            let data_lines: Vec<&str> = text
                .lines()
                .filter_map(|line| line.strip_prefix("data:"))
                .map(|rest| rest.strip_prefix(' ').unwrap_or(rest))
                .collect();
            if !data_lines.is_empty() {
                return Some(data_lines.join("\n"));
            }
        }
    }
}

fn find_event_boundary(buf: &[u8]) -> Option<(usize, usize)> {
    for i in 0..buf.len().saturating_sub(1) {
        if buf[i] == b'\n' && buf[i + 1] == b'\n' {
            return Some((i, 2));
        }
        if buf[i..].starts_with(b"\r\n\r\n") {
            return Some((i, 4));
        }
    }
    None
}

/// Converts a streaming HTTP response into a stream of JSON-decoded events.
///
/// The `data: [DONE]` sentinel used by the Chat Completions API terminates the
/// stream; APIs without the sentinel terminate when the connection closes.
pub(crate) fn sse_events<T>(response: reqwest::Response) -> EventStream<T>
where
    T: DeserializeOwned + Send + 'static,
{
    Box::pin(async_stream::try_stream! {
        let mut body = response.bytes_stream();
        let mut buffer = SseBuffer::default();
        'read: while let Some(chunk) = body.next().await {
            buffer.push(&chunk?);
            while let Some(data) = buffer.next_data() {
                if data == "[DONE]" {
                    break 'read;
                }
                let event: T =
                    serde_json::from_str(&data).map_err(|e| OpenAiError::parse(e, &data))?;
                yield event;
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_events_across_chunk_boundaries() {
        let mut buffer = SseBuffer::default();
        buffer.push(b"event: message\ndata: {\"a\"");
        assert_eq!(buffer.next_data(), None);
        buffer.push(b":1}\n\ndata: [DONE]\n\n");
        assert_eq!(buffer.next_data(), Some("{\"a\":1}".to_string()));
        assert_eq!(buffer.next_data(), Some("[DONE]".to_string()));
        assert_eq!(buffer.next_data(), None);
    }

    #[test]
    fn supports_crlf_delimiters() {
        let mut buffer = SseBuffer::default();
        buffer.push(b"data: hello\r\n\r\n");
        assert_eq!(buffer.next_data(), Some("hello".to_string()));
    }

    #[test]
    fn joins_multi_line_data() {
        let mut buffer = SseBuffer::default();
        buffer.push(b"data: line1\ndata: line2\n\n");
        assert_eq!(buffer.next_data(), Some("line1\nline2".to_string()));
    }

    #[test]
    fn skips_comment_only_events() {
        let mut buffer = SseBuffer::default();
        buffer.push(b": keep-alive\n\ndata: x\n\n");
        assert_eq!(buffer.next_data(), Some("x".to_string()));
    }
}
