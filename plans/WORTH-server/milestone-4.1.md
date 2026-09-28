# Milestone 4.1: Canonical Product Results And Durable Product Mutation Join

## Goal

Make every successful product operation publish one canonical, schema-versioned
result artifact, and add the product-owned durable mutation boundary required by
host connections, admitted manifests, deployments, and other durable product
truth.

## Why This Milestone Exists

Milestone 4 closes the product operation runtime, but its success surface still
publishes only a result key and adapter-supplied digest, while product mutation
authority remains draft/session scoped. Downstream products therefore need
side-channel result stores and cannot admit durable product mutation honestly.

Milestone 4.1 closes those prerequisites before lease and sync work builds on
the product operation runtime.

## Governing Summaries

- `MENTALITY.md` protects the hard foundation and the single canonical artifact.
  This milestone must close the crash boundary before shipping durable product
  features and must not preserve a result side store as ordinary architecture.
- `arch_laws.md` protects typed phase progression, proof-carrying authority,
  self-describing envelopes, explicit failure topology, and facade ownership.
  Result validation and durable conclusions must be types consumed by later
  phases rather than conventions rediscovered during execution.
- `composition_laws.md` protects named responsibilities. Result contracts,
  canonicalization, durable attempts, conclusions, recovery, and transport
  projection must not accumulate in one outcome or runtime-support file.
- `domain_structure_laws.md` protects truth-source and authority separation.
  Product result publication is derived server output; durable product state,
  basis comparison, mutation legality, and atomic completion remain owned by
  the installed product persistence boundary.
- `perf_laws.md` protects bounded work and scoped coordination. Canonicalization
  occurs once in `O(result bytes)`, idempotency lookup is indexed by declared
  identity, and scheduler locality may not become a global mutation lock.
- `Worth_server_roadmap.md` protects sequencing and the server's role as a
  network operation runtime rather than a second truth engine. This milestone
  extends the Milestone 4 product runtime before later lease and sync work.

The user-provided implementation plan also explicitly requires the Worth Server
vision, platform glossary, Store durability posture, and server test
requirements to shape this milestone. Those sources additionally protect the
Query-first ownership model, reserve replay for certification reconstruction,
require crash-before-ack honesty, and demand narrow hostile proof artifacts.

## Adversarial Constraint

A server process or node may die after a product-owned mutation changes durable
truth but before Worth Server acknowledges the request or records a local
completion. An identical retry after fresh runtime construction must resolve to
the one canonical result without a second mutation. A stale basis, cross-tenant
key collision, or key reuse with different semantic meaning must perform zero
mutation and must not disclose another request's completion.

At the same time, a successful result body may be large, differently ordered at
the JSON authoring boundary, or projected through different transports. Every
surface must expose the same schema identity, canonical body meaning, and
artifact digest without recanonicalizing or consulting a producer-local result
store.

## Product Decision Lock

- A successful product operation owns a `ProductResultArtifact`; a result key or
  digest alone is not a successful result.
- Output schema identity, version, encoding, canonicalization version, and
  inline-size policy are mandatory declaration-time contracts.
- Product adapters author typed values. Worth Server canonicalizes the erased
  boundary representation once and computes the body and artifact digests.
- Milestone 4.1 admits bounded canonical JSON results. Results exceeding the
  declared inline budget deny before envelope publication. Content-addressed
  binary result publication requires a separately installed durable body
  capability and may not fall back to an arbitrary compatibility store.
- Draft product mutation and durable product mutation are distinct authority
  regimes. Draft mutation remains session-scoped and process-local.
- The request basis is a product precondition, never current authority. The
  authoritative basis comparison occurs inside the product-owned transaction.
- The durable product executor atomically owns idempotency admission, basis
  comparison, mutation, next-basis publication, result artifact persistence,
  and completion persistence.
- Worth Server owns declaration admission, planning, scoped scheduling,
  transport projection, and diagnostics. It never owns durable product truth.
- Ordinary idempotent resolution is named retry, not replay. Replay remains a
  certification-only reconstruction concept.
- Local scheduler lanes reduce conflicting work but do not provide multi-node
  correctness. The durable transaction remains authoritative.

## Phase Plan

### Phase 1: Freeze Result And Durability Registration Contracts

This phase makes result and durability posture registration-time truth instead
of execution-time discovery.

**Relevant subsystems**

- product operation declaration and adapter registry
- server builder and validated runtime assembly
- operation-declared route inventory

**Relevant APIs**

- `WorthServerProductOperationDeclaration`
- `WorthServerProductResultContract`
- `WorthServerDurableProductMutationContract`
- `WorthServerProductDurabilityCapability`
- `WorthServerProductApplicationAdapterRegistration`

