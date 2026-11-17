//! Condition of the O1 decision: `fs-conformance` is used as a DEV-dependency only, and itself depends on nothing but
//! `async-trait` (and later `fs-types`). The scan reads the `Cargo.toml` files of the workspace members.

use std::fs;
use std::path::PathBuf;

const ALLOWED_OWN: [&str; 2] = ["async-trait", "fs-types"];

/// The names under normal-dependency tables (`[dependencies]`, `[target.*.dependencies]`, `[dependencies.NAME]`,
/// `[build-dependencies]`) of a manifest.
fn normal_deps(manifest: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_normal = false;
    for line in manifest.lines().map(str::trim) {
        if let Some(head) = line.strip_prefix('[') {
            let head = head.trim_end_matches(']');
            if let Some((table, name)) = head.split_once(".dependencies.") {
                if !table.contains("dev") {
                    out.push(name.to_owned());
                }
                in_normal = false;
                continue;
            }
            let tail = head.rsplit('.').next().unwrap_or(head);
            in_normal = tail == "dependencies" || tail == "build-dependencies";
            if let Some(name) = head
                .strip_prefix("dependencies.")
                .or_else(|| head.strip_prefix("build-dependencies."))
            {
                out.push(name.to_owned());
                in_normal = false;
            }
        } else if in_normal
            && !line.starts_with('#')
            && let Some((name, _)) = line.split_once('=')
        {
            out.push(name.trim().to_owned());
        }
    }
    out
}

fn manifests() -> Vec<(String, String)> {
    let crates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut v = Vec::new();
    for entry in fs::read_dir(crates).expect("crates dir") {
        let dir = entry.expect("entry").path();
        let file = dir.join("Cargo.toml");
        if let Ok(text) = fs::read_to_string(&file) {
            v.push((
                dir.file_name()
                    .expect("name")
                    .to_string_lossy()
                    .into_owned(),
                text,
            ));
        }
    }
    v
}

/// No other workspace member lists `fs-conformance` under normal or build dependencies.
#[test]
fn only_dev_dependency() {
    for (name, text) in manifests() {
        if name != "fs-conformance" {
            assert!(
                !normal_deps(&text).iter().any(|d| d == "fs-conformance"),
                "{name} depends on fs-conformance in normal dependencies"
            );
        }
    }
}

/// `fs-conformance` itself depends on nothing but the allowed crates.
#[test]
fn own_dependencies_minimal() {
    let (_, text) = manifests()
        .into_iter()
        .find(|(n, _)| n == "fs-conformance")
        .expect("own manifest");
    for d in normal_deps(&text) {
        assert!(
            ALLOWED_OWN.contains(&d.as_str()),
            "unexpected dependency {d}"
        );
    }
}

/// The scanner finds normal dependencies and ignores dev ones.
#[test]
fn scanner_self_test() {
    let bad = "[dependencies]\nfs-conformance = { path = \"../x\" }\n";
    assert_eq!(normal_deps(bad), ["fs-conformance"]);
    assert_eq!(
        normal_deps("[dependencies.fs-conformance]\npath = \"x\"\n"),
        ["fs-conformance"]
    );
    assert_eq!(
        normal_deps("[target.'cfg(unix)'.dependencies]\nfs-conformance = \"1\"\n"),
        ["fs-conformance"]
    );
    assert_eq!(
        normal_deps("[dev-dependencies]\nfs-conformance = \"1\"\n[package]\nname = \"a\"\n"),
        Vec::<String>::new()
    );
}
