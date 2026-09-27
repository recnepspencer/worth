# Milestone 9.7 Engineering Spec: Concurrent Read Authority And Deterministic Submission

> **Status:** Draft
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Primary predecessors:** [milestone-9.6.md](./milestone-9.6.md), [milestone-9.5.md](./milestone-9.5.md), [milestone-9.4.md](./milestone-9.4.md)
>
> **Purpose:** decompose the Query runtime into authority-typed subsystems so
> committed-snapshot reads scale concurrently across consumers while truth
> mutation and derived maintenance remain single-owner and deterministic â€”
> before Milestone `10` freezes store-backed shapes around the current
> single-borrow topology.

## Goal

Give Query a real multi-consumer operating topology: sealed, basis-bound,
`Send + Sync` shared read contexts that exploit the immutability MVCC already
guarantees; one deterministic submission seam through which all mutation and
declaration work enters in total order; and a published-artifact rule that
keeps derived computation single-owner so concurrent readers can never
perturb evaluation â€” all without changing canonical query meaning or adding a
second semantics path beside the workspace.

## Why This Milestone Exists

Every workspace operation today â€” including read-shaped ones like opening a
preview â€” takes `&mut self`, and sessions are lifetime-bound borrows. The
borrow checker therefore enforces one operation in flight per workspace,
forever, regardless of how concurrent `worth-relational`'s MVCC substrate is
underneath. Snapshot immutability is a structural fact the type system has
never been told.

Without this milestone:

- server-grade consumers must choose between a global lock (throughput cliff,
  MVCC wasted) and branch-per-connection (truth-fork semantics spent on a
  transport problem, subscription sharing defeated, abandoned forks to
  collect) â€” both are exactly the consumer-invented runtime folklore the
  Query hard prohibitions exist to prevent
- Milestone `10` freezes store-backed adapter and execution shapes around the
  single-borrow topology, making the `Send` boundary retrofit dramatically
  more expensive than the decomposition costs now
- determinism remains an accident of single-threadedness instead of a stated
  property of total commit order plus ordered maintenance, pinned by receipts

`arch_laws.md` Law 1 already states the design: read-path constructs borrow
only the observation subsystem, and snapshot isolation is a structural
consequence of correct decomposition. This milestone is that law applied to
the Query runtime.

## Governing Summaries

- `MENTALITY.md`: solve the hard problem first â€” concurrency topology is
  load-bearing infrastructure that must precede the store-backed features
  that would otherwise freeze around its absence.
- `arch_laws.md`: Law 1 (autonomous subsystems; read path borrows only
  observation), Law 10 (methods must not hold artificial locks against data
  they do not touch), Law 18 (phase-typed observation), Law 27 (parallel
  admission only through planner-carried structural disjointness), Law 33
  (authority versus derived state), Law 36 (checkpoint plus bounded journal),
  Law 41 (sealed proof-carrying types).
- `composition_laws.md`: the read authority, submission seam, and publication
  boundary are named structural homes, not behavior smeared through the
  existing workspace internals.
- `domain_structure_laws.md`: authoritative state, derived state, and
  observation must not share structural space; the decomposition must be
  visible in the tree, not just in the types.
- `perf_laws.md`: throughput scales exclusively with data independence;
  reference counting is disguised contention; locality and equivalence
  contracts come before tuning. The read hot path must be lock-free and
  refcount-quiet by structure.
- `worth_query_roadmap.md`: Query owns typed expression, lowering, and result
  shaping; relational owns truth; signal owns reactive evaluation. This
  milestone redistributes *access topology* only â€” no authority moves.

## Adversarial Constraint

N concurrent shared read contexts under sustained commit pressure, preview and
branch session churn, and live maintenance load must produce byte-identical
results and receipts for the same `(canonical declaration, basis capability)`
pair as a fully serialized execution of the same schedule â€” while journal
replay of the submission stream reconstructs identical truth, identical
published derived artifacts, and identical receipts, with zero locks acquired
on the committed-read hot path and zero derived evaluations triggered by any
reader.

This milestone fails if any covered path:

- lets a reader observe non-committed, mid-mutation, or torn state
- lets read concurrency change any result, receipt, digest, or delivery
  content (timing may vary; values may not)
- lets a reader trigger signal-graph evaluation or mutate memoization state
- serializes committed reads through a lock, a refcount storm, or the
  mutation owner
- creates a second query semantics path beside the workspace facade

## Product Decision Lock

- Decomposition per arch law 1: the read lane borrows only observation; write
  authority remains fully accessible beside concurrent readers.
- Committed snapshots become sealed, basis-bound, `Send + Sync` read
  contexts. Mutation keeps exactly one exclusive owner.
- Readers never evaluate. Derived results are published, digest-stamped
  artifacts consumed through the existing projection-consumption lane;
  missing or stale derivations surface as the existing async result-state
  vocabulary, not as reader-side evaluation.
- The submission stream is the journal. Total intake order at the mutation
  owner is the deterministic basis for replay.
- The workspace facade survives unchanged as the single-owner convenience
  over the decomposed authorities. Existing consumers compile untouched.
- No `Mutex`/`RwLock` smearing. Concurrency arrives only through structural
  immutability proofs; any future parallel-write work must arrive through
  planner-carried disjointness proofs per arch law 27, not through this
  milestone.
- All new receipts, journal identity, and published-artifact digests lower
  through the Milestone `9.6` canonical evidence-identity primitive.

## Phase Plan

Phases **1 through 10** establish the concurrency topology, facade families,
and an **interim** hostile schedule. They may ship prototype scaffolding, but
they do **not** close the milestone.

Phases **11 through 18** are the mandatory end-cap honesty program. They exist
because an earlier pass claimed closure while still using snapshot-copy shared
reads, `Mutex` on the read substrate, `commit_identity` string folklore for
journal order, and serial-only hostile certification. Each end-cap phase owns
its substrate **and** its proof obligations â€” inventory slices, forbidden-pattern
scans, hostile schedules, and sabotage tests close **inside** the phase that
ships the substrate, not in a later audit bucket. **Milestone `9.7` may not
report `Closed` until Phase 18 passes with derived proof, not API presence.**

### Phase 1: Backend Adapter Authority-Lane Decomposition Boundary

Split the backend adapter contracts by authority lane â€” committed read,
declaration intake, patch consumption, mutation â€” so no trait object mixes
read access with mutation access, and place `Send + Sync` bounds on the read
lane contracts. Zero behavior change.

**Relevant subsystems**
- `runtime` backend contracts and parts assembly
- runtime builder
- test-support and downstream adapter implementations

**Relevant Query source surfaces**
- [runtime/backend/contracts.rs](../../crates/worth-query/src/runtime/backend/contracts.rs)
- [runtime/backend/parts.rs](../../crates/worth-query/src/runtime/backend/parts.rs)
- [runtime/builder.rs](../../crates/worth-query/src/runtime/builder.rs)

