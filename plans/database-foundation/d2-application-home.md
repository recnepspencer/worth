# D.2 Application Home

The design note for [D.2](roadmap.md#d2-application-home). The roadmap governs: one open call, the
home as a value, `memory()` on today's runtime, `at(path)` refused until D.9, no temporary backend.
`D/` is `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/`, `E/` is
`D/primary_graph/`, `W/` is `crates/worth-runtime-world/src/`, `R/` is `crates/worth-relational/src/`.

## 1. The open call

Two entries replace the eight `in_memory*` constructors, which are deleted. Both stay in the
`application_installation` facade module.

```rust
let app = application_installation::program(validated, declaration, configuration, limits)
    .roster(successors)                    // optional, program entry only
    .authorization_time_source(clock)      // optional
    .initial_state(seed)                   // optional; runs only when the home starts empty
    .adopt_on_open(adoption)               // optional, program entry only
    .open(ApplicationHome::memory())?;     // or ApplicationHome::at(path)

let app = application_installation::declaration(declaration, configuration, limits)
    .authorization_time_source(clock)
    .initial_state(seed)
    .open(home)?;
```

- **Entries.** `declaration(ApplicationSchemaDeclaration<Schema>, <Schema::Contributions as
  WorthQueryApplicationContributionTuple<Schema>>::Configuration, WorthQueryApplicationLimits) ->
  WorthQueryDeclarationOpen<'open, Schema>` replaces `in_memory`. `program(ValidatedApplicationProgram
  <Schema, Program>, ApplicationSchemaDeclaration<Schema>, <Program::Contributions as ..>::Configuration,
  WorthQueryApplicationLimits) -> WorthQueryProgramOpen<'open, Schema, Program>` replaces the other
  seven, with today's bounds. `'open` covers the roster borrow and the boxed closures. Required
  values are arguments, optional ones methods (DX 6, DX 8); limits have no principled default. No
  typestate: entry-only options exist only on `WorthQueryProgramOpen`. `open(self, ApplicationHome)
  -> Result<Runtime, WorthQueryApplicationOpenRefusal>` returns today's runtime types.
- **The refusal returns the home by phase.** `WorthQueryApplicationOpenRefusal { home:
  WorthQueryRefusedHome, denial: WorthQueryApplicationOpenDenial }`, where `WorthQueryRefusedHome` is
  - `Unchanged(ApplicationHome)`: refused before any effect; the original home;
  - `Successor(ApplicationHome)`: the adoption was performed and acknowledged, then a later step
    failed; reopening it resumes at the successor (today's `CheckpointTransitionAcknowledged`
    checkpoint, `E/application_installation/program/checkpoint_transition.rs:31-76`);
  - `InRepair(WorthQueryOpenAdoptionRecovery)`: settlement deferred or capture stopped; there is no
    home, and the capsule's `repair(self) -> Result<ApplicationHome, Self>` is the only path
    (`E/bootstrap/checkpoint_transition/repair.rs:76-115`).

  The capsule moves out of the denial: the `Adoption{Deferred, CaptureStopped}` arms carry only the
  cause, and `CheckpointTransitionAcknowledged` is deleted.
- **Entry mismatch.** Today a declaration entry resuming a program image skips activation recovery
  (`E/application_installation/mod.rs:208-227`). The image kind is derived, not stored: a program
  install seeds its activation record as its first Relational transaction, before any bootstrap row
  (`E/bootstrap/program_activation_seeding.rs`), and a declaration install never seeds one. So an
  image with an activation is a program image. Resuming through the other kind is refused with
  `EntryKindMismatch { image, entry: WorthQueryOpenEntryKind }` (`Declaration | Program`),
  `Unchanged`. No format field is added.
- **How the home starts:** `opening() -> WorthQueryHomeOpening` on the runtime:
  - `Started`: an empty home; `initial_state` ran; the installed revision is this program's;
  - `Resumed { installed: ApplicationProgramRevision }`: the image's activation names a revision in
    the admitted roster (the primary or a successor), as `recover_program_activation` accepts today
    (`E/bootstrap/program_activation_recovery.rs:14-44`, `tests/application_graph/adoption/
    rostered_restore.rs:14-61`); `installed` is that recorded revision;
  - `Adopted { from: WorthQueryOpenAdoptionPredecessor, installed }`: the activation names the
    adoption predecessor and the step ran.

  An activation in neither the roster nor the predecessor is refused, `Unchanged`. Seed and
  adoption are declared once and run only when they apply, so one open call serves every start.
- **Close.** `close(self) -> Result<ApplicationHome, WorthQueryApplicationCloseRefusal<Self>>` on
  both runtimes and on `WorthQueryWorkflowApplicationRuntime`
  (`E/application_installation/program/workflow_runtime.rs:22-46`), which closes its inner runtime;
  the workflow spec is installation truth and is retained again after each open (`:100`). Close
  refuses, handing the application back, when:
  - a publication is in flight or a settlement is unsettled (today's capture admission,
    `R/runtime/state/subsystems/canonical_publication_routes/checkpoint_admission.rs:31-72`);
  - `AdmissionsActive`: a handle operation is in flight through the lifecycle gate below.
- **Close fence: a revocation gate, not uniqueness.** The integration handle is a `Clone` bundle of
  Arcs (`E/root.rs:214-229`). The runtime holds clones internally
  (`product_world/source_owner.rs:37-39,120`, `application_runtime/installation.rs:63,118`);
  contribution patterns move binding-scope authority into the runtime
  (`conditional_contribution.rs:99-118`); `facade.rs:236` `retain_primary_graph_integration_handle`
  is public. A uniqueness check would refuse close forever. Instead one Query lifecycle gate:
  - `close` refuses while an admission is in flight;
  - a later admission gets `WorthQueryHandleDenial::Closed`, never a dead runtime;
  - an admission that arrives while close is deciding waits, then proceeds if close refused or
    gets `Closed` if it succeeded. No admission is ever denied by a close that was refused.

  The gate is the source owner's own mutex, not a second counter. The cell becomes a two-state
  value under that mutex, open with the runtime or closed. An admission is a lock that finds it
  open. `close` takes the lock without waiting (a held lock is the in-flight refusal), does every
  step that can refuse while holding it, and only then stores the closed state.
- **Close is one decision, in this order,** all under the source owner's lock:
  1. hold Relational admission (`try_hold_admission`, below); an admission in flight, an
     unsettled or in-flight publication, or an outstanding prepared commit refuses close;
  2. capture the closing image through the hold; a capture error refuses close;
  3. seal the hold, which cannot fail;
  4. store the closed state and build the home.

  Every refusal happens before the seal, releases the hold, and hands back a runtime that works.
  Nothing can change between the capture and the seal, because a held owner is quiescent. Sealing
  first would leave a sealed runtime behind a refused close; capturing first without the hold would
  let a carried Relational port publish between the capture and the seal, and the seal would then
  drain that settlement as owner loss.
- **Where the gate sits.** `WorthQueryRelationalSourceOwner` (`D/execution_runtime/product_world/
  source_owner.rs:21-28`) is the one cell holding `Arc<Mutex<RelationalRuntime>>`. Its
  `with_runtime`, `with_runtime_mut` and `with_runtime_mut_unwind_isolated` (`:76-107`) admit
  first and return `Result<_, WorthQueryHandleDenial>`. Everything built on the integration handle
  reads through them, so it is gated by construction. A standalone owner (`::new`, used by the
  ordinary Query runtime) gets a gate nobody can seal; only the application runtime holds the seal.
- **Raw escapes, closed.** Checked against what each caller needs:
  - `with_runtime` on the integration handle (`E/root.rs:309`) stays `#[doc(hidden)] pub`, now
    gated: the `worth-query` crate's backend reads through it in production
    (`worth-query/src/runtime/backend/primary_graph_runtime.rs:53-58`, `relational_owner.rs:20-21`,
    `bridge_backed/relational_execution.rs:150,164`), so `pub(crate)` would break a real user. Its
    closure can still carry Relational-minted handles (owner service ports, retention leases) out
    of the guard. So close also seals Relational's own owner lifecycle in place, in two typed
    steps on one lifecycle word that holds the state and the in-flight count (today only `Drop`
    closes, and `RelationalRuntimeOwnerBinding::close` drains and blocks,
    `owner_lifecycle.rs:61-79`):
    - `try_hold_admission` is one compare-and-swap from open-and-idle to held. If any admission is
      in flight it changes nothing and refuses with `AdmissionsActive`, which Query's `close`
      reports as its own in-flight refusal, so close never hangs on an admitted runtime carried
      out of a closure. Once held, it also refuses, and releases, when a publication is in flight
      or unsettled (the checkpoint admission's own condition, `checkpoint_admission.rs:31-72`) or
      a prepared commit candidate is outstanding (its `Drop` changes record-identity state the
      image reads, `mvcc/publication/candidate.rs:296`). A hold that is returned is therefore
      quiescent and stays so: while it lives, a new admission waits, neither admitted nor denied,
      and nothing the image reads can change. The hold gives only the reads a close needs, the
      native checkpoint and the branch names, never the runtime itself: an unadmitted mutator
      (a retention pass) would change the image after capture, and an owner-admitting call would
      wait on its own hold.
    - The hold then either seals or is released. `seal` cannot fail, and never drains a live
      settlement, because the hold proved there is none. Dropping the hold, including on unwind,
      releases it and the waiting admissions proceed.

    Both reach close authority through the owner tenure (`runtime_state/mod.rs:97-107`,
    `close_authority.rs:53-57`) via the `&mut` Query holds under its mutex. After a seal, every
    service that admits through Relational's owner binding denies with `OwnerUnavailable`
    (`R/branch/fork.rs:93-96`), even while a stray `Arc` keeps the runtime alive. `Drop` after a
    seal skips explicitly, including `publication.close`.
  - **Not every carried handle is denied.** Only handles that admit through the owner lifecycle
    (owner service ports, admitted runtimes) are. Snapshot and basis handles carried out are frozen
    reads of a published state, never writes; they keep reading after close.
  - `prepare_product_source` (`E/root.rs:380`) stays public and becomes **revocable**. The
    ordinary Query runtime composes over an application's graph in production
    (`granular_invalidation_installation()`, `E/application_runtime.rs:290`, then
    `worth-query/src/runtime/builder/construction.rs:42-46` and `installed_product.rs:71-77`). The
    token is opaque (`source_installation.rs:12-16`); it now carries the gate, and
    `WorthQueryProductRuntime` admits through it on every entry, so the bridge source and ports
    inside reach the runtime only through the gate.
  - `relational_bridge_source` (`E/root.rs:341`) is **restricted** behind
    `test-primary-graph-faults`. Outside the crate only certification uses it
    (`restored_primary_backend/adapters.rs:53`, `granular_invalidation/query_runtime_world/
    bridge.rs:64`, `source.rs:111`, `financial_runtime_world/query_source.rs:187`), and
    certification already enables that feature. Production keeps the in-crate path. A revocable
    Bridge source would need fallible reads across all of Bridge, a change out of scope for a
    test-only escape. Its owner-tenure mutations do not admit through the owner lifecycle
    (`worth-runtime-bridge/src/relational_source/runtime_source/mod.rs:73,89`), so a bridge source a
    test retains past close is unguarded; this is a stated test-only limitation, not a production
    path.
- **Gate coverage.** The integration handle and everything minted from it: invariant projection
  authority (`E/invariant_projection.rs:147-157`) and its snapshot
  (`WorthQueryApplicationInvariantProjectionSnapshot`, `:87-101`, exported at
  `facade/primary_graph.rs:77`; `field()` becomes `Result<Option<Value>, _>`); the binding scope
  (`E/conditional_operation/installation/application_binding_scope.rs:30`); the granular
  invalidation installation; the product source token and the product runtime built from it; the
  workflow runtime. Basis leases (`basis_registry/lease.rs:19`) are `pub(crate)` and gated through
  their graph handle.
- **Drop after the seal.** Release on drop (snapshot, basis lease, retention, basis) admits; on
  `Closed` it is a no-op and never panics. A release that reaches Relational after its in-place
  seal is likewise a no-op.
- **Adoption at open.** `WorthQueryOpenAdoption::new(predecessor, resources, author)` bundles what
  `in_memory_rostered_program_from_checkpoint_with_transition` takes. It stays separate from live
  branch adoption: the predecessor is a 64-hex rendering, never an admitted revision, and the step
  runs before any World exists. Renames:
  - `WorthQueryCheckpointMigrationWriter` -> `WorthQueryOpenAdoptionWriter`;
  - `WorthQueryCheckpointProgramPredecessor` -> `WorthQueryOpenAdoptionPredecessor`;
  - `WorthQueryCheckpointTransitionResources` -> `WorthQueryOpenAdoptionResources`;
  - `WorthQueryCheckpointTransitionRecovery` -> `WorthQueryOpenAdoptionRecovery`;
  - `CheckpointTransition{SettlementFailed, Deferred, CaptureStopped}` -> `Adoption{..}`.
- **Limits and profile.** `WorthQueryInMemoryApplicationLimits` -> `WorthQueryApplicationLimits`,
  `WorthQueryInMemoryApplicationProfile` -> `WorthQueryApplicationProfile`. Nothing in them is
  about memory; fields and methods are unchanged.
- **Retained authority after open.** `WorthQueryPrimaryGraphApplicationRuntime::
  retain_invariant_projection_authority(&self)` replaces the bootstrap accessor, so no caller keeps
  per-open authority inside `initial_state`, which a resumed home skips. Bank-server does exactly
  that today and then `expect`s it (`bank-server/src/identity_runtime/installation.rs:76-112`). The
  binding-scope accessor stays for contributions binding during open; both mint from one private
  constructor.
- **Typed phases inside open** (Arch 16). The core `in_memory_with_contributions` takes four
  `Option`s that permit illegal mixes, such as a transition without a checkpoint. The builder
  lowers into a private `OpenPlan` with `OpenEntry::{Declaration, Program(step)}` and
  `HomeStart::{Empty { initial_state }, Resume { image, adoption: Option<OpenAdoption> }}`.
- **Product-branch ordinal recovery.** `next_public_branch_ordinal` restarts at 1
  (`D/execution_runtime/product_world/runtime.rs:37,82`) and names Relational branches
  `query-product-{n}-relational` (`worth-query-execution/src/basis/product_branch/creation.rs:108-117`),
  so a fork after reopen collides with a restored branch (`DuplicateTarget`,
  `R/branch/fork.rs:24,154`). Live names alone are not enough: restore clears Relational's
  `retired_names` (`R/runtime/state/subsystems/history_recovery.rs:120`, `R/branch/registry.rs:11,137`),
  so after deleting the highest branch and reopening, its number would be reused and fork into the
  retired name would be admitted (`RetiredTarget` holds only within one process,
  `R/branch/fork.rs:25,156`). Fixed structurally in Relational:
  - the durable checkpoint carries `retired_names` (bounded by `maximum_names`,
    `registry.rs:130`) and restore restores them instead of clearing; the native checkpoint format
    bumps and older images are refused (disposable-store policy);
  - the resume phase sets the ordinal to one past the highest `n` across live and retired names; an
    id with that prefix that does not parse refuses the open; no separate high-water record;
  - the next ordinal past `u64::MAX` is `IdentityExhausted`, never a wrap.

  Regression test: fork twice, delete the highest, close, reopen, fork; the new name is fresh, and
  fork into the retired name is refused with `RetiredTarget` after reopen.

## 2. ApplicationHome

```rust
pub struct ApplicationHome(HomeForm);          // not Clone
enum HomeForm { Memory(Option<ClosedImage>), At(PathBuf) }
impl ApplicationHome {
    pub fn memory() -> Self;                   // empty memory home
    pub fn at(path: impl Into<PathBuf>) -> Self;
    pub fn capabilities(&self) -> WorthQueryHomeCapabilities;
}
```

- **Lives in** `E/application_home/`, exported through `application_installation`.
- **`memory()`** starts empty. `close` returns a memory home holding the closed image (today's
  `WorthQueryApplicationCheckpoint`, now private). `decode(self)` consumes its bytes
  (`E/application_checkpoint.rs:78,271-275`), so open decodes a copy and keeps the original for an
  `Unchanged` refusal: one extra image copy per resume, gone at D.10.
- **`at(path)`** is refused before any filesystem call with
  `WorthQueryApplicationOpenDenial::Home(WorthQueryHomeAbsent { form, deferral })`, `Unchanged`. The
  deferral is the same value the capability rows carry (section 4): one spelling.
- **Denial rename.** `WorthQueryInMemoryApplicationDenial` -> `WorthQueryApplicationOpenDenial`,
  now `#[non_exhaustive]`; Display drops "in-memory". Bank-server gate.
- **Not Clone.** Two runtimes from one image would be two diverging histories of one home.

## 3. Checkpoint restore until D.9

**Chosen: a memory home that closes and reopens inside one process.** Reopen becomes the home's
lifecycle, the same shape a durable home has.

- **Not restore behind `certification-test-authority`:** that feature exists in no Query crate,
  and a restore entry behind it would be a composition root unavailable to production (Testing 20)
  and a second open path.
- **Not an operator restore:** backup and restore belong with S.10 and S.11 on Store's format;
  binding them to today's image would change a public API at D.9.
- **Not a temporary backend:** it is `memory()` on today's runtime, which the roadmap sanctions
  until D.10. The codec is private; close and reopen are the API that D.7 and D.9 media keep.
- **The test seam.** Tampering, a fresh process (`tests/application_graph/restored_replay/
  cross_process.rs`) and opening one state twice (reuse on and off) need bytes. Under the existing
  `test-durability-faults`, `#[doc(hidden)]`: `ApplicationHome::image_bytes_for_durability_test(&self)
  -> Option<&[u8]>` and `memory_from_image_bytes_for_durability_test(Vec<u8>) -> Self`;
  `rewrite_authenticated_body_for_durability_test` moves onto the home. The seam builds a home
  value; `open` stays the single path. Enablers, none a production crate: worth-query-certification
  and consumer_root (normal dependencies of test crates); bank-server's `test-controls`, reached by
  its dev self-dependency, bank-http-adapter's dev-dependency and bank-courtroom's non-dev
  `cold-docker-courtroom` feature (a certification harness); topology_entry gains a forwarding
  feature for its reuse tests.
- **Public checkpoint surface removed:** `WorthQueryApplicationCheckpoint`, its section-bytes types
  and `capture_application_checkpoint*`; section inspection moves behind the seam.
- **At D.9:** `at(path)` opens on Store; the cross-process test moves to `at(path)` in a fresh
  process. **At D.10:** `memory()` runs on D.7 memory media and close drains; codec, capture and
  seam are deleted, and tamper coverage is Store's media corruption suite. The D.2 reopen tests run
  unchanged on memory media, which is D.10's parity acceptance.

## 4. Reopen inventory

"Expected" is the code reading of an in-process `memory()` close and reopen; the row's probe
(below) decides. World rebuilds only the root product branch on reopen (`W/branch/bootstrap.rs:43-58`),
so every record row covers the root branch only, and `ForkedBranches` covers the rest.

| Item | Owner and file | Expected | Durable |
|---|---|---|---|
| `GraphRecords` (root): records, versions, adjacency, aspects, schema images, identity allocator, symbols, lineage, index definitions | Relational, `R/durability/data/checkpoint_images.rs:283` | Resumed | D.9 |
| `ForkedBranches`: every non-root branch and its records | Relational cells restored, World unaware | Absent | D.10 |
| `BranchLifecycle`: Live, Archived, Deleting | the image carries no posture: restore gives Live (`R/branch/reference_checkpoint.rs:94`), or Deleting for a cell whose name is retired | Absent | D.9 |
| `PendingSettlements` | Relational settlement registry; close refuses until drained | Absent | D.9 |
| `IdempotencyRecords` (root), probed by retry | rows `E/schema_layout/provider_idempotency.rs`; retry answers `CommittedReceiptNotRetained` (`E/provider/idempotency/snapshot_resolution.rs:225`, `E/application_attempt/idempotency_resolution/denial.rs:30-34`) | Absent | D.10 |
| `DispatchOutbox` (root), probed by correlation | rows co-committed; correlation reads memory, `E/provider/committed_dispatch_outbox/correlation.rs:14-33` | Absent | D.10 |
| `InboundReceipts` (root), probed by terminal lookup | rows need the World `Performed` pairing, `E/provider/inbound_terminal_index.rs:1-5` | Absent | D.10 |
| `AftermathCausality` (root) | rows `E/schema_layout/provider_aftermath_causality.rs` | Resumed | D.9 |
| `WorkflowRecords` (root): definitions, instances, transitions, approvals | `E/workflow/schema/layout.rs` | Resumed | D.9 |
| `CapabilityGrants` (root): delegation, elevation status, review approvals | `D/authorization/decision_facts/authorization/delegation.rs`, `D/authorization/elevation_progression/approval.rs` | Resumed | D.9 |
| `ProgramRevisions`: revision per branch | main only, `E/bootstrap/program_activation_recovery.rs` | Absent | D.10 |
| `ProgramSupport`: Active, Retiring, Retired | `E/program_occurrence/support_lifecycle.rs:10-37` | Absent | D.10 |
| `AcceptedOutputs` and lineage facts | image body, `E/application_checkpoint.rs`, `E/application_installation/checkpoint_lineage.rs` | Resumed | D.10 |
| `CompletedEvidence`: completed-commit evidence and window | `E/provider.rs:106`, `E/provider/evidence_window.rs` | Absent | D.10 |
| `CommitReceipts`: receipt basis retention | `E/provider.rs:109` | Absent | D.10 |
| `UnpublishedIdempotency` | `E/provider.rs:107` | Absent | D.10 |
| `OutstandingDispatch` | `E/provider.rs:110` | Absent | D.10 |
| `InboundTerminalIndex` | `E/provider.rs:111`; rebuild needs World history | Absent | D.10 |
| `PendingPublications` | `E/provider.rs:112` | Absent | D.10 |
| `RecoveryHandles` | `D/managed_run/recovery_registry.rs:118` | Absent | D.10 |
| `InboundCustody` | `D/application_aftermath/external_effect/inbound/custody.rs:90` | Absent | D.10 |
| `TransportCompletionCustody` | `E/application_runtime.rs:153` | Absent | D.10 |
| `OutputDemandInterests` | `E/application_output_demand/registry.rs:316` | Absent | D.10 |
| `QueryContinuations` | caller-held, bound to a basis lease and instance, `E/application_query/continuation/` | Absent | D.10 |
| `ConditionalOperations` | `E/conditional_operation/lifecycle/registry.rs` | Absent | D.10 |
| `RetiredBranchNames` | Relational, `R/branch/registry.rs:11`; checkpointed from D.2.1 | Resumed | D.9 |
| `ProductBranchOrdinal` | `D/execution_runtime/product_world/runtime.rs:37`; recovered at open from live and retired names (section 1) | Resumed | D.10 |
| `RuntimeOrdinals`: dispatch attempt, producer attempt, mutation partition | `E/application_runtime.rs:141,168-169`, restart at 1 | Absent | D.10 |
| `ProductBranches`: registry, reference cells, generations | `W/branch/registry.rs:42-78`, `W/branch/reference_cell.rs` | Absent | D.10 |
| `CompositeHistory`: catalog, publication revisions, envelopes | `W/history/catalog.rs:81-102`, `W/history/publication.rs:37-46` | Absent | D.10 |
| `ComponentCustody` | `W/branch/custody/registry.rs` | Absent | D.10 |
| `UnpublishedRecovery`: ProductUnpublished catalog | `W/recovery/catalog.rs:43-61` | Absent | deleted in D.10 |
| `WorldIssuerCounters`: incarnation, commit, attempt | `W/identity/owner.rs:72-78`, restart at 0 per owner; the owner id is a process static (`:47`), so they repeat only across processes | Absent | D.10 |

**Not reopen state:** leases and pins (Relational pins, snapshots, open transactions; Query basis
leases, result buffers; World retention pins); in-flight attempts, which close refuses on; derived
state rebuilt at open (derived indexes, readiness, invalidation, source meanings); configuration
derived from the declaration (authorization registry, elevation rules, producers, routes);
transports, inbound verifiers and the authorization time source, which the host supplies at every
open.

**Capability rows.**

- `WorthQueryReopenPosture::{Resumed, Absent(WorthQueryReopenDeferral)}`, with
  `WorthQueryReopenDeferral { owner: WorthQueryStateOwner::{Relational, World, QueryHost},
  return_point: WorthQueryReturnPoint::{RelationalOnStore, DurableRuntimeState} }`.
- One exhaustive `match (form, item)` in `E/application_home/capabilities.rs` is the only row
  source; `WorthQueryReopenItem::ALL` is guarded by an exhaustive match in a test. `at(path)` is
  all Absent until D.9. A partially resumed item is Absent.
- **Extend, do not add.** Three single-arm postures are deleted and carry `WorthQueryReopenPosture`
  from the runtime's home instead: `WorthQueryRecoveryDurabilityPosture` and
  `WorthQueryDispatchOutboxDurabilityPosture` (`D/application_aftermath/recovery_posture.rs`,
  re-exported at `worth-query-execution/src/facade/primary_graph.rs:22,32`, snapshot
  `facades.toml:538,710`) and `WorthQueryPublishedRecoveryDurability`
  (`worth-query-publication/src/application_aftermath/recovery.rs:14-17`, mapped in
  `access_and_disclosure.rs`). `as_decision58_label` moves to the one posture.
- **Every row is tested by behavior.** A probe table maps each item to a probe through an exhaustive
  match, so a new item without a probe fails to compile. Each probe acts through the public path
  (retry by key, correlation lookup, terminal lookup, read on a fork), closes, reopens, and reports
  `Survived` or `Lost`; the test asserts `Resumed` iff `Survived`. A row cannot lie either way, and
  D.9 and D.10 flip rows only by making the probe pass. Expected values that the probe contradicts
  are corrected in the slice, not argued.
- **D.6 alignment.** Store's rows stay in Store's vocabulary; Query never imports Store. D.9's
  binding maps them onto these. D.6's first slice reads this inventory.

## 5. Slices

Each lands green with one implementer and one fresh reviewer. The root `Cargo.toml` excludes
`workspaces/*`, so every command names a manifest: QM `workspaces/worth-query/Cargo.toml`, FX
`.../worth-query-certification/fixtures/consumer_entry/Cargo.toml`, BM
`workspaces/worth-query-bank-world/Cargo.toml`, UM `workspaces/worth-ui/Cargo.toml`, BC
`tools/boundary-check/Cargo.toml` (config `tools/boundary-check/config/road1.toml`). A step over 5
minutes is split by target and filter first. The old constructors live from D.2.2 to D.2.8 only.

- **D.2.1 Relational: retired names and the lifecycle word.** Its own slice: another workspace, a
  native format bump, and the preconditions for ordinal recovery and close. The durable checkpoint
  carries `retired_names`; restore restores them; older images are refused. The owner's state and
  in-flight count live in one lifecycle word, so a stop and an admission can never each miss the
  other. In today's persisted local-file mode a retirement is durable only through a checkpoint;
  roadmap D.9 makes it a durable fact and deletes that mode. Tests: delete a branch, checkpoint,
  restore, fork into its name is `RetiredTarget`; the bound holds after restore; after a seal a
  retained owner service port is denied with `OwnerUnavailable`, a retained snapshot still reads,
  and a late snapshot release does not panic. Root workspace:
  `cargo test -p worth-relational --lib tests::branch`, then each new test by exact name.
- **D.2.2 Open call and renames.** Home, both builders, `OpenPlan`, phased refusal, entry mismatch,
  `opening` (on the program runtime; when a recorded revision is both rostered and the adoption
  predecessor, adoption wins), open-time adoption, the runtime authority accessor, the seam, the
  deferral type for `at(path)`. Every
  rename in sections 1 and 2 across all users (43 files), including
  `bank-server/src/identity_runtime.rs:12`, `src/error.rs:6,37`, `tests/identity_denials.rs:8,75`.
  The eight constructors delegate to the builder. It migrates `worth-query-host/tests/
  temporal_conditional_operation/contribution_installation/checkpoint_transition.rs`, which matches
  the deleted denial arms. Add a `worth-query-host::application_installation` exports entry to
  `facades.toml` (today it lists module names only, lines 93-134). Tests: QM `-p
  worth-query-execution --lib application_home`, `--lib primary_graph::tests`; QM `-p
  worth-query-host --test temporal_conditional_operation`; BM `-p bank-server`. Compile every
  renamed user: QM `-p worth-query --all-targets`, QM `-p worth-query-certification
  --all-targets`, FX `-p worth-query-consumer-root -p worth-query-topology-entry --all-targets
  --all-features`, UM `-p worth-ui-query-binding --all-targets` (`cargo check`). BC run.
  Bank-server gate.
- **D.2.3 Relational: admission hold, seal, and branch names.** `crates/worth-relational` only.
  `try_hold_admission` and the hold's `seal` replace the one-step `try_seal` (section 1): a hold
  proves quiescence or refuses, a held owner admits nothing and denies nothing, `seal` cannot
  fail, and a dropped hold releases. One public read of live and retired branch names.
  Tests: a hold with an admission in flight refuses and admission still works; an admission that
  arrives during a hold waits, then proceeds after a release and is denied `OwnerUnavailable` after
  a seal; a checkpoint is captured through a hold; a hold with a deferred settlement refuses, the
  settlement is still pending with no owner-loss release, and it settles afterward; a hold with a
  prepared candidate outstanding refuses and the candidate still publishes; a hold dropped
  on unwind releases; the D.2.1 seal tests, moved to the two steps; the name read returns live and
  retired names after a delete and after a restore. Root workspace:
  `cargo test -p worth-relational --lib tests::branch`, then each new test by exact name.
- **D.2.4 Close and ordinal recovery.** The source owner's cell becomes open-or-closed under its
  mutex and `with_runtime*` becomes fallible at every call site; `close` on the three runtimes,
  in the order of section 1, returning the home; the revocable product source token; the bridge
  source behind `test-primary-graph-faults`. Open recovers the product-branch ordinal from
  Relational's live and retired names; `u64::MAX` is `IdentityExhausted`. Tests: fork, delete the
  highest branch, close, reopen, fork again gets a fresh ordinal; a deferred settlement via
  `fail_next_durable_append_for_test` refuses close and the runtime still works; close refuses
  during an in-flight admission and while a prepared commit is outstanding; a retained integration handle is denied with `Closed` after
  close; a retained invariant projection snapshot's `field()` returns `Closed` after close and its
  drop is a no-op; a product runtime installed from the source token is denied after close; the
  workflow runtime closes and reopens. QM `-p worth-query-execution --lib application_home`,
  `--lib primary_graph::tests`; QM `-p worth-query-host --test temporal_conditional_operation`;
  the D.2.2 compile list again, because every `with_runtime*` caller changes. BC run.
- **D.2.5 Host and execution tests.** `worth-query-host/tests/temporal_conditional_operation/
  contribution_installation*`, `publication_limit.rs`; execution's
  `E/tests/restored_first_commit.rs`, `E/tests/retired_index_checkpoint.rs`,
  `E/tests/application_attempt/optional_output_role/indexed_selection.rs`, and the second restore
  path `E/tests/fixture/world_installation.rs:88-118`, which moves to `open(home)`. QM `-p
  worth-query-host --test temporal_conditional_operation`, QM `-p worth-query-execution --lib
  primary_graph::tests`.
- **D.2.6 Certification fixtures.** consumer_root (`application_invariant_acceptance/*`) and
  topology_entry (`src/checkpoint_recovery.rs` module root, `checkpoint_recovery/*`,
  `required_chain/*`, `reuse_opt_out*`). Its checkpoint modules are gated behind its four existing
  features, so FX `-p worth-query-consumer-root`, then FX `-p worth-query-topology-entry --features
  test-invalidation-equivalence,test-output-delivery-faults,test-query-execution-observer,
  test-world-operation-control,test-durability-faults`.
- **D.2.7 Certification tests and examples.** `tests/application_graph/` modules `adoption`,
  `document_retention_model/host`, `producer_predicate_checkpoint`, `restored_*`;
  `tests/program_example_authority.rs`; `examples/product_workflow_support/application.rs`. QM
  `-p worth-query-certification --test application_graph -- <module>` per module, `--test
  program_example_authority`, `--examples`.
- **D.2.8 Bank, UI, deletion.** `bank-server/src/identity_runtime/installation.rs` moves to the
  builder and the runtime authority accessor; `src/bank_projection/tests.rs`;
  `worth-ui-query-binding/src/product_projection/application_runtime.rs`. A bank reopen test (open,
  commit, close, reopen, read principals and invariants). Then delete the eight constructors and the
  public checkpoint surface. Application publication becomes an internal phase that only the
  builder can reach: the direct routes in `E/application_runtime/publication_entry.rs:35,63,88,117`
  and `E/conditional_operation/installation.rs:339` leave the public facade, and their fixture
  callers move to the builder. With the four public spellings gone, the open publication phase
  (`E/application_installation/open_core/publication.rs`) calls two internal routes, plain and
  conditional, that each take the optional time source. Refresh `facades.toml`. BM `-p bank-server`, UM `-p
  worth-ui-query-binding`, QM `-p worth-query-host`, QM `-p worth-query-execution --lib`, QM `-p
  worth-query-certification --all-targets` (check), FX both packages `--all-targets
  --all-features`, BC run. Bank-server gate.
- **D.2.9 Inventory and rows.** `WorthQueryReopenItem`, postures, `capabilities`, behavior probes
  for every row, the three postures folded, the ordinal audit (section 6). QM
  `-p worth-query-execution --lib reopen_inventory`, QM `-p worth-query-publication`, QM
  `-p worth-query-host`, BM `-p bank-server`; `facades.toml` refresh; bank-server gate.
- **D.2.10 Docs and reconciliation.** Section 7. QM `-p worth-query-host --doc`, QM
  `-p worth-query-certification --examples`.

## 6. Bug classes and contracts

- *A second installation path.* D.2.8 deletes the constructors; the `application_installation`
  snapshot entry pins its exports; the seam never builds a runtime.
- *A reopen that silently drops state, or a partial resume reported as resumed.* One exhaustive
  row source and a behavior probe per row (`Resumed` iff `Survived`); record rows are root-scoped,
  `ForkedBranches` is probed on a fork, `ProgramRevisions` on a non-main branch.
- *A failed open that loses the home.* `WorthQueryRefusedHome` names the home for each phase; the
  only homeless arm holds the linear `#[must_use]` repair capsule.
- *Two histories from one home; a closed runtime reached through a handle.* `ApplicationHome` is
  not Clone and `close` consumes the runtime. Handles are many and some are held inside the runtime,
  so one lifecycle gate sits at the single choke point, the source owner's `with_runtime*`, and
  everything built on it is gated by construction (multi-site): close seals it, a retained handle
  gets `Closed`, close refuses while an admission is in flight. The raw escapes are revocable
  (product source token), test-only (bridge source), or covered by Relational's non-blocking
  hold and seal (owner-admitting handles carried out of a `with_runtime` closure; carried snapshots
  stay frozen reads).
- *Seed or adoption on the wrong start; the wrong entry kind; per-open authority in a seed hook.*
  `HomeStart`, `opening()`, the typed entry mismatch, the runtime accessor; a test per arm and the
  bank reopen test.
- *An ordinal reused across reopen.* Ordinals are written into authoritative state: the
  product-branch ordinal names Relational branches, so it is recovered at open from live and
  retired names (checkpointed from D.2.1), exhausts as `IdentityExhausted` rather than wrapping,
  and has the delete-highest-then-reopen test. D.2.9 audits every other counter for the same (the
  mutation partition first); any that reaches authoritative state is recovered the same way in
  that slice, the rest are Absent rows.
- *`at(path)` doing real work.* Refused before any filesystem call; the test asserts the path does
  not exist afterward.
- *Test support creating authority.* The seam sits behind the existing `test-durability-faults`,
  whose enablers are all test or certification crates (section 3).

`worth-proof` is not needed beyond what exists: the adoption capsule is already linear, and the
other contracts are ownership, the lifecycle gate and exhaustive matches. No type guards a single site.

## 7. Docs

- `docs/api.md` §3.2 `application_installation` row (line 110): entries, `ApplicationHome`, close,
  capability rows.
- `docs/build-an-application.md`: §4, and lines 28, 39, 79, 760-765, 1493-1518.
- `docs/how-it-works.md`: every installation and checkpoint mention (about 12 lines).
- `workspaces/worth-query/crates/worth-query-host/README.md`; `worth-query/docs/AI_README.md`;
  `worth-query/docs/foundations/ordinary-application-front-door.md` and `programs-and-adoption.md`
  (adoption at open replaces checkpoint transition);
  `worth-query/docs/domain-capabilities/execution-resource-admission-and-managed-runs.md:182`.
- Certification examples compile in CI (DX 18).
- [Runtime Integration Roadmap](../worth-store/runtime-integration-roadmap.md): mark each milestone
  replaced or partly replaced, list what it keeps, and renumber return points. The slice checks each
  against its Must Ship list. Proposed:
  - 1 replaced by D.2 (`open_persistent`, `worth-application-store` become the open call and D.9's
    binding crate);
  - 2 partly, by D.8 and D.9; keeps the Signal, Bridge, package, stream and dispatch ports and the
    codecs;
  - 3 partly, by D.9; keeps the stage chain and the writer fence;
  - 4 partly, by D.5 and D.8; keeps arenas and prefetch;
  - 5 partly, by D.4 and D.9; keeps parallel admission and typed busy and rebase outcomes;
  - 6 partly, by D.8 and D.10; keeps drain and watermarks;
  - 7 partly, by D.9 and D.10; keeps the CDC cursors;
  - 8 partly, by D.1, D.9 and D.10; keeps PITR, replica, bootstrap and degraded outcomes;
  - 9 partly, by D.8 and D.9; keeps pushdown parity;
  - 10 partly, by D.10 (durable branches); keeps previews, diff and merge;
  - 11 to 19 remain, with return points after D.10.
- This roadmap's D.2 section gets its completed note.
