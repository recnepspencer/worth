# Milestone 15 Engineering Spec: Subscription Delivery Families, Continuation, and Shared Consumer Contracts

> **Status:** Complete
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-14.md](./milestone-14.md)
>
> **Prior closeout:** [milestone-14-closeout.md](./milestone-14-closeout.md)
>
> **Shipped closeout:** [milestone-15-closeout.md](./milestone-15-closeout.md)
>
> **Bridge certification companion:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** turn Milestone 14's admitted subscription declarations and retained lifecycle artifacts into active bridge protocol entities whose delivery, sharing, continuation, checkpoint, replay, and preview behavior are canonical, typed, and certifiable without host-local caches or callback folklore.

## Summary

Milestone 14 closed the declaration side of bridge-native subscriptions:

- declaration families are canonical
- basis binding is explicit
- admitted subscription identity is separate from slice, stream, and consumer identity
- signal strategy lowering is retained and replay-visible
- lifecycle can reach activation-ready and deactivated retained artifacts

What it deliberately did not ship is active subscription behavior.

Milestone 15 is the next structural layer. It makes an admitted, activation-ready subscription durable as a bridge protocol entity while it delivers members, fans out to consumers, survives identity evolution, resumes after restart, replays from retained artifacts, and participates in preview discard or promotion boundaries.

This is not an app-facing watcher API. It is the lower-level bridge contract that careful manual hosts can use directly and that higher-level systems such as WORTH Query can later automate.

The milestone must preserve the Milestone 14 ceiling:

- declaration identity stays already-proven
- basis admission stays already-proven
- signal strategy selection stays already-proven

Milestone 15 consumes those proofs. It must not rediscover them during delivery.

## Goal

Make active subscriptions bridge-native protocol entities by shipping canonical delivery-family contracts, admitted consumer contracts, shared fanout semantics, continuation artifacts, subscription checkpoint and resume rules, replay artifacts, and preview-scoped subscription lifecycle guarantees.

## Why This Milestone Exists

Milestone 15 belongs immediately after Milestone 14 because declaration and admission are only half of the subscription story.

After Milestone 14, the bridge can say:

- what was declared
- what truth basis was admitted
- what subscription identity means
- what signal strategy was selected
- what retained lifecycle artifact is activation-ready

But the bridge still cannot honestly say:

- what active delivery family is being used
- which consumers are admitted to share a subscription
- whether coalescing changed pacing only or changed delivered meaning
- what checkpoint identity resumes the subscription rather than the raw stream
- how an active subscription continues after replace, split, merge-like, or branch-divergent truth evolution
- what preview-scoped delivery leaves behind after discard

Without this milestone, hosts would still stitch together long-lived delivery from admitted handles, stream checkpoints, signal observers, and local callback registries. That would recreate the exact host-local folklore Milestone 14 removed from declaration.

Milestone 15 therefore earns its place by making active subscription behavior canonical before Milestone 16 turns the full subscription story into end-to-end certification bundles and reference workloads.

## Hard Part

The hard part is not invoking callbacks.

The hard part is preserving subscription meaning while everything around the subscription changes:

- consumers attach, detach, pause, resume, and receive at different paces
- delivery may be canonical-member, coalesced, replay-oriented, audit-oriented, or route-focused
- truth identity may replace, split, merge-like reconcile, or diverge by branch
- stream checkpoints may advance differently from subscription checkpoints
- preview subscriptions may be discarded or promoted while authoritative subscriptions continue
- diagnostics richness may vary without changing delivery truth

A naive implementation will collapse at least two of these meanings:

- consumer identity becomes subscription identity
- raw stream offset becomes subscription checkpoint identity
- signal observer handle becomes bridge delivery identity
- coalesced window shape becomes canonical delivery truth
- preview subscription state leaks into authoritative lifecycle state
- lineage remap is treated as best-effort host callback repair

Milestone 15 exists to make those collapses unrepresentable or mechanically rejected.

## Adversarial Constraint

Milestone 15 must survive the following hostile condition:

> A long-lived host activates multiple admitted subscription families from Milestone 14, attaches equivalent and non-equivalent consumers with different pacing and coalescing policies, receives bursty truth changes, checkpoints after partial delivery windows, restarts from retained artifacts, replays active delivery, crosses replace, split, admitted merge-like, and branch-divergent identity evolution, and runs preview-scoped subscriptions through discard and promotion boundaries, while preserving canonical subscription meaning, delivery truth, continuation meaning, checkpoint identity, typed failure localization, exact counters, and zero authoritative residue every time.

If any supported path:

- lets consumer pacing redefine canonical subscription meaning
- lets coalescing hide, invent, or reorder canonical delivered members
- uses raw stream offsets as the only subscription resume identity
- continues after identity evolution without an explicit continuation artifact
- shares a subscription across non-equivalent consumer contracts
- promotes preview subscription state without an explicit promotion boundary
- requires host-local callback caches or process memory to explain replay
- or changes delivery semantics under diagnostics-tier variation

then Milestone 15 has failed.

## Explicit Assumptions

- `worth-relational` remains the authority for truth identity, branch semantics, lineage, merge ontology, historical retention, and final authoritative publication.
- `worth-signal` remains the authority for observation execution, scheduling, coalescing mechanics, and internal observer lifecycle.
- Milestone 14 already provides canonical declaration families, admitted subscription identities, explicit basis binding, retained lifecycle records, and selected signal strategy descriptors.
- Milestone 15 consumes Milestone 14 admitted artifacts rather than accepting raw declarations.
- Preview subscription support requires new bridge-owned preview basis artifacts
  derived from the earlier preview/speculation boundary work. An ordinary
  snapshot or branch-head basis is not preview-capable unless wrapped by that
  preview proof.
- Continuation consumes truth-owned lineage and merge artifacts. The bridge owns continuation interpretation, admission, rejection, and retained explanation.
- Store-backed durable persistence may remain out of scope unless needed to prove restart through retained bridge artifacts; this milestone should be restart-safe at the artifact contract level, not necessarily through final Store productization.

## Product Decision Lock

- Active delivery is family-aware. There is no universal subscription delivery shape.
- Active delivery must consume a precomputed `SubscriptionDeliveryCostProfile`
  or equivalent sealed cost posture. Delivery hot paths may not infer cost
  mode by inspecting consumers, callbacks, or retained diagnostics at dispatch
  time.
- Consumer contracts are admitted bridge artifacts, not raw callback objects.
- Consumer sharing requires a bridge-minted `SubscriptionSharingEligibilityWitness`
  or equivalent sealed proof. Matching strings, equal callback types, or equal
  pacing labels are not enough.
- Shared subscription fanout is legal only when consumer contracts prove equivalent sharing eligibility.
- Shared fanout must lower into a compact fanout layout before delivery. The
  delivery path may project from that layout, but it may not scan consumer
  contracts or rebuild sharing groups per member.
- Coalescing is a delivery pacing strategy, not a change to canonical delivered member truth.
- Coalescing must carry an explicit density posture: sparse member delivery,
  bounded coalesced window, dense restart/rebuild, or rejected over-budget
  delivery. Silent widening from sparse to dense is out of spec.
