# Stage 2 seams: the three nest-or-merge seams

Design only. Source state `43d28b12e9`. Counts are from `module_coupling.coupling()` over
`worth-query-execution` at that commit ("current") and from
`ownership/simulation/helpers-carry-types` ("after the helper moves"). Paths are relative to
`workspaces/worth-query/crates/worth-query-execution/src/domain_computation/` (`pg/` =
`primary_graph/`).

Verdict in one line each:

| seam | one responsibility? | recommendation |
|---|---|---|
| `product_operation` ↔ `application_attempt` | no | neither nest nor merge: place product_operation above the attempt (host order and draft stack already do), then three small moves |
| `authorization` ↔ `primary_graph` | no | neither nest nor merge: lower the primary graph's foundational vocabulary below authorization, and move authorization's attempt-driving steps up into the primary graph |
| `convergence_epoch` ↔ `managed_run` | no, and **not a cycle in the source** | place convergence_epoch above managed_run; break the one real edge (`provider_session` → `convergence_epoch`) by moving the convergence provider port to provider_session's semantic provider ports |

Law 12 (domain structure) asks whether each strongly connected component is one
responsibility. In all three the answer is no: the two sides differ in lifecycle and failure
topology, and the crossing is concentrated in a few families that have an obvious owner.

---

## 1. `product_operation` ↔ `application_attempt`

### Evidence

- **References.** Current: product_operation → attempt 75, attempt → product_operation 39.
  After the greedy helper moves: 19 up in 18 items (product_operation counted as the lower side).
- **The greedy layering is the cause.** The greedy order puts `product_operation` (position 48)
  below `application_attempt` (61). Two other orders disagree:
  - the host-supertrait order already compiled in phase 2:
    `pg/product_operation/host.rs` declares `ProductOperationHost<Schema>: … + ApplicationAttemptHost<Schema> + …`,
    so product_operation is the higher module;
  - the draft stack (`ownership/stack-primary_graph.json`) puts `product_operations` two layers
    above `attempt_and_commit`.
  Under the draft stack the other agent's run (`simulation/helpers-carry-types-draft-stack`)
  leaves **one item** in this pair: `HistoryProducerInvocation`, 9 references, attempt →
  product_operation.
- **The greedy simulation makes it worse.** To satisfy the greedy order it moves 37 attempt
  units *into* product_operation and moves the hub `WorthQuerySelectedProductOperation` down to
  `application_program`. Neither is a placement anyone would choose.
- **Different responsibilities.** product_operation is product-scoped selection and the
  operations run against a selected product (`context`, `correction_source`, `program_adoption`,
  `generated_materialization`, `document_history`, `branches`, `transaction`); its lifecycle is a
  product branch lease plus a basis lease. application_attempt is one compare-and-commit attempt
  (`compare_and_commit`, `provider_execution`, `effect_program`, `idempotency`, `read_set`); its
  lifecycle is one commit and its failures are `WorthQueryApplicationAttemptDenial` /
  `WorthQueryApplicationCommitDenial`. product_operation calls into attempts (75 references:
  idempotency binding 16, output correspondence 8, workflow dispositions 5, commit receipt 4, …);
  that is the ordinary downward direction.
- **What points up from the attempt (current, 39 refs).**
  - `WorthQuerySelectedProductOperation` (18): 8 inherent impl blocks in the attempt
    (`workflow_transition_program/preparation{,/observation}.rs`, `workflow_instance_program/preparation.rs`,
    `workflow_proposal_program/preparation.rs`, `workflow/captured_input.rs`,
    `workflow/definition/{retirement,preparation}.rs`, plus a door) and 4 more sites that take
    `selected: &WorthQuerySelectedProductOperation` as a parameter
    (`workflow_transition_program/preparation/adapter_preparation.rs`,
    `workflow_instance_program/preparation/adapter.rs`, `…/succession/preparation.rs`,
    `…/birth/preparation.rs`). These prepare workflow attempts *from*
    a selected product operation.
  - `HistoryProducerInvocation` (9): `provider_execution/entry{,/output_source,/application_commit}.rs`
    and `phase/prepared/entry_custody/{output_source,application_commit,forwarding}.rs` carry
    `Option<HistoryProducerInvocation<'_>>` (`pg/product_operation/document_history/navigation/output_authority/producer_invocation.rs:14`:
    borrowed currentness, request, principal binding and two `HistoryAuthorityLifetime`s).
  - `WorthQueryAdmittedApplicationConditionalDefinition` (7, `effect_program/{conditional_definition,model}.rs`, `provider_binding`):
    the draft-stack run moves it into the attempt whole, so it is movable.
  - `SelectedQueryOrigin`, `SharedSelectedProductOperation` (1 each, `snapshot_lease.rs`): the
    draft-stack run moves both into the attempt.
  - singles: `WorthQueryProgramMigrationPreparationDenial` (`migration.rs`),
    `AuthoredCorrectionCandidateDenial` (`provider_binding/registration.rs`),
    `ObservationRoomScopedOps` (`phase/prepared/currentness.rs`).

