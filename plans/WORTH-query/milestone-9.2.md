# Milestone 9.2 Engineering Spec: Subscription-Family-Backed Live Delivery, Sharing, And Lifecycle Parity

> **Status:** Completed. Closeout: [milestone-9.2-closeout.md](./milestone-9.2-closeout.md).
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.1.md](./milestone-9.1.md)
>
> **Prior closeout:** [milestone-9.1-closeout.md](./milestone-9.1-closeout.md)
>
> **Next milestone:** Milestone 9.3 will own automatic subscription diagnostics,
> bridge parity explanation, and runtime-backed subscription certification
> closure.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make active query subscriptions consume
> only `SubscriptionActivationInput` and preserve query-shaped delivery,
> sharing, continuation, and preview isolation without redefining canonical
> subscription meaning after admission.
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-8-closeout.md](./milestone-8-closeout.md)
> - [milestone-9-closeout.md](./milestone-9-closeout.md)
> - [milestone-9.1.md](./milestone-9.1.md)
> - [milestone-9.1-closeout.md](./milestone-9.1-closeout.md)

## Goal

Make active query subscriptions first-class runtime objects whose lifecycle,
sharing, continuation, preview isolation, and delivery windows are derived from
admitted subscription-family activation input. The same canonical subscription
meaning must produce query-shaped deliveries for single consumers, shared
consumers, preview-scoped consumers, and admitted continuation/remap scenarios
without host cache inference, raw CDC delivery, or generic active handles.

## Why This Milestone Exists

Milestone 9.1 closed the declaration boundary. An admitted live query can now
be selected into one query subscription family, frozen into one declaration,
lowered into one bridge-facing declaration and basis request, admitted, and
handed forward only as `SubscriptionActivationInput`.

That is necessary, but not sufficient. A subscription declaration is still not
an active subscription.

Without Milestone 9.2, the next layer would be forced to invent long-lived
runtime behavior around the declaration:

- active handles could be generic lifecycle tokens rather than family-aware
  query subscription handles
- equivalent subscriptions could be deduplicated by host cache keys instead of
  query-owned equivalence evidence
- delivery windows could carry raw CDC or bridge-local patches instead of
  query-shaped delivery envelopes
- multi-consumer fanout could accidentally let one consumer's pacing mutate
  another consumer's query semantics
- identity evolution could remap active delivery by coincidence rather than by
  explicit continuation evidence
- preview-scoped subscriptions could leak into authoritative subscription
  state after discard or promotion

Milestone 9.2 therefore owns the active runtime boundary that 9.1 deliberately
excluded. It answers:

- how admitted subscription activation input becomes an active query handle
- how equivalent subscriptions share one maintenance lane without sharing
  consumer-local delivery state
- how query-shaped delivery batches, patch groups, acknowledgement frontiers,
  and continuation windows are represented
- how identity-evolution continuation and remap evidence stays typed
- how preview subscriptions remain isolated from authoritative subscriptions
  until an explicit promotion boundary admits the transfer
- how active lifecycle counters prove fanout, delivery, continuation, preview
  discard, and residue checks without hiding broad refreshes

It intentionally stops before 9.3 diagnostics/certification expansion and
before Milestone 10/11 store-backed durability. Runtime-backed active
subscription behavior can ship now; durable restart/replay cannot.

## Governing Summaries

- `MENTALITY.md`: protects adversarial foundation work over convenient feature
  shape. For 9.2, the hard problem is not "return a subscription handle"; it
  is making lifecycle, sharing, delivery, continuation, and preview isolation
  mechanically preserve the admitted declaration meaning under hostile
  long-lived operation.
- `arch_laws.md`: Laws 7, 18, 21, 30, 32, 34, 35, 37, 40, and 41 dominate this
  milestone. Active subscriptions must be framework-owned resources with
  phase-typed handles, self-describing delivery envelopes, proof-bearing
  continuation, separate domain effect and delivery lifecycles, and sealed
  construction.
- `perf_laws.md`: active subscription work is only honest if fanout width,
  delivery batch width, patch group width, continuation remap breadth, preview
  residue checks, allocation scope, and acknowledgement advancement are visible
  in exact counters. Sharing may amortize maintenance, but it may not hide
  consumer-specific delivery cost or broad refresh.
- `domain_laws.md`: lifecycle handles, shared maintenance lanes, consumer
  attachments, delivery windows, acknowledgement frontiers, continuation
  remaps, preview isolation, diagnostics, counters, and certification are
  separate responsibilities. They must not collapse into a single
  `subscription_runtime.rs` bag.
- `worth_query_vision.md`: read-to-subscribe promotion, incremental result
  maintenance, query-to-signal bridging, view-shaped patches, preview/branch
  workflows, and lineage/correspondence queries are explicit product pillars.
  Milestone 9.2 turns the active runtime side of those promises into query
  artifacts.
- `worth_query_roadmap.md`: Milestone 9.2 must prove subscription-family-backed
  live delivery, sharing, continuation, and preview isolation preserve
  query-shaped parity for the same canonical query and admitted live family.
- `test-requirements.md`: the `Subscription Lifecycle Sharing And Preview
  Parity Test` is the required closeout proof. It must compare one-shot,
  active subscription, shared consumer, continuation/remap, and preview discard
  lanes through canonical bundles.
- `milestone-8-closeout.md`: view-shape semantics are already planner-visible
  and delivery-visible. 9.2 must preserve table/detail/grouped/inspector
  delivery contracts in active subscription windows instead of flattening them
  into generic patches.
- `milestone-9-closeout.md`: policy, tenant, relationship-proof, live, and
  delivery admission are already closed for runtime-backed meaning. 9.2 must
  consume those admitted digests and must not re-mask, re-tenant, or re-prove
  inside active lifecycle code.
- `milestone-9.1.md` and `milestone-9.1-closeout.md`: subscription declaration,
  bridge lowering, basis binding, admission, and `SubscriptionActivationInput`
  are already closed. 9.2 may consume activation input, but it may not recreate
  declaration, lowering, bridge basis, or admission from raw live descriptors.

## Adversarial Constraint

Milestone 9.2 must survive the following hostile condition:

> The same admitted subscription declaration is activated for one consumer,
> activated for multiple equivalent consumers, advanced through query-shaped
> delivery windows under different consumer pacing, remapped through admitted
> identity-evolution continuation, and activated inside a preview session that
> is later discarded or promoted; every admitted lane must preserve the same
> canonical query, subscription-family, basis, policy, tenant,
> relationship-proof, view-shape, bridge, and signal-strategy meaning while
> ensuring that active lifecycle, fanout, continuation, and preview churn never
> redefine query semantics or leak residue.

Concretely, the design must remain correct when all of the following are true:

- two consumers attach to semantically equivalent subscription declarations but
  request different delivery pacing
- a third consumer has a meaning-changing policy, tenant, basis, view-shape, or
  relationship-proof digest and must not share the active lane
- grouped and inspector subscriptions share bridge families beneath them but
  must remain distinct query-side active families
