use std::fs::create_dir_all;
use std::mem;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::collections::hash_map::Iter;

use time::precise_time_ns;

use archiver::Archiver;
use hash::dir_hash;
use result::Result;

/// Monitor構造体  
/// バックアップ対象とmd5ハッシュ値のペアを管理しており、
/// ファイルのバックアップの可否判断、バックアップ処理の指示を行う。
pub struct Monitor<A: Archiver + Default> {
    paths: HashMap<PathBuf, Vec<u8>>,
    archiver: A,
    destination: PathBuf,
}

impl<A: Archiver + Default> Monitor<A> {
    /// Monitor構造体のコンストラクタ
    pub fn new(archiver: A, paths: HashMap<PathBuf, Vec<u8>>, destination: PathBuf) -> Self {
        debug!("Monitor::new destination: {:?}", destination);
        Monitor { paths, archiver, destination }
    }

    /// バックアップ対象のパスとmd5ハッシュ値のキャッシュを設定する。
    pub fn set_paths(&mut self, paths: HashMap<PathBuf, Vec<u8>>) {
        self.paths = paths;
    }

    /// バックアップ対象のパスとmd5ハッシュ値のキャッシュをイテレータとして取得する。
    pub fn get_paths_iter(&self) -> Iter<PathBuf, Vec<u8>> {
        self.paths.iter()
    }

    /// 更新検知 & バックアップ処理関数  
    /// 一定時間ごとに本関数を呼び出すことを期待する。
    pub fn now(&mut self) -> Result<usize> {
        let mut count = 0;

        for (path, h) in self.paths.iter_mut() {
            let new_hash = dir_hash(path).unwrap_or_default();

            // 変更を確認する。
            // 変更がない場合は正常処理として次のバックアップ対象の比較に移る。
            if *h != new_hash {
                // 変更がある場合

                *h = new_hash;

                // バックアップ先のパスを生成する。
                let p = Path::new(path);

                // 1) ルートを含んでいるか、確認する。
                // ルートを含んでいる場合はルート箇所の削除を行う。
                // そうでない場合はそのままpに値をバインドする。
                let p = if p.has_root() {
                    // TODO: 将来的にWINDOWSなどの対応を行う場合は修正を行う。
                    p.strip_prefix("/").map(|s| s.to_path_buf()).unwrap_or_default()
                } else {
                    p.to_path_buf()
                };

                // 2) パスがファイルかどうか確認する。
                // ファイルの場合はファイル名を取り除いたパスにする。
                // ディレクトリの場合はそのままdir_nameに値をバインドする。
                let dir_name = if p.is_file() {
                    p.parent().map(|p| p.to_path_buf()).unwrap_or_default()
                } else {
                    p
                };

                // TODO: zip拡張子をハードコーディングしているため、その他に対応する際に修正する。
                let file_name = PathBuf::from(format!("{}", precise_time_ns())).with_extension("zip");
                
                debug!("{:?}", self.destination);
                debug!("{:?}", dir_name);
                debug!("{:?}", file_name);

                // バックアップ先のパスの生成
                let dest_path = self.destination
                    .join(dir_name)
                    .join(file_name);

                debug!("{:?}", dest_path);

                // ファイル名を除くフォルダを取得し、
                // 存在するか確認する。その際に存在しない場合は、生成する。
                if let Some(dest_dir_path) = dest_path.parent() {
                    if !dest_dir_path.exists() {
                        create_dir_all(dest_dir_path)?;
                    }
                }

                // [memo]
                // アーカイブファイルを作成し、出力先のディレクトリは
                // /hoge/fuga の場合 -> ~/.backupfs_archive/hoge/fuga としたい
                // しかし、現状のバックアップファイルは /hoge/fuga 内に作成されるため、
                // バックアップが実行されるたびに、md5ハッシュが変更となり、
                // 内部の対象ファイルが変更されていない状態でも変更検知が誤作動を起こし、
                // backupが実行されている状態となっている.
                let archiver = mem::replace(&mut self.archiver, A::default());
                let res = archiver.archive(path, &dest_path);
                if res.is_err() {
                    error!("{:?}", res.unwrap_err());
                    continue
                }
                
                count += 1;
            }
        }

        Ok(count)
    }
}