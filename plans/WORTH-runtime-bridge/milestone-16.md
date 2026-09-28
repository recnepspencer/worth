# Milestone 16 Engineering Spec: Subscription Family Certification and End-to-End Subscription Workload

> **Status:** Planned
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-15.md](./milestone-15.md)
>
> **Prior closeout:** [milestone-15-closeout.md](./milestone-15-closeout.md)
>
> **Bridge certification companion:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** turn Milestones 14 and 15 subscription declaration, admission, active delivery, fanout, continuation, checkpoint, replay, and preview artifacts into one offline-certifiable subscription story with canonical bundles, typed subscription failure localization, and a concrete end-to-end reference workload.

## Summary

Milestone 14 made bridge-native subscription declarations real:

- declaration families are canonical
- basis binding is explicit
- admitted subscription identity is separate from slice, stream, and consumer identity
- signal strategy lowering is retained and replay-visible
- lifecycle reaches activation-ready and retained deactivation artifacts

Milestone 15 closed the active bridge protocol layer above those admitted declarations:

- delivery families are explicit
- consumer contracts and shared fanout are admitted
- canonical delivery member truth is retained
- continuation across truth identity evolution is typed
- subscription checkpoint, resume, and replay have bridge-owned identity
- preview subscription discard and promotion boundaries are proof-bearing
- descriptor-only fanout, replay, resume, and preview-work records preserve
  bridge meaning without requiring host callback execution

Milestone 16 is the certification layer above those two milestones.

It does not invent a new subscription runtime, a Query feature, or a UI-facing
watch API. It proves that the subscription protocol already built by Milestones
14 and 15 composes end to end under hostile lifecycle pressure.

The milestone must answer:

- can an auditor compare original execution, replay, restart, hostile adapter variation, and diagnostics-tier variation from canonical subscription bundles alone
- can branch-local, historical, authoritative, preview, shared-consumer, and continuation subscription flows coexist without semantic drift
- can subscription-specific failures be localized without collapsing into generic stream, source, or host errors
- can a concrete reference workload prove long-lived ongoing observation without making the bridge own domain semantics

## Goal

Make bridge-native subscriptions certifiable as a complete protocol family by shipping:

- one canonical subscription certification bundle model
- subscription-specific additions to the bridge failure taxonomy
- subscription-specific extensions to the public bridge diagnostics entrypoint
- offline bundle comparison rules for subscription equivalence, divergence, and rejection
- one Rust-only reference workload extension proving long-lived subscriptions across authoritative, historical, branch-local, preview, fanout, restart, continuation, discard, and promotion flows
- certification coverage for multiple admitted declaration families and their distinct `worth-signal` strategy lowerings

## Why This Milestone Exists

Milestone 16 belongs immediately after Milestone 15 because active subscription
behavior is not enough by itself.

After Milestone 15, the bridge can produce retained artifacts for active
delivery, sharing, continuation, checkpoint, replay, and preview residue. But
the bridge still needs one final proof:

- those artifacts compose into one end-to-end subscription bundle
- semantically equivalent lanes compare equal even when produced by different hosts or replay paths
- intentionally different declarations, bases, delivery families, continuation decisions, or preview outcomes compare unequal where they should
- rejected subscription paths fail at the correct boundary with typed evidence
- an offline auditor can diagnose the workload from canonical artifacts without live host state, process memory, or logs

Without Milestone 16, Milestones 14 and 15 would be individually strong but not
platform-grade. A host could still need bespoke interpretation to decide whether
a long-lived subscription survived restart, branch divergence, continuation,
preview discard, or shared fanout correctly.

Milestone 16 also creates the clean handoff to WORTH Query. Query may later
compile high-level live query intent into bridge subscription declarations and
delivery contracts, but Query must not become the only place where subscription
correctness can be certified.

## Hard Part

The hard part is not producing more diagnostics.

The hard part is making diagnostics, replay, and reference workload proof
strictly subordinate to canonical subscription meaning.

A naive certification layer will drift in one of these ways:

- it treats host logs as proof
- it compares only success paths and ignores intentional divergence
- it certifies one happy declaration family while claiming family-wide coverage
- it lets diagnostics richness change retained bundle meaning
- it merges declaration-family identity, active instance identity, consumer identity, and delivery-family identity into one digest
- it cannot distinguish subscription continuation failure from generic stream replay failure
- it proves preview discard by absence of visible callbacks rather than positive residue artifacts
- it lets the reference workload become a domain-specific integration test instead of a bridge certification fixture

Milestone 16 exists to make those failures mechanically visible.

## Adversarial Constraint

Milestone 16 must survive the following hostile condition:

> A Rust-only reference workload activates multiple admitted bridge subscription declaration families over authoritative, historical, branch-local, and preview truth bases; attaches shared and separate consumers with different admitted delivery contracts; drives bursty truth changes, replace and split lineage evolution, admitted merge-like continuation, branch divergence, restart, replay, diagnostics-tier variation, hostile adapter variation, preview discard, and preview promotion; and must preserve canonical subscription meaning, bundle digests, typed failure localization, residue proofs, exact counters, and offline diagnosis from canonical artifacts alone.

If any supported path:

- requires host logs, callback identity, process memory, or live runtime state to prove subscription meaning
- changes canonical subscription truth under diagnostics-tier variation
- lets replay reconstruct a different declaration, basis, delivery, continuation, or fanout meaning
- treats intentionally different subscription declarations as equivalent
- collapses subscription-specific failure into generic stream, source, or string errors
- allows branch-local or preview subscription state to leak into authoritative subscription bundles
- proves discard, continuation, or restart by absence rather than positive canonical artifacts
- or certifies only one declaration family while claiming family-aware bridge proof

then Milestone 16 has failed.

## Explicit Assumptions

- `worth-relational` remains the authority for truth identity, mutation history, lineage, merge ontology, branches, historical retention, and authoritative publication.
- `worth-signal` remains the authority for observation execution, scheduling, internal delivery strategy mechanics, and derived recomputation.
- The bridge owns subscription protocol artifacts, family-aware lowering evidence, continuation interpretation, diagnostics, replay bundles, and certification comparison rules.
- Milestone 14 has already shipped declaration-family admission, basis binding, signal-strategy lowering, lifecycle artifacts, diagnostics, and replay for declarations.
- Milestone 15 has shipped active delivery, consumer contracts, fanout, continuation, checkpoint, resume, retained replay planning, preview basis, preview work, preview discard, promotion, and zero-residue proof artifacts.
- Milestone 16 consumes Milestone 14 and 15 artifacts. It must not reopen declaration admission, active delivery, descriptor-only fanout/projection, checkpoint, resume, continuation, preview basis, promotion, or residue semantics as new ambient runtime behavior.
- The reference workload is a harness-owned certification fixture. Domain semantics used by that workload must not become bridge API semantics.
- Store-backed permanent persistence may remain outside the milestone unless needed to prove restart from retained canonical artifacts. The proof target is offline artifact sufficiency, not final product persistence.

## Product Decision Lock

- Milestone 16 is a certification milestone, not a new subscription feature milestone.
- The canonical subscription certification bundle is mandatory.
- The canonical bundle schema must be versioned. Bundle schema identity,
  digest algorithm identity, canonical ordering rules, and omission semantics
  are part of the certified artifact, not implementation details.
