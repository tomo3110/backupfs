extern crate backupfs;
extern crate clap;
extern crate ctrlc;
extern crate env_logger;
extern crate filedb;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use backupfs::archiver::ZIP;
use backupfs::monitor::Monitor;
use backupfs::PathItem;
use backupfs::result::Result;

use clap::{Arg, ArgMatches, App};

use filedb::FileDB;
use filedb::callback::*;

fn main() {
    env_logger::init();

    let mut ctx: Context = Context::init();
    match ctx.run() {
        Ok(_) => info!("exit"),
        Err(err) => error!("{:?}", err),
    }
}

/// Context構造体  
/// 主要な構成要素をまとめる。
pub struct Context {
    args: ArgMatches<'static>,
    monitor: Monitor<ZIP>,
    db: FileDB,
}

impl Context {
    /// 初期化処理
    /// 本コマンドを利用する際の初期のセットアップを担当する。
    pub fn init() -> Self {
        let db = FileDB::default();
        let args = Context::parse_args();
        let mut ctx = Context::new(db, args);
        if let Ok(paths) = ctx.load() {
            ctx.monitor.set_paths(paths);
        }
        ctx
    }

    /// Context構造体のコンストラクタ
    /// 内部にて、バックアップ先のディレクトリの設定、
    /// Monitor構造体の生成などをおこなっている。
    pub fn new(db: FileDB, args: ArgMatches<'static>) -> Self {
        let path = if let Some(dest) = args.value_of("dest") {
            PathBuf::from(dest)
        } else {
            env::home_dir().unwrap_or_default().join(".backupfs_archive")
        };
        debug!("destination_path: {:?}", path);
        let monitor = Monitor::new(ZIP, HashMap::new(), path);

        Context { args, monitor, db }
    }

    /// コマンドインターフェースの定義
    /// 本コマンドが想定する、コマンド引数を設定している。
    pub fn parse_args() -> ArgMatches<'static> {
        App::new("backupd")
            .version("0.0.1")
            .author("s tomo <uotias64_mole@yahoo.co.jp>")
            .about("backup system deamon")
            .arg(Arg::from_usage("--dest [PATH] 'dest path'"))
            .get_matches()
    }

    fn load(&mut self) -> Result<HashMap<PathBuf, Vec<u8>>> {
        let mutex = self.db.c("paths")?;
        let mut cmap = HashMap::new();
        if let Ok(col) = mutex.lock() {
            col.for_each(|_, data| {
                let res = serde_json::from_slice(&data);
                if res.is_err() {
                    error!("{:?}", res.unwrap_err());
                    return ForEachResultValue::new(false);
                }
                let path_item: PathItem = res.unwrap();

                cmap.entry(path_item.path()).or_insert(path_item.hash());

                ForEachResultValue::new(false)
            })?;
        }
        Ok(cmap)
    }

    fn save(&mut self) -> Result<()> {
        let paths: HashMap<PathBuf, Vec<u8>> = self.monitor.get_paths_iter()
            .map(|(path, hash)| (PathBuf::from(path), hash.clone()))
            .collect::<HashMap<PathBuf, Vec<u8>>>();

        let mutex = self.db.c("paths")?;
        if let Ok(mut col) = mutex.lock() {

            col.select_each(move |_, data| {

                let res = serde_json::from_slice(&data);
                if res.is_err() {
                    error!("{:?}", res.unwrap_err());
                    return SelectResultValue::new(false, data.clone(), false);
                }

                let mut path_item: PathItem = res.unwrap();
                if let Some(hash) = paths.get(&path_item.path()) {
                    if &path_item.hash() != hash {
                        path_item.set_hash(hash.clone());
                    }
                }

                let json = serde_json::to_vec(&path_item).unwrap_or(data);
                
                SelectResultValue::new(false, json, false)
            })?;

        }

        Ok(())
    }

    /// 実行処理  
    /// 本コマンドの終了処理の設定、
    /// ワーカーの呼び出しを行う。
    pub fn run(&mut self) -> Result<()> {
        info!("starting");

        // 終了処理の伝達用のチャンネルの生成
        let (exit_sender, exit_receiver) = mpsc::channel::<()>();
        let send = exit_sender.clone();

        // 終了処理
        // ctrl + c 押下時の処理
        let res = ctrlc::set_handler(move || {
            info!("goodbye...");
            if let Err(err) = send.send(()) {
                error!("{:?}", err);
            }
        });

        match res {
            Err(err) => error!("{:?}", err),
            _ => {}
        };

        // ワーカー呼び出し
        self.watch_worker(&exit_receiver)
    }

    fn watch_worker(&mut self, exit_receiver: &mpsc::Receiver<()>) -> Result<()> {
        // ワーカーなので、loop
        loop {
            // 終了処理
            // チャンネルよりデータが渡されると、
            // return によってループを抜ける。
            // その際にバックアップ対象のパスとmd5ハッシュ値のキャッシュをfiledbへ保存する。
            if let Ok(_) = exit_receiver.try_recv() {
                return self.save();
            }

            // 実際の変更検知 & バックアップ処理の受付
            // 基本的にエラー発生時もログに出力するのみで、
            // ハンドリングは行わず、次の処理へ移る。
            match self.monitor.now() {
                Ok(count) => {
                    if count > 0 {
                        info!("changed {} files", count);
                    } else {
                        info!("not changed");
                    }
                    // TODO: 保存周期の設定を柔軟にする必要がある。
                    thread::sleep(Duration::new(5, 0));
                },
                Err(err) => {
                    // TODO: 失敗時のハンドルが必要な際は追記すること
                    warn!("{:?}", err);
                },
            };
        }
    }
}