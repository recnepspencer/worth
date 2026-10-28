# Milestone 3 survival audit — Part A: runtime owners

Source baseline: `45ef6cec2a`. This is a source inspection, not a durability
certification. Governing requirements are Milestone 3 Phase 1, D6, D9, D10,
D13 and D15; the target-phase assignments below follow Phases 4–9.

## 1. Families

Paths in the tables use these exact repository-relative prefixes: **R** =
`crates/worth-relational/src/`, **W** = `crates/worth-runtime-world/src/`,
**S** = `crates/worth-signal/src/`, **B** =
`crates/worth-runtime-bridge/src/`, **J** = `crates/worth-signal-wasm/src/`,
and **E** = `crates/worth-execution/src/`. Thus `W/identity/owner.rs:56`
means `crates/worth-runtime-world/src/identity/owner.rs:56`. Each cited
declaration and the state described beside it were read.

Classes describe the required survival semantics, not what the current code
already guarantees. “In-memory” does not mean durable. A serializable snapshot
or a caller-held record is not an owner-backed durable publication. Phase
numbers for derived families identify when their reconstruction must work;
they do not request another authoritative journal. Process-local rows name
fresh admission instead of a persistence phase. Mixed current structures have
separate rows only for independently governed state within them, with the
boundary called out explicitly.

Milestone 2 already supplies canonical Relational commit, branch-transition,
record-allocation and checkpoint durability. See `milestone-2.md` D2, D4,
D7–D9 and D11, and R/branch/canonical_runtime_frame.rs:1. Those mechanisms
are not re-audited here. Existing row persistence below means persistence
through that route when the runtime has its durable medium; it does not
assert that today's Query application host opens such a medium.

### Relational

| Family | Owner type | Class | Current persistence | Process-local identities recorded | Target phase | Creating operations | Notes |
|---|---|---|---|---|---|---|---|
| AggregateIdentity | `AggregateRecord`, R/aggregate_identity/record.rs:39 | authoritative | Relational entity kind `worth-aggregate-identity`, aspect `aggregate-identity`; registration R/schema/aggregate_identity/storage.rs:23 | None in this grammar: identity/definition digests, birth commit, seed/current record coordinates and retirement commit | 4; continued original-owner use 9 | Commits whose relation integrity declares `AggregateIdentityContractDeclaration`; aggregate birth, split, merge and retirement consequences | Mint inputs are definition, commit and full native member key, not a runtime id (R/aggregate_identity/transition/mint.rs:7). Do not re-mint recovered aggregates. |
| AggregateMembership | `AggregateIdentityContractDeclaration`-governed relation rows, R/schema/aggregate_identity.rs:18 | authoritative | `worth-aggregate-membership-{membership_kind}`; endpoints member → aggregate; projected members additionally use `aggregate-projected-membership.cases` (R/aggregate_identity/projected_storage/mod.rs:13) | None in declared grammar | 4; advancement 9 | Same aggregate-integrity commits, including membership edits and split/merge consequences | Physical and projected membership have different logical-key rules; preserve the projected case list. |
| AggregateLineage | `LineageRecord`, R/aggregate_identity/record.rs:49 | authoritative | `worth-aggregate-lineage` relation; `aggregate-lineage` aspect has occurrence and kind; R/schema/aggregate_identity/storage.rs:36 | None: native `CommitId`, endpoints and split/merge tag | 4; correction integration 9 | Aggregate split/merge during consequence planning (`AggregateTransition`, R/aggregate_identity/transition.rs:36) | This is aggregate lineage, not Query's application causality grammar. |
| AggregateProjectedCoordinates | `AggregateRecord` projected coordinate tail, R/aggregate_identity/record/projected.rs:11 | authoritative | Same aggregate entity, optional aspect `aggregate-projected-coordinate`: domain, seed-case, current-case; R/aggregate_identity/projected_storage/mod.rs:12 | None in the serialized domain/case strings by this grammar | 4; advancement 9 | A projected aggregate source declared by `AggregateIdentityStorage.projected`; aggregate creation and anchor changes | A separately admitted optional schema extension, not an alternative aggregate identity authority. |
| RelationalPendingSettlement | `RelationalPublicationSettlementRegistry`, R/runtime/state/runtime_state/publication_settlement_registry.rs:36; record R/runtime/state/runtime_state/publication_settlement_registry/record.rs:62 | mandatory obligation | In-memory `Mutex<HashMap<CommitId, Arc<PendingRelationalPublicationSettlement>>>`; underlying canonical effect has M2 durability | Record `runtime_instance_id`; reserved/performed completion contains live visibility, control and retention handles | 4 | Any commit/fork publication that can perform before settlement; interruption, lost witness or settlement failure also creates outstanding custody | Persist/reconstruct the obligation and its association, not the mutex or old completion capability. `Reserved` before any effect is not itself a durable obligation. |
| RelationalSettledResult | `SettledRelationalSettlement`, R/runtime/state/runtime_state/publication_settlement_registry/record.rs:43 | derived | In-memory receipt, optional repaired `CommitResult`, published-snapshot closeout | Receipt and closeout carry live owner bindings | 4 | Immediate settlement and commit-identity repair | Rebuild result facts from canonical artifacts; reacquire closeout authority. The optional result copy is not durable retry authority. |
| RelationalRuntimeSequences | `RuntimeSequenceState`, R/runtime/state/subsystems/services/sequence.rs:10 | process-local | In-memory atomics for runtime, transaction and proposal ids | Runtime static `NEXT_RUNTIME_INSTANCE_ID` at :18; transaction/proposal counters start at one | Fresh `RelationalRuntime` owner admission; recover durable commit/record frontiers separately through M2 | Runtime construction, preparation, transaction creation | Canonical artifacts use a reserved neutral frame, not these live ids. Caller-written scalar fields are not automatically normalized by that frame. |
| RelationalSimulationArtifacts | `CompiledArtifactStore`, R/runtime/state/subsystems/services/compiled_artifacts.rs:20 | derived | In-memory compiled artifact cache, accessed through `RuntimeServices` at R/runtime/state/subsystems/services/mod.rs:52 | Runtime-local compiled artifact selectors | 4 | Compile/simulate an execution artifact | Recompile from installed schema and admitted intent; never restore a simulation handle as execution authority. |
| RelationalPublicationBundles | `PublishedBundles`, R/runtime/state/subsystems/publication.rs:16 | derived | In-memory latest bundle plus bounded retired `VecDeque` | Component receipts/replay views can be runtime-affine | 4 | Post-commit publication | Canonical commit remains the truth source; bundle replacement is not another durable commit. |
| RelationalDiagnosticArtifacts | `RelationalDiagnosticArtifactStore`, R/runtime/state/subsystems/publication_diagnostics.rs:14 | optional retained | In-memory artifact vector and operation capture maps | Capture id is an `Arc` address cast to `u64` (:45); capture routing uses `ThreadId` | 4; retention policy decision in section 4 | Diagnostic emission during both successful and refused operations | Retained richness is optional; active captures are process-local and must be restarted, never restored by address. No expiry policy is inferred from a vector. |
| RelationalDerivedIndexes | `IndexingSubsystem` / `IndexingState`, R/runtime/state/subsystems/indexing.rs:40, :130 | derived | Runtime-owned index generations; optional deferred checkpoint image | Live snapshot/generation admission is runtime-bound | 4; changed-build stale handling 6 | Index declarations, commit maintenance, demand/recovery | M2 checkpoint materialization is an accelerator; it must not replace row/source authority. |

