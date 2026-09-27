# WORTH Runtime Bridge Test Requirements

## Scope

This document defines the certification-grade bridge test requirements for:

- Milestone 6
- Milestone 7
- Milestone 8
- Milestone 9
- Milestone 10
- Milestone 11
- Milestone 12
- Milestone 12b
- Milestone 13
- Milestone 14
- Milestone 15
- Milestone 16
- Milestone 17
- Milestone 18
- Milestone 19

Milestones 1 through 5 already have their own acceptance and closeout proof
surfaces. This document starts at Milestone 6 because the bridge is now moving
from geometry-kernel-critical routing foundations into broader protocol,
policy, merge, preview, and writeback surfaces that need a much stronger
crate-level certification bar.

## Purpose

The bridge cannot be treated as shipped merely because individual milestone
features appear to work in direct tests.

From Milestone 6 onward, the bridge is making claims about:

- protocol correctness under restart, replay, and multi-consumer pressure
- host-agnostic source contracts
- structural remapping without identity fusion
- merge-bearing history interpretation
- speculative branch coordination
- cross-runtime policy propagation
- bridge-mediated writeback
- end-to-end causality and bridge-native certification
- temporal-basis binding after `worth-signal` owns time
- async/resource completion causality after `worth-signal` owns async lifecycle
- mixed truth/time/async subscription delivery, restart, replay, and offline
  certification

Those are all adversarial surfaces. They need certification tests, not just
behavior checks.

Cross-runtime note:

- [../worth-query/runtime-authoritative-mutation-evidence-plan.md](../worth-query/runtime-authoritative-mutation-evidence-plan.md)
  is the shared hardening gate for authority-mutation evidence, causality, and
  provenance carry-forward. Where bridge tests cover writeback, causality,
  continuity, or replay-safe provenance, they are part of that same end-to-end
  contract, not a separate story.

## Global Adversarial Constraint

The bridge test suite from Milestone 6 onward must prove the following:

> Under restart, replay, host-adapter variation, branch divergence, merge-like
> history pressure, speculative execution, policy changes, hostile diagnostics
> tiers, temporal clock advances, async/resource completion races, and
> writeback failure injection, the bridge must preserve canonical truth
> interpretation, canonical routing meaning, replay-safe artifacts, typed
> failures, explicit authority boundaries, and machine-checkable diagnostics
> without allowing host-local glue, scheduler timing, transport completion
> order, ambient process clocks, or convenience policy to redefine canonical
> semantics.

If a bridge surface works only under one adapter, one scheduling mode, one
consumer shape, one diagnostics tier, or one happy-path history shape, it is
not certified.

## Meta-Rules

These tests are all certification tests. They must:

- emit canonical machine-checkable artifacts, not "logs looked good"
- compare canonical digests across equivalent runs
- prove typed failure localization for rejection paths
- prove diagnostics richness changes retained detail, not semantic truth
- prove replay from canonical bridge artifacts rather than ambient host state
- verify counter contracts where the milestone makes boundedness or scale claims
- prove that any lowered bridge taxonomy remains losslessly traceable back to
  the authoritative truth/runtime taxonomy it consumes
- prove replay parity over full canonical result bundles when a milestone claims
  replay-safe explanations, routing, continuity, remap, or denial
- prove discovery work boundedness when a milestone claims bounded lowering or
  bounded execution cost, not just post-lowering work boundedness

These requirements are mandatory, not advisory. A test does not satisfy this
document merely because it exercises the right API shape or emits a non-empty
bundle. Certification requires mechanical comparison across independently
produced runs with declared semantic relationships.

### Global Certification Shape

Every named certification suite must define at least these lanes unless the
suite explicitly states a narrower reason:

- `control_lane` â€” the canonical no-failure, no-hostility baseline
- `hostile_lane` â€” the adversarial variation being certified
- `replay_lane` â€” replay, resume, restart, or retained-artifact reproduction of
  the same semantic workload

If a suite is about explicit rejection, the hostile lane may terminate in a
typed failure rather than a completed result, but the suite must still compare
that failure against a successful or otherwise equivalent control basis.

### Mandatory Assertion Classes

Every named certification suite must include all applicable assertion classes:

- equality assertions for semantically equivalent lanes
- inequality assertions for intentionally divergent semantic lanes
- typed-failure assertions for rejected lanes
- zero-or-absence assertions for forbidden residue, forbidden fallback, or
  forbidden diagnostics influence

A test that only checks one lane in isolation, only checks that a bundle is
present, or only checks that a digest is non-empty does not satisfy this
document.

### Certification Bundle Rules

Certification bundles must use fixed, milestone-appropriate fields rather than
free-form "debug info". A suite may emit additional detail, but it must emit a
stable canonical bundle shape for its scope.

At minimum, every certification bundle must make it possible to answer all of
these questions mechanically:

- did two semantically equivalent lanes produce the same canonical truth?
- did an intentionally different semantic lane produce different canonical
  truth?
- did a rejected lane fail at the correct protocol or planning boundary?
- did diagnostics richness change retained detail only?
- did counters remain within the contract claimed by the milestone?

If a bundle cannot answer those questions without reading host logs or runtime
internals, the bundle is insufficient.

### Mutation-Sensitivity Rule

Every named certification suite must include at least one perturbation from each
applicable class:

- a perturbation that changes pacing, scheduling, retention timing, or
  diagnostics richness without changing canonical meaning
- a perturbation that changes canonical meaning and must therefore change at
  least one declared digest or typed report
- a perturbation that must fail explicitly before semantic drift occurs

This rule exists to prevent dead bundles and ornamental assertions. If a suite
never proves what must stay the same, what must change, and what must fail, it
has not certified its surface.

### Counter Assertion Rule

Whenever a milestone claims boundedness, scale-path correctness, replay safety,
resource discipline, or pacing discipline, the suite must assert exact counter
values for the representative scenario, including counters that must remain
zero.

Presence-only counter checks do not satisfy this document. Range checks are
acceptable only when the suite explicitly proves why the counter is
intentionally variable and why that variability is itself part of the contract.

### Offline Sufficiency Rule

Certification should be possible from the canonical bundle alone. Suites may use
live execution to produce the bundle, but pass/fail analysis must not depend on
ambient host state, ad hoc log inspection, or debugger-only knowledge.

Milestone 13 makes this requirement explicit for the full matrix, but every
Milestone 6+ suite should be written so that an auditor can evaluate the
certification result from the emitted bundle and declared comparison rules
alone.

At minimum, certification bundles should emit digests or structured reports for:

- `stream_digest`
- `window_digest`
- `checkpoint_digest`
- `routing_digest`
- `truth_view_digest`
- `consumer_contract_digest`
- `subscription_digest`
- `subscription_registry_digest`
- `subscription_basis_digest`
- `subscription_lifecycle_digest`
- `subscription_continuation_digest`
- `subscription_delivery_digest`
- `subscription_share_digest`
- `policy_digest`
- `replay_digest`
- `diagnostics_digest`
- `failure_digest`
- `counter_snapshot`

Not every test uses every artifact, but every test should emit the canonical
bundle that matches its scope.

### Anti-Fake-Test Rule

The following do not count as certification:

- asserting that a run "completed successfully"
- asserting that a digest or report field is merely present or non-empty
- comparing a value only to itself from the same run
- treating log text as the primary proof artifact
- validating only happy-path success without an adversarial lane
- validating only a failure lane without a control or replay comparison basis

A certification test must compare independently produced artifacts under a
declared semantic relationship. If the relationship is not explicit, the test is
not certification-grade.

## Milestone 6 Named Certification Suites

### 1. Change Stream Checkpoint Fracture Equivalence Test

