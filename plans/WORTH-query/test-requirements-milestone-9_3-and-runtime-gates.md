## Milestone 9.3 Named Certification Suites

### 9.3. Query Subscription Bridge Parity And Diagnostic Sufficiency Test

Purpose

Prove that every admitted automatic query subscription family is
bridge-explainable, support-reportable, offline-diagnosable, and
runtime-certified without inventing hidden semantics beyond canonical query,
bridge, signal, and runtime lifecycle artifacts.

Scenario

- use the concrete `EmployeeRecord` fixture from Milestones 9.1 and 9.2
- derive admitted automatic subscription families for:
  - detail exact
  - inspector detail exact
  - ordered collection membership
  - grouped collection membership
  - bounded materialization where admitted
- assemble for each admitted family:
  - support report for explicit declaration/lifecycle/preview support subjects
  - support lookup receipt
  - manual bridge witness
  - bridge parity explanation
  - bridge parity receipt
  - lifecycle certification bundle
  - offline admitted diagnostic bundle
  - diagnostic assembly receipt
  - runtime family certification bundle
  - certification coverage receipt
- exercise:
  - declaration-family-preserving lifecycle closure
  - grouped and inspector families that share lower bridge classes while
    remaining query-side distinct
  - continuation and preview evidence carried into diagnostic bundles
  - hostile support, bridge-parity, and certification-coverage denials
- vary:
  - masked and unmasked policy digests
  - tenant truth/schema basis digests
  - relationship-proof digests
  - view-shape digests
  - current, branch-head, runtime snapshot, and preview-scoped bases
  - admitted and denied family coverage sets

Required concrete lanes

- detail support lane where admitted declaration, bridge lowering, lifecycle
  certification, and diagnostic bundle all bind the same declaration and bridge
  digests
- grouped family lane where grouped query-side meaning remains distinct from an
  ordered collection family even if the bridge family is shared underneath
- preview certification lane where preview discard or promotion evidence is
  surfaced in the diagnostic bundle and family certification scope
- support-overclaim denial lane where runtime-backed support is requested for a
  family lacking required hostile coverage
- denied diagnostic bundle lane where declaration or bridge denial is still
  emitted as one offline-readable denied bundle without fake lifecycle slots
- bridge-parity mismatch lane where a foreign declaration or signal strategy
  source is supplied and parity fails before a certification bundle exists

Must verify

- support reporting is query-family-aware and does not collapse distinct query
  subscription families into shared bridge-family claims
- runtime-backed support cannot be claimed for uncertified families
- support reporting is phase-typed and distinguishes declaration support,
  active lifecycle support, continuation support, preview-closeout support, and
  deferred durable/store-backed support
- support reporting exposes exact lookup posture and lookup width through a
  receipt; indexed lookup is admitted, while broader scans are explicit debt or
  denial
- store-backed restart, durable replay, and persisted continuation claims
  remain explicit deferred or denied support surfaces
- every admitted automatic family has one bridge-facing explanation binding
  query family, declaration, bridge family, bridge slices, basis posture, and
  signal strategy
- every bridge-facing explanation is bound to one tangible manual bridge
  witness describing the host-equivalent bridge request that Query claims to
  automate
- bridge parity is comparison over pre-lowered or canonically composed witness
  artifacts; witness rebuild or semantic rediscovery after witness construction
  is explicit debt or denial
- grouped and inspector families remain mechanically distinct in support,
  diagnostics, and certification artifacts even when their bridge lowerings
  share infrastructure below Query
- diagnostic bundles are sufficient for offline explanation of admitted and
  denied paths without re-running Query or consulting hidden host state
- admitted and denied diagnostic bundles are different proof types rather than
  one optional-hole envelope
- admitted and denied bundle assembly emit receipts proving stage evidence and
  semantic labels were composed rather than re-derived
- diagnostic traces localize whether failure occurred during family selection,
  declaration, bridge lowering, support reporting, lifecycle certification,
  bridge parity, or coverage closure
- declaration-family drift remains distinct from lifecycle-instance churn such
  as lane handle, attachment, delivery sequence, continuation epoch, or preview
  closeout changes
- runtime family certification requires at least one admitted row and one
  hostile row for each supported family
- runtime family certification also requires representative basis, policy,
  tenant, relationship-proof, view-shape, and lifecycle-class variation rows
  for each supported family
- runtime family certification consumes family-scoped indexed coverage handles
  or explicit matrix-scan debt/denial posture; raw row iteration is not an
  invisible implementation choice
- support, bridge parity, lifecycle certification, and runtime family
  certification bind the same canonical query/declaration/bridge digests
- compile-fail boundaries prove external callers cannot mint support reports,
  bridge parity explanations, diagnostic bundles, or runtime family
  certification bundles directly
- small/medium/larger fixture runs prove support and bundle assembly cost
  slopes are bounded by family coverage width, diagnostic bundle width, and
  indexed coverage width rather than unrelated row count

Required verification output

- `query_digest`
- `subscription_family_digest`
- `subscription_declaration_digest`
- `subscription_equivalence_digest`
- `bridge_declaration_digest`
- `bridge_basis_digest`
- `signal_strategy_digest`
- `support_report_digest`
- `support_matrix_digest`
- `support_lookup_receipt_digest`
- `manual_bridge_witness_digest`
- `bridge_parity_digest`
- `bridge_parity_receipt_digest`
- `diagnostic_trace_digest`
- `admitted_diagnostic_bundle_digest`
- `denied_diagnostic_bundle_digest`
- `diagnostic_assembly_receipt_digest`
- `lifecycle_certification_digest`
- `runtime_certification_bundle_digest`
- `certification_coverage_receipt_digest`
- `continuation_digest`
- `preview_isolation_digest`
- `failure_digest`
- `counter_snapshot`
- `subscription_support_scale_slope_digest`
- `compile_fail_boundary_digest`

Pass condition

Automatic subscription family selection remains bridge-honest, support claims
remain synchronized with certified runtime behavior, diagnostic bundles remain
offline-sufficient, and unsupported or uncertified family claims fail-closed
before store-backed or durable milestones build on top of them.

## Milestone 9.3.1 Named Certification Suites

### 9.3.1. Cross-Runtime Causal Explanation Envelope Test

Purpose

Prove that Query inspection can expose one bridge-owned cross-runtime causal
explanation envelope for query-observed outcomes, anchored to the Query
operational artifact that produced the observation, without requiring
downstream domains to stitch runtime bridge diagnostics, relational runtime
evidence, and signal graph internals directly.

Scenario

- use a concrete fixture with:
  - a relational truth mutation that changes a query-observed result
  - a relational truth mutation that is causally relevant but suppressed by
    query shape, policy, tolerance, or result equivalence
  - a branch or preview mutation whose basis must remain explicit
  - a historical or replayed observation whose materialization path must remain
    explicit
- ask Query inspection for causal explanations of changed, suppressed, denied,
  branch/preview, and replayed observations
- derive a causal observation anchor and evidence-reference set from each Query
  operational artifact before inspection admission
- prove the ordered progression with proof-bearing phase artifacts so request,
  reference, admission, bridge-envelope, and materialization phases cannot be
  skipped
- assemble one bridge-owned causal explanation envelope for each admitted
  or advisory inspection request
- materialize one Query-owned causal inspection artifact from each admitted or
  advisory bridge envelope
- exercise hostile missing-evidence lanes for missing bridge route evidence,
  missing signal evidence, incompatible relational authority, policy-redacted
  detail, and unsupported explanation families
- include a Worth-style consumer lane that is allowed to consume only Query
  inspection artifacts, not runtime bridge diagnostics facade handles,
  relational runtime internals, or signal graph internals

Required concrete lanes

- changed-result lane where relational authority, bridge route/evaluation,
  bridge source/structural/preview/writeback evidence where applicable, signal
  invalidation/evaluation, signal forensic availability, lineage, provenance,
  replay posture, and query observation digests all bind the same query
  observation anchor
- no-op/suppression lane where the causal chain remains referenceable even
  though the query-shaped public result does not change
- advisory-redaction lane where the causal explanation is valid but detail is
  narrowed with a success/advisory/violation admission trace rather than
  collapsed into a binary admitted/denied result
- branch/preview lane where basis identity and preview closeout posture remain
  visible in the causal envelope
- replay lane where materialization path and replay posture are explicit and
  distinguished from live invalidation
- missing-signal-evidence denial lane where the bridge route exists but signal
  invalidation or evaluation evidence cannot be admitted
- missing-bridge-route denial lane where signal or relational evidence alone is
  insufficient to mint a public causal explanation
- policy-redaction lane where richer narrative detail is denied or narrowed
  without changing causal digests or query meaning
- public-boundary lane where domain code that tries to construct the ordinary
  explanation through lower-runtime imports fails at the certified boundary

Must verify

- Query owns the public causal inspection request, admission record, redaction
  posture, result-shape context, causal observation anchor, evidence-reference
  set, and materialized inspection artifact
- runtime bridge owns the cross-runtime causal explanation envelope and names
  which lower runtime supplied each evidence family
