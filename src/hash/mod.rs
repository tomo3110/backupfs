
use std::path::Path;
use std::fs::Metadata;
use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;

use chrono::prelude::*;
use crypto::digest::Digest;
use crypto::md5::Md5;
use walkdir::WalkDir;

use result::Result;

/// パスごとのハッシュ生成関数
/// ディレクトリ/ファイルのメタデータからmd5ハッシュ値を生成する。
/// 与えられるパスは実在することを期待するが、ディレクトリ/ファイルを選ばない
pub fn dir_hash<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut md5 = Md5::new();
    let mut output: [u8; 64] = [0; 64];
    let mut w: Vec<u8> = Vec::new();

    let dir = WalkDir::new(path).into_iter();

    for entry in dir {
        if let Ok(ent) = entry {
            let path: &Path = ent.path();
            if !path.is_file() {
                continue;
            }
            let mut file: File = File::open(path)?;
            let mut info: Metadata = file.metadata()?;

            let created: DateTime<Utc> = info.created().unwrap_or(SystemTime::now()).into();
            let modified: DateTime<Utc> = info.modified().unwrap_or(SystemTime::now()).into();
        
            let _ = writeln!(w, "{}", path.to_string_lossy());
            let _ = writeln!(w, "{}", created.timestamp());
            let _ = writeln!(w, "{}", modified.timestamp());

            let file_type = info.file_type();

            if file_type.is_dir() {
                let _ = writeln!(w, "is_dir");
            }
            if file_type.is_file() {
                let _ = writeln!(w, "is_file");
            }
            if file_type.is_symlink() {
                let _ = writeln!(w, "is_symlink");
            }
            if info.permissions().readonly() {
                let _ = writeln!(w, "readonly");
            }
        }
    }

    md5.reset();
    md5.input(w.as_slice());
    md5.result(&mut output);

    Ok(output.to_vec())
}