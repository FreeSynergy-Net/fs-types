//! Gate "Crate version = tag" (ADR 0064 no. 5, ENTSCHEID-Kaskade K-4).
//!
//! On a tag run (`GITHUB_REF_TYPE=tag`, `GITHUB_REF_NAME=vX.Y.Z`) the crate version must equal the tag and
//! `CHANGELOG.md` must have a non-empty section `## X.Y.Z`. Off a tag the real check does nothing. Why it exists: tag
//! v0.0.6 was cut while `Cargo.toml` still said 0.0.5 (no release workflow of the repository checks it), so the
//! version of the built crate and the tag disagreed. The rule is a plain function so the red cases are tested too.

/// `Ok` when the run is no tag run or when version and changelog agree with the tag; `Err` names the cause.
fn check(ref_type: &str, ref_name: &str, version: &str, changelog: &str) -> Result<(), String> {
    if ref_type != "tag" {
        return Ok(());
    }
    let Some(tag_version) = ref_name.strip_prefix('v') else {
        return Err(format!("tag '{ref_name}' does not start with 'v'"));
    };
    if tag_version != version {
        return Err(format!(
            "tag version {tag_version} differs from the crate version {version} in Cargo.toml"
        ));
    }
    let heading = format!("## {version}");
    let mut lines = changelog.lines();
    if !lines.any(|l| l.trim_end() == heading) {
        return Err(format!("CHANGELOG.md has no section '{heading}'"));
    }
    let body_empty = lines
        .take_while(|l| !l.starts_with("## "))
        .all(|l| l.trim().is_empty());
    if body_empty {
        Err(format!("CHANGELOG.md section '{heading}' is empty"))
    } else {
        Ok(())
    }
}

const LOG: &str = "# Changelog\n\n## 0.0.7\n\n- something\n\n## 0.0.6\n\n- older\n";

#[test]
fn no_tag_run_is_never_checked() {
    assert_eq!(check("branch", "main", "0.0.5", ""), Ok(()));
}

#[test]
fn tag_equal_to_version_with_notes_is_green() {
    assert_eq!(check("tag", "v0.0.7", "0.0.7", LOG), Ok(()));
}

#[test]
fn tag_differing_from_version_is_red() {
    // the v0.0.6 case: tag newer than Cargo.toml
    let e = check("tag", "v0.0.6", "0.0.5", LOG).unwrap_err();
    assert!(e.contains("differs"), "{e}");
}

#[test]
fn missing_or_empty_changelog_section_is_red() {
    assert!(
        check("tag", "v0.0.8", "0.0.8", LOG)
            .unwrap_err()
            .contains("no section")
    );
    assert!(
        check("tag", "v0.0.7", "0.0.7", "## 0.0.7\n\n\n## 0.0.6\n- x\n")
            .unwrap_err()
            .contains("empty")
    );
    assert!(
        check("tag", "0.0.7", "0.0.7", LOG)
            .unwrap_err()
            .contains("'v'")
    );
}

/// The real gate: reads the CI environment and the files of this repository.
#[test]
fn this_checkout_agrees_with_its_tag() {
    let ref_type = std::env::var("GITHUB_REF_TYPE").unwrap_or_default();
    let ref_name = std::env::var("GITHUB_REF_NAME").unwrap_or_default();
    let changelog =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../CHANGELOG.md"))
            .unwrap_or_default();
    if let Err(e) = check(&ref_type, &ref_name, env!("CARGO_PKG_VERSION"), &changelog) {
        panic!("version gate: {e}");
    }
}

/// Every path dependency of a member names the crate version (`version = "X"` next to `path`): a packed crate asks
/// the registry for exactly the version the workspace has. `Err` lists the lines that differ or have no `version`.
fn check_path_deps(manifest: &str, version: &str) -> Result<(), String> {
    let bad: Vec<String> = manifest
        .lines()
        .map(str::trim)
        .filter(|l| l.contains("path = \"") && !l.starts_with('#'))
        .filter(|l| !l.contains(&format!("version = \"{version}\"")))
        .map(str::to_owned)
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "path dependencies without version {version}: {bad:?}"
        ))
    }
}

#[test]
fn path_dependency_versions_are_checked() {
    let ok =
        "fs-types = { path = \"../fs-types\", version = \"0.0.7\", registry = \"freesynergy\" }\n";
    assert_eq!(check_path_deps(ok, "0.0.7"), Ok(()));
    assert!(check_path_deps(ok, "0.0.8").unwrap_err().contains("0.0.8"));
    let none = "fs-types = { path = \"../fs-types\" }\n";
    assert!(check_path_deps(none, "0.0.7").is_err());
    assert_eq!(
        check_path_deps("serde = \"1\"\n# path = \"x\"\n", "0.0.7"),
        Ok(())
    );
}

/// The real check: the manifests of the members of this workspace (any run, not only a tag run).
#[test]
fn members_path_dependencies_carry_the_crate_version() {
    let crates = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    for entry in std::fs::read_dir(crates).expect("crates dir") {
        let manifest = entry.expect("entry").path().join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        if let Err(e) = check_path_deps(&text, env!("CARGO_PKG_VERSION")) {
            panic!("{}: {e}", manifest.display());
        }
    }
}

/// The licence texts packed into each `.crate` are real copies of the root files (no symlinks, Windows): a copy that
/// drifts from the root file would ship another licence text than the repository has.
#[test]
fn member_licence_copies_equal_the_root_files() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    for name in ["LICENSE-MIT", "LICENSE-APACHE"] {
        let want = std::fs::read(format!("{root}/{name}")).expect("root licence file");
        for member in ["fs-types", "fs-conformance", "fs-records"] {
            let got = std::fs::read(format!("{root}/crates/{member}/{name}"))
                .unwrap_or_else(|e| panic!("crates/{member}/{name}: {e}"));
            assert!(got == want, "crates/{member}/{name} differs from the root {name}");
        }
    }
}