### Recommended structure

No nest and no merge. `product_operation` sits above `application_attempt` (as the hosts
already say). The attempt keeps the commit pipeline; product_operation keeps selection and the
product-scoped operations; the cross-module behaviour is product_operation's.

### Migration

1. **Layering.** Record product_operation above application_attempt in the governing order
   (owner decision, see `README.md`); regenerate `module_coupling.py baseline` for primary_graph.
2. **Workflow preparations move up** (authority transfer, part of `proposal-03-plan.md` step 4).
   The 8 inherent blocks on `WorthQuerySelectedProductOperation` in the attempt move into
   `pg/product_operation/workflow_preparation/` (one file per program: transition, instance,
   proposal, captured input, definition), as inherent impls in the type's own module, so they
   survive a crate split. Each calls the attempt's preparation functions with the parts it needs
   (`snapshot_lease`, `product`, `application_basis`), not the context. The 4 attempt sites
   taking `&WorthQuerySelectedProductOperation` change their parameter to those parts (or move
   up with their program if they only exist to serve it).
   Typed refusals unchanged: the moved methods return the same attempt denials.
3. **Hand moves** (draft-stack run verdicts): `WorthQueryAdmittedApplicationConditionalDefinition`,
   `SelectedQueryOrigin`, `SharedSelectedProductOperation` → application_attempt (whole units,
   `module_move.py`).
4. **`HistoryProducerInvocation`.** The attempt only forwards it from entry to entry custody to
   the output source and application commit (verify: no method call on it in the six attempt
   files). Lower it into `pg/application_attempt/provider_execution/entry/` together with what
   it holds that is product_operation's (`HistoryAuthorityLifetime`); the
   `HistoryProducerCutover` that wraps it stays in product_operation. If `HistoryAuthorityLifetime`
   cannot move (check its impls), the attempt takes a lower-owned `&dyn HistoryOutputAuthority`
   with the two calls the output source makes on it.
5. **Singles.** The two denials are refusal values the attempt carries: move each with its
   impls to the attempt's `denial` module if its impls name nothing above; `ObservationRoomScopedOps`
   becomes an `ApplicationAttemptHost` method (it is an extension trait called on the host).
6. Re-measure; the pair should read 0 up.

Size: ~12 files moved, ~60 call-site lines; step 2 is the careful one and is shared with
proposal 03.

---

## 2. `authorization` ↔ `primary_graph` (domain level)

### Evidence

- **References.** Current: authorization → primary_graph 264 references to 90 items;
  primary_graph → authorization 723 references to 110 items. After the helper moves: 120 up in
  42 items. The simulation also moves 42 primary-graph units *into* authorization.
- **Size.** authorization is 31,143 lines, primary_graph 408,492. Merging adds authority code to
  a module already far beyond any crate ceiling (iteration law 1) and removes the one boundary a
  later crate split would keep.
- **authorization is consumed outside the primary graph.** `operation_binding` → authorization 3,
  `provider_session` → authorization 3, `application_aftermath` → authorization 9. Nesting it
  under primary_graph would turn those 15 references into upward edges into primary_graph.
  Its own downward needs are domain-level: provider_session 78, runtime_time 25,
  execution_runtime 16, request_execution 12.