**Relevant APIs and product surfaces**
- `WORTHQueryRuntimeSourceAdapter` and sibling adapter contracts, decomposed
  so read methods (`live_entities`-shaped) and consumption/mutation methods
  (`drain_live_patches`-, `declare_live_view`-shaped) live behind separate
  authority-typed contracts

**Warnings**
- Do not change any adapter behavior in this phase; the split is topology
  only, certified by parity.
- Do not leave a convenience super-trait that re-merges the lanes for
  ergonomics; that is the artificial lock returning with a friendlier name.
- Do not let test-support adapters keep the merged shape; they are part of
  the covered surface.

**Test requirements**
- Add a `Backend Authority Lane Decomposition Parity Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: run a representative covered workload through the
  pre-split and post-split assemblies and prove identical results, receipts,
  and digests.
- Adversarial boundary localization: a compile-fail contract proving a
  read-lane adapter reference cannot reach declaration, consumption, or
  mutation methods.

**Engineering decisions**
- Lane membership of every existing adapter method is decided and recorded in
  this phase; no method remains ambiguously dual-lane.
- `Send + Sync` bounds land on read-lane contracts here even though nothing
  is concurrent yet, so drift is caught at the earliest boundary.

**Open questions**
- None.

### Phase 2: Runtime Assembly Lifecycle Enforcement Boundary

Freeze lane-aware runtime construction: parts intake and builder assembly
consume the decomposed lane contracts, and omitting or failing to propagate a
required lane is a compile-time error at every construction and fork site per
arch law 9.

**Relevant subsystems**
- runtime builder and backend-parts assembly
- construction lifecycle propagation

**Relevant Query source surfaces**
- [runtime/builder.rs](../../crates/worth-query/src/runtime/builder.rs)
- [runtime/backend/parts.rs](../../crates/worth-query/src/runtime/backend/parts.rs)

**Relevant APIs and product surfaces**
- `WORTHQueryRuntime::builder()` lane-typed part intake
- `build_backend_from_parts()` over decomposed lane contracts

**Warnings**
- Do not enforce lane completeness with runtime checks where typestate or
  exhaustive struct expressions can make omission uncompilable.
- Do not let optional lanes blur into required lanes; optionality is part of
  the lane contract, not a builder default.

**Test requirements**
- Add a `Lane-Aware Assembly Enforcement Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: prove a fully assembled lane-typed runtime is
  behaviorally identical to the Phase 1 parity baseline for the covered
  workload.
- Adversarial localization: compile-fail contracts proving builder assembly
  without a required lane, and lane addition without propagation through
  every construction site, both fail to compile.

**Engineering decisions**
- Assembly enforcement is compile-time per the `MENTALITY.md` enforcement
  hierarchy; runtime posture checks remain only for genuinely dynamic backend
  capability.
- This phase changes construction shape only; no execution semantics move.

**Open questions**
- None.

### Phase 3: Snapshot Generation Pinning And Retirement Boundary

Freeze the snapshot retention substrate the shared read lane stands on:
generation-indexed pinning of committed snapshots with explicit retirement,
so read contexts can hold immutable bases without per-operation reference
counting and without unbounded retention.

**Relevant subsystems**
- snapshot retention and generation pinning (new boundary home inside
  `runtime`)
- basis capability lifecycle

**Relevant Query source surfaces**
- [runtime/state_snapshot.rs](../../crates/worth-query/src/runtime/state_snapshot.rs)
- [runtime/workspace.rs](../../crates/worth-query/src/runtime/workspace.rs)

**Relevant APIs and product surfaces**
- generation-indexed snapshot pin and retirement surfaces consumed by the
  Phase 4 read context (internal authority surfaces, not ordinary consumer
  API)

**Warnings**
- Do not pin snapshots with per-call `Arc` traffic; perf law names refcount
  storms as disguised contention. Generation-indexed pinning with explicit
  retirement is the required shape.
- Do not let retention become unbounded; retirement is part of this boundary,
  not a later cleanup task.
- Phase 3 may land internal scaffolding only. Honest closure of pinning
  substrate, hot-path lock posture, and residue counters is deferred to Phases
  **12 and 13**. A registry that copies derived rows or takes a global
  `Mutex` on mint does not satisfy this milestone.

**Test requirements**
- Add a `Snapshot Generation Pinning Scaffold Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: prove a pinned generation observes identical
  snapshot content for its full pin lifetime under sustained commit pressure
  on the covered scaffold harness.
- Adversarial residue: prove that after a hostile pin/retire schedule, zero
  orphaned generations and zero unretired pins remain, as exact counter
  assertions sourced from runtime-owned counters â€” not inferred values.
- This phase does **not** close hot-path lock posture; that is Phase 12.

**Engineering decisions**
- Pin identity is generation-indexed (index + generation) per perf-law
  identity guidance, not key-based lookup.
- Retirement is explicit and observable; generation drain is a counted event,
  not a destructor side effect.

**Open questions**
- None.

### Phase 4: Sealed Basis-Bound Shared Read Context Boundary

Admit the shared read lane: a sealed read context minted only at commit
boundaries, carrying its basis capability proof, `Send + Sync`, cheaply
shareable across threads, and holding its snapshot through the Phase 3
pinning substrate.

**Relevant subsystems**
- read authority (new boundary home inside `runtime`)
- basis capability lifecycle
- snapshot generation pinning (consumed from Phase 3)

**Relevant Query source surfaces**
- [runtime/workspace.rs](../../crates/worth-query/src/runtime/workspace.rs)
- [runtime/workspace_queries.rs](../../crates/worth-query/src/runtime/workspace_queries.rs)
- [runtime/state_snapshot.rs](../../crates/worth-query/src/runtime/state_snapshot.rs)

**Relevant APIs and product surfaces**
- the sealed shared read context constructor on the workspace (the only legal
  mint, per arch law 41)
- read execution over the context for admitted one-shot query families
- `workspace.state(...)`-shaped readiness over the shared lane

**Target shape (illustrative, not frozen API)**

The single-borrow topology this phase opens up, as it constrains consumers
today (`crates/worth-kernel/src/construction/runtime_proof/runtime_basis.rs`):

```rust
// BEFORE: even pure inspection reports demand exclusive access,
// so no two of these can ever run at once
pub fn prepare_primitive_construction_branch_preview_runtime_report(
    workspace: &mut WORTHQueryWorkspace,
    intent: impl PrimitiveConstructionAuthoringInput,
) -> Result<..., PrimitiveConstructionRuntimeBasisError>
```

The target shape after this phase:

```rust
// AFTER: read-shaped work moves to a shared, Send + Sync, basis-pinned
// context; N reports run on N threads against the same committed basis
let read_ctx: WORTHQuerySharedReadContext = workspace.shared_read_context()?;