- Bundle comparison must prove equality, inequality, typed rejection, residue absence, and diagnostics-tier invariance where applicable.
- Every bundle must distinguish declaration-family identity, admitted instance identity, basis identity, lifecycle identity, delivery identity, sharing identity, continuation identity, and replay identity.
- Every required bundle field must be represented as `Present`, `NotExercised`,
  or `RejectedBeforeProduced` or equivalent typed field state. A missing field
  with no typed reason is a bundle insufficiency failure.
- Subscription failure taxonomy must be bridge-native and subscription-specific. It may wrap parent-runtime failures, but it must not collapse them into generic stream/source failures or strings.
- The public bridge diagnostics entrypoint must expose the subscription certification story coherently rather than requiring milestone-local debug helper calls.
- The reference workload must include at least two admitted declaration families and prove their distinct signal-strategy lowerings.
- Historical subscriptions must be certified through explicit retained basis artifacts. No lane may silently fall back to latest reachable truth.
- Branch-local and preview subscriptions must remain distinct from authoritative subscriptions until explicit continuation or promotion records say otherwise.
- Preview discard is certified by positive residue-proof artifacts and counters, not by object drop, lack of callbacks, or lack of host-visible changes.
- Diagnostics richness may alter retained detail only. It must not alter canonical bundle digests that define subscription meaning.
- Offline certification must be possible from emitted bundles and comparison rules alone.

Normative consequence:

- no host-local replay registry can be the source of subscription truth
- no Query-only path can be required to certify lower-level bridge subscriptions
- no diagnostic artifact can become the hidden authority for canonical subscription meaning
- no delivery-family specific bundle may hide the declaration-family and strategy-lowering provenance that produced it
- no unordered map, host insertion order, pointer identity, randomized hash
  state, wall-clock timestamp, thread id, task id, callback address, or
  allocation address may participate in any canonical bundle digest
- no string label alone may act as a certified comparison relationship,
  failure boundary, basis kind, declaration family, delivery family, or
  diagnostics tier

## Scope

### In Scope

- canonical subscription certification bundle schema
- canonical digest and comparison rules for subscription bundles
- subscription-specific bridge failure taxonomy additions
- diagnostics-entrypoint extensions for subscription certification artifacts
- offline certification reports for original, replay, restart, hostile adapter, and diagnostics-tier lanes
- certification of suites 35 through 37 in [test-requirements.md](./test-requirements.md)
- Rust-only reference workload extension for long-lived subscriptions
- reference workload lanes for authoritative, historical, branch-local, preview, shared-consumer, restart, continuation, discard, promotion, and hostile failure paths
- exact counter snapshots for bundle production, comparison, replay reconstruction, diagnostics materialization, failure localization, residue inspection, and family-aware strategy lowering proof
- compile-fail or equivalent boundary coverage preventing external construction or mutation of canonical certification bundles, failure taxonomy variants, residue proofs, and comparison witnesses

### Explicitly Out Of Scope

- new subscription declaration families beyond the minimum needed to certify family-aware behavior
- new active delivery semantics not already required by Milestone 15
- Query-owned live query lowering
- UI, wasm, or app-facing watch ergonomics
- broad permanent Store productization of subscription bundle persistence
- redefining `worth-signal` observation or delivery scheduler internals
- redefining `worth-relational` branch, lineage, merge, preview, or retention authority

Milestone 16 must leave the bridge with certifiable lower-level subscription
protocols that Query and Store can later consume. It must not make Query or
Store prerequisites for proving bridge-native subscription correctness.

## Governing Design Rules

### 1. Certification Consumes Prior Proofs

Milestone 16 starts from retained Milestone 14 and 15 artifacts.

It must consume:

- declaration-family registry identity
- normalized declaration identity
- admitted subscription instance identity
- basis binding identity
- selected signal strategy identity
- lifecycle typestate records
- active delivery records
- consumer contract records
- fanout and sharing records
- continuation records
- checkpoint and resume records
- replay records
- preview discard, residue, and promotion records

It must not recompute or reinterpret those identities from raw host declarations,
callback objects, signal observer handles, stream offsets, or logs.

The certification bundle is a derived artifact. The prior bridge artifacts are
the authority for subscription protocol meaning.

### 1.1 Minimum Proof Type Inventory

Milestone 16 must introduce concrete proof-bearing types rather than a loose
set of reports and helper functions.

The exact names may change, but the architecture must include equivalents of:

- `SubscriptionReferenceWorkloadManifestDraft`
- `SubscriptionReferenceWorkloadManifestSealed`
- `SubscriptionCertificationBundleDraft`
- `SubscriptionCertificationBundleSealed`
- `SubscriptionBundleHeader`
- `SubscriptionBundleField<T>`
- `SubscriptionBundleFieldState`
- `SubscriptionSourceArtifactIndex`
- `SubscriptionBundleCompletenessReport`
- `SubscriptionComparisonPlan<Relationship>`
- `SubscriptionComparisonReport<Relationship, Outcome>`
- `SubscriptionFailureBoundary`
- `SubscriptionFailurePrecedenceStage`
- `SubscriptionResidueProof`
- `SubscriptionStrategyLoweringProvenance`
- `SubscriptionOfflineAuditReport`
- `SubscriptionCertificationCounterSnapshot`

`SubscriptionBundleFieldState` or its equivalent must distinguish at least:

- `Present`
- `NotExercised`
- `RejectedBeforeProduced`
- `UnavailableBecausePriorArtifactMissing`
- `UnavailableBecauseSchemaIncompatible`

The field-state type must be sealed or otherwise impossible for external code
to construct as proof without bridge-owned validation.

The public facade may expose read-only inspection of these types. It must not
expose constructors that allow callers to synthesize proof.

### 2. Bundle Identity Must Preserve Semantic Layers

The canonical subscription certification bundle must keep separate digests or
records for:

- `subscription_digest`
- `subscription_registry_digest`
- `subscription_basis_digest`
- `subscription_lifecycle_digest`
- `subscription_delivery_digest`
- `subscription_share_digest`
- `subscription_continuation_digest`
- `consumer_contract_digest`
- `checkpoint_digest`
- `routing_digest`
- `replay_digest`
- `diagnostics_digest`
- `failure_digest`
- `residue_digest`
- `strategy_lowering_digest`
- `counter_snapshot`

A top-level `certification_bundle_digest` may summarize these records, but it
must not replace them.

The first concrete bundle shape must include these required records:

- `bundle_header`: schema version, bridge crate version or build identity,
  digest algorithm identity, canonical ordering identity, diagnostics policy
  identity, and artifact policy identity
- `lane_identity`: stable lane id, lane semantic relationship target, workload
  scenario id, and branch or preview scope identity where applicable
- `source_artifact_index`: retained Milestone 14 and 15 artifact identities
  consumed by the bundle
- `subscription_identity_records`: declaration-family, admitted instance,
  basis, lifecycle, signal-strategy, active-delivery, and replay identities
- `delivery_records`: canonical delivery member and window digests, delivery
  family, density posture, acknowledgement frontier, and omitted-payload
  reasons where payloads are intentionally absent
- `sharing_records`: consumer contract, sharing eligibility, fanout layout,
  per-consumer frontier, and rejected-sharing records
- `continuation_records`: lineage or merge source identity, continuation
  decision, child continuation identities, rejection reason, and branch scope
- `checkpoint_records`: subscription checkpoint identity, raw stream checkpoint
  reference where relevant, acknowledged canonical member boundary, resume
  admission, and stale or incompatible checkpoint rejection
