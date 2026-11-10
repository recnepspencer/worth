# P5 CP3 plan: S6 read-your-writes edit envelope

Approved with changes (D1588; review `orchestration/reviews/pubquery-cp3-plan-review.md`, 8 binding changes, all taken in below). CP3 lands as two landings, L1 and L2. Each has its own header and its own review (two rounds at most). All 22 P5 mutants stay required (D1397); the S6 subset M-S1..M-S5 is mapped in section 6.

## L1 header: authored stages, envelope entry, slots, keys, receipt, DAG

- **Usable result:** a multi-step authored edit with several required roots of one program is one Native commit with one envelope key and one receipt, and the receipt carries each authored stage's typed result. Counted on the cabinet-shaped fixture at 1 and 4 authored stages: 1 commit at both. Today each authored step is its own entry and commit (the product's source commits, `worth-cad-runtime/src/runtime/installed/run.rs:104-155`). Stored format: L1 adds one variant, `CanonicalCoupledAuthoredSources::Stages`, and changes no existing one; old stores decode unchanged (D1590), and the variant has its cold court (row 11).
- **Requires:**
  - the Phase 1 base merge of public `5034b49b25` into `4058790f09`, committed with green Phase 1 courts (D1585);
  - D1397 B3 (`orchestration/log/orchestration-log-2026-11-07.md`, D1397 rulings: read-your-writes inside the transaction, DAG order, typed cycle);
  - `synchronous-edit-publication.md:19-23` (S6 scope) and `:501-505` (an authored stage is not a producer posture);
  - Native's pin path (`NC/part.rs:96-236`) and the authored writer custody (`NC/writer_custody.rs:12-23`, `Writer::AuthoredPrefix` at `NC/completion.rs:100-105`).
