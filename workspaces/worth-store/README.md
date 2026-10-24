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
