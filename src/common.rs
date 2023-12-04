use openssl::error::ErrorStack;
use std::fs;
use std::io::Write;
use std::result::Result;
use std::sync::Arc;
use tempfile::{NamedTempFile, PersistError};
use thiserror::Error;

#[derive(Debug, Clone, uniffi::Object)]
pub struct FileData {
    path: Option<String>,
    bytes: Option<Vec<u8>>,
    file_name: Option<String>,
}

#[uniffi::export]
impl FileData {
    #[uniffi::constructor]
    pub fn new(
        path: Option<String>,
        bytes: Option<Vec<u8>>,
        file_name: Option<String>,
    ) -> Arc<Self> {
        Arc::new(FileData {
            path,
            bytes,
            file_name,
        })
    }

    pub fn get_bytes(&self) -> Result<Vec<u8>, SimpleC2PAError> {
        if let Some(bytes) = &self.bytes {
            return Ok(bytes.clone());
        }

        if let Some(path) = &self.path {
            return Ok(fs::read(path)?);
        }

        return Err(SimpleC2PAError::Failure {
            message: "No bytes or path".to_owned(),
        });
    }

    pub fn get_path(&self) -> Result<String, SimpleC2PAError> {
        if let Some(path) = &self.path {
            return Ok(path.clone());
        }

        if let Some(bytes) = &self.bytes {
            let mut file = NamedTempFile::new()?;
            file.write_all(bytes)?;
            let file_name = &self.file_name.clone().unwrap_or("file".to_owned());
            let path = format!("/tmp/{}", file_name);
            file.persist(path.clone())?;
            return Ok(path.clone());
        }

        return Err(SimpleC2PAError::Failure {
            message: "No bytes or path".to_owned(),
        });
    }
}

#[derive(Error, Debug, uniffi::Error)]
pub enum SimpleC2PAError {
    #[error("Failed with message: {message}")]
    Failure { message: String },

    #[error("unexpected id: {id}")]
    Unexpected { id: i32 },
}

impl From<std::io::Error> for SimpleC2PAError {
    fn from(error: std::io::Error) -> Self {
        SimpleC2PAError::Failure {
            message: error.to_string(),
        }
    }
}

impl From<PersistError> for SimpleC2PAError {
    fn from(error: PersistError) -> Self {
        SimpleC2PAError::Failure {
            message: error.to_string(),
        }
    }
}

impl From<ErrorStack> for SimpleC2PAError {
    fn from(error: ErrorStack) -> Self {
        SimpleC2PAError::Failure {
            message: error.to_string(),
        }
    }
}

impl From<c2pa::Error> for SimpleC2PAError {
    fn from(error: c2pa::Error) -> Self {
        SimpleC2PAError::Failure {
            message: error.to_string(),
        }
    }
}
