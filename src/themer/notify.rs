use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::{env, path::PathBuf, thread};

pub fn start_watching() {
    thread::spawn(|| {
        let path = PathBuf::from(env::var("HOME").expect("HOME environment variable not set"))
            .join(".cache")
            .join("wal")
            .join("colors.json");

        println!("Watching: {}", path.display());

        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        println!("Pywal theme changed");
                    }
                }

                Err(err) => {
                    eprintln!("Watcher error: {err}");
                }
            })
            .unwrap();

        watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

        loop {
            thread::park();
        }
    });
}
