## Milestone 9.4 Named Certification Suites

### 9.4. Temporal Query Basis And Time-Aware Subscription Parity Test

Purpose

Prove that time-aware query contexts and temporal subscription lowerings
preserve canonical query meaning while keeping historical truth basis distinct
from signal/runtime temporal execution basis.

Scenario

- use the concrete `EmployeeRecord` fixture from Milestones 9.1 through 9.3
- construct equivalent time-aware live queries through:
  - direct canonical construction
  - scope-composed construction
  - template-instantiated construction
  - saved-query exact reuse where admitted
  - unified facade helper construction
- exercise admitted temporal query families for:
  - stale-after detail and inspector queries
  - refresh-interval collection queries
  - rolling-window timeline queries
  - tolerance-aware chart queries with time-only reevaluation
- vary:
  - current, branch-head, snapshot, historical, and preview-scoped truth bases
  - monotonic clock, manual test clock, deadline, and interval wake classes
  - time-only wakes, truth-only patches, and truth-plus-time interleavings
  - masked and unmasked policy contexts
  - tenant schema variants
  - supported and unsupported temporal bridge basis requests

Required concrete lanes

- time-only stale-after lane where no truth patch occurs but delivery changes
  result freshness state through a query-shaped temporal cause
- historical-truth-versus-clock lane where the same query runs against a
  historical snapshot while temporal execution advances independently
- rolling-window lane where an entity enters or leaves a window because clock
  time advances, not because truth changed
- ambient-clock denial lane where a temporal query attempts to use an unbound
  host clock or implicit timer
- bridge-temporal-basis mismatch lane where query temporal basis and bridge
  temporal basis digests diverge before delivery

Must verify

- truth basis and temporal execution basis produce separate digests and cannot
  be substituted for each other
- equivalent time-aware query declarations lower to the same temporal
  subscription declaration and bridge temporal basis request
- time-only deliveries are query-shaped and never expose raw signal wake events
- historical truth reads do not advance merely because temporal execution time
  advances
- temporal wakes do not widen projection, bypass policy masks, or rescan
  unrelated truth outside declared query scope
- previous-value comparison basis is explicit where temporal delivery depends
  on prior result state
- unsupported clock classes, unbound timers, ambient host timers, and
  unsupported temporal bridge basis requests fail typed and early
- diagnostics localize temporal denial to query basis binding, bridge temporal
  lowering, signal temporal strategy, previous-value basis, or support
  metadata
- small/medium/larger fixture runs prove temporal delivery cost slopes are
  bounded by declared temporal wake width, projection width, affected result
  width, and previous-value comparison width rather than unrelated row count

Required verification output

- `query_digest`
- `subscription_declaration_digest`
- `subscription_equivalence_digest`
- `basis_digest`
- `temporal_basis_digest`
- `bridge_temporal_basis_digest`
- `signal_strategy_digest`
- `previous_value_basis_digest`
- `result_digest`
- `delivery_digest`
- `temporal_delivery_digest`
- `diagnostic_trace_digest`
- `failure_digest`
- `counter_snapshot`
- `temporal_query_scale_slope_digest`
- `compile_fail_boundary_digest`

Pass condition

Temporal query basis binding and time-aware subscription lowering remain
query-owned, bridge-honest, time-travel-honest, and fail-closed before ambient
clock or raw signal timing can leak into query results.

## Milestone 9.5 Named Certification Suites

### 9.5. Phase 1 Named Scope Expansion Productization Closure Test

Purpose

Prove that runtime-backed named scope expansion is a verified composition lane:
equivalent direct construction and admitted scope-composed construction must
produce the same canonical query meaning while preserving explicit scope
lineage, typed basis evidence, and honest support-profile posture.

Scenario

- construct equivalent detail and collection queries through:
  - direct canonical authoring
  - named predicate scope expansion
  - named ordering scope expansion
  - named projection scope expansion
  - named traversal-bound scope expansion
  - basis-aware scope expansion where admitted
  - admitted multi-scope composition combining projection, ordering, and
    traversal-bound scopes in one path
- vary:
  - detail and collection query families
  - single-scope and multi-scope expansion
  - current-basis evidence on basis-aware scope paths
  - illegal traversal widening
  - unsupported scope-family admission

Required concrete lanes

- predicate-scope parity lane where direct and scope-composed detail queries
  converge on the same canonical query digest
- multi-scope collection parity lane where projection, ordering, and
  traversal-bound scopes converge on the same canonical query and result-shape
  digests as equivalent direct construction
- basis-aware parity lane where admitted basis evidence preserves canonical
  query meaning while emitting explicit basis metadata
- basis-aware mismatch denial lane where evidence bound to a different canonical
  query fails typed and early
- traversal-widening denial lane where a traversal-bound scope attempts to
  exceed its declared depth bound
- unsupported-scope denial lane where non-admitted scope families fail before
  authored-request lowering

Must verify

- named scope expansion is canonical declaration composition rather than string
  substitution or caller-owned rewrite folklore
- equivalent direct and scope-composed declarations produce the same canonical
  `query_digest`
- result-shape parity holds for admitted scope families on the covered paths
- `scope_lineage_digest` is explicit and non-empty on admitted named-scope
  expansion artifacts
- basis-aware scope evidence stays typed, query-bound, and fail-closed
- admitted named-scope paths keep `scope_rediscovery_count == 0`
- support/profile publication reports `named_scope_expansion:verified`
- diagnostics localize denial to unsupported scope family, illegal widening, or
  basis-evidence query mismatch rather than broad composition failure
- small/medium/larger fixture runs prove scope expansion cost stays bounded by
  declared scope count and scope width rather than unrelated row or schema
  breadth

Required verification output

- `query_digest`
- `result_shape_digest`
- `composition_digest`
- `scope_lineage_digest`
- `basis_digest`
- `support_profile_digest`
- `failure_digest`
- `counter_snapshot`
- `scope_expansion_scale_slope_digest`
- `compile_fail_boundary_digest`

Pass condition

Named scope expansion is certified only when admitted scope families preserve
canonical query meaning, typed basis evidence, explicit lineage, zero
rediscovery on the ordinary path, and a public `verified` support posture while
unsupported or widening neighbors fail typed and early.

### 9.5. Phase 2 Template Instantiation Productization Closure Test

