# Tree-based undo and redo over owned models

Public WORTH platform specification for completing owned-model document history
on top of `WorthQueryEditEnvelope` and authored corrections (public issues
`#53`, `#54`, `#55`, `#56`, and `#57`; supersedes the earlier working log in
[history-owned-models.md](history-owned-models.md) for remaining work).

## Goal

An application whose model owns child entities, relations, and multiple
generated-output trees records every edit envelope and authored corrective
retirement as a node on one platform-owned history tree (`HistoryDocument`).
Callers can undo any number of steps, redo along any recorded branch without
losing prior branches, check out any retained history node, and inspect a
paginated tree of recorded steps through `worth-query-decl` and
`worth-query-host`.

**Primary rule (Architecture 1, 10, 18):**
Public WORTH owns history-tree ancestry, ownership enforcement, multi-tree
output restoration, branching redo choices, pagination, and retention.
Applications own schema ownership declarations, descriptive step summaries, and
domain invariants. Downstream applications (such as CAD, BIM, and Studio UIs)
consume Phases 1–2 through `worth-query-decl` and `worth-query-host` without
building private undo stacks or inverse-geometry caches.

## Scope and current boundary

### Already present on `master` (verified 2026-11-21)

- `ApplicationHistoryModelDomain::OwnedModelGraph` and declarative
  `ApplicationHistoryOwnershipRelation`
  (`workspaces/worth-query/crates/worth-query-declaration/src/application_history/{definition,ownership}.rs`)
  with automatic Native single-owner commit-boundary enforcement (`#53`, `#55`).
- Runtime-length heterogeneous gesture groups via
  `WorthQueryApplicationHistoryDocument::gesture`
  (`workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/group/gesture.rs`,
  `#54`).
- Multi-tree generated-output restoration via
  `WorthQueryDocumentHistory::bind_history_output_forest` and
  `HistoryOutputForest`
  (`workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/navigation/output_profile/forest.rs`,
  `#57`).
- Paginated tree inspection (`history_page`, `WorthQueryHistoryPageCursor`),
  branching redo choices (`WorthQueryHistoryRedoChoices`), typed navigation
  (`navigate::<N>().undo_step()`, `.redo_to(child)`, `.checkout(node)`),
  retention windows, bookmarks, and explicit `close()`.

*(Issues `#53`, `#54`, `#55`, and `#57` remain open despite their core machinery
landing on `master`. Phase 1 verifies their completeness under multi-stage
coupled edit envelopes and closes each satisfied issue.)*

### Missing today (the gaps this spec closes)

1. **Edit envelopes cannot record into `HistoryDocument`:** Multi-stage
   application mutations execute as one atomic `WorthQueryEditEnvelope`
   (`request.edit_envelope().idempotency(key).runtime_stages(application).ordered_authored()`),
   combining authored stages, coupled required producers, and
   `ApplicationNoOutputGraph` finalization stages. Today
   `WorthQueryApplicationHistoryDocument` only accepts single-action
   `.edit(...)` or sequential `.gesture(...)`, and any direct
   `request.edit_envelope()` commit makes the next document call refuse
   `HeadDiverged`.
2. **Long-lived session custody:**
   `WorthQueryApplicationHistoryDocument<'runtime, ...>` borrows the runtime by
   lifetime `'runtime`. Long-lived session runtimes that own
   `WorthQueryProgramApplicationRuntime<S, P>` cannot store per-model attached
   documents beside their runtime without an owned handle backed by the same
   `Arc` runtime custody.
3. **Authored corrections cannot record into `HistoryDocument` (`#56`):**
   Corrective retirement envelopes
   (`WorthQueryCorrectionPreparation::edit_envelope`) currently advance the
   branch outside the document (`HeadDiverged`), and navigation does not cross
   recorded retirement boundaries.

### Out of scope

- **Persistent naming across undo/redo/checkout:** Specified in
  [milestone-11.md](milestone-11.md) (`#71`).