Purpose

Prove that checkpoint, resume, and replay remain semantically exact under
partial progress, restart, and stream truncation pressure.

Scenario

- run one canonical change stream with bursty commit windows
- consume it through the bridge with checkpoint publication at several
  boundaries
- inject restart after partial batch delivery
- resume from canonical checkpoint tokens
- replay from canonical bridge records
- compare against a no-failure control run

Must verify

- resumed consumption preserves routing semantics exactly
- checkpoint meaning is tied to canonical acknowledged members, not latest seen
- truncation and incompatibility fail explicitly and typed
- replay from checkpoint records matches original admitted delivery windows
- equivalent resumed and control lanes compare equal on canonical stream and
  routing digests
- stale, truncated, or incompatible checkpoint lanes compare unequal and fail at
  the declared protocol boundary

Required verification output

- `stream_digest`
- `checkpoint_digest`
- `resume_matrix`
- `replay_digest`
- `protocol_failure_digest`

Pass condition

Equivalent runs converge to identical canonical stream and routing truth; bad
checkpoint resumes fail explicitly.

### 2. Multi-Consumer Coalescing Parity Test

Purpose

Prove that different admitted consumer shapes can consume the same canonical
stream with different pacing and coalescing policies without changing stream
meaning.

Scenario

- feed one canonical stream into at least:
  - a routing consumer
  - a replay/audit consumer
- run one lane with narrow windows and no coalescing
- run one lane with wider legal coalescing windows
- vary consumer pacing and restart points

Must verify

- canonical member interpretation stays identical across consumers
- coalesced windows remain reconstructable into canonical member truth
- duplicate observation handling stays contract-correct
- diagnostics richness does not create a third stream meaning
- pacing-only and coalescing-only perturbations preserve canonical member,
  routing, and replay truth while allowing window-shape differences
- illegal coalescing boundaries fail explicitly rather than widening silently

Required verification output

- `consumer_contract_digest`
- `window_digest`
- `coalescing_report`
- `routing_digest`
- `diagnostics_digest`

Pass condition

Consumers may differ in window shape and retained detail, but not in canonical
stream meaning.

### 3. Backpressure And Retention Anchor Hostility Test

Purpose

Prove that backpressure changes pacing only and that retention-anchor loss or
lag is localized as typed protocol failure rather than semantic drift.

Scenario

- run slow and saturated consumers against a bursty stream
- vary backpressure classes during delivery
- drop retention for older stream material in hostile timing windows
- attempt resume with stale and valid checkpoint anchors

Must verify

- backpressure never reorders or semantically merges stream members
- pressure only affects admitted pacing and optional richness
- lost retention anchors fail explicitly
- pressure and truncation diagnostics identify the exact failed boundary
- pressure-only perturbations leave canonical stream and routing digests equal to
  the control lane
- retention-anchor loss produces typed failure with zero false-success residue

Required verification output

- `pressure_report`
- `checkpoint_digest`
- `retention_anchor_matrix`
- `failure_digest`
- `counter_snapshot`

Pass condition

Pressure changes cost and pacing only; retention loss is typed and explicit.

## Milestone 7 Named Certification Suites

### 4. Multi-Host Source Parity Test

Purpose

Prove that multiple host-shaped source implementations satisfy one canonical
bridge source contract without changing read semantics.

Scenario

- implement at least two source adapters with different host shapes
- run snapshot, historical, branch, and admitted field/facet reads through both
- execute identical bridge evaluation requests against each adapter

Must verify

- identical source contracts yield identical read and routing truth
- host adapter shape does not leak into public bridge semantics
- parity holds across current, historical, and branch-local reads

Required verification output

- `truth_view_digest`
- `source_contract_digest`
- `adapter_parity_matrix`
- `routing_digest`

Pass condition

Different host adapters produce identical canonical bridge-visible results.

### 5. Source Capability Rejection Boundary Test

Purpose

Prove that unsupported or mismatched source capabilities fail during admission,
not late during evaluation.

Scenario

- request unsupported historical, branch, or facet modes
- vary source capability declarations across adapters
- attempt evaluation under incompatible source contracts

Must verify

- unsupported source modes fail before materialized read execution
- failure classes distinguish capability mismatch from host transport failure
- no adapter-specific fallback silently widens the request

Required verification output

- `source_contract_digest`
- `capability_matrix`
- `failure_digest`
- `diagnostics_digest`

Pass condition

Capability mismatches are typed, early, and non-ambiguous.

### 6. Builder Surface Swap Parity Test

Purpose

Prove that bridge setup remains explicit and comprehensible while swapping
source adapters, diagnostics policy, and registration order.

Scenario

- construct equivalent bridge setups through different explicit builder orders
- swap source adapters while keeping the same admitted source contract
- vary diagnostics policy and registration ordering

Must verify

- setup order does not change canonical source contract truth
- builder/config surfaces remain explicit rather than ambient
- lifecycle propagation is complete when adapters are swapped

Required verification output

- `source_contract_digest`
- `builder_configuration_digest`
- `setup_parity_matrix`
- `counter_snapshot`

Pass condition

Construction order and adapter swap do not alter canonical bridge meaning.

## Milestone 8 Named Certification Suites

### 7. Structural Match Ambiguity Torture Test

Purpose

Prove that structural identity assists remapping only when it is actually
honest, and that ambiguous matches fail explicitly.

Scenario

- create histories with several structurally similar but semantically distinct
  candidates
- run remapping under branch divergence and historical comparison
- inject cases with one exact structural match, many ambiguous matches, and no
  safe match

Must verify

- structural ambiguity never silently picks a winner
- exact matches remain subordinate to authoritative identity
- mismatch and ambiguity diagnostics remain replay-safe

Required verification output

- `structural_match_digest`
- `ambiguity_report`
- `remap_artifact_digest`
- `failure_digest`

Pass condition

Structural signals help only when unambiguous and never fuse identity.

### 8. Structural Reuse Without Identity Fusion Test

Purpose

Prove that structural reuse can improve remapping and comparison without
collapsing truth identity or signal identity into one fused space.

Scenario

- run replacement and restore sequences with structurally equivalent shapes
- compare remapping behavior across branches and after replay
- include same-shape/different-authority cases

Must verify

- reuse surfaces remain advisory to authoritative identity
- replay reproduces the same reuse decisions
- restore and branch-local history do not fabricate identity continuity

Required verification output

- `structural_reuse_digest`
- `identity_separation_report`
- `replay_digest`
- `diagnostics_digest`

Pass condition

Reuse is explicit and helpful, but never becomes accidental identity authority.

### 9. Branch Comparison Drift Test

Purpose

Prove that structural branch comparison stays deterministic under drift,
oscillation, and near-match histories.

Scenario

- compare branches with repeated small structural edits
- oscillate between near-identical states
- replay comparisons after restart and after additional unrelated publication

Must verify

- comparison outcomes remain deterministic
- branch-local drift does not contaminate other branches
- explanation surfaces can localize what changed structurally

Required verification output

- `branch_compare_digest`
- `structural_diff_report`
- `replay_digest`
- `counter_snapshot`

Pass condition

Branch comparison remains deterministic, local, and explainable.

## Milestone 9 Named Certification Suites

### 10. Merge Parent Order Determinism Test

Purpose

Prove that ordered multi-parent truth history is consumed deterministically and
that parent order is never treated as incidental.

Scenario

- construct merge-bearing histories with ordered parent lists
- vary host adapter ordering and replay paths
- vary canonical relational merge-class labels that lower into the same
  bridge-consumption class
- run invalidation, continuity, and remapping over those histories

Must verify