Purpose

Prove that runtime-backed template instantiation is a verified composition
lane: equivalent direct construction and admitted template-instantiated
construction must produce the same canonical query meaning while preserving
explicit typed binding artifacts, exact binding counters, and honest
support-profile posture.

Scenario

- construct equivalent detail and collection queries through:
  - direct canonical authoring
  - detail template instantiation with typed predicate binding
  - collection template instantiation with typed traversal binding
  - basis-aware template instantiation where admitted
- vary:
  - detail and collection query families
  - binding kinds across predicate, ordering, projection, and traversal slots
  - missing, duplicate, undeclared, and kind-mismatched bindings
  - duplicate slot declaration
  - deferred template-family admission for observed inspector, focused
    inspector, and grouped collection template families

Required concrete lanes

- detail-template parity lane where direct and template-instantiated detail
  queries converge on the same canonical query and result-shape digests
- collection-template parity lane where direct and template-instantiated
  collection queries converge on the same canonical query and result-shape
  digests
- basis-aware template parity lane where admitted basis evidence remains bound
  to the fully instantiated canonical query and emits explicit basis metadata
- missing-binding denial lane where a declared slot lacks a bound value
- duplicate-binding denial lane where one slot receives multiple bindings
- undeclared-slot denial lane where a binding targets a slot the template did
  not declare
- binding-kind mismatch lane where slot kind and binding kind diverge
- duplicate-slot declaration lane where the same slot is declared twice on one
  template
- deferred-template-family lane where observed inspector, focused inspector,
  and grouped collection template families remain typed-and-early deferred

Must verify

- template instantiation is canonical declaration composition rather than
  caller-owned rewrite folklore
- equivalent direct and template-instantiated declarations produce the same
  canonical `query_digest`
- result-shape parity holds for admitted template families on the covered
  paths
- `template_binding_digest` is explicit and non-empty on admitted
  template-instantiation artifacts
- composition reports preserve the same binding digest as the emitted
  instantiation artifact
- admitted template-instantiated paths keep `template_rediscovery_count == 0`
- basis-aware template evidence stays typed, canonical-query-bound, and
  fail-closed
- diagnostics localize denial to missing binding, duplicate binding, undeclared
  slot, binding-kind mismatch, duplicate slot declaration, or deferred
  template-family posture rather than broad composition failure
- support/profile publication reports `template_instantiation:verified`
- small/medium/larger fixture runs prove template instantiation cost stays
  bounded by declared slot count and binding width rather than unrelated row or
  schema breadth

Required verification output

- `query_digest`
- `result_shape_digest`
- `composition_digest`
- `template_binding_digest`
- `basis_digest`
- `support_profile_digest`
- `failure_digest`
- `counter_snapshot`
- `template_instantiation_scale_slope_digest`
- `compile_fail_boundary_digest`

Pass condition

Template instantiation is certified only when admitted template families
preserve canonical query meaning, typed binding identity, exact slot and
binding counters, zero rediscovery on the ordinary path, and a public
`verified` support posture while malformed or deferred neighbors fail typed and
early.

## Milestone 9.6 Named Certification Suites

### 9.6. Bridge Truth Identity Hard Exposure Gate Test

Purpose

Prove that the bridge-truth identity lowering milestone begins with a real
compile-time break at the public string folklore boundaries rather than a soft
migration that leaves string construction or string receipt fields available to
ordinary callers.

Scenario

- exercise public caller attempts to:
  - construct bridge truth identities from raw string literals
  - read bridge truth identities back as raw string text
  - construct Query mutation receipts and deltas with public string fields
  - implement runtime backend/source adapter snapshot-token methods returning
    owned strings
- run the hard-break workspace exposure command after deleting the public
  string gates
- compare the surfaced errors against the milestone Collapse Matrix

Required concrete lanes

- bridge facade compile-fail lane where `TruthCommitIdentity::new("commit-1")`
  and `TruthCommitIdentity::as_str()` are rejected for an external caller
- query receipt compile-fail lane where struct literals with
  `commit_identity: String`, `snapshot_token: String`, and
  `entity_identity: String` have installed trybuild fixtures and expected
  stderr during the red exposure phase, then execute once the query crate
  compile frontier reaches the fixture runner
- adapter trait compile-fail lane where `snapshot_token(&self) -> String` is no
  longer a trait member, with installed trybuild fixtures during the red
  exposure phase and execution once the query crate compile frontier reaches
  the fixture runner
- workspace-red exposure lane where `cargo check --workspace --keep-going`
  records the first hard dependency frontier and maps every surfaced error to
  the Collapse Matrix

Must verify

- the hard break is applied at bridge/query facade choke points only
- no downstream production call sites are fixed in the hard-break phase
- the workspace is red after the gates land
- query compile-fail fixtures and expected stderr are installed before Phase 2
  closes; their execution is a hard Phase 5/6 certification gate because Phase
  2 intentionally leaves `worth-relational` red
- every surfaced compile error is grouped by crate and error kind in
  `plans/worth-query/bridge_truth_identity_exposure_report.md`
- any surfaced path missing from the Collapse Matrix is added before the phase
  closes
- the exposure report is a one-time break catalog, not an incremental greening
  scoreboard

Required verification output

- `bridge_truth_identity_compile_fail_boundary_digest`
- `query_receipt_string_field_compile_fail_boundary_digest`
- `adapter_snapshot_token_compile_fail_boundary_digest`
- `workspace_red_exposure_digest`
- `collapse_matrix_cross_check_digest`

Pass condition

The hard exposure gate is certified only when ordinary external callers cannot
mint or read truth-routing identity through strings, Query receipt string
fields are not public construction authority, adapter snapshot-token string
methods are gone, and the resulting workspace-red exposure is fully cataloged
against the authoritative Collapse Matrix.

### 9.6. Phase 1 Canonical Evidence Identity Stability Test

Purpose

Prove that the first public runtime-backed evidence-identity surfaces stop
teaching caller-owned string folklore and instead emit one Query-owned,
scheme-versioned canonical evidence identity with typed same-scheme comparison
semantics.

Scenario

- exercise the first covered runtime-backed evidence surfaces for:
  - runtime public support matrix row identity
  - runtime public support matrix aggregate identity
  - runtime state snapshot identity
  - preview intent admission identity
  - preview intent receipt identity
  - intent denial evidence identity
