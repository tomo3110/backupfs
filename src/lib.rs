extern crate zip;
extern crate walkdir;
extern crate chrono;
extern crate crypto;
extern crate time;
extern crate filedb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::fmt;
use std::path::PathBuf;


pub mod archiver;
pub mod hash;
pub mod monitor;
pub mod result;


/// PathItem構造体  
/// バックアップ対象とパスと、md5ハッシュ値を格納する。
/// jsonへパースを行い、データをfiledbに格納する為に利用。
#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Default, Debug)]
pub struct PathItem {
    path: PathBuf,
    hash: Vec<u8>,
}

impl PathItem {
    /// PathItem構造体のコンストラクタ
    pub fn new(path: PathBuf, hash: Vec<u8>) -> Self {
        PathItem { path, hash }
    }

    /// Vec<u8>のバイナリからPathItem構造体へ変換する。
    pub fn from_vec(v: Vec<u8>) -> result::Result<Self> {
        let path_item: PathItem = serde_json::from_slice(v.as_slice())?;
        Ok(path_item)
    }

    /// パスを取得する。
    pub fn path(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    /// md5ハッシュ値を取得する。
    pub fn hash(&self) -> Vec<u8> {
        self.hash.clone()
    }

    /// md5ハッシュ値を設定する。
    pub fn set_hash(&mut self, hash: Vec<u8>) {
        self.hash = hash;
    }
}

impl fmt::Display for PathItem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(formatter, "backuppath: {:?}", self.path)?;
        Ok(())
    }
}

