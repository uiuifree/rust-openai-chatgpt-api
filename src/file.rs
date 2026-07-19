use std::path::Path;

use reqwest::multipart::Part;

use crate::error::OpenAiError;

/// An in-memory file attached to a multipart request (audio and image endpoints).
#[derive(Debug, Clone)]
pub struct FilePart {
    filename: String,
    bytes: Vec<u8>,
    mime: Option<String>,
}

impl FilePart {
    /// Creates a file from in-memory bytes; the extension of `filename`
    /// determines the MIME type unless [`FilePart::mime`] overrides it.
    pub fn new(filename: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            filename: filename.into(),
            bytes,
            mime: None,
        }
    }

    /// Reads the file at `path` into memory.
    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)?;
        let filename = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "file".to_string());
        Ok(Self::new(filename, bytes))
    }

    /// Overrides the MIME type guessed from the file extension.
    pub fn mime(mut self, mime: impl Into<String>) -> Self {
        self.mime = Some(mime.into());
        self
    }

    pub(crate) fn into_part(self) -> Result<Part, OpenAiError> {
        let mime = self
            .mime
            .unwrap_or_else(|| guess_mime(&self.filename).to_string());
        Ok(Part::bytes(self.bytes)
            .file_name(self.filename)
            .mime_str(&mime)?)
    }
}

fn guess_mime(filename: &str) -> &'static str {
    let extension = filename
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "mp3" | "mpga" => "audio/mpeg",
        "mp4" | "m4a" => "audio/mp4",
        "wav" => "audio/wav",
        "webm" => "audio/webm",
        "ogg" | "oga" => "audio/ogg",
        "flac" => "audio/flac",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guesses_mime_from_extension() {
        assert_eq!(guess_mime("voice.MP3"), "audio/mpeg");
        assert_eq!(guess_mime("photo.png"), "image/png");
        assert_eq!(guess_mime("archive.zip"), "application/octet-stream");
        assert_eq!(guess_mime("noextension"), "application/octet-stream");
    }
}