- relevant and irrelevant truth changes arrive in bursts and sparse deltas
- a continuation event remaps an entity identity, collection membership, or
  grouped membership through admitted identity-evolution evidence
- a preview-scoped subscription produces deliveries and then the preview is
  discarded or promoted
- a naive implementation would be tempted to:
  - make one generic `ActiveSubscriptionHandle`
  - deduplicate by raw query digest only
  - let shared fanout reuse one consumer's acknowledgement frontier for all
    consumers
  - deliver raw CDC events and let hosts reshape them
  - treat continuation as a cache update instead of typed remap evidence
  - discard preview state without proving zero authoritative residue
  - claim restart-stable continuation without durable store support

If any supported path:

- activates from anything other than admitted `SubscriptionActivationInput`
- shares subscriptions whose subscription equivalence digests differ
- lets consumer pacing, acknowledgement state, or fanout width change query
  meaning
- emits raw CDC as the caller-facing delivery contract
- collapses grouped, inspector, detail, collection, and bounded materialization
  active families into one generic lifecycle lane
- remaps identity or membership without typed continuation evidence
- allows discarded preview subscription state to influence authoritative
  subscriptions
- implies durable checkpoint or restart parity before Milestones 10 and 11
  close

then Milestone 9.2 has failed.

## Product Decision Lock

- `worth-query` owns active query subscription handles, active lifecycle
  artifacts, shared-lane equivalence, consumer attachment records,
  query-shaped delivery windows, acknowledgement frontiers, continuation/remap
  evidence, preview isolation artifacts, lifecycle diagnostics, counters, and
  certification for admitted runtime-backed Milestone 9.2 surfaces.
- `worth-runtime-bridge` remains authoritative for bridge subscription protocol
  semantics and bridge-facing observation execution beneath an admitted
  activation input.
- `worth-signal` remains authoritative for dependency tracking, invalidation,
  scheduling, and signal strategy behavior beneath the active query
  subscription lane.
- `worth-relational` remains authoritative for truth semantics, identity
  evolution, correspondence, branch basis, and preview promotion authority.
- Active lifecycle must consume already admitted artifacts:
  - `SubscriptionActivationInput`
  - query digest
  - subscription family digest
  - subscription declaration digest
  - subscription equivalence digest
  - policy, tenant, relationship-proof, view-shape, basis, bridge, and signal
    strategy digests
  - delivery intent and slice intent digests
- Active subscription identity is not subscription declaration identity:
  - declaration identity says what long-lived observation was requested
  - activation input says the declaration was admitted and can be activated
  - active lane identity says one runtime maintenance lane exists for that
    admitted meaning
  - consumer attachment identity says one consumer has delivery and
    acknowledgement state attached to that lane
  - durable subscription identity remains later store-backed work
- Active lane authority is registry-owned, not handle-owned:
  - `ActiveSubscriptionLane` is an internal framework-owned resource
  - public code may receive an `ActiveSubscriptionLaneHandle` or
    `ActiveSubscriptionLaneLease`, but that handle is not authority to mutate
    lane meaning
  - attaching consumers requires a crate-owned registry lookup that proves the
    lane is still active and the attachment request still matches the lane's
    active meaning digest
  - cloning a handle may duplicate a reference to the same lane, but it may not
    clone maintenance authority, acknowledgement state, delivery windows, or
    continuation indexes
  - closing the final consumer attachment may allow the framework to dispose
    the lane only after all delivery windows and continuation obligations have
    reached a terminal proof state
- Sharing is allowed only through subscription equivalence evidence:
  - equal equivalence digest plus equal activation-class digests may share one
    maintenance lane
  - unequal policy, tenant, proof, basis, view-shape, delivery intent, slice
    intent, bridge declaration, or signal strategy digest must deny sharing
  - each consumer retains independent pacing, delivery cursor, acknowledgement
    frontier, diagnostics richness, and backpressure state
- Active delivery must be query-shaped:
  - detail/inspector lanes deliver field or focused-aspect patch groups
  - collection/grouped lanes deliver membership, order, group, and projected
    delta patch groups according to admitted view-shape semantics
  - bounded materialization lanes deliver declared scoped patch groups only
    where the bridge and signal strategy admitted those slices
  - raw CDC may exist beneath the bridge, but it is never the caller-facing
    active query delivery contract
- Active delivery input is not raw CDC:
  - bridge/signal updates must first lower into a
    `QuerySubscriptionMaintenanceDelta`
  - maintenance deltas are family-typed as detail field deltas, inspector focus
    deltas, collection membership/order deltas, grouped membership deltas, or
    bounded-materialization scope deltas
  - a maintenance delta carries the active lane digest, bridge declaration
    digest, signal strategy digest, delivery intent digest, slice intent digest,
    and affected query-scope proof before a delivery window can consume it
  - if the bridge can only supply raw CDC for a requested family, 9.2 must deny
    with `DeniedRawCdcFallback` rather than wrap that CDC in a query delivery
    batch
- Acknowledgement and backpressure are typed consumer-local state:
  - `SubscriptionAcknowledgementFrontier` belongs to one
    `SubscriptionConsumerAttachment`
  - `QueryDeliverySequence` is monotonic per attachment and cannot be minted by
    callers
  - `DeliveryBackpressurePolicy` must be `RetainWithinWindow`,
    `DropWithGapNotice`, `TerminateConsumer`, or `DebtExplicit`
  - acknowledgement may advance only by presenting an emitted
    `QueryDeliveryBatchReceipt`
  - a gap notice is a query-shaped delivery event with its own digest, not a
    silent instruction for the host to re-fetch
- Continuation/remap is a typed active-subscription event, not a host cache
  update. Identity remap, correspondence advisory, explicit identity break,
  collection membership remap, and grouped membership remap are distinct
  continuation classes.
- Preview subscriptions are isolated runtime objects. Preview discard must
  prove zero authoritative residue; preview promotion must cross an explicit
  authority boundary and mint new authoritative active evidence.
- Preview isolation must reuse the bridge preview typestate vocabulary:
  - active preview subscriptions bind to `PreviewActive`
  - discarded preview closeouts bind to `PreviewDiscarded`
  - promoted preview handoffs bind to `PreviewPromoted`
  - residue reports classify at least authoritative routing, checkpoint,
    replay, diagnostics, and writeback residue separately from temporary
    preview execution or diagnostics residue
- Active lifecycle must declare one `ActiveSubscriptionLifecyclePosture`:
  `SingleConsumer`, `SharedEquivalent`, `PreviewScoped`,
  `ContinuationRemapped`, `DeniedMeaningMismatch`, or
  `DeniedDurableOverclaim`.
- Active delivery must declare one `ActiveSubscriptionDeliveryPosture`:
  `QueryShapedPatch`, `GroupedPatch`, `FocusedInspectorPatch`,
  `BoundedMaterializationPatch`, `DeniedRawCdcFallback`, or `DebtExplicit`.
- Durable continuation checkpoints, restart-stable active handles,
  store-backed subscription replay, and snapshot-plus-tail subscription
  continuation remain Milestone 10/11 scope.