- compare:
  - runtime-emitted evidence identities
  - independently composed evidence identities through the public
    `WORTHQueryEvidenceIdentity::compose(...)` surface
- vary:
  - punctuation-heavy field values including pipe, colon, and tag-shaped text
  - equivalent semantic fields across different surface scopes
  - hostile sequence layouts that would collide under joined-string folklore
  - alternate evidence-identity scheme versions for typed mismatch behavior

Required concrete lanes

- support-matrix parity lane where runtime-emitted row and matrix identities
  are scheme-versioned canonical tokens
- state-snapshot parity lane where basis, result-shape, lane, and explanation
  identity remain canonical under punctuation-heavy values
- preview admission/receipt lane where Query-owned preview evidence identity is
  emitted without caller-owned digest assembly
- intent denial lane where invariant evidence with hostile delimiter content
  does not collapse distinct evidence sets
- cross-scheme comparison lane where same-scope identities with different
  scheme versions fail typed rather than comparing raw bytes

Must verify

- every covered phase-1 surface emits a Query-owned evidence identity token
  rather than an unversioned raw digest string
- the evidence identity token carries scheme identity in the value itself so
  scheme drift is detectable from the emitted token alone
- the public constructor surface is real and parity-capable from birth rather
  than being runtime-internal helper plumbing
- hostile delimiter and separator content cannot collapse distinct field sets
  into one identity
- same-scheme comparison is explicit and cross-scheme comparison fails typed
  rather than degrading into byte comparison
- the covered runtime-backed product surfaces no longer require caller-owned
  `hash_parts(...)`, `Debug`, `Display`, or joined-string identity folklore to
  produce machine-checkable evidence identity

Required verification output

- `evidence_scope_digest`
- `evidence_identity_token`
- `scheme_version_token`
- `support_matrix_digest`
- `state_digest`
- `preview_admission_digest`
- `preview_receipt_digest`
- `intent_denial_digest`
- `failure_digest`

Pass condition

Phase 1 closes only when the first ordinary runtime-backed evidence surfaces
emit scheme-versioned canonical evidence identity through a public,
proof-carrying constructor lane, and hostile delimiter or scheme drift
pressure cannot silently collapse or miscompare identities.

### 9.6. Phase 2 Query Digest Surface Migration Closure Test

Purpose

Prove that the covered Query-owned digest surfaces stop teaching string-joined
digest folklore and instead lower through the runtime-owned canonical
evidence-identity primitive introduced in Phase 1.

Scenario

- exercise the covered Query-owned digest surfaces for:
  - runtime public API family contract digest
  - runtime public API aggregate contract digest
  - runtime public support matrix row and aggregate digests
  - runtime state snapshot digest
  - runtime public API transcript digest
  - application support report digest
- compare:
  - runtime-emitted digests from the ordinary product surfaces
  - independently recomposed digests through the public
    `WORTHQueryEvidenceIdentity::compose(...)` surface
- vary:
  - punctuation-heavy identity fields, including pipe, colon, and tag-shaped
    values
  - supported, deferred, and unsupported public API family rows
  - optional support report profile publication presence and absence
  - support-gated transcript denial digest sequences

Required concrete lanes

- public-api-contract parity lane where family-contract and aggregate-contract
  digests recompose exactly from the same typed evidence
- support-matrix parity lane where the runtime-emitted public support matrix
  row and aggregate digests lower through canonical evidence identity
- state-snapshot parity lane where state digests recompose exactly from typed
  lane, basis, result-shape, explanation, and optional posture inputs
- transcript parity lane where runtime public API transcript evidence lowers
  through canonical identity even under punctuation-heavy neighbor-denial
  digest sequences
- support-report parity lane where support report publication posture changes
  only the declared support-report identity fields while preserving canonical
  recomposition
- residue-audit lane where the covered Query source surfaces prove zero
  remaining `hash_parts(...)` digest construction through an exact structural
  assertion

Must verify

- every covered Phase 2 digest surface emits a scheme-versioned canonical
  evidence identity token rather than a format-string digest
- independently recomposing the same typed evidence reproduces each covered
  digest exactly
- no covered surface preserves the old string-joined digest value by
  re-encoding old separator folklore inside the new primitive
- optional publication surfaces such as support-report profiles participate in
  digest identity through explicit optional identity fields rather than ad hoc
  string placeholders
- the covered Query surfaces contain zero remaining `hash_parts(...)`,
  `Debug`, `Display`, or joined-string digest construction for the covered
  identity lane

Required verification output

- `public_api_family_contract_digest`
- `public_api_contract_digest`
- `support_matrix_digest`
- `state_digest`
- `public_api_transcript_digest`
- `support_report_digest`
- `scheme_version_token`
- `failure_digest`

Pass condition

Phase 2 closes only when every covered Query-owned digest surface lowers
through the canonical evidence-identity primitive, independently recomposes
exactly from typed evidence, and leaves zero format-string digest residue in
the covered source surfaces.

### 9.6. Phase 3 Typed Stop Class Taxonomy Test

Purpose

Prove that the existing `WORTHQueryRuntimeError` topology exposes one
runtime-owned typed stop-class accessor for covered denial and stop paths, so
consumer control flow can match on typed semantics instead of message text.

Scenario

- classify covered runtime stop paths for:
  - runtime bootstrap/component-missing failures
  - support/admission denials
  - existing-truth assertion, probe, binding, continuity, naming, and symbolic
    reference denials
  - graph composition and graph-composition domain-invariant denials
  - read composition and read domain-invariant denials
  - runtime lookup, missing-artifact, declaration-failure, and preview-promotion
    failures
  - effect-policy, unsupported-authority, and intent failure paths
- compare:
  - the original `WORTHQueryRuntimeError` payload
  - the typed `error.stop_class()` projection
- vary:
  - multiple denial kinds inside the same family
  - message rewording on message-bearing variants
  - future-variant drift pressure on the runtime error enum

Required concrete lanes

- denial-payload preservation lane where rich denial payloads remain reachable
  through the stop-class context rather than being flattened into generic tags
- unsupported-family lane where support posture, denied family, and reason stay
  typed across the stop-class boundary
- preview-promotion lane where all promotion failure variants converge on one
  typed promotion stop class carrying the original denial evidence