**Required build shape**

- Add a result contract to every product operation declaration.
- Add a durable mutation declaration constructor distinct from draft mutation.
- Add explicit idempotency retention and product authority scope types.
- Add an optional durable mutation executor to adapter registration.
- Reject durable declarations when no executor is installed or its capability
  cannot satisfy the declaration.
- Include output contract and durability posture in registration receipts and
  route identity so different versions cannot compare equal accidentally.

**Warnings**

- Do not add defaults that let old declarations register without output truth.
- Do not let a route publish successfully and discover its missing persistence
  capability only on the first request.

**Test requirements**

- `missing_output_contract_denies_registration_before_route_publication` proves
  no route or operation inventory row is produced for an incomplete result
  declaration.
- `durable_declaration_without_atomic_executor_denies_server_build` proves a
  draft-only runtime cannot advertise durable product mutation.
- `durability_capability_mismatch_is_localized_to_registration` proves an
  installed but weaker executor cannot satisfy the declaration by convention.

**Engineering decisions**

- Result schema version begins at nonzero `u32` and is part of artifact identity.
- Idempotency retention is explicit; this milestone does not promise
  exactly-once resolution beyond the declared retention window.

**Open questions**

- None.

### Phase 2: Build Canonical Schema-Bearing Result Artifacts

This phase replaces adapter-authored digests with one server-validated result
publication artifact.

**Relevant subsystems**

- `product_result/contract`
- `product_result/canonicalization`
- `product_result/artifact`
- product adapter success and product operation envelope

**Relevant APIs**

- `WorthServerProductResultSchema`
- `WorthServerProductResultContract`
- `WorthServerProductResultArtifact`
- `WorthServerProductOperationSuccess::json`

**Required build shape**

- Add typed schema identity, version, encoding, canonicalization version, and
  maximum inline byte count.
- Canonicalize JSON object keys recursively and serialize once.
- Compute a SHA-256 body digest from canonical bytes and an artifact digest from
  the complete result contract plus body digest and byte length.
- Retain the canonical JSON body for direct and HTTP projection.
- Validate the returned artifact against the operation declaration before
  envelope construction.
- Remove the constructor that accepts an adapter-supplied result digest.

**Warnings**

- A schema identity string without version and validation is not a typed result.
- Envelope digesting must consume the artifact digest, not debug formatting or
  transport serialization.

**Test requirements**

- `canonical_json_ordering_produces_one_result_artifact` proves differently
  ordered equivalent objects have equal canonical bytes and digests.
- `result_schema_mismatch_denies_before_envelope_publication` proves an adapter
  cannot return a valid artifact under another declaration's contract.
- `oversized_inline_result_is_rejected_without_success_publication` proves the
  declared byte budget is enforced before a success envelope exists.
- `inline_budget_stops_serialization_before_unbounded_materialization` proves
  the budget stops typed serialization itself rather than rejecting only after
  an arbitrarily large JSON tree has already been allocated.

**Engineering decisions**

- Canonical JSON v1 is the admitted structured result encoding for this
  milestone.
- Canonical bytes are retained with the artifact so downstream projection does
  not repeat canonicalization.

**Open questions**

- None.

### Phase 3: Admit Durable Product Mutation Authority

This phase introduces durable product mutation as its own proof-carrying
authority, planning strategy, scope, and scheduler lane.

**Relevant subsystems**

- operation admission metadata, authority kind, scope, and footprint
- operation planning strategy and receipt
- product operation lowering and scheduler admission
- compatibility and Worth-native mutation commands

**Relevant APIs**

- `WorthServerDurableProductMutationContract`
- `WorthServerProductAuthorityScope`
- `WorthServerAdmittedDurableProductMutation`
- `WorthServerOperationAuthorityKind::DurableProductMutation`
- `WorthServerSchedulerLane::DurableProductMutation`

**Required build shape**

- Lower durable declaration authority from the admitted tenant/workspace,
  declared product authority scope, request basis precondition, and mandatory
  idempotency key.
- Add a durable product operation scope distinct from product draft/session
  scope.
- Carry the exact scope and basis into the lowered plan and admitted durable
  attempt.
- Derive scheduler lanes from tenant, workspace, and product authority scope.
- Let durable product mutations enter without creating a product draft session.
- Deny missing basis or idempotency before durable executor invocation.

**Warnings**

- The scheduler lane is conflict localization, not durable correctness.
- A request digest, label, or session identity must never promote itself into
  durable product authority.

**Test requirements**

- `draft_authority_cannot_execute_a_durable_declaration` proves the authority
  kinds are not interchangeable.
