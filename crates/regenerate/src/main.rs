//! Regenerates `crates/fs-types/src/types_capnp.rs` from `crates/fs-types/schema/types.capnp` with our own pure-Rust
//! compiler (`capnp-compiler`, ADR 0040). No C++ toolchain needed. Run from the repository root.
//!
//! ```text
//! cargo run -p regenerate
//! ```
//!
//! Pattern: Command (one run, one output).

use capnp_compiler::CompileCommand;

fn main() {
    let mut cmd = CompileCommand::new();
    cmd.src_prefix("crates/fs-types/schema");
    cmd.file("crates/fs-types/schema/types.capnp");
    let request = cmd.run_bytes().expect("schema compilation failed");
    std::fs::create_dir_all("crates/fs-types/src").expect("create src");
    capnpc::codegen::CodeGenerationCommand::new()
        .output_directory("crates/fs-types/src")
        .run(&request[..])
        .expect("code generation failed");
    println!("regenerated crates/fs-types/src/types_capnp.rs");
}