- **Cross-restart durable history home:** Owned by Store Milestone 3
  (`plans/database-foundation/milestone-3.md`). This spec adds no new
  checkpoint-byte reopen stopgaps.
- **Downstream application cutover:** Consumer runtime, session, and UI
  adoption (Phases 3–5 of the cross-repository milestone) lives in the consumer
  repository.

## Shared definitions

- **History node (`WorthQueryHistoryNodeId`):** Public opaque identity of one
  committed state on the model's history tree (initial root, recorded edit
  envelope, recorded corrective retirement, or recorded navigation checkout).
- **Current selection (`WorthQueryHistorySelection`):** Public proof of the
  document's currently checked-out node and physical product branch, minted only
  by the history owner.
- **Branch point:** Any history node with more than one child step (created when
  a caller undoes or checks out an earlier node and then commits a new edit).
  Prior branches remain intact until explicitly pruned by retention policy.
- **Step metadata:** Bounded descriptive metadata attached to a recorded step by
  the application under `ApplicationHistoryRetentionResources`. Descriptive only;
  never read as execution or navigation authority (Architecture 10, 18).

## Phase overview

| Phase | Usable result | Requires | Establishes for later phases |
|---|---|---|---|
| **1. Record edit envelopes in document history** | Attached owned-model `HistoryDocument` executes and records a multi-stage coupled `WorthQueryEditEnvelope` as one atomic history step, holds an owned session handle, and navigates (`undo_step`, `redo_to`, `checkout`) across multi-tree generated outputs. | Envelope scheduling PR C merged on `master` | Atomic envelope history recording, `WorthQueryOwnedApplicationHistoryDocument`, verified `#53`/`#54`/`#55`/`#57` closure. |
| **2. Record authored corrections and navigate retirements (`#56`)** | Authored corrective retirements (cascading delete) record as steps on the same `HistoryDocument` tree without `HeadDiverged`, and undo/redo/checkout cleanly restores or re-retires owned subgraphs and their output forests. | Phase 1 | Unified public tree covering edits, deletions, branching, and undo/redo/checkout; closes `#56` and unblocks downstream consumer adoption (Phases 3–5). |

---

## Phase 1: Record edit envelopes in document history

### Outcome, scope and dependencies

An attached `WorthQueryApplicationHistoryDocument` (in `OwnedModelGraph` mode)
can execute a multi-stage coupled `WorthQueryEditEnvelope` — authored stages
(including stages scoped to model-owned entities), coupled required producers,
and `ApplicationNoOutputGraph` finalization stages — as **one atomic history
step**, with full idempotent replay, typed refusal leaving the document head
unchanged, and multi-tree output-forest capture so that subsequent `undo_step`,
`redo_to`, and `checkout` restore both authored rows and every generated-output
tree cleanly.

This phase also introduces an owned handle
(`WorthQueryOwnedApplicationHistoryDocument`) so long-lived runtimes can hold an
attached document beside their `WorthQueryProgramApplicationRuntime` without
self-borrowing, and verifies that `#53`, `#54`, `#55`, and `#57` have no
remaining contract gaps when exercised through multi-stage edit envelopes.

**Requires:** Envelope scheduling PR C merged on `master`.
**Out of scope:** Authored corrections / cascade retirements (Phase 2, `#56`).

### Target API and caller DX

**Existing surfaces reused unchanged:**
- `worth_query_declaration::facade::application_history::{ApplicationHistoryBinding, ApplicationHistoryModelDomain::OwnedModelGraph, ApplicationHistoryOwnershipRelation}`
- `worth_query_publication::facade::application_entry::{WorthQueryApplicationHistoryDocument, WorthQueryHistoryNavigationIntent, WorthQueryHistoryOutputProfile, WorthQueryHistoryRedoChoices}`
- `worth_query_publication::facade::application_entry::{WorthQueryEditEnvelope, WorthQueryEditEnvelopeOutcome, WorthQueryEditEnvelopeReceipt, WorthQueryEditStageToken}`