- unsupported-authority lane where authority denial becomes a named stop-class
  variant instead of caller-owned message probing
- completeness lane where the stop-class classifier matches the full covered
  runtime error enum with no wildcard escape hatch

Must verify

- every covered runtime stop path classifies to exactly one stop class
- the stop-class accessor is an accessor over `WORTHQueryRuntimeError`, not a
  second parallel error family
- rich denial payloads remain available through typed context on the stop class
- preview-promotion variants classify through one typed promotion class keyed by
  `WORTHQueryPreviewPromotionDenialKind`
- support/admission denials carry their denied facade family through typed
  payload access, not through message text
- changing message wording on message-bearing variants does not change the stop
  class
- adding a new covered runtime error variant requires an explicit stop-class
  mapping rather than silently falling into `Other`/`Unknown`

Required verification output

- `stop_class_digest`
- `support_denial_digest`
- `preview_promotion_digest`
- `intent_denial_digest`
- `failure_digest`

Pass condition

Phase 3 closes only when the covered runtime stop paths classify through one
typed stop-class accessor with payload-preserving context, zero catch-all
escape hatches, and zero control-flow dependence on message text.

### 9.6. Phase 4 Typed Stop Class Matching Closure Test

Purpose

Prove that real consumer-side control flow can handle every covered Query stop
class through typed matching alone, including public family-admission denials,
without string routing on denial presentation text.

Scenario

- route covered stop classes through a consumer-shaped matcher fed by:
  - manually constructed representative runtime errors for broad taxonomy
    coverage
  - runtime-generated public family-admission denials
  - runtime-generated read, intent, preview-promotion, and routing failures
- compare:
  - the runtime-owned `error.stop_class()` projection
  - the consumer-owned typed route decision
- vary:
  - support-gated public family denials
  - message wording on denial presentation text
  - runtime-generated versus manually constructed stop paths

Required concrete lanes

- public-family admission lane where `workspace.admit_public_api_family(...)`
  yields a typed denial carrying denied family, support status, teaching
  posture, and reason
- consumer-routing parity lane where a consumer-shaped router handles every
  covered stop class with type-level matching and zero string operations
- runtime-generated lane where preview-promotion, intent, routing, and read
  invariant failures route through the same consumer matcher without special
  message parsing
- message-drift lane where denial wording changes while typed matching stays
  stable and a prior wording probe fails
- residue-audit lane where the covered consumer route helper proves zero
  `error.to_string()` or substring routing in its ordinary control flow

Must verify

- public family-admission denials expose denied facade family, support status,
  teaching posture, and reason through typed stop-class payload access
- a consumer-shaped matcher can handle every covered stop class without calling
  `to_string()` or probing message substrings
- runtime-generated public/runtime entrypoints route through the same typed
  consumer lane as manually constructed representative errors
- changing denial wording does not break typed matching while a wording probe
  demonstrably drifts
- the covered consumer route helper contains zero string-matched control flow

Required verification output

- `consumer_stop_route_digest`
- `public_family_admission_digest`
- `runtime_generated_route_digest`
- `message_drift_probe_digest`
- `failure_digest`

Pass condition

Phase 4 closes only when a consumer-shaped typed matcher handles the covered
stop classes end to end, public family-admission denials expose their typed
payloads directly, message wording drift cannot break control flow, and the
ordinary consumer route helper contains zero string-matched control flow.

### 9.6. Phase 5 Canonical Session Label Artifact Test

Purpose

Prove that the session-label identity surface is a real typed artifact rather
than a validated display string: namespace, ordered name segments, and
canonical evidence-identity participation must determine identity under hostile
construction and collision pressure.

Scenario

- construct session labels through:
  - typed namespace plus typed segment construction
  - convenience string-based construction through the public surface
- compare:
  - artifact equality
  - independently recomposed canonical evidence identity
  - display rendering as a non-authoritative projection
- vary:
  - equivalent semantic labels across different construction paths
  - ordered segment permutations
  - namespace changes with identical name segments
  - render-collision cases where distinct typed labels produce the same dotted
    display string
  - empty namespace, empty name-segment, and missing-segment invalid inputs

Required concrete lanes

- construction-path parity lane where typed and convenience construction
  produce the same label identity and canonical digest participation
- ordered-segment drift lane where the same segment set in a different order
  produces a distinct label identity and distinct digest
- namespace drift lane where identical name segments under a different
  namespace remain a distinct label identity and digest
- render-collision lane where two distinct typed labels render to the same
  display string but remain distinct artifacts with distinct digests
- invalid-input lane where empty namespace, empty segment, and missing segment
  inputs fail typed and early

Must verify

- session label identity is determined by typed namespace plus ordered typed
  name segments rather than by display rendering
- equivalent semantic labels produce the same canonical evidence identity
  regardless of construction path
- display rendering is a projection over the artifact and does not participate
  in equality
- namespace and segment ordering both participate in identity and digest
  derivation
- render-collision pressure cannot collapse distinct label identities
- invalid label parts fail typed and early through the public constructor
  surface

Required verification output

- `session_label_identity_digest`
- `session_label_scope_token`
- `session_label_display`
- `failure_digest`

Pass condition

Phase 5 closes only when session labels behave as canonical typed identity
artifacts whose equality and digest participation survive hostile construction,
ordering drift, namespace drift, render-collision pressure, and invalid input
attempts.

### 9.6. Phase 6 Canonical Session Label Intake Test

Purpose

Prove that preview and branch session entry admit only canonical typed session
labels, record label identity through basis-admission evidence, and stop
equivalent label replay with a typed collision class instead of silently
merging on rendered strings.

Scenario

- admit preview and branch sessions through the ordinary public workspace and
  runtime entrypoints using typed session labels
- compare basis-admission evidence against independently recomposed canonical
  evidence identities built from the admitted session-label identity
- replay equivalent session-label identities within the same session family
- replay the same session-label identity across different session families
- admit distinct typed labels that render to the same dotted display string

Required concrete lanes

- preview-admission identity lane where preview basis admission stores the
  canonical session-label identity and recomposes to the same digest
- branch-admission identity lane where branch basis admission stores the
  canonical session-label identity and recomposes to the same digest
- same-family replay collision lane where re-admitting an equivalent preview or
  branch label stops with `WORTHQueryStopClass::SessionLabelCollision`