- `preview_records`: preview basis, preview scope, discard residue proof,
  promotion boundary, and rejected cross-scope reuse records
- `comparison_inputs`: the records selected for equality, inequality, rejection,
  diagnostics-only, residue, replay, and counter comparison
- `failure_records`: typed subscription failure boundary, cause chain,
  precedence stage, and retained failure digest
- `counter_snapshot`: exact named counters for the lane
- `diagnostics_records`: diagnostics digest and retained detail class, separate
  from canonical semantic digests

Canonical ordering rules:

- subscription records are ordered by canonical admitted subscription identity
- delivery records are ordered by delivery epoch, canonical member sequence,
  and source route identity
- consumer records are ordered by admitted consumer contract identity
- continuation records are ordered by source truth identity, continuation
  decision class, and child continuation identity
- checkpoint records are ordered by subscription checkpoint identity and
  acknowledged canonical member boundary
- failure records are ordered by failure precedence stage and then failure
  boundary identity
- diagnostics records never participate in semantic equality except through
  explicit diagnostics-only comparison rules

Digest rules:

- every digest input must be canonical serialized data, not debug formatting
- every digest must carry its digest algorithm identity
- every digest must include the schema version of the record it summarizes
- absent optional data must be encoded through typed field state, never by
  eliding the field from canonical serialization
- rich diagnostics payloads may have their own digest, but that digest must be
  excluded from semantic equality unless the comparison rule explicitly targets
  diagnostics detail

The bundle must make at least four relationships mechanically checkable:

- equivalent lanes compare equal on canonical subscription meaning
- intentionally different lanes compare unequal on the declared semantic axis
- rejected lanes fail at the expected subscription boundary
- diagnostics-tier variation changes retained detail only

### 3. Comparison Rules Are First-Class Artifacts

Bundle comparison must be explicit, typed, and replay-visible.

The public comparison path must be phase-typed. The exact names may vary, but
the proof chain must have equivalents of:

- `SubscriptionCertificationBundleDraft`
- `SubscriptionCertificationBundleSealed`
- `SubscriptionComparisonPlan<Relationship>`
- `SubscriptionComparisonExecuted<Relationship>`
- `SubscriptionComparisonReport<Relationship, Outcome>`
- `SubscriptionOfflineAuditReady`
- `SubscriptionOfflineAuditReport`

`Relationship` must be a sealed type-level or enum-level relationship selected
from admitted comparison families such as:

- `SemanticEquivalence`
- `IntentionalDivergence`
- `ExpectedRejection`
- `DiagnosticsOnlyVariation`
- `ResidueAbsence`
- `ReplayEquivalence`
- `CounterContract`
- `BundleCompleteness`

External callers must not be able to construct a passing
`SubscriptionComparisonReport` directly or change the relationship after the
comparison plan is sealed.

The milestone must define comparison outcomes such as:

- equivalent
- intentionally divergent
- rejected at expected boundary
- rejected at unexpected boundary
- diagnostics-only difference
- residue mismatch
- replay mismatch
- counter-contract violation

Comparison must not be hard-coded into one test assertion per scenario. The
comparison rule selected for a lane must be retained in the certification
report so an auditor can tell what relationship was being proven.

Relationship-specific required comparisons:

- `SemanticEquivalence` compares semantic subscription records, delivery
  records, continuation records, checkpoint records, replay records, residue
  records, and relevant counters; it ignores rich diagnostics detail except for
  diagnostics policy identity
- `IntentionalDivergence` must name the axis expected to differ, such as
  basis, declaration family, delivery family, continuation decision, branch
  scope, preview outcome, or strategy lowering; all unrelated axes must still
  compare equal or be reported as unexpected drift
- `ExpectedRejection` must name the expected rejection boundary and precedence
  stage; a rejection at another stage is not a pass
- `DiagnosticsOnlyVariation` must prove semantic digests equal, diagnostics
  digests allowed to differ, and diagnostics influence counters equal zero
- `ResidueAbsence` must compare positive residue proof records and exact zero
  counters across authority, bridge, delivery, checkpoint, and signal-visible
  scopes
- `ReplayEquivalence` must compare original and replay records without live
  host state and must report retained-artifact insufficiency separately from
  semantic mismatch
- `CounterContract` must compare exact counter values or explicitly declared
  variable counters with bounded rationale; presence-only counter checks fail
  the relationship

### 4. Subscription Failures Must Localize To Subscription Boundaries

Milestone 16 must extend the bridge failure taxonomy with subscription-specific
classes for at least:

- declaration equivalence drift
- registry drift
- basis drift
- lifecycle transition mismatch
- consumer-contract mismatch
- illegal sharing reuse
- delivery-family mismatch
- delivery digest drift
- continuation denial or ambiguity
- checkpoint incompatibility
- replay mismatch
- preview residue
- promotion-boundary mismatch
- historical-basis unavailability
- branch-leakage attempt
- diagnostics influence
- strategy-lowering provenance mismatch
- bundle insufficiency

Parent-runtime failures may appear as causes, but the bridge-facing failure must
identify the subscription boundary where certification failed.

Failure localization must use an explicit precedence ladder so multi-failure
lanes do not pass by reporting whichever error happens to surface first.

Required precedence order:

1. bundle schema or digest incompatibility
2. missing required retained artifact or typed field state mismatch
3. declaration-family or registry drift
4. basis binding or truth-view drift
5. signal-strategy lowering provenance drift
6. lifecycle transition mismatch
7. consumer contract or sharing eligibility mismatch
8. delivery family, delivery member, or coalescing truth mismatch
9. continuation, lineage, merge, or branch-scope mismatch
10. checkpoint, resume, or replay reconstruction mismatch
11. preview scope, residue, discard, or promotion mismatch
12. diagnostics influence
13. counter-contract violation

When a lower-precedence failure is present behind a higher-precedence failure,
the report may retain it as a suppressed cause, but the primary failure must be
the highest-precedence violated boundary. Tests must include at least one lane
with multiple injected failures to prove precedence stability.

### 5. Reference Workload Must Be Concrete But Not Domain-Authoritative

The reference workload should extend the existing bridge certification story
from Milestone 13 rather than invent a disconnected toy.

The recommended first workload is subscription-aware pricing:

- `worth-relational` owns products, components, component costs, branches, history, and authoritative commits
- `worth-signal` owns derived tariff, tax, margin, product total, collection membership, and comparison nodes
- the bridge owns subscription declarations, admission, active delivery, fanout, continuation, checkpoint, replay, preview coordination, diagnostics, and certification bundles

The workload must include:

- at least 100 products with shared component dependencies
- one detail-oriented subscription family such as product final-price detail
- one collection-oriented subscription family such as affected-product membership
- main-branch live cost churn
- historical-basis subscription replay over retained truth
- branch-local subscription isolation
- shared equivalent consumers and separate equivalent subscriptions
- restart and resume from subscription checkpoints
- identity evolution requiring continuation records
- preview subscription discard with zero residue
- preview promotion through explicit promotion-boundary records
- hostile adapter or diagnostics-tier variation
- typed failures for illegal basis drift, illegal sharing, stale checkpoint, denied continuation, and preview residue

The workload is a certification fixture. Pricing, tariff, tax, or product
semantics must not leak into bridge protocol ownership.

First-ship fixture contract:

- products: exactly 128 products in the canonical fixture
- components: at least `steel`, `rubber`, `copper`, `glass`, and `labor`
- dependency shape: every product depends on at least two components; at least
  32 products share `steel`, at least 32 share `rubber`, and at least 16 share
  both so fanout and overlap are visible
- detail subscription: `DetailExact` over final price for at least 10 named
  products spanning shared and non-shared component sets
- collection subscription: `CollectionMembership` over the affected-product set
  for component-cost changes
- authoritative churn: a main-branch `steel` cost wave with at least 60 commit
  windows or an equivalent retained batch sequence that proves high-fanout
  subscription stability without requiring wall-clock timing
- branch-local shock: a speculative or branch-local `rubber +300%` change that
  affects branch-local subscriptions and does not alter authoritative bundles
- historical lane: replay a retained pre-shock basis and a retained post-shock
  basis, proving they differ on declared axes only
- continuation lane: one replace and one split-style truth identity evolution
  that requires continuation records for at least one detail subscription and
  one collection subscription
- preview discard lane: activate preview subscriptions, deliver at least one
  preview window, discard, and prove zero residue through positive residue
  records
- preview promotion lane: promote one preview outcome through explicit
  promotion-boundary records and prove preview identity is not mutated into
  authoritative identity in place
- hostile adapter lane: rerun the same canonical workload through an adapter
  with different host construction order, delivery pacing, and diagnostics
  tier while preserving admitted contracts
- failure lane: inject at least basis drift, illegal sharing, stale checkpoint,
  denied continuation, preview residue, and bundle insufficiency

The fixture must emit a stable `subscription_reference_workload_manifest`
record containing the fixture schema version, product ids, component ids,
branch ids, subscription declarations, admitted consumer contracts, expected
semantic relationships, and lane ids. The manifest is part of the bundle input
and must be canonicalized before any lane executes.

### 6. Diagnostics Are Derived, Not Canonical Authority

The diagnostics entrypoint must expose enough structured detail to inspect:

- declaration and basis provenance
- family-aware strategy lowering
- lifecycle transitions
- delivery windows and coalescing posture
- fanout and sharing decisions
- continuation and rejection decisions
- checkpoint and resume boundaries
- preview discard, residue, and promotion records
- replay and comparison outcomes
- counter snapshots and violated contracts

But diagnostics must not become the only place where canonical meaning lives.

Canonical meaning belongs in subscription protocol artifacts and certification
bundle fields. Diagnostics may explain them, filter them, or render richer
reports according to policy.

### 7. Offline Sufficiency Is The Proof Bar

Milestone 16 is complete only if an auditor can receive emitted canonical
bundles and declared comparison rules and decide whether the workload passed
without:

- live host state
- callback registries
- open signal observers
- current truth runtime handles
- process-local replay memory
- ad hoc logs
- debugger inspection

Live execution may produce bundles. It must not be required to interpret them.

### 8. Performance Is Carried By Plans, Profiles, And Lifetimes

Milestone 16 must not treat performance as a bag of counters attached after
bundle assembly.

Every expensive certification operation must consume a precomputed,
proof-bearing plan or profile:

- `SubscriptionBundleAssemblyPlan` or equivalent for selecting retained
  artifacts, canonical record groups, ordering keys, field-state expectations,
  and diagnostics policy before bundle assembly begins
- `SubscriptionBundleAssemblyCostProfile` or equivalent for selecting sparse,
  bounded-window, dense-workload, or rejected-over-budget assembly posture
- `SubscriptionComparisonPlan<Relationship>` for selecting compared record
  groups, ignored diagnostics detail, divergence axes, expected rejection
  boundary, and counter contract before comparison begins
- `SubscriptionOfflineAuditPlan` for selecting bundle fields and comparison
  reports that the offline audit may inspect
- `SubscriptionResidueInspectionPlan` for selecting preview-scope indexes
  rather than scanning all retained subscriptions
- `SubscriptionCertificationScratch` or equivalent arena/scratch lifetime for
  canonicalization buffers, comparison worksets, sorted record ids, and
  diagnostics materialization buffers

These plans and profiles must be constructed before the operation they govern.
Execution may consume them, but it may not rediscover strategy, density,
diagnostics richness, or artifact selection by inspecting raw bundle contents
inside the hot path.

The performance architecture must make these transitions explicit:

- raw retained artifacts -> indexed source artifact view
- indexed source artifact view -> assembly plan
- assembly plan + cost profile + scratch scope -> draft bundle
- draft bundle -> sealed bundle
- sealed bundle + relationship plan -> comparison report
- sealed bundle + audit plan -> offline audit report

If an implementation directly builds a certification bundle by iterating
whatever records happen to be reachable from a host object graph, it violates
the milestone even if the emitted counters look acceptable for the first
fixture.

### 9. Certification Uses Indexed Views, Not Broad Discovery

Milestone 16 must define indexed certification views over retained artifacts
before bundle assembly and offline audit.

Required indexed views:

- source artifact index keyed by retained Milestone 14 and 15 artifact identity
- subscription identity index keyed by admitted subscription identity
- delivery window index keyed by subscription identity and delivery epoch
- consumer fanout index keyed by fanout identity and consumer contract identity
- continuation index keyed by source truth identity and branch scope
- checkpoint index keyed by subscription checkpoint identity
- preview residue index keyed by preview scope identity
- failure index keyed by failure precedence stage and failure boundary
- diagnostics index keyed by diagnostics policy and retained detail class

Bundle assembly, comparison, residue inspection, and offline audit must consume
these indexes. They must not linearly scan unrelated retained subscription
history, all active subscriptions, all consumers, all preview scopes, or all
diagnostics records to answer a scoped certification question.

The indexes are derived state. They must be rebuildable from retained
authoritative bridge artifacts and must not become new subscription authority.

## Complexity Contracts

Milestone 16 must name and prove boundedness for:

- subscription bundle assembly
- bundle comparison
- failure localization
- diagnostics materialization
- reference workload bundle emission
- replay reconstruction from retained artifacts
- residue inspection
- family-aware strategy-lowering provenance checks
- source artifact indexing
- assembly-plan construction
- comparison-plan execution
- offline-audit-plan execution
- certification scratch allocation and reuse

The named boundary contracts should be stated in terms of:

- `a`: admitted subscription artifact count in the bundle
- `d`: delivery record count included in the certification window
- `c`: consumer contract count included in the bundle
- `f`: fanout group count included in the bundle
- `k`: checkpoint or resume boundary count
- `r`: replay record count
- `e`: identity evolution or continuation event count
- `p`: preview scope count
- `x`: comparison lane count
- `m`: emitted failure count
- `i`: source artifact index entries relevant to the bundle
- `q`: queried audit field count
- `s`: scratch buffer capacity admitted by the cost profile

Representative complexity targets:

- source artifact indexing: `O(i)` for the retained artifact set admitted into
  the certification window, not total bridge history
- assembly-plan construction: `O(a + d + c + f + k + r + e + p)` over indexed
  relevant records, with no unrelated family or global subscription scans
- bundle assembly: `O(a + d + c + f + k + r + e + p)` for the emitted certification window
- bundle comparison: `O(x * compared_digest_count)` after bundle digests are assembled, with no replay of live host behavior during comparison and no re-canonicalization of sealed bundle records
- failure localization: `O(m)` against typed failure records, not log scans
- diagnostics materialization: `O(selected_detail_width)` and policy-governed, not mandatory hot-path construction
- residue inspection: `O(preview_scope_width)` against preview-scope residue indexes, never a scan of all authoritative subscriptions
- strategy-lowering provenance check: `O(a)` against retained lowering records, never rediscovery through signal internals
- replay reconstruction: `O(a + d + k + r + e + p)` against retained bridge artifacts, not live host registries
- offline audit: `O(q)` against sealed bundle indexes and comparison reports,
  not a full bundle walk unless the audit plan explicitly requests full-bundle
  completeness verification