- parent ordering survives bridge ingestion and replay
- merge-influenced routing changes are deterministic
- no single-parent fallback assumptions leak into bridge logic
- bridge class lowering remains losslessly traceable back to canonical
  relational merge ontology
- canonical result bundles stay identical across adapter-order variation, not
  only routing digests

Required verification output

- `merge_history_digest`
- `merge_ontology_mapping_report`
- `parent_order_report`
- `routing_digest`
- `result_bundle_digest`
- `replay_digest`

Pass condition

Ordered multi-parent histories produce identical canonical result bundles across
runs, and every lowered bridge merge class remains losslessly attributable to
canonical relational merge ontology.

### 11. Unsupported Merge Class Denial Test

Purpose

Prove that unsupported merge classes fail explicitly and typed instead of
degrading into heuristic branch reconciliation.

Scenario

- feed supported and unsupported merge classes through the same bridge surface
- vary causal frontier metadata and merge-policy outcomes
- include cases denied at different precedence stages:
  lineage, deletion/topology gates, causal admissibility, and policy outcome
- attempt replay and diagnostics queries over rejected cases

Must verify

- unsupported classes are rejected before semantic drift occurs
- diagnostics distinguish unsupported merge class from malformed history
- rejected merges leave no misleading derived artifacts
- denial artifacts identify the exact precedence stage that blocked
  continuation or publication
- structural similarity never reopens continuity or remap after an
  authoritative merge denial
- rejected cases preserve canonical ontology provenance even when no
  bridge-level publication artifact is emitted

Required verification output

- `merge_support_matrix`
- `merge_denial_stage_report`
- `failure_digest`
- `diagnostics_digest`
- `counter_snapshot`

Pass condition

Unsupported or denied merge cases fail closed, remain mechanically attributable,
and cannot be reinterpreted into continuity or reconciliation through
structural convenience.

### 12. Merge Replay And Explanation Parity Test

Purpose

Prove that merge-bearing histories replay identically and that diagnostics can
explain merge-influenced invalidation and continuity behavior.

Scenario

- run merge-bearing histories through original execution
- replay from canonical bridge artifacts
- reconstruct explanations and continuity records for selected subscriptions
- vary diagnostics richness and adapter implementation shape across equivalent
  replay lanes
- include histories whose lowering requires non-trivial causal-frontier,
  lineage, and structural consultation work

Must verify

- replay matches original merge-aware routing
- explanation artifacts identify merge inputs and merge outcomes exactly
- continuity behavior remains branch- and merge-aware after restart
- replay matches the original full canonical result bundle:
  routing, continuity, remap, denial/failure, and explanation
- diagnostics richness changes retained detail only, not causal meaning
- discovery-work counters remain bounded and parity-safe across equivalent runs

Required verification output

- `merge_history_digest`
- `result_bundle_digest`
- `continuity_digest`
- `explanation_digest`
- `replay_digest`
- `counter_snapshot`

Pass condition

Merge-aware replay is bundle-equivalent and causally exact: routing,
continuity, remap, denial, explanation, and bounded discovery work all remain
parity-safe across replay.

## Milestone 10 Named Certification Suites

### 13. Speculative Discard Zero-Residue Test

Purpose

Prove that discarded preview and speculative flows leave no authoritative bridge
residue.

Scenario

- open speculative truth and compute branches
- run preview evaluation and branch-local derived work
- discard selected speculative flows after heavy routing and diagnostics use

Must verify

- discard leaves zero authoritative stream, checkpoint, routing, or writeback residue
- temporary bridge resources are fully reclaimed
- later authoritative runs do not see speculative leftovers

Required verification output

- `speculative_resource_digest`
- `discard_residue_report`
- `routing_digest`
- `counter_snapshot`

Pass condition

Discarded preview work is fully non-authoritative and leaves no residue.

### 14. Speculative Commit Boundary Clarity Test

Purpose

Prove that speculative outcomes become authoritative only through explicit
commit semantics and remain distinguishable from previews.

Scenario

- run parallel preview and authoritative flows over similar branch states
- commit some speculative results and discard others
- replay both histories after restart

Must verify

- committed speculative flows become canonical and explainable
- preview and authoritative outcomes cannot be confused in artifacts
- branch identity remains explicit across commit and discard transitions

Required verification output

- `speculative_commit_digest`
- `preview_vs_authoritative_matrix`
- `replay_digest`
- `diagnostics_digest`

Pass condition

Commit and discard boundaries stay explicit, typed, and replay-safe.

### 15. Preview Lifecycle Leak Resistance Test

Purpose

Prove that repeated preview churn, branch churn, and diagnostics churn do not
leak temporary resources or blur branch-local semantics.

Scenario

- create and discard many preview sessions
- vary branch-local truth views and diagnostics richness
- interleave authoritative work with preview churn

Must verify

- temporary resource lifecycles stay bounded
- branch-local preview state never leaks into authoritative runs
- diagnostics retention policy does not silently retain authoritative-looking preview state

Required verification output

- `preview_lifecycle_digest`
- `resource_bound_report`
- `branch_isolation_matrix`
- `counter_snapshot`

Pass condition

Preview churn remains bounded and isolated.

## Milestone 11 Named Certification Suites

### 16. Policy Provenance Equivalence Test

Purpose

Prove that deterministic and optimized policy modes can change execution policy
without changing the canonical explanation of what policy changed.

Scenario

- run identical bridge flows under deterministic and optimized modes
- vary artifact policy and diagnostics policy explicitly
- compare policy provenance after replay

Must verify

- policy provenance artifacts explain exactly which policy surfaces changed behavior
- identical policy inputs produce identical policy digests
- replay preserves policy-source attribution

Required verification output

- `policy_digest`
- `policy_provenance_report`
- `routing_digest`
- `replay_digest`

Pass condition

Policy changes are explicit, replay-safe, and non-ambient.

### 17. Illegal Policy Combination Rejection Test

Purpose

Prove that illegal or unsupported cross-runtime policy combinations fail
explicitly rather than silently degrading into defaults.

Scenario

- request incompatible combinations of deterministic mode, optimization mode,
  diagnostics richness, and artifact retention
- vary host and truth-side policy inputs

Must verify

- invalid combinations are rejected before execution
- failure classes localize policy-source ambiguity versus policy illegality
- no fallback default hides a bad policy request

Required verification output

- `policy_matrix`
- `failure_digest`
- `diagnostics_digest`

Pass condition

Unsupported policy combinations fail early, typed, and non-ambiguously.

### 18. Ambient Policy Leak Resistance Test

Purpose

Prove that policy does not leak across branches, requests, or consumer contexts
through hidden ambient state.

Scenario

- alternate bridge flows with different policy bundles
- interleave branch-local and historical requests
- replay with reordered host call sequences

Must verify

- each request consumes only its declared policy bundle
- branch-local flows do not inherit stale policy from prior requests
- reordered host execution does not change policy attribution

Required verification output

- `policy_digest`
- `request_policy_matrix`
- `replay_digest`
- `counter_snapshot`

Pass condition

Policy propagation remains explicit and request-scoped.

## Milestone 12 Named Certification Suites

### 19. Bridge Writeback Idempotence And Diff Truth Test

Purpose

Prove that bridge-mediated writeback is idempotent where declared and that the
bridge reflects only explicit output diffs into truth mutation plans.

Scenario

- run signal-driven writeback over repeated equivalent inputs
- compare no-op, changed-output, and repeated-output cases
- replay writeback histories after restart

Must verify

- idempotent writeback emits no extra authoritative mutations
- output diffs are explicit and replay-safe
- read-only and writeback flows remain distinguishable

Required verification output

- `writeback_digest`
- `mutation_plan_digest`
- `idempotence_report`
- `replay_digest`