**Proposed surfaces on `worth_query_publication::facade::application_entry` (re-exported via `worth_query_host::facade::application_entry`):**

```rust
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequest, WorthQueryEditEnvelopeOutcome,
    WorthQueryHistoryEditEnvelopeOutcome, WorthQueryHistoryOutputProfile,
    WorthQueryOwnedApplicationHistoryDocument,
};
use worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

// 1. Attach an owned move-only history document backed by the runtime's Arc custody.
//    Returns the original owned product head on refusal (no leaked slot).
let mut document: WorthQueryOwnedApplicationHistoryDocument<S, H> =
    request.attach_owned_history::<H, P>(&application, &phase, owned_head, access)?;

// Bind the model's multi-family generated-output forest once at attachment.
let forest: WorthQueryHistoryOutputProfile =
    document.bind_output_forest(&phase, &tree_profiles)?;
let mut document = document.with_outputs(forest)?;

// 2. Record one multi-stage coupled edit envelope as a single atomic history step.
//    Reuses the exact ordered_authored stage builder and wave scheduler.
let mut envelope = document
    .edit_envelope(&application, &request, idempotency_key)
    .ordered_authored()?;
let stage_0 = envelope.push_stage::<BirthTree, _>(authored_input)?;
let final_stage = envelope.push_stage::<ApplicationNoOutputGraph, _>(frontier_edit)?;

// Execute atomically: advances the document head and returns the document on every exit.
let recorded = envelope.execute_recorded(
    step_metadata, // bounded descriptive summary (serialized bytes under declared limit)
    ExecutionAllocationPolicy::SystemAllocation,
);
match recorded {
    Ok(WorthQueryHistoryEditEnvelopeOutcome::Recorded {
        document: next,
        step,      // WorthQueryHistoryStep (node id, parent id, branch, metadata)
        receipt,   // WorthQueryEditEnvelopeReceipt (typed per-stage results via tokens)
    }) => {
        document = next;
        let ack = receipt.result(&final_stage)?;
    }
    Ok(WorthQueryHistoryEditEnvelopeOutcome::RecoveredCommitted {
        document: next,
        step,
        receipt,
    }) => {
        document = next;
    }
    Err(failure) => {
        // Every refusal returns the intact document at its prior head; no partial step.
        document = failure.document;
        handle_denial(failure.denial);
    }
}

// 3. Navigate the tree (existing navigation builder, also exposed on owned handle).
let nav = document
    .navigate::<Nav>()
    .undo_step()
    .expected_selection(selection)
    .idempotency(undo_key)
    .execute(&application, &request, ExecutionAllocationPolicy::SystemAllocation)?;

// 4. Explicit disposal releases the owned product head and history custody.
let close_receipt = document.close(&request, &phase)?;
```

- **Caller ownership and DX (DX 1, 2, 4, 8, 19):** `document.edit_envelope(...)`
  moves `document` into the envelope and returns it on both `Ok` and `Err`, so a
  caller cannot issue concurrent mutations against a stale head or drop the
  attached head on refusal. `execute_recorded` exposes the commit/allocation
  boundary explicitly (Arch 15, DX 3). `WorthQueryOwnedApplicationHistoryDocument`
  carries `#[must_use]` and requires explicit `.close(...)` (Arch 19, DX 4).

### Authority, typed progression and architectural laws

- **Shared lifecycle, no second wave scheduler (Arch 12, 13):**
  `WorthQueryHistoryEditEnvelope` parameterizes the existing
  `WorthQueryEditEnvelope` execution owner with a history-recording seal rather
  than duplicating stage ordering, coupled wave scheduling, or touch projection.
- **Single atomic commit point (Arch 14, 16; DX 19):** History node creation,
  output-forest binding capture, and the envelope's authored + producer writes
  commit in **one** Native/Query transaction. There is no two-commit window where
  an edit commits and history recording fails (or leaves `HeadDiverged`).