Required density postures:

- `SparseCertificationWindow`: ordinary bundle assembly over a small set of
  subscription identities and delivery windows
- `BoundedWorkloadWindow`: the 128-product reference workload window with
  declared maximum product, component, subscription, delivery, and lane counts
- `DenseCertificationRebuild`: explicit dense posture for full workload bundle
  reconstruction when sparse indexing would cost more than rebuilding the
  certification view
- `RejectedOverBudgetCertification`: typed rejection before rich diagnostics,
  broad comparison, or dense reconstruction when declared limits are exceeded

Minimum counters:

- `subscription_certification_bundle_count`
- `subscription_bundle_assembly_count`
- `subscription_bundle_assembly_plan_count`
- `subscription_bundle_cost_profile_count`
- `subscription_bundle_comparison_count`
- `subscription_bundle_comparison_mismatch_count`
- `subscription_comparison_plan_count`
- `subscription_offline_audit_plan_count`
- `subscription_equivalence_lane_count`
- `subscription_divergence_lane_count`
- `subscription_failure_lane_count`
- `subscription_replay_lane_count`
- `subscription_source_artifact_index_entry_count`
- `subscription_source_artifact_index_scan_count`
- `subscription_global_history_scan_count`
- `subscription_global_subscription_scan_count`
- `subscription_dense_rebuild_count`
- `subscription_over_budget_rejection_count`
- `subscription_scratch_allocation_count`
- `subscription_scratch_reuse_count`
- `subscription_failure_localization_count`
- `subscription_failure_boundary_mismatch_count`
- `subscription_diagnostics_materialization_count`
- `subscription_diagnostics_influence_count`
- `subscription_reference_workload_bundle_count`
- `subscription_reference_workload_lane_count`
- `subscription_residue_inspection_count`
- `subscription_residue_violation_count`
- `subscription_strategy_lowering_provenance_check_count`
- `subscription_offline_bundle_field_count`
- `subscription_host_log_dependency_count`
- `subscription_live_state_dependency_count`
- `subscription_allocation_count`
- `subscription_clone_count`

Counters that must be zero in representative control lanes:

- `subscription_diagnostics_influence_count`
- `subscription_host_log_dependency_count`
- `subscription_live_state_dependency_count`
- full-registry scan counters inherited from Milestones 14 and 15
- `subscription_global_history_scan_count`
- `subscription_global_subscription_scan_count`
- `subscription_over_budget_rejection_count`
- global authoritative subscription scans during preview residue inspection
- rich diagnostics hot-path materialization counters unless explicitly admitted

Minimum representative exact counter assertions:

| Lane | Counter | Required value |
| --- | --- | --- |
| semantic-equivalence control | `subscription_bundle_comparison_count` | `1` per planned comparison |
| semantic-equivalence control | `subscription_bundle_comparison_mismatch_count` | `0` |
| diagnostics-only variation | `subscription_diagnostics_influence_count` | `0` |
| offline audit | `subscription_host_log_dependency_count` | `0` |
| offline audit | `subscription_live_state_dependency_count` | `0` |
| preview discard | `subscription_residue_violation_count` | `0` |
| expected rejection | `subscription_failure_boundary_mismatch_count` | `0` |
| bundle insufficiency | `subscription_failure_localization_count` | `1` for the primary insufficiency |
| strategy provenance | `subscription_strategy_lowering_provenance_check_count` | equal to admitted subscription count in the bundle |
| replay equivalence | `subscription_replay_lane_count` | `1` per replay lane |
| sparse assembly control | `subscription_bundle_assembly_plan_count` | `1` |
| sparse assembly control | `subscription_bundle_cost_profile_count` | `1` |
| sparse assembly control | `subscription_global_history_scan_count` | `0` |
| sparse assembly control | `subscription_global_subscription_scan_count` | `0` |
| 128-product workload | `subscription_source_artifact_index_entry_count` | exact manifest-derived artifact count |
| 128-product workload | `subscription_dense_rebuild_count` | `0` unless the lane explicitly selects `DenseCertificationRebuild` |
| over-budget lane | `subscription_over_budget_rejection_count` | `1` |
| diagnostics-only variation | `subscription_scratch_allocation_count` | unchanged from semantic-equivalence control |

Any counter that cannot honestly have a fixed value in a representative lane
must be declared as variable with:

- the variable input that controls it
- the exact formula for the expected value
- the reason a fixed value would be dishonest

Range-only assertions are not acceptable unless the range itself is the
contract being certified and the test proves both lower and upper bounds.

## Phases

### Phase 1: Canonical Subscription Certification Bundle

Define and implement:

- the `subscription_reference_workload_manifest` shape and canonicalization
  rules before broad workload execution begins
- source artifact indexes over retained Milestone 14 and 15 artifacts
- `SubscriptionBundleAssemblyPlan`, `SubscriptionBundleAssemblyCostProfile`,
  and `SubscriptionCertificationScratch` or equivalents
- the canonical subscription certification bundle schema
- top-level and nested bundle digests
- explicit bundle field requirements for declaration, basis, lifecycle, delivery, sharing, continuation, checkpoint, replay, diagnostics, failure, residue, strategy-lowering, and counter records
- retained comparison-rule declarations
- bundle assembly from Milestone 14 and 15 artifacts
- typed bundle insufficiency failures

Phase 1 implementation guidance:

- define schema version, digest algorithm, canonical serialization, typed field
  state, and ordering rules before adding scenario-specific bundle fields
- build the source artifact index and assembly plan before any bundle records
  are materialized
- make the cost profile choose sparse, bounded workload, dense rebuild, or
  over-budget rejection before assembly begins
- make scratch allocation belong to one explicit certification lifetime rather
  than allocating per record group
- start with bundle assembly over retained Milestone 14 declaration artifacts and Milestone 15 active delivery artifacts
- keep the top-level digest as a summary, not a replacement for nested semantic records
- define omission semantics explicitly; if a field is absent because the lane did not exercise that surface, the bundle must say so structurally
- reject free-form diagnostic-only bundles as insufficient
- avoid reference-workload-specific fields in the core bundle schema

Phase 1 is complete only when original and replay lanes can emit comparable
subscription certification bundles without reading host logs or live runtime
state, and when the reference workload manifest can be canonicalized and
digested independently of workload execution. It is not complete if bundle
assembly can reach into global retained history or allocate rich diagnostics
payloads without an admitted assembly plan and cost profile.

### Phase 2: Bundle Comparison And Subscription Failure Taxonomy

Implement:

- first-class bundle comparison rules
- equality and inequality comparison outcomes
- expected and unexpected typed rejection outcomes
- diagnostics-only difference detection
- residue mismatch detection
- replay mismatch detection
- counter-contract violation detection
- subscription-specific failure taxonomy additions
- replay-stable failure localization artifacts

Phase 2 implementation guidance:

- define comparison as a bridge certification API, not as scattered test helper logic
- make comparison consume sealed bundles and a comparison plan; it may not
  re-canonicalize bundle records or rediscover compared fields from raw bundle
  traversal