- Subscription checkpoint identity is distinct from raw stream checkpoint identity wherever delivery, continuation, acknowledgement, or coalescing requires additional basis.
- Subscription checkpoint publication must bind the last acknowledged canonical
  delivery member, not the last produced member or latest seen stream offset.
- Continuation across truth identity evolution is explicit and typed: continue, split, merge-like continue, branch-local continue, or reject.
- Preview subscription activation requires a bridge-owned
  `PreviewSubscriptionBasisBinding` or equivalent sealed proof derived from
  the earlier preview/speculation boundary work. Milestone 15 must not fake
  preview by tagging an ordinary branch-head subscription as preview.
- Preview subscriptions are non-authoritative until explicit promotion. Discard must prove zero authoritative and bridge-visible residue.
- Replay reconstructs active subscription meaning from retained subscription artifacts, not host callback registries.
- External callers cannot synthesize active delivery records, fanout records, continuation records, checkpoint proofs, preview promotion records, or residue proofs without bridge-owned transition functions.

Normative consequence:

- callback shape cannot define consumer equivalence
- stream offset alone cannot justify subscription resume
- signal observer handles cannot be the bridge's public delivery identity
- delivery strategy cannot be selected during callback dispatch
- fanout grouping cannot be recomputed per delivered member
- rich diagnostics cannot ride the canonical delivery hot path by default
- lineage repair cannot be host-local best effort
- continuation cannot be inferred from structural similarity after truth-owned
  lineage or merge authority denied it
- preview discard cannot be proven by "nothing panicked" or object drop alone

## Scope

### In Scope

- active subscription activation from Milestone 14 activation-ready artifacts
- admitted consumer contract vocabulary and consumer equivalence records
- delivery-family vocabulary for canonical-member, admitted coalesced, replay/audit-oriented, and route-focused delivery
- delivery records and delivery bundle digests tied to admitted subscription identity
- shared subscription fanout plans and typed sharing rejections
- subscription checkpoint identity, resume plans, and stale or incompatible checkpoint rejection
- active delivery replay from retained subscription artifacts
- continuation artifacts for replace, split, admitted merge-like, and branch-divergent truth evolution
- preview-scoped active subscription basis, discard residue proof, and explicit promotion-boundary records
- counters and diagnostics for fanout, coalescing, delivery, continuation, checkpoint, replay, preview discard, preview promotion, and rejected sharing or reuse attempts
- certification satisfying suites 31 through 34 in [test-requirements.md](./test-requirements.md)

### Explicitly Out Of Scope

- Query-owned semantic lowering into subscription declarations
- end-to-end subscription reference workload certification bundles beyond suites 31 through 34
- broad app-facing watch/callback ergonomics
- permanent store productization of subscription bundles unless needed for artifact-level restart proof
- redefining `worth-signal` observation execution or internal delivery scheduler semantics
- redefining `worth-relational` lineage, merge, branch, or preview authority semantics
- subscription-specific public UI or wasm integration

Milestone 15 must leave Milestone 16 with canonical active-subscription artifacts ready for full end-to-end certification. It must not absorb Milestone 16's reference workload and offline bundle sufficiency program.

## Governing Design Rules

### 1. Active Delivery Consumes Admission Proofs

Milestone 15 starts from `BridgeSubscriptionActivationReady` or a direct
successor type that contains the same admitted subscription identity, retained
lifecycle identity, registry identity, basis identity, and signal strategy
identity.

It must not accept:

- raw declaration intent
- raw slice intent
- raw signal strategy names
- raw source handles
- raw stream offsets as complete resume proof

Every active delivery path must carry:

- admitted subscription identity
- admitted declaration-family identity
- admitted basis identity
- retained lifecycle identity
- admitted signal strategy identity

Those identities are not recomputed during delivery. They are consumed as prior proofs.

### 2. Active Lifecycle Must Be Phase-Typed

Milestone 14 stops at activation-ready retained lifecycle handles.

Milestone 15 must introduce an active delivery proof chain such as:

- `BridgeSubscriptionActivationReady`
- `BridgeActiveSubscription`
- `BridgeDeliveryWindowOpen`
- `BridgeDeliveryWindowSealed`
- `BridgeDeliveryCheckpointReady`
- `BridgeSubscriptionCheckpointPublished`
- `BridgeSubscriptionResumeAdmitted`
- `BridgeActiveSubscriptionDeactivated`

The exact names may change, but the proof chain may not collapse into one
mutable active-subscription bag.

Compile-time enforcement must prevent:

- delivering members from an activation-ready handle that has not become active
- checkpointing an open, unsealed delivery window
- resuming from a checkpoint that has not passed subscription checkpoint
  admission
- continuing a subscription after identity evolution without a continuation
  decision artifact
- promoting or discarding a non-preview active subscription through preview
  APIs
- delivering from a discarded preview subscription

Every transition must consume the prior proof type and produce the next proof
type. Runtime booleans such as `is_active`, `is_preview`, `is_checkpointed`,
or `is_discarded` are out of spec if they are the primary enforcement surface.

### 3. Consumer Contracts Are Bridge Artifacts

A consumer contract describes how a consumer may receive a subscription without redefining the subscription.

The first-ship consumer contract must include:

- consumer contract family
- delivery expectation class
- pacing capability and backpressure posture
- coalescing admissibility
- replay or audit requirement class
- sharing eligibility basis
- diagnostics retention class if it affects retained detail

It must exclude:

- callback object address
- host-local channel identity
- thread or task identity
- incidental registration order

Consumer equivalence must be canonical and replay-visible. Two consumers may share one active subscription only if their admitted consumer contracts prove sharing equivalence or compatible fanout under an explicit fanout plan.

The public shape should be a proof chain such as:

- raw consumer intent
- normalized consumer contract
- admitted consumer contract
- sharing eligibility witness
- fanout-admitted consumer binding

External callers must not be able to construct admitted contracts or sharing
witnesses directly.

### 4. Delivery Families Preserve Canonical Member Truth

Delivery family identity must distinguish:

- canonical member delivery
- admitted coalesced delivery
- replay or audit-oriented delivery
- route-focused delivery
- family-specific delivery forms admitted later

Milestone 15 must define one concrete first-ship canonical delivery member
envelope. At minimum, every canonical member record must bind:

- admitted subscription identity
- delivery family identity
- delivery epoch or window identity
- canonical member sequence within the delivery window
- source route or slice identity
- truth-view or branch basis identity
- causality or routing digest from the upstream bridge path
- delivered member class such as update, removal, continuity remap, replay
  member, or heartbeat/no-op where admitted
- canonical payload digest or explicit payload-omitted reason
- diagnostics tier class
- counter snapshot or counter-digest reference

The member envelope is the canonical delivery truth. Callback payloads,
transport packets, host channel messages, and signal observer handles are
derived views.

Coalescing may alter:

- delivery window shape
- pacing
- retained richness
- batching boundaries

Coalescing must not alter:

- canonical member identity
- canonical ordering semantics
- admitted subscription identity
- continuation basis
- checkpoint meaning

