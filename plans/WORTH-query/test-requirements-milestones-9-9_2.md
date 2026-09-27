## Milestone 9 Named Certification Suites

### 9. Policy, Tenant Schema, And Relationship-Proof Boundary Test

Purpose

Prove that policy masking, tenant basis/schema variation, and graph-native
relationship proofs fail closed and preserve parity across execution modes.

Scenario

- run equivalent queries under:
  - masked and unmasked policy contexts
  - two tenant schema variants
  - valid and broken relationship-proof chains
  - one-shot, live, and historical execution modes where admitted

Must verify

- masked aspects never enter the execution plan
- tenant-specific schema variation changes validation/projection explicitly
- relationship-proof denials fail before unauthorized truth is exposed
- one-shot/live/historical policy behavior remains parity-safe for the same
  declared basis

Required verification output

- `query_digest`
- `policy_digest`
- `result_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Policy and tenant boundaries are structural, typed, and non-leaking.

## Milestone 9.1 Named Certification Suites

### 9.1. Query Subscription Declaration And Lowering Parity Test

Purpose

Prove that query-owned subscription declaration families, basis binding, bridge
lowering, and admission preserve canonical live query meaning across equivalent
construction paths, while policy, tenant, basis, relationship-proof, and
view-shape variations that change live meaning change subscription meaning
explicitly or fail before activation.

Scenario

- use a concrete `EmployeeRecord` fixture with:
  - visible fields `identity.employee_id`, `profile.display_name`,
    `profile.department`, and `management.manager_id`
  - masked field `compensation.salary_band`
  - tenant variants where compensation schema differs or is masked
  - relationship-proof path `employee -> department -> manager`
- lower equivalent live query inputs into subscription declarations through:
  - direct canonical live query construction
  - scope-composed construction
  - template-instantiated construction
  - saved-query exact reuse
  - admitted runtime facade helper construction
- exercise admitted subscription family lowerings for:
  - detail exact
  - inspector detail exact
  - ordered collection membership
  - grouped collection membership
  - bounded materialization where required bridge slice kinds are admitted
- vary:
  - masked and unmasked policy basis
  - tenant truth/schema basis
  - valid and broken relationship-proof admission
  - current, branch-head, snapshot, and unsupported basis requests
  - table/detail/grouped/inspector view-shape posture
  - bridge-supported and bridge-unsupported slice families

Must verify

- equivalent live query inputs lower to the same query subscription family,
  declaration digest, equivalence digest, bridge declaration digest, and bridge
  basis request where semantics are equal
- each phase carries a declared work budget, slice budget, bridge-lowering
  budget, or admission budget before it performs work
- budget denials are typed distinctly from semantic unsupported-family denials
- policy, tenant, relationship-proof, basis, and view-shape differences that
  change live meaning also change subscription declaration meaning explicitly
- admitted query subscription declarations lower to explicit bridge
  declaration families and bridge basis requests
- grouped and inspector query-side subscription meanings remain distinct in
  query declaration digests even when they lower onto bridge collection or
  detail families
- masked field influence in subscription slice intent, ordering, grouping, live
  relevance, or delivery intent denies before bridge lowering
- unsupported bridge families, unsupported slice kinds, unsupported bases,
  raw CDC fallback, host observer inference, generic subscription kinds, and
  durable overclaims fail typed and early
- no activation input can be produced without query-owned declaration,
  bridge-lowering, basis-binding, and admission evidence
- declaration code does not allocate active lifecycle state, fanout state,
  delivery windows, acknowledgement frontiers, checkpoints, or continuation
  indexes
- small/medium/larger fixture runs prove declaration and bridge-lowering cost
  slopes are bounded by declared slice, projection, grouping, proof, and
  registry widths rather than unrelated fixture row count
- bridge lowering consumes deduplicated slice proof from query declaration and
  does not rediscover projection, policy masks, saved-query registries, or
  view-shape registries

Required verification output

- `query_digest`
- `live_family_digest`
- `subscription_family_digest`
- `subscription_declaration_digest`
- `subscription_equivalence_digest`
- `policy_digest`
- `tenant_basis_digest`
- `relationship_proof_digest`
- `view_shape_digest`
- `basis_digest`
- `bridge_declaration_digest`
- `bridge_basis_digest`
- `signal_strategy_digest`
- `admission_digest`
- `failure_digest`
- `fixture_digest`
- `subscription_work_budget_digest`
- `subscription_scale_slope_digest`
- `compile_fail_boundary_digest`
- `counter_snapshot`
- `support_matrix_digest`

Pass condition

Subscription declaration and lowering remain query-owned, bridge-honest,
basis-explicit, parity-safe for equivalent live meaning, and fail-closed before
activation for unsupported or ambiguous subscription semantics.

## Milestone 9.2 Named Certification Suites

### 9.2. Subscription Lifecycle Sharing And Preview Parity Test

Purpose

Prove that admitted query subscription activation input becomes active
runtime-backed query subscription lifecycle without changing canonical query or
subscription meaning, and that active delivery, sharing, continuation, and
preview isolation remain query-shaped, parity-safe, and residue-free.

Scenario

- use the concrete `EmployeeRecord` fixture from Milestone 9.1
- activate admitted subscription declarations for:
  - detail exact
  - inspector detail exact
  - ordered collection membership
  - grouped collection membership
  - bounded materialization where admitted
- exercise:
  - single-consumer active lifecycle
  - equivalent-subscription sharing
  - multi-consumer fanout with different delivery pacing
  - query-shaped delivery windows and patch batches
  - bridge/signal update lowering into family-typed query maintenance deltas
  - receipt-based acknowledgement and per-consumer backpressure
  - continuation/remap after admitted identity evolution or correspondence
  - preview-scoped active subscriptions
  - preview discard and preview promotion boundaries
- vary:
  - equal and unequal subscription equivalence digests
  - masked and unmasked policy digests
  - tenant truth/schema basis digests
  - relationship-proof digests
  - view-shape digests
  - current, branch-head, runtime snapshot, and preview-scoped bases
  - sparse and bursty active delivery workloads

Required concrete lanes

- detail exact lane where `management.manager_id` changes and delivery emits a
  detail field patch, receipt, and acknowledgement frontier advancement
- grouped collection lane where an employee moves from `engineering` to
  `design`, producing grouped membership movement rather than host-side
  regrouping
- masked sharing denial lane where masked and unmasked salary-band policies
  cannot share one active lane
- identity continuation lane where advisory successors or explicit identity
  break remain continuation events rather than ordinary field patches
- preview discard lane where an emitted preview delivery is followed by a
  `PreviewDiscarded` closeout with zero authoritative residue by residue class

Must verify

- active lifecycle can be admitted only from `SubscriptionActivationInput`
- active lanes are registry-owned resources and public handles cannot mutate
  lane meaning, acknowledgement state, delivery windows, or continuation
  indexes
- active lane identity preserves query, subscription-family, declaration,
  equivalence, policy, tenant, relationship-proof, view-shape, basis, bridge,
  and signal-strategy meaning
- equivalent subscriptions can share one active maintenance lane while
  retaining independent consumer attachments, acknowledgement frontiers,
  pacing, diagnostics richness, and delivery windows
- meaning-changing policy, tenant, basis, proof, view-shape, delivery, bridge,
  or signal-strategy differences deny sharing before an active lane is joined
- active delivery emits query-shaped patch batches rather than raw CDC
- raw bridge invalidation or raw CDC must lower into a family-typed
  `QuerySubscriptionMaintenanceDelta` before any delivery window consumes it
- maintenance deltas cover distinct detail field, inspector focus, collection
  membership, collection order, grouped membership, bounded-materialization
  scope, continuation, and gap-notice variants
- patch groups cover distinct detail field, focused inspector, collection
  membership/order, grouped membership, bounded-materialization scope,
  continuation, and delivery-gap variants
- detail/inspector and collection/grouped delivery semantics remain distinct
  in active delivery digests even when bridge families are shared beneath them
- slow-consumer or backpressure behavior does not change another consumer's
  query meaning or delivery digest
- acknowledgement frontiers advance only when presented with an emitted
  `QueryDeliveryBatchReceipt` for the same attachment and sequence
- delivery-window overflow emits explicit `RetainWithinWindow`,
  `DropWithGapNotice`, `TerminateConsumer`, or `DebtExplicit` posture rather
  than hidden replay/drop behavior
- hot-path budgets are typed dimension bundles, not raw numeric knobs, and
  include fanout width, delivery-window width, maintenance-delta width,
  patch-group width, continuation-remap width, preview-residue width,
  allocation-scope width, and registry-lookup width
- active lane admission records `ActiveLaneLookupClass`; direct/generation or
  equivalence-index lookup is admitted, while linear scan is either explicit
  debt or typed denial
- delivery emission consumes an `ActiveDeliveryWorkPacket` built from a lowered
  maintenance delta, affected lane/attachment ids, density posture, patch
  width, continuation width, preview residue width, allocation scope, and a
  consumed budget receipt
- sparse delivery logic runs only under `SparseDelta` or `BurstCoalesced`;
  dense refresh must become a delivery-gap patch group, explicit debt, or typed
  denial rather than hidden full-result rebuild
- shared maintenance cost and per-consumer fanout/delivery/acknowledgement cost
  are asserted separately
- allocation posture is phase-local and bounded; unbounded heap allocation is
  explicit debt or denial
- performance receipt digests change when lookup class, density posture,
  allocation posture, or budget consumption changes even if functional query
  results are identical
- continuation/remap consumes typed identity-evolution or correspondence
  evidence before active delivery state changes
- advisory correspondence, explicit identity breaks, collection membership
  remaps, and grouped membership remaps remain distinct continuation classes
- preview-scoped subscriptions cannot share with authoritative subscriptions
  before promotion
- preview discard emits zero authoritative residue evidence for active lanes,
  delivery windows, acknowledgement frontiers, and continuation indexes
- preview residue evidence distinguishes authoritative routing, checkpoint,
  replay, diagnostics, and writeback residue rather than one generic residue
  flag
- preview promotion crosses an explicit authority boundary and mints new
  authoritative active evidence rather than mutating preview state in place
- durable checkpoint, durable restart, and store-backed replay claims fail
  typed and early
- small/medium/larger fixture runs prove active lifecycle cost slopes are
  bounded by declared fanout width, delivery width, patch width, continuation
  remap width, preview residue width, and allocation scope rather than
  unrelated fixture row count
- scale fixtures vary one dimension at a time: unrelated row count, active
  lane count, consumers per lane, patch width, group count, delivery-window
  width, continuation-remap width, preview-residue width, and allocation scope

Required verification output

- `query_digest`
- `subscription_family_digest`
- `subscription_declaration_digest`
- `subscription_equivalence_digest`
- `active_lane_digest`
- `active_lane_handle_digest`
- `active_lane_lookup_class_digest`
- `subscription_budget_digest`
- `subscription_performance_receipt_digest`
- `consumer_attachment_digest`
- `acknowledgement_frontier_digest`
- `delivery_window_digest`
- `maintenance_delta_digest`
- `active_delivery_work_packet_digest`
- `active_delivery_density_posture_digest`
- `allocation_posture_digest`
- `delivery_batch_digest`
- `patch_group_digest`
- `delivery_receipt_digest`
- `continuation_digest`
- `preview_isolation_digest`
- `preview_residue_digest`
- `policy_digest`
- `tenant_basis_digest`
- `relationship_proof_digest`
- `view_shape_digest`
- `basis_digest`
- `bridge_declaration_digest`
- `signal_strategy_digest`
- `failure_digest`
- `lifecycle_denial_digest`
- `counter_snapshot`
- `subscription_lifecycle_scale_slope_digest`
- `compile_fail_boundary_digest`
- `support_matrix_digest`

Pass condition

Active subscription lifecycle, sharing, query-shaped delivery, continuation,
and preview isolation remain family-aware, parity-safe with one-shot meaning,
and fail-closed before lifecycle, delivery, or preview residue can drift.