- relational remains the authority for truth, commits, snapshots, and
  relational decision evidence
- signal remains the authority for observation, invalidation, scheduling,
  signal lineage, and signal provenance evidence
- bridge, relational, and signal evidence digests agree with the lower-runtime
  records they summarize
- Query evidence references resolve through existing lower-runtime records,
  Query inspection evidence, signal forensic availability, and relational
  authority identities rather than a new Query-owned diagnostics store
- changed, suppressed, denied, branch/preview, and replayed observations all
  receive typed artifacts rather than optional holes in one loose structure
- success, advisory, and violation outcomes are represented as typed admission
  decisions with decision-trace indexes
- worth-proof or an equivalent proof-bearing substrate enforces phase ordering,
  authority witnesses, fixed-shape/canonical evidence-reference collections,
  and trust-boundary readmission between Query and runtime bridge
- missing evidence fails as typed diagnostic denial and redacted/narrowed
  evidence becomes typed advisory detail rather than best-effort prose
- diagnostic richness changes only cold-path materialization detail, not query
  meaning, bridge route meaning, signal invalidation meaning, or relational
  authority
- hot-path query execution and signal invalidation emit only bounded evidence
  references, counters, and digests needed for later cold-path explanation
- ordinary downstream domains can explain query observations through Query
  inspection without direct imports from runtime bridge diagnostics,
  relational runtime internals, or signal graph internals
- small/medium/larger fixture runs prove causal inspection cost slopes are
  bounded by evidence-reference width, requested richness, and admitted
  materialization width rather than unrelated runtime graph size or total
  bridge diagnostics retention width
- performance counters are reported separately for observation anchor
  derivation, evidence-reference resolution, inspection admission, bridge
  envelope assembly, redaction, materialization, and artifact serialization

Required verification output

- `query_digest`
- `query_observation_receipt_digest`
- `causal_observation_anchor_digest`
- `inspection_target_digest`
- `inspection_request_digest`
- `admitted_inspection_digest`
- `advisory_inspection_digest`
- `denied_inspection_digest`
- `causal_envelope_digest`
- `causal_evidence_reference_digest`
- `decision_trace_index_digest`
- `relational_authority_digest`
- `bridge_route_digest`
- `bridge_evaluation_digest`
- `bridge_source_materialization_digest`
- `bridge_structural_digest`
- `bridge_stream_digest`
- `bridge_preview_digest`
- `bridge_writeback_digest`
- `signal_invalidation_digest`
- `signal_evaluation_digest`
- `signal_forensic_availability_digest`
- `signal_lineage_digest`
- `signal_provenance_digest`
- `query_observation_digest`
- `materialization_policy_digest`
- `replay_posture_digest`
- `redaction_policy_digest`
- `admitted_causal_inspection_artifact_digest`
- `advisory_causal_inspection_artifact_digest`
- `denied_causal_inspection_artifact_digest`
- `causal_materialization_receipt_digest`
- `causal_inspection_proof_shape_digest`
- `causal_inspection_phase_progression_digest`
- `causal_inspection_witness_authority_digest`
- `causal_evidence_reference_collection_proof_digest`
- `causal_bridge_readmission_proof_digest`
- `failure_digest`
- `counter_snapshot`
- `causal_inspection_scale_slope_digest`
- `causal_anchor_derivation_slope_digest`
- `causal_reference_resolution_slope_digest`
- `causal_admission_slope_digest`
- `causal_bridge_envelope_slope_digest`
- `causal_materialization_slope_digest`
- `causal_artifact_serialization_slope_digest`
- `compile_fail_boundary_digest`

Pass condition

Query inspection exposes one authority-preserving cross-runtime causal
explanation surface, changed and non-changed outcomes are equally explainable,
missing evidence fails closed, redacted evidence is represented as typed
advisory/narrowed detail, and domain consumers no longer need direct
lower-runtime stitching to explain why query-observed outcomes happened.

## Milestone 9.3.2 Named Certification Suites

### 9.3.2. Query Basis Capability Lifecycle Test

Purpose

Prove that Query basis is a phase-typed capability lifecycle rather than a raw
branch, preview, tenant, policy, snapshot, historical, or runtime identifier.
Every observation, mutation-preparation, replay, inspection, materialization,
subscription, and certification surface must consume admitted basis capability
proof or fail typed before constructing operational artifacts.

Scenario

- use concrete runtime-backed fixtures with current-head, branch-head, explicit
  branch snapshot, preview, preview-derived, runtime snapshot, historical,
  tenant-scoped, and policy-scoped basis intents
- construct equivalent basis intents through at least two public or
  compatibility paths and prove they normalize to the same capability envelope
- construct intentionally different basis meanings and prove their declared
  digest fields diverge
- consume admitted basis capabilities through observation, mutation
  preparation, replay, inspection, materialization, subscription declaration,
  subscription activation, preview closeout, and certification lanes where
  those operation families are admitted
- compile golden DX transcripts for current-head observation, branch-head
  mutation preparation, inspection with lower-runtime evidence materialization,
  support discovery, and typed denial handling
- prove common-path callers can express basis intent, choose an operation lane,
  execute the admitted operation, inspect support posture, and read the basis
  envelope without manually constructing proof internals
- exercise the lifecycle as a linear typestate chain where each transition
  consumes the prior proof type and returns the next proof type or a typed
  denial
- prove operation lanes are unWORTHable lane witnesses or operation-specific
  wrappers rather than booleans, strings, or loose enum flags on one universal
  token
- bind basis capabilities to retained relational, runtime-bridge, and signal
  evidence without letting Query mint lower-runtime authority records
- prove the lower-runtime binding path reuses bridge subscription, truth-view,
  continuity, preview, writeback, and causal-envelope facade artifacts;
  relational branch/head/snapshot/history facade artifacts and
  `RuntimeBridgeRelationalSource`; and signal snapshot, replay, lineage, and
  diagnostic facade artifacts
- emit an API reuse matrix with one row for each basis-adjacent lower-runtime
  surface naming owning crate, owning facade type/function/trait, existing
  authority artifact, Query wrapper type, allowed carried fields, forbidden
  duplicate fields, consuming operation lanes, denial/deferred posture, and
  the test or compile-fail proof that enforces the row
- exercise hostile lanes for stale preview, inaccessible branch, policy mask,
  tenant/schema mismatch, historical replay unsupported, lower-runtime binding
  mismatch, missing signal observation basis, durable reload overclaim,
  temporal deferred, and async/resource deferred requests

Required concrete lanes

- current-head observation lane where canonical current-head intent becomes an
  admitted observation capability and emits a basis use receipt
- branch-head mutation-preparation lane where mutation setup consumes scoped
  basis capability proof rather than a raw branch identifier
- preview closeout lane where stale preview basis drift denies before authority
  execution or closeout artifacts exist
- historical/replay lane where admitted historical posture is distinct from
  durable replay or store-restored snapshot-plus-tail claims
- tenant/policy lane where equivalent scoped intents normalize while policy or
  tenant/schema mismatch denies typed and early
- causal-inspection lane where the 9.3.1 causal observation anchor consumes
  basis capability proof and preserves lower-runtime authority names
- bridge-readmission mismatch lane where returned bridge or relational basis
  evidence conflicts with the admitted Query capability and fails before a use
  receipt exists
- future-neighbor lane where temporal, async/resource, store-backed, and
  durable basis requests report deferred or unsupported posture with zero
  operational residue

Must verify

- raw basis identifiers are not capabilities and cannot reach read, mutation,
  replay, inspection, materialization, or subscription entrypoints directly
- target DX APIs expose ordinary basis authoring through narrow, memorable
  facade methods while keeping expensive or boundary-crossing work visible
  through explicit calls such as admission, execution, lower-runtime evidence
  inclusion, materialization, and certification
- golden DX transcripts remain synchronized with executable admission behavior,
  support metadata, receipts, and self-describing envelopes
- common-path inspection/envelope APIs expose lower-runtime authority bindings
  without requiring caller-side bridge, signal, or relational stitching
- equivalent basis intents normalize to the same `NormalizedBasisIntent` and
  self-describing basis envelope
- intentionally different basis authority, scope, visibility, lifecycle,
  tenant/schema, policy, operation lane, or lower-runtime binding changes the
  relevant digest
- eligibility precedes operational artifact construction for every operation
  lane
- observation, mutation-preparation, replay, inspection, materialization,
  subscription, preview closeout, and certification lanes are distinct
  capability proofs rather than booleans on one raw token
- advisory, success, and violation eligibility outcomes cannot be substituted
  for one another in downstream phase signatures
- plain digests cannot be passed to executable APIs where proof-bearing
  lifecycle types are required
- lifecycle typestate tests prove production APIs cannot skip directly from
  `RawBasisIntent` or `NormalizedBasisIntent` to scoped use, lower-runtime
  binding, receipt, envelope, or certification artifacts without consuming the
  required proof-bearing transition types
- compatibility debt is registered with owner, entrypoint, target lifecycle
  phase, blocking reason, and denial/adapter posture; no unowned compatibility
  bucket may satisfy certification
