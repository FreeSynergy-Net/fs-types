# FreeSynergy Types: the values every program shares

**This is a building block. It is why an e-mail address, a time zone or a reference to a person means the same thing in every program, and why each of them is checked the same way everywhere.**

FreeSynergy Types holds the small, exact values that all FreeSynergy programs exchange: e-mail address, phone number, name, postal address, place (`geo:` point and named location), date and date-time, instant, zone, language tag, link, image, key, choice, keywords, reference to a person, object id, revision, media type, file reference. Each one is a value object: it is valid when it exists, it is compared by value, it can tell you why a typed text is not valid, and it can suggest a repair. Next to the values it holds the type description every program reports about its own data (fields, commands, what is required) and the Cap'n Proto schema `types.capnp` that gives all of this one wire form.

What is different here: the values and their wire schema are one library with no FreeSynergy dependency. It sits below the interface contracts (fs-interfaces), the object store (fs-objects) and the calendar adapter (fs-jscalendar), so none of them has to wait for another to move. It is written to be reused: anyone who needs a checked e-mail address, a zone or a person reference can take this crate without taking our topic contracts along.

## What it can do today

Version: `0.0.7` (see the tags of this repository for the released states).

- Check, normalise and suggest for the basic types of ADR 0066 (`BasicValue`, `BasicRules`), with problem codes instead of sentences.
- Shared value types of the topic crates: `Utc`, `LocalDateTime`, `Zone`, `PersonRef`, `Attendee`, `ContactAddress`, `ObjectId`, `Revision`, `MediaType`, `FileRef`, `Page`, `PageCursor`, `GeoPoint`, `PositionFix`, `Location`.
- Contact values `values::{EmailAddress, PhoneNumber}` with the trait `ContactValue` (international domains, IDNA).
- Type descriptions (`TypeDescription`, `TypeRegistry`, `Edit`) with record checks, readable and writable on the wire.
- Codes with typed parameters instead of sentences (`coded`: `ErrorCode`, `Coded`, `Code`, `Param`), the shared form of every failure report (ADR 0057). New in `v0.0.2`.
- The workspace also holds `fs-conformance` (`crates/fs-conformance`), the frame of every conformance run: rules, mutants that prove the rules can fail, reports as codes. It is used only as a dev-dependency, in a `*-testdata` crate, or optional behind the feature `conformance`; see its README. New in `v0.0.2` (no tag of its own).
- The schema `types.capnp` (file id `@0x916fad50f1c14440`) and its generated Rust (`types_capnp`).

## What is not there yet

- Not published on crates.io; use it from the registry `freesynergy` (see "For developers"). Until the first registry entry exists, 0.0.7 cannot be fetched.
- The `TypeCatalog` server is not here (it is transport and moves to fs-rpc) and the provider rule `Capabilities` stays in fs-interfaces (the plain list `Capabilities` in this crate stays for now; how the two relate is open).
- The shared stand-ins for actor kind, bot reference and word code come in a later tag.

## Part of FreeSynergy

FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours. The documentation, the manifesto and the overall plan live in [fs-documentation](https://github.com/FreeSynergy-Net/fs-documentation). Written with the help of Claude (Anthropic).

## For developers

- Workspace `fs-types`: the library crate `fs-types` (`crates/fs-types`), `fs-conformance` (`crates/fs-conformance`, the conformance frame) and `regenerate` (`crates/regenerate`, not published).
- Up to fs-interfaces v0.0.26 this code was part of the repository fs-interfaces (crate `fs-types`, schema `types.capnp`, module `values`); the history of those paths is kept. `fs_interfaces::values::X` is now `fs_types::values::X` (also `fs_types::X`), `fs_interfaces::types_capnp` is now `fs_types::types_capnp`. The decision is ADR 0093 (accepted, strategy 2026-11-16).
- The generated code `src/types_capnp.rs` is committed (ADR 0040); `cargo run -p regenerate` rewrites it with our own compiler (fs-capnpc, tag v0.0.1, pinned). The tests `tests/generated_matches_official.rs` compare it with our compiler and with the official one.
- Use it from the registry `freesynergy` (from 0.0.7): `fs-types = { version = "~0.0.7", registry = "freesynergy" }`, and in `.cargo/config.toml` of the consuming repo `[registries.freesynergy]` with `index = "https://github.com/FreeSynergy-Net/fs-crates"` (the same string everywhere, see fs-release `docs/crate-index.md`). A version range replaces the git tag, so a new type does not force every consumer to re-tag; one build still holds one copy of the types.
- Gates: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings -W clippy::pedantic`, `cargo test --workspace --all-features --locked`.
- Licence: MIT OR Apache-2.0.

### Using the schema

A crate with its own schemas that need the basic types (for example `fs-interfaces`) must not copy `types.capnp`. It depends on fs-types and reads the schema folder from the source that cargo unpacked of the pinned version:

1. The schema says `import "/types.capnp";` (a leading slash: found through the import path, not next to the file).
2. The crate that regenerates its code (a build step or a `regenerate` binary) depends on fs-types, passes `fs_types::SCHEMA_DIR` as import path (`CompileCommand::import_path`), and tells the code generator that the file id `0x916fad50f1c14440` is provided by the crate `fs_types` (`CodeGenerationCommand::crates_provide_map`). The generated code then names `::fs_types::types_capnp::...` and there is no second copy of the types.
3. `SCHEMA_DIR` is `concat!(env!("CARGO_MANIFEST_DIR"), "/schema")`: the absolute path of the crate source at the time fs-types was compiled, so for a registry dependency it points into cargo's unpacked crate (`$CARGO_HOME/registry/src/...`). It is valid as long as that source exists, which is the case on the machine that just built it. Use `SCHEMA_DIR` only at build time (a `build.rs` or the `regenerate` binary), never at run time of a shipped program.

`tests/generated_matches_official.rs` (`a_dependent_schema_imports_the_types_through_schema_dir`) runs exactly this on a small schema.

A `match` on `BasicType` (generated, new kinds are appended) or on `TypeProblem` (`#[non_exhaustive]`, new codes are appended) needs a `_` arm.

## Auf Deutsch

FreeSynergy Types ist die unterste Bibliothek der Familie: die kleinen, genauen Werte, die alle Programme austauschen (E-Mail-Adresse, Telefonnummer, Name, Anschrift, Ort, Datum, Zeitpunkt, Zone, Sprachkennzeichen, Link, Bild, Schlüssel, Auswahl, Schlagwörter, Verweis auf eine Person, Objekt-Kennung, Revision, Medientyp, Datei-Verweis). Jeder Wert ist ein Wertobjekt: gültig, sobald er existiert, nach Wert verglichen, mit Grund, warum ein getippter Text nicht passt, und mit Verbesserungsvorschlag. Dazu kommen die Typbeschreibung, die jedes Programm über seine Daten meldet, und das Cap'n-Proto-Schema `types.capnp`, das allem eine gemeinsame Drahtform gibt. Die Bibliothek hat keine Abhängigkeit auf ein anderes FreeSynergy-Repository; sie liegt unter den Schnittstellenverträgen (fs-interfaces), dem Objektspeicher (fs-objects) und dem Kalender-Adapter (fs-jscalendar) und ist so geschrieben, dass auch andere Projekte sie nutzen können. Bis fs-interfaces v0.0.26 war der Code Teil des Repositorys fs-interfaces (die Historie dieser Pfade ist erhalten); die Entscheidung steht in ADR 0093 (angenommen, Strategie 2026-11-16). Wer eigene Schemata hat, die `types.capnp` brauchen, liest den Schema-Ordner über `fs_types::SCHEMA_DIR` aus dem gepinnten Stand (Abschnitt "Using the schema"), kopiert die Datei aber nie. Ein `match` auf `BasicType` (erzeugt, neue Arten werden angehängt) oder auf `TypeProblem` (`#[non_exhaustive]`, neue Codes werden angehängt) braucht einen `_`-Arm. Lizenz: MIT OR Apache-2.0.