### Runtime World

| Family | Owner type | Class | Current persistence | Process-local identities recorded | Target phase | Creating operations | Notes |
|---|---|---|---|---|---|---|---|
| WorldIdentityFrontiers | `RuntimeWorldIdentityIssuer`, W/identity/owner.rs:56 | authoritative | In-memory owner plus five checked issuer cursors | Owner is process-static `NEXT_OWNER_IDENTITY` (:47); all emitted identities embed it | 4 | Owner construction; bootstrap, branch incarnation, composite commit, publication attempt, unpublished custody issuance | D6 requires home identity and recovered durable maxima; branch reference generation is carried by its cell, not an issuer field. |
| WorldProductBranches | `ProductBranchRegistry`, W/branch/registry.rs:72 | authoritative | In-memory installed entries, name reservations, lifecycle map and root selector | World owner, branch incarnation; cells contain component basis admissions | 4 | Bootstrap; `ProductBranchCreationIntent`, fork plans, retirement and name reuse | Reservations/locks are fresh process resources. Persist branch lifecycle facts independently of live references. |
| WorldProductHeads | `ProductBranchReferenceCell`, W/branch/reference_cell.rs:60 | authoritative | In-memory reference image under per-branch synchronization | World owner/branch/incarnation/generation and runtime-affine composite basis | 4 | Bootstrap, branch creation and successful product CAS | Component effects and product movement are distinct. `ProductBranchReferenceSnapshot` is evidence, not publication authority. |
| WorldCompositeHistory | `CompositeHistoryCatalog`, W/history/catalog.rs:91 | authoritative | In-memory commit-keyed `HashMap<..., Arc<OnceLock<...>>>`, root and spliced-parent entries (:96) | Composite owner/commit/attempt ids and component admission identities | 4 | Bootstrap, branch creation, ordinary composite publication | Persist occurrence meaning and parentage. Rebuilt indexes and live pins have separate rows below. |
| WorldPerformedPublication | `CanonicalPublicationEnvelope`, W/history/publication.rs:37 | authoritative | In-memory `OnceLock<PerformedPublicationFacts>` with committed/delivery atomics; catalog entry W/history/catalog/entry.rs:7 owns it | Commit/attempt ids; expected/successor observations bind live owners | 4 | Successful composite publication, including response loss after movement | Durable occurrence must supply movement and exact component facts. Delivery claim atomics and cached observation are process-local, not receipt truth. |
| WorldAttemptProgress | `ActiveAttemptRecord` / `ActiveAttemptState`, W/publication/custody/record.rs:16, :26 | mandatory obligation | In-memory progress, successor, resources, abandonment and cause; `CompositeAttemptProgress`, W/publication/progress.rs:236 | Attempt/custody ids and owner-issued performed/settlement/fork evidence | 4; Signal component 5 | Composite operations capable of owner effects; cancellation, deadline, unwind, failed observation and lost CAS after effect | Before effect, reservations alone need not survive. After effect, progress must reconcile to occurrence or custody through D5, without rerunning an owner. There is no separate durable progress catalog today. |
| WorldUnpublishedCustody | `ProductUnpublishedRecoveryCatalog`, W/recovery/catalog.rs:60; `ProductUnpublishedOwnerEffectsRecord`, W/recovery/product_unpublished.rs:43 | mandatory obligation | In-memory catalog slots owning `Arc` records independently of returned caller view | World custody/attempt/branch/incarnation; `catalog_affinity: usize` at :59; live component pins and bases | 4; mixed/conditional definitions 5 | Owner effect followed by publication/settlement failure, unresolved durable fate, late cancellation/deadline, caller abandonment; branch-creation partial effects | `ProductUnpublishedRecoveryHandle` also records catalog affinity (W/recovery/product_unpublished/handle.rs:6). Recovery must issue a fresh claim; release/continue dispositions themselves are authoritative under D10. |
| WorldComponentBranchCustody | `OwnerCreatedComponentCustodyRegistry`, W/branch/custody/registry.rs:20 | mandatory obligation | In-memory `Vec<OwnerCreatedComponentCustodyRecord>` plus reservation accounting | Product branch identity/incarnation and component branch identities | 4; Signal forks 5 | Branch creation with owner-native fork plans, including a fork performed before another owner or product installation fails | Native branch custody is not interchangeable with product-head ownership. Unresolved M2 forks must retain the original fate. |
| WorldConditionalDefinitionCustody | `ConditionalDefinitionAttemptCustody`, W/publication/custody/conditional_definition.rs:14 | mandatory obligation | In-memory prepared Bridge extension, admitted Signal publication, advance binding and applied extension in separate mutexes | Signal/Bridge installation and branch-bound capabilities | 5 | Runtime conditional-definition publication; failure between preparation, owner application, activation and product movement | Live capability values cannot be the durable representation of already-applied definitions. |
| WorldHistoryReachability | `HistoryReachabilityIndex`, W/history/catalog/reachability.rs:45 | derived | In-memory commit-keyed dependency/protection counts | World commit ids; live direct/product-head protections | 4 | History admission, head protection, traversal, retirement | Rebuild from recovered history, heads and obligations, then admit new reader protections. |
| WorldComponentPins | `RuntimeWorldRetentionOwner`, W/retention/registry/owner.rs:121 | derived | In-memory retained component pins, dependency counts, single-flight records and observation capacity | Owner, lease ordinals and weak component service affinity (:91) | 4; Signal evidence 5 | History/observation retention and unpublished custody | Reconstruct required retention from durable history/custody and reacquire native leases. Reader-only reservations are process-local. Do not serialize `Condvar`/weak pointers. |
| WorldBootstrapAdmission | `RuntimeWorldBootstrapState`, W/lifecycle/owner.rs:45; `RuntimeWorldBootstrapIntent`, W/branch/bootstrap.rs:12 | process-local | In-memory Unperformed/InProgress/Performed gate; intent carries admitted component bases | Fresh bootstrap attempt plus live component/World owner identity | Fresh owner constructs from recovered artifacts; do not bootstrap an existing durable home as a new root | Owner construction, fresh or recovered bootstrap | Durable root occurrence belongs to `WorldCompositeHistory`. Current constructor resets gate at W/lifecycle/owner.rs:173. |
| WorldRecoveredRootAuthority | `RecoveredRuntimeWorldRootAuthority`, W/branch/bootstrap.rs:174 | process-local | None outside returned linear value | Bootstrap attempt and admitted composite/product branch | Fresh Relational recovery proof → `RuntimeWorldBootstrapIntent::recovered` (:43) → performed bootstrap → `into_recovered_root_authority` (:147); Phase 4 must bind recovered occurrence | Derived-output adoption at a proven recovered root | Explicitly non-serializable; checkpoint presence alone does not mint it. Current recovered intent starts generation at initial (:55), not the old World head. |
| WorldOperationAdmission | `RuntimeWorldOperationLedger`, W/lifecycle/owner/operation/attempt.rs:29 | process-local | In-memory active/recovery counts and per-attempt phase mutex | Live operation reservation affinity | Fresh operation admission under new incarnation and host close policy | Every owner service operation and close/recovery admission | Accepted effect obligations are the custody families, never inferred from restoring an active count. |
| WorldPublicationCapacity | `RuntimeWorldPublicationCapacityLedger`, W/lifecycle/owner/operation/publication_capacity.rs:12 | process-local | In-memory bounded active-attempt reservations | Live reservation ownership | Fresh budget admission against recovered obligations | Composite publication preparation | Capacity must be recharged for surviving custody; old reservation values grant nothing. |
| WorldHostServices | `RuntimeWorldOwnerState`, W/lifecycle/owner.rs:76, fields :86–106 | process-local | In-memory component ports, installed execution, clock, close contract and liveness | Weak owner bindings, monotonic clock/deadline context | Host supplies trusted clock/resources/execution and readmitted owner ports, then new incarnation-scoped service admission | Host composition and owner open/close | This row covers assembly resources only, not history/branches/custody listed separately. |

