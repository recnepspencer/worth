# CP3 assertion changes

This ledger lists existing court assertions CP3 changed and any typed-error or stored-shape changes, landing by landing. Purely new courts are listed at the end of each landing.

## L1a (Native authored stages)

### `ForeignFreshCoupledRecord` names its writers with a two-case enum

- `AuthoredRetirementRefusal::ForeignFreshCoupledRecord { creator, writer, .. }`: `creator` was `Option<usize>` and `writer` was `usize`. `creator: None` meant the authored prefix. Both fields are now `CoupledFreshWriter { Authored, Producer(usize) }`, because authored stages also write as the authored writer (D1590 follow-up: no meaning hidden in an absence).
- **In-memory and wire only.** `PublicationError` is carried by `TransactionCommitError` and read back only through serde wire readmission (`invariant_execution_failure/tests.rs:311-323`, a different variant). No history, checkpoint or durable store holds it, so no cold court is needed.
- **Changed court:** `synchronous_edit/fresh_custody.rs:75-82`. It asserted `creator == None`/`writer == 0` (authored case) and `creator == Some(0)`/`writer == 1` (producer case). It now asserts `Authored`/`Producer(0)` and `Producer(0)`/`Producer(1)`. Same meaning, same records.

### Stored format

- Current canonical storage has only Edit, Retirement and Restoration with their actual consumed source sequences. Artifact version9 replaces the development grammar; old saved encodings are not supported or migrated.
- Current encode/readmission retains the genuine source kind and ordered repeated scopes. Existing actual history/checkpoint and private codec owner controls establish that behavior.
- The current Edit source grammar contains one actual prefix followed by zero or more consumed later sources. An empty list refuses before readmission; one source is valid. The displaced Stages-only minimum-two court and its public cut-list test door are removed. The existing current codec unit owner checks empty-prefix refusal without constructing an extra publication or retaining an obsolete format API.

### Producer-path numbering

- Pins, local-key scopes and continuation tokens are now numbered over every applied part, authored stage or producer (`applied_parts()`).
- With no authored stage this is the producer count, as before, so no existing court changed.

### New courts (`crates/worth-court-relational-certification/.../relational/coupled_producers/stages.rs`)

- `an_authored_stage_reads_the_candidate_and_commits_with_the_prefix_once`, with and without an owned producer. It checks:
  - the rows equal a plain-commit oracle;
  - the actual recorded Edit sources;
  - the authored effects are the prefix's and the stage's only;
  - each stage's touches project from its own commands (`authored_stage_mutation_touches`; L1a review round 1 blocker);
  - a cold reopen (mixed case).
- `stages_outside_the_admitted_roster_refuse_without_effect`: an unadmitted stage, a missing stage and a stage after a producer each refuse typed, and C is unchanged.
- `a_later_stage_cannot_rewrite_a_field_an_earlier_stage_wrote`: refuses with `ConflictingIntent` on the record whose field was rewritten, and C is unchanged.
- `a_stage_sources_owned_route_is_an_admitted_writer_of_the_envelope`: Native's owner union classifies a route rooted at a later stage's source as surviving (`declared_owner`, which Query consults before running a producer), and its Preserve publishes in the same envelope.
- `a_stage_relation_is_part_of_the_authored_relation_footprint`: a stage's link appears in the proposal's authored relation footprint, so authored relation preimage capture sees it (kills M-a8).

### Mutants

M-a1..M-a10 are killed (`l1a-mutants.log`, `-a6`, `-a7`, `-r1b`, `-r1c`): out-of-order stage, stage after a producer, missing stage, sources recorded as `Edit`, dropped stage effects, owner union without stage owners, a one-element stage list accepted, stage records dropped from the authored records, a stage projection without its own commands, and no stage projection.
