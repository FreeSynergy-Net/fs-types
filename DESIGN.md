# fs-types - Design

The basic types of ADR 0066 and the type description every program reports. Wire form: `crates/fs-types/schema/types.capnp` (this repo).

## Where it lives, and why (`coding.md` §4, §8)

Its own repository `fs-types`, the lowest library of the family: it depends on no FreeSynergy repository, only on free crates (`capnp`, `chrono`, `idna`, `iri-string`, `geo-uri`, `oxilangtag`, `unicode-normalization`). Up to `fs-interfaces` v0.0.26 it was a crate inside the repository `fs-interfaces`, and the schema `types.capnp` and the contact values (`values`) lived there; that bound the value objects to the topic contracts the wrong way round. Here the schema, its generated code (`src/types_capnp.rs`, committed, ADR 0040), the contact values and the rules are one crate with one tag, so they cannot drift apart. The decision is ADR 0093 (accepted by the strategy, 2026-11-16). A consumer writes `fs-types = { version = "~0.0.7", registry = "freesynergy" }` (registry `freesynergy` in its `.cargo/config.toml`: `index = "https://github.com/FreeSynergy-Net/fs-crates"`; ADR 0093, fs-release `docs/crate-index.md`); a crate with its own schemas that import `types.capnp` reads the schema folder from the unpacked registry crate through `SCHEMA_DIR` (README, "Using the schema").

Not here: the `TypeCatalog` server (transport, goes to `fs-rpc`), the provider rule `Capabilities` report of `fs-interfaces` (a different type from the `Capabilities` list in `paging.rs`; how the two relate is open, question V3 / ADR 0093 "open": users are `person_directory_v1`, `tasks_v1`, `appointments_v1`), the topic schemas.

## Pattern map

| Part | Pattern | Why |
|---|---|---|
| `BasicValue` and one file per kind (`text`, `contact`, `language`, `iri`, `datetime`, `place`, `key`, `keywords`, `name`, `address`, `image`, `choice`) | Value object + Template method (`parse`/`canonical` per type, `check`/`suggest` provided) | valid on construction, equal values have equal canonical text; the trait is the interface (ADR 0043) |
| `moment` (`Utc`, `LocalDateTime`, `Zone`), `person` (`PersonRef`, `Attendee`, `ContactAddress`), `ids` (`ObjectId`, `Revision`, `MediaType`, `FileRef`), `paging` (`PageCursor`, `Page`, `Capabilities`, `Problem`) | Value object (no `BasicType` kind, so not `BasicValue`) | the types the topic crates share (ADR 0085); `TypeProblem` is the union of their stand-ins |
| `language`, `iri`, `datetime`, `place`, `contact` | Adapter over a free crate (`oxilangtag`, `iri-string`, `chrono`, `geo-uri`, `idna`) | do not rebuild what exists (ADR 0015, `coding.md` §4) |
| `rule.rs` `Rule`, `BasicRules` | Registry of Strategy objects | a generated frontend knows only a field's `BasicType`; it reaches check/normalise/suggest by kind |
| `describe/` `FieldDescription`, `CommandDescription`, `TypeDescription` | Value object + Builder | read from a program, written by chaining |
| `describe/describes.rs` `DescribesType` | Registry entry point (trait first) | a program lists each data type it owns |
| `describe/registry.rs` | Registry | name to description, validated on the way in |
| `describe/validate.rs`, `check.rs` | Template method | one flow for all programs: the admin frontend and a program's own `check` command say the same |
| `describe/wire.rs` | Adapter | generated capnp types to plain structs and back |
| `values/` (`EmailAddress`, `PhoneNumber`, `ValueProblem`, trait `ContactValue`) | Value object + Template method | valid on construction, small, immutable, compared by value |

## Decisions (provisional, marked for the strategy)