Every coalesced window must be reconstructable into canonical delivered member truth from retained delivery artifacts.

### 5. Fanout Must Be Planned, Not Emergent

Shared subscription fanout must be represented by a bridge-owned plan:

- one admitted active subscription
- one or more admitted consumer contracts
- one fanout identity
- one delivery-family choice per admitted consumer class where needed
- one sharing or rejection explanation

The fanout plan must be constructed before delivery begins.

The delivery path must not rediscover sharing legality or consumer equivalence while dispatching members.

The fanout plan must lower into an execution layout before delivery begins.
That layout should be flat, indexed, and stable for the active delivery window:

- active subscription slot
- delivery family slot
- canonical member buffer range
- consumer binding slots
- per-consumer frontier slots
- acknowledgement policy slot
- diagnostics policy slot
- fanout projection ranges

Implementation may choose concrete storage, but the architecture must make the
layout visible as a proof artifact. A vector of callback objects or a map keyed
by host identifiers is not an acceptable hot-path layout.

Fanout must also carry a pacing and backpressure decision:

- all consumers caught up
- consumer lag admitted and bounded
- consumer lag requires independent delivery cursor
- consumer lag exceeds contract and rejects sharing
- consumer backpressure changes pacing only

Slow consumers may change their own admitted delivery windows. They may not
change canonical subscription delivery truth for the shared subscription.

### 6. Delivery Cost Profiles And Work Budgets Must Be Explicit

Every active delivery window must carry a `SubscriptionDeliveryCostProfile` or
equivalent sealed artifact selected before delivery execution.

The cost profile must bind:

- delivery density posture:
  - sparse member delivery
  - bounded coalesced window
  - dense restart/rebuild
  - rejected over-budget delivery
- maximum canonical member count for the window
- maximum coalesced member width
- maximum admitted fanout width
- maximum continuation event width
- maximum retained replay artifact width
- diagnostics richness posture
- allocation arena or buffer lifecycle scope
- clone budget and allowed clone reasons

The delivery path must consume this profile. It must not reinterpret policy,
consumer count, diagnostics tier, or stream density on each member.

If a delivery window exceeds the admitted profile, the bridge must either:

- seal the current window and open a new admitted window
- switch through an explicit dense restart/rebuild posture
- or reject with a typed over-budget failure before constructing rich delivery
  artifacts

Silent fallback to unbounded vectors, maps, or rich diagnostic assembly is out
of spec.

### 7. Subscription Checkpoints Are Not Raw Stream Offsets

An active subscription checkpoint must bind:

- admitted subscription identity
- delivery-family identity
- canonical delivered member frontier
- canonical acknowledged member frontier
- per-consumer acknowledged frontier where fanout has multiple pacing classes
- raw stream checkpoint where relevant
- continuation frontier where identity evolution has occurred
- coalescing frontier where windows were admitted
- basis identity and branch identity where relevant

Raw stream checkpoint identity may be included as an input, but it is not sufficient by itself.

Milestone 15 must distinguish at least these delivery frontiers:

- produced frontier: the bridge produced canonical member records
- delivered frontier: the bridge handed records to an admitted consumer binding
- acknowledged frontier: the consumer contract acknowledged the records under
  the admitted acknowledgement policy
- checkpoint frontier: the bridge sealed a resume-safe subscription checkpoint

Only acknowledged, bridge-admitted frontiers may publish resume checkpoints.

Resume must preserve idempotence:

- replaying already acknowledged members must either suppress them through an
  explicit duplicate-delivery proof or redeliver them under an explicitly
  idempotent consumer contract
- missing acknowledged members must fail typed
- out-of-order acknowledged frontiers must fail typed unless the delivery
  family declared unordered member semantics and carries a canonical set digest

Resume must fail typed when:

- the retained subscription identity mismatches
- the basis identity has drifted illegally
- the delivery family is incompatible
- retained stream material is truncated
- continuation frontier is stale or incompatible
- preview scope has been discarded

### 8. Continuation Is A First-Class Bridge Decision

Continuation consumes truth-owned lineage and merge artifacts but produces bridge-owned continuation artifacts.

Supported continuation outcomes must be typed:

- unchanged continuation
- replace continuation
- split continuation
- admitted merge-like continuation
- branch-local continuation
- explicit continuation rejection

Unsupported, ambiguous, or authority-denied cases must fail before active delivery produces misleading members.

Continuation records must bind:

- pre-evolution admitted subscription identity
- truth lineage or merge provenance consumed
- continuation outcome kind
- post-evolution subscription or delivery identity
- affected canonical slice set or route set
- branch basis where relevant
- typed rejection where continuity is denied

First-ship continuation must use a closed admitted continuation table:

- `Unchanged`
- `ReplaceOneToOne`
- `SplitOneToMany`
- `MergeLikeAdmitted`
- `BranchLocalOnly`
- `RejectedUnsupported`
- `RejectedAmbiguous`
- `RejectedAuthorityDenied`
- `RejectedBranchLeak`

The implementation may use different enum names, but it must be closed-world
for Milestone 15. A catch-all continuation handler is out of spec.

Continuation planning must consume locality indexes derived before the
continuation boundary:

- affected slice index
- branch-local subscription index
- lineage affected-identity index
- merge affected-identity index where merge-like continuation is admitted
- preview-scope index where preview subscriptions are active

The continuation planner may touch the affected subscription slice set and the
affected identity-evolution records. It must not scan the full active
subscription registry to discover candidates.

### 9. Preview Subscriptions Must Be Residue-Proven

Preview-scoped subscriptions are active bridge protocol entities, but they are not authoritative.

Preview activation requires:

- a preview basis identity
- a preview scope identity
- a parent authoritative or branch basis identity
- a preview lifecycle identity
- a preview residue scope

These must be carried by a proof-bearing preview basis artifact, not by raw
strings or an `is_preview` flag.

Preview discard must emit explicit zero-residue evidence for:

- authoritative truth subscription residue
- bridge subscription registry residue
- active delivery residue
- fanout and consumer contract residue
- continuation residue
- checkpoint and replay residue
- signal-branch visible residue where the bridge can observe it

Preview promotion must emit an explicit promotion-boundary record that binds:

- preview subscription identity
- preview basis identity
- target authoritative basis or branch identity
- admitted promotion outcome
- promoted subscription identity where promotion is admitted
- rejected promotion reason where promotion is denied

Preview identity must never silently become authoritative identity.

Preview residue proof must also be scope-indexed. Discarding one preview scope
must inspect only:

- subscriptions in that preview scope
- delivery windows in that preview scope
- checkpoint records in that preview scope
- continuation records in that preview scope
- signal-visible bridge artifacts in that preview scope

It must not scan all authoritative or all preview subscriptions unless the
discard scope itself is global and that breadth is explicitly reflected in the
cost profile and counters.

### 10. Diagnostics Must Stay Off The Hot Delivery Path

Milestone 15 must split delivery truth from diagnostics materialization.

The hot delivery path may emit compact diagnostic references:

- diagnostics policy identity
- counter digest
- delivery bundle digest
- failure digest where a failure occurs

It must not materialize rich explanation trees, full callback payload mirrors,
or replay narratives unless the admitted diagnostics posture requires it for
that boundary.

Rich diagnostics should be reconstructed from retained canonical artifacts
through the diagnostics entrypoint. If changing diagnostics tier changes
allocation count, clone count, member delivery count, fanout width, or
checkpoint frontier meaning on the ordinary delivery lane, the implementation
has path conflation.

### 11. Replay Must Reconstruct Delivery Meaning Offline

Replay must be able to reconstruct active subscription meaning from retained artifacts:

- admitted subscription bundle
- lifecycle record
- consumer contract record
- fanout record
- delivery record
- checkpoint record
- continuation record
- preview discard or promotion record

Replay may use parent-runtime replay surfaces to re-materialize truth inputs, but it must not depend on:

- live host callback objects
- process-local subscription registries beyond retained canonical registry identity
- ad hoc logs
- wall-clock delivery timing

### 12. Failure Classes Must Stay Typed And Local

Milestone 15 must add or reserve subscription-specific failure classes for:

- consumer contract rejection
- sharing incompatibility
- delivery-family rejection
- coalescing boundary violation
- delivery cost profile rejection
- delivery density transition rejection
- acknowledgement frontier violation
- checkpoint stale, truncated, or incompatible
- continuation unsupported, ambiguous, or authority-denied
- branch-leak prevention
- preview reuse after discard
- preview promotion denial
- replay mismatch

Host strings may decorate these failures. They may not define them.

## Complexity Contracts

Milestone 15 must name and prove boundedness for:

- active subscription activation
- consumer contract admission
- delivery cost profile selection
- shared fanout planning
- fanout execution layout construction
- delivery-family selection
- canonical member delivery record assembly
- coalesced delivery window assembly
- subscription checkpoint publication
- subscription resume admission
- continuation planning
- continuation candidate localization
- preview discard residue proof
- preview promotion admission
- active delivery replay
- diagnostics reference emission
- rich diagnostics reconstruction

The named boundary contracts should be stated in terms of:

- `c`: admitted consumer contract count for one active subscription
- `m`: canonical delivered member count in one delivery boundary
- `w`: admitted coalesced window width
- `s`: lowered slice count from the admitted subscription
- `e`: truth identity-evolution event count in one continuation boundary
- `r`: retained replay artifact count for the subscription
- `p`: preview-scoped active subscription count in one discard or promotion boundary
- `g`: fanout projection group count after fanout planning
- `a`: affected active subscription count from the locality index for one
  continuation or preview boundary
- `d`: retained diagnostic detail width requested after the hot path
- `b`: active delivery windows admitted as one batch

Representative complexity targets:

- consumer contract admission: `O(1)` or `O(log C)` lookup against frozen contract metadata, never a scan over host callbacks
- fanout planning: `O(c log c)` if canonical ordering is required, otherwise `O(c)`
- fanout execution layout construction: `O(c + g)` once per delivery window,
  not per delivered member
- per-member fanout projection: `O(g_touched)` for projection groups affected
  by that member, never `O(c)` unless all consumers are semantically touched
- canonical member delivery record assembly: `O(m)`
- coalesced delivery reconstruction: `O(w)` with retained member basis, not a broad stream rescan
- checkpoint publication: `O(1)` for frontier identities plus `O(w)` only when the current coalesced window must be sealed
- resume admission: `O(1)` for identity checks plus bounded retained artifact lookup, not replay of all historical delivery
- continuation planning: `O(e + s + a)` for the affected identity-evolution
  boundary, admitted slice set, and locality-indexed candidate subscriptions,
  not full subscription registry scan
- preview discard residue proof: `O(p)` over preview-scoped active subscriptions in the discard scope, not all authoritative subscriptions
- replay reconstruction: `O(r)` over retained subscription artifacts, not host-local runtime state
- diagnostics reference emission on the delivery hot path: `O(1)` per delivery
  window plus `O(1)` per failure, not `O(d)`
- rich diagnostics reconstruction: `O(d)` from retained canonical artifacts on
  the diagnostics path, not the delivery path
- batch delivery admission: `O(b)` for batch envelope work plus summed admitted
  window widths, without repeating family registry freeze, strategy selection,
  or consumer contract admission per member

Minimum counters:

- `subscription_activation_count`
- `subscription_delivery_cost_profile_selection_count`
- `subscription_delivery_cost_profile_rejection_count`
- `subscription_delivery_over_budget_rejection_count`
- `subscription_delivery_density_sparse_count`
- `subscription_delivery_density_coalesced_count`
- `subscription_delivery_density_dense_restart_count`
- `subscription_consumer_contract_admission_count`
- `subscription_consumer_contract_rejection_count`
- `subscription_shared_fanout_plan_count`
- `subscription_shared_fanout_rejection_count`
- `subscription_fanout_layout_build_count`
- `subscription_fanout_projection_group_count`
- `subscription_fanout_per_member_consumer_scan_count`
- `subscription_callback_identity_scan_count`
- `subscription_active_registry_scan_count`
- `subscription_continuation_candidate_index_lookup_count`
- `subscription_continuation_full_registry_scan_count`
- `subscription_delivery_record_count`
- `subscription_delivery_member_count`
- `subscription_delivery_family_selection_count`
- `subscription_coalesced_window_count`
- `subscription_coalesced_member_count`
- `subscription_acknowledgement_frontier_count`
- `subscription_acknowledgement_rejection_count`
- `subscription_checkpoint_publication_count`
- `subscription_checkpoint_rejection_count`
- `subscription_raw_stream_offset_only_rejection_count`
- `subscription_resume_admission_count`
- `subscription_resume_rejection_count`
- `subscription_continuation_plan_count`
- `subscription_continuation_rejection_count`
- `subscription_branch_leak_rejection_count`
- `subscription_preview_activation_count`
- `subscription_preview_discard_count`
- `subscription_preview_residue_check_count`
- `subscription_preview_residue_nonzero_count`
- `subscription_preview_promotion_count`
- `subscription_preview_promotion_rejection_count`
- `subscription_delivery_replay_count`
- `subscription_delivery_replay_mismatch_count`
- `subscription_diagnostics_reference_emit_count`
- `subscription_rich_diagnostics_hot_path_materialization_count`
- `subscription_rich_diagnostics_reconstruction_count`
- `subscription_delivery_arena_reset_count`
- `subscription_delivery_buffer_reuse_count`
- `subscription_diagnostics_bundle_count`
- `subscription_allocation_count`
- `subscription_clone_count`

Representative control lanes must assert zero for:

- `subscription_callback_identity_scan_count`
- `subscription_active_registry_scan_count`
- `subscription_fanout_per_member_consumer_scan_count`
- `subscription_continuation_full_registry_scan_count`
- `subscription_raw_stream_offset_only_rejection_count`
- `subscription_preview_residue_nonzero_count`
- `subscription_delivery_replay_mismatch_count`
- `subscription_rich_diagnostics_hot_path_materialization_count`