- **Ownership proof on every stage (Arch 3, 10, 16):** Every stage in the
  envelope must either target the model root or an entity proven reachable from
  the model root via declared `ApplicationHistoryOwnershipRelation` edges at
  admission; foreign-model stages refuse before commit with
  `WorthQueryDocumentHistoryDenial::ForeignModel` / `UnownedScope`.
- **Descriptive metadata isolation (Arch 10, 18, 20):** Step metadata is
  bounded by `ApplicationHistoryRetentionResources`, validated before effects,
  and never read by execution or navigation as authority.

| Transition | Exact input / proof | Granting owner and admission | Output / proof | Next permitted consumer | Refusal and effect posture |
|---|---|---|---|---|---|
| Attach owned head | `WorthQueryOwnedProductHead<S>`, `H::Access`, `Arc`-backed runtime | `WorthQueryApplicationRequest::attach_owned_history` (verifies installed `H` and branch match) | `WorthQueryOwnedApplicationHistoryDocument<S, H>` | `with_outputs`, `edit_envelope`, `navigate`, `history_page`, `close` | Returns original `WorthQueryOwnedProductHead` unchanged; zero writes. |
| Bind output forest | `&[WorthQueryHistoryOutputProfile]` (all same model & program, `OwnedModelGraph`) | `WorthQueryDocumentHistory::bind_history_output_forest` | `WorthQueryHistoryOutputProfile` (forest) | `document.with_outputs(forest)` | Refuses `IntentDrift` / `ModelDomain` before any candidate work. |
| Open history envelope | Owned `document`, `&WorthQueryProgramApplicationRuntime`, `&request`, `key` | `document.edit_envelope(...)` (checks runtime identity and current document head) | `WorthQueryHistoryEditEnvelope` (moves `document`) | `.ordered_authored()`, `.push_stage::<Root, _>(...)` | Returns intact `document` + `ForeignRuntime` / `HeadDiverged`; zero writes. |
| Execute & record | Filled envelope, bounded metadata, `ExecutionAllocationPolicy` | Coupled envelope executor + `document_history::sealing` (one transaction: verifies owned scope for every stage, runs wave, binds output forest, seals step) | `Recorded { document, step, receipt }` or `RecoveredCommitted { document, step, receipt }` | Next edit, `navigate`, `history_page`, `close` | Returns intact `document` at prior head; zero authored rows, zero history node. |
| Navigate tree | `WorthQueryHistoryNavigationIntent` (`UndoStep` / `RedoTo(child)` / `Checkout(node)`), expected `WorthQueryHistorySelection`, `key` | `document_history::navigation` (eligibility + union owned-graph restoration + multi-tree output forest restoration) | Updated `document`, new `WorthQueryHistorySelection`, navigation completion | Reads on new branch, further edits (branching if non-tip), `close` | Refuses typed (`AtRoot`, `UnknownNode`, `HeadDiverged`, `RequiresSemanticCorrection`) with document intact. |

### Operational behavior and cutover

1. **Admission:** Validate runtime identity, document head correspondence
   (catching external `HeadDiverged` before wave work), metadata byte bound, and
   each stage's operation enrollment in `H` (either model-scoped or reachable via
   declared `.owns(...)` relations).
2. **Idempotency check:** Resolve `key` against the document's recorded history
   attempts. Same-key replay returns `RecoveredCommitted` with the original
   `WorthQueryHistoryStep` and `WorthQueryEditEnvelopeReceipt` without creating
   a second node or second commit; conflicting payload refuses typed.
3. **Execution & sealing:** Run the existing coupled edit-envelope wave
   (including `ApplicationNoOutputGraph` stages and qualified use / reopen /
   support windows), verify the owned-model single-owner invariant, capture the
   step's output-forest correspondence, and seal the `HistoryNode` atomically in
   the same commit.