- **What authorization reads, after the helper moves (120 references):**

  | family | refs | items |
  |---|---|---|
  | schema layout | 16 | `WorthQueryPrimaryGraphLayout` |
  | principal and entity identity | 11 + 17 + 3 | `WorthQueryAuthenticatedPrincipal`; `entity_resolution::{resolved::WorthQueryApplicationEntityIdentity, WorthQueryResolvedEntity, WorthQueryEntityResolutionTruth, product::issued_scope::WorthQueryIssuedSelectedScope, WorthQueryInstalledEntityResolutionContext}`; `resolution::…::WorthQueryIssuedSelectedPrincipal` |
  | freshness and currentness | 5 + 1 | `freshness::{validate_freshness_at_snapshot, WorthQueryDurablePrincipalCurrentness, WorthQueryPrincipalFreshnessEvidence}`, `principal_currentness_capture::capture_principal_currentness_admitted` |
  | source facts, inspection target, program meaning | 4 + 5 + 2 | `source_facts::allocation::SourceFactsAllocation`, `causal_inspection::…::WorthQueryApplicationInspectionTarget`, `program_occurrence::original_meaning::OriginalProgramMeaning` |
  | workflow actor facts (in the attempt) | 16 | `WorkflowRecordedActorObservation` 10, `workflow::actor::encoding::encode` 4, `WorthQueryWorkflowActorContext` 2 |
  | attempt observation and identity | 9 | `fact::consult_observation` 4, `idempotency::WorthQueryApplicationIdempotencyBinding` 3, `snapshot_lease::WorthQueryApplicationSnapshotLease` 2 |
  | attempt pipeline steps | ~15 | `precondition_binding::{bind_mutation_preconditions, bind_empty_mutation_preconditions_admitted}`, `provider_binding::WorthQueryPreparedApplicationProviderAttempt`, `provider_execution::{outcome::*, phase::…::registered::*, resource_request::application_resource_request, application_attempt_affinity::WorthQueryApplicationAttemptBasis}`, `compare_and_commit::commit_outcome::WorthQueryApplicationCommitOutcome`, `elevation_approval_outcome::WorthQueryElevationApprovalOutcome`, `IdempotencyResolutionScopedOps`, `ElevationApprovalReplayScopedOps` |
  | retention, demand, provider facts, query | 6 + 3 + 5 + 2 | family A items; `RequiredOutputDemandContext`; `provider::application_decision_fact::*`; `WorthQueryApplicationQueryAccessContext` |

  Sources (current): `operation_admission/**` 83, `application_disclosure/**` 34,
  `decision_facts/**` 33, `elevation_progression/**` 19, `retained_capability_request/**` 12.
  Two of the 36 remaining runtime bridges are here:
  `delegation_progression/replay.rs:192`, `elevation_progression/approval/replay.rs:145`.
- **Two different kinds of crossing.** (a) Admission reads the primary graph's lowest
  vocabulary: layout, principal, entity identity, freshness. These are the bottom layers of
  primary_graph's own draft stack (`identity_keys_and_refusals`, `schema_and_native_facts`,
  `principals_and_resolution`), and `provider_session` reads the same items (its seam lists
  layout, inspection target, snapshot lease, source-facts allocation). (b) The progression code
  (elevation and delegation replay, provider commit authorization) drives attempts. That is
  orchestration above both domains (domain structure law 11), currently written inside the
  authority.

### Is the foundational vocabulary liftable? (checked)

Outgoing primary-graph edges of the candidate modules (current):

| module | lines | depends on (within primary_graph) |
|---|---|---|
| `schema_layout` | 5,768 | `denial` 35, `application_attempt` 3 (`workflow::schema::layout::WorthQueryWorkflowLayout`, `lower_workflow`), `application_discovery` 2 |
| `denial` | 415 | `application_contribution` 1 |
| `authenticated_principal` | 281 | `freshness` 1 |
| `freshness` | 371 | `observations` 5, `resolution_denial` 4, `application_attempt` 3, `schema_layout` 3, `authenticated_principal` 1 |
| `entity_resolution` | 1,567 | `entity_resolution_denial` 20, `resolution` 6, `product_operation` 5, `application_query` 4, `schema_layout` 3, … |
| `source_facts` | 1,133 | `structural_walk` 6, `derived_source` 4, `application_attempt` 3, `admitted_capture` 2, `invariant_projection` 2, `schema_layout` 1 |

`schema_layout` + `denial` + the attempt's workflow schema layout is a closed family once the
workflow layout (`pg/application_attempt/workflow/schema/`) moves with it (it is part of the
graph's layout; `application_discovery` reads it too, which also settles most of the greedy
`application_discovery` → attempt seam) and the one `denial` → contribution reference is cut.
`authenticated_principal` is nearly free. `entity_resolution` is not: it reaches product
operation and query, so only its identity types (`resolved::WorthQueryApplicationEntityIdentity`,
`WorthQueryResolvedEntity`) lift, not the module.

