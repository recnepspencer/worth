# Stage 2 seams: contract designs

Design only. Nothing here has moved. Source state: `43d28b12e9` (phase-2 milestone, clean `.rs`
tree when read). Seam list: `ownership/simulation/helpers-carry-types/{primary_graph,domain_computation}/simulation.md`,
which is what remains after the scripted "helper carries its types" moves being applied now.
Current reference counts below come from `module_coupling.coupling()` run over
`workspaces/worth-query/crates/worth-query-execution/src/lib.rs` at the same commit (plain
Python, no cargo). They are counts before the helper moves; the simulation's counts are after.

Paths are relative to `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/`
(`pg/` = `primary_graph/`).

## 0. Read this first: two findings that change the list

**0.1 Six of the thirteen primary-graph "contract" seams exist only under the greedy layering.**
The simulation ranks modules with `coupling_graph.layering` (greedy). The draft stack
(`ownership/stack-primary_graph.json`) and the host-supertrait order that phase 2 already
compiles (`ProductOperationHost: ApplicationAttemptHost`, `ApplicationQueryHost: ProductOperationHost`,
`ApplicationEntryHost: ApplicationQueryHost`, `ApplicationContributionHost: ApplicationEntryHost + ApplicationQueryHost`,
`ApplicationInstallationHost: ApplicationContributionHost`) put several pairs the other way up.
The other agent's untracked `ownership/simulation/helpers-carry-types-draft-stack/` run confirms it:
these seams are absent there, and the reverse edge is the (smaller) seam instead.

| greedy seam (low → high) | under the draft stack | under host order | verdict |
|---|---|---|---|
| `product_operation` → `application_contribution` (3 items) | downward; reverse seam is 2 items (`HistoryProducerInvocation`, `ProductOperationContextScopedOps`) | contribution host extends entry and query, not product_operation | layering artifact |
| `application_discovery` → `application_attempt` (9) | downward; reverse is 1 item (`current_definition`) | — | layering artifact |
| `application_output_demand` → `application_query` (4) | downward (query sits below the output layer) | — | layering artifact |
| `application_entry` → `application_attempt` (4) | downward; reverse is 2 items (`HandlerResult`, `build_handler_candidate`) | entry host extends query, query extends product_operation, which extends attempt | layering artifact |
| `causal_inspection` → `application_contribution` (3) | downward (`causal_inspection` is in the surface layer) | — | layering artifact; **no cycle in the source today**: contribution → causal_inspection is 0 references at `43d28b12e9`; the simulation creates the reverse edge by moving 16 `application_query` units into `causal_inspection` |
| `conditional_operation` → `application_contribution` (3) | downward; reverse is 3 items (`classify_bridge_signal`, `WorthQueryConditionalApplicationRuntimeInstallation`, `prepare_dependency_conditional_installation`) | — | layering artifact |

At the domain level, `application_aftermath` → `authorization` (3 items) is the same kind:
the draft stack puts aftermath above authorization, and the draft-stack run has no such seam.

These are designed in section 3 (what to do if the greedy layering is kept), but the
recommendation is to fix the layering first (section 5).

**0.2 Four of the remaining seven are one family.** `output_lineage`, `application_attempt`,
`invariant_projection` and `application_output_demand` all reach into
`application_contribution::partitioned_computation::incremental` for the same retention
vocabulary (`PriorAbsence`, `Suppression`, `WorthQueryPartitionedComputationFullCause`,
`SealedComputationRetention`, `CompletedComputationRetention`, `RetainedComputation`,
`SealedComputationRun`, `NativeComputationReads`, `ComputationPrior`, `ComputationDeposit`).
Four contracts over one family would give it four owners. It is designed once, as family A.

A seventh cross-cutting item, `application_contribution::execution_denial`
(`advancement_scope_stop`, `advancement_checkpoint_stop`), appears in four seams at both
levels and is a plain refusal translation over `request_execution` vocabulary (family F).

## 1. The robust seams (present under both layerings)

### Family A — computation retention (4 seams, 46 references up after the helper moves)

Seams: `output_lineage` → contribution (22 refs, 8 items), `application_attempt` → contribution
(12, 5), `invariant_projection` → contribution (9, 7), `application_output_demand` → contribution
(8, 5; the retention part is `SealedComputationRetention` and `ComputationPrior`).

**What crosses, verified.**
- `pg/output_lineage/retained_computation.rs` holds `RecordedComputation::{Retained(Arc<RetainedComputation>), Absent(PriorAbsence)}`,
  builds `ComputationPrior::new(edition, Err(PriorAbsence::FirstRun.full_cause()), None)` and
  destructures `SealedComputationRun { state, cloned_from, attribution_memory }`.
  `retained_computation/changes.rs` calls `state.original_facts()` and compares `Arc::ptr_eq`.
  `prepared_slot.rs`, `prepared_slot/{computation,recovery}.rs` carry `SealedComputationRetention`;
  `recorded_output.rs` calls `published_state::observe_published`; `restoration*.rs` builds `PriorAbsence`.
- `pg/application_attempt/read_set/{mutation_read_attempt_core,native_completion}.rs` build
  `CompletedComputationRetention::Absent(PriorAbsence::NotProduced)`; `complete_read_set_core.rs`
  carries `SealedComputationRetention`; `read_set/computation_routing{,/ranges,/reuse}.rs` take
  `NativeComputationReads` and `&CompletedComputationRetention`.
- `pg/invariant_projection/operation_reader/decision_reads.rs` names `ComputationPrior`,
  `ComputationDeposit`, `CompletedComputationRetention`, `PriorAbsence`, `Suppression`,
  `NativeComputationReads`, `WorthQueryPartitionedComputationFullCause`; `computation_calls{,/ranges}.rs`
  name `Comparator` and `ComputationPrior`.
