use std::env;
use std::fs;
use std::mem;
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::time;

use threadpool::ThreadPoolExecutor;

fn main() {
    let roots = env::args()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();
    let executor = ThreadPoolExecutor::new();
    let (sender, receiver) = sync_channel(1_000_000);

    let now = time::Instant::now();
    for root in roots {
        let tx = SyncSender::clone(&sender);
        executor.submit(move || walk_dir(root, tx).unwrap());
    }

    mem::drop(sender);
    executor.submit(move || {
        let mut nfiles = 0;
        let mut nbytes = 0;
        for size in receiver {
            nfiles += 1;
            nbytes += size;
        }
        println!("{} files {} GB", nfiles, nbytes as f64 / 1e9);
        println!("time: {:?}", now.elapsed());
    });
}

fn walk_dir(dir: PathBuf, tx: SyncSender<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry.unwrap();
        let f_info = entry.metadata()?;

        if f_info.is_dir() {
            walk_dir(entry.path(), SyncSender::clone(&tx))?;
        } else if f_info.is_file() {
            tx.send(f_info.len())?;
        }
    }
    Ok(())
}
