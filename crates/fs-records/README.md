# fs-records: one way to read the records of every program

**This is a building block. It is why a frontend described in fs-forms can show the tasks of fs-tasks, the events of fs-calendars and the people of fs-people with the same lines and no code per program.**

Every program offers the trait `Records` next to each capability (beside `TypeCatalog`, which describes). A frontend asks one data type with closed conditions (equals, any of, range, empty, term), sort keys (a field or a named order), a page size and cursor, an optional grouping and a field projection. The provider answers as the asking person:

- a record she may not read is not there (not counted, not hinted at; `get` gives `None`);
- a field she may not read is listed as hidden with a reason and has no value;
- a record may carry a lock (who, until when, why) and the commands she may run on it now.

Size and time limits (`Limits`) are settings: the program gives a default, the instance sets a ceiling and an admin may lower or raise the values below it; a value above the ceiling is refused (`Limits::admit`). `within` bounds every call in time.

`MemoryRecords` is the double for tests. With the feature `conformance` (never in `default`) the crate ships `RecordsSuite` (22 rules) and `bench_mutants` (26 deliberate faults that the suite must catch); `fs-conformance` is therefore never in the normal dependency graph of a shipped program.

Writing is a later stage (E4w) and its own trait. Written with the help of Claude (Anthropic).

License: MIT OR Apache-2.0.