let reports: Vec<_> = std::thread::scope(|s| {
    intents.iter()
        .map(|intent| s.spawn(|| prepare_parity_report(&read_ctx, intent))) // &, not &mut
        .map(|handle| handle.join().unwrap())
        .collect()
});
// byte-identical receipts to serialized execution â€” certified in Phase 16
```

**Warnings**
- Do not let the read context be constructible from a raw snapshot id, branch
  id, or label; it exists only as the product of basis capability admission.
- Do not give the read context any method whose execution writes â€” including
  "convenience" cache warming or lazy index repair.
- Phase 4 may ship a serial-only read context over copied snapshot state.
  `Send + Sync`, real N-thread equivalence, and structural pinning honesty are
  deferred to Phases **13 and 16**. Minting a context by cloning derived rows
  into a side registry is prototype debt, not closure.

**Test requirements**
- Add a `Shared Read Context Serial Scaffold Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: a single-threaded schedule reading through shared
  contexts under commit pressure produces byte-identical results and receipts
  to the same schedule executed through the workspace convenience path.
- Adversarial denial: a context whose snapshot generation has been retired
  fails closed with a typed stale-basis stop; it must never silently rebind
  to a newer snapshot.
- N-thread concurrency equivalence is **not** closure for this phase; it is
  Phase 16.

**Engineering decisions**
- Read contexts are immutable values; refreshing to a newer basis is an
  explicit re-mint through the workspace, never an in-place mutation.
- Receipts emitted through the shared lane record the basis proof and lower
  identity through the `9.6` evidence primitive.

**Open questions**
- None.

### Phase 5: Deterministic Submission Intake And Journal Boundary

Admit the single-writer submission seam: typed intake of canonical
declaration and mutation work into the mutation owner, where total intake
order is the journal, admission flows through the existing intent admission
lattice, and completed work returns the existing receipt and envelope
artifacts.

**Relevant subsystems**
- submission intake (new boundary home inside `runtime`)
- intent admission lattice
- write receipts and boundary envelopes
- journal identity

**Relevant Query source surfaces**
- [runtime/workspace.rs](../../crates/worth-query/src/runtime/workspace.rs)
- [runtime/workspace_declaration.rs](../../crates/worth-query/src/runtime/workspace_declaration.rs)

**Relevant APIs and product surfaces**
- the typed submission surface accepting canonical declarations and admitted
  intent families
- ordered submission receipts carrying journal position identity
- the existing admission, receipt, and envelope vocabulary â€” reused, not
  duplicated

**Target shape (illustrative, not frozen API)**

```rust
// AFTER: mutation work enters as canonical declarations through one ordered
// intake; the receipt records the journal position the work committed at
let receipt = workspace.submissions().submit(construction_declaration)?;
assert!(receipt.journal_position().is_committed());

// admission stops are the same typed lattice outcomes as today:
// denied/advisory/violation with decision traces â€” submission adds ordering,
// not a second admission vocabulary
```

**Warnings**
- Do not build a second admission path inside the seam; submissions enter the
  existing lattice or they do not enter.
- Do not let the seam accept raw payloads, closures, or host callbacks; the
  intake vocabulary is canonical declarations and admitted intents only.
- Do not make submission ordering depend on thread scheduling accidents;
  intake order is assigned at one authority point and recorded in the
  receipt.

**Test requirements**
- Add a `Deterministic Submission Journal Replay Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: submit an interleaved multi-producer workload,
  record the journal, replay it, and prove identical truth, receipts, and
  digests â€” including across process restart of the runtime-backed harness.
- Adversarial rejection: duplicate submission of the same canonical identity
  and submission against a stale basis both stop typed at admission, with
  decision traces preserved, and leave no journal residue.

**Engineering decisions**
- The journal is the submission stream itself; checkpoint-plus-journal
  reconstruction per arch law 36 is certified here at runtime-backed scope,
  while durable journal persistence remains Milestone `10`/`11` scope.
- Journal position identity lowers through the `9.6` evidence primitive.

**Open questions**
- None.

### Phase 6: Consumer-Facing Journal Replay Boundary

Admit the public replay surface over recorded journal segments, so downstream
applications stop re-deriving state by folding locally retained change
artifacts. Today both `workflow-editor` and `hadwiger-research` implement
replay as consumer-side refolds because no runtime-owned replay lane exists.

**Relevant subsystems**
- consumer replay surface (new boundary home over the Phase 5 journal)
- basis capability lifecycle
- journal segment identity

**Relevant Query source surfaces**
- [runtime/workspace.rs](../../crates/worth-query/src/runtime/workspace.rs)
- [runtime/workspace_declaration.rs](../../crates/worth-query/src/runtime/workspace_declaration.rs)

**Relevant APIs and product surfaces**
- typed replay requests binding a basis capability to a journal segment
  identity
- replay outputs as ordinary receipts and envelopes â€” not a parallel result
  vocabulary

**Warnings**
- Do not expose raw journal entries as the replay vocabulary; consumers
  request replay outcomes, not journal internals.
- Do not let replay become a second execution semantics; replay re-derives
  through the same admission, execution, and receipt paths the original run
  used.

**Test requirements**
- Add a `Consumer Journal Replay Closure Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: a consumer-shaped test replays a recorded journal
  segment through the public replay surface and proves the reconstructed
  state, receipts, and digests match the original run exactly â€” without the
  consumer retaining or refolding any change artifacts of its own.
- Adversarial denial: replay requests against a mismatched basis, an unknown
  segment identity, or a cross-scheme digest stop typed at admission and
  leave no replay residue.

**Engineering decisions**
- The replay surface is consumer product surface, not internal determinism
  machinery.
- Replay request and segment identity lower through the `9.6` evidence
  primitive.

**Open questions**
- None.

### Phase 7: Maintenance-Owner Publication Boundary

Close the owner half of the rule that keeps determinism alive under
concurrency: derived computation is evaluated only by the single maintenance
owner, which publishes atomic, digest-stamped result artifacts in commit
order.

**Relevant subsystems**
- signal-facing maintenance ownership
- published artifact registry

**Relevant Query source surfaces**
- [runtime/workspace_queries.rs](../../crates/worth-query/src/runtime/workspace_queries.rs)
- [projection_consumption/receipt.rs](../../crates/worth-query/src/projection_consumption/receipt.rs)

**Relevant APIs and product surfaces**
- maintenance-owner publication receipts binding artifact digest to basis and
  commit position
- the published artifact registry the Phase 8 reader lane consumes

**Warnings**
- Do not let lazy pull escape the maintenance owner; evaluation legality
  lives here, before any reader lane exists to abuse it.
- Do not let publication and the Phase 5 journal develop independent
  orderings; the maintenance owner is the same single-writer locus as the
  submission seam or is fed by it in commit order.