### Recommended structure

Neither nest nor merge. Split the crossing by kind:

1. **Lower the foundational vocabulary** to domain-level siblings below `provider_session` and
   `authorization` (in the draft domain stack, between `execution_resources` and
   `sessions_authority_and_aftermath`):
   - `domain_computation::graph_layout`: `schema_layout` (whole), primary graph `denial` (whole:
     it is the installation refusal the layout returns), the workflow schema layout and
     `lower_workflow`.
   - `domain_computation::application_principal`: `authenticated_principal` (whole), the
     principal currentness evidence types of `freshness` that authorization reads
     (`WorthQueryDurablePrincipalCurrentness`, `WorthQueryPrincipalFreshnessEvidence`), and the
     entity identity values (`WorthQueryApplicationEntityIdentity`, `WorthQueryResolvedEntity`)
     if their impls name nothing above (verify).
   - snapshot lease, inspection target and source-facts allocation follow when their blockers
     are cut (the lease's impl names the layout: it follows the layout; the inspection target's
     impls name causal inspection's `InspectionOriginal` / `SourceOriginal`: contract owned by the
     lowered module, `InspectionOriginalBytes`, implemented in causal inspection).
   Names are by what each owns; primary_graph keeps importing them, now downward.
2. **Move the attempt-driving steps up** (authority transfer). Elevation-approval replay,
   delegation replay and the attempt parts of provider commit authorization move out of
   authorization into the primary graph, beside what they drive
   (`pg/application_attempt/elevation_approval_replay`, which already exists, and a sibling for
   delegation). Authorization keeps the decision: functions that take the observed data and
   return its typed refusals (`WorthQueryOperationAuthorizationDenial`, `…DenialKind`,
   `…RefusalKind`) unchanged. This also removes the two runtime bridges listed above.
3. **What remains** — workflow actor facts (16), attempt observation and identity (9),
   retention (family A), `RequiredOutputDemandContext` (family D), provider decision facts —
   is handled by the families in `contracts.md` and by lowering the workflow actor
   observation types with the workflow schema layout (step 1).

### Migration

1. Owner decision on the two new domain-level module names and their place in the domain stack.
2. `graph_layout`: move `schema_layout`, `denial`, `application_attempt/workflow/schema/` with
   `module_move.py` (no shims); cut `denial` → contribution (1 ref) and `schema_layout` →
   discovery (2 refs) first. Typed refusal `WorthQueryPrimaryGraphInstallationDenial` and its
   kinds move unchanged; the facade re-exports point at the new path.
3. `application_principal`: same, for the principal family.
4. Snapshot lease, inspection target, source-facts allocation, each after its blocker.
5. Move the progression steps up (step 2 above), one progression at a time, with the courts
   in `worth-query-court-execution-primary-graph-authorization` after each.
6. Re-measure at both levels; `provider_session` → primary_graph should lose layout, inspection
   target, snapshot lease and allocation (13 of its 22 references) at the same time.

Size: ~7,000 lines moved (mostly `schema_layout`), several hundred import lines across
primary_graph, authorization and provider_session; progression moves are ~15 files. Careful
design; the layout move is mechanical once its three references are cut.

---

## 3. `convergence_epoch` ↔ `managed_run` (domain level)

### Evidence

- **No cycle in the source.** Current: `managed_run` → `convergence_epoch` **0** references;
  `convergence_epoch` → `managed_run` 119 (admitted, running, yielded, terminal and readmission
  types, e.g. `iteration_owner/direct/association/{admit,completion,step,readmission/*}.rs`).
  convergence_epoch consumes managed runs; nothing in managed_run names an epoch (the
  `terminate_for_convergence` methods in `managed_run/{direct,workflow}.rs` take a terminal kind,
  not an epoch type; `bind_convergence_candidate_evidence` returns domain evidence-binding types).
- **Where the 32/32 seam comes from.** The greedy layering places convergence_epoch below
  managed_run, so the simulation moves **66 managed_run units down into convergence_epoch**
  (`WorthQueryRunningDirectRun`, `WorthQueryAdmittedWorkflowRun`, the yield, cleanup and
  readmission types, …). Those moved units' references back into managed_run are the seam.