- `caller_basis_cannot_mint_current_product_authority` proves stale comparison
  remains inside the durable executor and produces zero mutation.
- `durable_scopes_localize_scheduler_conflicts_without_global_serialization`
  proves equal scopes share a lane while independent scopes do not.

**Engineering decisions**

- Durable authority scope is product-declared and tenant/workspace-bound.
- Product sessions remain coordination artifacts and are not required for
  durable product mutation.

**Open questions**

- None.

### Phase 4: Join Product Mutation And Durable Completion

This phase installs the product-owned transactional port and makes its typed
conclusion the only durable mutation execution path.

**Relevant subsystems**

- `durable_product_mutation/attempt`
- `durable_product_mutation/executor`
- `durable_product_mutation/completion`
- `durable_product_mutation/conclusion`
- `durable_product_mutation/recovery`
- product operation runtime

**Relevant APIs**

- `WorthServerDurableProductMutationExecutor`
- `WorthServerAdmittedDurableProductMutation`
- `WorthServerDurableProductMutationCompletion`
- `WorthServerDurableProductMutationConclusion`
- `WorthServerDurableProductMutationRecoveryHandle`

**Required build shape**

- Build an admitted attempt carrying operation identity, tenant/workspace,
  authority scope, expected basis, idempotency identity, request binding,
  payload, result contract, and lowered plan identity.
- Require the installed product executor to perform idempotency admission,
  current-basis comparison, state mutation, next-basis publication, canonical
  result persistence, and completion persistence atomically.
- Admit typed conclusions for committed, previously committed, stale basis,
  idempotency conflict, product rejection, indeterminate recovery, and failure.
- Validate every committed completion against the admitted attempt before
  publishing it through the server.
- Add recovery resolution through the same registered product executor.
- Re-admit recovery against the current surface, operation-name allowlist, and
  authorization policy before asking product persistence to resolve it.
- Bypass the process-local product retry store for every durable declaration.

**Warnings**

- Executing the product adapter and then recording completion is forbidden for
  durable mutations even if both writes target durable stores.
- Indeterminate work may not be automatically executed again; callers receive a
  typed recovery handle.

**Test requirements**

- `crash_boundary_matrix_resolves_to_one_durable_conclusion` covers failure
  before mutation, after intent, after product mutation, after commit, and
  before acknowledgment.
- `same_key_same_binding_returns_original_completion_without_second_effect`
  proves retry resolution returns the same result artifact and next basis.
- `same_key_different_binding_conflicts_without_product_mutation` pressures
  payload, scope, tenant, operation version, and expected-basis changes.
- `indeterminate_conclusion_requires_recovery_handle_resolution` proves the
  runtime does not hide uncertainty behind a generic failure or blind retry.
- `recovery_rechecks_current_operation_authorization` proves a retained handle
  does not preserve access after the current server policy removes it.

**Engineering decisions**

- The executor is a product-owned facade installed during server construction.
- The semantic completion is persisted; transport envelopes are derived after
  the transaction and are not product truth.

**Open questions**

- None.

### Phase 5: Cut Every Product Surface Over To Canonical Results

This phase makes the canonical result artifact the direct and network-visible
success body and removes ordinary product replay vocabulary.

**Relevant subsystems**

- Worth-native product facade
- compatibility HTTP product facade
- operation-declared Axum response projection
- product operation envelope and diagnostics
- process-local draft retry store

**Relevant APIs**

- `WorthServerProductOperationRetryReceipt`
- `WorthServerProductOperationRetryDiagnostics`
- `WorthServerCompletedProductOperation::result_artifact`
- product route JSON response

**Required build shape**

- Project schema identity, schema version, encoding, canonicalization version,
  body, body digest, artifact digest, and result key on successful HTTP routes.
- Expose the same artifact object through Worth-native completion.
- Replace product `ReplayReceipt`, `ReplayClass`, and replay diagnostics with
  retry terminology and executed/previously-committed distinctions.
- Rename the process-local product operation store as a retry store and restrict
  it mechanically to non-durable declarations.
- Reconstruct retry/recovery transport envelopes from semantic completion plus
  current delivery evidence without persisting an HTTP response.

**Warnings**

- Diagnostics policy may change evidence richness but never result body or
  artifact identity.
- A result key may remain a correlation label but may not be required to fetch
  an ordinary structured result.

**Test requirements**

- `worth_native_and_http_project_identical_result_artifacts` compares exact
  schema, body, body digest, and artifact digest.
- `durable_retry_after_runtime_reconstruction_returns_original_artifact`
  proves semantic result equality without re-executing product mutation.
- `diagnostic_richness_does_not_change_product_result_artifact` proves result
  truth remains independent of forensic materialization.