- cross-family coexistence lane where the same session-label identity may be
  admitted once in preview and once in branch without a fake global collision
- render-collision lane where two distinct typed labels with the same display
  string admit independently because collision scope is identity-based, not
  display-based
- raw-string eradication lane where ordinary-path preview and branch entry call
  sites no longer pass free-form strings

Must verify

- ordinary-path preview and branch entry requires `WORTHQuerySessionLabel`
  rather than `impl Into<String>`
- preview and branch basis-admission digests record
  `session_label_identity` rather than rendered string folklore
- equivalent label replay within the same session family stops with a typed
  session-label collision class
- collision posture is scoped per session family rather than enforced through a
  global workspace registry
- display-colliding but identity-distinct labels do not silently merge or
  falsely collide

Required verification output

- `preview_session_basis_digest`
- `branch_session_basis_digest`
- `session_label_collision_stop_class`
- `render_collision_admission_digest`
- `raw_string_entrypoint_audit`

Pass condition

Phase 6 closes only when preview and branch session entry are typed-label
boundaries, basis admission evidence records canonical session-label identity,
same-family replay collisions stop through a typed class, and no ordinary-path
entrypoint remains on raw strings.

### 9.6. Phase 7 Milestone 9.6 Identity And Stop-Class Hostile Certification Matrix

Purpose

Prove that evidence identity, typed stop-class matching, and session-label
identity boundaries hold together under combined drift pressure and that the
support report publishes derived zero-folklore residue only when the covered
inventory scan is clean.

Scenario

- run the identity-boundary hostile closure matrix across:
  - digest delimiter injection pressure on basis admissions
  - message rewording on family-admission and domain-invariant denials
  - session-label render collision and same-family replay collision
- compare:
  - support-report `identity_boundary_closure()` artifacts
  - inventory-derived residue status
  - hostile-matrix digest publication
- verify:
  - covered inventory paths contain zero `hash_parts(` digest folklore
  - consumer route helper contains zero string-matched control flow
  - ordinary session entrypoints require typed labels under CRLF-normalized audits

Required concrete lanes

- combined drift lane where digest, message, and label pressure run in one program
- derived residue lane where `residue_status` is computed from inventory scans
- hostile-matrix registration lane with canonical and rejection row names
- certification-output hygiene lane where milestone 9.6 certification modules do
  not call `hash_parts`

Must verify

- named suite **"Milestone 9.6 Identity And Stop-Class Hostile Certification Matrix"**
  is registered in certification requirements
- support report publishes `hostile_matrix_digest` alongside closure digests
- inventory module is the single owner of covered path lists
- excluded folklore paths are documented explicitly outside milestone scope

Required verification output

- `identity_boundary_closure_digest`
- `hostile_matrix_digest`
- `residue_status_token`
- `consumer_stop_route_digest`
- `session_label_identity_digest`
- `failure_digest`

Pass condition

Phase 7 closes only when the hostile certification matrix, derived residue
reporting, and inventory-backed exact-zero audits agree the three identity
boundaries are closed ordinary product surfaces.

### 9.6. Mixed Truth Time Async Query Delivery Ordering Test

Purpose

Prove that mixed truth, temporal, async, retry, cancellation, policy, tenant,
preview, promotion, and discard causes produce one deterministic query-shaped
delivery stream whose meaning is independent of host event arrival order.

Scenario

- activate admitted temporal and async query subscriptions from Milestones 9.4
  and 9.5
- deliver equivalent cause sets through multiple arrival orders:
  - truth patch before temporal wake
  - temporal wake before truth patch
  - async completion racing with truth patch
  - cancellation racing with completion
  - retry racing with policy or tenant remask
  - preview discard racing with temporal/async residue
  - preview promotion followed by authoritative rebinding
- compare live delivery to replay of the canonical cause sequence
- compare final live result to fresh query execution for the same basis

Required concrete lanes

- truth-plus-time lane where a field patch and a stale-after wake coalesce into
  one deterministic query-shaped delivery
- async-plus-truth lane where stale completion is rejected because the truth
  patch changes the query basis first
- cancellation-plus-completion lane where two host arrival orders produce the
  same canonical cancellation result
- policy-remask-plus-wake lane where a temporal wake cannot reveal a newly
  masked aspect
- preview-discard residue lane where all temporal and async residue attached
  to the preview basis is closed out with zero authoritative residue
- unsupported-composition lane where a mixed-cause combination with no admitted
  ordering rule fails before delivery

Must verify

- host event arrival order variation does not change canonical delivery digest
  for admitted mixed-cause families
- canonical cause ordering is explicit and replayable
- coalescing and suppression decisions carry receipts and cannot erase
  semantic differences
- mixed-cause deliveries remain query-shaped and never expose raw CDC, raw
  signal wake events, raw resource completions, or transport-local events as
  the consumer contract
- policy and tenant masking hold under every admitted cause ordering
- preview promotion mints new authoritative temporal/async basis evidence
  rather than mutating preview residue in place
- preview discard proves zero authoritative residue for temporal wakes, async
  generations, retry state, cancellation state, diagnostic state, and delivery
  windows
- unsupported mixed-cause compositions fail typed and early rather than
  choosing host event order as an implicit policy
- replay of the same canonical cause sequence produces identical query,
  result, delivery, and diagnostic digests
- small/medium/larger fixture runs prove delivery ordering cost is bounded by
  declared cause width, coalescing width, suppressed group width, preview
  residue width, and affected result width rather than unrelated row count

Required verification output

- `query_digest`
- `basis_digest`
- `temporal_basis_digest`
- `async_resource_digest`
- `cause_ordering_digest`
- `coalescing_receipt_digest`
- `suppression_receipt_digest`
- `preview_residue_digest`
- `policy_digest`
- `tenant_basis_digest`
- `result_digest`
- `delivery_digest`
- `replay_digest`
- `failure_digest`
- `diagnostic_trace_digest`
- `counter_snapshot`
- `mixed_cause_scale_slope_digest`
- `compile_fail_boundary_digest`

Pass condition

Mixed truth/time/async delivery remains deterministic, replayable,
query-shaped, policy-safe, and fail-closed before unsupported interleavings can
be interpreted by host arrival order.

## Milestone 9.7 Named Certification Suites