### Signal and Signal WASM

| Family | Owner type | Class | Current persistence | Process-local identities recorded | Target phase | Creating operations | Notes |
|---|---|---|---|---|---|---|---|
| SignalNodeDefinitions | `NodeDefinitionData`, S/data/node/entry/definition.rs:13 | authoritative | In-memory arena definition lane; also serialized in `CheckpointNodeImage` (S/data/node/checkpoint_image.rs:43) | `conditional_contract_occurrence` is minted from process-static `NEXT_CONDITIONAL_CONTRACT_OCCURRENCE` (S/data/conditional_execution/contract.rs:4) | 5 | Node construction, install/update conditional contract, tombstone, definition-publication extension | Evaluation configuration is authored; cached values/versions/causes are not. Snapshot serde is not the Phase 5 owner codec. |
| SignalTopology | `SignalGraph`, S/data/graph/runtime/graph.rs:189, arena/topology at :194 | authoritative | In-memory arena allocation/generation and topology; checkpoint arena/topology S/state/authority.rs:20, :27 | Graph `instance_id` is deliberately skipped by serde (:192); node slot generations remain in checkpoint | 5 | Graph declaration, node/edge mutation, installed/runtime-published definitions and branch fork | Preserve authored topology and node incarnations; distinguish authored edges from dependency caches discovered during evaluation. Current aggregate topology representation needs the split noted in section 4. |
| SignalBranchCells | `SignalBranchCellState`, S/branch/owner_services/branch_cell_state.rs:25 | authoritative | In-memory per-branch `BranchState`, definition basis, head generation and restore snapshot; registry S/branch/owner_services/branch_registry.rs:63 | `owner_runtime_instance_id`, branch handle/id, cell incarnation in owner service binding | 5 | Branch creation/fork, advance, restore, definition activation and retirement | Recover every branch, not just the selected branch. `BranchState` at S/logic/transaction/runtime/state/branching/branches/authority.rs:141 explicitly contains both authority and derived state. |
| SignalOwnerFrontiers | `SignalOwner`, S/branch/owner_services/owner.rs:50 | authoritative | In-memory next branch id; metadata's next snapshot id at S/logic/transaction/runtime/state/branching/branches/owner_partition.rs:54 | Runtime id/definition basis; global cell incarnation issuer S/branch/owner_services/cell_incarnation.rs:4 | 5 | Owner partition/sealing, branches, snapshots, basis admissions and conditional occurrence issuance | Persist home-scoped frontiers; retain only lifecycle/resource guards as process-local. Graph id issuer at S/data/graph/runtime/graph.rs:302 remains process-local. |
| SignalBranchMetadata | `SignalOwnerMetadataState`, S/logic/transaction/runtime/state/branching/branches/owner_partition.rs:40 | authoritative | In-memory children-by-parent, retirement receipts and snapshot identities; live wrapper S/branch/owner_services/owner_metadata.rs:29 | Branch/snapshot ids and wrapper runtime/lifecycle ids | 5 | Fork ancestry, branch retirement, snapshot creation | Retained snapshot payload is a different family below; active merge/retirement reservations are fresh resources. Do not serialize the wrapper's lifecycle identity. |
| SignalEvaluationState | `SignalGraph` evaluation lanes, S/data/graph/runtime/graph.rs:194; `CheckpointNodeImage`, S/data/node/checkpoint_image.rs:43 | derived | In-memory node state, dirty aspects, dependency/aspect versions, pending causes and traces; captured in current checkpoint image | Node-local dependency/cause selectors; graph instance itself is omitted | 5; changed-build checks 6 | Demand evaluation, invalidation delivery, conditional evaluation, replay/restore | New advances must not carry derived values. Destroying these must leave authored definitions sufficient for reconstruction. |
| SignalCheckpointRuntime | `DerivedState`, S/logic/transaction/runtime/state/reconstructability/authority.rs:36, `checkpoint` at :41 | derived | In-memory `CheckpointRuntime`, telemetry and reconstruction state | Current branch/node/replay selectors | 5 | Evaluation, checkpoint/journal capture, replay/restore | Existing type name `AuthorityState` (:9) includes a stateful graph; it is not the specification's narrower authored-image classification. |
| SignalSnapshotPayload | `SignalSnapshotV1`, S/state/snapshot.rs:17 | optional retained | Caller-visible serde snapshot containing authority image, diagnostic graph, telemetry, diagnostics and reconstructability | Bare runtime-local branch/snapshot ids in `SignalSnapshotMeta` (S/state/lifecycle.rs:41); serialized conditional occurrence in nested node image | 5 | Explicit snapshot/capture, branch snapshot and replay retention | Snapshot identity/branch ancestry are authoritative separately. Optional diagnostic retention is explicit at S/state/lifecycle.rs:62. This payload is not a second durable publication lane. |
| SignalTemporalRuntime | `TemporalRuntimeState`, S/logic/transaction/runtime/state/temporal.rs:21; conditional cells S/branch/owner_services/conditional_execution/temporal/partition.rs:55 | derived | In-memory scheduled/ready/retired wake maps, clock basis, frontiers and previous-value capability epoch | Wake ids/ordinals; conditional partition owner runtime/lifecycle at :56 | 5; conditional/temporal reconstruction 7 | Clock observations, temporal scheduling, previous-value and retry/timeout scheduling | Wakes are reconstructed from durable intent and host time. This does not authorize dropping the managed request/intent obligation that caused a wake. |
| SignalResourceLifecycle | `ResourceRuntimeState`, S/logic/transaction/runtime/state/resource/state.rs:16 | derived | In-memory descriptors, lifecycle/retry indexes and histories | Request/descriptor/generation/lifecycle ordinals and restore epoch | 5; managed request integration 7 | Resource declaration, request/completion, timeout, cancellation, retry, restoration | Physical runtime resources are reacquired. Mandatory in-flight custody has its own row; optional history/omission policy must not erase it. |
| SignalOwnedAsyncCustody | `SignalOwnedAsyncManagedAdmission`, S/branch/owner_services/conditional_execution/owned_async/managed_request.rs:32 | mandatory obligation | Caller/controller-held native occurrence, in-flight request, queue binding, retention, retry/predecessor and cleanup | Origin carries live basis, authority and cell incarnation (S/branch/owner_services/conditional_execution/owned_async.rs:56) | 7 | `admit_managed_owned_async_request`; initial admission, replacement/coalescing, retry and timeout; `CleanupRequired` branch (:102) | Each request resumes or takes its contract-defined typed terminal posture. Loss of the carrier is not cancellation or success. Process-local-release contracts remain distinct. |
| SignalOwnedAsyncQueue | `SignalOwnedAsyncManagedQueueBinding`, S/branch/owner_services/conditional_execution/owned_async/managed_queue.rs:8 | mandatory obligation | Native resource queue binding carried with managed admission | Live source/basis and resource attempt/request | 7; actual queue/resource is freshly admitted | Managed queue request, enqueue/drain and replacement; queue cleanup can fail | Persist obligation/progress, not physical queue or old `ResourceManagedQueueBinding` authority. |
| SignalBasisRegistry | `SignalBranchBasisRegistry`, S/branch/basis_registry.rs:51 | process-local | Weak exact-basis entries and single-flight reservation ids in `HashMap` (:31) | Runtime/cell-bound observations, registration/reservation ids | Readmit recovered branch descriptors under fresh Signal owner and reacquire leases | Basis admission, retention acquisition, exact canonicalization | Weak registrations are not authored definitions or durable occurrence identity. |
| SignalOwnerLifecycle | `SignalOwnerLifecycleState`, S/branch/owner_services/lifecycle_state.rs:66 | process-local | Atomics, admission table, transition mutex/condvar and cleanup guards | Static lifecycle issuer at :22; live owner runtime id | Fresh owner lifecycle and admission table; new incarnation fences old handles | Owner seal/open/close, service operation admission | Recover obligations first; never restore counts/locks to imply an operation is running. |
| WasmSourceTruth | `RuntimeStore.sources`, `StoredSource`, J/runtime/core/state.rs:59, :218 | authoritative | In-memory string-keyed source map; `RuntimeStoreSnapshot.sources` serializes value and versions (J/runtime/summaries.rs:255) | Source names are semantic; versions are evaluation metadata, not owner identity | 5 | Source declaration/initial values and source edits through `RuntimeCore` | Source values live outside native `SignalGraph`. Persist the value as source truth; rebuild versions under reuse rules. |
| WasmRecipeDefinitions | `StoredRecipe` / `StoredRecipeDefinition`, J/runtime/core/state.rs:65, :82 | authoritative | In-memory recipes, source/recipe families and keyed/dense declarations; portable definitions export J/runtime/core/envelopes.rs:117 | Callback definitions contain realm token slot/generation (:88), unlike expression specs | 5; callbacks require fresh host attachment | Recipe/family declaration, portable definition publication and callback registration | Cached value/version/initialized are derived. `RuntimeStoreSnapshot` alone omits expression definitions; full export includes definitions separately. |
| WasmBranchCatalog | `BranchRuntimeState`, J/runtime/core/state.rs:195; `RuntimeCore`, J/runtime/core.rs:74 | authoritative | In-memory branch metadata/catalog/node-name/dense-grid mappings, store snapshot and authored graph generation | Maps keyed by runtime-local branch/snapshot ids | 5 | Branch commands, fork, snapshot, restore and definition publication | Preserve authored branch metadata; rebuild reverse lookup indexes. Current exact root restore explicitly excludes sibling/history transfer (J/runtime/core/envelopes.rs:32). |
| WasmDerivedValues | `StoredRecipe`, J/runtime/core/state.rs:65, and `RuntimeCore` caches J/runtime/core.rs:74 | derived | Cached recipe values/versions, callback diagnostic map, runtime snapshots, admitted snapshots and metrics | Native node ids, snapshot keys, live admitted snapshot authority | 5; changed build 6 | Run/evaluate, read/serialize, snapshot and callback observation | Readmit references and recompute lazily; do not promote a stored computed value into a lost recipe. |
| WasmComputeCallbacks | `ComputeCallbackRegistry`, J/runtime/compute_callbacks/registry.rs:31 | process-local | Thread-local token/generation registry of JS/native closures | Realm-local slot/generation (`StoredCallbackRecipeSnapshot`, J/runtime/summaries.rs:290) | Fresh host registers/reattaches callbacks under explicit unavailable-callback admission | Callback registration, invocation and disposal | Portable export emits `UnavailableCallbackArtifact` rather than pretending to serialize a closure (J/runtime/core/envelopes.rs:132). |
| WasmObservationCallbacks | `WebCallbackRegistry`, J/runtime/web_callbacks.rs:68 | process-local | Thread-local per-runtime scope map, pending callback deque | Runtime scope, observer/handle and token slot/generation ids | Fresh runtime callback scope and explicit subscriptions | Watch/effect registration and queued delivery | Dying callback delivery does not define a durable application effect. Any external custody is owned outside this queue. |
| WasmDiagnosticCallbacks | `DiagnosticsCallbackRegistry`, J/runtime/diagnostics_callbacks.rs:32 | process-local | Thread-local scope map, callbacks and pending deque | Scope and token slot/generation | Fresh diagnostics subscription/callback registration | Diagnostic callback registration/delivery/disposal | Reacquisition does not restore the old JS realm. |
| WasmRestoreTokens | `RestoreTokenRegistry`, J/boundary/restore_tokens.rs:23 | process-local | Thread-local bounded map of exact/runtime/snapshot artifacts; evicts oldest (:63) | Realm-local incrementing token | Fresh artifact import/readmission and new token; no redemption across processes | Snapshot/export APIs and worker mutation publication | A token string is not durable transport or durable authority. |
| WasmWorkerShell | `WorkerRuntimeShell`, J/runtime/worker_host/worker_runtime_shell.rs:31 | process-local | In-memory host-boundary sequence, latest reports/requests/acknowledgements and subscriptions | Boundary sequence and subscription ids; live `RuntimeCore` | Fresh worker construction, host capability ingress, subscriptions and callback reattachment | Worker bootstrap, host effects/callback requests, output/observation delivery | `WorkerRuntimeIdentity` (J/runtime/worker_host/worker_runtime_identity.rs:5) is descriptive deployment strings, not a unique issuer. Host effect custody cannot be inferred from “latest report.” |