Normative consequence:

- any implementation path that creates active subscription handles without
  `SubscriptionActivationInput` is out of spec
- any implementation path that shares by raw query digest alone is out of spec
- any implementation path that stores acknowledgement or delivery cursor state
  on the shared maintenance lane rather than the consumer attachment is out of
  spec
- any implementation path that exposes raw CDC to consumers as the active query
  delivery contract is out of spec
- any implementation path that handles preview discard without residue proof is
  out of spec
- any implementation path that claims durable restart survival through
  runtime-backed active lifecycle state is out of spec

## Typed Phase Progression Lock

Milestone 9.2 must define one proof-bearing phase chain. Active lifecycle must
not be a loose runtime wrapper around declaration artifacts.

Required phase progression:

- `SubscriptionActivationInput`
  - the 9.1 artifact proving declaration, bridge lowering, basis binding, and
    runtime-backed admission already happened
- `ActiveSubscriptionLaneAdmission`
  - proves the activation input can create or join one active maintenance lane
    under lifecycle, sharing, preview, and allocation budgets
- `ActiveSubscriptionRuntime`
  - facade-owned runtime object that owns the active lane registry; callers do
    not receive mutable registry access
- `ActiveSubscriptionLane`
  - framework-owned runtime resource for one admitted subscription meaning and
    one admitted signal strategy
- `ActiveSubscriptionLaneHandle`
  - public read/attachment token for one registry-owned lane; it is not
    maintenance authority and does not carry consumer delivery state
- `SubscriptionConsumerAttachment`
  - consumer-local delivery, pacing, acknowledgement, diagnostics, and
    backpressure state attached to an active lane
- `SubscriptionAcknowledgementFrontier`
  - consumer-local monotonic acknowledgement proof for emitted delivery
    sequence numbers
- `QueryDeliveryWindow`
  - query-shaped pending delivery state for one consumer attachment
- `QuerySubscriptionMaintenanceDelta`
  - family-typed bridge/signal maintenance input already narrowed to admitted
    query meaning
- `QueryDeliveryBatch`
  - one emitted caller-facing batch, patch group, or continuation event with
    digests and counters
- `SubscriptionContinuationEvidence`
  - optional proof-bearing remap artifact consumed before delivery state can
    be transformed across identity evolution, correspondence, branch movement,
    or preview promotion/discard
- `SubscriptionLifecycleCloseout`
  - terminal artifact proving detach, discard, termination, promotion handoff,
    or explicit debt posture with residue counters

Rules:

- no API may construct `ActiveSubscriptionLane` or mutate active lane registry
  state from raw declarations, raw bridge declarations, raw live descriptors,
  or host observer state
- no API may construct `SubscriptionConsumerAttachment` without an active lane
  handle plus registry-owned equivalence verification
- no API may construct `QueryDeliveryBatch` without a delivery window
- no API may construct `QueryDeliveryBatch` from raw CDC, raw bridge
  invalidation, or host patch JSON
- no API may mutate subscription family, basis, policy, tenant,
  relationship-proof, view-shape, bridge, or signal strategy digests after lane
  admission
- no API may share an active lane unless equivalence was proven before lane
  admission
- no API may advance `SubscriptionAcknowledgementFrontier` without presenting a
  `QueryDeliveryBatchReceipt` emitted for the same consumer attachment
- no API may remap active delivery state without
  `SubscriptionContinuationEvidence`
- no API may discard a preview lane without producing
  `SubscriptionLifecycleCloseout`

## Compile-Time Enforcement Policy

Milestone 9.2 must classify which active lifecycle guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible active lanes without activation input, equivalence
  digest, bridge declaration digest, signal strategy digest, lifecycle posture,
  and active lane digest
- publicly constructible active lane handles that imply ownership of the lane
  resource rather than registry-scoped access to it
- publicly constructible consumer attachments without consumer-local
  acknowledgement, delivery cursor, pacing, diagnostics, and backpressure state
- publicly constructible delivery batches that do not carry query,
  subscription family, view-shape, delivery window, and patch group digests
- publicly constructible maintenance deltas that do not carry a family-specific
  query scope proof and admitted bridge/signal digests
- publicly constructible acknowledgement frontiers or delivery sequence numbers
  without an emitted batch receipt
- publicly constructible continuation evidence that does not identify the
  remap class, source identity, target identity, basis, and authority digest
- publicly constructible preview discard closeouts without residue counters

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `ActiveSubscriptionLaneAdmission`,
  `ActiveSubscriptionLane`, `SubscriptionConsumerAttachment`,
  `QueryDeliveryWindow`, `QueryDeliveryBatch`,
  `QuerySubscriptionMaintenanceDelta`, `SubscriptionAcknowledgementFrontier`,
  `SubscriptionContinuationEvidence`, `PreviewSubscriptionIsolationArtifact`,
  or `SubscriptionLifecycleCloseout`
- public APIs that activate from raw live descriptors, raw bridge declarations,
  raw CDC filters, raw query plans, or host observer callbacks
- public APIs that expose mutable lane registries, active lane maps, fanout
  maps, acknowledgement maps, or delivery-window maps
- public APIs that accept booleans such as `shared`, `preview`, `raw_cdc`,
  `replay_from_start`, `drop_slow_consumers`, or `durable`
- public APIs that allow active lane digests or admitted policy/tenant/proof
  digests to be patched after activation
- public APIs that advance acknowledgement frontiers on the shared lane rather
  than on consumer attachments
- public APIs that convert preview active lanes into authoritative active lanes
  in place
- public APIs that acknowledge delivery by raw sequence number without a typed
  batch receipt
- public APIs that accept raw cost dimensions for active hot-path work instead
  of sealed budget/dimension types
- public APIs that emit delivery batches from raw maintenance deltas without an
  `ActiveDeliveryWorkPacket`

`Construction-time rejection`:

- activation input whose active lifecycle family is unsupported
- sharing request with mismatched subscription equivalence or activation-class
  digests
- consumer attachment budget exhausted
- delivery window budget exhausted
- zero-width delivery window budget
- unbounded delivery retention budget
- dense-refresh posture without gap, debt, or denial evidence
- allocation outside declared allocation posture
- raw CDC fallback request
- raw bridge invalidation without query maintenance lowering
- acknowledgement receipt/attachment mismatch
- acknowledgement sequence regression or skipped sequence without gap notice
- backpressure policy omitted or hidden behind default behavior
- unsupported continuation/remap class
- identity-evolution or correspondence evidence mismatch
- preview session lifecycle mismatch
- preview discard with nonzero authoritative residue
- preview promotion without authority-boundary evidence
- preview residue class collapsed into one generic residue flag
- durable restart, durable checkpoint, or store-backed replay claim
- active lifecycle allocation outside admitted allocation scope

## Phases

### Phase 1: Active Lifecycle Vocabulary And Lane Admission

Define the runtime-backed active subscription vocabulary that consumes 9.1
activation input.

Must ship:

- `ActiveSubscriptionLifecyclePosture`
- `ActiveSubscriptionDeliveryPosture`
- `ActiveSubscriptionRuntime`
- `ActiveSubscriptionLaneAdmission`
- `ActiveSubscriptionLane`
- `ActiveSubscriptionLaneRegistry`
- `ActiveSubscriptionLaneHandle`
- `ActiveSubscriptionLaneDigest`
- `ActiveSubscriptionCounters`
- `ActiveSubscriptionWorkBudget`
- `ActiveSubscriptionAllocationPolicy`

Proof obligations:

- activation from raw descriptors is compile-fail or typed denial
- admitted single-consumer lane preserves all 9.1 activation digests
- lane handles are registry-scoped access tokens and cannot mutate lane
  meaning
- unsupported family and durable overclaim deny before active lane exists
- exact counters for active lane creation, registry lookup, handle issue,
  join denial, allocation denial, and durable overclaim denial

### Phase 2: Consumer Attachment, Sharing, And Fanout

Separate shared maintenance from consumer-local delivery state.

Must ship:

- `SubscriptionConsumerAttachment`
- `SubscriptionConsumerAttachmentDigest`
- `SubscriptionFanoutPlan`
- `SubscriptionFanoutReport`
- `SubscriptionAcknowledgementFrontier`
- `QueryDeliverySequence`
- `QueryDeliveryBatchReceipt`
- `DeliveryBackpressurePolicy`
- per-consumer delivery cursor artifacts

Proof obligations:

- two equivalent consumers share one active lane and retain distinct attachment
  digests
- policy/tenant/basis/view/proof mismatch denies sharing
- one slow consumer does not change another consumer's delivery digest
- acknowledgement advances only with a batch receipt for the same attachment
- `DropWithGapNotice` emits a query-shaped gap notice and changes delivery
  digest explicitly
- exact counters for shared lane count, fanout width, attachment count,
  acknowledgement advancement, gap notice, backpressure denial, and sharing
  denial

### Phase 3: Query-Shaped Delivery Windows And Patch Batches

Make active delivery caller-facing and query-shaped.

Must ship:

- `QueryDeliveryWindow`
- `QueryDeliveryBatch`
- `QueryPatchGroup`
- `QuerySubscriptionMaintenanceDelta`
- `ActiveDeliveryWorkPacket`
- `QueryMaintenanceDeltaLoweringReport`
- `SubscriptionPerformanceReceipt`
- family-specific patch classes for detail, inspector, collection, grouped,
  and bounded-materialization lanes
- delivery window budget and allocation evidence

Required `QuerySubscriptionMaintenanceDelta` variants:

- `DetailFieldDelta`
- `InspectorFocusDelta`
- `CollectionMembershipDelta`
- `CollectionOrderDelta`
- `GroupedMembershipDelta`
- `BoundedMaterializationScopeDelta`
- `ContinuationDelta`
- `GapNoticeDelta`

Required `QueryPatchGroup` variants:

- `DetailFieldPatchGroup`
- `InspectorFocusedPatchGroup`
- `CollectionMembershipPatchGroup`
- `CollectionOrderPatchGroup`
- `GroupedMembershipPatchGroup`
- `BoundedMaterializationScopePatchGroup`
- `ContinuationPatchGroup`
- `DeliveryGapPatchGroup`

Proof obligations:

- active delivery converges to one-shot re-execution for admitted families
- raw bridge invalidation lowers into `QuerySubscriptionMaintenanceDelta`
  before any delivery window consumes it
- delivery emission consumes `ActiveDeliveryWorkPacket`, not raw maintenance
  deltas, and the packet proves affected lane width, affected attachment width,
  patch-group width, density posture, continuation width, preview residue
  width, allocation scope, and consumed budget
- detail/inspector and collection/grouped lanes produce distinct patch digests
- raw CDC fallback is denied before a delivery batch exists
- exact counters for maintenance deltas, work-packet construction, delivery
  batches, patch groups, projected field width, group movement width,
  focused-inspector width, gap notices, density posture, performance receipts,
  and allocation scope

### Phase 4: Continuation And Identity-Evolution Remap

Represent continuation as typed evidence rather than cache coincidence.

Must ship:

- `SubscriptionContinuationEvidence`
- `SubscriptionContinuationClass`
- `SubscriptionContinuationReport`
- identity, correspondence, membership, group, and preview-promotion remap
  evidence where admitted

Proof obligations:

- admitted identity remap preserves query meaning while changing continuation
  digest
- collection and grouped membership remaps are patch-visible
- advisory and identity-break cases remain typed and distinct
- exact counters for remap count, remap width, advisory continuation, identity
  break, and remap denial

### Phase 5: Preview Isolation, Discard, And Promotion Boundary

Keep preview subscriptions isolated from authoritative subscription state.

Must ship:

- `PreviewSubscriptionIsolationArtifact`
- preview active lane posture
- preview discard closeout
- preview promotion handoff evidence
- preview residue report
- bridge preview typestate binding for `PreviewActive`, `PreviewDiscarded`,
  and `PreviewPromoted`
- residue classes aligned with bridge `BridgePreviewResidueClass`

Proof obligations:

- discarded preview lane leaves zero authoritative residue
- preview and authoritative lanes with otherwise matching query meaning do not
  share before promotion
- promoted preview lane emits promotion-boundary evidence and a new
  authoritative active digest
- residue reports distinguish authoritative routing, checkpoint, replay,
  diagnostics, and writeback residue rather than one generic residue flag
- exact counters for preview activation, preview delivery, discard residue
  checks, promotion handoff, and preview sharing denial

### Phase 6: Certification, Support, And Compile-Fail Closure

Close the runtime-backed proof surface for 9.2.

Must ship:

- `SubscriptionLifecycleCertificationBundle`
- 9.2 harness rows for lifecycle, sharing, delivery, continuation, preview
  isolation, rejection, and scale slope
- support profile additions for runtime-backed active subscription families
- compile-fail boundaries for public construction and shortcut traps
- trybuild targets for:
  - `active_subscription_lane_constructor_private.rs`
  - `active_subscription_lane_handle_no_authority.rs`
  - `active_subscription_raw_activation_forbidden.rs`
  - `active_subscription_raw_bridge_declaration_forbidden.rs`
  - `active_subscription_raw_cdc_delivery_forbidden.rs`
  - `active_subscription_generic_handle_forbidden.rs`
  - `active_subscription_shared_ack_frontier_forbidden.rs`
  - `active_subscription_ack_without_receipt_forbidden.rs`
  - `active_subscription_maintenance_delta_constructor_private.rs`
  - `active_subscription_delivery_work_packet_required.rs`
  - `active_subscription_delivery_batch_constructor_private.rs`
  - `active_subscription_raw_fanout_width_forbidden.rs`
  - `active_subscription_raw_delivery_window_width_forbidden.rs`
  - `active_subscription_zero_delivery_window_width_forbidden.rs`
  - `active_subscription_public_vec_patch_group_forbidden.rs`
  - `active_subscription_dense_refresh_without_posture_forbidden.rs`
  - `active_subscription_linear_scan_lookup_without_debt_forbidden.rs`
  - `active_subscription_unbounded_heap_allocation_forbidden.rs`
  - `active_subscription_preview_in_place_promotion_forbidden.rs`
  - `active_subscription_preview_discard_without_closeout_forbidden.rs`
  - `active_subscription_durable_checkpoint_forbidden.rs`