### 9.7. Generation Pinning Hot-Path Lock Posture Test

Purpose

Prove that shared-read generation pinning is a real runtime-owned substrate:
committed-read context minting does not acquire hot-path locks, retired
generations drain through explicit lifecycle accounting, and published artifact
generations are retained only while their pinned read generations require them.

Scenario

- drive repeated shared-read context minting across sustained commit and
  derived publication pressure
- hold an old shared-read context across a newer commit and prove it continues
  to resolve the old published artifact without observing a blend
- drop old leases, advance ordinary publication, and prove both pin registry
  and published-artifact generation residue drain through runtime diagnostics
- perturb the release-compiled certification hot-path lock measurement and
  prove Phase 12 posture reopens
- verify the pinning inventory names every ordinary pin, release, retire, drain,
  retain, resolve, and hot-path measurement operation

Must verify

- committed-read hot-path lock acquisitions remain exact-zero under the hostile
  schedule
- old pinned generations are explicitly retired but retained while their leases
  exist
- old published artifact generations remain resolvable for the pinned lifetime
  and are dropped after ordinary generation retention advances
- final runtime-owned counters show exact-zero orphaned generations and
  exact-zero unretired pins
- certification uses production runtime diagnostics and release-compiled
  measurement hooks rather than `cfg(test)`-only counter fiction

Required verification output

- `shared_read_hot_path_counter_snapshot`
- `shared_read_pin_lifecycle_diagnostic_digest`
- `published_artifact_generation_retention_digest`
- `shared_read_pinning_operation_inventory_digest`
- `failure_digest`

Pass condition

Phase 12 closes only when generation pinning, explicit retirement, published
artifact retention, hot-path lock measurement, and inventory completeness agree
through runtime-owned diagnostics. Full sealed shared-read context boundary
closure remains Phase 13.

### 9.7. Shared Read Context And Pinning Boundary Closure Test

Purpose

Prove that `WORTHQuerySharedReadContext` is the sealed, basis-bound product
surface for shared-read consumption and that the complete pinning boundary is
closed before journal and certification phases build on it.

Scenario

- mint multiple shared-read contexts at the same generation and prove basis
  inspection, published artifact identity, and consumed facts are identical
- drive sustained commit and publication pressure while old legal contexts
  continue to observe their pinned generation rather than rebinding
- explicitly invalidate an old basis and prove the context fails with typed
  stale-basis denial instead of resolving newer registry content
- prove `WORTHQuerySharedReadContext` and published artifact handles satisfy
  `Send + Sync` and resolve through `std::thread::scope` without wrappers
- derive the shared-read pinning boundary closure posture from inventory,
  hostile matrix, portability proof, stale-denial proof, and runtime counters
- perturb each closure input and prove the posture no longer reports `Closed`

Must verify

- shared-read context basis inspection is typed and stable for the context's
  legal lifetime
- re-minting through the workspace is the only way to observe a newer committed
  generation
- stale contexts fail closed with `SharedReadStaleBasis` carrying the original
  snapshot identity
- full pinning-boundary inventory covers shared-read authority, artifact
  consumption, published artifact retention, diagnostics, and pin registry
  paths
- exact-zero hot-path locks, orphaned generations, and unretired pins come from
  runtime-owned counters after the hostile matrix
- support/profile posture reports `Closed` only when all closure inputs are
  green

Required verification output

- `shared_read_basis_digest`
- `shared_read_artifact_equivalence_digest`
- `shared_read_stale_basis_denial_digest`
- `shared_read_send_sync_proof_digest`
- `shared_read_pinning_boundary_inventory_digest`
- `shared_read_pinning_boundary_counter_digest`
- `shared_read_pinning_boundary_closure_digest`
- `failure_digest`

Pass condition

Phase 13 closes only when the real shared-read context type is sealed,
basis-bound, portable across scoped threads, stale-basis typed, inventory
complete, and certified by exact-zero runtime residue counters. Later journal
and certification phases may assume shared-read pinning is closed.

### 9.7. Typed Journal Position Identity Boundary Test

Purpose

Prove that committed write receipts expose journal order as a sealed typed
runtime artifact, not as a parsed convention from commit identity display text
or mutation receipt formatting.

Scenario

- submit authoritative writes through the workspace submission lane and inspect
  `WORTHQueryWriteReceipt::journal_position()`
- execute a batch write and inspect `WORTHQueryBatchWriteReceipt::journal_positions()`
- replay the same multi-write schedule in a fresh runtime and compare typed
  journal-position evidence identities
- derive certification from journal identity inventory scans plus schedule
  evidence, then perturb each input and prove the posture opens

Must verify

- committed journal positions are minted from typed bridge commit payloads
  rather than `commit_identity` strings, suffixes, digit runs, display text, or
  test-only helpers
- journal position identity is distinct from both commit identity and evidence
  identity while remaining evidence-addressable
- batch receipts carry typed journal positions in component order and include
  journal-position identities in the batch digest basis
- replay produces the same typed position sequence for the same authoritative
  schedule
- inventory scans cover minting, receipt carry, batch carry, and certification
  paths with exact-zero forbidden parsing residue

Required verification output

- `journal_position_identity_digest`
- `journal_position_schedule_digest`
- `journal_identity_inventory_digest`
- `journal_identity_certification_digest`
- `failure_digest`

Pass condition

Phase 14 closes only when journal order is a runtime-carried typed artifact,
authoritative submissions and batches expose it without string parsing, replay
stability and collision checks are proven, and inventory certification opens on
any sabotaged proof input.

### 9.7. Consumer Journal Segment Replay Surface Test

Purpose

Prove that consumers replay typed journal segments through the runtime-owned
workspace facade, and that replay reconstructs the same ordinary truth artifacts
without exposing raw journal internals or introducing a second semantics path.

Scenario

- submit a downstream-shaped write workload through the public workspace surface
- derive a typed journal segment identity from committed receipt positions
- replay the segment through a typed `WORTHQueryJournalReplayRequest`
- compare replay receipts, journal schedule, truth digest, published artifact
  digest, and replay outcome digest against the committed lane
- deny stale-basis, unknown segment, cross-scheme, and gapped replay requests
  with typed errors and no journal residue
- derive journal boundary posture from inventory, schedule, and replay evidence
- sabotage gap evidence and truth evidence independently