No implementation may:

- scan all active subscriptions for ordinary delivery of one active subscription
- scan all host callbacks to decide consumer sharing
- scan all consumers per delivered member after fanout layout construction
- scan the full active subscription registry to find continuation candidates
- reconstruct subscription meaning from live callback identity
- replay all historical stream material to admit a checkpoint resume
- publish a resume checkpoint from produced-but-unacknowledged delivery members
- materialize rich diagnostics on the delivery hot path unless diagnostics policy admits it
- allocate fresh delivery member vectors for every window when the admitted cost
  profile permits buffer reuse
- downgrade from sparse member delivery to dense reconstruction without an
  explicit density transition artifact and counter
- clone delivery member vectors unless the clone buys a retained-artifact, fanout-isolation, or replay-boundary ownership reason
- continue through identity evolution without a retained continuation artifact
- or treat preview discard as successful without residue counters and residue artifacts

## Phases

### Phase 1: Active Delivery And Consumer Contract Foundation

Define and implement:

- active subscription activation from Milestone 14 activation-ready artifacts
- phase-typed active delivery states for active, window-open, window-sealed,
  checkpoint-ready, checkpoint-published, resume-admitted, and deactivated
  surfaces
- delivery cost profile vocabulary with sparse, coalesced, dense-restart, and
  over-budget rejection postures
- bridge-owned consumer contract vocabulary
- consumer contract identity and admission artifacts
- sharing eligibility witness vocabulary, even if shared fanout itself waits
  for Phase 2
- typed consumer contract rejection
- initial delivery-family taxonomy:
  - canonical member delivery
  - admitted coalesced delivery
  - replay or audit-oriented delivery
  - route-focused delivery
- the first-ship canonical delivery member envelope with sequence, route/slice,
  basis, causality, member class, payload digest or omitted reason, diagnostics
  tier, and counter reference fields
- delivery-family identity and delivery record artifacts
- hot-path diagnostics reference artifacts separated from rich diagnostics
  reconstruction artifacts
- delivery diagnostics tying admitted subscription identity, consumer contract identity, delivery family, and selected signal strategy together

Phase 1 implementation guidance:

- begin with the existing Milestone 14 families `DetailExact` and `CollectionMembership`
- make activation consume retained lifecycle artifacts directly
- treat consumer callbacks as opaque host endpoints outside canonical identity
- keep signal observer registration behind admitted signal strategy descriptors
- make coalescing reconstructable from canonical member records before adding richer fanout
- reject delivery windows that exceed the admitted cost profile before
  constructing rich artifacts
- use a delivery-window arena or reusable buffer plan from the start, even if
  the first buffer implementation is simple
- add compile-fail coverage preventing external construction of active
  delivery records, delivery windows, admitted consumer contracts, and sharing
  eligibility witnesses

Phase 1 is complete only when a single active subscription can deliver canonical member records through at least two delivery families without changing admitted subscription meaning.

### Phase 2: Shared Fanout, Coalescing Parity, And Delivery Diagnostics

Implement:

- shared fanout plans over admitted consumer contracts
- compact fanout execution layout construction before delivery
- separate-but-equivalent subscription activation comparison lanes
- typed sharing rejection for non-equivalent or incompatible consumer contracts
- coalesced delivery parity artifacts proving pacing changes only
- per-consumer delivery views derived from one canonical delivery truth
- explicit consumer pacing and backpressure outcomes for caught-up, lagged,
  independent-cursor, lag-rejected, and pacing-only lanes
- exact fanout, coalescing, member, allocation, and clone counters
- exact per-member consumer scan and fanout layout counters
- certification coverage for suite 31

Phase 2 implementation guidance:

- fanout planning should consume admitted consumer contracts, not raw consumer objects
- fanout layout construction is allowed once per delivery window; per-member
  dispatch must consume the layout and assert zero consumer scans
- canonical delivery truth should be produced once per active subscription boundary and then projected into per-consumer views
- retained diagnostics should explain why sharing was admitted or rejected without materializing callback internals
- coalesced delivery must retain enough member identity to reconstruct canonical delivery truth offline
- slow consumer lanes must prove the slow consumer affects only its admitted
  consumer frontier or sharing eligibility, not canonical subscription delivery
  truth

Phase 2 is complete only when shared and separate-but-equivalent subscriptions compare equal on canonical subscription and delivery truth while rejected sharing fails typed before delivery.

### Phase 3: Continuation Across Identity Evolution

Implement:

- bridge-owned continuation decision artifacts
- locality indexes for affected slices, branch-local subscriptions, lineage
  affected identities, merge affected identities, and preview scopes
- continuation outcomes for unchanged, replace, split, admitted merge-like, branch-local, and rejected continuity
- a closed first-ship continuation table covering unchanged, one-to-one
  replace, one-to-many split, admitted merge-like, branch-local-only,
  unsupported, ambiguous, authority-denied, and branch-leak outcomes
- continuity planning over truth-owned lineage, merge, and branch artifacts
- continuation digests and replay-visible explanation records
- branch-leak rejection when branch-local identity evolution attempts to affect unrelated subscriptions
- exact continuation counters and typed failure classes
- exact candidate-index lookup and full-registry-scan counters
- certification coverage for suite 32

Phase 3 implementation guidance:

- consume existing bridge lineage, merge, structural-remap, and branch artifacts rather than inventing a new lineage model
- build continuation candidate sets from locality indexes before planning; do
  not discover candidates by scanning active subscriptions
- make ambiguous continuity an explicit rejection, not a fallback to structural similarity
- continuation must bind pre-evolution and post-evolution subscription or delivery identity
- split continuation may produce multiple child continuation records, but each child must remain attributable to one parent continuation decision
- admitted merge-like continuation must remain traceable back to truth-owned merge ontology and policy outcome
- if truth authority denies continuity, structural identity may appear only as
  diagnostic context; it must not reopen continuation

Phase 3 is complete only when active subscriptions continue, split, merge-like continue, branch-local continue, or reject deterministically and replay preserves continuation meaning.

### Phase 4: Subscription Checkpoint, Resume, And Delivery Replay

Implement:

- subscription checkpoint identity distinct from raw stream checkpoint identity
- checkpoint publication records for delivery frontiers, coalesced windows, and continuation frontiers
- checkpoint cost profile checks that reject over-wide retained replay windows
  before replay artifact construction
- distinct produced, delivered, acknowledged, and checkpoint frontiers
- acknowledgement policy descriptors attached to admitted consumer contracts
- resume admission plans from retained subscription checkpoint artifacts
- typed stale, truncated, incompatible, basis-drift, delivery-family-drift, and preview-discarded checkpoint rejection
- typed rejection for raw-stream-offset-only resume attempts
- duplicate-delivery suppression or idempotent-redelivery proofs for replayed
  acknowledged members
- active delivery replay from retained artifacts
- replay mismatch localization for subscription identity, delivery identity, continuation identity, and checkpoint identity
- exact resume, replay, checkpoint, and mismatch counters
- exact replay reconstruction, diagnostics-reference, rich-diagnostics, buffer
  reuse, and allocation counters