Proof obligations:

- the named 9.2 test suite passes with canonical bundles
- required output digests are emitted for admitted and rejected lanes
- small/medium/larger fixtures prove active lifecycle cost slopes are bounded
  by fanout width, delivery width, patch width, remap width, and preview
  residue width rather than unrelated row count

## Public Facade And Typestate API Shape

Required facade shape, subject to local naming adjustment:

```rust
pub fn admit_active_subscription_lane(
    activation: SubscriptionActivationInput,
    budget: ActiveSubscriptionWorkBudget,
) -> Result<ActiveSubscriptionLaneAdmission, ActiveSubscriptionLifecycleError>;

pub fn open_active_subscription_lane(
    runtime: &mut ActiveSubscriptionRuntime,
    admission: ActiveSubscriptionLaneAdmission,
) -> Result<ActiveSubscriptionLaneHandle, ActiveSubscriptionLifecycleError>;

pub fn attach_subscription_consumer(
    runtime: &mut ActiveSubscriptionRuntime,
    lane: ActiveSubscriptionLaneHandle,
    request: SubscriptionConsumerAttachmentRequest,
    budget: SubscriptionConsumerAttachmentBudget,
) -> Result<SubscriptionConsumerAttachment, SubscriptionConsumerAttachmentError>;

pub fn open_query_delivery_window(
    attachment: SubscriptionConsumerAttachment,
    budget: QueryDeliveryWindowBudget,
) -> Result<QueryDeliveryWindow, QueryDeliveryWindowError>;

pub fn emit_query_delivery_batch(
    window: QueryDeliveryWindow,
    work: ActiveDeliveryWorkPacket,
) -> Result<QueryDeliveryBatch, QueryDeliveryError>;

pub fn acknowledge_query_delivery_batch(
    attachment: SubscriptionConsumerAttachment,
    receipt: QueryDeliveryBatchReceipt,
) -> Result<SubscriptionConsumerAttachment, SubscriptionAcknowledgementError>;

pub fn apply_subscription_continuation(
    window: QueryDeliveryWindow,
    evidence: SubscriptionContinuationEvidence,
) -> Result<QueryDeliveryWindow, SubscriptionContinuationError>;

pub fn close_subscription_lifecycle(
    runtime: &mut ActiveSubscriptionRuntime,
    lane: ActiveSubscriptionLaneHandle,
    request: SubscriptionLifecycleCloseRequest,
) -> Result<SubscriptionLifecycleCloseout, SubscriptionLifecycleCloseError>;
```

Rules:

- each function consumes the prior proof type and returns the next proof type
- fallible phases consume explicit budget proof before doing work, and the
  returned proof must include a consumed/remaining budget receipt for the next
  phase
- `open_active_subscription_lane` touches `ActiveSubscriptionRuntime`, whose
  internal registry is not public, and is therefore fallible for registry
  lifecycle conflicts; it may not reinterpret subscription meaning
- public budgets must be typed dimension bundles, not raw `usize` knobs. The
  required dimensions are `FanoutWidth`, `DeliveryWindowWidth`,
  `MaintenanceDeltaWidth`, `PatchGroupWidth`, `ContinuationRemapWidth`,
  `PreviewResidueWidth`, `AllocationScopeWidth`, and `RegistryLookupWidth`
- active lane handles cannot be cloned into independent authority; sharing
  creates consumer attachments through equivalence-governed admission
- delivery batches can consume only `ActiveDeliveryWorkPacket`, never raw
  bridge invalidation or raw CDC. The packet must be derived once from
  `QuerySubscriptionMaintenanceDelta` plus lane/attachment/delivery-window
  budget evidence, and later phases may not re-scan the registry or full
  result set to rediscover work
- acknowledgement requires a `QueryDeliveryBatchReceipt` emitted for the same
  attachment and sequence
- diagnostics-rich helpers may wrap these functions, but may not expose weaker
  inputs or alternate construction paths

## Representative Scenario Matrix

Minimum canonical rows:

- `single-consumer-detail-lifecycle`
- `single-consumer-collection-lifecycle`
- `equivalent-detail-shared-lane`
- `equivalent-collection-shared-lane`
- `slow-consumer-does-not-mutate-fast-consumer`
- `detail-query-shaped-patch-parity`
- `inspector-focused-patch-parity`
- `collection-membership-patch-parity`
- `grouped-membership-patch-parity`
- `bounded-materialization-patch-parity`
- `identity-remap-continuation-parity`
- `correspondence-advisory-continuation-explicitness`
- `identity-break-continuation-termination`
- `preview-scoped-lifecycle-isolation`
- `preview-discard-zero-residue`
- `preview-promotion-authority-boundary`
- `active-scale-slope-honesty`

Minimum rejection rows:

- `raw-live-descriptor-activation-forbidden`
- `raw-bridge-declaration-activation-forbidden`
- `raw-cdc-delivery-forbidden`
- `generic-active-handle-forbidden`
- `sharing-equivalence-mismatch-denied`
- `shared-lane-policy-digest-mismatch-denied`
- `shared-lane-tenant-digest-mismatch-denied`
- `shared-lane-view-shape-digest-mismatch-denied`
- `shared-lane-relationship-proof-digest-mismatch-denied`
- `acknowledgement-frontier-on-shared-lane-forbidden`
- `unsupported-continuation-remap-denied`
- `preview-authoritative-sharing-forbidden`
- `preview-discard-residue-denied`
- `preview-in-place-promotion-forbidden`
- `durable-checkpoint-overclaim-denied`
- `store-backed-restart-overclaim-denied`

Required output fields for every admitted row:

- `query_digest`
- `subscription_family_digest`
- `subscription_declaration_digest`
- `subscription_equivalence_digest`
- `active_lane_digest`
- `active_lane_lookup_class_digest`
- `subscription_budget_digest`
- `subscription_performance_receipt_digest`
- `consumer_attachment_digest`
- `delivery_window_digest`
- `active_delivery_work_packet_digest`
- `active_delivery_density_posture_digest`
- `allocation_posture_digest`
- `delivery_batch_digest`
- `patch_group_digest`
- `continuation_digest` where relevant
- `preview_isolation_digest` where relevant
- `policy_digest`
- `tenant_basis_digest`
- `relationship_proof_digest`
- `view_shape_digest`
- `basis_digest`
- `bridge_declaration_digest`
- `signal_strategy_digest`
- `counter_snapshot`

Required output fields for rejection rows:

- `failure_digest`
- `lifecycle_denial_digest`
- `counter_snapshot`
- `forbidden_residue_zero_report` where relevant

### Concrete EmployeeRecord Walkthroughs

The certification matrix must include tangible lanes over the same
`EmployeeRecord` fixture used in 9.1. These lanes exist to prevent
implementation from satisfying the spec with abstract digest plumbing only.