Pass condition

Equivalent writeback inputs do not corrupt truth with duplicate authority effects.

### 20. Strategy Failure Containment Test

Purpose

Prove that failing, panicking, or divergent bridge-mediated strategies do not
corrupt authoritative truth.

Scenario

- inject failures during mutation-plan production, validation, and commit handoff
- run successful and failing strategies over equivalent inputs
- compare authoritative truth before and after failure

Must verify

- failed strategies publish no authoritative truth
- failure diagnostics distinguish strategy failure from truth-runtime rejection
- later successful retries match the no-failure control run

Required verification output

- `writeback_digest`
- `failure_digest`
- `truth_integrity_report`
- `retry_parity_digest`

Pass condition

Strategy failure is contained, typed, and non-corrupting.

### 21. Authority Bypass Rejection Test

Purpose

Prove that bridge-mediated writeback cannot bypass invariant, merge, or commit
authority.

Scenario

- attempt bridge flows that try to skip invariant validation
- attempt merge-bearing writeback without admitted merge authority
- attempt direct publication-shaped shortcuts

Must verify

- all bypass attempts fail explicitly and typed
- the bridge remains an effect producer, not a second commit authority
- rejection artifacts localize the blocked authority boundary

Required verification output

- `authority_boundary_matrix`
- `failure_digest`
- `diagnostics_digest`

Pass condition

The bridge cannot create a parallel authority path into truth mutation.

## Milestone 12b Named Certification Suites

### 22. Multi-Family Writeback Admission Boundary Test

Purpose

Prove that multiple writeback families can be admitted through one bridge-owned
protocol boundary without turning new families into bridge-core special cases
or opaque host payloads.

Scenario

- admit at least two materially different writeback families
- run equivalent and non-equivalent requests through each family
- attempt undeclared family, wrong-family, and opaque-family admission
- attempt compile-time construction of unwired or skipped-phase family values
- replay admitted and rejected family-bearing paths

Must verify

- family identity remains explicit from admission through replay
- different families do not alias into one bridge writeback meaning
- undeclared or opaque family paths fail before authority execution
- family-aware diagnostics remain bridge-native rather than host-defined
- undeclared families and skipped proof phases are uncompilable wherever the
  type system can enforce them

Required verification output

- `writeback_family_digest`
- `family_contract_digest`
- `bridge_effect_digest`
- `failure_digest`
- `replay_digest`
- `family_admission_record_digest`
- `decision_trace_digest`
- `counter_snapshot`

Pass condition

Multiple families can enter the bridge honestly, and family identity remains a
first-class bridge artifact rather than host-local folklore.

### 23. Cross-Family Replay And Loop Isolation Test

Purpose

Prove that replay, idempotence, and loop-prevention semantics remain
family-correct and do not collapse across writeback families that may appear
structurally similar.

Scenario

- run same-causality and changed-causality flows through at least two admitted
  families
- include same-output-lookalike and different-output family pairs
- replay original and restart-shaped family-bearing histories
- inject bridge-origin feedback pressure into multiple families
- assert exact family lookup, family dispatch, and decision-log retention
  counters on the hot path

Must verify

- cross-family lookalike outputs do not alias replay identity when family
  semantics differ
- loop-prevention classification remains family-visible and replay-stable
- idempotence suppression remains scoped to the correct family contract
- same-family equivalent runs remain equal while cross-family divergent runs
  remain mechanically distinct
- family tracing remains reconstructable from retained native execution and
  replay records alone

Required verification output

- `writeback_family_digest`
- `causality_digest`
- `idempotence_report`
- `loop_prevention_report`
- `replay_digest`
- `counter_snapshot`
- `family_execution_record_digest`
- `family_replay_record_digest`
- `decision_trace_digest`

Pass condition

Replay and feedback pressure preserve family-specific meaning instead of
flattening all families into one writeback lane.

### 24. Host Mapper Parity And Shadow-Protocol Rejection Test

Purpose

Prove that host mappers remain translation layers into admitted family
contracts rather than becoming shadow writeback protocols that redefine bridge
semantics outside canonical artifacts.

Scenario

- run equivalent family-bearing workloads through at least two host mappers
- vary mapper implementation shape while preserving admitted family semantics
- attempt mapper-side redefinition of no-op, retry, failure, or family identity
- attempt family execution with missing bridge-visible mapper evidence
- attempt mapper outputs that try to author replay identity, idempotence
  identity, loop disposition, failure class, or authority classification

Must verify

- mapper-only implementation differences do not change canonical bridge meaning
- host attempts to redefine bridge protocol semantics fail explicitly
- mapper paths cannot hide family-specific authority or loop semantics inside
  host-local strings or opaque payloads
- parity holds only when the mapper is translation, not shadow protocol
- illegal mapper outputs are compile-time rejected where the API can enforce it
- mapper parity and rejection are explainable from retained mapper and
  execution records without reopening host code

Required verification output

- `writeback_family_digest`
- `mapper_parity_matrix`
- `authority_boundary_matrix`
- `failure_digest`
- `diagnostics_digest`
- `family_mapper_record_digest`
- `family_execution_record_digest`
- `decision_trace_digest`
- `counter_snapshot`

Pass condition

Host mappers can translate into bridge writeback families, but they cannot
become the real writeback protocol.

## Milestone 13 Named Certification Suites

### Milestone 13 Reference Workload Requirement

Milestone 13 must include one concrete Rust-only reference workload in addition
to the abstract suite definitions below.

The required reference shape is:

- authoritative products and component costs in `worth-relational`
- derived tariff, tax, margin, and final-price nodes in `worth-signal`
- bridge-coordinated live updates, speculative branch-local shocks, discard,
  commit promotion, replay, and diagnostics over the boundary

Minimum required reference scenarios:

- a high-fanout main-branch component-cost wave over at least 100 products
- a speculative branch-local `rubber +300%` style shock that remains isolated
  from the main branch
- a main-versus-speculative branch comparison bundle over the same fork basis
- a speculative discard lane proving zero authoritative and bridge residue
- a speculative commit-promotion lane proving clear authority-boundary
  promotion through the Milestone 12 writeback contract
- a restart-safe replay lane over the same ordinary pricing route
- a restart-shaped replay-drift lane proving typed mismatch localization after
  truth-shape change

The reference workload should converge on one top-level workload certification
bundle rather than remaining a loose set of unrelated end-to-end assertions.
At minimum that workload bundle should contain nested artifacts for:

- ordinary path reference and replay
- aspect-aware routing and comparison
- hostile missing-snapshot or missing-basis failure
- discard lifecycle under interleaved main churn
- promotion lifecycle under interleaved main churn
- 100-product high-fanout live churn
- restart-safe replay
- restart-shaped replay drift rejection
- writeback authority outcomes covering commit, noop, and typed rejection
- merge-bearing pricing history with revisitable pre-merge, speculative, and
  merged truth states
- historical provenance sufficient to inspect retained shock criteria from
  bridge-visible truth at a historical commit

This workload is a certification fixture, not a bridge-owned finance product
surface. Its role is to prove that the bridge can carry a concrete dual-runtime
story end to end without relying on a UI or on host-local debug folklore.

### 25. End-To-End Causality Bundle Equivalence Test

Purpose

Prove that causality survives from truth commit through bridge routing into
derived explanation and replay.

Scenario

- run change, routing, historical, merge-bearing, and policy-bearing bridge flows
- capture original execution bundles
- replay from canonical bridge artifacts
- compare end-to-end causality records

Must verify

- the same causality tokens survive original execution and replay
- explanation surfaces remain aligned with routing and truth-view records
- diagnostics tiers change retained richness only
- the reference workload preserves the same main-branch and speculative-branch
  causality digests across original execution and replay
