use std::path::Path;
use result::Result;

mod zipper;
pub use self::zipper::ZIP;

/// Archiverトレイト
/// アーカイブ処理を行う構造体を定義するトレイト
/// Monitor構造体に本トレイトが満たされていれば
/// 利用することができる。
pub trait Archiver {
    /// アーカイブ処理関数
    /// 二つのパスを要求し、1つ目がアーカイブ対象、2つ目がアーカイブ先(出力先)のパスとなる。
    /// 1つ目はファイル/ディレクトリを問わないが、2つ目はアーカイブ後のファイルを表すパスとなる。
    fn archive<P: AsRef<Path>>(&self, src: P, dest: P) -> Result<()>;
}

