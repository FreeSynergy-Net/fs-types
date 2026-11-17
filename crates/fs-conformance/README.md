# fs-conformance: the frame of every conformance run

**This is a building block. It is why the rules of one FreeSynergy contract (a tasks provider, a calendar provider) are run, proven and reported the same way everywhere.**

A building block ships its own rules: a `Suite` of `Rule`s over a fixture type it defines itself. This crate only runs them (`run`), proves that every rule can fail (`prove` with `Mutant`s: one deliberately wrong behaviour, and the rules that must catch it), and reports codes with typed parameters, never sentences (`Report`, `Code`, from `fs_types::coded`). It knows no program, no transport and no window.

Rules of the frame:

- A skipped rule is never a passed rule: it is listed with the need that was missing.
- A rule that no mutant breaks proves nothing: `prove` reports it. Ids of rules and mutants are checked for form and doubles (codes `rule-id-invalid`, `duplicate-rule`, `mutant-id-invalid`, `duplicate-mutant`).
- Every mutant runs on a fresh fixture.

## Where it may stand (condition O1)

Never in the normal dependency graph of a shipped program. Allowed:

1. `[dev-dependencies]` of the crate under test;
2. a normal dependency of a `*-testdata` crate (the fakes and suites of a topic);
3. `optional = true` behind a feature `conformance` that is in no `default`.

`tests/placement.rs` checks the members of this workspace (also a renamed dependency); the check over the program repos is a gate of fs-release (`cargo tree -e normal,build -i fs-conformance`).

## Writing rules

A fixture type written as `dyn Trait` in `impl Rule<dyn Trait>` needs `&mut (dyn Trait + 'static)` in `check`, or an alias `type Fx = dyn Trait;`. Generic rules `impl<F: Trait + ?Sized> Rule<F>` avoid the problem and are the recommended form.

## What is not there yet

Not published on crates.io; the first tag comes with the next fs-types tag. Programs still carry their own conformance code; they move over one by one (people, calendars, tasks, groups).

## Part of FreeSynergy

FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours: [fs-documentation](https://github.com/FreeSynergy/fs-documentation). Written with the help of Claude (Anthropic). Licence: MIT OR Apache-2.0.

## Auf Deutsch

Der Rahmen jedes Konformitätslaufs in FreeSynergy: Regeln eines Vertrags laufen (`run`), werden mit Mutanten bewiesen (`prove`: jede Regel muss scheitern können) und als Codes mit typisierten Parametern gemeldet, nie als Sätze. Er steht nie im normalen Abhängigkeitsgraphen eines ausgelieferten Programms: erlaubt sind `[dev-dependencies]`, ein `*-testdata`-Crate oder `optional` hinter dem Feature `conformance`, das in keinem `default` steht. Geschrieben mit Hilfe von Claude (Anthropic).
