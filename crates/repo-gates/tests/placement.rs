//! Condition O1 (made exact by decision b7, 17.11.): `fs-conformance` never stands in the normal graph of a shipped
//! program. Allowed are `[dev-dependencies]`, a normal dependency of a `*-testdata` crate, and `optional = true` behind a
//! feature `conformance` that no `default` lists. The scan reads the `Cargo.toml` files of the workspace members; it also
//! finds a renamed dependency (`x = { package = "fs-conformance" }`). It sees this workspace only: the check over the
//! program repos is a gate of fs-release (`cargo tree -i`), not this test.

use std::fs;
use std::path::PathBuf;

const ALLOWED_OWN: [&str; 3] = ["async-trait", "fs-types", "fs-conformance"];
const NAME: &str = "fs-conformance";

/// One normal or build dependency: its key and everything written about it (inline value or table body).
struct Dep {
    key: String,
    body: String,
}

/// The normal dependency tables (`[dependencies]`, `[target.*.dependencies]`, `[dependencies.NAME]`,
/// `[build-dependencies]`) of a manifest, dev tables left out.
fn normal_deps(manifest: &str) -> Vec<Dep> {
    let mut out: Vec<Dep> = Vec::new();
    let mut in_normal = false;
    let mut in_named = false;
    for line in manifest.lines().map(str::trim) {
        if let Some(head) = line.strip_prefix('[') {
            let head = head.trim_end_matches(']');
            in_named = false;
            in_normal = false;
            if let Some((table, key)) = head.split_once("dependencies.") {
                if !table.contains("dev")
                    && (table.is_empty() || table.ends_with('.') || table.ends_with("build-"))
                {
                    out.push(Dep {
                        key: key.to_owned(),
                        body: String::new(),
                    });
                    in_named = true;
                }
                continue;
            }
            let tail = head.rsplit('.').next().unwrap_or(head);
            in_normal = tail == "dependencies" || tail == "build-dependencies";
        } else if line.starts_with('#') || line.is_empty() {
        } else if in_named {
            if let Some(last) = out.last_mut() {
                last.body.push_str(line);
                last.body.push('\n');
            }
        } else if in_normal && let Some((key, value)) = line.split_once('=') {
            out.push(Dep {
                key: key.trim().to_owned(),
                body: value.trim().to_owned(),
            });
        }
    }
    out
}

/// The text of the array that belongs to `key` in the table `[features]` (empty if there is none).
fn feature(manifest: &str, key: &str) -> String {
    let mut in_features = false;
    let mut text = String::new();
    let mut open = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_features = line == "[features]";
            open = false;
        } else if in_features
            && (open || line.split_once('=').is_some_and(|(k, _)| k.trim() == key))
        {
            text.push_str(line);
            open = !line.contains(']');
        }
    }
    text
}

/// Whether the dependency is `fs-conformance`, also under another key.
fn is_conformance(dep: &Dep) -> bool {
    dep.key == NAME || dep.body.contains(&format!("package = \"{NAME}\""))
}

/// The reason a manifest of crate `krate` may not list `fs-conformance` as it does, or `Ok`.
fn placement(krate: &str, manifest: &str) -> Result<(), String> {
    for dep in normal_deps(manifest).iter().filter(|d| is_conformance(d)) {
        if krate.ends_with("-testdata") {
            continue;
        }
        let gate = feature(manifest, "conformance");
        let default = feature(manifest, "default");
        let named = |text: &str| {
            text.contains(&format!("dep:{}", dep.key)) || text.contains(&format!("\"{}\"", dep.key))
        };
        if !dep.body.contains("optional = true") || !named(&gate) {
            return Err(format!(
                "{krate} lists {NAME} in normal dependencies: use dev-dependencies, a *-testdata crate, or optional behind the feature conformance"
            ));
        }
        if named(&default) || default.contains("\"conformance\"") {
            return Err(format!(
                "{krate} turns the feature conformance on by default"
            ));
        }
    }
    Ok(())
}

fn manifests() -> Vec<(String, String)> {
    let crates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut v = Vec::new();
    for entry in fs::read_dir(crates).expect("crates dir") {
        let dir = entry.expect("entry").path();
        if let Ok(text) = fs::read_to_string(dir.join("Cargo.toml")) {
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

/// No other workspace member lists `fs-conformance` against the rule.
#[test]
fn placement_of_members() {
    for (name, text) in manifests() {
        if name != NAME {
            placement(&name, &text).unwrap_or_else(|e| panic!("{e}"));
        }
    }
}

/// `fs-conformance` itself depends on nothing but the allowed crates.
#[test]
fn own_dependencies_minimal() {
    let (_, text) = manifests()
        .into_iter()
        .find(|(n, _)| n == NAME)
        .expect("own manifest");
    for d in normal_deps(&text) {
        assert!(
            ALLOWED_OWN.contains(&d.key.as_str()),
            "unexpected dependency {}",
            d.key
        );
    }
}

/// The scanner finds normal dependencies (also renamed ones) and ignores dev ones.
#[test]
fn scanner_self_test() {
    let keys = |m: &str| {
        normal_deps(m)
            .into_iter()
            .map(|d| d.key)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        keys("[dependencies]\nfs-conformance = { path = \"../x\" }\n"),
        [NAME]
    );
    assert_eq!(
        keys("[dependencies.fs-conformance]\npath = \"x\"\n"),
        [NAME]
    );
    assert_eq!(
        keys("[target.'cfg(unix)'.dependencies]\nfs-conformance = \"1\"\n"),
        [NAME]
    );
    let none: Vec<String> = Vec::new();
    assert_eq!(
        keys("[dev-dependencies]\nfs-conformance = \"1\"\n[package]\nname = \"a\"\n"),
        none
    );
    assert_eq!(
        keys("[target.'cfg(unix)'.dev-dependencies]\nfs-conformance = \"1\"\n"),
        none
    );
    let renamed =
        normal_deps("[dependencies]\nx = { package = \"fs-conformance\", path = \"y\" }\n");
    assert!(is_conformance(&renamed[0]));
}

/// The rule of A4: dev, testdata and optional behind the feature are fine; the rest is not.
#[test]
fn placement_rule() {
    let normal = "[dependencies]\nfs-conformance = { path = \"../x\" }\n";
    assert!(placement("fs-tasks", normal).is_err());
    assert!(placement("fs-tasks-testdata", normal).is_ok());
    assert!(placement("fs-tasks", "[dev-dependencies]\nfs-conformance = \"1\"\n").is_ok());
    let renamed = "[dependencies]\nx = { package = \"fs-conformance\", path = \"y\" }\n";
    assert!(placement("fs-tasks", renamed).is_err());
    let gated = "[dependencies]\nfs-conformance = { path = \"x\", optional = true }\n[features]\nconformance = [\"dep:fs-conformance\"]\n";
    assert!(placement("fs-tasks", gated).is_ok());
    let on = format!("{gated}default = [\"conformance\"]\n");
    assert!(placement("fs-tasks", &on).is_err());
    let ungated = "[dependencies]\nfs-conformance = { path = \"x\", optional = true }\n";
    assert!(placement("fs-tasks", ungated).is_err());
}