4. **Navigation & branching:** `undo_step` checks out `current.parent()`.
   Committing a new envelope while checked out at a non-tip node creates a new
   sibling child under that node without erasing the prior branch. `redo_choices`
   lists all direct children of the current node; `redo_to(child)` checks out
   the chosen child; `checkout(node)` jumps to any retained node in the tree,
   restoring the union of current and target owned graphs and every output tree
   in the bound forest.
5. **Issue cutover:** Close `#53`, `#54`, `#55`, and `#57` once Phase 1 courts
   confirm zero remaining gaps under multi-stage edit envelopes.

### Ownership and meaningful placement

- **Extended owners (no new crate, no catch-all module — Domain Structure 1, 2, 11):**
  - `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/`:
    add `envelope_recording.rs` (sealing a coupled edit-envelope outcome and its
    multi-tree output forest into one `HistoryNode`, reusing `sealing.rs` and
    `recording_outputs/`).
  - `workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/`:
    add `owned.rs` (`WorthQueryOwnedApplicationHistoryDocument`, delegating to
    `WorthQueryApplicationHistoryDocument` methods via `Arc`-backed runtime) and
    `edit_envelope.rs` (`WorthQueryHistoryEditEnvelope` delegating to
    `mutation/envelope/`).
  - `workspaces/worth-query/crates/worth-query-host/src/facade.rs`: re-export the
    new public entry types.
- **Forbidden placements:** Do not copy wave-scheduling logic into
  `document_history/`, and do not let `mutation/envelope/` directly mutate
  history metadata without going through `document_history::sealing`.

### Phase 1 test matrix

| Requirement / law | Setup and entry | Action or failure sequence | Expected result and effects | Decisive observation / defect exposed | Test boundary and lane |
|---|---|---|---|---|---|
| Atomic multi-stage envelope recording (Arch 12, 16) | Attached `OwnedModelGraph` document with 2 output-tree families + `ApplicationNoOutputGraph` seal stage | Commit one `edit_envelope` touching model + owned child + dependent producer + seal stage | `Recorded` with 1 `HistoryNode`; both output trees committed; `history_page` shows root → step 1 | Proves single commit (no `HeadDiverged`, no 2-commit split) and `ApplicationNoOutputGraph` compatibility | Decl/Host court (`worth-query-court-decl-host`, `document_history`) |
| Tree undo, branching edit, and multi-branch redo (`#53`, `#57`) | Document with steps `root → A → B` across 2 generated-output trees | `undo_step` to `A`; commit new envelope `C` (`A → C`); inspect `redo_choices` at `A`; `redo_to(B)` then `checkout(C)` | `A` lists `[B, C]`; `redo_to(B)` restores exact authored + both output trees of `B`; `checkout(C)` restores `C` | Exposes lost branch on fork, stale second output tree, or partial owned-graph restore | Decl/Host court (`document_history::navigation`) |
| Owned-model scope enforcement (`#55`, Arch 10) | Two sibling models `M1`, `M2` under same installed schema | Submit envelope on `M1`'s document whose second stage targets an entity owned by `M2` | Refuses `ForeignModel` / `UnownedScope`; returns `M1`'s document intact at prior head; `M2` unchanged | Exposes missing per-stage ownership validation inside multi-stage envelopes | Decl/Host court (`document_history::membership`) |
| Idempotent replay & conflict (DX 3, 19) | Document at step `A` | Replay `A`'s exact `idempotency_key` + payload; then send same `key` with different payload | Exact replay returns `RecoveredCommitted` with same `WorthQueryHistoryNodeId` and 0 new nodes; mismatch refuses typed | Exposes duplicate history node on retry or silent overwrite on key collision | Decl/Host court (`document_history::commands`) |
| Refusal leaves head intact & non-diverged (Arch 14, 22) | Document at step `A` | Submit envelope whose producer stage fails a domain invariant (or hits budget/cancellation) | Returns `Err` with intact `document` at `A`; next valid envelope commits `A → B` without `HeadDiverged` | Exposes leaked candidate/head drift after failed envelope | Decl/Host court (`document_history::commands`) |
| Owned document lifecycle & explicit close (Arch 9, 19; DX 4) | `WorthQueryOwnedApplicationHistoryDocument` held in a struct alongside `WorthQueryProgramApplicationRuntime` | Record 2 steps, undo 1, call `close(&request, &phase)`; plus compile-fail test attempting to move/duplicate owned handle | Clean `WorthQueryDocumentHistoryCloseReceipt` releasing product slot; duplicate handle fails to compile (`E0382`) | Proves non-self-referential session storage and linear head custody | Decl/Host court + doctest (`worth-query-publication`) |
| Structural proportionality of navigation (Perf Core Law 1) | Model with 20 vs 80 unrelated sibling entities; edit 1 entity and undo/redo | Measure navigation work units on 20-entity vs 80-entity model for a 1-entity step | Work bounded by touched owned delta + output forest, not full-history replay | Exposes hidden O(history_len) replay during checkout | Cost court (`cost::document_history`) |
| Tree-history state-machine laws (`proptest` 1.x; Testing 2) | 32–64 bounded `proptest` sequences (`proptest-regressions/` committed) generated from exact public constructors over a 2-output-tree owned-model fixture | Mix `edit_envelope`, `undo_step`, `redo_to`, `checkout(any retained node)`, same-key replay, refused edit, and `history_page(limit 1..=5)` | After every step, authored state + both output trees equal the snapshot recorded at that `NodeId`; `undo_step` + `redo_to` round-trips; refused edits leave tree & head unchanged; pagination matches full tree | Exposes order-dependent branch corruption, stale output-forest state after arbitrary jumps, or pagination drift | Decl/Host court (`worth-query-court-decl-host`, `document_history::navigation`) |