- denied basis capability artifacts are different proof types from admitted
  basis capabilities
- lower-runtime relational, bridge, and signal evidence is readmitted by digest
  and authority name rather than re-minted as Query-owned facts
- Query does not create twins of bridge `ValidatedSubscriptionBasisBinding`,
  bridge truth-view authority, bridge continuity authority, bridge writeback
  bases, relational `BranchHead`/`SnapshotHandle`/`CanonicalCommitEnvelope`,
  or signal `SignalSnapshotV1`/`SignalCheckpointImage`/`LineageRecord`/
  `ReplayCursor`
- bridge-bound relational truth flows use `RuntimeBridgeRelationalSource`
  instead of Query-side commit/snapshot/history loaders
- any lower-runtime API gap is reported as deferred or unsupported support
  posture rather than patched with a private-module import or parallel Query
  authority model
- every lower-runtime adapter carries only facade-returned identity, digest,
  receipt, denial, counter, support-posture, authority-label, and Query
  lifecycle proof fields; no adapter carries reconstructive lower-runtime
  fields sufficient to replay, restore, re-resolve, or reissue authority-owned
  records
- the API reuse matrix is executable evidence: if a row lists a forbidden
  duplicate field, compile-fail or structural tests prove Query cannot expose
  or construct that duplicate
- support metadata, executable admission, and certification coverage agree for
  admitted, advisory, denied, deferred, and unsupported basis families
- support metadata is derived from executable admission facts or certification
  fails when admission and support rows drift
- temporal, async/resource, store-backed parity, durable reload, and
  restart-stable basis envelopes remain typed deferred or unsupported until
  their owning milestones close
- compile-fail boundaries prove external callers cannot mint normalized basis
  intents, admitted capabilities, scoped bases, receipts, envelopes, support
  rows, certification bundles, or lower-runtime authority witnesses
- small/medium/larger fixture runs prove normalization, eligibility,
  lower-runtime binding, scoped-use construction, receipt emission, envelope
  materialization, support lookup, and certification costs are bounded by basis
  evidence width, operation-lane width, and lower-runtime binding width rather
  than unrelated runtime graph size or retained diagnostics width

Required verification output

- `query_digest`
- `raw_basis_intent_digest`
- `normalized_basis_intent_digest`
- `basis_family_digest`
- `basis_authority_digest`
- `basis_scope_digest`
- `basis_visibility_digest`
- `basis_lifecycle_digest`
- `basis_policy_digest`
- `basis_tenant_schema_digest`
- `basis_operation_lane_digest`
- `basis_eligibility_digest`
- `admitted_basis_capability_digest`
- `denied_basis_capability_digest`
- `scoped_basis_digest`
- `basis_use_receipt_digest`
- `basis_envelope_digest`
- `relational_basis_authority_digest`
- `bridge_basis_authority_digest`
- `signal_basis_authority_digest`
- `lower_runtime_basis_binding_digest`
- `basis_readmission_proof_digest`
- `basis_target_dx_digest`
- `basis_golden_transcript_digest`
- `lower_runtime_api_reuse_matrix_digest`
- `adapter_shape_contract_digest`
- `typestate_transition_digest`
- `lane_witness_digest`
- `phase_artifact_manifest_digest`
- `compatibility_debt_registry_digest`
- `basis_transition_digest`
- `basis_support_matrix_digest`
- `basis_future_neighbor_denial_digest`
- `basis_proof_shape_digest`
- `basis_phase_progression_digest`
- `failure_digest`
- `counter_snapshot`
- `basis_normalization_slope_digest`
- `basis_eligibility_slope_digest`
- `basis_lower_runtime_binding_slope_digest`
- `basis_scoped_use_slope_digest`
- `basis_receipt_slope_digest`
- `basis_envelope_materialization_slope_digest`
- `basis_support_lookup_slope_digest`
- `compile_fail_boundary_digest`

Pass condition

Every runtime-backed Query basis consumed by ordinary public surfaces is
admitted, denied, or deferred through one phase-typed capability lifecycle;
raw lower-runtime identifiers cannot act as capability tokens; lower-runtime
authority remains relational/bridge/signal-owned; and later temporal,
async/resource, store-backed, and durable milestones inherit this lifecycle
instead of adding parallel basis APIs.

## Milestone 9.3.3 Named Certification Suites

### 9.3.3. Authority-Scoped Effect Execution Pipeline Test

Purpose

Prove that every admitted Query effect executes through one authority-scoped,
proof-bearing pipeline rather than letting executors rediscover basis,
authority family, strategy identity, invariant scope, preview posture, or
artifact policy at execution time.

Scenario

- use concrete runtime-backed fixtures covering direct authoritative mutation,
  ordered batch execution, branch-local preview mutation, merge execution,
  query-triggered bridge writeback, and explicit typed denial families
- require the concrete workflow lowering chain to stay visible:
  - `WorkflowContextBinding`
  - `QueryWorkflowDeclaration`
  - `LoweredMutationIntentDeclaration`
  - `LoweredMergeWorkflowDeclaration`
  - `QueryWritebackDeclaration`
- construct equivalent effect intents through at least two public or
  compatibility authoring paths and prove they normalize to the same lowered
  execution plan and receipt meaning
- construct intentionally different effect families, basis postures, authority
  lanes, or strategy identities and prove the declared digest fields diverge
- require every effect family to consume an admitted 9.3.2 basis capability
  rather than raw branch, preview, tenant, policy, or historical identifiers
- compile golden DX transcripts for ordinary mutation execution, bridge-backed
  writeback execution, typed denial handling, support discovery, inspectable
  lowered-plan inspection, batch-native execution, and receipt/envelope
  inspection
- exercise the lifecycle as a linear typestate chain where each transition
  consumes the prior proof type and returns the next proof type or a typed
  denial
- prove relational execution families lower only through relational facade
  authority and bridge-backed execution families lower only through bridge
  facade authority
- prove the executor accepts only `LoweredEffectExecutionPlan` and cannot be
  called with raw intent, normalized intent, eligibility, or merely
  authority-scoped plans
- exercise hostile lanes for stale basis, preview-read-only execution,
  advisory-only promotion, authority-family mismatch, host strategy override,
  unsupported effect family, lower-runtime lowering mismatch, durable replay
  overclaim, and store-backed execution overclaim

Required concrete lanes

- branch mutation lane where an admitted branch basis lowers one mutation or
  merge family into a relational execution plan and emits one effect receipt
- relational mutation lane where Query lowering emits
  `RawStrategyCommitRequest` through `LoweredMutationIntentDeclaration`
  instead of reconstructing strategy identity inside the executor
- relational merge lane where Query lowering emits `MergeExecutionRequest`
  through `LoweredMergeWorkflowDeclaration` instead of letting merge authority
  rediscover branch pairing or intent semantics
- preview denial lane where preview-read-only or stale preview posture denies
  before any executor, mutation batch, or receipt exists
- bridge writeback lane where one admitted query-triggered writeback lowers
  into `BridgeWritebackDeclaration`, then bridge contract/effect/idempotence/
  request/receipt authority, without Query inventing writeback protocol meaning
- batch lane where one ordered multi-component effect preserves aggregate
  authority, component result posture, and execution counters through one
  receipt family
- batch-lane denial where mixed authority lanes or mixed basis lanes fail before
  a batch execution artifact exists
- relational oracle lane where the final Query mutation or merge receipt is
  checked against independently inspected relational authority state and
  authority artifacts rather than only against another Query-produced digest
- bridge oracle lane where the final Query writeback receipt/envelope is
  checked against independently inspected bridge authority outcome and
  `TruthWritebackReceipt`
- host-override denial lane where a hostile caller tries to replace the
  admitted authority family or strategy after lowering and fails typed
- preview-rebind lane where preview-derived writeback or mutation cannot
  silently execute and instead returns typed denial or explicit rebind posture
- stale-after-admission lane where basis admission succeeds, lower-runtime truth
  changes, and subsequent lowering or execution denies or rebinds exactly where
  the contract says it must
- stale-after-lowering lane where a lowered artifact is retained across an
  authority-changing perturbation and replay/execution proves the expected
  denial, mismatch, or authoritative divergence artifact
- replay/deferred lane where durable replay, store-backed execution, or
  restart-stable effect envelopes remain typed deferred or unsupported with
  zero operational residue
- seeded-random lane where multiple effect authoring paths, family choices,
  batch widths, preview/rebind perturbations, and denial/deferred neighbors are
  generated from a fixed seed and replayed with identical canonical outputs

Must verify

- raw effect intents are not executable permission and cannot reach executors
  directly
- basis capability proof from Milestone 9.3.2 is mandatory input to effect
  eligibility and lowering
- equivalent effect authoring paths normalize to the same
  `NormalizedEffectIntent`, authority-scoped plan, lowered execution plan, and
  execution receipt meaning
- equivalent Query-produced artifacts also match independently observed lower-
  runtime truth outcomes for mutation, merge, and writeback lanes
- intentionally different authority family, strategy identity, basis posture,
  effect family, or artifact policy changes the relevant digest
