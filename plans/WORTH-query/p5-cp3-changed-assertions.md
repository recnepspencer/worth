# CP3 assertion changes

This ledger lists existing court assertions CP3 changed and any typed-error or stored-shape changes, landing by landing. Purely new courts are listed at the end of each landing.

## L1a (Native authored stages)

### `ForeignFreshCoupledRecord` names its writers with a two-case enum

- `AuthoredRetirementRefusal::ForeignFreshCoupledRecord { creator, writer, .. }`: `creator` was `Option<usize>` and `writer` was `usize`. `creator: None` meant the authored prefix. Both fields are now `CoupledFreshWriter { Authored, Producer(usize) }`, because authored stages also write as the authored writer (D1590 follow-up: no meaning hidden in an absence).
- **In-memory and wire only.** `PublicationError` is carried by `TransactionCommitError` and read back only through serde wire readmission (`invariant_execution_failure/tests.rs:311-323`, a different variant). No history, checkpoint or durable store holds it, so no cold court is needed.
- **Changed court:** `synchronous_edit/fresh_custody.rs:75-82`. It asserted `creator == None`/`writer == 0` (authored case) and `creator == Some(0)`/`writer == 1` (producer case). It now asserts `Authored`/`Producer(0)` and `Producer(0)`/`Producer(1)`. Same meaning, same records.

### Stored format

- The one addition is `CanonicalCoupledAuthoredSources::Stages(Vec<EntityId>)`, appended last in the enum. No existing variant, field or tag changes, so existing stores decode unchanged.
- The Native cold court reopens a `Stages` record (`coupled_producers/stages.rs`).
- The header refuses a recorded list shorter than two. No court reaches that check under corruption yet (public #140). The mutant that accepts a short list survives, as scoped.

### Producer-path numbering

- Pins, local-key scopes and continuation tokens are now numbered over every applied part, authored stage or producer (`applied_parts()`).
- With no authored stage this is the producer count, as before, so no existing court changed.

### New courts (`crates/worth-court-relational-certification/.../relational/coupled_producers/stages.rs`)

- `an_authored_stage_reads_the_candidate_and_commits_with_the_prefix_once`, with and without an owned producer. It checks:
  - the rows equal a plain-commit oracle;
  - the recorded `Stages` sources;
  - the authored effects are the prefix's and the stage's only;
  - a cold reopen (mixed case).
- `stages_outside_the_admitted_roster_refuse_without_effect`: an unadmitted stage, a missing stage and a stage after a producer each refuse typed, and C is unchanged.
- `a_later_stage_cannot_rewrite_a_record_an_earlier_stage_wrote`: refuses with `ConflictingIntent` on the rewritten record, and C is unchanged.
- `a_stage_sources_owned_route_is_an_admitted_writer_of_the_envelope`: the Native owner union admits a Preserve of a route rooted at a later stage's source in the same envelope, and it publishes one origin.