**Test requirements**
- Add a `Maintenance Owner Publication Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: for every covered derived family, prove the
  published artifact at basis B equals a from-authority recomputation at
  basis B (arch law 33 honesty).
- Adversarial atomicity: prove an observer of the registry during
  republication sees either the old artifact or the new one â€” never a blend â€”
  and that publication order matches commit order exactly under hostile
  scheduling.

**Engineering decisions**
- Publication is atomic per artifact: digest-stamped, basis-bound, commit-
  ordered, and lowered through the `9.6` evidence primitive.
- Two orderings may not exist; publication order is commit order.

**Open questions**
- None.

### Phase 8: Reader Consumption And Async Posture Boundary

Close the reader half: shared read contexts consume published artifacts
through the projection-consumption lane, missing or republishing derivations
surface as typed async result-state, and no reader-reachable path can trigger
evaluation.

**Relevant subsystems**
- projection consumption
- async result-state surfaces
- shared read context (from Phase 4)

**Relevant Query source surfaces**
- [projection_consumption/source.rs](../../crates/worth-query/src/projection_consumption/source.rs)
- [projection_consumption/receipt.rs](../../crates/worth-query/src/projection_consumption/receipt.rs)

**Relevant APIs and product surfaces**
- published derived-artifact consumption through the shared read context
- the existing async result-state vocabulary (`pending`, `current`, `stale`,
  `revalidating`, `superseded`) carrying not-yet-published posture

**Target shape (illustrative, not frozen API)**

```rust
// AFTER: a reader consuming a derivation gets the last published,
// digest-stamped artifact through the projection-consumption lane â€”
// it can never trigger evaluation itself
match read_ctx.consume_projection(derived_facts_declaration)? {
    WORTHQueryProjectionConsumption::Current(facts) => {
        // facts.receipt() binds artifact digest to basis and commit position
    }
    WORTHQueryProjectionConsumption::ResultState(state) => {
        // not-yet-published or republishing posture arrives as the existing
        // async result-state vocabulary: pending / stale / revalidating â€”
        // never as a reader-side recomputation
    }
}
```

**Warnings**
- Do not let any reader-reachable path call into signal evaluation; lazy pull
  remains legal only inside the Phase 7 maintenance owner.
- Do not invent a new "derived read" API; consumption flows through the
  projection-consumption lane Milestone `9.5` made first-class.
- Do not erase merged `9.4` temporal/async meaning when publication posture
  carries it; a time-driven artifact that has not republished is `stale` or
  `revalidating`, never silently recomputed reader-side.

**Test requirements**
- Add a `Published Artifact Reader Isolation Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: concurrent readers consuming a covered derived
  family during republication observe either the old artifact or the new one
  with matching receipts â€” never a blend â€” and identical fact content to a
  serialized consumer of the same schedule.
- Adversarial leakage: exact-zero counter assertion that no reader execution
  path performed a signal-graph evaluation, plus a hostile probe proving a
  reader requesting an unpublished derivation receives typed `pending`
  posture rather than a result.

**Engineering decisions**
- Reader consumption is receipt-backed projection consumption; retained rows
  and payload bags do not return as the derived-read vocabulary.
- Async posture preservation is part of this boundary's closure, not a
  temporal follow-up.

**Open questions**
- None.

### Phase 9: Workspace Facade Re-Expression Boundary

Re-express `WORTHQueryWorkspace` as the single-owner convenience facade over
the decomposed authorities so existing consumers compile unchanged, the new
lanes are reachable from the facade, and construction lifecycle propagation
is compile-enforced.

**Relevant subsystems**
- workspace facade
- runtime construction and lifecycle propagation
- support/admission reporting for the new families

**Relevant Query source surfaces**
- [runtime/workspace.rs](../../crates/worth-query/src/runtime/workspace.rs)
- [runtime/builder.rs](../../crates/worth-query/src/runtime/builder.rs)
- [runtime/support/profile.rs](../../crates/worth-query/src/runtime/support/profile.rs)
- [runtime/support_matrix.rs](../../crates/worth-query/src/runtime/support_matrix.rs)

**Relevant APIs and product surfaces**
- `WORTHQueryWorkspace` â€” unchanged signatures for existing families, plus
  mint points for shared read contexts and the submission seam
- support matrix and admission rows for the new `SharedRead` and
  `Submission` facade families, fail-closed where a backend posture cannot
  honor them

**Warnings**
- Do not deprecate the single-owner workspace; kernel-grade single-threaded
  consumers are first-class, not legacy.
- Do not expose the decomposed internals as public topology; the facade and
  the sealed artifacts are the only surface per domain structure law 21.
- Do not let the new families ship visible-but-unadmitted without fail-closed
  admission rows; support honesty is part of this phase, not Phase 6 polish.

**Test requirements**
- Add a `Facade Lane Parity And Lifecycle Propagation Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: the same covered operation executed through the
  facade convenience path and through the decomposed lanes produces identical
  receipts and digests.
- Adversarial localization: a compile-fail contract proving that adding a new
  runtime authority subsystem without propagating it through every
  construction and fork site fails to compile (arch law 9), and that
  downstream consumer code cannot reach decomposed internals past the facade.

**Engineering decisions**
- Existing downstream call sites (including `worth-kernel`) must compile
  without modification; this is an acceptance criterion, not a hope.
- New facade families enter the support matrix with the same admission,
  teaching-posture, and fail-closed discipline as every existing row.

**Open questions**
- None.

### Phase 10: Interim Hostile Schedule Baseline Boundary

Land an interim hostile schedule that exercises Phases 1â€“9 in one program and
records the gaps the end-cap phases must close. This phase proves the harness
exists; it does **not** close the milestone.

**Relevant subsystems**
- runtime-backed certification harness (the Milestone `9.5` raw runtime read
  bootstrap is the required entry)
- all Phase 1â€“9 boundaries

**Relevant Query source surfaces**
- [tests/support/public_bridge_runtime/mod.rs](../../crates/worth-query/tests/support/public_bridge_runtime/mod.rs)
- [runtime/tests/support/bridge/runtime_support.rs](../../crates/worth-query/src/runtime/tests/support/bridge/runtime_support.rs)

**Warnings**
- Do not treat a serial schedule as concurrent certification. Real N-thread
  readers and M submitters are Phase 16.
- Do not accept threshold-shaped assertions where exact counters are
  achievable; lock count and reader-evaluation count are exact zeros, not
  "low".
- Do not close this phase if journal gap counting parses `commit_identity`
  text, if pin residue is hard-coded zero, or if shared read mint copies
  derived rows into a side registry. Those are explicit debt markers for
  Phases 11â€“18.

**Test requirements**
- Add a `Milestone 9.7 Interim Hostile Schedule Baseline Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: the interim schedule produces byte-identical
  receipts, results, and published-artifact digests to its serialized replay
  on the covered harness.
- Adversarial debt publication: the schedule artifact must record which
  guarantees are still modeled (snapshot copy, `Mutex` on read substrate,
  string-derived journal order, serial-only readers) so Phases 11â€“18 cannot
  be skipped silently.

**Engineering decisions**
- Certification artifacts lower through the `9.6` evidence primitive so the
  interim matrix is replay-comparable.
- This phase is a harness checkpoint, not milestone closure.

**Open questions**
- None.

### Phase 11: Published Artifact Registry Authority Boundary

Establish a single runtime-owned published-artifact registry as the only legal
source of derived facts for shared read and projection consumption. Shared read
must hold generation handles into this registry, not cloned materialization
rows. **This phase seeds the pinning inventory** and must close on registry-
slice proof before Phase 12 begins.

**Relevant subsystems**
- published derived-artifact registry
- projection consumption lowering
- shared-read mint path
- pinning inventory (registry and mint slice)

