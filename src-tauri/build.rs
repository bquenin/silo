fn main() {
    if std::env::var_os("CARGO_FEATURE_PORTABLE").is_some() {
        embed_webview2();
    }
    tauri_build::build()
}

fn embed_webview2() {
    use sha2::{Digest, Sha256};
    use std::{fs::File, io::Read, path::PathBuf};

    println!("cargo:rerun-if-env-changed=SILO_WEBVIEW2_CAB");
    println!("cargo:rerun-if-changed=resources/webview2-runtime.json");
    let metadata: serde_json::Value =
        serde_json::from_str(include_str!("resources/webview2-runtime.json"))
            .expect("invalid pinned WebView2 metadata");
    assert_eq!(
        std::env::var("TARGET").unwrap(),
        metadata["target"].as_str().unwrap(),
        "the portable runtime must match the build architecture"
    );
    let cab = PathBuf::from(
        std::env::var_os("SILO_WEBVIEW2_CAB")
            .expect("portable builds need the pinned WebView2 CAB; run npm run build:portable"),
    )
    .canonicalize()
    .expect("WebView2 CAB is missing");
    println!("cargo:rerun-if-changed={}", cab.display());
    let mut file = File::open(&cab).expect("cannot read WebView2 CAB");
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = file.read(&mut buffer).expect("cannot read WebView2 CAB");
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
    }
    let hash = format!("{:x}", digest.finalize());
    assert_eq!(
        hash,
        metadata["sha256"].as_str().unwrap(),
        "WebView2 CAB checksum mismatch"
    );
    println!("cargo:rustc-env=SILO_WEBVIEW2_CAB={}", cab.display());
    println!("cargo:rustc-env=SILO_WEBVIEW2_SHA256={hash}");
    println!(
        "cargo:rustc-env=SILO_WEBVIEW2_DIRECTORY=Microsoft.WebView2.FixedVersionRuntime.{}.{}",
        metadata["version"].as_str().unwrap(),
        metadata["architecture"].as_str().unwrap()
    );
}
