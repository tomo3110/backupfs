use std::path::Path;
use std::io::prelude::*;
use std::fs::File;

use zip::ZipWriter;
use zip::write::FileOptions;
use walkdir::WalkDir;

use archiver::Archiver;
use result::Result;

/// ZIP構造体
/// ZIPアーカイブを行う
pub struct ZIP;

impl Archiver for ZIP {
    fn archive<P: AsRef<Path>>(&self, src: P, dest: P) -> Result<()> {
        let mut out = File::create(dest).unwrap();
        let mut writer = ZipWriter::new(&mut out);
        let walk_dir = WalkDir::new(&src);        
        let dir = walk_dir.into_iter().filter_map(|e| e.ok());
        let mut buffer = Vec::default();
        
        for entry in dir {
            let path = entry.path();
            let name = path.strip_prefix(&src).unwrap().to_str().unwrap();

            if path.is_file() {
                writer.start_file(name, FileOptions::default())?;
                let mut f = File::open(path)?;

                f.read_to_end(&mut buffer)?;
                writer.write_all(&buffer)?;
                buffer.clear();
            }
        }
        
        writer.finish()?;
        Ok(())
    }
}

impl Default for ZIP {
    fn default() -> Self {
        ZIP
    }
}