Additional independently owned Signal structures:

| Family | Owner type | Class | Current persistence | Process-local identities recorded | Target phase | Creating operations | Notes |
|---|---|---|---|---|---|---|---|
| SignalRuntimeDefinitions | `SignalRuntimeConfig`, S/logic/transaction/runtime/config.rs:22 | authoritative | In-memory node metadata, tier/comparator policies, computation registrations, keyed-node assignments and key registry | Interned runtime string ids and node handles | 5 | Computation/tier registration, keyed node declaration, runtime definition publication | The same current struct also has a memo cache; only definitions/assignments are authoritative. Native node snapshots alone do not preserve this config. |
| SignalMemoCache | `SignalRuntimeConfig.memo_cache`, S/logic/transaction/runtime/config.rs:35 | derived | In-memory interned-key to `NodeEvaluationResult` map | Runtime key registry ids | 5; changed build 6 | Memoized computation evaluation | Rebuild under current definitions/equivalence contract; do not serialize the memo result as source. |
| SignalBranchRetention | `SignalBranchRetentionRegistry`, S/branch/retention/registry.rs:61 | derived | Owner/cleanup ledger with admitted/external leases, target and branch counts (:85) | Runtime id, lease ids, canonical target encoding bound to live owner | 5 | Basis/history pin acquisition and release | Reconstruct required pins from durable owner associations; fresh reader leases are process-local. |
| SignalConditionalRetention | `SignalConditionalRetentionLedger`, S/data/retained_storage/conditional_retention/ledger.rs:17 | derived | Mutex usage for evaluation slots/bytes, temporal partitions/wakes | Arc reservation ownership | 5; obligation recharging 7 | Conditional evaluation, retained evidence and temporal partition admission | Rebuild usage from recovered definitions/custody and fresh resource reservations. |
| SignalObservationRegistrations | `RuntimeObservationRegistry`, S/logic/transaction/runtime/state/runtime_observation/registry.rs:28 | process-local | Listener registrations, node reverse index, observer/handle cursors | Observer and handle ids; live listener closures | Fresh observation registration after owner readmission | Watch/effect subscription, listener delivery/disposal | Durable active-demand obligations, if a caller declares them, must recreate registrations through its own custody. |
| SignalEventSubscribers | `EventBus`, S/logic/events/runtime.rs:27 | process-local | Pending events, subscriber callbacks/order/context and transactional readiness vectors | Live subscriber context/callback identity | Fresh subscriber installation; replay only through the owning admitted contract | Event emission, checkpoint flush, subscriber execution | This local queue is not a durable workflow/outbox. A contract promising effects across failure must have an explicit obligation owner. |

