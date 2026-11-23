//! The committed generated code (`src/types_capnp.rs`) must equal what the official compiler (`capnpc-embedded`, capnp
//! 1.4.0 as WASM) produces, and what our own compiler (`capnp-compiler`) produces. Output goes to the test target
//! (`CARGO_TARGET_TMPDIR`), never to `/tmp` (memory).

use std::path::{Path, PathBuf};

fn schema_dir() -> PathBuf {
    PathBuf::from(fs_types::SCHEMA_DIR)
}

fn generate(request: &[u8], tag: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("fs-types-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    capnpc::codegen::CodeGenerationCommand::new()
        .output_directory(&out)
        .run(request)
        .unwrap();
    out
}

fn assert_same(dir: &Path, who: &str) {
    let committed = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fs-types/src/types_capnp.rs"),
    )
    .unwrap();
    let fresh = std::fs::read_to_string(dir.join("types_capnp.rs")).unwrap();
    let same = committed == fresh;
    let _ = std::fs::remove_dir_all(dir);
    assert!(same, "types_capnp.rs differs from the output of {who}");
}

#[test]
fn official_compiler_output_is_committed() {
    let schema = schema_dir();
    let mut cmd = capnpc_embedded::CompileCommand::new();
    cmd.file(schema.join("types.capnp"));
    let request = cmd.src_prefix(&schema).compile().expect("official compile");
    assert_same(&generate(&request, "official"), "capnpc-embedded");
}

#[test]
fn own_compiler_output_is_committed() {
    let schema = schema_dir();
    let mut cmd = capnp_compiler::CompileCommand::new();
    cmd.src_prefix(&schema);
    cmd.file(schema.join("types.capnp"));
    let request = cmd.run_bytes().expect("own compile");
    assert_same(&generate(&request, "own"), "capnp-compiler");
}

#[test]
fn schema_dir_holds_the_schema_and_its_file_id() {
    let text =
        std::fs::read_to_string(Path::new(fs_types::SCHEMA_DIR).join("types.capnp")).unwrap();
    assert!(
        text.contains("@0x916fad50f1c14440;"),
        "file id of types.capnp"
    );
}

/// What a dependent crate does (README, "Using the schema"): its own schema says `import "/types.capnp"`, the compiler
/// gets `SCHEMA_DIR` as import path, and the generated code points at `::fs_types::types_capnp` for the types.
#[test]
fn a_dependent_schema_imports_the_types_through_schema_dir() {
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("fs-types-dep-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("topic.capnp"),
        "@0x9a1b2c3d4e5f6071;\nusing T = import \"/types.capnp\";\nstruct Probe { edit @0 :T.Edit; }\n",
    )
    .unwrap();
    let mut cmd = capnp_compiler::CompileCommand::new();
    cmd.src_prefix(&dir);
    cmd.import_path(fs_types::SCHEMA_DIR);
    cmd.file(dir.join("topic.capnp"));
    let request = cmd.run_bytes().expect("dependent compile");
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let mut gen_cmd = capnpc::codegen::CodeGenerationCommand::new();
    gen_cmd.output_directory(&out);
    gen_cmd.crates_provide_map(std::collections::HashMap::from([(
        0x916f_ad50_f1c1_4440_u64,
        "fs_types".to_owned(),
    )]));
    gen_cmd.run(&request[..]).unwrap();
    let code = std::fs::read_to_string(out.join("topic_capnp.rs")).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        code.contains("::fs_types::types_capnp::edit"),
        "type comes from fs-types, not a copy"
    );
}
