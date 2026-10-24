# Worth Store Rebuild Workspace

The active integration plan is the
[Database Foundation Roadmap](../../plans/database-foundation/roadmap.md).
It connects canonical durable publication to Relational and the production
Query host, then closes database scale and complete runtime recovery.

Milestone 1 is in place: `ServingPhysicalRuntime::canonical_commits()` is the
canonical commit journal. It publishes owner payloads at the next position
under one durable root decision, resolves positions from durable state after
recovery, and serves point and range reads. Its crash-consistency oracle is in
`worth-store-recovery-runtime/tests/production_entry` (`canonical_journal*`).
See the [Milestone 1 specification](../../plans/database-foundation/milestone-1.md).

`ServingPhysicalRuntime::anchored_records()` is the generic anchored-record
door (Milestone 2 D6): a named record holds one current value, replaced whole
at `generation + 1` under the same durable root decision and fates as the
journal, and read back verified against its anchor. Its oracle is
`anchored_records*` beside the journal's.

Milestone 2: `worth-store-relational` is the Relational home.
`RelationalStoreHome::open` creates a Store only at an unused location and
otherwise recovers it, checks the home descriptor, hands the selected
checkpoint and the journal after it to Relational for readmission, and
attaches the journal as the runtime's only durable medium. Every commit is
durable before it is live. `checkpoint()` selects a new Relational checkpoint
and completes a Store physical checkpoint. See the
[Milestone 2 specification](../../plans/database-foundation/milestone-2.md).

This workspace is the clean rebuild of Worth Store. It separates the semantic
durability roadmap from the physical database foundation so the new store does
not inherit the heap-shaped implementation boundary from the legacy crate.

Dependency direction:

- `worth-store-contracts` is the bottom shared vocabulary.
- Roadmap 2 physical foundation crates may depend on contracts and lower
  physical crates.
- Roadmap 1 semantic durable-program crates may consume physical contracts and
  facades, but physical foundation crates may not depend on semantic programs.
- `worth-store` is a thin public composition crate.
- certification crates sit at the top and consume evidence from lower crates.