### Bridge

| Family | Owner type | Class | Current persistence | Process-local identities recorded | Target phase | Creating operations | Notes |
|---|---|---|---|---|---|---|---|
| BridgeConditionalDefinitions | `BridgeInstalledConditionalLowering`, B/conditional_execution/installed_lowering.rs:30 | authoritative | In-memory contract/location/projection, semantic correspondence and installed Signal contract; runtime map B/conditional_execution/contract.rs:46 | Bridge runtime key; Signal graph, branch admission and installation/cell-bound ports | 5 | Installed conditional declarations and runtime conditional-definition extension publication | Persist authored definition/claims in Bridge's Signal transition section; rebuild provider/port authority, counters and lowering realization. |
| BridgeLoweringRegistry | `BridgeConditionalLoweringRegistry`, B/conditional_execution/lowering_registry.rs:21 | derived | In-memory `BTreeMap` slots, exact-basis index and reservations | Exact live Signal basis/node keys and lowering authority | 5 | Conditional lowering install, exact-basis readmission and publication reservations | Explicitly derived in Phase 5; not another definition journal. |
| BridgeConditionalTargets | `BridgeOwnedConditionalTargetIndex`, B/conditional_execution/owned_target_index.rs:12 | derived | In-memory dependency → target reference-count buckets | Installed correspondence targets bind live Signal graph | 5 | Conditional definition install/extension and retirement | Rebuild counts from recovered definitions. |
| BridgeSemanticRegistrations | `AdmittedSemanticDependencyRegistry`, B/correspondence/semantic_dependency_registry.rs:12 | authoritative | In-memory authoritative registration vector; accompanying indexes | Optional `signal_graph_instance_id` (:17), installed binding identity/generation | 5 | Bridge installation and runtime semantic dependency extension | Preserve authored registration/claim meaning; binding-currentness index is derived (next row). Program-installed declarations can be readmitted from the program; runtime extensions need artifacts. |
| BridgeBindingCurrentness | `InstalledBindingCurrentnessIndex`, B/correspondence/semantic_dependency_registry.rs:27 | derived | In-memory `HashMap` of installed bindings to generations | Live binding keys | 5 | Registration install/rebuild (`rebuilt_from`, :41) | Cannot survive as an independent authority without registration parity. |
| BridgeCorrespondenceAllocation | `CorrespondenceAllocationRegistry`, B/correspondence/slot_allocation.rs:62 | authoritative | In-memory authoritative allocation records, owner sets and retained target→aspect assignments | `AllocationKey.signal_graph_instance_id` (:14) and partition/node/aspect | 5 | Declared slot allocation and semantic correspondence extension | Preserve allocation meaning/selected authored slots; reconstruct indexes and bind to the fresh graph. Despite shape, this is not merely the derived target-count index. |
| BridgeRetentionLedger | `BridgeRetentionLedger`, B/conditional_execution/retention/ledger.rs:20 | derived | In-memory checked clocks/intents/definition-candidates/bytes usage | Arc reservation ownership and live closed flag | 5; managed obligation charges 7 | Definition candidates, managed clock/intents, observation retention | Rebuild/recharge from recovered definitions and obligations, not serialized reservation guards. |
| BridgeManagedTemporalIntents | `BridgeManagedTemporalIntentRecord`, B/conditional_execution/managed_time/clock_lane.rs:13 | mandatory obligation | In-memory clock-lane `intents` map (:38), idempotency identity, revision, due coordinate, source identity and observation baselines | Native wake id plus live source/baseline bindings | 7 | `reconcile_managed_temporal_intent`, B/conditional_execution/managed_time/intent_reconciliation.rs:8; active/revised intent before terminal disposition | Intent meaning must survive; eligible/deferred/suppressed wakes and delivery indexes are derived/process-local. Idempotency string is provided by caller here, not derived by this method. |
| BridgeManagedClock | `BridgeManagedClockLane`, B/conditional_execution/managed_time/clock_lane.rs:24 | process-local | In-memory lease, Signal temporal partition, last observation, quarantine and wake index | Lifecycle token, live lowering pointer, clock lease and Signal owner binding | Fresh trusted clock lease and temporal partition, then reconstruct from durable intents and host-fed time | `install_managed_clock`, clock observation and due-wake delivery | Durable intents are separate from clock resource authority. Never restore the old lease or pointer equality. |
| BridgeOwnedAsyncDefinitions | `BridgeInstalledOwnedAsyncDeclaration`, B/conditional_execution/owned_async/installation.rs:11 | authoritative | In-memory async declaration map at B/conditional_execution/contract.rs:43 | Lowered resource node/Signal binding; process-local-retention policy bit is a contract, not durable custody | 5 definitions; operations 7 | `BridgeOwnedAsyncRequestResponseDeclaration` installation; resource/retry/timeout/queue declarations | An installed policy permitting process-local release does not imply every request may be silently forgotten. |
| BridgeOwnedAsyncCustody | `BridgeOwnedAsyncManagedRequestAdmission`, B/conditional_execution/owned_async/managed_request.rs:24 | mandatory obligation | Caller/controller-held native managed admission, business provenance, optional request and retained refusal | `Arc<()>` owner authority, live Signal port, native occurrence and request identity | 7 | Managed request/initial request, retry/replacement/progress/completion; failures after native admission also leave cleanup (`cleanup_required`, :49) | Dropping the Bridge carrier does not cancel/drain/retire the native request. Recover the logical request and fresh native authority, or contract-defined terminal result. |
| BridgeWritebackReplay | `BridgeWritebackReplayBundle`, B/writeback/replay.rs:12 | optional retained | In-memory canonical digest/basis record returned from execution | No live identity field directly; nested canonical text/digests retain their inputs' affinity | 5; policy unresolved in section 4 | Writeback execution/replay, semantic-noop suppression, success and refusal evidence | Descriptive replay evidence is not durable effect authority. Existing `TruthWritebackReceipt`/Relational artifact is the effect source. |
| BridgeWritebackComparison | `BridgeWritebackReplayRecord`, B/writeback/replay_record.rs:12 | optional retained | In-memory expected/replayed digest comparison and failure/counters | Digest-derived record identity; no explicit runtime id field | 5; policy unresolved in section 4 | Comparing expected vs replayed bundles, including replay failure | Independently returned comparison, not the replay bundle itself. |
| BridgeWritebackExecution | `BridgeWritebackExecutionRecord`, B/writeback/execution.rs:20 | optional retained | In-memory contract/effect/outcome/authority request/receipt/failure record | Request/receipt affinity may be embedded in digest/basis; record id is digest-derived | 5; policy unresolved in section 4 | Writeback execution, mapping/candidate rejection, authority failure and success | Do not retry based on loss of this diagnostic record. Ordinary writeback must resolve through its original authority. |
| BridgeExecutionReservations | `BridgeExecutionBasisReservationRegistry`, B/execution_basis/reservation.rs:18 | process-local | Mutex/hash-set reservations keyed by managed intent | Live registry and intent binding | Fresh managed-execution admission against recovered custody | Managed execution admission, release, request-lane fork | Empty memory after a crash is not proof that an earlier managed effect did not occur. |
| BridgeSealedAssembly | `BridgeSealedRuntimeAssembly`, B/conditional_execution/sealed_runtime_assembly.rs:22 | process-local | Live Bridge-owned Signal runtime and exact Signal/World correspondence service ports | `BridgeSealedOwnerAffinity.signal_graph_instance_id` (:45); live owner/basis ports | Fresh installation plus owner readmission; seal one assembly against fresh Signal graph and incarnation | Host assembly/sealing, managed request-lane composition | `RuntimeBridge` source/sink/writeback/provider adapters (B/facade/runtime.rs:20) must be supplied by trusted composition. `NEXT_SIGNAL_RUNTIME_KEY` (B/facade/runtime/historical_and_replay.rs:3) is not durable identity. |