### Completion gate and documentation

- **Verification:** Filtered `worth-query-court-decl-host` (`document_history`),
  `worth-query-publication` doctests, `consumer_root` `document_history`
  acceptance binary, `./loop gate --step guard-diff`, `--step line-caps`, and
  check-compile of dependent query test targets.
- **Docs:** Update `docs/capability-index.md` and the `document_history`
  consumer guide/example to document `edit_envelope` history recording and
  `WorthQueryOwnedApplicationHistoryDocument`. Close `#53`, `#54`, `#55`, and
  `#57` with evidence links.

---

## Phase 2: Record authored corrections and navigate retirements (`#56`)

### Outcome, scope and dependencies

Authored corrections — specifically `retire_created` / `restore_retired` and
cascade retirements of owned subgraphs — execute and record as first-class steps
on `WorthQueryOwnedApplicationHistoryDocument` (and
`WorthQueryApplicationHistoryDocument`) without causing `HeadDiverged`.
Navigating (`undo_step`, `redo_to`, `checkout`) across a recorded retirement
restores the retired owned entities, their relations, and their generated-output
forest cleanly when stepping before the retirement, and re-retires them cleanly
when stepping forward onto or past the retirement — including when a caller
undoes a deletion and branches into a new edit.

**Requires:** Phase 1 landed on `master`.
**Out of scope:** Non-history out-of-band correction APIs remain unchanged for
applications that do not attach a `HistoryDocument`.

### Target API and caller DX

Extends Phase 1's `WorthQueryOwnedApplicationHistoryDocument` and
`WorthQueryApplicationHistoryDocument` with one corrective envelope entry
reusing `WorthQueryCorrectionPreparation` (from
`worth-query-publication/src/application_entry/correction/preparation/envelope.rs`):