- `pg/application_output_demand/registry/required_context.rs` stores `ComputationPrior` and
  `SealedComputationRetention` (`retain_computation_prior`, `take_sealed_computation`);
  `prerequisite_claims/evidence_handoff.rs` carries `SealedComputationRetention`.

**Why a call contract does not work here.** The state makes a round trip through lower modules:
contribution seals a run, the attempt's read set carries it, output_lineage stores
`Arc<RetainedComputation>` on the record, and output_lineage hands the same `Arc` back inside
`ComputationPrior` to contribution, which needs the concrete type (it reads `basis`, `typed`,
`facts`, `checked_calls`). A lower-owned trait object would need a downcast to come back
(proposal 03 option C, outside the rulings); a generic slot would make
`WorthQueryApplicationOutputLineage`, the read set and the demand registry generic over the
state type for one instantiation.

**Design: lower the retention envelope, invert its two classifying impls.**
1. New module `pg::computation_retention` (one responsibility: what one partitioned computation
   retains between runs of its producer, and why a record retains nothing). Placed below
   `application_attempt` — its lowest consumer — and above `application_checkpoint` and
   `request_execution`, which it needs.
2. It takes, whole (type plus all its impls and court doors):
   `PriorAbsence`, `Suppression` (from `incremental/prior_absence.rs`; closed: `PriorAbsence`'s
   only impl, `full_cause`, names only the cause enum), `WorthQueryPartitionedComputationFullCause`
   (derive-only; public through `facade.rs` — the facade re-export path is updated, no shim),
   `ComputationInstallation`, `RetainedBasis`, `RetainedComputation`, `SealedComputationRun`,
   `SealedComputationRetention`, `CompletedComputationRetention` with `CompletedComputationRun`,
   and `ComputationDeposit` (`incremental/deposit.rs`, depends only on the two retention enums).
3. What blocks that today, and how each is cut:
   - `RetainedComputation.facts: SealedComputationFacts` and `.checked_calls: CheckedCallShape`
     live in `pg/application_attempt/read_set/computation_routing{,/layout}.rs`. Both move with
     the envelope (they are the facts a sealed run read). `computation_routing.rs` itself stays
     in the attempt.
   - `RetainedBasis.edition: InstalledProducerEdition` (`application_contribution/producer/registry/semantic_edition.rs`)
     is `Copy` data. Its two impl methods name contribution types:
     `admits_binding::<Schema, Binding: WorthQueryApplicationProducerBinding<Schema>>` and
     `from_declaration(&DeclaredProducerBinding, ApplicationBuildIdentity) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial>`.
     **Contract (impl inversion):** trait `ProducerEditionAdmission` owned by
     `application_contribution::producer::registry`, implemented there for
     `InstalledProducerEdition`, holding those two methods with today's bodies. The type moves
     to `computation_retention::producer_edition` with its pure methods (`retains_computation`,
     `retaining_computation`) and one new restricted constructor
     `pub(in crate::domain_computation::primary_graph) const fn from_parts(digest, build_identity, binding_type, source_type, operation_binding_type, provider_type, retains_computation) -> Self`
     plus `matches_types(&self, binding, source, operation_binding, provider: TypeId) -> bool`,
     so the generic `admits_binding` is a shell over one concrete comparison (iteration law 7).
   - `CompletedComputationRun.carried: Option<CarriedCalls>` drags `CarriedItems`,
     `CarriedPartitions` and native ranges. **Decision point:** either the carried-call
     custody moves too (it is retained-state custody: `incremental/retained.rs` lines 137-185),
     or `CompletedComputationRetention` stays in contribution and the attempt's three uses are
     cut by family-A step 4. Recommended: move it; it is the same lifecycle (sealed or dropped
     with the run).
   - `NativeComputationReads` holds `Arc<RetainedComputation>`, a `SnapshotHandle` and
     `Option<output_lineage::invalidation::ComputationChanges>`. `ComputationChanges` sits in
     output_lineage, above the new module. Contract: `NativeComputationReads` stays in
     contribution; the attempt's three uses (`computation_routing.rs:135,158`, `ranges.rs`,
     `reuse.rs`) take it through an attempt-owned trait (below).
4. **Call contract for what still points up (attempt and invariant projection):**
   trait `NativeCarriageReads` owned by `pg::application_attempt::read_set::computation_routing`,
   implemented in `application_contribution/partitioned_computation/incremental/native_reads.rs`
   for `NativeComputationReads`:
   ```rust
   pub(in crate::domain_computation::primary_graph) trait NativeCarriageReads {
       fn same_native(&self, other: &dyn NativeCarriageReads) -> bool;          // today: same_native(&self, &Self)
       fn at_snapshot(&self, snapshot: &SnapshotHandle) -> bool;
       fn closes_native_ranges(&self, completed: &CompletedComputationRetention,
                               ranges: &Arc<NativeCallCarriage>) -> bool;      // today on CompletedComputationRetention
   }
   ```
   `same_native` compares `Arc::ptr_eq` of the prior and the snapshot; through `dyn` it needs a
   `fn identity(&self) -> (*const (), &SnapshotHandle)` accessor (pointer identity only, no
   downcast). `call_binding` is not in the trait: it is `pub(super)` (`native_reads.rs:19`), so
   only contribution's `incremental` calls it. `closes_native_ranges` reads
   `CompletedComputationRun.carried`; it lands in the trait only if `CompletedComputationRun`
   stays in contribution (decision point in step 3), otherwise it is an inherent method of the
   lowered retention enum.
   `ComputationPrior` stays in contribution (its impl names `PriorComputationRecord` and
   `ComputationChanges`, both output_lineage): output_lineage then must stop constructing it.
   `WorthQueryApplicationOutputLineage::computation_prior_at_address` returns a lineage-owned
   `PriorRecordSelection { retained: Result<Arc<RetainedComputation>, WorthQueryPartitionedComputationFullCause>, record: Option<PriorComputationRecord> }`,
   and its caller in contribution builds `ComputationPrior::new(edition, selection.retained, selection.record)`.
   `application_output_demand/registry/required_context.rs` keeps `ComputationPrior` by value:
   that is the remaining upward reference in this family and it needs the owner's call
   (move the two `ComputationPrior` slots of `RequiredOutputDemandContext` into the producer's
   own context in contribution, or accept `ComputationPrior` moving down once output_lineage's
   two types are reached through `PriorRecordSelection`).