Additional Bridge record/resource owners:

| Family | Owner type | Class | Current persistence | Process-local identities recorded | Target phase | Creating operations | Notes |
|---|---|---|---|---|---|---|---|
| BridgeDiagnosticRetention | `BridgeDiagnosticsState`, B/diagnostics/state/mod.rs:48 | optional retained | In-memory record deques plus latest-by-identity indexes for routes, source materialization/failure, continuity, merge, historical/structural evaluation, preview, writeback and stream evidence | Nested records may carry live session/runtime bindings; latest indexes add no identity authority | 5 | All Bridge diagnostic-enabled execution paths, including refusals | Existing `BridgeDiagnosticsConfig` has enabled flags and count limits (B/diagnostics/state/config.rs:2). Writeback eviction is explicit (B/diagnostics/state/writeback/retention.rs:144, :217); accessors answer `Option`, not typed expiry. Rebuild indexes over retained records; do not promote them into effect authority. |
| BridgePreviewSession | `BridgePreviewSessionLivenessOwner`, B/facade/preview_session_liveness.rs:11 | process-local | Live liveness lock and preview-session identity; diagnostics retain descriptive preview records separately | Session identity and Arc-backed liveness | Fresh preview admission against readmitted truth; old session closes on Drop (:48) | Preview construction, execution, promotion/discard | A prior process's preview observer cannot authorize new work. Any successful promotion's actual effect belongs to its truth owner. |

### Execution

Execution supplies physical authority and reusable algorithms, not an
application occurrence journal. A yielded domain run must persist its custody
in its owning World/Query/Bridge family. The following rows must never be
mistaken for such custody.

| Family | Owner type | Class | Current persistence | Process-local identities recorded | Target phase | Creating operations | Notes |
|---|---|---|---|---|---|---|---|
| ExecutionPhysicalAuthority | `ExecutionAuthority` / `AuthorityInner`, E/authority/lease.rs:103 | process-local | In-memory thread pool, equivalence registry and resource ledger | Process `OnceLock` at E/authority/lease/process_authority.rs:9; node/lease ids | Host supplies the process authority; a fresh process calls `try_construct_with_equivalences` (:39); same-process reopen uses its existing authority | Composition-root construction | One authority per process lifetime, even after Drop. Reopen must not call the constructor per home. OS-threadless targets use no pool (:50). |
| ExecutionLeaseLineage | `ExecutionResourceLease`, E/authority/lease.rs:142, `LeaseNode` :130 | process-local | Arc parent lineage, worker/memory limits, `Instant` deadline and cancellation | Ledger node id and live authority binding | New `request_lease` against supplied execution authority and new deadlines/cancellation | Request execution, child/nested lease admission | No old worker occupancy or monotonic `Instant` is restored. |
| ExecutionMemoryGrants | `SerialMemoryBudget`, E/authority/memory.rs:77; `ExecutionMemoryReservation` :107 | process-local | Live ledgers, reservations and allocation backing | Ledger issue order, lease node/resource identity | Reserve fresh bytes under admitted request/serial budget | Every retained allocation and executing pattern | Recovered logical material must pay for new physical backing; old grants cannot be decoded from receipts. |
| ExecutionCancellation | `CancellationSource` / `CancellationToken`, E/authority/cancellation.rs:11, :24 | process-local | `Arc<AtomicBool>` signal | Pointer/lineage ownership; no durable id | Fresh authenticated request supplies cancellation source/tokens | Request construction and cancellation | Crash is not a successful cancel/rollback; durable obligations remain with their semantic owner. |
| ExecutionRetainedResult | `ExecutionRetainedResult`, E/backend/retained_result.rs:40 | derived | Arc settled value, partition identity, work/span, optional native receipt and memory grant | Physical backing/receipt admission; partition identity supplied by semantic owner | 5; changed build 6 | Actual native evaluation and canonical settlement | No constructor from arbitrary value/receipt; recover/recompute through the semantic owner, not by forging this carried proof. |
| ExecutionRetainedRun | `ExecutionRetainedRun`, E/pattern/map/retained_run.rs:47; stopped prefix :89 | derived | Persistent `RunTree` of native results/costs | Result backing and partition keys; no retained old request/lease | 5; managed custody integration 7 | Retained map completion/stopped prefix and incremental advance | Native run explicitly carries no source, codec, installation or session authority (:38). Stopped prefix is not completion. |
| ExecutionKeyedPartition | `KeyedPartitioner`, E/partition/keyed.rs:48 | derived | Ordered item/key/partition maps and charged key bytes | Domain-provided item/partition identity, live grants | 5; reconstruct from authoritative membership on demand | Keyed partition creation and edits | A semantic owner's original membership must survive elsewhere; this routing data does not mint aggregate identity. |
| ExecutionComponentPartition | `ComponentPartitioner`, E/partition/components.rs:30 | derived | Adjacency, routes, member maps, retained charge | Partition is least member id; grant is live | 5; reconstruct from authoritative graph on demand | Component partition creation and structural edits | Persist domain edges in their owner, not this accelerator as replacement truth. |
| ExecutionBisection | `Bisection`, E/partition/bisection.rs:50 | derived | Weighted item/edge/incident maps, retained cut tree/routes | Partition paths and live retained charge | 5; reconstruct under admitted policy | Bisection/weight/coupling edits | Algorithmic retained state is not a durable physical authority. |
| ExecutionReduction | `ReductionTree`, E/reduction/tree.rs:92; `RetainedReduction`, E/pattern/reduce/retained.rs:8 | derived | Cached reduction tree, optional retained run and reports | Partition keys and native charged backing | 5; fresh evaluation under reuse contract | Reduce/map-reduce and retained edits | Recomputed value/cost evidence requires the actual current kernel and fresh resource admission. |
| ExecutionDecomposition | `ExecutionDecompose`, E/pattern/decompose.rs:225 | derived | Canonical identities, combine closure and optional prior `Snapshot` | Domain partition keys, live closure/backing | 5; reconstruct lazily with fresh kernel/authority | Decompose initial run and changed-input runs | A closure or previous snapshot is not independently reopenable. |
| ExecutionOrderedStorage | `ExecutionOrderedMap`, E/storage/ordered.rs:27 | process-local | Live ordered-tree cells/backing | Physical allocation grants | Recreate/fund through the owning algorithm's current request | All consumers of ordered retained storage | Container has no additional domain truth beyond the families whose entries it holds. |