**Relevant Query source surfaces**
- [runtime/shared_read.rs](../../crates/worth-query/src/runtime/shared_read.rs)
- [runtime/state.rs](../../crates/worth-query/src/runtime/state.rs)
- [runtime/workspace_shared_read.rs](../../crates/worth-query/src/runtime/workspace_shared_read.rs)
- [application/support/shared_read_pinning_inventory.rs](../../crates/worth-query/src/application/support/shared_read_pinning_inventory.rs) (new)

**Relevant APIs and product surfaces**
- runtime-owned published-artifact registry with generation-indexed entries
- shared-read mint that acquires a registry lease, not a row snapshot copy
- versioned pinning inventory listing every ordinary registry and mint path

**Warnings**
- Do not satisfy this phase by copying `materialization` rows, projection
  facts, or digest tables into `WORTHQuerySharedReadContext` fields.
- Do not let shared read bypass the registry through direct workspace state
  reads when the consumer need is published derived truth.
- Do not treat a `HashMap` side cache populated at mint time as a registry.
- Do not close this phase without seeding the pinning inventory and running
  forbidden-pattern scans on the registry/mint slice. Inventory is substrate,
  not a later audit chore.

**Test requirements**
- Add a `Published Artifact Registry Authority Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- **Inventory seed:** create the embedded, versioned pinning inventory and
  register every ordinary `runtime/` and `application/` path that owns registry
  authority or shared-read mint. Later phases extend this inventory; they may
  not replace it with ad-hoc greps.
- **Forbidden-pattern scans (registry/mint slice):** scans fail closed on:
  - materialization or projection-fact row clones at shared-read mint
  - `HashMap` or side-cache population at mint time standing in for registry
    authority
  - display-string or `commit_identity` parsing used as generation identity
- Adversarial equivalence: two shared-read contexts minted against the same
  committed generation observe identical published facts through the registry
  without duplicating row storage per context.
- Adversarial boundary localization: compile-fail or source-scan contracts
  proving shared-read mint cannot call row-clone helpers on inventoried paths.
- Adversarial republication: derived republication updates registry entries
  for new generations while pinned contexts continue to observe their leased
  generation.
- Adversarial sabotage: reintroducing a row-clone mint helper on an
  inventoried path fails the phase test.

**Engineering decisions**
- The registry is structural runtime state, not a convenience cache.
- Shared-read contexts are cheap leases over registry generations.
- Pinning inventory begins here; each subsequent pinning phase extends and
  re-scans its slice rather than deferring proof.

**Open questions**
- None.

### Phase 12: Generation Pinning Substrate And Hot-Path Lock Posture Boundary

Replace prototype pin bookkeeping with runtime-owned generation-indexed pinning,
explicit retirement, and mechanically provable hot-path lock freedom on
committed reads. **This phase extends the pinning inventory** to every pin and
retire path and must close on pin-substrate proof before Phase 13 begins.

**Relevant subsystems**
- snapshot generation pinning and retirement
- shared-read pin registry substrate
- runtime measurement counters
- pinning inventory (pin and retire slice)

**Relevant Query source surfaces**
- [runtime/shared_read_pins/registry.rs](../../crates/worth-query/src/runtime/shared_read_pins/registry.rs)
- [runtime/state_snapshot.rs](../../crates/worth-query/src/runtime/state_snapshot.rs)
- [runtime/workspace.rs](../../crates/worth-query/src/runtime/workspace.rs)
- [application/support/shared_read_pinning_inventory.rs](../../crates/worth-query/src/application/support/shared_read_pinning_inventory.rs)

**Relevant APIs and product surfaces**
- typed pin and retire surfaces with runtime-owned residue counters
- always-on measurement hooks for lock acquisitions on the shared-read hot path
- inventory extension covering every ordinary pin, retire, and generation-drain
  path

**Warnings**
- Do not use `Mutex`, `RwLock`, or equivalent exclusive locks on the
  committed-read hot path. Pin book-keeping may use lock-free or single-writer
  structures only.
- Do not model pin retirement through `Drop` heuristics alone; retirement must
  be explicit, observable, and certifiable.
- Do not satisfy residue counters with constants, `assert_eq!(0, 0)`, or
  certification-only helpers that are not wired to runtime measurement state.
- Do not close this phase without extending the inventory and running
  forbidden-pattern scans on the pin/retire slice.

**Test requirements**
- Add a `Generation Pinning Hot-Path Lock Posture Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- **Inventory extension:** register every ordinary path that pins, retires, or
  drains snapshot generations. Any path omitted here keeps pinning posture
  `Open` regardless of later phase claims.
- **Forbidden-pattern scans (pin/retire slice):** scans fail closed on:
  - `Mutex` / `RwLock` on shared-read hot paths
  - `Drop`-only pin retirement without explicit retire surfaces
  - hard-coded zero residue helpers not sourced from runtime counters
  - formatted `snapshot_token` conventions standing in for typed pin identity
- Adversarial equivalence: a pinned generation observes identical registry
  content for its full pin lifetime under sustained commit pressure.
- Adversarial residue: hostile pin/retire schedules prove exact-zero orphaned
  generations and exact-zero unretired pins through runtime-owned counters.
- Adversarial contention proof: exact-zero lock acquisitions on the shared-read
  hot path under a hostile read schedule, with measurement hooks compiled in
  for release builds used by certification â€” not `#cfg(test)` fiction.
- Adversarial sabotage: perturb pin-residue counters or inject a deliberate
  unretired pin and prove this phase's test fails.

**Engineering decisions**
- Pin identity is a real runtime artifact with explicit generation semantics,
  not a formatted `snapshot_token` convention.
- Measurement counters are ordinary runtime fields read by certification; they
  are never test-only stubs.

**Open questions**
- None.

### Phase 13: Shared Read Context And Pinning Boundary Closure

Make `WORTHQuerySharedReadContext` the real product surface â€” sealed,
basis-bound, `Send + Sync`, holding a registry lease and pinned generation â€”
and **close the entire pinning boundary end-to-end inside this phase**. No
later phase re-audits pinning; journal and certification phases may assume
pinning is honestly `Closed` when Phase 13 passes.

**Relevant subsystems**
- read authority
- basis capability lifecycle
- shared-read execution path
- pinning inventory (completeness and closure posture)
- hostile pinning certification

**Relevant Query source surfaces**
- [runtime/shared_read.rs](../../crates/worth-query/src/runtime/shared_read.rs)
- [runtime/workspace_shared_read.rs](../../crates/worth-query/src/runtime/workspace_shared_read.rs)
- [runtime/workspace_queries.rs](../../crates/worth-query/src/runtime/workspace_queries.rs)
- [runtime/shared_read_pins/](../../crates/worth-query/src/runtime/shared_read_pins/)
- [application/support/shared_read_pinning_inventory.rs](../../crates/worth-query/src/application/support/shared_read_pinning_inventory.rs)
- [runtime/tests/shared_read_pinning/](../../crates/worth-query/src/runtime/tests/shared_read_pinning/) (new hostile matrix home)