- make each comparison retain the intended semantic relationship between lanes
- keep parent-runtime failures as causes while reporting subscription-boundary failure classes at the bridge surface
- implement the failure precedence ladder as data or typed ordering consumed by
  the localization path, not as ordering hidden in test assertions
- ensure diagnostics richness changes appear as diagnostics-only differences unless the diagnostics policy illegally altered canonical meaning
- include negative tests where intentionally divergent declarations, bases, delivery families, continuations, or preview outcomes must compare unequal
- include one multi-failure lane that injects at least three failures from
  different precedence stages and proves the primary failure plus suppressed
  causes are stable under replay

Phase 2 is complete only when equivalent, divergent, rejected, replay-mismatched,
residue-mismatched, diagnostics-only, and multi-failure precedence lanes
produce distinct typed comparison reports.

### Phase 3: Subscription Diagnostics Entry Point And Offline Audit

Ship:

- subscription-specific extensions to the public bridge diagnostics entrypoint
- offline audit reports over canonical subscription bundles
- bundle completeness reports
- diagnostics-tier invariance tests
- compile-fail or equivalent privacy boundaries for bundle and comparison witness construction
- exact counter assertions for bundle assembly, comparison, diagnostics materialization, failure localization, residue inspection, and replay reconstruction

Phase 3 implementation guidance:

- expose diagnostics as interpretation of retained artifacts, not as the authority for those artifacts
- make offline audit consume bundles and comparison rules only
- include a positive completeness report and typed incompleteness report
- prove external callers cannot synthesize passing bundle, comparison, failure, or residue witnesses directly
- make counter snapshots part of the bundle and audit surface

Phase 3 is complete only when an auditor can diagnose subscription equivalence,
divergence, typed failure, replay drift, diagnostics-only variation, and residue
outcomes from canonical bundles alone.

### Phase 4: End-to-End Subscription Reference Workload

Build the Rust-only reference workload extension.

Minimum workload lanes:

- authoritative live subscription lane
- historical-basis subscription replay lane
- branch-local subscription isolation lane
- shared-consumer fanout lane
- separate-but-equivalent subscription lane
- restart and resume lane
- identity-evolution continuation lane
- preview discard zero-residue lane
- preview promotion-boundary lane
- hostile adapter variation lane
- diagnostics-tier variation lane
- typed rejection lanes for basis drift, illegal sharing, stale checkpoint, denied continuation, preview residue, and replay mismatch

The workload must exercise at least:

- one detail-oriented admitted declaration family
- one collection-oriented admitted declaration family
- two distinct `worth-signal` strategy lowerings
- one active delivery family that emits canonical member records
- one shared fanout path
- one continuation path
- one replay path
- one preview discard path
- one preview promotion path

Phase 4 implementation guidance:

- prefer extending the existing pricing-shock reference workload lineage from Milestone 13 so the bridge certification story stays coherent
- keep pricing/product semantics inside harness fixtures
- make all workload pass/fail decisions flow through canonical bundle comparison rather than scenario-local assertions
- include at least one workload perturbation that should preserve canonical meaning, one that should change canonical meaning, and one that should fail explicitly
- ensure the final workload bundle is one coherent nested artifact rather than unrelated lane-local blobs

Phase 4 is complete only when the reference workload can prove suites 35 through
37 from canonical subscription bundles and comparison reports.

## Must Ship

- canonical subscription certification bundle schema
- nested canonical digests for subscription declaration, registry, basis, lifecycle, delivery, sharing, continuation, consumer contract, checkpoint, replay, diagnostics, failure, residue, strategy-lowering, and counters
- typed bundle comparison rules and comparison reports
- subscription-specific bridge failure taxonomy additions
- subscription-specific diagnostics entrypoint extensions
- bundle assembly plans, comparison plans, offline audit plans, cost profiles,
  density postures, indexed certification views, and scratch-lifetime
  management
- offline audit reports over emitted subscription bundles
- bundle completeness and insufficiency reports
- exact counter contracts for assembly, comparison, localization, diagnostics, replay, residue, and strategy-lowering provenance
- Rust-only end-to-end subscription reference workload extension
- reference workload coverage for authoritative, historical, branch-local, preview, shared-consumer, restart, continuation, discard, promotion, hostile adapter, diagnostics-tier, and typed rejection lanes
- certification satisfying suites 35 through 37 in [test-requirements.md](./test-requirements.md)
- compile-fail or equivalent external-boundary coverage preventing external construction of canonical bundles, passing comparison witnesses, typed failure witnesses, residue proofs, or offline audit success markers

## Must Preserve

- truth authority remains in `worth-relational`
- observation and derived execution authority remains in `worth-signal`
- the bridge remains the subscription protocol and certification boundary, not a query engine or observation runtime
- Milestone 14 declaration and admission proofs remain prior proofs
- Milestone 15 active delivery, fanout, continuation, checkpoint, replay, and preview artifacts remain prior proofs
- certification bundles are derived from retained bridge artifacts and do not become new semantic authority
- diagnostics explain canonical artifacts but do not define canonical subscription meaning
- branch-local, historical, preview, and authoritative subscription bases remain explicit and non-interchangeable
- preview discard and promotion remain positive artifact stories, not absence stories
- family-aware strategy lowering remains visible in bundles and diagnostics
- replay and offline audit do not depend on host-local callbacks, logs, process memory, live runtime handles, or Query-only interpretation
- reference workload domain semantics remain harness-owned fixtures

## Acceptance Evidence

Milestone 16 is complete only when the bridge harness can prove all of the following:

- original execution, replay, restart, hostile adapter variation, and diagnostics-tier variation emit comparable canonical subscription bundles
- bundle assembly consumes indexed retained artifacts, an assembly plan, an
  admitted cost profile, and an explicit scratch lifetime
- comparison consumes sealed bundles and a comparison plan without
  re-canonicalizing records or replaying host behavior
- offline audit consumes an audit plan and sealed bundle indexes rather than
  scanning raw bundle contents or live runtime state
- semantically equivalent subscription lanes compare equal on canonical subscription meaning
- intentionally different declarations, bases, delivery families, continuation outcomes, preview outcomes, or strategy lowerings compare unequal on the declared semantic axis
- rejected lanes fail at expected subscription boundaries with typed bridge-native failure artifacts
- unexpected failure boundaries are reported distinctly from expected rejection
- diagnostics richness changes retained detail only and does not change canonical subscription meaning
- offline audit can diagnose bundle equivalence, divergence, rejection, replay mismatch, residue mismatch, diagnostics-only variation, and counter violations from emitted bundles alone
- bundle insufficiency is itself a typed failure rather than a silent skipped assertion
- branch-local subscription state does not leak into authoritative subscription bundles
- historical-basis subscription replay uses explicit retained truth basis artifacts and never falls back to latest truth
- preview discard proves zero authoritative truth residue, zero bridge-visible subscription residue, zero delivery residue, and zero checkpoint residue through positive residue artifacts and counters
- preview promotion produces explicit promotion-boundary records and does not mutate preview identity into authoritative identity in place
- subscription-specific failure taxonomy distinguishes declaration, registry, basis, lifecycle, consumer, sharing, delivery, continuation, checkpoint, replay, preview, diagnostics, strategy-lowering, and bundle-sufficiency failures
- at least two admitted declaration families remain distinguishable through bundle records and strategy-lowering provenance
- the reference workload proves long-lived active subscriptions across authoritative, historical, branch-local, preview, shared-consumer, restart, continuation, discard, and promotion pressure
- exact Milestone 16 counters match declared values for representative control, hostile, replay, diagnostics-tier, rejection, and offline audit lanes
- host-log and live-state dependency counters remain zero in representative certification lanes
- certification suites 35 through 37 pass with canonical machine-checkable bundles