* Kinds in this version: the thirteen of the task list plus `text` and `multilineText`. `onlineService`, `timeZone`, `recurrence`, `quantity`, `money`, `sound`, `video`, formatted text are in `map.md` and are appended to `BasicType` later (append only; 0.0.3 added `integer`, `duration`, `flag`, `whole` as `@17` to `@20`; 0.0.5 added `time` `@21` (RFC 3339 partial-time, strict, leap second refused) and `color` `@22` (CSS Color Level 4 hex, sRGB with alpha, canonical lower case long form); a reader meeting a newer kind treats the field as plain text).
* Canonical forms: e-mail as `values::EmailAddress` makes it; phone `+<digits>`; language RFC 5646 case; link with a host (`//`) normalised by RFC 3986 syntax rules (scheme and host lower case, dot segments removed, percent codes of unreserved characters decoded and hex digits upper case), link without a host (`mailto:`, `urn:`, `tel:`) only the scheme in lower case; reference kept as written (an identity); date `YYYY-MM-DD`; date-time with offset, `Z` for UTC (two date-times are equal only if instant and offset are; `-00:00`, "offset unknown", is refused); place as `geo:` URI; fingerprint bare upper-case hex; keywords lower case, sorted, `, `-joined; integer and whole plain decimal (no `+`, no leading zeros, `-0` is `0`; whole refuses negatives with `OutOfRange`); flag `true`/`false` (`yes`/`no` read); duration RFC 8984 upper case (`P1W`, `P1DT2H30M`, `PT0.5S`; units kept, never converted; no sign, years or months).
* Registry values (0.0.7): a word from an open registry (RFC 9553 `kind`, RFC 8984 `relativeTo`) is read whole and written back unchanged. `kind.rs` (`NameKind`, `AddressKind`) and `RelativeTo` in `location.rs` are `#[non_exhaustive]` enums with `Other(String)` (strum `EnumString`, default variant); on the wire the word is `Text`. Capnp enums are only for lists of our own. `ContactAddress` takes any absolute URI except the schemes of the one unsafe-link list (`iri.rs`, shared with `Link`).
* Limits (ours, provisional): text 4096 chars, multi-line 65536, URI 2048 bytes, keyword 64 chars and 256 per set, name/address parts 1024 chars, fraction of a duration's seconds 9 digits (RFC 8984 allows any number; ours, provisional).
* Unsafe schemes (provisional): a `Link` may not use `javascript:`, `vbscript:` or `data:`, an `Image` not `javascript:` or `vbscript:` (a frontend may make them clickable); `TypeProblem::UnsafeScheme`. Characters refused in every text: control characters, U+2028/U+2029 and the bidirectional overrides and isolates; multi-line texts and `Address.full` keep `\n`.
* `Reference` has no scheme block (an id is an identity, not a target to open); a frontend must not make references clickable by scheme alone, it shows the target's compact component. A reference target in a description is a bare type name (UpperCamel), no program prefix.
* A description read from the wire is validated leniently: a kind or command kind this version reads as `unknown` passes (with its list or target), so one newer program cannot make the whole catalog unreadable; `register`/`add` stay strict.
* A name separator is literal (RFC 9553 section 2.2.1.2), so `", "` must carry its own space.
* A name typed on one line is never split into given and surname (wrong for many names); components come from a form.
* Problems are codes (`TypeProblem::code`, translation key suffix `problem.<code>`), never sentences (ADR 0057, `coding.md` §2a).
* Labels in a description are translation keys: lower-case words of letters and digits joined by `.` or `-`.

## Commands, change and supported (decided 2026-11-10, written in `types.capnp`)

* **`supported` per command** (`CommandDescription.supported @5`): a description lists every command the type is meant to have; one the core lacks has `supported = false` (`CommandDescription::unsupported()`), a frontend shows it locked and frees it when the core reports it. Unset on the wire reads as false (the safe side), so a writer must set it; the builder default is true.
* **`change`** = `Edit` = list of `FieldEdit` (path, basic type, values). Missing field = unchanged; no values = cleared; else replaced (`Edit::apply`). `Edit::check` validates against the description (unknown field, wrong type, several values for a single field, clearing a required field, duplicate path, rule of the basic type) and returns canonical values. `FieldEdit::pointer` gives the PatchObject key (RFC 8984 §1.4.9: steps joined by `/`, escaped as RFC 6901, no leading `/`). Values empty after trimming count as not given. The record id travels in the program's call, not in the `Edit`. A `change` command has no input fields (`ChangeWithInput`). Paths deeper than one step are `Unsupported` in `check` (`RecordProblem::NestedPath`): descriptions have flat fields.
* **`readOnly` per field** (`FieldDescription.readOnly @7`, `FieldDescription::read_only()`): the program sets it itself (uid, created, updated, prodId); a generated form shows it as display only; `Edit::check` refuses it (`RecordProblem::ReadOnly`). Unset = editable. `check_record` does not demand a required read-only field and does not refuse a value for it (an import may bring a uid); the program decides. A provider must check `change` against the description with `Edit::check`, the `Edit` alone cannot tell.
* **list = find**: `CommandDescription::standard_find` (input `term`, empty = list; paging with the `Cursor` of `common.capnp` in the contract call).
* **Record commands** (`TypeDescription::record_commands`): input has a `reference` to the type itself (or without target), plus `change` and `delete`; `type_commands` is the rest.

## JSContact and vCard: free crate checked (task point 4)