- **Establishes:** L2 may rely on the authored stage part kind, the single assembly per envelope with one refusal slot, the envelope key and receipt with per-stage results, several roots of one program, and the declared-scope DAG with typed stage and mixed cycles. It may not assume authored stages after a producer wave, the extended final qualification, or the seal-shaped stage.
- **Not in parallel with:** another writer in `NC/` or in `Q/application_contribution/producer/{coupled_assembly,synchronous_outputs}`, `Q/application_attempt/coupled_outputs.rs` or `P/mutation/performed/`; Phase 4 (#131) if it touches `authority/commit/pipeline/publication_execution.rs` (decided at #131's plan review); the parked 3b branch.
- **Closes when:** the L1 rows of section 6 are green with their mutants (M-S1, M-S3, stage cycle, mixed cycle, subject-less refusal, refusing authored stage), the cold-reopen court for the new part variant passes, and commits per edit are counted at 1 and 4 authored stages.

## L2 header: stage B after waves, final qualification, seal-shaped stage

- **Usable result:** an edit, its dependents and its seal are one transaction. A CAD edit can then be one commit where today it is S + P + 1, at least 3 and at least 9 in the product (`transaction.rs:3-122`, `run.rs:104-155`, `document.rs:31-103`). Counted on the cabinet-shaped fixture at 3 and 12 dependents: 1 commit at both. A healthy envelope whose producers are all current makes 0 producer contacts and 1 commit, with the seal folded in.
- **Requires:** L1 landed and certified; the same B3 ruling; the existing final-support owner (`Q/provider/application_attempt_state/commit_preparation/relational_commit/commit_execution/coupled/support.rs:91-104`, `CoupledReadSupportChanged`).
- **Establishes:** Phase 8 (proprietary #93) may rely on one envelope covering edit, dependents and seal, with one key, one receipt carrying the seal stage's typed result, and typed refusals that name their subject and publish nothing. It may **not** assume a verification run (3b, public #137), more than one effectful round per route (#119, #115), a multi-program envelope, or that a later stage may change data an earlier participant read (refused typed, section 4).
- **Not in parallel with:** as L1.
- **Closes when:** every section 6 row is green with its mutant, including M-S2, M-S4 and M-S5; commits per edit are counted at 3 and 12 dependents; the capability index's row (now `capability-index.md:510`, "General demand continuations and the seal envelope remain in P5/S6") states the entry; `synchronous-edit-publication.md:150-152` is edited for B3; and the work is on public master (section 7).

Paths: N = `crates/worth-relational/src`; NC = `N/mvcc/publication/application_publication/coupled_assembly`; Q = `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph`; P = `workspaces/worth-query/crates/worth-query-publication/src/application_entry`.

## 1. What exists after CP2b (read at `4058790f09`)

- **Producers already read earlier producers' unpublished outputs on the same candidate.** The candidate keeps one pin. Each appended part's commands apply over it, and the pin is replaced by a successor that includes the part (`NC/part.rs:216-236`). On the Query side, `prepare_one` pins before each producer, appends, then pins again as the route's after-own pin (`Q/application_contribution/producer/coupled_assembly/preparation.rs:92-155`).
- **Order follows the program's dependency ranks.** A cycle refuses `CyclicConnection` when the program is validated (`workspaces/worth-query/crates/worth-query-declaration/src/application_program/program/dependency_graph.rs:62-66`).
- **Missing for S6:**
  - **No authored stage after the prefix:** a part is a producer posture only (`N/mvcc/publication/application_publication/coupled_assembly.rs:106-113`).
  - **No envelope seal, generation or frontier check:** "seal" in Native is per-route publication authority (`N/mvcc/publication/application_publication/custody.rs:103, 141`).
  - **One root slot:** `DeclaredCoupledOutputDemands.synchronous: Option<_>` (`Q/application_attempt/coupled_outputs.rs:38`).
  - **No whole-envelope key or receipt:** each producer route binds its input-derived key (`Q/application_contribution/producer/execution/coupled_preparation.rs:103`) and has its own receipt (`:249`, `commit_receipt`).
  - **One refusal slot, cleared per assembly:** `DeclaredProducerAssembly` keeps one `denial` slot (`Q/application_contribution/producer/coupled_assembly.rs:37`). The caller maps a missing denial to the generic uncommitted outcome, with no subject (`Q/application_attempt/provider_execution/phase/prepared/running/progression/registered.rs:231-234`).

## 2. Envelope shape

```text
envelope (one key, one Native commit, one receipt with per-stage results):
  stage A1..Ak  authored stages, each admitted at C, applied in DAG order on the candidate   [L1]
  wave          producers required by the authored stages, rank order (CP2b's wave)           [L1]
  stage B1..Bm  authored stages after the wave: read earlier outputs at the candidate pin      [L2]
  wave          producers and dependents required by B (ranks continue)                       [L2]
  seal-shaped   the fixture's last authored stage: writes and compares generation/frontier    [L2]
  final qualification: every participant's retained reads against every later writer          [L2]
refusal: any stage, producer, seal or qualification refusal names its subject and publishes nothing
```

- **Authored stage part kind (L1).** `AuthoredStage` is a new Native part kind beside producer parts. It has its own admission (C admission, D data and final support). It is not a producer posture in disguise (`synchronous-edit-publication.md:501-505`). Each stage's commands apply over the current candidate pin, through `part.rs`'s apply path, so later stages and producers read them (read-your-writes). Authored stages run under the authored writer custody (`Writer::AuthoredPrefix`, `NC/completion.rs:100-105`) and the authored invariant proof, not `Writer::Producer`. The part variant is a new durable shape in the commit envelope; it gets a cold-reopen court. Frame version 7 is unchanged, as for CP2b's `Edit(EntityId)`; a master decoder rejecting the variant is accepted.
- **Stage DAG from declared scopes (L1; change 1a).**
  - An authored handler's exact rows are known only after it runs. At admission only declared scopes are known, so the DAG is built from them: program ranks for producers, and each stage binding's declared reads and writes for authored stages.
  - An edge runs from a writer to every participant whose declared reads overlap its declared writes. Declaration order never decides.
  - A cycle refuses typed at envelope admission, before any stage applies. That covers stage-only cycles and mixed authored↔producer cycles. Producer-only cycles are already refused at program validation.
- **Several roots of one program (L1; change 6).**
  - The envelope carries several required roots of the one program of `execute_performed<Program, Root>` (`P/mutation/performed/selected.rs:70`) and `WorthQueryProgramApplicationRuntime<Schema, Program>`. CAD has one program. A multi-program envelope is a different slice and is out of scope.
  - `synchronous: Option<_>` becomes a collection. A dropped or overwritten root is M-S3.
  - **Shared-route rule:**
    - two slots naming one root are automatic demands and coalesce into one route (the capability index's coalescing of one full-generation source/family/partition route);
    - two explicit writers of one route still refuse `RepeatedCoupledOwner` (`NC/part.rs:38`);
    - a B wave re-demanding an A-wave route whose declared reads B does not write makes 0 contacts and 0 extra commits; it reuses the A-wave result.
- **Keys and receipt (L1; change 3).**
  - The envelope's idempotency key is the only caller-visible key.
  - Authored stages get ordinal sub-keys under it, as CAD's `source_key` ordinals are today (`worth-cad-model/src/document/intent.rs:29-44`).
  - Producer routes keep CP2b's input-derived keys (`coupled_preparation.rs:103`) and receipts (`:249`), because later demands and Ready adoption recheck them.
  - The envelope receipt carries each authored stage's typed result. Phase 8 needs the seal's acknowledgement (`transaction.rs:118-120`). A stage token exposes a typed result only from the committed envelope that issued it (`synchronous-edit-publication.md:152`).
  - A same-key replay returns the whole receipt. A same key with a different stage list or different stage input refuses `IdempotencyIntentDrift` (M-S1).
- **One assembly and one refusal slot per envelope (L1; change 2).**
  - The envelope makes one `assemble` call. Its refusal slot is set once, by the first refusing stage, wave, seal or qualification, and is never cleared before the caller reads it.
  - A break or `Ok(false)` with no recorded subject refuses typed as corruption (name for the code review: `CoupledRefusalWithoutSubject`). The `None => outcome` arm at `registered.rs:231-234` goes away.
- **The seal (L2; change 4).**
  - The public deliverable is the authored stage kind. The seal is not a platform concept: generation and frontier are CAD content. CAD's handler already reads both fields and refuses `GenerationChanged`/`NameFrontierChanged` (`worth-cad-lifecycle/src/document/handler.rs:19-60`). The post-commit re-read at `document.rs:82-101` is what Phase 8 removes.
  - The public fixture's last stage is seal-shaped: it reads and bumps a generation field and a frontier field and refuses typed on a mismatch.
  - The seal is a DAG node like any other. Its write set is checked by the final qualification.
  - The public plan's "Authored stages, including the seal, prepare before derived discovery" (`synchronous-edit-publication.md:150-152`) is superseded by B3 and is edited in L2.

## 3. Atomicity and refusal

- Every stage, producer and the seal run before the single Native commit. A refusal anywhere ends the assembly with nothing applied: Query's break and abort path (`Q/application_contribution/producer/coupled_assembly.rs:218-246`); Native returns before the required-owner check and the commit on a break (`NC/completion.rs:125-146`).
- Each refusal is typed and names its subject: the authored stage, the producer route, the seal-shaped stage, or, for a qualification refusal, both the earlier reader and the later writer. The single slot (section 2) keeps the first subject.
- M-A1-shaped courts: a refusal at stage B, at the seal-shaped stage or at qualification leaves C, the head and every complete row unchanged (`synchronous_edit/failure.rs` shape).

## 4. One effectful round per route (#115's entry criterion): **CP3 creates no multi-round producer**

- CP2b's guard stays: a continued round with an effect refuses `CoupledContinuationProvenancePostponed { issue: 119 }` (`NC/continuation/round.rs:18, 34-61`).
- **Final qualification across the envelope (L2; change 1b and 1c).**
  - **The check:** every participant's retained read set (authored stage, producer route, seal-shaped stage) is checked against every later participant's applied write set. A hit refuses typed and names both the earlier reader and the later writer.
  - **Not a second owner:** this extends CP2b's existing final support check, which refuses a later contribution that invalidates verified parent support (`commit_execution/coupled/support.rs:91-104`, `CoupledReadSupportChanged`; `synchronous_edit/ordinary_support.rs`). It now covers authored stages as readers and writers.
  - **What it guards:** the declared-scope DAG puts a route that reads B's declared writes after B. The qualification catches what the declared scopes did not predict.
- So no participant is re-run: no producer publishes more than one effectful round per route, and **public #115 stays off the path** (cost programme Phase 3; the ruling on #115 is the orchestrator's to post).

## 5. What a later verification run (3b's V, public #137) reuses

- **A producer run on its own provider session, reading the source at a candidate pin:** `prepare_coupled_writer` (`Q/application_contribution/producer/execution/coupled.rs:143`) and `coupled_source::read` (`.../execution/coupled_source.rs:97`). S6 uses them for stage B's producers as they are. CP3 builds only what stage B's producers need and **adds no entry for V** (change 6).
- **Session custody inside the candidate:** writer sessions, `root_session`, and `retain/reset_candidate_read_support` (`Q/application_contribution/producer/coupled_assembly.rs:90-124, 227-236`), `retain_coupled_after_own`.
- **The rank-ordered wave.**
- V needs none of S6's authored stages, seal or envelope key. Native's `verify_producer_round` and the parked step (i) (`081855a754`) stay as they are.
- The L2 handoff records exactly what V can reuse, and 3b is re-sized against it.

## 6. Courts and mutants (cost programme Phase 2 matrix)

| # | Law | Court | Mutant that must fail it | Landing |
|---|---|---|---|---|
| 1 | Several authored stages are one commit and one receipt | 1 and 4 authored stages on the cabinet fixture: 1 commit each; the receipt carries each stage's typed result | a commit per stage | L1 |
| 2 | Envelope idempotency covers every stage; a replay is never a fresh commit | same-key replay returns the typed `AlreadyCommitted` outcome with 0 commits, and the committed rows equal the live run's (D1590); twin: same key with drifted stage bindings (a changed second stage, a changed stage order) refuses `IdempotencyIntentDrift` | **M-S1**: the identity covers only the first stage | L1 |
| 3 | Producer keys stay input-derived | after an envelope, a fresh demand of a produced route reuses its receipt with 0 contacts | producer keys derived from the envelope ordinal | L1 |
| 4 | Several roots are all kept | two roots in one envelope: both roots' outputs published | **M-S3**: a second root replaces the first (`Option` overwrite) | L1 |
| 5 | Shared route: automatic coalesces, explicit refuses | two slots naming one root: one route publication; two explicit writers: `RepeatedCoupledOwner` | the shared route runs twice | L1 |
| 6 | Stage order follows declared scopes | two stages, one reading the other's declared writes, declared in the opposite order: same final rows | order taken from declaration | L1 |
| 7 | A stage cycle refuses typed | stages declaring a cycle: typed refusal at admission, nothing applied | the stage-cycle check removed | L1 |
| 8 | A mixed authored↔producer cycle refuses typed | a stage that writes what a producer reads, and reads that producer's output: typed refusal at admission | only stage-only edges checked | L1 |
| 9 | A refusal always names its subject | an authored stage refusing with a domain denial: nothing published, the refusal names the stage | the refusal slot cleared between waves (subject lost) | L1 |
| 10 | A subject-less break is corruption | a fault seam yielding a break with no recorded subject: typed corruption refusal | the `None => outcome` arm restored | L1 |
| 11 | The new part variant survives cold reopen | cold reopen after an envelope with authored stages | a decode fault on the variant | L1 |
| 12 | A later stage reads an earlier stage's unpublished output | cabinet fixture: source, required producer, stage B whose dependent reads the producer's result; one commit; rows equal the multi-commit path's final rows | **M-S5**: B reads the producer output at C | L2 |
| 13 | A producer refusal in a B wave is atomic | a producer refusing in B's wave: nothing published, the refusal names the producer | the prefix published on the later failure (M-A1 shape) | L2 |
| 14 | The final qualification spans the envelope | stage B2 writes a row B1 read at c2: typed refusal naming B1 and B2 | qualification checks producer routes only | L2 |
| 15 | The seal-shaped stage is in the one commit | a derived stage fails after the seal-shaped stage applied: no commit, typed refusal | **M-S2**: the seal stage committed separately on a failed derived stage | L2 |
| 16 | The seal stage's comparison holds | the fixture stage bumps generation twice: typed refusal | **M-S4**: the generation/frontier comparison skipped | L2 |
| 17 | A healthy envelope with the seal folded in | all producers current: 0 producer contacts, 1 commit | the seal stage forces a producer contact or a second commit | L2 |
| 18 | B re-demands an untouched A-wave route | 0 contacts, 0 extra commits | the route re-run | L2 |
| 19 | Commits per edit do not grow | 3 and 12 dependents: 1 commit each, counted with the all-branch counter (`reconciliation/commit_count.rs:100-140`, `synchronous_edit/courts.rs:64-68`) | a commit per dependent | L2 |

- Every court runs hot and worker-only. Row 11 is the one cold court.
- Mutants: commit first, inject, `git checkout --`, `git diff --exit-code`.
- The other 17 P5 mutants (D1397) keep passing through the existing CP2a/CP2b courts; each landing's handoff lists them with their status.

## 7. Public ref at close

- The base merge puts this branch on the pin sha, public `5034b49b25` (D1585). The other machine's 9.19b Step 0 sets public master to `5034b49b25` and private main to integration.
- After Step 0, the ref main can pin is **public master**. CP3 closes when its work reaches public master through the public gate. The orchestrator gates and pushes; nothing is pushed from this lane.
- If Step 0 has not landed on master when CP3 is ready, the orchestrator decides the interim ref.

## 8. CAD's born-stale shape (D1571, D1582 (d)) and the predecessor posture

- **No field or optional-aspect case found in CAD** (read in proprietary `worth-proprietary-main` at `de023137fb`). CAD's only producers are the body realization producer (`worth-cad-lifecycle/src/body/realization/producer.rs:62`) and the generator producer (`worth-cad-entry/src/families/generator/producer.rs:65`). Every field they both read and write is on their own outputs: body record, stage realization records, generator child features, topology cells. Fields read and then written on a non-output entity (body `occurrence`, feature `parameters`) are written only by authored handlers, never by a producer.
- **What makes a producer born stale in public WORTH** (`Q/provider/application_attempt_state/commit_preparation/relational_commit/evidence_seal/postcommit_currentness/resolution.rs`):
  - a `reads_source` fact kind its own commit moves (`:112-140`, `Movement::Moved` gives `KeepSuperseded`);
  - an `OptionalAspectAbsent` that is now present (`:98-110`).

  `AbsentField` is not one (`:200-215`, it resolves to `Field { revision }`).
- **Nearest create-edit analogue: incoming-relation reads.**
  - On a create, the producer reads an incoming relation set on a non-output entity, finds nothing, then links into it (`header.rs:35, :84-97`; `worth-cad-model/src/realization/mod.rs:97-106`; generator `native_publish.rs:31, :43-60`).
  - For a producer, `Fact::Relation` and `Fact::Adjacency` are re-observed at the committed snapshot and **replaced** (`resolution.rs:276-290`; `apply` at `:353-356`, `Replace(rebased)`). The stored facts then describe the committed link, and the route is not superseded.
  - CAD's `header.rs:35` `relations_to` is such a decision adjacency read. So **CAD's create edits are not born stale**. That is inferred from source with no CAD run; Phase 8 courts it.
- **Predecessor origins:** the body producer declares predecessor reads in Preserve mode only (`producer.rs:82-89`). CAD's `class` is exhaustive and maps `PredecessorBasisUnavailableForOrigin(_)` to a named pending refusal, `Class::Pending`. A unit test covers both `AuthoredRestoration` and `HistoricalSelection` (`regenerate/capture/before.rs:251-264, 306-316`). That is CAD's side of public #138; Phase 8's court drives it end to end.

## 9. Size and stops

- **Estimate, not a measurement.**
  - **L1:** Native stage part, admission and cold court, 500-800 lines; Query envelope entry, slots, keys, receipt, refusal slot and DAG, 500-700 lines; courts (rows 1-11), 500-700 lines.
  - **L2:** B waves, extended qualification and seal-shaped fixture stage, 300-500 lines; courts (rows 12-19), 400-600 lines; plan and index edits.
- **Stops:**
  - Stop for review if the stage part needs a stored-format change beyond a new envelope variant (with its cold court).
  - Stop if any route would need a second effectful round (that is #119/#115, not CP3).
  - Stop if the seal-shaped stage cannot read the candidate without minting Current or Ready from candidate data.
  - Stop if a stage binding cannot declare its read and write scopes without a new consumer contract beyond the existing operation access declaration.

## 10. L1 decisions from the code map (2026-11-10, decided by the lane, reported to the orchestrator)

- **Replay results.** A same-key replay returns the envelope receipt as `AlreadyCommitted`, with no per-stage typed results. That is today's contract for one mutation (`P/mutation/outcome.rs:17-18`). Per-stage typed results come only from the live committed envelope (`synchronous-edit-publication.md:152`). Durable per-stage result custody would be a stored-format change beyond the envelope variant, which section 9 stops on. Phase 8 reads the seal's acknowledgement from the live outcome; on replay CAD keeps today's path. A replay carries no per-stage results; a caller reads the committed state instead (D1590). Durable per-stage receipts are postponed to public #139, whose entry criterion is Phase 8's decision; Phase 8 reads the seal acknowledgement on a replay from the committed document.
- **Envelope identity.** The drift identity folds in the key, the ordered list of (stage binding identity, stage input identity) and each stage's root. Today it is the key, one input and one binding (`Q/application_attempt/idempotency/mutation_binding.rs:32-41`). M-S1 is that fold reduced to the first stage.
- **Stage 1 stays the authored prefix.** It is the transaction's first merged plan (`commit_plan.rs:68-76`). Stages 2..k are the new Native part kind. `CoupledPublicationPart.publication` becomes producer-or-authored-stage. Authored stages use a writer custody arm with `AuthoredPrefix`'s meaning and their own part ordinal. The producer-only checks (`part.rs:22-42`, `completion.rs:133-146`, `coupled_authored.rs:138-142`) skip authored stages.
- **Stage handlers read the candidate.** A stage after the first runs its handler with C authorization and D data at the current candidate pin, through the provider-session read the producers use (`coupled_source::read`). This is L1's one new Query read mechanism.
- **API.** An envelope request takes `.stage::<Root>(intent)` per authored stage, then `.idempotency(..)` and `execute_performed::<Program>`. Each stage names its root of the one program. The roots seed one wave (`synchronous_outputs/capture.rs:144-171` becomes a collection).
- **Durable shape.** The new variant is `CanonicalCoupledAuthoredSources::Stages`, beside `Edit` (`coupled_authored.rs:12-17`), and is courted cold (row 11). An envelope with no producer participant commits through the existing `install_empty` path.

## 11. L1 lands as L1a (Native) and L1b (Query) (2026-11-10, lane decision: land small)

L1 as a single landing would combine a stored-format addition with a new generic Query entry. L1a's Native contract can be reviewed on its own.

### L1a header: Native authored stages

- **Usable result:** one Native coupled commit carries the authored prefix, later authored stages and producer parts.
  - Native admits each stage source up front (`readmit_edit_coupled_stage_owners`, the union of their current owners) and applies each stage once, in roster order, before any producer, over the candidate pin (`append_authored_stage`).
  - Each stage's touches project from its own commands (`authored_stage_mutation_touches`).
  - The canonical event records the stage sources (`CommitPublication::coupled_authored_stage_sources`).
  - Commits per envelope: 1 (Native court).
  - Stored format: adds the variant `CanonicalCoupledAuthoredSources::Stages`, appended last, and changes no existing one.
- **Requires:**
  - merge `d39fc1fd21`;
  - plan `b9ae180743` (D1588, D1590);
  - Native's pin path (`coupled_assembly/part.rs`, now `part/suffix.rs`);
  - the authored writer custody (`writer_custody::Writer::AuthoredPrefix`).
- **Establishes:**
  - L1b may rely on the entries above and on `CoupledFreshWriter`.
  - L1b may not assume:
    - a stage after a producer: refused `UnadmittedAuthoredStage` until L2's qualification;
    - a later stage writing a record an earlier stage wrote: one Native commit carries one intent per record, so it is refused `ConflictingIntent`, and stages write disjoint records;
    - unscoped local client keys: Native scopes each applied part's local keys, so L1b resolves a stage's created references through the stage's applied outcome;
    - a courted short-list header check (public #140).
- **Not in parallel with:** as L1.
- **Closes when:** the Native stage courts and the Phase 1 regression courts are green, mutants M-a1..M-a6 are killed (`l1a-mutants*.log`), and the review is certified.

### L1b header

L1b is L1's header with the Native items removed. It delivers the envelope entry; stage handlers at the candidate pin with C authorization and D data; each stage's reads in the final support check; several roots; the drift identity with D1590's replay courts; the single refusal slot; the DAG; and the capability-index sentence. Rows 1-10 of section 6 are its courts.
