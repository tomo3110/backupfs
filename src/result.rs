use std::error;
use std::fmt;
use std::io;
use std::sync::PoisonError;
use std::result;

use filedb::Error as FileDBError;
use serde_json::Error as JsonError;
use walkdir::Error as WalkDirError;
use zip::result::ZipError;

pub enum Error<T> {
    FileDB(FileDBError),
    Io(io::Error),
    Json(JsonError),
    Poison(PoisonError<T>),
    WalkDir(WalkDirError),
    Zip(ZipError),
}

impl<T> fmt::Debug for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FileDB(ref err) => write!(f, "[backup-fs] {}", err),
            Error::Io(ref err) => write!(f, "[backup-fs] {}", err),
            Error::Json(ref err) => write!(f, "[backup-fs] {}", err),
            Error::Poison(ref err) => write!(f, "[backup-fs] {}", err),
            Error::WalkDir(ref err) => write!(f, "[backup-fs] {}", err),
            Error::Zip(ref err) => write!(f, "[backup-fs] {}", err),
        }
    }
}

impl<T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FileDB(ref err) => write!(f, "[backup-fs] {}", err),
            Error::Io(ref err) => write!(f, "[backup-fs] {}", err),
            Error::Json(ref err) => write!(f, "[backup-fs] {}", err),
            Error::Poison(ref err) => write!(f, "[backup-fs] {}", err),
            Error::WalkDir(ref err) => write!(f, "[backup-fs] {}", err),
            Error::Zip(ref err) => write!(f, "[backup-fs] {}", err),
        }
    }
}

impl<T> error::Error for Error<T> {
    fn description<'a>(&'a self) -> &'a str {
        match *self {
            Error::FileDB(_) => "",
            Error::Io(ref err) => err.description(),
            Error::Json(ref err) => err.description(),
            Error::Poison(ref err) => err.description(),
            Error::WalkDir(ref err) => err.description(),
            Error::Zip(ref err) => err.description(),
        }
    }
}

impl<T> From<FileDBError> for Error<T> {
    fn from(err: ::filedb::Error) -> Self {
        Error::FileDB(err)
    }
}

impl<T> From<io::Error> for Error<T> {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl<T> From<JsonError> for Error<T> {
    fn from(err: ::serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl<T> From<PoisonError<T>> for Error<T> {
    fn from(err: PoisonError<T>) -> Self {
        Error::Poison(err)
    }
}

impl<T> From<WalkDirError> for Error<T> {
    fn from(err: ::walkdir::Error) -> Self {
        Error::WalkDir(err)
    }
}

impl<T> From<ZipError> for Error<T> {
    fn from(err: ::zip::result::ZipError) -> Self {
        Error::Zip(err)
    }
}

pub type Result<T> = result::Result<T, Error<T>>;