Candidate `calcard` 0.3.14 (Stalwart Labs; `Apache-2.0 OR MIT`; first released 2025-05, updated 2026-09-15, 15 releases in 2026-08/09, about 89000 downloads; repository `stalwartlabs/calcard`). Described as "iCalendar/JSCalendar and vCard/JSContact parsing, building and conversion library". Tried in a throw-away crate (not in this repo): parse a vCard 4.0 text, `VCard::into_jscontact::<String, String>()` gives a JSContact JSON value (`name.components`, `emails`, `phones`, `addresses`, `vCard.convertedProperties` for what vCard keeps), and `JSContact::into_vcard()` writes vCard 4.0 back (`FN`, `N`, `EMAIL`, `TEL;VALUE=URI`, `ADR` with `JSCOMPS`, `PROP-ID`). Other crates seen: `jscontact` 0.2.1 (RFC 9553 types only, 7000 downloads, last 2025-01), `vcard` 0.6.0 (vCard 4.0 RFC 6350 and 9554, parse and generate, no JSContact), `vcard4` 0.7.3 (parser only). **Result: use `calcard` for vCard import and export in `fs-people` (thread P); do not build our own.** Not done here: no dependency added to this repo (nothing in `fs-types` needs it), the field-by-field coverage of RFC 9553 against `description.md` is thread P's check; `calcard` pulls in more than we need (iCalendar), accepted for now; its types are generic over the id type, `String` works.

## Shared types of the topic crates (v0.0.22)

`moment.rs` (`Utc`, `LocalDateTime`, `Zone`), `person.rs` (`PersonRef`, `Attendee`, `ContactAddress`), `ids.rs`, `paging.rs`: Value objects, Pattern lines in each module; `TypeProblem` is the union of all stand-ins. Names, mapping and deliberate differences: `DESIGN.md` of the repository, section v0.0.22.

## State

| Area | State |
|---|---|
| `types.capnp` draft 0.1: `BasicType` (21 values), `Name`, `Address`, `Image` and their kinds, type description (`FieldDescription`, `CommandDescription`, `TypeDescription`, `CommandKind`), `TypeCatalog.describe` | done, draft |
| Rules for all 16 kinds: check, normalise (suggest where it makes sense: e-mail domains, language, link scheme, reference, keywords, choice) | done, tested (each kind, incl. the cases the independent review found) |
| `BasicRules` registry, unknown kind as text | done, tested |
| Description: builders, validation, registry, record check, wire round trip | done, tested (the `CatalogServer` moved to `fs-rpc`) |

## Unsupported (named, not faked)

* `Edit` paths deeper than one step (map key, then property): the wire form carries them, `Edit::check` refuses them (`NestedPath`) because descriptions have flat fields; nested typing comes with nested descriptions.

* `onlineService`, `timeZone`, `recurrence`, `quantity`, `money`, `colour`, `sound`, `video`, formatted text: not yet in `BasicType`.
* IANA time zones: `Zone` checks the form only; that a name exists in the tz database, and the conversion between a zone and UTC, are not here (a recurrence or a calendar needs them). `LocalDateTime` has no fraction of a second (JSCalendar allows one; an importer cuts it off and says so).
* Language: well-formed only, the IANA subtag registry is not consulted. Address: country code checked for shape, not against the ISO 3166-1 list. Place: points only, see `GeoPoint` for the plain coordinate (no `GeoJSON` shapes; unknown `geo:` parameters are dropped; the poles are not canonicalised). Key: OpenPGP fingerprints only (no SSH, age, X.509). Phone and e-mail: no existence check (a network matter).
* Suggestions: phone, date, date-time, place, key, name, address, image, text offer none (`suggest` returns an empty list on purpose).
* Components (`input`, `display`, `compact`, `cell`) per basic type are thread C's, in `fs-components`; nothing of the look is here.

## Secret<T> (unreleased)

`Secret<T: Zeroize>` is the one type for credentials, keys and tokens in the Rust code of every program (layer 0, so that `fs-objects`,
the programs and the tools of `fs-interfaces` can use it without a new dependency chain). Why here and not in a program: two programs already wrote
their own (`Password` in `fs-messages-mail`, `Secret` in `fs-broadcast-core`), the rule is the same everywhere, and a type that keeps a secret out of
every answer, log and export has to be the same type everywhere to be searched for. Form: no `derive`, hand-written `Debug`, no `Display`, no
`Clone`, no `Serialize`; the value is wiped on drop; `expose_secret()` is the only door. How it is enforced without trust: `compile_fail`
doctests here; in a program the `disallowed-methods` entry named in the CHANGELOG, `#[allow]` only in named modules; and the canary test of
`fs-mcp` (a marker planted in a `Secret` must not come out of any answer, error text or log). What it does not do: stop a copy made
before (`expose_secret().clone()`), or a swap to disk.

## Interface name and version (unreleased)

`fs_types::interface` holds `InterfaceVersion` (`name@x.y.z`) and `InterfaceReq` (`name@<req>`, Cargo grammar, comma between comparisons): the one place for "name and version" (ADR 0045, 0077). Pattern: Value object. The numbers are read by `semver` 1.0.28 only; a two-part version (`tree@1.0`) is refused. The name follows `[a-z][a-z0-9.-]*`. `default_req()` is the generator's preset: `>=0.0.N, <0.1.0` for the `0.0.x` line, the Cargo caret from `0.1` on.