- effect eligibility precedes construction of lower-runtime execution packets
- advisory, denied, deferred, and admitted effect postures are distinct proof
  families that cannot be substituted for one another in downstream phase
  signatures
- `AuthorityScopedEffectPlan` is not an executor input; only
  `LoweredEffectExecutionPlan` may cross the execution boundary
- workflow lowering and runtime receipt shaping stay synchronized:
  equivalent admitted work must not produce one story through
  `worth-query::workflow` and another through `WORTHQueryWriteReceipt` or
  `WORTHQueryIntentExecution`
- batch execution does not degrade into scalar loop orchestration that
  re-admits basis or re-discovers authority/strategy per component
- executors do not re-decide authority family, basis scope, merge strategy,
  preview posture, writeback family, or artifact policy
- relational execution remains relational-authoritative and bridge-backed
  execution remains bridge-authoritative
- the certification suite can distinguish "same correct answer twice" from
  "same wrong answer twice" by using independent oracle comparisons
- `worth-signal` may contribute aftermath, invalidation, and explanation
  evidence, but it never becomes the authority lane for mutation, merge, or
  writeback execution
- Query receipts and envelopes expose authority names, decision traces,
  structural deltas, integrity markers, and counters without caller-side lower-
  runtime stitching
- support metadata, executable behavior, and certification coverage agree for
  admitted, advisory, denied, deferred, and unsupported effect families
- the golden DX transcripts prove all intended caller stories exist as first-
  class public paths:
  - common-path intent authoring for both relational mutation/merge and
    bridge-backed writeback
  - inspectable advanced lowering
  - support/discovery before execution
  - denial/rebind handling
  - batch-native execution
  - receipt-first explanation/diagnostics
- temporal, async/resource, store-backed execution, durable replay, and
  restart-stable effect envelopes remain typed deferred or unsupported until
  their owning milestones close
- compile-fail boundaries prove external callers cannot mint normalized effect
  intents, admitted authority scopes, lowered execution plans, execution
  receipts, envelopes, or authority witnesses
- compile-fail boundaries also prove external callers cannot:
  - execute from raw or normalized effect intent
  - execute from admitted-but-unlowered planning artifacts
  - construct lowered declarations or envelopes directly
- exact counter assertions prove:
  - executor rediscovery count remains zero
  - scalar execution lowering count is one per effect
  - batch lowering count is one per batch
  - batch execution does not re-admit basis per component
  - support lookup width follows support matrix width rather than runtime graph
    breadth
  - envelope materialization does not reopen authority execution
- small/medium/larger fixture runs prove normalization, eligibility, lowering,
  execution, receipt materialization, envelope materialization, and support
  lookup costs are bounded by effect width, authority width, and lowering width
  rather than unrelated runtime graph size or diagnostics retention width
- seeded randomized certification replays the same seed and proves identical
  canonical bundles, while a meaningfully changed seed or perturbation changes
  the expected declared digests or failure artifacts

Required verification output

- `query_digest`
- `raw_effect_intent_digest`
- `normalized_effect_intent_digest`
- `effect_family_digest`
- `effect_authority_digest`
- `effect_basis_digest`
- `effect_scope_digest`
- `effect_policy_digest`
- `effect_strategy_digest`
- `effect_eligibility_digest`
- `authority_scoped_effect_plan_digest`
- `lowered_effect_execution_plan_digest`
- `effect_execution_receipt_digest`
- `effect_envelope_digest`
- `relational_effect_authority_digest`
- `bridge_effect_authority_digest`
- `effect_decision_trace_digest`
- `effect_structural_delta_digest`
- `effect_integrity_marker_digest`
- `effect_target_dx_digest`
- `effect_golden_transcript_digest`
- `effect_support_matrix_digest`
- `effect_proof_shape_digest`
- `effect_phase_progression_digest`
- `effect_replay_parity_digest`
- `relational_oracle_digest`
- `bridge_oracle_digest`
- `seeded_sequence_digest`
- `seed_replay_digest`
- `compile_fail_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `executor_rediscovery_count`
- `batch_lowering_count`
- `batch_basis_reuse_count`
- `authority_reopen_count`
- `effect_normalization_slope_digest`
- `effect_eligibility_slope_digest`
- `effect_lowering_slope_digest`
- `effect_execution_slope_digest`
- `effect_receipt_materialization_slope_digest`
- `effect_envelope_materialization_slope_digest`
- `effect_support_lookup_slope_digest`

Pass condition

Every admitted runtime-backed Query effect executes only from one lowered,
proof-bearing authority plan; executors do not rediscover authority, basis,
strategy, or artifact policy locally; lower-runtime authority remains
relational- or bridge-owned; the public effect API reads like a deliberate
framework surface rather than an ad hoc wrapper over internals; independently
observed lower-runtime truth agrees with Query-produced receipts and envelopes;
and later projection-consumption, admission, lower-runtime-routing,
store-backed, and durable milestones inherit this execution contract instead of
bypassing it.

## Milestone 9.3.4 Named Certification Suites

### 9.3.4. Declared Projection Consumption And Materialized Fact Receipt Test

Purpose

Prove that every admitted consumed projection fact flows through one declared,
typed, receipt-backed lifecycle rather than letting consumers rediscover fact
meaning from relational truth, preview internals, signal aftermath, or
host-local caches after materialization.

Scenario

- use concrete runtime-backed fixtures covering detail materialization,
  collection membership materialization, grouped or topology-style
  materialization, query-context payload materialization, effect-produced
  materialization aftermath, and explicit typed denial/deferred families
- require the concrete source chain to stay visible:
  - `AuthorizedProjectionArtifact`
  - materialization basis receipt from Milestone 9.3.2
  - one admitted materialization source family
  - one consumed projection contract
  - one consumed projection fact set
  - one projection-consumption receipt/envelope
- construct equivalent consumed-fact declarations through at least two public
  or compatibility authoring paths and prove they normalize to the same
  contract, fact-set, and receipt meaning
- construct intentionally different fact families, source families,
  authorized-projection postures, or result-shape meanings and prove the
  declared digest fields diverge
- require every admitted consumed-fact lane to consume an admitted 9.3.2
  materialization basis capability rather than raw branch, preview, snapshot,
  history, tenant, or policy identifiers
- compile golden DX transcripts for ordinary read-backed fact consumption,
  effect-backed fact consumption, support/discovery before consumption, typed
  denial/deferred handling, and receipt/envelope inspection
- compile negative DX and boundary transcripts proving forbidden raw-row,
  raw-json, weak-source, and lower-runtime-direct consumption paths do not
  compile or are not publicly reachable
- exercise the lifecycle as a linear typestate chain where each transition
  consumes the prior proof type and returns the next proof type or a typed
  denial
- prove source-family adapters consume only declared materialization artifacts
  and do not reopen lower-runtime authority locally
- exercise hostile lanes for masked fact requests, source-family overclaim,
  stale or policy-drifted materialization, basis/source mismatch, deferred
  durable reload, and host-side raw row parsing attempts

Required concrete lanes

- detail lane where one admitted authorized projection and materialization basis
  yield identity and label/display facts through one consumed-fact receipt
- collection-membership lane where one admitted materialized collection yields
  membership and view-local identity facts without reopening authority
- topology/Worth-style lane where one bounded materialization yields relation
  endpoint or shared-membership facts through the shared lifecycle rather than
  a topology-only helper surface
- effect-aftermath lane where one admitted 9.3.3 effect receipt provides the
  materialization source for consumed projection facts without replaying
  execution or re-reading authority
- query-context lane where an admitted query-context materialization exposes
  only the fact families that the source family actually proves
- masked-fact denial lane where a hidden or masked field/influence is requested
  as a consumed fact and fails typed before a fact set exists
- source-mismatch lane where a caller requests a fact family from a
  materialization source that does not prove it and receives a typed
  source-mismatch or denial artifact
- stale-after-materialization lane where materialization succeeds, source truth
  or policy drifts, and subsequent fact consumption denies or defers exactly
  where the contract says it must
- deferred lane where persisted fact receipts, durable reload, store-backed
  reconstruction, or portable export remain typed deferred with zero
  operational residue
- seeded-random lane where declaration shapes, fact-family mixes, source
  families, policy masks, and denial/deferred neighbors are generated from a
  fixed seed and replayed with identical canonical outputs

Required oracle posture

Every admitted lane must name and preserve one independent oracle strategy.
The certification harness may share fixtures with production code, but it may
not reuse the same Query consumption path as its oracle.

- detail lane oracle:
  independently compute expected visible identity/label facts from the
  admitted authorized projection and admitted materialization source evidence
  without calling the consumed-fact extractor under test
- collection-membership lane oracle:
  independently compute expected membership and view-local identity facts from
  the admitted collection/grouped source evidence without using Query's public
  membership extraction helpers
- topology/Worth-style lane oracle:
  independently compute the expected relation-endpoint or shared-membership
  facts from the admitted grouped/topology authority artifact rather than from
  any topology-specific Query convenience helper
- effect-aftermath lane oracle:
  independently verify expected target/source-reference/continuity facts from
  the admitted 9.3.3 receipt and its carried authoritative evidence without
  re-running effect execution or invoking the consumed-fact extractor under
  test
- query-context lane oracle:
  independently verify only the narrow context-proven fact families using the
  admitted query-context source artifact and declaration support posture, not a
  generic "all payload fields are facts" assumption

The harness must emit an oracle-manifest row for each admitted lane naming:

- lane name
- oracle owner module
- source artifacts consulted
- explicitly forbidden reused production helpers
- comparison digest fields

Must verify

- consumed projection facts are not ambient permission and cannot be recovered
  from raw materialized rows alone
- authorized projection from Milestone 9 is mandatory visibility input to fact
  eligibility and extraction
- basis capability proof from Milestone 9.3.2 is mandatory input to consumed
  fact admission
- equivalent consumed-fact authoring paths normalize to the same declaration,
  contract, fact set, and receipt meaning
- intentionally different fact families, source families, basis postures,
  policy postures, or result-shape meanings change the relevant digest
- consumed-fact eligibility precedes any fact-set construction
- admitted, admitted-with-warnings, denied, deferred, and source-mismatch
  outcomes are
  distinct proof families that cannot be substituted for one another in later
  phase signatures
- source adapters do not reopen relational truth, bridge preview internals,
  signal state, or host caches to fill in missing facts
- effect-produced materialization facts remain bound to 9.3.3 receipts and do
  not recreate executor-side authority decisions locally
- the certification suite can distinguish "same wrong answer twice" from "same
  correct answer twice" by using independent oracle comparisons against the
  admitted materialization/source evidence
- support metadata, executable behavior, and certification coverage agree for
  admitted, admitted-with-warnings, denied, deferred, and source-mismatch fact
  families
- support-matrix rows are traceable rather than narrative:
  every admitted or denied source-family/fact-family cell must identify
  exactly one admission rule, one hostile neighbor or denial lane, one
  certification lane, and one compile-fail or public-boundary proof when the
  cell's restriction is enforced structurally
- the golden DX transcripts prove all intended caller stories exist as
  first-class public paths:
  - common-path read-backed fact consumption
  - common-path effect-backed fact consumption
  - support/discovery before consumption
  - typed denial/deferred handling
  - receipt-first inspection/envelope derivation
- the negative DX transcripts prove the public surface does not admit the wrong
  ergonomics:
  - raw rows do not expose Query consumed-fact accessors
  - raw JSON/value bags are not the public consumed-fact API
  - lower-runtime source artifacts cannot skip declaration/eligibility/contract
    progression
  - denied/deferred artifacts do not expose admitted-only methods
- temporal, async/resource, store-backed reconstruction, durable reload, and
  portable receipt export remain typed deferred or unsupported until their
  owning milestones close
- compile-fail boundaries prove external callers cannot mint admitted
  declarations, contracts, fact sets, receipts, envelopes, inventories, or
  certification artifacts directly
- compile-fail boundaries also prove external callers cannot:
  - consume facts from raw materialized rows without an admitted contract
  - treat weaker source artifacts as admitted fact sets
  - construct source-family adapters or envelopes directly
- exact counter assertions prove:
  - authority reopen count remains zero
  - fact extraction width follows declared fact width rather than unrelated
    runtime graph breadth
  - support lookup width follows support-matrix width rather than host cache
    breadth
  - envelope materialization does not reopen source authority
- small/medium/larger fixture runs prove declaration, eligibility, contract
  binding, fact extraction, receipt materialization, envelope materialization,
  and support lookup costs are bounded by declared fact width, source evidence
  width, and materialized row width rather than unrelated runtime graph size or
  host cache size
- seeded randomized certification replays the same seed and proves identical
  canonical bundles, while a meaningfully changed seed or perturbation changes
  the expected declared digests or failure artifacts
- seeded randomized certification uses only declared generator classes whose
  semantics are visible in the output bundle:
  - declaration-shape generator
  - fact-family mix generator
  - source-family selection generator
  - policy-mask generator
  - denial/deferred neighbor generator
  - basis/source mismatch generator
- randomized lanes must emit enough metadata to replay the exact generator
  class choices, not just the raw seed

Required verification output

- `query_digest`
- `result_shape_digest`
- `authorized_projection_digest`
- `materialization_basis_digest`
- `projection_consumption_declaration_digest`
- `projection_consumption_eligibility_digest`
- `materialized_projection_contract_digest`
- `consumed_projection_fact_set_digest`
- `projection_consumption_receipt_digest`
- `projection_consumption_envelope_digest`
- `projection_source_digest`
- `projection_source_receipt_digest`
- `projection_fact_family_inventory_digest`
- `projection_support_matrix_digest`
- `projection_public_surface_digest`
- `projection_target_dx_digest`
- `projection_golden_transcript_digest`
- `projection_proof_shape_digest`
- `projection_phase_progression_digest`
- `projection_transition_rules_digest`
- `projection_oracle_digest`
- `projection_oracle_manifest_digest`
- `projection_support_traceability_digest`
- `seeded_sequence_digest`
- `seed_replay_digest`
- `seed_generator_class_digest`
- `compile_fail_boundary_digest`
- `negative_dx_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `authority_reopen_count`
- `fact_extraction_width`
- `projection_declaration_slope_digest`
- `projection_eligibility_slope_digest`
- `projection_contract_binding_slope_digest`
- `projection_fact_extraction_slope_digest`
- `projection_receipt_materialization_slope_digest`
- `projection_envelope_materialization_slope_digest`
- `projection_support_lookup_slope_digest`