## 2. Durable rows with process-local identities

This table distinguishes already durable native artifacts from caller-carried
snapshot/transport artifacts. Listing the latter is intentional: D6 also
governs what later phases may put on disk. Memory-only live handles are in
section 1, not falsely described as existing durable rows.

| Row or artifact | Field and code | Identity source | Replacement phase / disposition |
|---|---|---|---|
| Relational canonical commit branch-cell checkpoint, fork source and basis descriptor | Runtime frame and root identity, R/branch/canonical_runtime_frame.rs:24, :69 | Reserved `CANONICAL_RUNTIME_INSTANCE_ID`, `CANONICAL_ROOT_IDENTITY`; transaction frame also canonicalized | Already neutral under M2 D11. These are **not** leaking process ids. Fresh live references are rebound by native recovery. Do not flag their field names alone as a defect. |
| Signal snapshot node image (caller-carried today) | `CheckpointNodeImage.conditional_contract_occurrence`, S/data/node/checkpoint_image.rs:67; definition lane S/data/node/entry/definition.rs:19 | `NEXT_CONDITIONAL_CONTRACT_OCCURRENCE`, S/data/conditional_execution/contract.rs:4 | 5: home-scoped conditional occurrence and issuer frontier in definition artifacts. |
| Signal snapshot metadata / branch handle (caller-carried today) | `SignalSnapshotMeta.snapshot_id`, `.branch_id`, S/state/lifecycle.rs:41; `SignalBranchHandle`, :28 | Owner/runtime-local branch and snapshot cursors, S/branch/owner_services/owner.rs:58 and S/logic/transaction/runtime/state/branching/branches/owner_partition.rs:54 | 5: home-scoped snapshot/branch identities; keep fresh live admission distinct. |
| Signal serialized graph | `SignalGraph.instance_id`, S/data/graph/runtime/graph.rs:192 | `NEXT_SIGNAL_GRAPH_INSTANCE_ID`, :302 | No serialized leak: field is skipped, deserialization issues a fresh id (:259). Phase 5 must retain this separation while introducing durable owner identity. |
| WASM `RuntimeSnapshotEnvelope` nested callback recipe | `StoredCallbackRecipeSnapshot.token_slot`, `.token_generation`, J/runtime/summaries.rs:290; writer J/runtime/core/state.rs:294 | Thread-local `ComputeCallbackRegistry` slot/generation, J/runtime/compute_callbacks/registry.rs:31 | 5: callback remains process-local and needs fresh attachment. Do not persist token as executable authority; portable export already names unavailable callbacks. |
| WASM snapshot/runtime envelope nested native snapshot | `RuntimeSnapshotEnvelope.snapshot`, J/runtime/summaries.rs:248 | Native Signal snapshot metadata as above | 5, same home-scoped replacement; outer serde does not make the nested ids home-scoped. |

The required idempotency/outbox/application-causality caller-row inventory
cannot be inferred from native canonical framing: arbitrary caller aspect
scalars remain caller-owned data. The scoped carrier boundary and outstanding
schema coverage are recorded in sections 4–5 rather than inventing field names.

## 3. Defects

These are survival gaps relative to the milestone's destination. This audit
changes no product behavior and makes no claim that current memory-only APIs
promise process-death recovery.

| Location | Finding | Target phase |
|---|---|---|
| W/identity/owner.rs:47, :70 | World owner is a process-static ordinal; all other issuer cursors reset to zero. Reusing these in durable causality/occurrence rows would collide or change affinity after restart. | 4 |
| W/lifecycle/owner.rs:130, :145, :149 | New World owner allocates empty history, branch and recovery catalogs; native Relational recovery alone cannot restore product history or unpublished custody. | 4 |
| W/history/publication.rs:37; W/publication/product_cas/movement.rs:14 | Performed publication exists as live catalog facts/CAS with no World medium publication in this path. A durable Relational effect is not evidence that the product moved. | 4 |
| W/recovery/product_unpublished.rs:43, :59; W/recovery/product_unpublished/handle.rs:6 | Catalog-owned custody survives caller Drop within the process, but not process death. Address-like catalog affinity cannot reconnect a pre-crash handle. Continue/release need durable dispositions to avoid resurrecting released custody. | 4 |
| W/publication/progress.rs:42; R/runtime/state/runtime_state/publication_settlement_registry/record.rs:62 | Pending settlement and unresolved native effect are represented by live performed/completion/owner handles. No durable World association currently joins their fate to original product attempt. Do not misclassify `DurabilityUnresolved` as no effect (:36). | 4 |
| W/branch/bootstrap.rs:43, :55; W/lifecycle/owner.rs:173 | Recovered bootstrap proves a recovered Relational basis, then creates an initial-generation World root. It cannot by itself preserve old product generation/occurrence/branch identity. | 4 |
| S/data/node/checkpoint_image.rs:43; S/state/snapshot.rs:17 | Existing snapshot intermixes authored configuration with dirty/version/cause/trace/diagnostic evaluation material. Reusing it wholesale as Phase 5's durable authored image would promote derived state and omit the required owner codec. | 5 |
| S/data/conditional_execution/contract.rs:4; S/branch/owner_services/cell_incarnation.rs:4 | Process-static conditional occurrences and cell incarnations are not recovered frontiers. Only graph-instance id is safely omitted from serde; that omission does not fix the other durable identities. | 5 |
| J/runtime/core/state.rs:287, :325; J/runtime/core/envelopes.rs:117 | Snapshot alone carries recipe results and callback selectors, not expression definitions; restoration only updates already-installed recipes. A snapshot-only reopen cannot reconstruct runtime-authored recipes. Full portable envelope export has a separate definitions section; do not confuse the two paths. | 5 |
| J/runtime/core/envelopes.rs:32; J/boundary/restore_tokens.rs:23 | Exact in-memory root artifact excludes historical snapshots/sibling branches; restore-token registry is realm-local and evicting. Neither can be the durable multi-branch owner home. | 5 |
| B/conditional_execution/installed_lowering.rs:31; B/conditional_execution/sealed_runtime_assembly.rs:45 | Lowering/assembly authority is tied to Bridge runtime/Signal graph identities and live ports. Restoring definitions requires owner readmission and fresh provider/port authority, not copied lowering objects. | 5 |
| B/conditional_execution/managed_time/clock_lane.rs:13, :38 | Temporal intent revision/idempotency association and observation baselines disappear with the clock lane. Reconstructing wakes without restoring original intent association can repeat the logical intent. | 7 |
| S/branch/owner_services/conditional_execution/owned_async/managed_request.rs:32; B/conditional_execution/owned_async/managed_request.rs:24 | Native request/predecessor/retry/queue cleanup custody and Bridge refusal provenance are process values. Failed descriptive admission can leave real native cleanup; neither clean-close refusal nor loss of carrier discharges it. | 7 |

Known-defect disposition within this part: the **temporal idempotency key's
derivation** is not confirmed or refuted here. Bridge accepts a supplied
`idempotency_identity` (B/conditional_execution/managed_time/intent_reconciliation.rs:23)
and retains it; the caller's derivation is outside the assigned paths.
`PayloadRejected` classification and support lifecycle reset to Active are
also caller-owner concerns outside these crates. **Main-only activation
verification** is not established by Relational's generic activation-patch
admission: R/durability/authority/recovery/codec_transition/transition_admission.rs:66
consumes an application-owner proof, while
R/durability/authority/recovery/codec_transition/activation_commitment.rs:16
explicitly leaves activation protocol admission to that owner. These four
claims require the Query/dispatch audit; this part does not label absent
evidence as a refutation.