- certification coverage for suite 33

Phase 4 implementation guidance:

- checkpoint records should include raw stream checkpoint input where relevant, but never treat it as the complete subscription checkpoint
- resume should reject before observer activation if identity or basis proof does not match
- replay should reconstruct delivery meaning from retained artifacts and parent-runtime truth replay surfaces, not live host callback state
- replay should consume retained delivery artifacts in window order; it must not
  rebuild delivery by replaying unrelated stream history
- coalesced delivery replay must reconstruct canonical member truth before comparing digests
- no checkpoint publication path may consume a produced or delivered frontier
  where an acknowledged frontier is required

Phase 4 is complete only when restart and replay lanes preserve active delivery semantics exactly and hostile checkpoint drift fails typed before partial delivery.

### Phase 5: Preview-Scoped Subscription Isolation, Discard, And Promotion

Implement:

- preview-scoped active subscription admission over explicit preview basis artifacts
- `PreviewSubscriptionBasisBinding` or equivalent sealed proof derived from the
  preview/speculation boundary, with preview scope, parent basis, lifecycle, and
  residue-scope identities
- preview delivery records and lifecycle records distinct from authoritative active subscription records
- preview discard residue proof artifacts
- preview-scope indexes for active subscription, delivery, checkpoint,
  continuation, and signal-visible residue records
- preview promotion-boundary records and typed promotion rejection
- illegal preview reuse and cross-session sharing rejection
- preview replay artifacts that do not depend on ambient current state
- exact preview activation, discard, residue, promotion, rejection, and replay counters
- exact preview-scope index lookup and non-preview registry scan counters
- certification coverage for suite 34

Phase 5 implementation guidance:

- preview basis must consume Milestone 10 speculative/preview boundary
  artifacts and Milestone 14 basis machinery. If either side lacks a required
  proof, the implementation must add the missing proof or reject preview
  activation typed; it must not create a parallel preview-basis shortcut.
- discarded preview subscriptions must prove zero residue through explicit artifact categories and counters
- discard must inspect preview-scope indexes, not the full authoritative or
  global preview registry
- promotion must create a new authoritative-boundary record; it must not mutate preview identity into authoritative identity in place
- preview sharing must be admitted only within the preview scope unless a promotion record proves a legal boundary crossing

Phase 5 is complete only when preview subscriptions can deliver, continue where admitted, discard with zero authoritative residue, and promote only through explicit typed boundaries.

## Must Ship

- active subscription activation contracts consuming Milestone 14 activation-ready artifacts
- phase-typed active lifecycle states for active, open delivery window, sealed
  delivery window, checkpoint-ready, checkpoint-published, resume-admitted,
  preview-active, preview-discarded, preview-promoted, and deactivated surfaces
- sealed delivery cost profiles and work budgets for sparse, coalesced,
  dense-restart, and rejected over-budget delivery windows
- admitted consumer contract vocabulary and typed consumer contract rejection
- canonical consumer contract identity and sharing equivalence records
- sealed sharing eligibility witnesses and fanout admission witnesses
- compact fanout execution layouts with per-consumer frontier slots and
  projection ranges
- delivery-family vocabulary for canonical member, admitted coalesced, replay/audit-oriented, and route-focused delivery
- one concrete canonical delivery member envelope with sequence, route/slice,
  basis, causality, member class, payload digest or omitted reason, diagnostics
  tier, and counter reference fields
- delivery records and delivery bundle digests tied to admitted subscription identity
- shared fanout plans and typed sharing incompatibility artifacts
- subscription checkpoint identity distinct from raw stream checkpoints where delivery semantics require it
- produced, delivered, acknowledged, and checkpoint frontier identities
- acknowledgement policy descriptors and acknowledgement rejection artifacts
- resume admission plans and typed checkpoint rejection artifacts
- active delivery replay artifacts and replay mismatch localization
- continuation records for replace, split, admitted merge-like, branch-local, unchanged, and rejected continuity outcomes
- a closed first-ship continuation outcome table with explicit unsupported,
  ambiguous, authority-denied, and branch-leak rejections
- locality indexes for continuation and preview residue candidate selection
- preview-scoped active subscription records, preview basis proof artifacts,
  discard residue proofs, and promotion-boundary records
- hot-path diagnostics references and separate rich diagnostics reconstruction
  surfaces
- diagnostics visible through the bridge diagnostics entrypoint for activation, consumer admission, fanout, delivery, coalescing, continuation, checkpoint, resume, replay, preview discard, and preview promotion
- exact counters for all named Milestone 15 boundaries
- compile-fail or equivalent privacy coverage preventing external construction of admitted active delivery, fanout, continuation, checkpoint, replay, preview, promotion, and residue-proof artifacts
- certification satisfying suites 31 through 34

## Must Preserve

- truth authority remains in `worth-relational`
- observation execution authority remains in `worth-signal`
- the bridge remains a protocol boundary, not an observation runtime or truth runtime
- Milestone 14 declaration, admission, basis, and strategy proofs remain prior proofs consumed by active delivery
- consumer pacing does not redefine subscription meaning
- coalescing does not redefine canonical delivered member truth
- fanout does not mutate subscription identity
- branch identity remains explicit through continuation, checkpoint, and replay
- preview identity remains separate from authoritative identity until explicit promotion
- diagnostics richness changes retained detail only
- replay and resume do not depend on host-local callbacks, logs, or process memory
- delivery cost scales with touched canonical members, admitted fanout
  projection groups, and affected continuation or preview scope, not with all
  consumers, all active subscriptions, all diagnostics detail, or all retained
  stream history
- rich diagnostics are reconstructed off the hot path unless explicitly
  admitted by diagnostics policy
- allocation and clone behavior remains tied to delivery-window lifecycle,
  retained-artifact ownership, fanout isolation, or replay boundaries

## Acceptance Evidence

Milestone 15 is complete only when the bridge harness can prove all of the following:

- one admitted subscription can activate from retained Milestone 14 artifacts without re-admitting declaration or basis meaning
- phase-typed APIs make it impossible for external callers to deliver before
  activation, checkpoint before window sealing, resume before checkpoint
  admission, or use preview APIs on authoritative subscriptions
- admitted consumer contracts are canonical and independent of callback identity
- sharing eligibility is proven by bridge-minted witnesses, not by matching
  raw consumer labels or callback shapes
- canonical delivery member envelopes retain enough route, basis, causality,
  class, payload, diagnostics, and counter identity to reconstruct delivery
  truth offline
- shared and separately activated equivalent subscriptions agree on `subscription_digest`, `subscription_share_digest`, and `subscription_delivery_digest`
- non-equivalent or incompatible sharing attempts fail explicitly before delivery
- admitted coalescing changes pacing and window shape only while preserving reconstructable canonical member delivery truth
- slow consumer and backpressure lanes alter only admitted consumer frontiers,
  independent cursor posture, or typed sharing eligibility, not canonical
  subscription meaning