**Relevant APIs and product surfaces**
- `WORTHQuerySharedReadContext: Send + Sync` with compile-time proof tests
- stale-basis typed denial when the pinned generation retires
- derived pinning-boundary posture (`Open`, `Partial`, `Closed`)

**Warnings**
- Do not silently rebind an old shared-read context to a newer committed
  snapshot when its pinned generation is retired; stale basis must fail typed.
- Do not add interior mutability to "refresh" basis in place.
- Do not claim `Send + Sync` without `std::thread::scope` proof on the real
  context type, not a test-only wrapper.
- Do not close on an inventory that omits sibling or feeder paths because they
  were "not in the original list." Same-class residue anywhere in ordinary
  runtime/product paths keeps posture `Open`.
- Do not exclude paths without naming a different milestone-class owner.

**Test requirements**
- Add a `Shared Read Context And Pinning Boundary Closure Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- **Inventory completeness:** extend the Phase 11â€“12 inventory to every ordinary
  path that executes reads through shared-read authority or consumes published
  artifacts on the shared lane. The merged inventory is the contract surface.
- **Forbidden-pattern scans (full pinning boundary):** re-scan every
  inventoried path for all pinning defect classes (row clone at mint, hot-path
  locks, hard-coded zeros, string-derived generation identity, `Drop`-only
  retirement). This phase may not close while any scan is red.
- Adversarial equivalence: contexts minted at the same generation produce
  byte-identical receipts and facts for their full legal lifetime; concurrent
  pin mint under sustained commit pressure matches the serial baseline until
  explicit retirement.
- Adversarial denial: retired-generation contexts fail closed with typed
  stale-basis stops and never observe newer registry content.
- Adversarial portability: `Send + Sync` compile assertions plus scoped-thread
  smoke proof on the real shared-read type.
- Adversarial residue: exact-zero orphaned generations, exact-zero unretired
  pins, and exact-zero hot-path lock acquisitions after the hostile pinning
  matrix â€” each from runtime-owned counters.
- Adversarial sabotage: perturb each pinning residue counter and prove this
  phase's closure test fails. A proof that stays green under sabotage is
  invalid evidence.
- **Derived pinning posture:** support/profile reports `Closed` for the pinning
  boundary only when inventory scans and the hostile pinning matrix are
  simultaneously green; otherwise `Open` or `Partial` with enumerated residue.

**Engineering decisions**
- Refreshing to a newer basis is an explicit re-mint through the workspace,
  never an in-place mutation.
- Receipts emitted through the shared lane record the basis proof and lower
  identity through the `9.6` evidence primitive.
- Pinning honesty closes here in the same spirit as Milestone `9.6`'s
  identity-boundary inventory: derived, adversarial, and end-to-end â€” but owned
  by the pinning phases, not deferred.
- Phase 13 must pass before journal or certification phases may claim closure.

**Open questions**
- None.

### Phase 14: Typed Journal Position Identity Boundary

Introduce typed journal position identity on committed submission receipts.
Journal order is a product artifact, not a naming convention emergent from
mutation receipt strings. **This phase seeds the journal inventory** and must
close on journal-identity proof before Phase 15 begins.

**Relevant subsystems**
- submission intake and receipt identity
- journal position artifact
- journal inventory (submission and receipt slice)

**Relevant Query source surfaces**
- [runtime/workspace_submission.rs](../../crates/worth-query/src/runtime/workspace_submission.rs)
- [runtime/backend/receipts.rs](../../crates/worth-query/src/runtime/backend/receipts.rs)
- [application/support/journal_identity_inventory.rs](../../crates/worth-query/src/application/support/journal_identity_inventory.rs) (new)

**Relevant APIs and product surfaces**
- `WORTHQueryJournalPosition` (or equivalent sealed typed artifact) on receipts
- typed accessors for journal order â€” never `commit_identity` suffix parsing
- versioned journal inventory listing every ordinary submission and receipt path

**Warnings**
- Do not keep treating `commit_identity` string suffixes, digit runs, or display
  formatting as journal order anywhere in runtime, support, or certification.
- Do not store journal order only in test helpers while production paths still
  parse strings.
- Do not conflate journal position with evidence identity; they lower through
  distinct typed artifacts.
- Do not close this phase without seeding the journal inventory and running
  forbidden-pattern scans on the submission/receipt slice.

**Test requirements**
- Add a `Typed Journal Position Identity Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- **Inventory seed:** create the embedded, versioned journal inventory and
  register every ordinary path that records, reads, or compares journal order.
- **Forbidden-pattern scans (submission/receipt slice):** scans fail closed on
  `commit_identity` parsing, digit-suffix extraction, or display-string ordering
  helpers in runtime, support, and certification crates.
- Adversarial equivalence: a multi-producer submission schedule records
  monotonic typed journal positions whose ordering is stable across replay.
- Adversarial collision: distinct commits never compare equal on journal
  position unless they are the same commit.
- Adversarial sabotage: reintroducing a `commit_identity` parsing helper on an
  inventoried path fails the phase test.

**Engineering decisions**
- Journal position lowers through the `9.6` evidence scheme as its own typed
  artifact, not as a derived string view.
- Journal inventory begins here; Phase 15 extends and closes it.

**Open questions**
- None.

### Phase 15: Consumer Journal Segment Replay Surface Boundary

Expose a first-class consumer replay lane: typed journal-segment identity in,
replay outcome out â€” and **close the journal boundary end-to-end inside this
phase**. No later phase re-audits journal identity.

**Relevant subsystems**
- consumer-facing replay surface
- journal segment identity
- replay lowering through runtime
- journal inventory (completeness and closure posture)

**Relevant Query source surfaces**
- [runtime/public_api.rs](../../crates/worth-query/src/runtime/public_api.rs)
- [runtime/workspace_submission.rs](../../crates/worth-query/src/runtime/workspace_submission.rs)
- [application/support/journal_identity_inventory.rs](../../crates/worth-query/src/application/support/journal_identity_inventory.rs)

**Relevant APIs and product surfaces**
- typed journal-segment / replay-request surfaces on the workspace facade
- replay results returned through ordinary receipt / envelope vocabulary
- derived journal-boundary posture (`Open`, `Partial`, `Closed`)

**Warnings**
- Do not expose raw journal internals when the consumer need is replay outcome.
- Do not build a second replay semantics path beside ordinary submission.
- Do not implement replay by re-running certification helpers; consumers use
  the public lane.
- Do not close without extending the journal inventory to every replay and
  certification path that derives journal order or gap counts.