`detail lifecycle`

- query: detail exact subscription for `identity.employee_id`,
  `profile.display_name`, and `management.manager_id`
- event: manager changes from `mgr-7` to `mgr-9`
- expected maintenance delta: `DetailFieldDelta` for `management.manager_id`
- expected delivery batch: one detail patch group, one delivery sequence, one
  receipt, one acknowledgement frontier advancement
- forbidden shortcut: raw CDC event for `EmployeeRecord` delivered to caller

`grouped collection sharing`

- query: grouped collection membership by `profile.department`, projecting
  `identity.employee_id` and `profile.display_name`
- consumers: two equivalent consumers with different pacing policies
- event: employee moves from `engineering` to `design`
- expected maintenance delta: `GroupedMembershipDelta` with old group, new
  group, membership identity, and projected row digest
- expected sharing: one active lane digest, two consumer attachment digests,
  two acknowledgement frontiers
- forbidden shortcut: host-side regrouping after ordinary collection delivery

`masked sharing denial`

- consumer A: masked policy basis excludes `compensation.salary_band`
- consumer B: unmasked policy basis admits `compensation.salary_band`
- expected result: sharing denied before active lane join because policy digest,
  delivery intent digest, and slice intent digest differ
- forbidden shortcut: share lane and strip salary for consumer A during
  delivery

`identity continuation`

- query: inspector detail exact subscription focused on identity-aware
  management fields
- event: employee identity splits into advisory successors
- expected continuation: `CorrespondenceAdvisoryContinuation` or
  `IdentityBreakContinuation`, not an ordinary field patch
- forbidden shortcut: pick the first successor and continue silently

`preview discard`

- query: preview-scoped collection subscription over draft branch
- event: preview emits one grouped patch, then preview is discarded
- expected closeout: `PreviewDiscarded` binding plus zero authoritative
  routing, checkpoint, replay, diagnostics, and writeback residue
- forbidden shortcut: remove handle from registry without residue proof

## Proposed Module Topology

Prefer focused modules that mirror responsibility boundaries:

```text
crates/worth-query/src/subscription_active/
  mod.rs
  runtime.rs
  registry.rs
  lane.rs
  lifecycle.rs
  handle.rs
  attachment.rs
  fanout.rs
  delivery_window.rs
  delivery_batch.rs
  maintenance_delta.rs
  patch_group.rs
  acknowledgement.rs
  backpressure.rs
  continuation.rs
  preview_isolation.rs
  residue.rs
  closeout.rs
  diagnostics.rs
  support.rs
  counters.rs
  certification.rs
  facade.rs
  tests.rs

crates/worth-query/src/harness/milestone_nine_two_certification/
  mod.rs
  matrix.rs
  tests.rs
```

The `subscription_active` module must not own declaration, bridge lowering, or
admission. Those remain in the 9.1 `subscription` module. The active module
consumes `SubscriptionActivationInput` and owns only lifecycle after admission.

## Store Dependency

- Core runtime-backed active subscription lifecycle, sharing, query-shaped
  delivery windows, continuation/remap evidence, and preview isolation are not
  blocked on `worth-store`.
- Store-backed restart and snapshot-plus-tail subscription continuation remain
  Milestone 10 scope.
- Durable subscription checkpoints, durable delivery cursors, durable active
  handle reload, restart-stable metadata, and portable subscription artifacts
  remain Milestone 11 scope.
- Milestone 9.2 may emit explicit durable-debt postures, but it may not claim
  durable restart, durable replay, or store-backed parity.

## Explicit Assumptions And Deferred Decisions

- 9.1 `SubscriptionActivationInput` is the only active lifecycle input.
- Active sharing is runtime-local and equivalence-digest-based. Cross-process
  sharing and restart-stable sharing remain durable scope.
- Preview activation is admitted only for preview basis postures already
  admitted by 9.1 and the runtime bridge. Unsupported preview basis combinations
  deny before active lane admission.
- Continuation admits only the identity-evolution and correspondence classes
  already represented by existing query identity/correspondence surfaces.
  Broader arbitrary remap graphs remain explicit debt.
- Delivery windows may retain runtime-local buffered patches while the process
  is alive, but retained patches are not durable checkpoints.
- Backpressure policy must be explicit per consumer attachment:
  - `RetainWithinWindow` keeps query-shaped batches up to the declared window
    width, then denies or emits a typed gap according to the policy
  - `DropWithGapNotice` emits a query-shaped gap notice whose digest proves the
    lost delivery sequence range and requires the consumer to re-establish
    from an admitted query basis
  - `TerminateConsumer` closes only the affected consumer attachment
  - hidden global drop/replay behavior is out of scope
- 9.3 may add richer diagnostics and automatic subscription path inspection,
  but it may not weaken 9.2 lifecycle proof types.

## Explicit Failure Taxonomy

- activation without `SubscriptionActivationInput`
- unsupported active lifecycle family
- active lane allocation denied
- sharing equivalence mismatch
- policy/tenant/proof/basis/view digest mismatch during sharing
- consumer attachment budget exceeded
- acknowledgement frontier shared-lane mutation attempt
- delivery window budget exceeded
- raw CDC delivery fallback attempt
- generic active handle attempt
- unsupported continuation class
- identity remap evidence mismatch
- advisory correspondence upgraded to continuity
- identity break flattened into ordinary patch
- preview/authoritative sharing attempt
- preview lifecycle epoch mismatch
- preview discard residue
- preview in-place promotion attempt
- durable checkpoint overclaim
- store-backed restart overclaim
- raw sequence acknowledgement without receipt
- gap notice omitted after delivery-window overflow
- consumer-local backpressure mutating shared maintenance lane
- bridge preview residue class collapse

## Anti-Patterns Explicitly Rejected

- active subscriptions created from raw live descriptors or raw bridge
  declarations
- one generic active subscription handle for every family
- sharing by raw query digest, host cache key, or bridge declaration digest
  alone
- shared acknowledgement frontier for multiple consumers
- raw sequence acknowledgement without a typed batch receipt
- hidden backpressure defaults that silently drop, replay, or re-fetch
- delivery windows that expose raw CDC to callers
- hot-path APIs that accept raw `usize` dimensions instead of sealed
  fanout/window/delta/patch/remap/residue/allocation/lookup budget types
- delivery emission that derives affected lanes, affected attachments, or patch
  width by re-scanning the active registry or full result set
- sparse update code that silently degrades into full-result rebuild under
  dense bursts
- allocation behavior hidden behind ordinary `Vec` growth or unbounded retained
  scratch without allocation posture evidence
- bridge invalidation events handed directly to delivery windows without
  `QuerySubscriptionMaintenanceDelta` lowering
- grouped and inspector active delivery implemented as ordinary collection or
  detail patches plus host reinterpretation
- continuation handled as consumer cache mutation rather than typed remap
  evidence
- preview discard implemented as best-effort cleanup without residue proof
- preview promotion implemented by mutating preview active lanes in place
- durable restart claims through runtime-local lifecycle state
- `Arc<Mutex<HashMap<...>>>` active lane registries exposed as public lifecycle
  API instead of a framework-owned registry facade
