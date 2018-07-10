use rocket::http::Status;
use rocket::http::uri::{Segments, SegmentError, Uri};
use rocket::request::{FromSegments};
use rocket::response::{Failure};
use std::path::{PathBuf, Path, StripPrefixError};

pub struct DirectoryPath {
    inner: PathBuf
}

impl DirectoryPath {
    pub fn new(path_buf: PathBuf) -> DirectoryPath {
        DirectoryPath { inner: path_buf }
    }

    pub fn from_str(path_str: &str) -> DirectoryPath {
        DirectoryPath { inner: PathBuf::from(path_str) }
    }

    pub fn as_path(&self) -> &Path {
        self.inner.as_path()
    }

    pub fn exists(&self) -> bool {
        self.inner.exists()
    }

    pub fn is_dir(&self) -> bool {
        self.inner.is_dir()
    }

    pub fn to_str(&self) -> Option<&str> {
        self.inner.to_str()
    }

    pub fn strip_prefix(&self, base: &str) -> Result<&Path, StripPrefixError> {
        self.inner.strip_prefix(base)
    }

    fn from_segments(segments: Segments) -> Result<DirectoryPath, SegmentError> {
        let mut buf = PathBuf::new();
        for segment in segments {
            let decoded = Uri::percent_decode(segment.as_bytes())
                .map_err(|e| SegmentError::Utf8(e))?;

            if decoded == ".." {
                buf.pop();
            } else if decoded.contains("..") {
                return Err(SegmentError::BadChar('.'))
            } else if decoded.starts_with('*') {
                return Err(SegmentError::BadStart('*'))
            } else if decoded.ends_with(':') {
                return Err(SegmentError::BadEnd(':'))
            } else if decoded.ends_with('>') {
                return Err(SegmentError::BadEnd('>'))
            } else if decoded.ends_with('<') {
                return Err(SegmentError::BadEnd('<'))
            } else if decoded.contains('/') {
                return Err(SegmentError::BadChar('/'))
            } else if cfg!(windows) && decoded.contains('\\') {
                return Err(SegmentError::BadChar('\\'))
            } else {
                buf.push(&*decoded)
            }
        }

        Ok(DirectoryPath::new(buf))
    }
}

impl AsRef<Path> for DirectoryPath {
    fn as_ref(&self) -> &Path {
        self.inner.as_path()
    }
}

impl<'a> FromSegments<'a> for DirectoryPath {
    type Error = Failure;

    fn from_segments(segments: Segments<'a>) -> Result<DirectoryPath, Failure> {
        match DirectoryPath::from_segments(segments) {
            Ok(directory_path) => Ok(DirectoryPath::new(directory_path.as_ref().into())),
            Err(_error) => Err(Failure(Status::BadRequest))
        }
    }
}