**Test requirements**
- Add a `Consumer Journal Segment Replay Surface Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- **Inventory completeness:** extend the Phase 14 inventory to every ordinary
  replay, certification, and support path that reads or compares journal order.
- **Forbidden-pattern scans (full journal boundary):** re-scan every
  inventoried path for `commit_identity` parsing and string-derived journal gap
  counting. This phase may not close while any scan is red.
- Adversarial equivalence: replaying a typed journal segment reconstructs
  identical truth, receipts, published artifacts, and digests.
- Adversarial rejection: stale-basis replay, unknown segment identity, and
  cross-scheme replay requests fail typed and leave zero journal residue.
- Adversarial consumer shape: at least one downstream-shaped test calls the
  public replay surface without bridge-only shortcuts.
- Adversarial sabotage: reintroducing string-derived journal ordering on an
  inventoried path fails this phase's closure test.
- **Derived journal posture:** support/profile reports `Closed` for the journal
  boundary only when inventory scans and replay proof are simultaneously green.

**Engineering decisions**
- Consumer replay is ordinary product surface because downstream refolds are
  the folklore this milestone eliminates.
- Journal honesty closes here â€” owned by Phases 14â€“15, not deferred.

**Open questions**
- None.

### Phase 16: Real Concurrent Hostile Certification Matrix Boundary

Replace the interim serial schedule with a true concurrent hostile matrix: N
reader threads, M submitter threads, derived republication, preview/branch
churn, and replay â€” combined in one program. **Counter integrity and sabotage
proof close inside this phase**; there is no separate certification-audit phase.

**Relevant subsystems**
- hostile certification harness
- shared read, submission, and replay lanes together
- runtime measurement counters

**Relevant Query source surfaces**
- [runtime/tests/support/bridge/hostile_certification_schedule.rs](../../crates/worth-query/src/runtime/tests/support/bridge/hostile_certification_schedule.rs)
- [runtime/tests/support/bridge/hostile_certification.rs](../../crates/worth-query/src/runtime/tests/support/bridge/hostile_certification.rs)

**Warnings**
- Do not certify axes in isolation.
- Do not substitute scoped-thread smoke tests for the full matrix.
- Do not claim concurrency if readers are still serialized behind a global
  lock or workspace `&mut` exclusivity.
- Do not leave hard-coded zero helpers in the certification path.
- Do not compute journal gaps by parsing `commit_identity` text.
- Do not gate measurement hooks behind `#[cfg(test)]` if certification reads
  them in integration runs.

**Test requirements**
- Add a `Real Concurrent Hostile Certification Matrix Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: the full hostile schedule produces byte-identical
  receipts, results, and published-artifact digests to its serialized replay,
  across repeated runs with distinct thread interleavings.
- Adversarial topology: at least three concurrent reader threads and two
  concurrent submitter threads on the real shared-read and submission types.
- Adversarial proof integrity: lock count, reader-evaluation count, pin
  residue, and journal-gap counters are read from runtime-owned measurement
  state in the certification artifact â€” not constants or parsed strings.
- Adversarial residue: after the schedule, exact-zero orphaned snapshot
  generations, exact-zero unretired read pins, exact-zero journal gaps, and
  exact-zero delivery residue.
- Adversarial sabotage: for each counter class, a targeted regression test
  proves this phase's matrix fails when that counter is intentionally perturbed
  or when a forbidden shortcut is reintroduced.
- Adversarial replay integrity: certification artifacts lower through the
  `9.6` evidence primitive and compare equal across replay.

**Engineering decisions**
- This phase supersedes Phase 10 as the concurrency certification authority.
- A certification artifact that stays green under sabotage is not acceptance
  evidence.

**Open questions**
- None.

### Phase 17: Public-Bridge Reader-Lane And Projection-Consumption Honesty Boundary

Close the public-bridge certification path so it consumes published derived
artifacts only through typed projection consumption â€” never through direct
materialization row reads. **Inventory, compile-fail localization, and sabotage
close inside this phase.**

**Relevant subsystems**
- public-bridge runtime certification harness
- projection-consumption product lane

**Relevant Query source surfaces**
- [tests/support/public_bridge_runtime/hostile_certification.rs](../../crates/worth-query/tests/support/public_bridge_runtime/hostile_certification.rs)
- [tests/support/public_bridge_runtime/mod.rs](../../crates/worth-query/tests/support/public_bridge_runtime/mod.rs)

**Relevant APIs and product surfaces**
- public-bridge hostile certification artifact
- projection-consumption receipts as the only reader truth path

**Warnings**
- Do not let public-bridge certification read materialization rows directly from
  bindings when the milestone claims projection-consumption isolation.
- Do not pass this phase with bridge-only helpers that ordinary consumers cannot
  reach.

**Test requirements**
- Add a `Public-Bridge Reader-Lane Honesty Closure Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: the public-bridge hostile matrix produces
  replay-stable artifacts while consuming published derived facts only through
  the typed reader lane.
- Adversarial boundary localization: public-bridge hostile certification loses
  compile access to direct internal materialization-reading shortcuts.
- Adversarial sabotage: reintroducing a row-spelunking shortcut fails
  certification.

**Engineering decisions**
- Public-bridge honesty is part of milestone closure, not a test-only nicety.

**Open questions**
- None.

### Phase 18: Derived Milestone Closure Posture And Closeout Boundary

Close Milestone `9.7` only when support/profile, docs, test-requirements, and
hostile certification agree â€” with posture **aggregated from phase-local
closure proofs**, not re-audited here.

**Relevant subsystems**
- support/profile closure publication
- milestone closeout docs
- test-requirements certification matrix

**Relevant Query source surfaces**
- [application/support/closure.rs](../../crates/worth-query/src/application/support/closure.rs)
- [plans/worth-query/test-requirements.md](./test-requirements.md)
- [plans/worth-query/milestone-9.7-closeout.md](./milestone-9.7-closeout.md) (new)

**Relevant APIs and product surfaces**
- derived `9.7` milestone posture (`Open`, `Partial`, `Closed`)
- closeout note citing phase-local inventory digests and certification artifacts

**Warnings**
- Do not mark the milestone `Closed` in roadmap or spec status while any
  Phase 11â€“17 gate is incomplete or red.
- Do not hard-code `Closed` in support/profile while any phase-local boundary
  (pinning, journal, certification, public-bridge) reports `Open` or `Partial`.
- Do not re-run pinning or journal audits in this phase; those closed in Phases
  13 and 15. This phase only aggregates their derived posture.