- one mega-module mixing declaration, active lifecycle, delivery, preview,
  continuation, diagnostics, and certification

## Sequencing Notes

Milestone 9.2 belongs immediately after 9.1 because active lifecycle needs one
canonical activation input and one subscription equivalence basis. Implementing
active handles before declaration/admission would have forced lifecycle code to
infer subscription meaning from runtime state.

It belongs before 9.3 because diagnostic sufficiency and bridge parity need
real active lifecycle behavior to inspect.

It belongs before Milestone 10 because store-backed parity should extend one
already query-shaped runtime-backed active subscription model rather than
inventing a backend-specific active lifecycle.

## Parallelization Notes

Once Phase 1 freezes active lane identity and lifecycle posture:

- fanout/consumer attachment work can proceed in parallel with delivery-window
  work
- continuation/remap work can proceed in parallel with preview isolation
  work
- compile-fail hardening can proceed in parallel with certification row
  construction
- final closure should wait until the same admitted families pass lifecycle,
  sharing, delivery, continuation, preview, support, and scale-slope rows

## Performance Encoding Lock

Milestone 9.2 must encode performance into active subscription architecture,
not merely observe it in benchmarks. Hot-path APIs must make the cost-bearing
dimensions impossible to omit and hard to confuse.

Required cost-bearing types:

- `ActiveSubscriptionWorkBudget`
  - carries `RegistryLookupWidth`, `FanoutWidth`, `AllocationScopeWidth`, and
    an `ActiveSubscriptionLaneAdmissionContract` token
  - cannot be constructed from raw integers outside sealed constructors
- `SubscriptionConsumerAttachmentBudget`
  - carries `FanoutWidth`, per-consumer delivery pacing, and attachment-window
    allocation scope
  - cannot borrow or mutate another consumer's acknowledgement frontier
- `QueryDeliveryWindowBudget`
  - carries `DeliveryWindowWidth`, `PatchGroupWidth`,
    `MaintenanceDeltaWidth`, `QueryDeliverySequence` bounds, and the selected
    `DeliveryBackpressurePolicy`
  - rejects zero-width delivery windows and unbounded retention before window
    construction
- `ActiveDeliveryWorkPacket`
  - carries one lowered `QuerySubscriptionMaintenanceDelta`, affected lane
    ids, affected consumer attachment ids, patch-group width, density posture,
    continuation width, preview residue width, allocation scope, and the budget
    receipt consumed to build the packet
  - is the only input allowed to active delivery emission
- `SubscriptionPerformanceReceipt`
  - records consumed and remaining budget dimensions for each phase
  - is included in active admission, consumer attachment, delivery batch,
    continuation, preview closeout, and lifecycle closeout artifacts

Required posture enums:

- `ActiveLaneLookupClass`
  - `DirectIndexGeneration`
  - `EquivalenceIndex`
  - `LinearScanDebtExplicit`
  - `LinearScanDenied`
- `ActiveDeliveryDensityPosture`
  - `SparseDelta`
  - `BurstCoalesced`
  - `DenseRefreshDenied`
  - `DenseRefreshDebtExplicit`
- `ActiveSubscriptionAllocationPosture`
  - `LifecycleArena`
  - `DeliveryWindowArena`
  - `PatchScratch`
  - `HeapAllocationDebtExplicit`
  - `HeapAllocationDenied`

Rules:

- public hot-path surfaces may not accept raw `usize` for fanout, patch width,
  delivery width, delta width, remap width, residue width, allocation scope, or
  registry lookup width
- registry lookup posture must be selected during lane admission and preserved
  in the active lane digest; any linear scan posture must be explicit debt or
  denial, never an invisible implementation detail
- sparse delivery code may run only under `SparseDelta` or `BurstCoalesced`
  posture. Dense refresh must emit `DeliveryGapPatchGroup`,
  `DenseRefreshDebtExplicit`, or a typed denial instead of silently rebuilding
  a full result
- shared-lane maintenance cost and per-consumer delivery cost must be recorded
  separately. Sharing may amortize maintenance, but cannot hide fanout,
  acknowledgement, delivery-window, or backpressure work
- allocation posture must be phase-local. A delivery window may use bounded
  scratch for patch assembly, but it may not retain unbounded heap state beyond
  the declared window
- phase outputs must include performance receipts whose digests change when
  lookup class, density posture, allocation posture, or budget consumption
  changes, even if the functional query result is identical
- scale certification must vary one dimension at a time: unrelated row count,
  active lane count, consumers per lane, patch width, group count, delivery
  window width, continuation remap width, preview residue width, and allocation
  scope
- the expected 9.2 slopes are:
  - active lane admission is bounded by activation input plus declared lookup
    posture, not total active lane count unless linear-scan debt is explicit
  - shared-lane maintenance is bounded by affected query scope and patch-group
    width, not consumers per lane
  - fanout is bounded by affected consumer attachment count
  - detail and inspector delivery are bounded by projected/focused field width,
    not unrelated fixture row count
  - collection and grouped delivery are bounded by affected membership/order or
    group movement width, not full collection width
  - preview discard is bounded by preview lane/window/frontier/residue-class
    width, not authoritative subscription count

## Complexity / Proof Obligations

Active subscriptions are long-lived hot-path resources. Performance claims must
be named at the active lifecycle boundary, not inferred from passing functional
tests.

Named contracts:

- `ActiveSubscriptionLaneAdmissionContract`
  - bounded by one activation input, one equivalence digest lookup, one active
    lane registry lookup, one declared `ActiveLaneLookupClass`, and one
    allocation-policy check
- `SubscriptionConsumerAttachmentContract`
  - bounded by fanout width for the selected lane, one consumer attachment
    request, one acknowledgement frontier, and one backpressure policy
- `QueryMaintenanceDeltaLoweringContract`
  - bounded by affected slice count, affected query scope width, projected
    field width, grouping key width, focused-inspector width, and bridge/signal
    strategy digest width
- `ActiveDeliveryWorkPacketContract`
  - bounded by one lowered maintenance delta, affected lane width, affected
    attachment width, declared density posture, patch-group width, continuation
    width, preview residue width, and allocation scope
- `QueryDeliveryWindowContract`
  - bounded by retained delivery sequence width, emitted patch group width, and
    consumer-local backpressure policy
- `SubscriptionContinuationRemapContract`
  - bounded by remap evidence width, affected identity/member/group count, and
    admitted correspondence/identity-evolution proof width
- `PreviewSubscriptionResidueContract`
  - bounded by preview lane count, preview delivery window count, preview
    acknowledgement frontier count, and bridge residue class count
- `ActiveSubscriptionScaleSlopeContract`
  - proves detail and inspector active delivery do not grow with unrelated row
    count, and collection/grouped active delivery grows with declared affected
    membership/group width rather than full collection width
- `SubscriptionPerformanceReceiptContract`
  - proves every admitted phase returns consumed and remaining budget
    dimensions, lookup posture, density posture where relevant, allocation
    posture, and scale-slope digest inputs

