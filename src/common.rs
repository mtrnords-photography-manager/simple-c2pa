use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::result::Result;
use std::sync::Arc;

use openssl::error::ErrorStack;
use tempfile::{NamedTempFile, PersistError};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct FileData {
    path: Option<PathBuf>,
    bytes: Option<Vec<u8>>,
    #[allow(dead_code)]
    file_name: Option<String>,
}

impl FileData {
    pub fn new(
        path: Option<PathBuf>,
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

        Err(SimpleC2PAError::Failure {
            message: "No bytes or path".to_owned(),
        })
    }

    pub fn get_path(&self) -> Result<PathBuf, SimpleC2PAError> {
        if let Some(ref path) = self.path {
            return Ok(path.clone());
        }

        if let Some(bytes) = &self.bytes {
            let mut file = NamedTempFile::new()?;
            file.write_all(bytes)?;
            return Ok(file.path().into());
        }

        Err(SimpleC2PAError::Failure {
            message: "No bytes or path".to_owned(),
        })
    }
}

#[derive(Error, Debug)]
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