Pass condition

Every admitted consumed projection fact comes from one declared materialization
contract and one receipt-backed fact set; consumers do not reopen source
authority to rediscover facts; authorized projection and materialization basis
remain explicit prerequisites; effect-backed and read-backed materializations
tell the same public consumed-fact story; and later admission, routing,
store-backed, and durable milestones inherit this contract instead of bypassing
it.

## Milestone 9.3.5 Named Certification Suites

### 9.3.5. Intent Admission Decision Lattice And Decision Trace Test

Purpose

Prove that every covered Query-crossing intent resolves through one structured
admission lattice with typed admitted, advisory, and violation outcomes before
construction, lowering, execution, or diagnostic materialization, and that the
resulting decision trace is offline-readable without reconstructing family
semantics from raw inputs or hidden lower-runtime state.

This suite must also prove that 9.3.5-covered public entrypoints delegate into
one canonical admission and execution-handoff path, that covered execution
cannot bypass typed handoffs, that execution receipts preserve provenance back
to the handoff and decision chain, and that the target DX and crate
documentation remain synchronized with executable behavior.

Scenario

- use concrete runtime-backed fixtures spanning at least:
  - basis-use intent
  - effect-execution intent
  - projection-consumption intent
  - inspection or diagnostic-materialization intent
  - lower-runtime capability-routing request authoring as admitted/deferred/
    denied pre-routing posture
- materialize the compile-visible covered-entrypoint and execution-seam
  inventory for the 9.3.5 implementation under test
- construct equivalent raw intents through at least two public paths for each
  admitted family and prove they normalize to the same decision meaning
- construct intentionally different policy, basis, support, projection/source,
  invariant, and routing-support posture variants and prove the relevant digest
  fields diverge
- derive family inventory, eligibility, admission decision, downstream plan or
  advisory/violation artifact, admitted execution handoff where applicable,
  execution receipt or result artifact where applicable, and decision-trace
  envelope for each lane
- exercise advisory lanes such as warning-bearing projection consumption,
  redacted inspection, rebind-required effect work, or deferred future-neighbor
  posture where the family honestly supports advisory meaning
- exercise violation lanes such as stale basis, masked or unsupported request,
  source mismatch, invariant denial, and unsupported routing-capability
  posture
- exercise supported legacy public entrypoints and canonical lattice entrypoints
  for the same covered path and prove they converge observationally because the
  legacy path delegates rather than emulates
- compile crate-facing documentation examples and golden DX transcripts for the
  common path and advanced path promised by the 9.3.5 milestone spec
- include a Worth-style consumer lane allowed to inspect only the shared
  admission lattice and decision-trace envelope rather than family-specific
  internals

Required concrete lanes

- basis parity lane where equivalent basis-use intents normalize to the same
  eligibility, decision, and trace meaning
- effect admitted lane where the shared lattice yields one admitted plan that
  the effect pipeline can lower without revalidating raw intent
- projection advisory lane where warning-bearing or deferred-neighbor
  projection-consumption posture is preserved as advisory rather than flattened
  to success or violation
- inspection advisory-redaction lane where detail narrowing remains explicitly
  advisory and still emits one canonical decision trace envelope
- violation lane where stale, masked, mismatched, or unsupported posture fails
  before downstream construction or lowering
- routing future-neighbor lane where lower-runtime capability-routing intent is
  classified as deferred or denied ahead of 9.3.6 execution semantics
- covered-entrypoint inventory lane where every covered public method and
  covered execution seam is enumerated, classified, and bound to one canonical
  authoring path
