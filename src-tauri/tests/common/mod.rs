use std::path::PathBuf;

pub fn corpus_dir() -> Option<PathBuf> {
    let Some(path) = std::env::var_os("TACITUS_REPLAY_CORPUS") else {
        eprintln!("TACITUS_REPLAY_CORPUS is unset; skipping real-replay test");
        return None;
    };
    let path = PathBuf::from(path);
    assert!(
        !path.as_os_str().is_empty() && path.is_dir(),
        "TACITUS_REPLAY_CORPUS must point to a replay folder: {}",
        path.display()
    );
    Some(path)
}