## Compile-Time Enforcement Obligations

Milestone 16 must add compile-fail or equivalent external-boundary tests proving
external crates cannot:

- construct or mutate `SubscriptionCertificationBundleSealed` or equivalent
  sealed bundle types
- construct `SubscriptionBundleAssemblyPlan`,
  `SubscriptionBundleAssemblyCostProfile`, `SubscriptionOfflineAuditPlan`,
  `SubscriptionResidueInspectionPlan`, or `SubscriptionCertificationScratch`
  directly
- call bundle assembly without an admitted assembly plan, cost profile, and
  scratch scope
- call comparison on draft bundles or unindexed bundle records
- construct `SubscriptionComparisonPlan<Relationship>` with a relationship
  not admitted by the bridge comparison taxonomy
- construct canonical subscription certification bundles directly
- construct passing bundle comparison witnesses directly
- construct bundle completeness reports without the offline audit path
- synthesize subscription failure taxonomy variants that claim bridge-owned proof
- synthesize residue proofs for preview discard or promotion
- mark diagnostics-only differences without running comparison logic
- convert a rejected lane into an equivalent lane
- substitute a top-level bundle digest for required nested semantic records
- construct strategy-lowering provenance without retained Milestone 14 or 15 lowering records
- construct offline audit success without emitted bundles and comparison rules
- mutate emitted bundle fields after digest computation
- pass host logs or live runtime handles where retained canonical artifacts are required
- pass an unsealed bundle where a sealed bundle is required for comparison
- pass a diagnostics bundle where a semantic subscription bundle is required
- compare bundles with different schema versions or digest algorithms without
  explicit migration or typed incompatibility handling
- mark a field as `NotExercised` after the source artifact index proves the
  surface was exercised
- construct expected-rejection success without naming the expected failure
  boundary and precedence stage
- construct residue-absence success without positive residue records for all
  required scopes
- construct offline audit success while host-log or live-state dependency
  counters are nonzero
- select `DenseCertificationRebuild` or `RejectedOverBudgetCertification`
  after assembly has already begun
- materialize rich diagnostics payloads through a semantic bundle assembly type
  that did not admit rich diagnostics in its cost profile

These tests prove that Milestone 16 certification is mechanically owned by the
bridge rather than by host convention.

## First-Ship Certification Matrix

| Lane | Families | Required proof |
| --- | --- | --- |
| authoritative live subscription | `DetailExact`, `CollectionMembership` | original and replay bundles compare equal on subscription and delivery meaning |
| historical basis replay | at least one family | retained historical basis is explicit and latest-truth fallback remains zero |
| branch-local isolation | both families | branch-local subscription bundle differs from authoritative where expected and cannot leak across branches |
| shared fanout equivalence | both families | shared and separate-but-equivalent subscriptions compare equal on canonical meaning |
| incompatible sharing rejection | at least one family | illegal sharing fails at consumer or sharing boundary before delivery |
| restart and resume | both families | subscription checkpoint and replay bundle preserve delivery meaning |
| stale checkpoint rejection | at least one family | rejected at checkpoint/resume boundary with typed failure |
| continuation after identity evolution | both families where admitted | continuation records preserve or split subscription meaning deterministically |
| denied continuation | at least one family | authority-denied or ambiguous continuation fails before misleading delivery records |
| preview discard | both families | zero residue across authority, bridge, delivery, checkpoint, and signal-visible scopes |
| preview promotion | at least one family | promotion creates explicit authoritative-boundary records |
| diagnostics richness | both families | rich and minimal diagnostics differ only in diagnostics records |
| hostile adapter variation | both families | adapter variation preserves canonical bundle meaning where contracts are equivalent |
| strategy-lowering provenance | both families | bundle proves distinct signal-strategy lowerings without rediscovery |
| bundle insufficiency | at least one family | missing required fields fail as typed bundle insufficiency |
| offline audit | mixed workload | audit succeeds or fails from emitted bundles only, with host-log/live-state counters at zero |
| multi-failure precedence | at least one family | injected basis drift plus stale checkpoint plus diagnostics influence reports the highest-precedence boundary as primary and lower-precedence failures as suppressed causes |
| schema incompatibility | at least one family | mismatched bundle schema or digest algorithm fails before semantic comparison |
| canonical ordering hostility | both families | host adapter insertion order changes do not change canonical bundle digest |
| sparse certification window | both families | assembly uses indexed relevant artifacts and zero global history or subscription scans |
| dense rebuild posture | mixed workload | dense rebuild is selected before assembly and remains diagnosable through cost profile and counters |
| over-budget certification rejection | at least one family | rejected before rich diagnostics, broad comparison, or dense reconstruction begins |
| scratch lifecycle reuse | mixed workload | repeated bundle assembly reuses admitted scratch buffers and does not allocate per record group |

Every matrix lane must emit:

- `certification_bundle_digest`
- relevant nested subscription digests
- comparison report
- typed failure digest for rejected lanes
- exact counter snapshot
- diagnostics digest
- bundle completeness report

No lane may pass by asserting that a digest is merely present or non-empty.

## Architectural Notes

Milestone 16 should extend the bridge crate with subdomains such as:

- `subscription/certification/bundle.rs`
- `subscription/certification/digest.rs`
- `subscription/certification/field_state.rs`
- `subscription/certification/comparison.rs`
- `subscription/certification/relationship.rs`
- `subscription/certification/offline_audit.rs`
- `subscription/certification/failure_taxonomy.rs`
- `subscription/certification/failure_precedence.rs`
- `subscription/certification/diagnostics.rs`
- `subscription/certification/residue.rs`
- `subscription/certification/counters.rs`
- `subscription/certification/source_artifact_index.rs`
- `subscription/certification/assembly_plan.rs`
- `subscription/certification/cost_profile.rs`
- `subscription/certification/scratch.rs`
- `subscription/certification/audit_plan.rs`
- `subscription/certification/workload_manifest.rs`
- `subscription/certification/reference_workload.rs`
- `subscription/certification/workload_lanes.rs`

Recommended implementation order:

- first land `certification/workload_manifest.rs`,
  `certification/source_artifact_index.rs`, `certification/field_state.rs`,
  `certification/digest.rs`, `certification/cost_profile.rs`,
  `certification/assembly_plan.rs`, `certification/scratch.rs`, and
  `certification/bundle.rs`
- then land `certification/relationship.rs`,
  `certification/comparison.rs`, `certification/failure_taxonomy.rs`, and
  `certification/failure_precedence.rs`
- then land `certification/diagnostics.rs`,
  `certification/audit_plan.rs`, `certification/offline_audit.rs`,
  `certification/residue.rs`, and `certification/counters.rs`
- finally land `certification/reference_workload.rs`,
  `certification/workload_lanes.rs`, and suite 35 through 37 harness coverage

Expected facade growth should look more like:

- `declare_subscription_reference_workload_manifest(...)`
- `build_subscription_certification_source_index(...)`
- `plan_subscription_certification_bundle(...)`
- `admit_subscription_certification_cost_profile(...)`
- `assemble_subscription_certification_bundle(...)`
- `seal_subscription_certification_bundle(...)`
- `compare_subscription_certification_bundles(...)`
- `plan_subscription_offline_audit(...)`
- `audit_subscription_certification_bundle_offline(...)`
- `inspect_subscription_certification(...)`
- `run_subscription_reference_workload(...)`

and not like:

- host logs passed as proof
- generic `debug_subscription(...)` surfaces that hide bundle semantics
- a Query-only certification path
- raw callback or signal observer identity used in comparison
- mutable bundle builders exposed after digest computation
- reference workload assertions that bypass canonical bundle comparison
- bundle assembly APIs that hide global scans, dense rebuilds, or rich
  diagnostics materialization behind cheap-looking method names
- flat `subscription_certification.rs` modules that mix bundle schema,
  comparison, failure taxonomy, diagnostics, counters, and workload fixtures
  into one responsibility
- test `support` modules that hide the fixture contract or construct invalid
  proof states directly

Temporary allowances during bring-up:

- the first offline audit surface may be narrow as long as it consumes canonical bundles and not live host state
- the reference workload may initially use the same pricing fixture family as Milestone 13 if subscription-specific lanes are clearly separated
- permanent persistence may remain a retained-artifact abstraction rather than final Store productization
- additional subscription declaration families may remain explicitly unsupported if two materially distinct admitted families are certified end to end

## Test And Harness Model

The harness must define at least these scenario verbs:

- `assemble_subscription_certification_bundle(...)`
- `plan_subscription_certification_bundle(...)`
- `admit_subscription_certification_cost_profile(...)`
- `compare_subscription_bundles_for_equivalence(...)`
- `compare_subscription_bundles_for_divergence(...)`
- `compare_subscription_bundles_for_expected_rejection(...)`
- `audit_subscription_bundle_offline(...)`
- `plan_subscription_offline_audit(...)`
- `run_authoritative_subscription_workload(...)`
- `run_historical_subscription_replay_workload(...)`
- `run_branch_local_subscription_workload(...)`
- `run_preview_subscription_discard_workload(...)`
- `run_preview_subscription_promotion_workload(...)`
- `run_shared_fanout_subscription_workload(...)`
- `run_subscription_restart_resume_workload(...)`
- `inject_subscription_failure(...)`

The harness must vary:

- declaration family
- truth basis class
- delivery family
- consumer contract shape
- shared versus separate subscription topology
- branch identity
- historical basis
- continuation outcome
- checkpoint boundary
- restart boundary
- preview discard versus promotion
- diagnostics tier
- host adapter shape
- replay path
- admitted versus rejected path

Minimum certification outputs:

- `certification_bundle_digest`
- `subscription_digest`
- `subscription_registry_digest`
- `subscription_basis_digest`
- `subscription_lifecycle_digest`
- `subscription_delivery_digest`
- `subscription_share_digest`
- `subscription_continuation_digest`
- `consumer_contract_digest`
- `checkpoint_digest`
- `routing_digest`
- `replay_digest`
- `diagnostics_digest`
- `failure_digest`
- `residue_digest`
- `strategy_lowering_digest`
- `bundle_comparison_report`
- `bundle_completeness_report`
- `counter_snapshot`
- `assembly_plan_digest`
- `cost_profile_digest`
- `source_artifact_index_digest`
- `scratch_lifecycle_digest`

## Anti-Patterns Explicitly Rejected

- treating host logs as certification artifacts
- comparing bundles only to themselves
- testing only happy-path bundle presence
- using one generic subscription digest that erases family, basis, delivery, continuation, and replay distinctions
- certifying one subscription family and claiming family-wide correctness
- letting diagnostics richness change canonical meaning
- proving preview discard by object drop or absence of callbacks
- reconstructing replay from host callback registries
- letting Query become the only subscription certification layer
- re-running live host behavior during offline audit
- hiding strategy-lowering provenance inside signal internals
- collapsing subscription failures into generic stream or source errors
- skipping bundle insufficiency failures when required fields are absent
- using `Option<T>` or empty vectors as the primary representation of missing
  bundle fields instead of typed field state
- hashing debug output, JSON object insertion order, map iteration order, or
  non-canonical serialization as a digest input
- letting a diagnostics-rich lane allocate or clone rich artifact payloads on
  the ordinary semantic bundle assembly path
- reporting the first encountered failure instead of the highest-precedence
  violated subscription boundary
- allowing reference workload fixture setup code to construct proof states that
  ordinary external hosts cannot construct
- assembling bundles by walking raw host object graphs or all retained bridge
  history instead of indexed retained artifacts
- selecting sparse, dense, or over-budget posture after bundle assembly has
  already started
- re-canonicalizing sealed bundle records during comparison
- using general heap allocation per record group when a certification scratch
  lifetime could own the buffers
- using one uniform assembly path that hides sparse and dense cost differences

## Sequencing Notes

Milestone 16 builds directly on:

- Milestone 13 end-to-end causality, failure taxonomy, diagnostics entrypoint, and reference workload discipline
- Milestone 14 subscription declaration-family, admission, basis, strategy-lowering, lifecycle, diagnostics, and replay artifacts
- Milestone 15 active subscription delivery, fanout, continuation, checkpoint, resume, replay, preview residue, and promotion artifacts

It closes the subscription arc that began in Milestone 14:

- Milestone 14 says what a subscription is
- Milestone 15 says how an active subscription behaves
- Milestone 16 proves the whole subscription family story can be certified offline

It also belongs before broad Query productization because Query should compile
into already-certifiable bridge subscription primitives rather than becoming
the first place where subscription correctness is proven.

## Self-Check

- This solves a real structural problem: Milestones 14 and 15 create subscription protocol artifacts, but without Milestone 16 there is no end-to-end offline certification story for long-lived subscriptions.
- The adversarial constraint is precise and load-bearing: restart, replay, branch divergence, historical basis, preview discard, promotion, shared fanout, continuation, hostile adapters, diagnostics-tier variation, and typed rejection are the failure modes that would break naive certification.
- Authority boundaries are preserved: truth owns truth, signal owns derived execution, and the bridge owns subscription protocol artifacts, diagnostics, comparison, and certification.
- The spec defines proof obligations, not chores: bundle equality, bundle inequality, expected rejection, residue proof, failure localization, offline audit, exact counters, and compile-time witness denial are all machine-checkable.
- A competent engineer can map this into honest modules, types, facade methods, harness verbs, counters, compile-fail tests, and certification suites.
- The milestone belongs in sequence: it consumes Milestone 14 and 15 subscription artifacts and provides the certification substrate Query and Store-facing product work should rely on.

## Closeout Standard

Milestone 16 is complete only when the bridge can emit canonical subscription
certification bundles, compare them under explicit semantic relationships,
localize subscription-specific failures, diagnose bundles offline through the
public diagnostics entrypoint, and prove a Rust-only end-to-end subscription
reference workload across authoritative, historical, branch-local, preview,
shared-consumer, restart, continuation, discard, promotion, hostile adapter,
diagnostics-tier, and typed rejection lanes.

If subscription proof still depends on host logs, if diagnostics become
semantic authority, if preview residue is proven by absence rather than positive
artifacts, if strategy-lowering provenance is hidden, if intentionally different
subscriptions compare equal, if equivalent subscriptions drift under replay, or
if offline audit needs live runtime state, Milestone 16 is not complete.