- **Why the greedy layering puts it low.** The only edge into convergence_epoch is
  `provider_session` → `convergence_epoch`: 2 references to `WorthQueryConvergenceDomainProvider`
  (`provider_session/graph_provider/bounded_step/provider_anchor.rs:147,183` hold and return
  `Option<Arc<dyn WorthQueryConvergenceDomainProvider>>`; `provider_installation.rs:54-61,217-231`
  coerce the installed provider into it in `install_convergent` and
  `install_convergent_invariant_capable`). That edge puts convergence_epoch into the component
  with provider_session and managed_run.
- **The draft stack agrees.** `stack-domain_computation.json` puts convergence_epoch in
  `convergence_and_attempts`, above primary_graph; the draft-stack run has no
  convergence_epoch ↔ managed_run seam.

### Recommended structure

convergence_epoch above managed_run (and above provider_session). Not one responsibility:
managed_run owns run admission, execution and terminal custody; convergence_epoch owns the
iteration owner that repeats runs until a domain decision. Break the single real edge.

**The port belongs with the other provider ports.** `WorthQueryConvergenceDomainProvider`
(`convergence_epoch/domain_provider.rs`) is a capability the installed graph provider implements,
installed with the provider anchor, invoked under panic isolation, failing with
`WorthQueryConvergenceDomainFailure`. That is the same authority, lifecycle and failure topology
as the decision-fact, provisional and invariant-execution ports the anchor already holds
(`provider_session/graph_provider/bounded_step/semantic_provider_ports.rs`). Its signature
vocabulary is self-contained (checked): `domain_assessment.rs` (44 lines; needs
`WorthQueryBoundGraphExecutionReceipt` from `provider_session/graph_provider/receipt.rs` and the
admission crate's `WorthQueryAdmittedConvergenceContract`), `comparison.rs` (105),
`domain_decision.rs` (168), `provider_families.rs` (113), `identity_validation.rs` (10),
`incumbent.rs` (`WorthQueryRetainedConvergenceCandidateEvidence`, 33),
`domain_work.rs` (`WorthQueryConvergenceDomainFailure` and the work evidence, 100). About 600
lines that name nothing in managed_run or the iteration owner.

Move that port family to `provider_session::graph_provider::convergence_port` (sibling of
`semantic_provider_ports`). The epoch (admission, iteration owner, terminal, cleanup, report,
denials) stays in convergence_epoch.

Alternatives rejected:
- keep the port in convergence_epoch and erase it in the anchor as `Arc<dyn Any>`: convergence
  admission would downcast (proposal 03 option C, outside the rulings);
- move the convergence capability out of the anchor into a second retained anchor on
  `WorthQueryInstalledGraphParticipationAuthority` (`worth-query-installation/src/installed_graph_participation.rs:176`,
  which already retains one `Arc<dyn Any>` anchor): a cross-crate change with a second registry
  for one capability;
- nest or merge: managed_run has no reference to convergence_epoch, so there is nothing to
  unify; merging would give run custody and the iteration owner one failure topology.

### Migration

1. Record convergence_epoch above managed_run and provider_session in the governing domain order.
2. Move the eight files above to `provider_session/graph_provider/convergence_port/`
   (`module_move.py`, no shim). Constructors that are `pub(super)` today
   (`WorthQueryConvergenceAssessment::new`, `WorthQueryRetainedConvergenceCandidateEvidence::new`,
   `WorthQueryConvergenceDomainWorkEvidence::empty`) are called by the iteration owner: they
   become `pub(in crate::domain_computation)` (restricted, not `pub`).
3. Public paths: the consumer facade (`worth_query_execution::facade::convergence_epoch::*`,
   used by `worth-query/src/domain_installation/graph_participation/registry{,/capability_registration}.rs`)
   re-exports from the new location; the consumer-facing names do not change.
4. The anchor's field and `install_convergent*` constructors are unchanged except for the path
   of the trait.
5. Re-run the domain-level simulation with convergence_epoch above managed_run; expect the seam
   to vanish and no managed_run unit to move into convergence_epoch.

Typed refusals preserved: `WorthQueryConvergenceDomainFailure`, `WorthQueryConvergenceEpochDenialKind::MissingConvergenceProvider`
(`iteration_owner/*/association/admit.rs:176,183`) and the families-mismatch refusal are
untouched.

Size: 8 files (~600 lines) moved, ~25 import lines, 3 visibility re-scopes. Mechanical once the
layering is recorded.

Risk: the in-flight helper moves follow the greedy layering, which moves the 66 managed_run
units into convergence_epoch at the domain level. Do not apply domain-level moves into
convergence_epoch until the layering is decided.