- legacy delegation parity lane where a supported legacy public entrypoint and
  the canonical lattice path produce the same decision, handoff, execution
  provenance chain, and observable result
- execution provenance lane where the admitted decision, admitted handoff, and
  resulting route/evaluation/write receipt all bind one retrievable provenance
  chain
- non-bypass boundary lane where direct execution with raw intent, weak tags,
  generic payloads, or loose target selectors fails at compile time or
  certified boundary enforcement
- trace parity lane where richer diagnostic materialization changes only the
  envelope detail and not the underlying admission decision
- common-path DX lane where the ordinary public path reads like intent ->
  admit -> execute -> inspect receipt
- advanced-path DX lane where the caller can inspect eligibility, decision,
  plan, handoff, and provenance as distinct phases
- crate-doc parity lane where crate documentation examples compile or otherwise
  execute under the crate-doc policy and stay synchronized with the final
  public names and target DX
- public-boundary lane where external code trying to mint admitted plans or
  decision traces directly fails at compile time

Must verify

- all covered intent families resolve through one shared public admission
  lattice rather than family-local binary-only surfaces
- the compile-visible covered-entrypoint inventory, support metadata, and named
  certification coverage stay synchronized; family-wide closure cannot be
  claimed while omitting concrete covered methods
- admitted, advisory, and violation outcomes are distinct proof families rather
  than one loose struct with optional holes
- family-specific correctness distinctions survive inside the shared lattice:
  advisory redaction, rebind-required, deferred support, source mismatch, and
  policy denial remain mechanically distinct where downstream behavior differs
- lower phases consume admitted plans or family-specific admitted wrappers
  derived from them instead of revalidating raw intent
- covered execution seams consume only family-specific typed admitted handoffs;
  raw intent, weak plans, string tags, or generic execution bags are not valid
  substitutes
- supported legacy public entrypoints delegate into the canonical lattice path
  rather than keeping observationally similar parallel implementations
- admitted decisions, admitted handoffs, and covered execution receipts retain
  one canonical execution provenance chain that later inspection and
  certification can recover
- decision traces carry structured policy, capability, invariant, basis,
  projection/source, and routing-support evidence where applicable
- decision traces are assembled from authority-owned evidence digests rather
  than reminting bridge, relational, signal, basis, or projection authority as
  Query-owned truth
- diagnostic richness changes only envelope detail, not the canonical
  admission decision
- offline consumers can localize which stage decided the outcome using the
  canonical envelope alone
- support metadata, executable admission behavior, and certification coverage
  agree for admitted, advisory-capable, deferred, and unsupported families
- unsupported temporal, async/resource, durable, and store-backed neighbors
  remain typed deferred or denied rather than partial support
- compile-fail boundaries prove external callers cannot mint admitted plans,
  advisory/violation artifacts, handoffs, trace rows, trace envelopes, or
  certification bundles directly
- compile-fail or certified negative-DX boundaries prove covered execution
  cannot be reached through raw runtime/bridge calls that bypass the admitted
  handoff model
- common-path and advanced-path golden DX transcripts remain synchronized with
  executable behavior, support metadata, receipt shape, and phase progression
- crate-facing documentation examples remain synchronized with the final public
  API and do not depend on internal module topology or milestone vocabulary
- exact counter assertions prove family lookup, eligibility resolution, and
  trace assembly widths follow declared family and decision width rather than
  unrelated runtime size
- exact counter assertions prove covered-entrypoint inventory lookup and
  execution provenance assembly widths follow declared coverage width rather
  than unrelated runtime size
- small/medium/larger fixture runs prove admission classification and trace
  assembly costs scale with declared family coverage width and decision width
  rather than unrelated row count
- small/medium/larger fixture runs also prove covered-entrypoint coverage,
  delegation parity, and execution provenance costs scale with coverage width
  rather than unrelated row count
- seeded randomized certification replays the same seed into the same
  canonical decision bundle, while changed generator choices or meaningfully
  different seeds change the expected digests or failure artifacts

Required verification output

- `query_digest`
- `intent_family_digest`
- `raw_intent_digest`
- `intent_eligibility_digest`
- `admission_decision_digest`
- `admitted_intent_plan_digest`
- `admitted_execution_handoff_digest`
- `advisory_decision_digest`
- `violation_decision_digest`
- `decision_trace_digest`
- `decision_trace_envelope_digest`
- `policy_decision_digest`
- `capability_decision_digest`
- `invariant_decision_digest`
- `basis_decision_digest`
- `projection_decision_digest`
- `routing_posture_digest`
- `intent_family_inventory_digest`
- `covered_entrypoint_inventory_digest`
- `execution_seam_inventory_digest`
- `intent_support_matrix_digest`
- `intent_public_surface_digest`
- `intent_target_dx_digest`
- `intent_golden_transcript_digest`
- `negative_dx_boundary_digest`
- `crate_doc_example_digest`
- `decision_proof_shape_digest`
- `decision_phase_progression_digest`
- `execution_provenance_chain_digest`
- `legacy_delegation_parity_digest`
- `decision_oracle_digest`
- `decision_support_traceability_digest`
- `seeded_sequence_digest`
- `seed_replay_digest`
- `seed_generator_class_digest`
- `compile_fail_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `intent_family_lookup_width`
- `covered_entrypoint_lookup_width`
- `decision_trace_width`
- `execution_provenance_width`
- `admission_classification_slope_digest`
- `decision_trace_assembly_slope_digest`
- `decision_support_lookup_slope_digest`
- `covered_entrypoint_inventory_slope_digest`
- `execution_provenance_assembly_slope_digest`
- `legacy_delegation_parity_slope_digest`
- `decision_certification_coverage_slope_digest`

Pass condition

Every covered Query-crossing intent resolves through one typed admission
lattice before expensive work starts; admitted, advisory, and violation cases
are equally inspectable; covered public entrypoints delegate into one canonical
handoff-driven execution path; execution provenance survives into covered
receipts; target DX and crate documentation stay synchronized with executable
behavior; downstream phases consume admitted plans and typed handoffs rather
than raw intents; and later routing, public runtime stabilization, and
temporal/async milestones inherit one canonical public admission model instead
of several drifting family-local ones.

## Milestone 9.3.6 Named Certification Suites

### 9.3.6. Lower-Runtime Capability Routing And Boundary Envelope Closure Test

Purpose

Prove that every covered Query-to-lower-runtime crossing is either deleted in
favor of an authoritative lower-runtime contract, routed through one typed
capability/receipt/envelope lifecycle, or explicitly deferred to a named later
milestone with owner, exit criteria, and certification coverage.

Scenario

- use concrete runtime-backed fixtures covering:
  - current read evaluation
  - historical or replay-aware materialization
  - preview basis readmission
  - subscription activation/continuity
  - writeback and signal invalidation routing
  - effect-backed lower-runtime execution
  - projection-consumption source intake
  - causal inspection bridge materialization
  - frontier-aware or signal-backed planning intake
- derive the executable crossing inventory for all covered families and seams
- classify every row as:
  - canonical lower-runtime reuse
  - Query boundary adapter
  - compatibility debt lane
  - deferred neighbor
  - forbidden duplicate
- exercise rows where:
  - the existing lower-runtime contract is sufficient and the direct seam must
    be deleted
  - Query still needs a narrower lower-runtime receipt or envelope and the
    route must bind that gap explicitly
  - the missing contract is added and the former direct seam disappears before
    closeout
  - a future-neighbor request must fail deferred or unsupported
  - a convenience bypass path attempts to reach a covered lower-runtime seam
- include one hostile internal seam lane where a covered Query module tries to
  use a direct lower-runtime import outside the routed lane and the
  compile-boundary or certification audit rejects it
- include one downstream domain-runtime fixture lane where a declared
  runtime-boundary subtree is allowed to implement Query runtime extension
  seams, while ordinary downstream modules fail the same direct-import audit

Required concrete lanes

- canonical-reuse lane where one covered read or subscription seam routes
  through an already-authoritative lower-runtime contract and emits the same
  route and evidence digests regardless of authoring path
- seam-deletion lane where a formerly direct specialist path is removed once
  the existing lower-runtime facade is used honestly
- missing-contract lane where Query can localize one exact missing bridge,
  relational, or signal contract instead of normalizing the direct seam as
  permanent architecture
- seam-elimination lane where a former direct seam survives only long enough
  to prove the missing lower-runtime contract was added and the direct seam was
  then deleted or reduced to a thin allowed adapter
- route-planning lane where the crossing emits one `LowerRuntimeRoutePlan`,
  one `BoundaryExecutionReceipt`, and one `LowerRuntimeBoundaryEnvelope`
- readmission-only lane where the crossing honestly skips route planning but
  still emits typed receipt and boundary envelope artifacts
- forbidden-duplicate lane where an alternate direct path that would duplicate
  or bypass an admitted lower-runtime route fails typed or compile-time
- deferred-neighbor lane where temporal, async/resource, mixed-cause,
  store-backed, or durable route claims remain explicit deferred or unsupported
- public-boundary lane where ordinary callers attempting to reach lower-runtime
  internals through the public Query surface fail at the certified boundary

Must verify

- every covered Query-to-lower-runtime crossing appears in one executable
  inventory with one classification and one authority owner
- no direct seam remains socially accepted or invisible to certification
- deletable seams are actually deleted rather than merely renamed or wrapped
- surviving specialist seams are treated as defects-to-delete by default and
  remain only when they correspond to one named missing lower-runtime contract
  during the implementation transition or one explicit deferred-neighbor row
- transition elimination rows bind:
  - the direct seam identity
  - the owning lower-runtime crate
  - the missing lower-runtime capability/receipt/envelope
  - required closeout
  - certification coverage
- route-planning seams emit one route plan naming authority, route family,
  capability, cost posture, failure topology, and retained evidence posture
- readmission/handoff-only seams remain mechanically distinct from real route
  planning where downstream behavior differs
- covered operational seams do not return only `()`, raw `String`, or weak
  incidental handles when a receipt or envelope is required
- boundary envelopes are assembled from lower-runtime authority artifacts and
  Query-owned routing meaning rather than reminting bridge, relational, or
  signal truth inside Query
- support metadata, executable routing behavior, and deferred/forbidden
  posture agree for admitted, deferred, unsupported, and forbidden rows
- compile-fail or certified negative-boundary fixtures prove ordinary callers
  cannot reach covered lower-runtime seams through the public surface
- compile-fail or certified internal-boundary fixtures prove covered Query
  modules cannot silently bypass the routed capability lane by convenience
  once the seam is declared covered
- small/medium/larger fixture runs prove inventory lookup, route-plan assembly,
  receipt shaping, envelope shaping, and deferred-neighbor lookup scale with
  crossing width, evidence width, and deferred-neighbor width rather than
  unrelated runtime graph size

Required verification output

- `query_digest`
- `capability_request_digest`
- `capability_family_digest`
- `capability_eligibility_digest`
- `lower_runtime_route_plan_digest`
- `boundary_execution_receipt_digest`
- `lower_runtime_boundary_envelope_digest`
- `crossing_inventory_digest`
- `crossing_classification_digest`
- `compatibility_debt_registry_digest`
- `debt_exit_criteria_digest`
- `route_authority_digest`
- `route_evidence_digest`
- `route_cost_posture_digest`
- `route_failure_topology_digest`
- `route_support_matrix_digest`
- `route_public_surface_digest`
- `route_target_dx_digest`
- `route_golden_transcript_digest`
- `route_proof_shape_digest`
- `route_phase_progression_digest`
- `route_parity_digest`
- `route_non_bypass_digest`
- `lower_runtime_gap_registry_digest`
- `compile_fail_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `crossing_inventory_width`
- `compatibility_debt_width`
- `route_plan_width`
- `boundary_evidence_width`
- `capability_eligibility_slope_digest`
- `route_plan_assembly_slope_digest`
- `boundary_receipt_assembly_slope_digest`
- `boundary_envelope_assembly_slope_digest`
- `support_lookup_slope_digest`
- `debt_registry_lookup_slope_digest`