Must verify

- the consumer replay surface accepts typed segment identity and returns ordinary
  write receipts and replay outcome evidence
- replay truth digest equals committed truth digest and includes receipt deltas,
  expected position count, resolved position count, and gap count
- published artifact digest and replay outcome digest are stable for identical
  replay inputs
- stale basis, unknown segment identity, cross-scheme identity, and journal gaps
  fail through typed replay denial kinds
- journal identity inventory covers replay, certification, support, and facade
  paths that read or compare journal order
- forbidden scans remain green for commit-identity parsing and string-derived
  journal gap counting across the full journal boundary
- journal boundary posture reports `Closed` only when inventory, schedule, and
  replay proof are simultaneously green, and reports `Partial` when gap or truth
  sabotage reopens the proof

Required verification output

- `journal_segment_identity_digest`
- `journal_replay_outcome_digest`
- `journal_replay_truth_digest`
- `published_artifact_replay_digest`
- `journal_identity_inventory_digest`
- `journal_boundary_posture`
- `failure_digest`
- `counter_snapshot`

Pass condition

Phase 15 closes only when journal segment replay is a typed consumer facade
operation, replay equivalence is proven from ordinary receipt/envelope
artifacts, all adversarial replay denials are typed and residue-free, and the
journal boundary posture can close only from green inventory plus green replay
truth proof.

### 9.7. Temporal Async Query Certification Matrix Sufficiency Test

Purpose

Prove that the runtime-backed temporal/async query surface is certified by
hostile bundles, support metadata, diagnostics, exact counters, and a reference
workload before store-backed and durable milestones build on top of it.

Scenario

- run a temporal/async certification matrix covering Milestones 9.4, 9.5, and
  9.6
- compare:
  - shipped temporal/async support metadata
  - executable admission behavior
  - bridge/signal lowering digests
  - named certification suite coverage
  - roadmap Vision Coverage Appendix rows
- execute a reference workload containing:
  - branch-scoped live query
  - time-only wake
  - truth patch plus temporal wake
  - async success
  - async failure
  - cancellation
  - retry
  - stale completion
  - supersession
  - preview promotion
  - preview discard
  - policy remask
  - tenant remask
  - unsupported ambient clock
  - unsupported resource family
  - unsupported mixed-cause composition

Must verify

- every advertised runtime-backed temporal/async query capability has at least
  one admitted row, one hostile row, and one unsupported-neighbor row
- temporal basis, async resource basis, cause ordering, support metadata,
  diagnostics, and certification coverage bind the same canonical query and
  subscription declaration digests
- diagnostic bundles are sufficient for offline localization without re-running
  Query, Bridge, Signal, or a host runtime
- support metadata cannot advertise uncertified temporal, async, or
  mixed-cause families
- durable restore, persisted inflight resource continuation, store-backed
  temporal replay, and durable mixed-cause continuation remain deferred or
  denied until Milestones 10 and 11
- the reference workload emits machine-checkable bundles rather than relying on
  visual inspection, logs, or self-comparison within one run
- exact counters prove certification coverage and support lookup cost are
  bounded by declared family coverage width rather than unrelated fixture size
- compile-fail boundaries prove external callers cannot mint certification
  bundles, support rows, temporal basis proofs, async completion causality, or
  mixed-cause ordering receipts

Required verification output

- `certification_bundle_digest`
- `coverage_matrix_digest`
- `support_matrix_digest`
- `capability_registry_digest`
- `query_digest`
- `subscription_declaration_digest`
- `temporal_basis_digest`
- `async_resource_digest`
- `cause_ordering_digest`
- `diagnostic_trace_digest`
- `admitted_diagnostic_bundle_digest`
- `denied_diagnostic_bundle_digest`
- `reference_workload_digest`
- `failure_digest`
- `counter_snapshot`
- `temporal_async_support_enforcement_report`
- `compile_fail_boundary_digest`

Pass condition

The temporal/async query surface is certified only when support claims,
admission behavior, diagnostic bundles, bridge/signal lowering, hostile
coverage, and reference workloads agree, while durable and store-backed claims
remain explicit later-milestone debt.

### 9.7. Published Artifact Reader Isolation Test

Purpose

Prove that the Phase 8 shared-read lane consumes only maintenance-owner
published derived artifacts, preserves typed async posture for unpublished or
republishing views, and never evaluates derived state from a reader path.

Scenario

- mint sealed shared read contexts from the workspace after declaration-only,
  after first publication, and during republication pressure
- compare:
  - reader-side projection-consumption results through typed published-artifact
    handles
  - serialized maintenance-owner publication order and receipts for the same
    schedule
- probe:
  - declared-but-unpublished derived handles
  - foreign derived handles from another runtime
  - republishing views with pending patches and refresh fallback posture

Must verify

- declared-but-unpublished derived handles surface typed `pending` async
  result-state instead of materialized facts
- foreign or unknown derived handles fail closed as missing runtime artifacts
  rather than masquerading as unpublished publication posture
- published derived artifacts consume through the projection-consumption lane
  with receipt-backed fact content identical to a serialized consumer of the
  same publication schedule
- reader consumption observes either the old published artifact or the new one
  during republication, never a blend
- exact-zero reader-side evaluation counters prove no shared-read path triggers
  derived reevaluation

Required verification output

- `shared_read_snapshot_token`
- `published_artifact_binding_digest`
- `publication_receipt_digest`
- `async_result_state_digest`
- `reader_isolation_counter_snapshot`
- `failure_digest`

Pass condition

Phase 8 closes only when shared-read consumers are publication-bound,
async-posture honest, fail closed for foreign handles, and unable to trigger
derived evaluation from any reader-reachable path.

### 9.7. Facade Lane Parity And Lifecycle Propagation Test

Purpose

Prove that `WORTHQueryWorkspace` remains the single-owner public convenience
facade while the new shared-read and submission lanes stay support-honest,
path-parity honest, and compile-time sealed against downstream topology leaks.

Scenario

- execute the same covered submission operation through:
  - the existing workspace convenience write surface
  - the new `workspace.submissions()` lane
- mint shared-read artifacts through:
  - the workspace-owned `shared_read_context()` mint point
  - the runtime-owned decomposed shared-read context inside the runtime test
    boundary