## 4. Spec conflicts

1. **Caller-row coverage crosses the assigned read boundary.** Relational
   declares storage kinds for generic entity/relation arenas
   (R/storage/substrate/record_arena/kinds.rs:41, :119) and accepts arbitrary
   registered mutation intents (R/transactions/data/intents/mutation_intent.rs:138).
   The scoped production search contains no idempotency, outbox, workflow or
   application-causality schema declaration. Phase 1 specifically demands
   their schema/field inventory, including `world-owner`, but their grammar
   belongs to callers outside Part A's assigned paths. The index must join
   this carrier audit to those caller-owner rows; a generic `CallerRows`
   refusal cannot substitute for their D15 family names. This is a coverage
   boundary, not a proposed relaxation of D6 or D9.
2. **Signal's present “authority” image is broader than the Phase 5 authored
   image.** `AuthorityState` holds a stateful `SignalGraph`
   (S/logic/transaction/runtime/state/reconstructability/authority.rs:9), and
   `SignalCheckpointAuthority` includes cause sets and diagnostics
   (S/state/authority.rs:35). The specification names topology as authored,
   but the current topology/checkpoint also contains dependency snapshots and
   subscriber/evaluation edges (S/state/authority.rs:27;
   S/data/node/checkpoint_image.rs:49). Phase 5 must specify which topology
   facts are authored versus rediscovered dependencies before its format is
   declared final. This audit does not choose a new graph format.
3. **WASM truth has a separate owner and callback boundary.** `RuntimeStore`
   owns source values and recipe/family definitions outside the native graph
   (J/runtime/core/state.rs:218). The Phase 5 artifact description does not
   state how this owner participates. Portable export can carry expression
   definitions but reports unavailable callbacks (J/runtime/core/envelopes.rs:132).
   Decide whether this state has an owner-defined durable section and fresh
   callback-attachment protocol, or an explicit unsupported durable
   capability. Serializing a JS token cannot satisfy authored recovery.
4. **Optional expiry outcomes need a durable contract.** Bridge writeback
   execution/replay/comparison records are independent descriptive values
   (B/writeback/execution.rs:20; B/writeback/replay.rs:12;
   B/writeback/replay_record.rs:12) are retained by `BridgeDiagnosticsState`
   under configured count limits (B/diagnostics/state/config.rs:2;
   B/diagnostics/state/writeback/retention.rs:144, :217). Accessors answer
   `Option` (B/diagnostics/state/writeback/accessors.rs:186), without
   distinguishing never retained from expired. Phase 5 says they follow the audit's class, while
   D10 requires declared optional expiry and typed unavailability. The
   optional-retained classification here must acquire that contract; it must
   not be read as permission to silently discard required effect evidence.
   The same limitation applies to the Relational diagnostic artifact vector.
5. **Derived resource state and mandatory async custody need a stated
   boundary.** Phase 5 calls the resource/temporal runtime derived, but
   `ResourceRuntimeState.in_flight_by_request` holds actual admitted request
   records (S/logic/transaction/runtime/state/resource/state.rs:39), and the
   independently held native managed admission owns queue/retry/cleanup
   (S/branch/owner_services/conditional_execution/owned_async/managed_request.rs:32).
   Phase 7 must identify the durable request/cleanup material that makes
   rebuilding the former lawful, including contract-defined process-local
   terminal cases. Treating the entire current `DerivedState` as disposable
   before extracting custody would lose obligations.

## 5. Coverage statement

All searches were scoped to the assigned crates, except the explicitly
requested governing documents and this output directory. Searches excluded
tests where inspecting production ownership; no test, build, formatter or
enforcement lane was run. No code or generated context file was changed.

The examined directory boundaries were:

- `crates/worth-relational/`: `src/runtime/state/runtime_state/` (including
  settlement records), `src/runtime/state/subsystems/` (services, publication,
  visibility and indexing), `src/schema/` (aggregate registrations),
  `src/aggregate_identity/` (record, projected storage, membership, transition
  and mint), `src/branch/`, `src/change_source/`, `src/visibility/`,
  `src/durability/` (canonical framing/readmission and activation boundaries),
  `src/storage/substrate/record_arena/` and `src/transactions/data/intents/`.
  Crate-scoped production searches checked caller-row names; canonical commit
  and branch-transition durability are the cited M2 boundary, not re-certified.
- `crates/worth-runtime-world/src/`: `identity/`, `lifecycle/` (owner
  construction/operation, bootstrap and service composition), `branch/`
  (registry, reference cells, bootstrap and component custody), `history/`
  (catalog entries, reachability and publication), `publication/` (progress,
  custody, component outcomes and product movement), `recovery/` (catalog,
  unpublished records and handles), and `retention/registry/`.
- `crates/worth-signal/src/`: `data/graph/`, `data/node/` (definition and
  checkpoint image), `data/conditional_execution/`, `state/`, `branch/`
  (basis registry and owner services, lifecycle, metadata, cell incarnation,
  conditional temporal and owned async), and `logic/transaction/runtime/state/`
  (branch partition/ancestry, reconstructability authority, resource and
  temporal owners).
- `crates/worth-signal-wasm/src/`: `runtime/` (core, state/snapshots, portable
  envelopes, callback registries, worker host shell/identity and summaries),
  `boundary/restore_tokens.rs`, and declaration discovery under `expression/`
  and `recipe/`. Worker reports are descriptive; their latest-value fields
  do not constitute a second durable external-custody implementation.
- `crates/worth-runtime-bridge/src/`: `conditional_execution/` (sealed runtime,
  installed lowering, registry/targets, retention, managed time and every
  owned-async production module), `facade/runtime/`, `correspondence/`
  (semantic registry/currentness and slot allocation), `execution_basis/`,
  `writeback/`, and targeted identity/relational-source declarations.
- `crates/worth-execution/src/`: `authority/` (physical construction, lease,
  memory and cancellation), `backend/` (retained result and settlement
  declarations), `pattern/` (retained map/run/reduce and decomposition),
  `partition/` (keyed/components/bisection), `reduction/` and `storage/`.

The unclassified boundary is the **caller-defined row schema inventory**:
idempotency bindings/outcomes, application causality/correction relations,
program activation, workflows, dispatch/outbox/inbound records and any other
caller-written family. Native storage code cannot classify their individual
fields, capability declarations or retention contracts. Section 4 names the
required cross-part join; no completeness claim for that caller inventory is
made here. Similarly, the caller-side temporal key and three named
Query/dispatch defects await that owner's evidence. All directly examined
runtime structures above have a survival classification, including resources
that must explicitly not survive.

For the coordinator's D13 interim-path inventory: Relational's native
`native_checkpoint`, `native_checkpoint_with_policy` and `checkpoint`
(R/durability/authority/checkpointing.rs:10, :25, :84), plus
`restore_native_checkpoint_with_authority` / `restore_native_checkpoint`
(R/durability/authority/recovery.rs:20, :46), are the scoped low-level owner
ports. Their existence does not prove which application transport callers
must retire. Query's `capture_application_checkpoint*`, `*_from_checkpoint*`
and `repair_to_checkpoint*` callers and their interim home choices belong in
the caller-owner part/index. Signal/WASM snapshot and token APIs above are
not silently added to D13's application-checkpoint deletion list.