- retained historical pricing commits can expose their upstream shock criteria
  through bridge-visible truth and bundle artifacts rather than hidden scenario
  memory

Required verification output

- `causality_digest`
- `routing_digest`
- `explanation_digest`
- `replay_digest`
- `reference_workload_bundle_digest`

Pass condition

End-to-end bridge causality is canonical, replay-safe, and mechanically inspectable.

### 26. Failure Taxonomy Localization Test

Purpose

Prove that bridge-native failure classes are complete enough to localize real
adversarial failures without collapsing into host strings.

Scenario

- inject failures across stream, source, remap, merge, preview, policy, and
  writeback paths
- compare typed failure capture across original execution and replay

Must verify

- failures map into explicit bridge-native classes
- failure localization identifies the exact failed protocol or planning boundary
- replay preserves failure meaning
- reference-workload failures such as wrong-branch comparison, preview misuse,
  source mismatch, and writeback denial remain typed and replay-stable
- retained provenance and residue surfaces are sufficient to distinguish
  routing, branch-isolation, policy, source, preview, merge, writeback, and
  residue failures mechanically

Required verification output

- `failure_digest`
- `failure_localization_matrix`
- `replay_failure_digest`
- `diagnostics_digest`
- `reference_workload_failure_bundle_digest`

Pass condition

Bridge failures are typed, composable, and replay-stable.

### 27. Certification Matrix Sufficiency Test

Purpose

Prove that the bridge certification bundle itself is sufficient to diagnose
routing, merge-aware, policy-aware, historical, preview, and writeback failures
mechanically.

Scenario

- run a mixed milestone 6-13 workload matrix
- produce canonical certification bundles only
- attempt offline diagnosis from those bundles without live host/runtime access

Must verify

- certification artifacts are enough to distinguish the major failure families
- bundle completeness does not depend on ambient runtime state
- the bridge has one coherent public diagnostics entrypoint
- the reference workload can be diagnosed offline for main-branch live runs,
  speculative runs, discard residue checks, and commit-promotion outcomes
- the reference workload can diagnose retained historical shock lineage from
  canonical artifacts alone

Required verification output

- `certification_bundle_digest`
- `bundle_completeness_report`
- `diagnostics_entrypoint_matrix`
- `counter_snapshot`
- `reference_workload_bundle_digest`
- `reference_workload_bundle_comparison`

Pass condition

The bridge can be diagnosed and certified from its canonical artifacts alone.

## Milestone 14 Named Certification Suites

### 28. Subscription Declaration Equivalence Test

Purpose

Prove that semantically equivalent bridge subscription declarations lower into
the same canonical subscription identity and admission artifacts without
depending on host-local declaration order or builder trivia.

Scenario

- declare equivalent subscriptions through more than one host-shaped
  construction path
- vary declaration order, diagnostics richness, and incidental builder detail
- admit the resulting declarations through the bridge
- compare against intentionally non-equivalent declarations

Must verify

- equivalent declarations yield identical `subscription_digest`
- equivalent declarations yield identical `subscription_registry_digest`
- equivalent declarations yield identical admission, slice-lowering, and basis
  records
- intentionally different declarations compare unequal on canonical
  subscription identity
- diagnostics tiers change retained detail only
- declaration identity remains replay-safe
- representative declaration lanes keep family-registry lookup breadth,
  normalization-sort count, allocation count, and clone count within the
  declared Milestone 14 contract
- representative declaration lanes prove unrelated family scans remain zero on
  the hot path after registry freeze

Required verification output

- `subscription_digest`
- `subscription_registry_digest`
- `subscription_basis_digest`
- `routing_digest`
- `diagnostics_digest`
- `replay_digest`
- `counter_snapshot`

Pass condition

Equivalent subscription declarations converge to identical canonical
subscription identity and non-equivalent declarations do not.

### 29. Subscription Basis Binding And Rejection Boundary Test

Purpose

Prove that subscription basis binding to explicit snapshot identities and
branch-head truth views is explicit, replay-safe, and fail-closed when
unsupported.

Scenario

- bind otherwise equivalent subscriptions to explicit snapshot identities and
  branch-head bases
- inject unsupported basis classes and hostile basis-resolution failures
- replay admitted and rejected basis-binding paths

Must verify

- basis selection is explicit and reflected in canonical basis digests
- unsupported basis requests fail explicitly before activation
- replay preserves both admitted and rejected basis meaning
- no lane silently falls back to latest reachable truth
- representative basis-admission lanes keep basis lookup count, strategy
  candidate count, lowered-slice count, allocation count, and clone count
  within the declared Milestone 14 contract
- unsupported basis lanes fail before broad lowering or family-wide rescans

Required verification output

- `subscription_digest`
- `subscription_registry_digest`
- `subscription_basis_digest`
- `truth_view_digest`
- `failure_digest`
- `replay_digest`
- `counter_snapshot`

Pass condition

Basis binding remains explicit, canonical, and fail-closed under admitted and
rejected variations.

### 30. Subscription Lifecycle Replay Parity Test

Purpose

Prove that subscription lifecycle transitions across admitted artifacts,
activation-ready handles, deactivated handles, and rejection explanation
surfaces remain replay-safe and do not depend on ambient host state.

Scenario

- admit subscriptions and prepare activation-ready handles
- deactivate some subscriptions and reject others before activation
- replay retained lifecycle bundles after restart and diagnostics-tier variation

Must verify

- lifecycle transitions produce canonical typestate records
- replay preserves lifecycle meaning exactly
- rejected lanes fail at the declared lifecycle boundary
- diagnostics tiers change retained richness only
- representative lifecycle and replay lanes prove declaration-family meaning is
  not rediscovered through repeated family scans or strategy reselection
- replay reconstruction keeps reconstruction count, allocation count, and clone
  count within the declared Milestone 14 contract

Required verification output

- `subscription_digest`
- `subscription_registry_digest`
- `subscription_lifecycle_digest`
- `diagnostics_digest`
- `failure_digest`
- `replay_digest`
- `counter_snapshot`

Pass condition

Subscription lifecycle meaning is canonical, replay-safe, and mechanically
inspectable.

## Milestone 15 Named Certification Suites

### 31. Shared Subscription Fanout Parity Test

Purpose

Prove that equivalent consumers can share one bridge-native subscription
without changing canonical subscription meaning or delivery truth.

Scenario

- attach at least two equivalent consumer contracts to one shared subscription
- compare against separately activated but equivalent subscriptions
- vary pacing, coalescing where admitted, and diagnostics richness

Must verify

- shared and separate-but-equivalent lanes agree on canonical subscription
  meaning
- fanout does not distort routing, delivery, or checkpoint truth
- admitted coalescing changes pacing only
- rejected sharing attempts fail explicitly and typed

Required verification output

- `subscription_digest`
- `subscription_share_digest`
- `subscription_delivery_digest`
- `consumer_contract_digest`
- `counter_snapshot`

Pass condition

Subscription sharing is parity-safe, explicit, and diagnosable.

### 32. Subscription Continuation Across Identity Evolution Test

Purpose

Prove that active subscriptions continue, split, merge, or reject continuity
deterministically when truth identity evolves.

Scenario

- activate subscriptions over truth surfaces that later undergo replace,
  split, merge-like, and branch-divergent evolution
- vary continuity outcomes across admitted and rejected classes
- replay continuation paths after restart

Must verify

- continuation outcomes are explicit and typed
- continuation digests remain stable for semantically equivalent runs
- ambiguous or unsupported continuity cases fail explicitly
- branch-local identity evolution does not leak across unrelated subscriptions
- replay preserves continuation meaning