- probe compile-fail boundaries for:
  - direct construction of the submission lane
  - direct access to the submission lane's decomposed runtime internals

Must verify

- the submission lane enters the public support matrix as an ordinary admitted
  facade family rather than piggybacking on `Write` or `Intent` vocabulary
- the shared-read lane enters the public support matrix as an ordinary admitted
  facade family rather than piggybacking on `Computed`
- equivalent submission work through the workspace convenience path and the
  submission lane produces identical receipt identity and mutation-summary
  digest
- the workspace-owned shared-read mint point produces the same published
  artifact handle as the decomposed runtime-owned mint point
- shared-read minting parity does not trigger extra derived maintenance work
- downstream callers cannot construct the submission lane or reach the runtime
  hidden behind it

Required verification output

- `submission_receipt_digest`
- `submission_mutation_summary_digest`
- `shared_read_snapshot_token`
- `shared_read_artifact_binding_digest`
- `shared_read_recomputation_count`
- `compile_fail_boundary_digest`

Pass condition

Phase 9 closes only when the workspace facade preserves existing call sites,
the new shared-read and submission lanes are explicit public support rows,
parity holds against the decomposed authorities, and downstream code cannot
reach past the facade.

### 9.7. Real Concurrent Hostile Certification Matrix Test

Purpose

Prove that the milestone closes on one hostile certification boundary rather
than a bag of isolated lane tests: interleaved readers, submissions, preview
churn, branch churn, derived republication, and replay must lower to one
machine-checkable artifact with exact-zero residue counters.

Scenario

- drive one runtime-backed hostile schedule that includes:
  - sealed shared-read consumption before publication, after publication, and
    across republication
  - repeated submission-lane writes under the same workspace
  - preview discard and preview promotion churn
  - repeated branch basis admission churn
- replay the same hostile schedule on a fresh runtime and compare the lowered
  certification artifacts byte-for-byte
- replay the same hostile schedule through both supported public
  runtime-bootstrap paths and prove the lowered certification artifact is
  bootstrap-path invariant
- repeat the hostile schedule again to prove run-to-run determinism

Must verify

- interleaved hostile execution and serialized replay produce identical
  certification artifacts
- repeated hostile runs produce the same certification artifact digest
- reader consumption remains bound to published artifacts and never triggers
  derived reevaluation
- preview and branch churn do not perturb authoritative receipt or published
  artifact identity
- exact counters remain at zero for committed-read hot-path locks,
  reader-triggered derived evaluation, orphaned snapshot generations,
  unretired read pins, journal gaps, and delivery residue

Required verification output

- `hostile_certification_digest`
- `receipt_digest`
- `reader_result_digest`
- `published_artifact_digest`
- `preview_closeout_digest`
- `branch_basis_digest`
- `counter_digest`

Pass condition

Milestone 9.7 closes only when the hostile certification artifact proves that
all covered lanes compose into one deterministic replay-stable boundary with
exact-zero residue and exact-zero reader-side reevaluation.

### 9.7. Public-Bridge Reader-Lane Honesty Closure Test

Purpose

Prove that public-bridge hostile certification consumes published derived facts
only through typed projection consumption and cannot silently fall back to
direct materialization row access.

Scenario

- execute the public-bridge hostile certification schedule through both:
  - common public runtime bootstrap
  - builder public runtime bootstrap
- consume every published derived artifact through the projection-consumption
  reader lane and issue receipt-backed evidence for the consumed facts
- inventory the public-bridge reader-lane helper for forbidden direct-read
  shortcuts
- sabotage the lane with the old row-spelunking pattern and require
  certification rejection
- compile-fail the public reader-lane certification boundary when a caller
  attempts to access a binding shortcut

Must verify

- common and builder bootstrap paths produce identical certification artifacts
- every consumed title is backed by a projection-consumption receipt digest
- public-bridge certification reports exact-zero direct materialization reads
- sabotage using `published_binding`, `materialization_by_name`, or `.rows()`
  is localized and rejected
- the public reader-lane certification type exposes no direct binding shortcut

Required verification output

- `public_bridge_hostile_certification_digest`
- `projection_consumption_receipt_digest`
- `published_artifact_digest`
- `direct_materialization_read_count`
- `sabotage_rejection_digest`
- `compile_fail_boundary_digest`

Pass condition

Phase 17 closes only when public-bridge certification proves replay-stable
published-artifact consumption through typed projection receipts and rejects
every direct materialization row shortcut at the certification boundary.

### 9.7. Derived Milestone Closure Posture Test

Purpose

Prove that Milestone 9.7 reports `Closed` only as a derived posture from
phase-local closure artifacts, never from API presence, support-profile optimism,
or a hard-coded milestone status.

Scenario

- aggregate the Phase 13 shared-read pinning closure artifact
- aggregate the Phase 15 journal/replay boundary certification
- aggregate the Phase 16 concurrent hostile matrix artifact
- aggregate the Phase 17 public-bridge reader-lane honesty artifact
- publish the Phase 18 support/profile row
  `milestone-9.7-derived-closure-posture`
- require `milestone-9.7-closeout.md`, `test-requirements.md`, and support
  matrix publication to agree on the derived posture
- sabotage each required phase-local posture independently and require the
  milestone posture to reopen
- remove a phase-local evidence digest and require the milestone posture to
  reopen

Must verify

- all required phase-local closures are present and evidence-bearing
- Milestone 9.7 `Closed` appears only when every required phase-local posture is
  `Closed`
- support/profile publication names Phase 18 and carries the derived closure
  contract digest
- support/profile publication does not hard-code `Closed` without supplied
  phase-local proof artifacts
- closeout docs enumerate defended exclusions instead of expanding Milestone
  9.7 ownership

Required verification output

- `milestone_9_7_derived_closure_digest`
- `phase_13_shared_read_pinning_closure_digest`
- `phase_15_journal_inventory_digest`
- `phase_16_concurrent_hostile_matrix_digest`
- `phase_17_public_bridge_reader_lane_digest`
- `support_profile_phase_18_row_digest`

Pass condition

Phase 18 closes only when milestone posture, support/profile publication,
test-requirements rows, and closeout documentation all derive from the same
phase-local evidence set and any missing, open, or digest-less required phase
prevents Milestone 9.7 from reporting `Closed`.