```rust
use worth_query_host::facade::application_entry::{
    WorthQueryHistoryEditEnvelopeOutcome, WorthQueryOwnedApplicationHistoryDocument,
};

// Enter corrective retirement through the document's owned head — never out-of-band.
let mut envelope = document
    .correction_envelope(&application, &request, idempotency_key, correction_selection)
    .ordered_authored()?;
let retire_stage = envelope.push_stage::<RetirementTree, _>(retire_input)?;
let final_stage = envelope.push_stage::<ApplicationNoOutputGraph, _>(frontier_edit)?;

// Same atomic execute_recorded outcome as Phase 1: returns Recorded / RecoveredCommitted
// with the updated document, or Err returning the intact document.
let outcome = envelope.execute_recorded(
    step_metadata,
    ExecutionAllocationPolicy::SystemAllocation,
)?;
```

- **Unified DX (DX 1, 8, 19):** Callers use the exact same `ordered_authored` →
  `push_stage` → `execute_recorded` progression established in Phase 1;
  corrective retirement is simply an envelope opened with a validated
  `WorthQueryCorrectionSelection` rather than a separate history-breaking path.

### Authority, typed progression and architectural laws

- **No competing correction authority (Arch 10, 12, 16):** The document validates
  that the correction target belongs to the document's `OwnedModelGraph` before
  entering the corrective envelope. An out-of-band `request.corrections()` call
  on the same branch continues to advance the branch outside the document and
  honestly triggers `HeadDiverged` on the document (preserving Arch 10: lost
  custody is never silently papered over).
- **Exact retirement / restoration custody during navigation (Arch 14, 18, 19):**
  Undoing a recorded retirement does not fabricate new entity IDs or re-run
  authored constructors from scratch; it navigates to the pre-retirement
  snapshot using `OwnedModelGraph` union restoration (restoring the retired
  entities at their original full-generation IDs, incoming ownership edges, and
  generated-output forest under original source-scope custody). Redoing the
  retirement checks out the post-retirement snapshot, retiring those entities and
  their cascaded outputs atomically.