**Engineering decisions**

- Ordinary product retry is not platform replay.
- Direct and HTTP surfaces derive from the same completed operation rather than
  separate response models.

**Open questions**

- None.

### Phase 6: Certify Product-Shaped Durable Pressure And Remove Displaced Paths

This phase proves the new boundary using host-connection, admitted-manifest,
and deployment-shaped product mutations and closes every local substitute path.

**Relevant subsystems**

- durable product mutation certification harness
- product operation result parity harness
- server runtime certification and counters
- downstream consumer integration boundary

**Relevant APIs**

- product-specific implementations of
  `WorthServerDurableProductMutationExecutor`
- durable mutation counter and certification snapshots
- result artifact parity assertions

**Required build shape**

- Build three product-shaped fixtures with distinct authority scopes, basis
  transitions, idempotency retention, result schemas, and conflict footprints.
- Prove host connection create/update, manifest admission, and deployment
  transition through the production server path.
- Remove adapter-supplied result digests, durable use of the local retry map,
  durable declarations expressed as draft mutation, and product ordinary
  replay types.
- Produce exact counters for durable attempts, basis comparisons, commits,
  previously committed conclusions, key conflicts, stale bases, indeterminate
  conclusions, result bytes, and oversized-result denials.
- Require downstream consumers to remove result side stores before claiming
  end-to-end product closeout. Consumers outside this repository remain an
  explicit external closeout dependency rather than a server-local fiction.

**Warnings**

- A fixture that calls the executor directly does not certify Worth Server.
- Cross-owner workflows must decompose into durable state-machine or outbox
  steps; this milestone does not pretend they are one distributed transaction.

**Test requirements**

- `product_shaped_mutations_survive_runtime_reconstruction_once` proves each
  specimen returns one durable effect and original result on retry.
- `cross_tenant_and_cross_scope_retry_identity_never_leaks_completion` proves
  isolation at the idempotency and authority boundaries.
- `same_basis_deployments_choose_one_winner_while_independent_targets_progress`
  proves scoped conflict correctness across independent server runtime
  instances, so one process-local scheduler lane cannot supply the proof.
- `declared_retention_controls_retry_resolution_honestly` proves conclusions
  are not promised beyond the installed product retention policy.

**Engineering decisions**

- Test-only in-memory providers are labeled fixtures and may certify the server
  protocol plus their bounded atomic test transaction. They do not, by
  themselves, certify a production persistence deployment; end-to-end closeout
  requires a real product implementation of the same capability.
- The integration-test root is exempt from the ten-file directory default
  because Cargo discovers integration crates only at that root; new harness
  implementation remains organized in responsibility-named subdirectories.

**Open questions**

- Which downstream repository owns `WorkflowHostPreviewResultStore` and the
  compatibility middleware to be deleted after this server contract lands?

## Must Ship

- mandatory product result contracts and canonical JSON result artifacts
- server-computed body and artifact digests with declared inline budgets
- durable product mutation declaration, authority, scope, plan, and scheduler lane
- product-owned atomic durable mutation executor and typed recovery contract
- product retry terminology and durable bypass of the process-local retry store
- Worth-native and compatibility HTTP result-artifact parity
- host-connection, manifest-admission, and deployment-shaped certification

## Must Preserve

- product domains own result meaning and mutation legality
- product persistence owns durable state, current basis, and atomic completion
- Worth Server owns admission, planning, scheduling, envelopes, transport, and
  diagnostics without becoming a product database
- product sessions remain distinct from durable product authority
- canonical result meaning remains independent of diagnostics and transport
- ordinary retry remains separate from cert-only replay

## Acceptance Evidence

- missing result or durability capabilities deny before route publication
- canonical JSON equivalence, schema mismatch, and size-budget hostility are
  proven with exact artifacts
- durable attempts survive every crash edge with one permitted conclusion
- identical retries return one original completion; conflicting bindings perform
  zero product mutation
- stale basis comparison occurs inside the product transaction
- direct and HTTP surfaces expose identical result schema, body, and digests
- scoped concurrency, tenant isolation, exact counters, line caps, boundary
  checks, and generated agent context are green

## Sequencing Notes

Milestone 4.1 follows Milestone 4 because it consumes the completed product
operation declaration, request, admission, planning, scheduler, and route
runtime. It precedes Milestone 5 because later server-managed product surfaces
must not depend on result side stores or draft authority for durable product
truth.

Milestone 10 remains responsible for the broader network mutation protocol,
branch-aware optimistic confirmation and rollback, and richer provenance. It
must consume Milestone 4.1 result and durable product mutation contracts rather
than inventing its first product durability lane.