Required verification output

- `subscription_digest`
- `subscription_continuation_digest`
- `routing_digest`
- `failure_digest`
- `replay_digest`

Pass condition

Subscription continuation across identity evolution is deterministic,
replay-safe, and fail-closed where required.

### 33. Subscription Resume, Replay, And Checkpoint Exactness Test

Purpose

Prove that active subscriptions resume and replay from canonical checkpoint
basis without changing delivered subscription meaning.

Scenario

- run long-lived active subscriptions over bursty change windows
- checkpoint after several admitted delivery boundaries
- inject restart after partial progress
- resume from canonical subscription checkpoint basis
- replay from canonical subscription artifacts

Must verify

- resumed subscriptions preserve delivery semantics exactly
- subscription checkpoint meaning remains explicit and distinct from raw stream
  offsets
- replay reconstructs canonical delivery truth
- stale, truncated, or incompatible continuation basis fails explicitly and
  typed
- equivalent resumed and control lanes compare equal on canonical subscription
  delivery digests

Required verification output

- `subscription_digest`
- `subscription_delivery_digest`
- `checkpoint_digest`
- `replay_digest`
- `failure_digest`

Pass condition

Subscription resume, replay, and checkpoint semantics are canonical, exact, and
diagnosable.

### 34. Preview Subscription Zero-Residue Test

Purpose

Prove that preview-scoped subscriptions remain isolated from authoritative
subscription state and leave zero authoritative residue after discard.

Scenario

- activate preview-scoped subscriptions under admitted preview bases
- drive routing, delivery, diagnostics, and continuation work through them
- discard some preview subscriptions and promote others through an explicit
  promotion boundary
- compare against authoritative-only control lanes

Must verify

- discarded preview subscriptions leave zero authoritative residue
- promotion records remain explicit and basis-bound
- preview and authoritative subscription identities remain distinct until
  explicit promotion
- preview subscription replay does not depend on ambient current state
- illegal preview reuse or cross-session sharing fails explicitly

Required verification output

- `subscription_digest`
- `subscription_basis_digest`
- `subscription_lifecycle_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Preview subscriptions are isolated, fail-closed, and zero-residue on discard.

## Milestone 16 Named Certification Suites

### 35. End-To-End Subscription Bundle Equivalence Test

Purpose

Prove that long-lived subscription meaning survives from declaration through
activation, delivery, continuation, and replay without semantic drift.

Scenario

- run an end-to-end subscription workload spanning authoritative, branch-local,
  historical, and preview-scoped subscriptions
- include shared-consumer fanout and restart-safe replay lanes
- compare original execution bundles against replay bundles

Must verify

- canonical subscription meaning survives original execution and replay
- equivalent end-to-end subscription lanes compare equal on canonical bundle
  digests
- intentionally different subscription lanes compare unequal where they should
- diagnostics tiers change retained detail only
- the reference workload preserves the same branch-local and authoritative
  subscription truths across replay

Required verification output

- `subscription_digest`
- `subscription_lifecycle_digest`
- `subscription_delivery_digest`
- `replay_digest`
- `reference_workload_bundle_digest`

Pass condition

End-to-end subscription behavior is canonical, replay-safe, and mechanically
comparable.

### 36. Subscription Failure Taxonomy Localization Test

Purpose

Prove that the bridge can localize subscription-specific failures into
bridge-native classes without collapsing into host strings or generic stream
errors.

Scenario

- inject failures across declaration, basis binding, activation, sharing,
  continuation, preview discard, promotion, and replay paths
- compare failure capture across original execution and replay

Must verify

- subscription failures map into explicit bridge-native classes
- localization identifies the exact failed subscription boundary
- replay preserves failure meaning
- equivalent failure lanes compare equal on canonical failure artifacts
- retained artifacts are sufficient to distinguish declaration, basis,
  lifecycle, continuation, delivery, and residue failures mechanically

Required verification output

- `subscription_digest`
- `failure_digest`
- `failure_localization_matrix`
- `diagnostics_digest`
- `replay_failure_digest`

Pass condition

Subscription failures are typed, localizable, and replay-stable.

### 37. Subscription Reference Workload Sufficiency Test

Purpose

Prove that the bridge emits enough canonical subscription artifacts for an
auditor to certify a realistic end-to-end subscription workload offline.

Scenario

- run a mixed subscription workload with authoritative, historical,
  branch-local, preview, shared-consumer, restart, and discard or promotion
  lanes
- produce canonical subscription bundles only
- attempt offline diagnosis and equivalence comparison from those bundles

Must verify

- bundle completeness does not depend on ambient runtime state
- the workload can diagnose active subscription meaning, branch isolation,
  continuation outcomes, preview discard residue, and promotion boundaries from
  canonical artifacts alone
- the bridge diagnostics entrypoint exposes one coherent subscription artifact
  story
- the workload remains boundary-honest rather than Query-specific or host-
  specific

Required verification output

- `certification_bundle_digest`
- `subscription_digest`
- `subscription_basis_digest`
- `reference_workload_bundle_digest`
- `reference_workload_bundle_comparison`

Pass condition

The bridge can certify end-to-end subscription workloads from canonical
artifacts alone.

## Milestone 17 Named Certification Suites

### 38. Temporal Bridge Basis Equivalence Test

Purpose

Prove that a bridge flow depending on signal time binds truth-view evidence and
signal temporal evidence into one canonical temporal bridge basis without
moving clock authority into the bridge or truth runtime.

Scenario

- construct equivalent truth views through snapshot, branch-head, and CDC
  cursor bases where admitted
- pair them with equivalent signal clock basis and ready-wake evidence
- vary host call order, diagnostics tier, adapter implementation shape, and
  non-authoritative wall-clock metadata
- include mismatched clock domains, stale wake evidence, missing truth basis,
  and branch-crossed temporal evidence as hostile lanes

Must verify

- equivalent truth-basis plus signal-temporal-basis inputs produce equivalent
  temporal bridge basis digests
- wall-clock or presentation metadata cannot become bridge temporal authority
- branch, snapshot, CDC cursor, clock basis, and wake identity are all visible
  in the basis artifact
- stale, missing, cross-branch, or wrong-domain temporal evidence fails
  explicitly and typed
- diagnostics-tier variation changes retained detail only

Required verification output

- `temporal_bridge_basis_digest`
- `truth_view_basis_digest`
- `signal_clock_basis_digest`
- `temporal_wake_evidence_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Temporal bridge basis is canonical, explicit, replayable, and cannot be
defined by host time or bridge-local convention.

### 39. Truth Patch Plus Clock Advance Replay Parity Test

Purpose

Prove that mixed truth-patch and signal clock-advance sequences replay to the
same bridge routing, delivery, and explanation artifacts even when host call
order varies.

Scenario

- run workloads where relational commits and signal clock advances interleave
  in multiple physically different orders
- include time-only eligibility where no relational patch occurred
- include truth patches that arrive before, after, and at the same logical
  delivery boundary as temporal wakes
- replay from canonical truth, temporal, and bridge artifacts only
- inject duplicate clock submissions, skipped host polling, and diagnostics-tier
  variation

Must verify

- equivalent canonical cause sets produce identical routing and delivery
  digests under replay
- time-only updates are not misreported as truth-patch-caused updates
- truth-patch-plus-clock updates retain both cause identities
- duplicate or stale clock submissions are typed denials rather than extra
  delivery
- replay does not consult ambient process time or host callback ordering

Required verification output

- `mixed_cause_digest`
- `routing_digest`
- `subscription_delivery_digest`
- `temporal_bridge_basis_digest`
- `explanation_digest`
- `replay_digest`
- `failure_digest`

Pass condition