Required counters:

- `active_lane_admission_count`
- `active_lane_registry_lookup_count`
- `active_lane_lookup_class_count`
- `active_lane_linear_scan_debt_count`
- `active_lane_linear_scan_denial_count`
- `active_lane_handle_issue_count`
- `active_lane_creation_count`
- `active_lane_join_count`
- `active_lane_join_denial_count`
- `active_lane_allocation_denial_count`
- `consumer_attachment_count`
- `consumer_attachment_denial_count`
- `fanout_width`
- `fanout_delivery_count`
- `affected_consumer_attachment_width`
- `acknowledgement_frontier_advance_count`
- `acknowledgement_receipt_mismatch_denial_count`
- `acknowledgement_sequence_regression_denial_count`
- `delivery_window_open_count`
- `delivery_window_overflow_count`
- `delivery_gap_notice_count`
- `maintenance_delta_lowering_count`
- `maintenance_delta_width`
- `active_delivery_work_packet_count`
- `active_delivery_work_packet_width`
- `active_delivery_density_sparse_count`
- `active_delivery_density_burst_coalesced_count`
- `active_delivery_density_dense_debt_count`
- `active_delivery_density_dense_denial_count`
- `raw_cdc_delivery_denial_count`
- `raw_bridge_invalidation_denial_count`
- `delivery_batch_count`
- `delivery_window_width`
- `patch_group_count`
- `patch_group_width`
- `detail_field_patch_width`
- `focused_inspector_patch_width`
- `collection_membership_patch_width`
- `grouped_membership_patch_width`
- `bounded_materialization_scope_patch_width`
- `continuation_remap_count`
- `continuation_remap_denial_count`
- `continuation_advisory_count`
- `continuation_identity_break_count`
- `preview_active_lane_count`
- `preview_authoritative_sharing_denial_count`
- `preview_discard_residue_check_count`
- `preview_residue_width`
- `preview_authoritative_residue_count`
- `preview_promotion_handoff_count`
- `durable_checkpoint_overclaim_denial_count`
- `store_backed_restart_overclaim_denial_count`
- `active_subscription_scale_fixture_row_count`
- `active_subscription_scale_slope_digest_part_count`
- `subscription_performance_receipt_count`
- `subscription_budget_consumption_width`
- `subscription_budget_remaining_width`
- `heap_allocation_debt_count`
- `heap_allocation_denial_count`

Counter rules:

- exact counter assertions are required; elapsed-time thresholds do not satisfy
  9.2
- admitted sharing rows must show one active lane creation and multiple
  consumer attachments, not multiple hidden maintenance lanes
- acknowledgement advancement must equal emitted and acknowledged receipts for
  that consumer attachment
- slow-consumer rows must show backpressure or gap counters only on the slow
  consumer attachment
- raw CDC and raw bridge invalidation denial counters must be distinct
- preview discard rows must assert zero authoritative residue counters for
  routing, checkpoint, replay, diagnostics, and writeback
- scale-slope rows must prove registry lookup count does not grow with active
  lane count except for the declared lookup strategy, and must mark any linear
  scan posture as explicit debt or denial
- active lane registry storage may be arena/index/generation-backed or another
  explicit lifecycle-managed structure, but it may not be an unbounded public
  map whose traversal cost is hidden behind handle lookup
- every admitted row must include at least one `SubscriptionPerformanceReceipt`
  digest, and that digest must differ when the same functional delivery uses a
  different lookup class, density posture, allocation posture, or budget
  consumption profile
- work-packet rows must prove delivery emission consumes one
  `ActiveDeliveryWorkPacket` and does not repeat registry-wide or result-wide
  discovery
- dense workload rows must show either coalesced bounded work, an explicit
  delivery-gap patch group, explicit debt, or typed denial; hidden full refresh
  is a failure

## Acceptance Evidence

Milestone 9.2 is complete only when `worth-query` can prove:

- the `Subscription Lifecycle Sharing And Preview Parity Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- active subscription delivery remains query-shaped and parity-safe with
  one-shot meaning for the same admitted live family
- equivalent subscriptions can share one active maintenance lane without
  changing meaning or conflating consumer-local delivery state
- continuation across admitted identity-evolution and correspondence scenarios
  remains explicit, typed, and patch-visible
- discarded preview subscriptions leave no authoritative query residue
- promotion from preview to authoritative subscription crosses an explicit
  authority boundary
- unsupported active lifecycle, sharing, raw CDC fallback, continuation,
  preview, durable checkpoint, and store-backed restart claims fail typed and
  early

Required verification output must include:

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

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it freezes the missing active runtime boundary after
subscription declaration: lifecycle, sharing, delivery, continuation, and
preview isolation.

The adversarial constraint is load-bearing because it forbids the naive
failure modes where active subscription behavior is inferred from host cache
state, consumer pacing, raw CDC, or preview cleanup conventions.

The milestone preserves crate authority boundaries because `worth-query` owns
active query subscription lifecycle and query-shaped delivery, while bridge and
signal remain authorities for bridge protocol and observation execution, and
relational remains authority for truth, identity, branch, and preview
promotion semantics.

The milestone defines proof obligations rather than implementation chores
because lifecycle parity, sharing denial, query-shaped delivery, continuation
remap evidence, preview residue checks, compile-fail targets, and exact
counters are all required closure artifacts.

A competent engineer should be able to map this spec into honest
`subscription_active::lane`, `attachment`, `fanout`, `delivery_window`,
`delivery_batch`, `patch_group`, `continuation`, `preview_isolation`,
`closeout`, `diagnostics`, `support`, and certification subdomains without
inventing architecture during implementation.

This milestone belongs at 9.2 because 9.1 provides activation input and 9.3
needs actual active lifecycle behavior before it can certify diagnostics and
bridge parity.

## Closeout Standard

Milestone 9.2 is complete only when all of the following are true:

- active lifecycle can start only from admitted `SubscriptionActivationInput`
- active lane identity preserves subscription declaration, equivalence, basis,
  policy, tenant, proof, view-shape, bridge, and signal strategy digests
- equivalent subscriptions can share one active maintenance lane while
  retaining independent consumer attachments and acknowledgement frontiers
- active delivery emits query-shaped patch batches for at least two admitted
  subscription families, including one collection/grouped family and one
  detail/inspector family
- hot-path performance is encoded through sealed budget dimensions, lookup
  class, density posture, allocation posture, active delivery work packets, and
  performance receipts
- continuation/remap evidence is typed, patch-visible, and denial-aware
- preview subscriptions are isolated, discard proves zero authoritative
  residue, and promotion crosses an explicit authority boundary
- durable continuation checkpoints and store-backed restart parity remain
  explicit debt rather than implied runtime-backed support

If code lands but active handles can still be created from raw live descriptors,
sharing still uses host cache keys, delivery still exposes raw CDC, continuation
still relies on host cache mutation, performance still hides behind raw numeric
budgets or benchmark-only assertions, or preview discard still lacks residue
proof, Milestone 9.2 is not complete.