- continuation after replace, split, admitted merge-like, and branch-divergent identity evolution is deterministic and typed
- ambiguous, unsupported, or authority-denied continuation fails before misleading delivery artifacts are emitted
- resumed subscriptions match original delivery semantics from acknowledged
  subscription checkpoint identity rather than raw stream offset alone
- produced-but-unacknowledged members cannot publish resume checkpoints
- duplicate replay of acknowledged members is either explicitly suppressed or
  redelivered only under idempotent consumer-contract proof
- stale, truncated, incompatible, basis-drifted, delivery-family-drifted, and preview-discarded checkpoint attempts fail typed
- replay reconstructs active delivery, fanout, checkpoint, and continuation meaning from retained artifacts without host-local callback state
- preview activation requires sealed preview basis proof derived from the
  preview/speculation boundary; ordinary branch-head basis cannot masquerade
  as preview basis
- preview subscription discard leaves zero authoritative truth residue, zero bridge-visible subscription residue, and zero retained delivery or checkpoint residue outside the preview scope
- preview promotion produces explicit basis-bound promotion records and never mutates preview identity into authoritative identity in place
- exact Milestone 15 counters match declared values for representative control, hostile, and replay lanes
- performance counters prove zero per-member consumer scans, zero continuation
  full-registry scans, zero callback identity scans, zero rich diagnostics
  hot-path materialization, and bounded allocation/clone behavior on
  representative control lanes
- sparse, coalesced, dense-restart, and over-budget delivery density postures
  are all distinguishable through typed artifacts and counters
- certification suites 31 through 34 pass with canonical machine-checkable bundles

## Compile-Time Enforcement Obligations

Milestone 15 must add compile-fail or equivalent external-boundary tests
proving external crates cannot:

- construct `BridgeActiveSubscription` or equivalent active handles directly
- open a delivery window without an active subscription proof
- seal or checkpoint a delivery window that was never opened
- publish a checkpoint from produced or delivered frontiers instead of an
  acknowledged frontier
- construct an admitted consumer contract without consumer-contract admission
- construct a sharing eligibility witness from matching labels or raw callback
  types
- pass a raw callback, host channel, or signal observer handle where an
  admitted consumer contract is required
- pass a raw stream checkpoint where a subscription checkpoint is required
- resume an active subscription without a resume-admission proof
- construct a continuation decision or continuation child record without the
  bridge continuation planner
- use a rejected continuation artifact as an admitted continuation artifact
- call preview discard or promotion APIs with an authoritative active
  subscription
- call authoritative delivery APIs with a discarded preview subscription
- construct preview basis, preview residue, or promotion-boundary proofs
  directly
- substitute a coalesced delivery window digest for canonical member delivery
  truth
- construct a delivery window without an admitted delivery cost profile
- downgrade from sparse delivery to dense restart without a density-transition
  proof
- call rich diagnostics reconstruction from the hot delivery window type
- pass an unindexed active subscription registry where a continuation
  candidate index is required
- pass a global preview registry where a preview-scope residue index is
  required
- construct clone-budget or allocation-budget witnesses outside the delivery
  cost-profile admission path

These compile-fail tests are not optional polish. They are the mechanical proof
that Milestone 15 did not reintroduce subscription meaning as host convention.

## First-Ship Certification Matrix

Milestone 15 should not wait until late QA to discover what concrete cases it
must prove. The first implementation must build toward this minimum matrix:

| Lane | Families | Required proof |
| --- | --- | --- |
| shared detail fanout | `DetailExact` | two equivalent consumers share one active subscription and match separate activation digests |
| shared collection fanout | `CollectionMembership` | shared and separate lanes preserve collection-member delivery truth |
| coalesced pacing | `DetailExact` and `CollectionMembership` | coalesced windows reconstruct the same canonical member sequence as non-coalesced delivery |
| dense restart | `CollectionMembership` | dense posture is selected through cost profile and counted, not by silent broad replay |
| over-budget delivery | at least one family | over-budget window rejects before rich artifact construction |
| incompatible consumer | `DetailExact` | non-equivalent replay/audit versus pacing-only consumer contract rejects sharing before delivery |
| slow consumer | `CollectionMembership` | lag affects only the admitted consumer frontier or independent cursor, not canonical subscription truth |
| fanout layout | both families | fanout layout builds once and per-member consumer scans remain zero |
| one-to-one replace | `DetailExact` | continuation carries truth-owned lineage and stable continuation digest |
| one-to-many split | `CollectionMembership` | parent continuation decision produces attributable child continuation records |
| merge-like admitted | at least one family | merge-like continuation is traceable to truth-owned merge ontology |
| ambiguous continuation | at least one family | structural similarity does not reopen denied or ambiguous continuity |
| branch-local divergence | `DetailExact` and `CollectionMembership` | branch-local continuation cannot leak into unrelated authoritative subscriptions |
| continuation locality | both families where admitted | continuation candidate lookup uses locality indexes and full-registry scan count remains zero |
| checkpoint resume | both families | resume from subscription checkpoint preserves delivery digest |
| raw-offset resume rejection | both families | raw stream offset without subscription checkpoint proof rejects typed |
| partial acknowledgement restart | at least one family | produced-but-unacknowledged members do not publish a resume checkpoint |
| duplicate replay | at least one family | acknowledged members are suppressed or redelivered only with idempotence proof |
| diagnostics richness | both families | rich diagnostics reconstruct off hot path and do not change delivery, allocation, or clone counters |
| preview discard | `DetailExact` and `CollectionMembership` | discard proves zero residue across authority, bridge, delivery, checkpoint, and signal-visible scopes |
| preview scope locality | both families | preview discard inspects preview-scope indexes and global registry scan count remains zero |
| preview promotion | at least one family | promotion creates a new authoritative-boundary record instead of mutating preview identity |

Every matrix lane must emit:

- canonical bundle digest
- relevant subscription/delivery/share/continuation/checkpoint/preview digest
- typed failure digest for rejected lanes
- exact counter snapshot
- diagnostics digest proving richer diagnostics did not alter canonical meaning

No lane may pass by checking only that a digest is non-empty.

## Architectural Notes

Milestone 15 should extend the bridge crate with subdomains such as:

- `subscription/delivery_cost.rs`
- `subscription/consumer_contract.rs`
- `subscription/delivery_family.rs`
- `subscription/active_delivery.rs`
- `subscription/fanout.rs`
- `subscription/fanout_layout.rs`
- `subscription/delivery_record.rs`
- `subscription/checkpoint.rs`
- `subscription/resume.rs`
- `subscription/continuation.rs`
- `subscription/continuation_index.rs`
- `subscription/preview.rs`
- `subscription/preview_scope.rs`
- `subscription/residue.rs`
- `subscription/delivery_replay.rs`
- `subscription/delivery_diagnostics.rs`
- `subscription/delivery_buffers.rs`

Recommended implementation order:

- first land `delivery_cost.rs`, `consumer_contract.rs`,
  `delivery_family.rs`, `active_delivery.rs`, and `delivery_buffers.rs`
- then land `fanout.rs`, `fanout_layout.rs`, `delivery_record.rs`, and
  delivery diagnostics references
