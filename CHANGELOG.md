# Changelog

Version = tag.

## 0.0.1

- First version of the repository. Up to `fs-interfaces` v0.0.26 this code was part of the repository `fs-interfaces`: the crate `fs-types` (14 commits of history), the schema `schema/types.capnp` (5), the module `values` (2) and the generated `types_capnp.rs` are taken over with their history (`git filter-branch` on the tag v0.0.26 = 9df19ee323145119e89c003e64982be68e9fc1da); one commit moves them to the new layout. The version restarts at 0.0.1 (the old crate was 0.0.26 by the version of fs-interfaces).
- Layout: crate `crates/fs-types` with `schema/types.capnp` (file id `@0x916fad50f1c14440`, unchanged), `src/types_capnp.rs` (committed, byte-identical to the file of fs-interfaces v0.0.26) and `src/values/`; `crates/regenerate` rewrites the generated code. New: the constant `SCHEMA_DIR` (path of the schema folder, for crates that import `types.capnp`).
- Paths: `fs_interfaces::types_capnp` is now `fs_types::types_capnp`, `fs_interfaces::values` is now `fs_types::values`. The crate no longer depends on `fs-interfaces`; `idna` is a plain dependency (the feature `values` is gone).
- Removed: the feature `rpc` with `CatalogServer` (it moves to `fs-rpc`), the dependency `capnp-rpc`.
- Kept unchanged: the list `Capabilities` of `paging.rs` (users: `person_directory_v1`, `tasks_v1`, `appointments_v1`); how it relates to the provider rule of `fs-interfaces` is open (V3).
- Auf Deutsch: Erste Version des Repositorys. Bis `fs-interfaces` v0.0.26 war der Code Teil des Repositorys `fs-interfaces` (Crate `fs-types`, Schema `types.capnp`, Modul `values`, erzeugter Code `types_capnp.rs`); er kommt mit seiner Historie hierher (`git filter-branch` auf dem Tag v0.0.26, 9df19ee323145119e89c003e64982be68e9fc1da), ein Commit schiebt ihn ins neue Layout. Die Version beginnt bei 0.0.1. Neu: `SCHEMA_DIR`. Pfade: `fs_interfaces::types_capnp` heißt jetzt `fs_types::types_capnp`, `fs_interfaces::values` heißt `fs_types::values`. Entfernt: Feature `rpc` samt `CatalogServer` (geht nach `fs-rpc`), `capnp-rpc`; keine Abhängigkeit mehr auf `fs-interfaces`. Unverändert bleibt die Liste `Capabilities` in `paging.rs` (Nutzer `person_directory_v1`, `tasks_v1`, `appointments_v1`); das Verhältnis zur Provider-Regel in `fs-interfaces` ist offen (V3).