Mixed truth and clock causality is replay-stable, order-honest, and
diagnosable.

### 40. Time-Aware Subscription Basis Rejection Test

Purpose

Prove that time-aware bridge subscriptions fail closed when their truth basis,
signal temporal basis, or admitted temporal family is missing, stale,
unsupported, or cross-wired.

Scenario

- declare time-aware subscriptions over authoritative, historical,
  branch-local, and preview-scoped truth views
- vary temporal policies across debounce, throttle, stale-after, interval, and
  explicit wake-driven flows where supported by `worth-signal`
- inject missing snapshot evidence, stale CDC cursor, wrong branch clock
  evidence, unsupported historical temporal replay, and host-WORTHd temporal
  family labels
- attempt resume from a checkpoint with incompatible truth or temporal basis

Must verify

- supported combinations admit with explicit basis artifacts
- unsupported combinations reject before activation or delivery
- host-WORTHd temporal labels cannot replace `worth-signal` temporal evidence
- rejection localizes the failed basis boundary exactly
- replay preserves both admitted and rejected outcomes

Required verification output

- `subscription_digest`
- `subscription_basis_digest`
- `temporal_bridge_basis_digest`
- `admission_digest`
- `failure_localization_matrix`
- `replay_digest`

Pass condition

Time-aware subscriptions are basis-bound, fail-closed, and replay-safe.

### 41. Historical Truth With Temporal Wake Replay Test

Purpose

Prove that temporal wake readiness over retained historical truth replays from
canonical truth and signal temporal artifacts, not from current truth, ambient
time, or host memory.

Scenario

- evaluate time-aware subscriptions against retained historical snapshots
- advance signal time so wakes become ready after the historical truth basis is
  selected
- mutate current truth aggressively after the historical basis is pinned
- restore and replay with retention-rich and retention-truncated lanes
- include previous-value-sensitive derived work near temporal threshold
  boundaries where supported by signal artifacts

Must verify

- historical truth reads remain pinned to the declared truth basis
- temporal wake readiness remains pinned to the declared signal temporal basis
- current truth mutation cannot affect historical time-aware results
- retention truncation fails explicitly when the replay basis is no longer
  available
- previous-value evidence, if present, remains branch- and checkpoint-honest

Required verification output

- `historical_truth_basis_digest`
- `temporal_bridge_basis_digest`
- `previous_value_evidence_digest`
- `derived_output_digest`
- `retention_basis_report`
- `replay_digest`
- `failure_digest`

Pass condition

Historical truth plus temporal readiness remains exact under replay, restore,
retention pressure, and current-truth churn.

## Milestone 18 Named Certification Suites

### 42. Async Source Lifecycle Bridge Parity Test

Purpose

Prove that bridge-declared async/resource sources lower into admitted
`worth-signal` async lifecycle families without letting transport adapters or
host callbacks define lifecycle truth.

Scenario

- declare async/resource bridge sources over authoritative, branch-local, and
  historical truth-view bases
- run fulfilled, rejected, cancelled, timed-out, retried, superseded, and
  stale-denied paths
- vary transport adapter implementation shape while preserving admitted
  bridge/source semantics
- replay from canonical bridge and signal async artifacts

Must verify

- equivalent source declarations and lifecycle outcomes compare equal across
  adapters
- each lifecycle transition is typed and bound to bridge declaration, truth
  basis, subscription instance where applicable, and signal request generation
- adapter-local status strings cannot become lifecycle authority
- diagnostics distinguish source-family rejection from signal lifecycle denial
  and transport failure
- replay preserves lifecycle meaning

Required verification output

- `async_source_declaration_digest`
- `async_lifecycle_digest`
- `truth_view_basis_digest`
- `signal_async_generation_digest`
- `failure_digest`
- `replay_digest`

Pass condition

Async/resource source lifecycle is bridge-visible, signal-owned,
adapter-agnostic, and replay-safe.

### 43. Out-Of-Order Completion Truth-Basis Supersession Test

Purpose

Prove that stale async completions cannot publish over newer truth bases,
newer signal generations, different branches, or superseded subscription
instances.

Scenario

- admit async requests R1, R2, and R3 over evolving truth bases and signal
  generations
- complete them in adversarial physical order, including old success after new
  success, old failure after retry admission, and completion after branch
  switch or preview discard
- include same-output-lookalike completions with different truth bases
- replay with completion order shuffled relative to original host delivery

Must verify

- only the completion matching the active admitted basis and generation can
  publish derived state
- stale completions fail typed even when payloads match the current output
- branch-crossed, preview-discarded, and subscription-superseded completions
  cannot commit
- denial artifacts name the superseded truth basis, signal generation, and
  subscription instance where applicable
- replay gives the same accepted and denied completion set

Required verification output

- `async_completion_digest`
- `supersession_report`
- `truth_view_basis_digest`
- `signal_async_generation_digest`
- `subscription_instance_digest`
- `failure_digest`
- `replay_digest`

Pass condition

Out-of-order completion cannot corrupt derived truth or hide stale causality.

### 44. Async Retry And Revalidation Causality Test

Purpose

Prove that retry and revalidation preserve bridge causality and compare equal
to no-failure control lanes when the admitted signal lifecycle semantics say
they should.

Scenario

- run equivalent no-failure and retry-after-failure lanes
- include timeout-driven retry, explicit revalidation, cancellation followed by
  fresh admission, and retry racing a truth-basis update
- vary temporal retry/backoff evidence where signal exposes it
- inject retry storms and starvation pressure with boundedness counters

Must verify

- retry/revalidation identity is bound to the original bridge source
  declaration and the current admitted truth/temporal basis
- retry lanes that semantically converge to the no-failure control produce
  equal canonical output and causality digests
- retries over stale truth basis or stale signal generation fail explicitly
- retry storms are bounded by admitted signal and bridge counters
- diagnostics distinguish retry scheduling from completion admission

Required verification output

- `retry_causality_digest`
- `async_lifecycle_digest`
- `temporal_bridge_basis_digest`
- `derived_output_digest`
- `counter_snapshot`
- `failure_digest`
- `replay_digest`

Pass condition

Retry and revalidation are causal, bounded, replayable, and not host-defined.

### 45. Async Completion Writeback Loop Prevention Test

Purpose

Prove that async completion writeback cannot create bridge-origin feedback
loops, bypass truth authority, or publish duplicate authoritative truth.

Scenario

- route async completion through admitted bridge writeback families
- include idempotent duplicate completion, completion after truth changed,
  completion from a preview branch, and completion whose payload would trigger
  the same source subscription again
- inject relational writeback rejection and bridge mapper failure
- replay with host adapter and completion order variation

Must verify

- authoritative truth changes happen only through admitted writeback artifacts
  and relational commit authority
- loop-prevention evidence links bridge-origin completion, writeback intent,
  resulting truth commit where admitted, and downstream invalidation
- duplicate completions are idempotent or typed denials according to admitted
  family semantics
- rejected writebacks leave no authoritative residue
- replay preserves idempotence, denial, and causality artifacts

Required verification output

- `async_completion_digest`
- `writeback_family_digest`
- `idempotence_report`
- `loop_prevention_report`
- `truth_commit_digest`
- `failure_digest`
- `replay_digest`

Pass condition

Async completion writeback preserves truth authority, idempotence, and
loop-safety under failure and replay.

## Milestone 19 Named Certification Suites

### 46. Temporal Async Subscription Bundle Equivalence Test

Purpose

Prove that subscriptions combining truth patches, signal time, and
async/resource completions preserve one canonical meaning from declaration
through replay.

Scenario

- run subscriptions over authoritative, historical, branch-local, preview, and
  shared-consumer bases