Pass condition

Every covered Query-to-lower-runtime crossing is explicitly inventoried and
classified, every admitted crossing is capability-routed and receipt-backed,
every remaining direct seam is intentional compatibility debt with an exit
story, and the runtime API stabilization gate can rely on one closed
lower-runtime boundary model rather than scattered convenience contact.

## Milestone 9.3.7 Named Certification Suites

### 9.3.7. Domain Capability Contribution And Canonical Runtime Materialization Test

Purpose

Prove that downstream domains can contribute semantic capability posture across
admission, support, invariant, workflow, continuity, aftermath, and
explanation categories through one public Query-owned contribution seam while
Query remains the sole owner of canonical runtime artifacts.

Scenario

- use concrete runtime-backed fixtures with one canonical `WORTHQueryIntentDeclaration`
  and at least two equivalent public domain-contribution builder paths for:
  - admission posture
  - declaration-scoped support posture
  - graph-composition capability-gap posture
  - graph-composition invariant-denial posture
  - workflow posture
  - continuity posture
  - aftermath posture
  - explanation posture
- materialize canonical Query artifacts from each admitted contribution
- include hostile lanes where:
  - a domain attempts to mint canonical runtime artifacts directly
  - a contribution uses free-form fallback payloads instead of typed posture
  - declaration-scoped support tries to mutate or impersonate the global
    support matrix
  - equivalent support evidence is reordered without changing semantic meaning
  - intentionally different category posture must change
    the canonical digests

Required concrete lanes

- admission parity lane where two equivalent domain admission contributions
  produce the same canonical admission artifacts
- declaration-scoped support lane where support materialization remains bound
  to one declaration and does not masquerade as a global inventory mutation
- capability-gap lane where domain capability posture materializes as canonical
  `WORTHQueryGraphCompositionCapabilitySupportRow` artifacts
- invariant-denial lane where domain invariant posture materializes as
  canonical `WORTHQueryGraphCompositionDomainInvariantDenial` artifacts
- workflow parity lane where equivalent workflow posture contributes the same
  canonical workflow / preview artifact family
- continuity parity lane where equivalent continuity posture contributes the
  same canonical continuity / lineage artifact family
- aftermath parity lane where equivalent aftermath posture contributes the
  same canonical consequence / aftermath artifact family
- explanation parity lane where equivalent explanation posture contributes the
  same canonical explanation / inspection artifact family
- direct-minting denial lane where callers outside Query cannot mint canonical
  runtime artifacts directly
- category-mismatch compile-fail lane where a contribution admitted for one
  category cannot be fed to another category's materializer
- contribution-shape denial lane where stringly or opaque fallback payloads
  fail typed before canonical materialization

Must verify

- the public contribution seam is declaration-scoped and bound to one Query
  declaration or admitted family context
- canonical runtime artifacts remain Query-owned and sealed
- equivalent domain contribution meaning normalizes to the same canonical Query
  artifact regardless of builder path or evidence ordering
- intentionally different semantic posture changes the declared digests
- declaration-scoped support and traceability do not mutate the global support
  matrix or certification inventory
- graph-composition capability and invariant posture materialize through
  canonical Query runtime artifacts rather than domain-local wrappers
- workflow, continuity, aftermath, and explanation posture each materialize as
  real canonical public families with ordinary, inspectable, and proof-bearing
  lanes that all hit the same semantics
- category mismatch is a type error, not a runtime branch
- contribution-aware trace rows localize whether failure occurred during
  contribution authoring, eligibility, canonical materialization, or runtime
  artifact shaping
- small/medium/larger fixture runs prove contribution materialization scales
  with contribution width, trace width, and support width rather than
  unrelated runtime graph size

Required verification output

