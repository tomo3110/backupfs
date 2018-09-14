extern crate backupfs;
extern crate clap;
extern crate filedb;
extern crate serde;
extern crate serde_json;

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use backupfs::PathItem;

use clap::{Arg, ArgMatches, App, SubCommand};

use filedb::FileDB;
use filedb::C;
use filedb::callback::*;

fn main() {
    let mut ctx = Context::init();

    if ctx.is_call_add() {
        return ctx.add_command().unwrap();
    }

    if ctx.is_call_remove() {
        return ctx.remove_command().unwrap();
    }

    if ctx.is_call_list() {
        return ctx.list_command().unwrap();
    }

    ctx.usage()
}

/// Context構造体  
/// 
pub struct Context {
    args: ArgMatches<'static>,
    config_path: PathBuf,
    db: FileDB,
}

impl Context {
    pub fn init() -> Self {
        let args = Self::parse_args();
        let path = if let Some(dest) = args.value_of("CONFIG_FILE") {
            PathBuf::from(dest)
        } else {
            env::home_dir().unwrap_or_default().join(".backupfs")
        };
        let db = FileDB::default();
        Self::new(args, path, db)
    }
    pub fn new<P: AsRef<OsStr>>(args: ArgMatches<'static>, config_path: P, db: FileDB) -> Self {
        let config_path = PathBuf::from(&config_path).to_path_buf();
        Context { args, config_path, db }
    }
    pub fn parse_args() -> ArgMatches<'static> {
        App::new("backup-client")
            .version("0.0.1")
            .author("s tomo <uotias64_mole@yahoo.co.jp>")
            .about("backup system client")
            .arg(Arg::from_usage("--config -c [CONFIG_FILE] 'config file path'"))
            .subcommand(SubCommand::with_name("add")
                .about("register backup target")
                .arg_from_usage("<PATH> 'directory or file path'")
            )
            .subcommand(SubCommand::with_name("remove")
                .about("delete backup target")
                .arg_from_usage("<PATH> 'directory or file path'")
            )
            .subcommand(SubCommand::with_name("list")
                .about("show backup target list")
            )
            .get_matches()
    }
    pub fn is_call_add(&self) -> bool {
        self.args.subcommand_matches("add").is_some()
    }
    pub fn is_call_remove(&self) -> bool {
        self.args.subcommand_matches("remove").is_some()
    }
    pub fn is_call_list(&self) -> bool {
        self.args.subcommand_matches("list").is_some()
    }

    pub fn add_command(&mut self) -> filedb::Result<()> {
        let option_add = self.args.subcommand_matches("add");
        if option_add.is_none() {
            return Ok(());
        }
        let matches = option_add.unwrap();
        if let Some(path_str) = matches.value_of("PATH") {
            let param_path = PathBuf::from(path_str);
            let current_dir = env::current_dir();
            let path = Self::to_absolute_path(current_dir.unwrap(), param_path);

            if path.exists() {
                let mutex: &Mutex<C> = self.db.c("paths")?;
                let mut col: MutexGuard<C> = mutex.lock().unwrap();

                let path_item = PathItem::new(path, Vec::new());

                let json = serde_json::to_vec(&path_item).unwrap();

                println!("[backupfs-client] added: {}", path_item.path().to_string_lossy());

                col.insert(&json.as_slice())?;
            }
        }
        Ok(())
    }

    pub fn to_absolute_path(current_dir: PathBuf, some_path: PathBuf) -> PathBuf {
        // 絶対パスの場合は早期リターン
        if some_path.is_absolute() {
            return some_path;
        }
        // ./ 前頭に上記のパスがついている場合は取り除く
        // そうでない場合はそのまま返却する。
        // ./foo/bar -> foo/bar
        let some_path = if some_path.starts_with("./") {
            some_path
                .strip_prefix("./")
                .map(|s| PathBuf::from(s))
                .unwrap_or_default()
        } else {
            some_path
        };
        // カレントディレクトリへ接着する。
        // /tmp/here が カレントディレクトリなら
        // /tmp/here + foo/bar -> /tmp/here/foo/bar とする.
        current_dir.join(some_path)
    }

    pub fn remove_command(&mut self) -> filedb::Result<()> {
        let option_remove = self.args.subcommand_matches("remove");
        if option_remove.is_none() {
            return Ok(());
        }
        let matches = option_remove.unwrap();
        if let Some(path_str) = matches.value_of("PATH") {
            let param_path = PathBuf::from(path_str)
                .strip_prefix("./")
                .map(|p| p.to_path_buf())
                .unwrap();

            let mut path: PathBuf = env::current_dir()?;

            path.push(param_path);

            let mutex: &Mutex<C> = self.db.c("paths")?;
            let mut col: MutexGuard<C> = mutex.lock().unwrap();

            col.remove_each(|_, b| {

                let path_item: PathItem = serde_json::from_slice(b.as_slice()).unwrap();

                RemoveResultValue::new(path_item.path() == path, false)
            })?;
        }
        Ok(())
    }

    pub fn list_command(&mut self) -> filedb::Result<()> {
        let option_list = self.args.subcommand_matches("list");
        if option_list.is_none() {
            return Ok(());
        }
        let mutex: &Mutex<C> =self.db.c("paths").unwrap();
        let col: MutexGuard<C> = mutex.lock().unwrap();
        col.for_each(|_, b| {
            let path: PathItem = serde_json::from_slice(b.as_slice()).unwrap();
            println!("{}", path);
            ForEachResultValue::new(false)
        })?;
        Ok(())
    }

    pub fn usage(&self) {
        println!("{}", self.args.usage());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_absolute_path() {
        let current_dir = PathBuf::from("/tmp/here");
        let some_path = PathBuf::from("./foo/bar");
        let res_path = Context::to_absolute_path(current_dir, some_path);
        assert_eq!(PathBuf::from("/tmp/here/foo/bar"), res_path);
    }
}