| Transition | Exact input / proof | Granting owner and admission | Output / proof | Next permitted consumer | Refusal and effect posture |
|---|---|---|---|---|---|
| Open corrective history envelope | Owned `document`, `&application`, `&request`, `key`, `WorthQueryCorrectionSelection` | `document.correction_envelope(...)` (verifies target is owned by document's model and selection matches current head) | `WorthQueryHistoryEditEnvelope` (corrective mode, moves `document`) | `.ordered_authored()`, `.push_stage(...)`, `.execute_recorded(...)` | Returns intact `document` + `ForeignModel` / `HeadDiverged` / `CorrectionDenied`; zero writes. |
| Execute & record retirement | Filled corrective envelope, bounded metadata, allocation policy | Corrective envelope executor + `document_history::sealing` (one atomic commit) | `Recorded { document, step, receipt }` | Subsequent edits, `undo_step`, `redo_to`, `checkout` | Returns intact `document`; target stays live; zero history node. |
| Undo recorded retirement | `UndoStep` / `Checkout(pre_delete_node)` | `document_history::navigation` (union restoration across retired owned closure + output forest) | Document at pre-delete node; retired entities & outputs live at original IDs | Edits (creates branch), `redo_to(delete_node)` | Refuses typed with document intact; zero partial restoration. |

### Operational behavior and cutover

1. **Recording:** Corrective envelope runs inside the document's head custody,
   performs cascade retirement + finalization stage + `HistoryNode` sealing in
   one atomic commit.
2. **Post-deletion continuity:** Subsequent `document.edit_envelope(...)` calls
   succeed against the post-deletion head without `HeadDiverged`.
3. **Undo / redo / branch across deletion:**
   - `undo_step` from a deletion step restores the deleted owned subgraph and
     all of its generated outputs cleanly.
   - Making a new edit after undoing a deletion creates a branch sibling to the
     deletion step; checking out the deletion branch re-retires the entity, and
     checking out the new-edit branch keeps it live with the new edit.
4. **Issue cutover:** Closes `#56`.

### Ownership and meaningful placement

- **Extended owners:**
  - `workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/edit_envelope.rs`:
    add `correction_envelope(...)` reusing
    `application_entry/correction/preparation/envelope.rs`.
  - `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/navigation/`:
    extend `eligibility.rs` and union restoration (`selection.rs`,
    `output_verification.rs`) to admit snapshots separated by recorded
    retirements and restore/re-retire owned entities and output trees
    symmetrically.

### Phase 2 test matrix

| Requirement / law | Setup and entry | Action or failure sequence | Expected result and effects | Decisive observation / defect exposed | Test boundary and lane |
|---|---|---|---|---|---|
| Delete + continue editing without `HeadDiverged` (`#56`) | Attached `OwnedModelGraph` document with 2 owned children + generated outputs | Record corrective retirement of child 2 via `correction_envelope`; then record ordinary `edit_envelope` on child 1 | Both return `Recorded`; child 2 retired, child 1 updated; 0 `HeadDiverged` | Exposes broken head correspondence after authored correction | Decl/Host court (`document_history::commands`) |
| Undo retirement restores exact IDs and outputs (Arch 14, 18) | Document at `root → create → delete` | `undo_step` back to `create`; read entity IDs, relations, and generated outputs; then `redo_to(delete)` | `undo_step` restores exact original entity IDs, relations, and outputs; `redo_to(delete)` re-retires them cleanly | Exposes lost output forest, orphaned relations, or ID regeneration on undo of delete | Decl/Host court (`document_history::navigation`) |
| Branch after undoing a deletion (`#56`, tree semantics) | Document at `root → create → delete` | `undo_step` to `create`; record edit `B` on the restored child; `checkout(delete)` then `checkout(B)` | `create` has children `[delete, B]`; both checkouts produce exact expected live/retired states | Exposes corrupted retirement custody across sibling branches | Decl/Host court (`document_history::navigation`) |
| Foreign-model correction refused (Arch 10) | Sibling models `M1`, `M2` | Call `M1.correction_envelope(...)` targeting an entity owned by `M2` | Refuses `ForeignModel` before effects; `M1` head intact, `M2` entity still live | Exposes missing ownership boundary on corrective entry | Decl/Host court (`document_history::membership`) |
| Out-of-band correction still reports `HeadDiverged` (Arch 10) | Attached document `M1` | Run raw `request.corrections()` bypassing `M1`; then call `M1.edit_envelope(...)` | Refuses `HeadDiverged` honestly; never silently overwrites | Proves document custody is not weakened or bypassed | Decl/Host court (`document_history::commands`) |
| Refused cascade retirement leaves history intact (Arch 14) | Child 2 depended on by live child 3 | Call `correction_envelope` retiring child 2 without retiring dependent child 3 | Refuses with dependency denial; returns intact document; no history node added | Exposes partial retirement or phantom history node on refused delete | Decl/Host court (`document_history::commands`) |
| Retirement + branching state-machine laws (`proptest` 1.x; Testing 2) | 16–32 bounded `proptest` sequences (`proptest-regressions/` committed) extending Phase 1's generator with `correction_envelope` retirements | Random interleaved edits, cascade retirements, undos across retirements, branching edits after undo-of-delete, and checkouts between live/retired sibling branches | After every step, live/retired sets, original full-generation entity IDs, ownership edges, and output forest equal the target node's original snapshot; zero `HeadDiverged` and zero orphaned relations | Exposes retirement custody drift, ID regeneration, or dangling edges across repeated delete/undo/branch cycles | Decl/Host court (`worth-query-court-decl-host`, `document_history::navigation`) |

### Completion gate and documentation

- **Verification:** Phase 1 + Phase 2 `document_history` courts in
  `worth-query-court-decl-host`, `consumer_root` acceptance binary,
  `./loop gate --step guard-diff`, `--step line-caps`, and check-compile of
  dependent query test targets.
- **Docs:** Update `docs/capability-index.md` and `document_history` docs to
  cover recorded corrective retirements and navigation across them. Close `#56`.