**Typed refusals preserved.** `PriorAbsence::full_cause` keeps its exhaustive match (11 arms)
byte for byte; `WorthQueryPartitionedComputationFullCause` keeps all 21 variants and its public
facade name; `RetainedBasis::drift` keeps returning the same variants. `from_declaration` keeps
`WorthQueryPrimaryGraphInstallationDenial` with the same kinds.

**dyn or generic.** `NativeCarriageReads` is `dyn`. `ProducerEditionAdmission::admits_binding`
stays generic (it is generic today and its type parameters change what is compared; its body
becomes one call into `matches_types`). No new generic type.

**Size.** About 25 files moved (prior_absence, deposit, carriage's `SealedComputationRun` door
part, retained, sealing, semantic_edition, two attempt files), about 70 lines of new trait,
accessor and constructor code, and about 120 import or path lines over the 32 referencing files
listed above plus 9 court-door files. Needs careful design (decision points in 3 and 4).

**Risks.**
- **Conflict with the in-flight helper moves.** The simulation moves `Suppression` to
  `invariant_projection` and `CurrentComputation` to `application_attempt`. `Suppression` must
  instead land in `computation_retention` with `PriorAbsence`; tell the script owner before
  it runs, or this design moves it a second time.
- Field visibility: `RetainedComputation`'s fields are `pub(super)` inside `incremental`.
  Contribution's `prepare*`, `tree_*`, `next_tree*` read them. Moving the type means accessors
  (or `pub(in crate::domain_computation::primary_graph)` fields). Never `pub`.
- Drop order: `retain_computation` drops `cloned_from` before `attribution_memory`; keep the
  destructuring and drop order byte for byte (the ledger custody relies on it).
- `thread_local!` test handoffs in `carriage.rs` (`PRIOR_IN_TEST`, `SEALED_IN_TEST`) move with
  the type whose door uses them (`SEALED_IN_TEST` with `SealedComputationRun`).

### Family B — `application_attempt` → `provider` (18 refs, 8 items; 63 current)

**What crosses, verified.** The attempt holds `&Arc<WorthQueryPrimaryGraphProvider>` in its
provider-execution phases (`provider_execution/phase/prepared/running/progression{,/registered,/fresh,/invariant,/commit_resolution,/mutation_cleanup}.rs`),
obtained through `ProviderHost::primary_provider()`, which `ApplicationAttemptHost` inherits as
a supertrait (`application_attempt/host.rs` line 8). It also owns an inherent
`impl WorthQueryPrimaryGraphProvider` block (`provider_binding/registration/provider_registration.rs:281`,
`register_application_attempt` / `prepare_application_attempt`), which is the reason
`WorthQueryPrimaryGraphProvider`'s unit cannot be judged at all. Methods the attempt calls:
`capture_application_read_set` (`provider/decision_facts/comparison.rs:30`),
`admit_primary_candidate` (`provider/invariant_execution.rs:97`),
`resolve_idempotency_binding_at_product` (`provider/idempotency.rs:157`),
`reserve_outstanding_dispatch` (`provider.rs:135`), `publish_application_commit_causality`
(`provider/commit_causality.rs:36`), `bind_application_idempotency_intent` (`provider.rs:176`),
`reserve_application_attempt` (`provider/application_attempt_state/registration.rs:19`) and the
test hook `after_application_attempt_registration_for_test` (`provider.rs:152`).