**Test requirements**
- Add a `Milestone 9.7 Derived Closure Posture Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial derived posture: `Closed` appears only when Phase 13 pinning
  posture, Phase 15 journal posture, Phase 16 hostile matrix (with sabotage),
  and Phase 17 public-bridge honesty are simultaneously green.
- Adversarial documentation parity: `milestone-9.7-closeout.md`,
  `test-requirements.md` matrix rows, and support/profile output agree on
  posture and enumerate any defended exclusions.
- Adversarial regression guard: returning any forbidden pattern re-opens the
  owning phase's boundary posture in CI.

**Engineering decisions**
- This is the only phase that may set milestone status to `Closed`.
- Closeout cites the Phase 13 pinning inventory digest, Phase 15 journal
  inventory digest, and Phase 16 certification artifact digests as primary
  evidence.

**Open questions**
- None.

## Closure Gate: Honest Completion Criteria

Milestone `9.7` is an all-or-nothing concurrency-and-authority milestone. It
is not enough for facade methods to exist, for a serial hostile schedule to
pass, or for shared-read types to compile. The real closure question is whether
Query can now be believed as the single ordinary-path owner of:

- runtime-owned shared read authority backed by generation pinning â€” not copied
  snapshots
- lock-free committed reads with exact-zero hot-path lock acquisitions
- typed journal position and consumer replay identity â€” not `commit_identity`
  string folklore
- true concurrent reader/submitter certification with sabotage-sensitive counters
- public-bridge consumption of published artifacts only through projection
  consumption

The milestone is still **Open** if any ordinary runtime/product path of the
same defect class survives, even when:

- the surviving path sits outside the originally curated scan inventory
- the surviving path is wrapped in a nominal type that still clones rows at mint
- certification reports success through hard-coded zeros or parsed receipt strings
- Phases 1â€“10 scaffolding is mistaken for end-cap closure
- support/profile declares `Closed` while residue scans or sabotage tests are red

The milestone is only **Closed** when all of the following hold together:

1. **Pinning (Phases 11â€“13):** registry authority, pin substrate, and shared-read
   context each closed with inventory slices, forbidden-pattern scans, hostile
   proof, and sabotage inside their owning phase; Phase 13 aggregates full
   pinning-boundary posture as `Closed`.
2. **Structural shared read honesty:** `WORTHQuerySharedReadContext` is `Send +
   Sync`, holds registry leases, denies stale basis typed, and never refreshes
   in place (Phase 13).
3. **Hot-path lock freedom:** committed reads take exact-zero locks on the
   measured hot path in certification builds (Phase 12).
4. **Journal identity (Phases 14â€“15):** typed journal position and consumer
   replay each closed with inventory slices and scans inside their owning
   phase; Phase 15 aggregates journal-boundary posture as `Closed`.
5. **Real concurrency (Phase 16):** N readers and M submitters with byte-
   identical equivalence to serialized replay; counters runtime-sourced;
   sabotage tests fail on regression â€” all inside Phase 16.
6. **Public-bridge honesty (Phase 17):** certification cannot compile with direct
   materialization-reading bypasses; sabotage fails on row-spelunking restore.
7. **Derived posture (Phase 18):** support/profile, docs, and test-requirements
   aggregate phase-local `Closed` postures; any defended exclusion names a
   different milestone-class owner.

In other words: this milestone does not merely add concurrent-looking APIs. It
removes the need, excuse, and surviving opportunity to treat copied snapshots as
pinning, strings as journal order, or serial schedules as concurrent proof.

## Must Ship

- authority-lane-decomposed backend contracts with `Send + Sync` read lanes
  (Phases 1â€“2)
- runtime-owned published-artifact registry authority with registry/mint
  inventory and scans (Phase 11)
- generation-indexed pinning with explicit retirement, lock-free committed-read
  hot path, pin/retire inventory extension, and runtime-owned residue counters
  (Phase 12)
- sealed basis-bound `Send + Sync` shared read contexts with full pinning-boundary
  closure â€” inventory completeness, hostile matrix, sabotage, derived posture
  (Phase 13)
- typed journal position identity with journal inventory seed and scans
  (Phase 14)
- consumer-facing journal-segment replay with journal-boundary closure
  (Phase 15)
- the published derived-artifact rule with reader evaluation structurally
  impossible (Phases 7â€“8, re-certified in Phases 13 and 17)
- the re-expressed workspace facade with unchanged existing consumer surface
  and fail-closed admission for the new families (Phase 9)
- real concurrent hostile certification matrix with runtime-owned counters and
  in-phase sabotage proof (Phase 16)
- public-bridge projection-consumption honesty (Phase 17)
- derived milestone closure posture and closeout doc (Phase 18)

## Must Preserve

- canonical query meaning across serialized and concurrent execution â€”
  timing may change, values may not
- lower-crate authority boundaries: relational owns truth, signal owns
  evaluation, the bridge owns causal routing; only access topology moves
- the single-owner workspace as a first-class consumer surface
- merged `9.4` temporal/async meaning and `9.5` projection-consumption
  semantics inside the published-artifact lane
- basis capability lifecycle as the only path to read authority

## Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- every Phase 10â€“18 certification suite added to
  [test-requirements.md](./test-requirements.md) passes with narrow
  machine-checkable artifacts
- Phase 13 pinning-boundary posture is `Closed`: inventory complete across
  Phases 11â€“13, forbidden-pattern scans green, hostile pinning matrix passed,
  sabotage tests fail when pin residue or lock counters are perturbed
- N concurrent reader threads and M concurrent submitter threads under write
  pressure produce byte-identical receipts and results to serialized execution,
  with exact-zero lock and reader-evaluation counters sourced from runtime
  measurement state (Phase 16, including in-phase sabotage)
- shared-read authority is backed by runtime-owned registry leases and
  generation pinning â€” not copied materialization snapshots â€” with typed
  stale-basis denial and exact-zero residue counters (Phases 11â€“13)
- Phase 15 journal-boundary posture is `Closed`: journal inventory complete,
  zero `commit_identity` parsing in runtime or certification crates, replay
  reconstructs identical truth, receipts, and published artifacts
- existing downstream consumers compile unchanged against the re-expressed
  facade
- the new facade families carry honest support/admission rows that fail
  closed where unbacked
- public-bridge hostile certification consumes published derived artifacts
  only through the typed projection-consumption lane (Phase 17)
- Phase 18 aggregates phase-local `Closed` postures; `milestone-9.7-closeout.md`
  cites Phase 13 pinning inventory digest, Phase 15 journal inventory digest,
  and Phase 16 certification artifact digests as primary evidence

## Sequencing Notes

- This milestone belongs after [milestone-9.6.md](./milestone-9.6.md) because
  journal identity, submission receipts, and published-artifact digests must
  be born on the canonical evidence-identity scheme.
- It belongs before Milestone `10` as a hard gate: store-backed execution
  must inherit lane-correct adapter contracts and the concurrency topology,
  not retrofit `Send` boundaries through frozen store-backed shapes.
- Durable journal persistence, store-backed replay reconstruction, and
  restart-stable published-artifact reload remain Milestone `10`/`11` scope.
- Phases 1â€“10 may land incrementally and may keep prototype debt explicit.
  Phases 11â€“18 are strictly sequential at the honesty layer:
  **11 â†’ 12 â†’ 13** (pinning substrate and proof close together; Phase 13 is
  the pinning hard gate), then **14 â†’ 15** (journal identity and proof close
  together; Phase 15 is the journal hard gate), then **16 â†’ 17** (concurrent
  certification with in-phase sabotage, then public-bridge honesty), then **18**
  (aggregated closeout only â€” no re-audit).
- Phase 13 is a hard gate: journal and certification phases may not claim
  closure while pinning-boundary posture is `Open` or `Partial`.
- Phase 15 is a hard gate: certification may not claim closure while journal-
  boundary posture is `Open` or `Partial`.
- If Milestone `9.6` journal-position typed artifacts are still incomplete,
  Phase 14 may require a small `9.6` follow-on slice before honest journal
  closure â€” but Phase 14 must not fall back to `commit_identity` parsing.