- include truth-triggered, time-triggered, async-triggered, and combined-cause
  deliveries
- vary host adapter shape, diagnostics tier, clock submission timing, and
  physical async completion order
- compare original, replayed, restarted, and control bundles

Must verify

- equivalent temporal/async subscription lanes compare equal on canonical
  bundle digests
- intentionally different truth, temporal, async, or subscription bases compare
  unequal
- diagnostics richness changes detail only
- all parent-runtime authority bases remain traceable from the bundle
- replay and restart preserve subscription meaning

Required verification output

- `temporal_async_subscription_bundle_digest`
- `subscription_digest`
- `temporal_bridge_basis_digest`
- `async_lifecycle_digest`
- `mixed_cause_digest`
- `replay_digest`

Pass condition

Temporal/async subscriptions are mechanically comparable across original
execution, replay, restart, and hostile adapter variation.

### 47. Mixed Cause Delivery Ordering Parity Test

Purpose

Prove that delivery ordering across truth patches, clock advances, temporal
wakes, async completions, retries, cancellations, and supersessions is
canonical and replay-stable.

Scenario

- create delivery windows containing multiple cause families at the same
  logical boundary
- shuffle host call order and physical async completion order
- include duplicate causes, stale causes, branch-local causes, and preview
  causes
- compare delivery order against replay and independently constructed
  equivalent runs

Must verify

- canonical mixed-cause ordering is stable and explainable
- stale or duplicate causes do not create extra delivery
- branch-local and preview causes do not leak into authoritative delivery
- consumer pacing and coalescing change delivery shape only through admitted
  contracts, not subscription meaning
- replay reconstructs the same ordered delivery digest

Required verification output

- `mixed_cause_order_digest`
- `subscription_delivery_digest`
- `consumer_contract_digest`
- `coalescing_report`
- `failure_digest`
- `replay_digest`

Pass condition

Mixed-cause delivery order is deterministic, bounded, and authority-honest.

### 48. Restart Resume With Clock And Inflight Basis Test

Purpose

Prove that restart and resume recover active time-aware and async-backed
subscriptions from canonical truth, temporal, inflight, and delivery basis
artifacts.

Scenario

- checkpoint subscriptions with pending temporal wakes, ready temporal wakes,
  inflight async requests, partial delivery, and shared consumers
- restart after partial progress and resume from canonical basis
- inject stale checkpoint, truncated retention, missing inflight generation,
  incompatible clock basis, and branch-crossed resume evidence
- compare against uninterrupted control lanes

Must verify

- resumed lanes match uninterrupted control lanes where basis is complete and
  compatible
- stale, truncated, incompatible, or cross-branch basis fails explicitly and
  typed
- inflight async requests resume, cancel, retry, or reject according to
  canonical signal async lifecycle evidence
- temporal wakes resume from retained temporal basis rather than broad
  rediscovery
- subscription checkpoint meaning remains distinct from raw stream offsets,
  raw clock ticks, and raw transport handles

Required verification output

- `checkpoint_digest`
- `temporal_bridge_basis_digest`
- `inflight_async_basis_digest`
- `subscription_delivery_digest`
- `resume_report`
- `failure_digest`
- `replay_digest`

Pass condition

Restart and resume are exact for time-aware and async-backed subscriptions, or
fail closed with localizable basis errors.

### 49. Temporal Async Failure Taxonomy Localization Test

Purpose

Prove that temporal/async bridge failures localize to precise bridge-native
classes instead of collapsing into generic stream, scheduler, transport, or
host errors.

Scenario

- inject failures across temporal basis binding, clock-domain mismatch,
  temporal wake evidence, historical retention, async source declaration,
  completion admission, retry/revalidation, writeback, mixed-cause delivery,
  resume, preview discard, and promotion
- run equivalent failure lanes through original execution and replay
- vary diagnostics richness and host adapter shape

Must verify

- every failure maps to a typed bridge-native class with parent-runtime
  provenance
- localization identifies the exact failed boundary and rejected artifact
- equivalent failures compare equal across replay and adapter variation
- diagnostics tier affects retained context only
- offline artifacts are sufficient to distinguish temporal, async, truth,
  subscription, delivery, and writeback failure families

Required verification output

- `failure_taxonomy_matrix`
- `failure_digest`
- `diagnostics_digest`
- `temporal_bridge_basis_digest`
- `async_lifecycle_digest`
- `mixed_cause_digest`
- `replay_failure_digest`

Pass condition

Temporal/async bridge failures are typed, localizable, replay-stable, and
offline diagnosable.

### 50. End-To-End Temporal Async Reference Workload Sufficiency Test

Purpose

Prove that the bridge emits enough canonical artifacts for an aerospace- or
fintech-grade auditor to certify a realistic end-to-end workload involving
truth changes, time-only updates, async resources, subscriptions, preview,
promotion, discard, restart, and replay.

Scenario

- run one mixed reference workload with authoritative truth commits,
  historical reads, branch-local preview, time-gated updates, stale-after or
  interval wakes, async success, async failure, retry, cancellation,
  supersession, shared-consumer fanout, writeback, restart, and replay
- include hostile lanes for stale completion, incompatible resume basis,
  retention truncation, branch-crossed temporal evidence, preview residue, and
  writeback rejection
- produce canonical bundles only
- attempt offline diagnosis, equivalence comparison, and failure localization
  from those bundles without live runtime or host logs

Must verify

- bundle completeness does not depend on ambient process clock, host transport
  state, host logs, or live runtime memory
- authoritative, historical, branch-local, preview, time-only, async-backed,
  shared-consumer, restart, and replay lanes are all certifiable from one
  coherent artifact story
- reference workload proves both success parity and hostile failure
  localization
- all truth, temporal, async, delivery, subscription, and writeback causes are
  traceable to their owning runtime or bridge boundary
- the workload remains a certification fixture, not bridge-owned domain
  semantics

Required verification output

- `temporal_async_reference_workload_digest`
- `certification_bundle_digest`
- `temporal_async_subscription_bundle_digest`
- `mixed_cause_order_digest`
- `failure_taxonomy_matrix`
- `offline_diagnosis_report`
- `counter_snapshot`

Pass condition

The bridge can certify temporal/async end-to-end behavior from canonical
artifacts alone under production-grade restart, replay, ordering, retention,
branch, preview, transport, and writeback pressure.

## What These Tests Collectively Prove

Together, these tests prove that the bridge from Milestone 6 onward is:

- protocol-grade rather than adapter-grade
- replay-safe under restart, backpressure, and multi-consumer pressure
- host-agnostic at its public source and stream boundaries
- explicit about structural identity, merge semantics, speculative flows, and policy provenance
- explicit about first-class subscription declaration, lifecycle, continuation,
  sharing, preview isolation, and replay
- explicit about temporal-basis binding, time-aware subscription lowering,
  async/resource lifecycle causality, mixed truth/time/async delivery ordering,
  and restart-safe temporal and inflight resume
- lossless about authority provenance when lowering parent-runtime truth into
  bridge-consumption vocabularies
- lossless about authority provenance when binding signal temporal and async
  lifecycle artifacts to truth-view and bridge subscription artifacts
- unable to bypass truth authority during writeback
- certifiable through canonical artifacts rather than intuition

## Milestone 6+ Certification Rule

No Milestone 6+ bridge capability should be considered closed until its named
certification suites emit canonical machine-checkable outputs and pass across:

- original execution
- replay from canonical bridge artifacts
- hostile adapter or scheduling variation
- hostile clock-advance, temporal-wake, and async-completion ordering variation
- diagnostics-tier variation where admitted

Without that, the bridge may still be promising, but it is not yet trust-grade.