**Direction check.** Greedy and draft stack both rank the provider above the attempt, and the
reverse edge (provider → attempt, 149 current references, mostly
`provider/application_attempt_state/**` storing attempt registration and commit types) is
larger. But the host DAG says the opposite: `ApplicationAttemptHost <: ProviderHost`, i.e. the
attempt calls provider extension methods. If the host DAG governs, this seam is provider →
attempt and the design flips (lower the attempt-record vocabulary the provider's store holds).
The design below assumes the provider is above, as both layerings do.

**Design: authority transfer of the provider half, then a narrow port.**
1. Move `application_attempt/provider_binding/registration/provider_registration.rs` (377 lines,
   with `published_causality.rs`) to `provider/application_attempt_state/registration/` beside
   `reserve_application_attempt`. It is the provider's half of registration: it prepares the
   provider store's records, reserves outstanding dispatch, and carries
   `PreparedSourceFactRebase`, `ApplicationFactTable`, `CompletedApplicationFacts` and
   `WorthQueryApplicationPublicationRecoveryReservation` — four of the eight seam items. The
   inherent provider impl then lives in the provider's own module (survives a crate split).
2. Port trait owned by the attempt, `pg::application_attempt::provider_port::AttemptProviderPort`,
   implemented for `WorthQueryPrimaryGraphProvider` in `provider/attempt_port.rs`:
   ```rust
   pub(in crate::domain_computation::primary_graph) trait AttemptProviderPort: Send + Sync {
       fn register_application_attempt(&self, registration: WorthQueryApplicationAttemptRegistration<'_>)
           -> Result<WorthQueryApplicationAttemptRegistrationCompletion, ApplicationAttemptRegistrationStop>;
       fn capture_application_read_set(&self, execution: &ExecutionRequest<'_, '_>,
           authority: &WorthQuerySessionReadAuthority<'_>, control: StorageControl<'_, '_>)
           -> Result<WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetFailure>;
       fn admit_primary_candidate(&self, execution: &ExecutionRequest<'_, '_>,
           session: WorthQueryProviderSessionView<'_>, allocation_policy: ExecutionAllocationPolicy<'_, '_>)
           -> Result<(), WorthQueryInvariantExecutionFailure>;
       fn resolve_idempotency_binding_at_product(&self, execution: &ExecutionRequest<'_, '_>,
           binding: WorthQueryApplicationIdempotencyBinding, product: &WorthQueryProductPublicationBinding)
           -> Result<WorthQueryProviderIdempotencyResolution, WorthQueryProviderIdempotencyResolutionDenial>;
       fn publish_application_commit_causality(&self, reservation: WorthQueryLivePublicationReservation,
           publication: WorthQueryCommittedProductPublication, emissions: WorthQueryAdmittedApplicationEmissionBatch,
           predicate: Option<PreparedPredicatePublication>) -> usize;
       fn bind_application_idempotency_intent(&self, batch: WorkerIntentBatch,
           idempotency: WorthQueryApplicationIdempotencyBinding, outcome_identity: WorthQueryApplicationCommitOutcomeIdentity,
           emitted_effect_count: u64, mutation_partition: PartitionId) -> WorkerIntentBatch;
   }
   ```
   `reserve_outstanding_dispatch` and `reserve_application_attempt` leave the list (only the moved
   file calls them). The registration test hook stays a sealed `*_for_test` door on the provider;
   its one attempt caller (`progression.rs:271`) is reached through the port only if
   `control-callers.txt` allows it, otherwise it moves into the registration file that now lives
   in provider (preferred: it is the provider's own operation control).
3. Host order: drop `ProviderHost` from `ApplicationAttemptHost`'s supertraits; add
   `fn attempt_provider(&self) -> &dyn AttemptProviderPort;` to `ApplicationAttemptHost`
   (runtime implementation in `application_runtime/host/application_attempt.rs`, returning
   `&**self.primary_provider()`); `ProviderHost` gains `ApplicationAttemptHost` as a supertrait
   if the provider's extension traits call attempt operations (they do: the provider is above).
   The progression structs hold `&'a dyn AttemptProviderPort` instead of `&'a Arc<WorthQueryPrimaryGraphProvider>`.
   No provider `Arc` is cloned in `progression.rs` or `progression/*.rs` (checked: the `clone()`
   calls there are on the publication request, locators, product runtime and original
   authority); `registered.rs:76` only returns the reference. The external-dispatch and
   `progressed.rs` sites that call `.primary_provider()` directly need the same check.
4. The remaining two items: `query_read_execution_kind` (`denial/request_read.rs:89`) and
   `ProviderCommittedDispatchOutboxScopedOps` (`provider_execution/external_dispatch.rs`) are
   respectively a helper the strict run already reports as blocked by `relational_execution_kind`
   (both are refusal-kind mappers over relational vocabulary: move the pair to the attempt's
   `denial` module, where 4 of `relational_execution_kind`'s 5 callers are) and a provider
   extension trait called on the host (becomes a port method `committed_dispatch_outbox(&self) -> ...`
   or moves with external dispatch, which proposal 02 already routes through the inbound state).

**Typed refusals preserved.** Every method keeps its `Result` error type
(`ApplicationAttemptRegistrationStop`, `WorthQueryDecisionReadSetFailure`,
`WorthQueryInvariantExecutionFailure`, `WorthQueryProviderIdempotencyResolutionDenial`); no
mapping is added at the port. `relational_execution_kind` / `query_read_execution_kind` move
with their match arms unchanged.

**dyn or generic.** `dyn`: no method is generic and none returns `Self`.
`ExecutionAllocationPolicy<'_, '_>` and `StorageControl<'_, '_>` are passed by value as today.

**Size.** One 377+ line file moved, a 60-line trait plus a 70-line impl, about 30 lines of
host-trait change, about 40 call-site lines in 12 attempt files. Needs careful design (host order
and the test hook).

**Risks.** Lock order inside `capture_application_read_set` (two `attempts.lock()` scopes) is
unchanged because the body does not move. `admit_primary_candidate` reads
`take_skipped_invariant_owner_execution`, a test control: confirm it is not reached through a
second door. The supertrait flip touches every `ApplicationAttemptHost` bound; run
`runtime_extension.py`'s host listing after it.

### Family C — `application_attempt` → `invariant_projection` (10 refs, 5 items; 37 current)

**What crosses, verified.** `ExpectedFactKeys` (`invariant_projection/operation_reader/decision_reads/expected_keys.rs`)
is built by invariant projection (`new` is `pub(super)`) and read by the attempt's read set:
`read_set.rs:101` (`Option<ExpectedFactKeys>`), `mutation_read_attempt_core.rs:21`
(`&mut Option<ExpectedFactKeys>`), `structural_walk.rs:23`, `components.rs:22`, `aggregate.rs:23`.
The other four items are one reference each: `ApplicationAttemptHost::mutation_projection()`
returns `&WorthQueryApplicationInvariantProjectionAuthority<Schema>` (`host.rs`),
`read_scope.rs` names `WorthQueryRealizedProjectionScope`, `read_set.rs` names
`WorthQueryApplicationOperationInvariantProjectionSnapshot`, `projected_extension.rs` names
`BorrowedOperationProjection`.

**Design: read contract for the expectations, host method for the authority.**
- Trait `DecisionFactExpectations` owned by `pg::application_attempt::read_set`, implemented in
  `invariant_projection/operation_reader/decision_reads/expected_keys.rs` for `ExpectedFactKeys`.
  Methods are exactly the ones the attempt calls (`expected_keys.rs:37-116`):
  `component(..)`, `source_authorization(..)`, `managed_source(..)`, `aggregate(..)`,
  `structural_walk(..)`, `source_publication(&self, key: &SourcePublicationRead) -> Option<&Arc<ObservedSourcePublication>>`,
  `attribution(&self) -> &ComputationFactAttribution`, and the two iterators.
- The two iterators (`iter`, `refresh_keys` return `impl Iterator<Item = &Key>`, each an
  ordered merge through the private `union`) are not object-safe. Their four callers are two
  equality checks (`read_set/completion.rs:21` `self.facts.keys().eq(expected.iter())`,
  `read_set/native_completion.rs:25` `.eq(expected.refresh_keys())`) and two loops
  (`read_set/projected_completion.rs:90,113`). A visitor does not serve `Iterator::eq`, so the
  trait returns `Box<dyn Iterator<Item = &Key> + '_>` for both (one allocation per call, four
  calls per completed read set), or, better, the trait gains the two questions the callers ask:
  `fn same_keys(&self, keys: &mut dyn Iterator<Item = &Key>) -> bool` and
  `fn same_refresh_keys(..) -> bool`, plus a `for_each_*` visitor for the two loops. The second
  keeps the merge inside invariant projection and allocates nothing.
- The attempt keeps the value slot: `Option<ExpectedFactKeys>` becomes
  `Option<Box<dyn DecisionFactExpectations + Send>>`. Invariant projection boxes on fill.
  That is one allocation per mutation read attempt; if the read set is a demonstrated hot path,
  record a generic `E: DecisionFactExpectations` on the read set instead (ruling exception, with
  its reason).
- `source_publication` returns `ObservedSourcePublication`, an output_lineage type above the
  attempt (`application_attempt` ↔ `output_lineage` is its own two-item seam). Resolve that seam
  first (move `ObservedSourcePublication` down with `PostconditionCarry`, its other item), or
  the trait inherits the upward name.
- `mutation_projection()`: the attempt only threads the authority to invariant projection's own
  readers. Replace the host method's return type with an attempt-owned
  `&dyn MutationProjectionSource<Schema>` exposing the two calls the attempt makes (the snapshot
  open and the realized scope), implemented by `WorthQueryApplicationInvariantProjectionAuthority`
  in invariant projection. `WorthQueryRealizedProjectionScope`, the snapshot and
  `BorrowedOperationProjection` then appear only as that trait's associated outputs; if they
  are data the attempt stores, they move down instead (they are each referenced once).

**Typed refusals preserved.** None of these methods returns a refusal; the snapshot open keeps
its current error type.

**dyn or generic.** `dyn` with the visitor change. Fallback generic recorded above.

**Size.** About 25 lines of trait, 40 lines of impl, 30 call-site lines in 7 attempt files.
Needs careful design (iterator object safety, the output_lineage dependency).

**Risks.** `refresh_keys` is a merge of two iterators (`expected_keys.rs:121`); the visitor must
keep the merge order (it feeds refresh ordering). Boxing changes the attempt's `Send` bound.

### Family D — `application_attempt` → `application_output_demand` (12 refs, 4 items; 15 current)

**What crosses, verified.**
- `RequiredOutputDemandContext` is a pass-through payload: `provider_binding/mod.rs:29,78`,
  `provider_binding/registration.rs:15`, `provider_execution/phase/prepared.rs:136`,
  `provider_registration.rs:22,70`. The attempt stores `Option<RequiredOutputDemandContext>` and
  hands it to provider registration; it calls no method on it in those files.
- `WorthQueryOutputDemandSettlement` is read by the workflow transition program's assessment
  (`workflow_transition_program/assessment{,/identity,/replay}.rs`,
  `preparation/adapter_{commit,replay}.rs`), through `SettlementScopedOps`.
- `WorthQueryOutputDemandDenialKind` is named once (`compare_and_commit/commit_deferred.rs:38`).

**Design.**
1. After family B moves `provider_registration.rs` into provider, three of the six
   `RequiredOutputDemandContext` references leave the attempt. The rest are the slot the
   attempt carries from the producer to the provider. Make the slot opaque to the attempt with
   a contract owned by the attempt: `pg::application_attempt::provider_binding::RequiredDemandCustody`
   — no methods the attempt calls; the attempt only moves it. Because the provider (above)
   needs the concrete context back, a trait object would need a downcast. So instead
   **authority transfer**: the producer-side code that puts the context into the attempt's
   registration (`provider_binding/mod.rs:78`, `fn with_required_output_demand(context)`) moves
   the context straight into the provider-side registration it now builds (family B step 1),
   and the attempt's own `WorthQueryApplicationAttemptRegistration` loses the field. Verify:
   `prepared.rs:136` is the only other holder; it hands the value to the registration.
2. Settlement read contract, trait `WorkflowSettlementObservation` owned by
   `pg::application_attempt::workflow_transition_program::assessment`, implemented for
   `WorthQueryOutputDemandSettlement` in `application_output_demand/settlement/`. Methods: the
   `SettlementScopedOps` methods the five files call (list them from `assessment.rs:39,258`,
   `identity.rs:91`, `replay.rs:40,139` before landing; the script `runtime_extension.py`
   prints the called set). The five parameters become `&dyn WorkflowSettlementObservation`.
3. `commit_deferred.rs:38` names the denial kind to build a deferred-commit refusal: move that
   construction to the output demand host method that already reports the deferral, or lower the
   one variant it needs as a value the attempt carries (`WorthQueryOutputDemandDenialKind` is
   output demand's typed refusal; do not re-encode it).

**Typed refusals preserved.** The deferred-commit path keeps `WorthQueryOutputDemandDenialKind`
as built today; no new error type.

**dyn or generic.** `dyn`.

**Size.** About 20 lines of trait, 30 lines of impl, 25 call-site lines; field removal in 3
structs. Mechanical once family B lands, except step 3.

**Risks.** Removing the registration field changes the registration's destructuring in
`provider_registration.rs:prepare_application_attempt` (its 19-field `let` pattern); keep the
order of the remaining fields.

### Family E — producer output evaluation (`application_output_demand` → contribution, non-retention part)

Items: `ProducerOutputEvaluation` (3 refs: `readiness_delivery.rs`, `registry/ready_backing.rs`,
`settlement/observation.rs`), `bridge_denial_kind` (1: `denial/output_comparison.rs`),
`PendingProducerPublication` (1: `registry/checkpoint.rs`). The causal-inspection seam names the
same evaluation trio (section 3).

**Design.**
1. `bridge_denial_kind` (`application_contribution/producer/demand/bridge_denial.rs`, 80 lines)
   and its only dependency `query_cause` (`demand/bridge_execution_denial.rs`, 33 lines) map
   Bridge denial kinds to `WorthQueryOutputDemandDenialKind`, output demand's own refusal type,
   and need only `worth_runtime_bridge` and `request_execution`. Move both files to
   `application_output_demand/denial/` (`bridge_conditional.rs`, `bridge_execution.rs`). No trait.
   Mechanical.
2. `ProducerOutputEvaluation`, `ProducerOutputEvaluationState`, `ComparedProducerOutput` and
   `WorthQueryProducerEvaluationIneligibility` (`producer/output_evaluation.rs`) are evidence the
   settlement and readiness delivery retain. Their only contribution-owned field type is
   `Arc<InstalledProducerOutputEquivalence>`, read through three accessors (`field`,
   `contract_revision`, `field_meaning`). Contract: trait `ProducerOutputFieldScope` owned by
   `application_output_demand::settlement::output_evaluation` (new child holding the four types),
   implemented for `InstalledProducerOutputEquivalence` in
   `application_contribution/producer/output_equivalence_contract.rs`:
   `fn locus(&self) -> &ApplicationFieldBindingLocus; fn contract_revision(&self) -> AspectContractRevision; fn meaning(&self) -> [u8; 32];`.
   Fields become `Arc<dyn ProducerOutputFieldScope + Send + Sync>`. Before landing, grep every
   `.field` / `.scope` read in contribution: if contribution reads anything beyond these three,
   it is added to the trait; if it needs the concrete equivalence back, stop (that is a downcast).
   The evaluation's constructor (in contribution, `output_evaluation/installation.rs`,
   `native.rs`) stays in contribution as a function producing the lowered type.
   The state enum names `application_attempt::OutputFieldCaptureFailure`, which is below both: fine.
3. `PendingProducerPublication` (`producer/execution/unpublished.rs`) is named once by the demand
   registry's checkpoint capture. Hand move with its impls if they name only output demand and
   lower types (verify `unpublished.rs`), otherwise route through the checkpoint host method.

**Typed refusals preserved.** `bridge_denial_kind`'s match keeps every arm and the
`ExecutionRequest` / `of_execution_interruption` split byte for byte.

**dyn or generic.** `dyn` (accessors only).

**Size.** Step 1: 2 files moved, 4 import lines. Step 2: 4 types moved (~150 lines), ~15 lines of
trait and impl, ~15 call-site lines. Step 1 mechanical; step 2 careful.

### Family F — `execution_denial` stop translation (cross-level)

`application_contribution::execution_denial::{advancement_scope_stop, advancement_checkpoint_stop}`
(`application_contribution/execution_denial.rs:243-276`) map `worth_execution::WorkCeilingDenial`
and `MapKernelStop` to `request_execution::WorthQueryAdvancementDenial` through
`request_execution::execution_stop::{checkpoint_denial, lease_denial}`. They name nothing in
primary_graph. The file's own doc says the lease and kernel-stop door sits "beside the request
execution that meets them". Callers: `pg/application_attempt/compare_and_commit/cleanup_required.rs`,
`execution_runtime/product_world/owner_cleanup/{admitted,retirement}.rs`,
`product_publication/{owner_close,discovery}.rs`, plus contribution, entry, query, invariant
projection, output lineage and product operation (15 files in all).

**Design.** Move both functions to `request_execution/execution_stop.rs` (beside
`checkpoint_denial`), visibility `pub(in crate::domain_computation)` as today; rewrite the 15
callers' paths (`module_move.py` item mode). No trait. The simulation missed it because callers
import the module (`use ...::execution_denial;`) and call `execution_denial::advancement_scope_stop`,
which it counts as an unmovable module reference.

**Effect.** Removes the `execution_denial` item from four seams: `application_attempt` →
contribution, `invariant_projection` → contribution (`advancement_scope_stop`),
`execution_runtime` → `primary_graph` (2 of 5 refs) and `product_publication` → `primary_graph`
(the whole 2-ref seam).

**Typed refusals preserved.** Same matches, same `WorthQueryAdvancementDenial` variants.

**Size.** About 40 lines moved, about 20 path lines. Mechanical.

## 2. Domain-level contract seams

### `provider_session` → `primary_graph` (22 refs, 9 items; 58 current)

**What crosses, verified.** `provider_session/graph_obligation` (the managed graph work
session) carries primary-graph payloads to primary-graph callbacks:
`owner_execution.rs` holds `Arc<WorthQueryPrimaryGraphLayout>` and runs
`read: impl FnOnce(&mut RelationalRuntime, &WorthQueryPrimaryGraphLayout) -> T`;
`session/{prepared_query_read,query_read,resource_read,inspection_read}.rs` pass the layout to
the same kind of closure; `session/{inspection_preparation,inspection_read,product_read}.rs`
hold `Arc<WorthQueryApplicationInspectionTarget>`; `session{,/mutation,/mutation_readmission}.rs`
hold `WorthQueryApplicationSnapshotLease`; `protocol/readmission/prepared/staged.rs` holds
`WorthQueryApplicationAttemptBasis` / `Affinity`; `decision_read_set/capture{,/primary}.rs`
take `RegisteredPrimaryRead` and `PrimaryDecisionReadSetComparison`;
`read_terminal/source.rs` holds `Arc<SourceFactsAllocation>`.

Blockers: `WorthQueryPrimaryGraphLayout`'s impl names `WorthQueryPrimaryGraphInstallationDenial`
(`pg/denial`); the snapshot lease's impl names the layout; the inspection target's impls name
`InspectionOriginal`/`SourceOriginal` (causal inspection).

**Design.** This is the same vocabulary as the authorization seam (layout, inspection target,
snapshot lease, source-facts allocation): see `merges.md`, section 2, which lowers it.
For the payload pass-through specifically:
- The layout: provider_session passes it to the closure (verify no method call on it in
  `graph_obligation/**` before landing; the reads above are all field holds and closure
  arguments). An erased `Arc<dyn Any>` slot would need a downcast, so instead the session's read
  entry points become `fn with_owner_read<T>(&self, read: impl FnOnce(&mut RelationalRuntime) -> T) -> T`
  and the primary-graph caller captures its own `Arc<WorthQueryPrimaryGraphLayout>` in the
  closure. The session loses the `layout` field (`owner_execution.rs:15`,
  `prepared_query_read.rs:21`). This is a signature change on 6 call paths, all in
  primary_graph callers (`pg/application_query`, `pg/causal_inspection`).
- The inspection target and the snapshot lease are held for their lifetime (custody), and
  returned on terminal. They need the lowering in `merges.md` §2 (with
  `WorthQueryPrimaryGraphLayout` and `pg/denial` moved below `provider_session`).
- `RegisteredPrimaryRead` / `PrimaryDecisionReadSetComparison`: the decision read-set capture
  calls into the primary provider's attempt store. Port trait `PrimaryReadSetCapture` owned by
  `provider_session::decision_read_set`, implemented by the primary graph provider in
  `pg/provider/decision_facts/comparison.rs` (same file that defines
  `capture_application_read_set`). Signature from `capture/primary.rs`: verify before landing.
- `WorthQueryApplicationAttemptBasis` / `Affinity`: identity values; hand move with their impls
  to provider_session's readmission (they are the readmission's affinity key) if their impls name
  nothing above; otherwise lower with the merges §2 family.

**dyn or generic.** The closure change keeps the existing method-level generic `T` (it is
generic today); `PrimaryReadSetCapture` is `dyn`.

**Size.** ~60 call-site lines for the closure change, ~30 for the port. Careful.

**Risks.** The read closures run under the session's unwind isolation; capturing the `Arc` in
the closure must not extend the layout's lifetime past session close (it is an `Arc` clone; the
session's own clone is dropped instead, so net references are unchanged).

### `application_aftermath` → `primary_graph` (8 refs, 3 items; 40 current)

Items: `WorthQueryApplicationIdempotencyBinding` (4: `recovery_handle/binding.rs`,
`recovery_progression/resolve.rs` and their doors), `WorthQueryPreparedCorrectionSource` (3:
`causal_fact.rs`, `commit_causality.rs`), `WorthQueryApplicationCommitDenial` (1:
`commit_causality.rs`).

**Design.**
- `WorthQueryApplicationIdempotencyBinding` (`pg/application_attempt/idempotency.rs:88`) is a
  value of digests. Two things hold it up: the helper `encode_identity`
  (`idempotency/encoding.rs`; helpers never carry helpers) and `bind_operation_scope(self, &WorthQueryOperationScopeBinding)`
  (authorization). **Impl inversion:** `bind_operation_scope` becomes an extension trait
  `OperationScopeIdempotency` in `authorization::operation_scope_binding`, implemented for the
  binding, whose body calls a new `bind_scope_identity(self, identity: &[u8; 32], ...)` on the
  binding with the bytes it reads from the scope today. Then the binding unit, `encoding.rs`
  and its `recorded_intent` encoders move to `domain_computation::application_outcome_identity`
  (where `WorthQueryApplicationOutcomeIdentity` already sits) or to `application_aftermath`
  (its other consumer). Recommended: `application_outcome_identity` — it is commit identity, not
  aftermath meaning. Verify the remaining `impl` blocks in `idempotency/*.rs` name only digests
  and lower vocabulary (`admitted_intent`, `producing_implementation` may name authorization's
  admitted operation: the simulation lists `WorthQueryAdmittedApplicationOperation` as a holder,
  so a second inversion is likely).
- `WorthQueryPreparedCorrectionSource`: aftermath records its causal fact. Contract owned by
  aftermath, trait `CorrectionCausality` with the accessors `causal_fact.rs` and
  `commit_causality.rs` call, implemented in `pg/product_operation/correction_source/preparation.rs`.
- `WorthQueryApplicationCommitDenial`: one use, building the causality refusal. Pass the denial's
  stage (`WorthQueryApplicationCommitDenialStage`) or move the construction to the primary graph
  caller.

**Typed refusals preserved.** Commit denial kinds unchanged; the inverted
`bind_operation_scope` writes the same bytes into the same slot.

**dyn or generic.** `dyn`.

**Size.** ~15 files moved for the binding family, ~20 lines of traits. Careful (the second
inversion).

### `application_aftermath` → `authorization` (8 refs, 3 items)

Layering artifact under the draft stack (aftermath above authorization). If the greedy order is
kept: `WorthQueryAdmittedApplicationOperation` (3) and `WorthQueryAdmittedApplicationCapabilityAccess` (1)
are read in `recovery_progression/{authority,binding_axis,disclose}.rs` and `commit_causality.rs`
for identity and capability bytes; `WorthQueryApplicationCommitReceipt` (4) is listed here only
because the simulation moves it into authorization. Contract owned by aftermath:
`RecoveryAuthorityBasis` with the identity accessors those four files call. Prefer fixing the
layering.

### `execution_runtime` → `primary_graph` (5 refs, 3 items; 8 current)

Items: `execution_denial` (2, family F), `SourceInvalidationOwner` (2:
`product_world/source_owner.rs` and its door), `WorthQueryPrimaryGraph` (1:
`runtime_root.rs`, with `WorthQueryPrimaryGraphIntegrationHandle`).

**Design.**
- Family F removes `execution_denial`.
- `SourceInvalidationOwner` (`pg/output_lineage/invalidation/owner.rs`) is installed into the
  product world's source owner slot. Port trait `ProductSourceInvalidation` owned by
  `execution_runtime::product_world::source_owner`, implemented by `SourceInvalidationOwner` in
  output_lineage, with the methods `source_owner.rs` calls (head-cell registration returning
  `HeadCellRegistrationStop` — that stop type moves down with the trait as its error, or is
  mapped at the impl; keep the typed stop, do not stringify).
- `WorthQueryPrimaryGraph` / integration handle in `runtime_root.rs`: the runtime root wires the
  graph. This is construction wiring: move the wiring line to `pg/root` (the graph registers
  itself with the runtime root) or the root holds a `dyn` integration port owned by
  execution_runtime. Read `runtime_root.rs` before choosing; one reference.

**Size.** ~30 lines. Mechanical after family F, except the root wiring choice.

## 3. Greedy-layering seams (designs if that layering is kept)

Recommended action for all six: adopt the layering first (section 5); then these are downward
edges and need nothing. If the greedy layering stands, the designs are:

| seam | items (refs) | design if kept |
|---|---|---|
| `product_operation` → contribution | `WorthQueryApplicationProducerBinding` trait (12, `generated_materialization/**`), `WorthQueryApplicationProducerProvider` trait (3), `WorthQueryInstalledApplicationProducerRegistry` (1, `product_operation/host.rs` `installed_producers()`) | Generated materialization implements reconstruction over a producer binding. Port owned by product_operation: `GeneratedOutputProducer<Schema>` (the binding methods `generated_materialization/reconstruction/output_{entities,roles}.rs` call), implemented by contribution for every binding. Generic, not dyn: the binding trait is `Sized + 'static` with associated types (`producer.rs:231`). Records a generic. ~80 lines. |
| `application_discovery` → attempt | workflow fact readers: `observe_adjacency` (4), `WorthQueryWorkflowLayout` (3), `workflow_instance_observation::{governed_record, audit::observe, history_projection, ReadFields}`, `observe_field_value`, `WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION` | Lower the workflow-instance fact readers out of the attempt into a `workflow_facts` module below discovery (the stack's `graph_root_and_workflow_facts` layer already names `workflow`). Blocker: `observe_adjacency` held by `observe_adjacency_checked`/`resolve_observation` (helpers). Hand move of the helper trio. ~10 files. |
| `application_output_demand` → query | `WorthQueryRetainedQueryDependencies` (4), `WorthQueryObservedSourceEpoch` (3), `WorthQueryManagedIndexDenial` (2), `…DenialCause` (1); plus 11 more items today (`WorthQueryApplicationReadObservation` ×15) | Data types the demand registry stores. Lower `observed_source::source_identity` (epoch, meaning, coordinate, checkpoint identity) below output demand. 47 current refs: large. |
| `application_entry` → attempt | `WorthQueryApplicationEffectProgram`, `WorthQueryCompleteApplicationReadSet`, `ReadSetScopedOps`, `SourceFactsScopedOps` (1 each) | Host-order problem: entry calls attempt extension traits. `ApplicationEntryHost` would extend `ApplicationAttemptHost`. Under the host order this is already downward. |
| `causal_inspection` → contribution | `ProducerOutputEvaluation`, `…State`, `ComparedProducerOutput` (1 each, `application_inspection/connection/evaluation.rs`) | Family E step 2 lowers them into output demand, which sits below causal inspection under every order. No extra work. |
| `conditional_operation` → contribution | `WorthQueryInstalledApplicationProducerRegistry`, `WorthQueryApplicationProducerBinding`, `WorthQueryOutputReadinessContractDenial` (1 each, `installation{,/denial,/output_readiness}.rs`) | Installation wiring: conditional installation reads the producer registry. Under the draft stack conditional_operation is installation-layer, above contribution. If kept: `ConditionalReadinessInstallation` port owned by conditional_operation, implemented in `application_contribution/conditional/`. ~40 lines. |

## 4. The two "hub" seams that 3+ items put in this class

`product_operation` → `application_query` (10 items) and `product_operation` →
`application_installation` (6 items) carry `AdmissionScopedOps`, `OutputReadScopedOps`,
`OneShotCorrectionDisclosureScopedOps`, `WorthQueryProgramApplicationRuntime`,
`WorthQueryProgramOwner`: the context-type problem of proposal 03. They are planned in
`proposal-03-plan.md`, not here.

## 5. Order of work

1. Owner decision: which order governs (greedy, draft stack, or the host-supertrait order that
   already compiles). The three disagree on `product_operation`, `application_query`,
   `application_entry`; see `README.md`.
2. Family F (mechanical, cross-level).
3. Family E step 1 (mechanical).
4. Family B (unblocks D).
5. Family D.
6. Family A (largest; coordinate the `Suppression` move with the helper script).
7. Family C (after `application_attempt` ↔ `output_lineage`).
8. Domain seams, after `merges.md` §2 lowers the foundational vocabulary.

After each step: `module_coupling.py report` at both levels, `boundary-check`, `agent-context check`,
the owning court crates, and the line-cap guard in `dirty` mode.