- `query_digest`
- `intent_declaration_digest`
- `domain_capability_contribution_request_digest`
- `domain_capability_contribution_eligibility_digest`
- `admitted_domain_capability_contribution_digest`
- `canonical_runtime_materialization_digest`
- `admission_artifact_digest`
- `declaration_scoped_support_digest`
- `workflow_artifact_digest`
- `continuity_artifact_digest`
- `aftermath_artifact_digest`
- `explanation_artifact_digest`
- `capability_support_row_digest`
- `domain_invariant_denial_digest`
- `decision_trace_digest`
- `support_traceability_digest`
- `public_boundary_digest`
- `compile_fail_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `contribution_width`
- `trace_width`
- `category_width`
- `support_width`
- `contribution_materialization_slope_digest`
- `trace_materialization_slope_digest`
- `category_materialization_slope_digest`
- `support_materialization_slope_digest`

Pass condition

Domains can publicly contribute broad semantic capability posture without
building local pseudo-Query runtime layers, canonical Query artifacts remain
Query-owned, and equivalent contribution meaning materializes identically
across builder-path variation and category boundaries.

## Runtime API Public Stabilization Gate Named Certification Suites

### Runtime API Golden DX And Async-Safe Facade Test

Purpose

Prove that the ordinary public runtime API is beautiful enough for serious
domain runtimes and structurally honest enough for later temporal/async
extension, without relying on lower-runtime plumbing or sync-only assumptions.

Scenario

- execute golden public-facade transcripts for:
  - workflow/editor surfaces with live sections, nested computeds, conditional
    effects, branch preview, branch-local intent, and inspection
  - geometry/kernel surfaces with topology neighborhood live views, expensive
    derived outputs, fallback posture, invariant-preserving intent, and branch
    experiment inspection
  - table/spreadsheet surfaces with visible rows, formula/dropdown/layout
    computeds, ordered/grouped delivery, batched edit intent, and async-ready
    state vocabulary
  - one composed adversarial surface spanning live subscription evidence,
    nested computeds, effect pending intents, authoritative/effect/branch
    intents, preview isolation, inspection, and feedback phase graph evidence
- exercise unsupported-neighbor requests for temporal basis, async/resource
  state, mixed-cause delivery, store-backed parity, and durable restart before
  those milestones are admitted

Must verify

- golden transcripts use the final public facade vocabulary only
- no transcript manually installs subscription declarations, bridge
  lowerings, signal observers, grouped baselines, active lanes, or CDC filters
- every transcript asserts meaningful proof artifacts: receipts, support
  posture, authority lanes, basis lanes, aspect contracts, dependency handles,
  delivery batches, residue counters, and inspection sections
- state/read APIs do not imply always-synchronous values for surfaces that may
  later admit temporal or async/resource state
- unsupported temporal/async/store/durable neighbors fail typed and early with
  zero forbidden delivery or authority residue
- inspection explains every advertised handle/receipt family from retained
  artifacts rather than debug strings or re-lowering
- final public names do not hide query execution, subscription activation,
  branch/preview binding, intent commit, temporal/async, or diagnostic
  boundary crossings
- support metadata and executable admission agree for stable, deferred, and
  unsupported public API families
- compile-fail boundaries prove ordinary callers cannot bypass the facade or
  synthesize proof-bearing handle/inspection/support artifacts externally

Required verification output

- `public_api_surface_digest`
- `golden_transcript_digest`
- `handle_contract_digest`
- `state_contract_digest`
- `aspect_contract_digest`
- `authority_lane_digest`
- `inspection_contract_digest`
- `support_matrix_digest`
- `deferred_temporal_async_gate_digest`
- `failure_digest`
- `counter_snapshot`
- `compile_fail_boundary_digest`

Pass condition

The public runtime API is stable enough for domain runtimes to build on now,
while temporal, async/resource, mixed-cause, store-backed, and durable claims
remain explicit deferred or unsupported surfaces until their owning milestones
close.

## Runtime Authoritative Mutation Evidence Gate Named Certification Suites

### Runtime Authoritative Mutation Evidence And Existing-Truth Binding Test

Purpose

Prove that the public mutation facade preserves enough authored and resolved
authority meaning that write-heavy domains do not need local target recovery,
existing-truth rebinding, causality/provenance reconstruction, or
naming/continuity explanation glue.

Scenario

- execute direct authoritative and preview-local mutation sessions covering:
  - insert of new truth
  - update of existing truth through admitted existing-target binding
  - delete of existing truth with explicit touched-aspect meaning
  - retained existing-truth assertion through `assert_existing(...)`
  - backend-verified existing-truth assertion through `verify_existing(...)`
  - backend-verified existing-truth probe through `probe_existing(...)`
  - backend-verified existing-target update through `update_existing_verified(...)`
  - backend-verified existing-target delete through `delete_existing_verified(...)`
  - ordered batch mixing same-batch symbolic references and existing-truth
    references
  - hostile mixed authority sessions that combine retained assertion,
    backend-verified assertion, backend-verified update/delete, same-batch
    symbolic naming, continuity-aware mutation evidence, and probe-before or
    probe-after truth checks
  - authoritative import/session aggregation where the session cannot honestly
    be summarized as one scalar write
  - admitted naming-aware mutation evidence
  - admitted continuity-aware mutation evidence
- exercise unsupported-neighbor requests for:
  - unresolved existing-truth binding
  - collection-mismatched or missing existing-truth binding
  - backend verification unsupported for assertion, probe, and verified
    mutation lanes
  - unsupported naming-writeback family
  - unsupported continuity-sensitive family

Must verify

- receipts preserve declared-versus-resolved target evidence together with
  touched-aspect fallout evidence
- retained assertions, backend-verified assertions, backend-verified probes,
  verified updates, and verified deletes remain semantically distinguishable
  across receipts, scalar inspection, batch inspection, and aggregate session
  evidence
- receipts preserve first-class causality and provenance bundles rather than
  flattening source lineage into incidental metadata
- preview and authoritative receipts use the same target-evidence model
- batch/session inspection exposes per-component and aggregate authority
  evidence honestly, including causality/provenance summaries
- same-batch symbolic targets, existing authoritative targets, and denied
  bindings are distinguished explicitly
- impossible mutation-family or assertion-mode combinations fail loudly rather
  than producing misleading aggregate summaries
- naming-aware and continuity-aware families either preserve explicit outcome
  evidence or deny typed and early
- support metadata and executable admission behavior agree for all admitted and
  denied mutation-evidence neighbors

Required verification output

- `mutation_receipt_digest`
- `mutation_target_evidence_digest`
- `batch_authority_evidence_digest`
- `existing_truth_mode_digest`
- `existing_truth_probe_digest`
- `mutation_causality_digest`
- `mutation_provenance_digest`
- `existing_truth_binding_digest`
- `naming_mutation_evidence_digest`
- `continuity_mutation_evidence_digest`
- `support_matrix_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

The public mutation surface preserves target, authority, and admitted
naming/continuity evidence strongly enough that downstream domains can rely on
Query receipts and inspection instead of rebuilding the same explanation layer
locally, and the bridge-side replay/provenance artifacts remain compatible with
that same public evidence story.

### Runtime Mixed-Shape Graph Authoring And Identity-Preserving Mutation Test

Purpose

Prove that the public runtime mutation facade can serve as a real graph-shaped
authoring runtime for serious downstream domains without forcing them to
reintroduce local relation-rewrite semantics, symbolic graph folklore, or
bridge-backed verification glue.

Scenario

- execute public runtime authoring sessions covering:
  - identity-preserving existing-target relation update
  - identity-preserving existing-target relation verified update
  - same-batch graph composition with:
    - created entity -> created relation
    - existing entity -> created relation
    - mixed existing-target and same-batch symbolic references
    - symbolic entity follow-up mutation
    - symbolic relation follow-up mutation
    - symbolic relation retirement
    - mixed create/update/delete composition in one canonical program
  - geometry-inspired hostile generic graph programs covering:
    - `EdgeSplit`
      - existing edge relation retired
      - two new edges created
      - one new vertex created
      - adjacency relations rewired
      - receipt proves lineage and identity outcomes
    - `LoopSuccessorRewire`
      - existing successor relation retargets old successor to new successor
      - relation identity is preserved
      - verification asserts old source/target assumptions before mutation
    - `FaceInnerLoopInsertion`
      - create loop entity
      - create relation from existing face to symbolic loop
      - create symbolic edges/vertices
      - resolution map exposes all symbolic-to-resolved identities
    - `FailedNonManifoldAdmission`
      - substrate can express the graph shape
      - domain invariant hook denies it
      - denial remains distinct from runtime support denial
  - bridge-backed backend-verified existing-truth assertion/probe/verified
    update/delete on admitted entity and relation families
- exercise hostile neighbors for:
  - replacement-shaped relation rewrite masquerading as update
  - unresolved symbolic graph references
  - reversed or illegal composition order
  - incomplete invariant-bearing graph subgraphs
  - bridge-backed runtime families lacking verification support
  - domain-invalid but substrate-expressible graph programs
  - target-collection and target-shape mismatch

Must verify

- existing-target relation update preserves authoritative relation identity
  across receipt, inspection, and batch/session aggregate evidence
- update denial fires typed and early whenever the lower runtime cannot preserve
  identity honestly
- graph composition is one public authoring family rather than caller-owned
  scalar batch folklore
- graph composition receipts preserve symbolic-to-resolved target mapping,
  component ordering, target evidence, lifecycle-step meaning, lifecycle
  outcome meaning, and affected live/computed breadth
- graph composition preserves and exposes assumption/read-set evidence
  distinctly from target binding and mutation result evidence
- graph composition denial distinguishes unresolved symbolic references,
  illegal ordering, incomplete subgraph workflows, unsupported mixed-shape
  capability families, lower-runtime identity-preservation gaps, verification
  substrate unavailability, domain invariant denial, and unsupported runtime
  families
- denied paths expose an admission trace showing the classification stage where
  execution stopped, and that trace is not the same artifact as a receipt
- bridge-backed backend-verified existing-truth surfaces report admitted versus
  unsupported posture honestly through support metadata and executable behavior
- memory or compatibility-backed support does not overclaim bridge-backed
  production support
- public docs/examples match the admitted and denied runtime story literally
- compile-fail boundaries prove external callers cannot mint proof-bearing
  graph-composition or support/closeout artifacts directly

Required verification output

- `relation_update_identity_digest`
- `relation_update_target_evidence_digest`
- `relation_update_denial_digest`
- `graph_composition_digest`
- `graph_symbolic_resolution_digest`
- `graph_composition_program_digest`
- `graph_composition_denial_digest`
- `graph_composition_lifecycle_digest`
- `verified_assumption_set_digest`
- `assumption_snapshot_digest`
- `verified_precondition_digest`
- `verification_read_set_breadth_digest`
- `admission_trace_digest`
- `verified_existing_bridge_support_digest`
- `verified_existing_runtime_receipt_digest`
- `support_matrix_digest`
- `failure_digest`
- `counter_snapshot`
- `compile_fail_boundary_digest`

Pass condition

The public runtime authoring facade is strong enough that downstream graph
domains can use Query as the ordinary mutation runtime for admitted relation
updates, same-batch graph composition, and bridge-backed verified existing-
truth work, while unsupported neighbors fail closed before local substitute
semantics can grow around them.