- then land `continuation.rs` and `continuation_index.rs`
- then land `checkpoint.rs`, `resume.rs`, and `delivery_replay.rs`
- finally land `preview.rs`, `preview_scope.rs`, and `residue.rs`

Expected facade growth should look more like:

- `admit_subscription_delivery_cost_profile(...)`
- `activate_subscription_delivery(...)`
- `admit_subscription_consumer_contract(...)`
- `plan_shared_subscription_fanout(...)`
- `build_subscription_fanout_layout(...)`
- `deliver_subscription_members(...)`
- `publish_subscription_checkpoint(...)`
- `admit_subscription_resume(...)`
- `plan_subscription_continuation(...)`
- `discard_preview_subscription(...)`
- `promote_preview_subscription(...)`
- `inspect_active_subscription(...)`

and not like:

- raw callbacks registered as subscription identity
- raw stream checkpoint resume APIs that bypass subscription checkpoint proof
- direct signal observer handles exposed as bridge delivery handles
- host-local lineage repair callbacks
- preview state toggled into authoritative state by mutation of one object
- generic `deliver(...)` surfaces that hide cost profile selection, fanout
  layout construction, diagnostics posture, or density transition

Temporary seams allowed during Milestone 15 bring-up:

- delivery may terminate in canonical bridge delivery records plus admitted signal delivery descriptors before broad live host adapter ergonomics are polished
- preview promotion may produce promotion-boundary records without full Store-backed persistence
- merge-like continuation may admit only the merge classes already consumed by earlier bridge milestones and reject the rest explicitly
- fanout may start with two consumer contracts and two admitted subscription families as long as the framework is not hard-coded to those cases
- the first delivery buffer may be a simple window-local arena or reusable
  scratch buffer, but the lifecycle and counters must exist immediately

## Test And Harness Model

Milestone 15 is active-subscription certification first.

The harness must define at least these scenario verbs:

- `activate_subscription_for_delivery(...)`
- `admit_delivery_cost_profile(...)`
- `attach_equivalent_subscription_consumers(...)`
- `attach_incompatible_subscription_consumer(...)`
- `build_fanout_layout(...)`
- `deliver_subscription_window(...)`
- `coalesce_subscription_delivery(...)`
- `force_dense_delivery_posture(...)`
- `reject_over_budget_delivery_window(...)`
- `checkpoint_active_subscription(...)`
- `resume_active_subscription(...)`
- `continue_subscription_after_identity_evolution(...)`
- `continue_subscription_from_locality_index(...)`
- `replay_active_subscription_delivery(...)`
- `reconstruct_subscription_diagnostics(...)`
- `discard_preview_subscription(...)`
- `promote_preview_subscription(...)`

The harness must vary:

- declaration family
- consumer contract shape
- consumer pacing
- coalescing policy
- delivery density posture
- delivery cost profile
- fanout group layout
- diagnostics richness posture
- diagnostics tier
- checkpoint boundary
- restart boundary
- truth identity evolution class
- branch identity
- preview discard versus promotion
- admitted versus rejected continuation paths

Minimum certification outputs:

- `subscription_digest`
- `subscription_share_digest`
- `subscription_delivery_digest`
- `subscription_continuation_digest`
- `subscription_basis_digest`
- `subscription_lifecycle_digest`
- `consumer_contract_digest`
- `checkpoint_digest`
- `routing_digest`
- `failure_digest`
- `replay_digest`
- `diagnostics_digest`
- `counter_snapshot`
- `delivery_cost_profile_digest`
- `fanout_layout_digest`
- `delivery_density_report`

## Anti-Patterns Explicitly Rejected

- treating callbacks as canonical consumers
- treating one raw signal observer handle as the bridge subscription
- treating stream offsets as complete subscription checkpoints
- recomputing fanout groups per delivered member
- scanning all active subscriptions for continuation or preview discard
- hiding dense restart behind an ordinary coalesced delivery path
- allowing rich diagnostics to allocate or clone on ordinary delivery hot paths
- using general heap allocation where the cost profile admitted a window-local
  buffer lifecycle
- selecting delivery family during dispatch instead of before delivery
- letting coalescing alter canonical member truth
- sharing subscriptions across consumers without admitted consumer-contract equivalence
- silently widening failed continuation into full resubscribe behavior
- using structural similarity to override truth-owned lineage or merge denial
- allowing branch-local continuation to affect unrelated authoritative subscriptions
- proving preview discard by object drop, absence of panic, or host log inspection
- reconstructing replay from host-local callback registries

## Sequencing Notes

Milestone 15 builds directly on:

- Milestone 2 fine-grained subscription slices
- Milestone 3 lineage-aware continuity
- Milestone 4 historical and branch-aware truth basis
- Milestone 6 stream checkpoint and multi-consumer protocol work
- Milestone 10 preview and speculative boundary work
- Milestone 13 diagnostics and certification-bundle discipline
- Milestone 14 declaration-family, admission, signal-strategy lowering, lifecycle, and replay artifacts

It belongs before Milestone 16 because Milestone 16 certifies the full subscription family story end to end. That final certification is only honest once active delivery, fanout, continuation, checkpoint/replay, and preview residue are already canonical bridge artifacts.

Milestone 15 also leaves a clean handoff to WORTH Query:

- Query may later compile live query intent into Milestone 14 declarations and Milestone 15 delivery contracts
- Query must not become the only owner of active subscription semantics
- manual hosts must retain a library-grade bridge surface for the same lower-level protocol

## Self-Check

- This solves a real structural problem: Milestone 14 produced admitted activation-ready subscriptions, but active delivery, sharing, continuation, checkpoint, replay, and preview behavior are still not bridge-native.
- The adversarial constraint is precise and load-bearing: consumer pacing, coalescing, identity evolution, restart, replay, and preview discard are the failure modes that would break a naive implementation.
- Authority boundaries are preserved: truth owns truth evolution, signal owns observation execution, and the bridge owns delivery protocol, continuation interpretation, fanout contracts, and retained proof artifacts.
- The spec defines proof obligations, not chores: sharing parity, coalescing reconstruction, checkpoint exactness, continuation determinism, replay parity, preview zero-residue, typed failures, and exact counters are all machine-checkable.
- A competent engineer can map this into honest modules, types, facade methods, counters, harness scenarios, and compile-fail boundaries.
- The milestone belongs in sequence: it consumes Milestone 14 admission/lifecycle proofs and produces the active-subscription artifacts Milestone 16 needs for end-to-end certification.

## Closeout Standard

Milestone 15 is complete only when the bridge can activate admitted subscription artifacts, admit consumer contracts, plan shared fanout, deliver canonical and coalesced subscription members, checkpoint and resume active subscriptions, continue or reject subscriptions across truth identity evolution, replay active delivery from retained artifacts, and prove preview subscription discard or promotion boundaries with exact counters and typed diagnostics.

If active subscription meaning still depends on host callback identity, if coalescing can change canonical member truth, if resume is justified by raw stream offsets alone, if continuation happens without retained bridge artifacts, if preview discard leaves unproven residue, or if replay requires process-local host state, Milestone 15 is not complete.
