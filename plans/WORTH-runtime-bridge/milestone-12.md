# Milestone 12 Engineering Spec: Bridge-Mediated Commit Strategies And Derived Writeback Contracts

> **Status:** Complete for the first admitted writeback family; Milestone 12b is required before Milestone 13 to make writeback family extensibility bridge-native and production-grade
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-11.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-11.md)
>
> **Bridge certification companion:** [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
>
> **Relational authority companion:** [milestone-8.5-plan.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/milestone-8.5-plan.md)
>
> **Primary architectural driver:** make bridge-origin effect production, writeback admission, relational strategy selection, idempotence classification, authority-boundary outcome, and replay-safe provenance first-class bridge protocol surfaces so derived execution can propose truth changes without giving the bridge mutation authority or creating duplicate authority effects under retry and replay pressure

## Summary

Milestones 1 through 11 made the bridge honest about routing, historical truth,
merge-aware history, speculation, and policy. The bridge can now explain how
truth drove derived execution, but it cannot yet explain how derived execution
may legally propose truth changes.

Without Milestone 12, the bridge still has a fatal hole:

- writeback can drift into adapter-local folklore
- bridge effect meaning can disappear before the authority boundary
- causality can fragment exactly where loop prevention needs it most
- retries can fabricate duplicate authoritative commits
- failing strategy execution can blur with truth-boundary rejection
- replay can prove final state only, not bridge-origin authority transfer

Milestone 12 exists to close that hole without turning the bridge into a second
commit runtime.

## Implementation Progress

The codebase now has a real Milestone 12 substrate rather than only a plan.

Implemented today:

- bridge-owned writeback declaration, validation, admission, effect, causality,
  feedback provenance, idempotence, loop-prevention, strategy-basis,
  strategy-compatibility, authority outcome, replay, and counter artifacts
- builder-owned optional truth writeback authority seam with freeze-at-build
  semantics
- runtime execution through a real writeback authority boundary
- first-class bridge-origin feedback provenance in runtime and harness flows
- first-class writeback strategy class and retry semantics for the first narrow
  admitted strategy family
- validated writeback candidate artifacts between pre-authority classification
  and authority execution
- authority receipt contract validation, including request/receipt coherence and
  malformed receipt fail-closed handling
- typed fail-closed handling for preview misuse, unbound authority, unsafe
  feedback, merge rejection, stale basis rejection, authority transport
  failure, and authority panic
- hostile certification lanes for duplicate-attempt boundedness, authority
  bypass rejection, merge-boundary rejection, unsafe feedback pre-authority
  rejection, contradictory feedback rejection, and bridge-origin feedback
  convergence through canonical no-op

Still intentionally incomplete:

- a broader admitted strategy surface beyond the current narrow Phase 1 class
- richer production-shaped authority adapters beyond the in-memory harness seam
- final certification breadth for every planned hostile lane in suites 19-21
- bridge-native extensible writeback families and mapper-containment rules for
  domain-honest multi-family writeback

It also has to respect the crate that actually exists today:

- the public `RuntimeBridge` facade is still overwhelmingly truth-to-signal
  oriented: ingest, plan, deliver invalidation, prepare signal evaluation,
  replay
- `BridgeRequestKind` currently lives in speculation and is closed over
  `Authoritative | Preview`; it is not yet a general bridge request taxonomy
- the builder currently wires truth-read sources, one signal invalidation sink,
  optional source/lineage adapters, and frozen registries; there is no existing
  truth-write adapter seam to piggyback on

Milestone 12 therefore cannot be an "add one more callback" change. It must
introduce a new authority-adjacent subsystem that fits the bridge's existing
phase-typed, registry-backed, canonical-record style.

## Goal

Make bridge-mediated effect production and derived writeback a deterministic,
replay-safe, bridge-owned protocol so bridge flows can propose relational truth
changes only through registered relational commit strategies, while preserving
serialized authority, invariant enforcement, merge authority, first-class
causality, and canonical replay.

## Why This Milestone Exists

Milestone 12 belongs immediately after Milestone 11 because writeback legality
is unsafe until policy and request-kind boundaries are explicit.

Milestone 10 established that preview and authority are structurally distinct.
Milestone 11 established that policy is explicit and request-scoped. Milestone
12 now has to establish the missing truth:

- one bridge-owned writeback declaration surface
- one bridge-owned derived-effect artifact that survives to replay
- one explicit admission boundary into relational commit strategies
- one closed equivalence contract for idempotence and no-op publication
- one typed failure topology for strategy failure, invariant rejection, merge
  rejection, stale basis, and authority bypass

This milestone also belongs before Milestone 12b and Milestone 13 because
end-to-end causality is not certifiable while bridge-mediated writeback still
disappears into host code, and production-grade reference workloads are not
honest until the bridge can admit more than one writeback family without
falling back to host-local shadow protocols.

## Hard Part

The hard part of Milestone 12 is not effect production in isolation. It is
allowing a `worth-signal` node to write back into `worth-relational` through the
bridge without:

- turning the bridge into a shadow commit engine
- publishing duplicate authoritative commits under retry, replay, or race
- feeding a bridge-origin writeback commit straight back into the same derived
  node set and creating an infinite invalidation loop
- relying on host-local "debounce" or "just don't do that" folklore to stay
  safe

The code path will likely be easier than the proof path. The harness burden is
therefore part of the milestone, not cleanup after it.

## Adversarial Constraint

Milestone 12 must survive the following hostile condition:

> Repeated equivalent bridge evaluations, preview and authoritative request
> interleaving, merge-bearing truth history, policy variation, restart between
> effect production and commit handoff, failing or panicking relational strategy
> executors, diagnostics-tier variation, and adapter-specific retry behavior
> must produce the same writeback admission, the same effect digest, the same
> causality chain, the same no-op-versus-commit decision, the same failure
> class, and the same replay result every time, while never allowing the bridge
> to mutate truth except through one explicit relational authority boundary and
> while never allowing bridge-origin writeback commits to induce unbounded
> self-triggered recomputation loops.

If any path:

- treats writeback as a loose boolean on an ordinary bridge request
- allows direct bridge mutation without registered relational strategy identity
- erases bridge effect meaning into raw mutations too early
- fails to preserve the triggering truth/route/evaluation causality chain into
  the writeback artifact
- decides idempotence by adapter folklore
- reuses preview artifacts as authority input without re-admission
- lets failed strategies leave authoritative residue
- hides invariant or merge denial behind adapter fallback
- relies on "eventually the loop settles" rather than a declared bridge/runtime
  loop-prevention contract

then the milestone has failed.

## Explicit Assumptions

Milestone 12 must make the following assumptions explicit instead of leaving
them ambient:

- writeback is not admitted for every signal evaluation; it is opt-in and
  declaration-bound
- some writeback classes are contractually idempotent and some may not be; the
  milestone must force that distinction into types
- the bridge can observe enough canonical truth/view/route identity to build a
  replay-safe equivalence basis
- `worth-relational` commit strategies remain the only authority-owned way to
  turn a bridge effect into truth mutation
- causality tokens from truth commit through bridge routing and derived
  evaluation are available or can be lowered into canonical bridge artifacts
- loop prevention cannot depend on scheduler timing, sink pacing, or diagnostic
  retention policy
- no-op suppression must be semantic, not merely request-identity based
- the first shipped writeback classes should be narrow and structurally honest,
  not generic â€œarbitrary node can mutate arbitrary truthâ€

If any one of those assumptions is false in implementation, the spec must be
revised before code lands.

## Product Decision Lock

- The bridge remains an effect producer and coordinator, never a truth authority.
- Milestone 12 must not overload the existing speculation-only
  `BridgeRequestKind` with writeback meaning. If a writeback taxonomy needs a
  request-mode enum, it must be a new closed writeback-specific vocabulary.
- Read-only execution, derived-effect production, strategy admission, validated
  writeback candidate, no-op outcome, authoritative commit outcome, and replay
  bundle are distinct types.
- Causality is first-class in the writeback story: a bridge writeback artifact
  must preserve the triggering truth/route/evaluation basis as canonical data,
  not as diagnostics-only decoration.
- Every writeback-capable request must target an explicit registered relational
  strategy descriptor.
- Bridge-derived effect artifacts must remain first-class through replay and
  diagnostics; they may not disappear into a raw mutation batch immediately.
- Idempotence is a declared equivalence contract, not an inferred convenience.
- Canonical no-op writeback is an explicit outcome, not silent omission.
- Strategy failure, stale-basis rejection, invariant rejection, merge denial,
  replay mismatch, and authority bypass are distinct failure classes.
- Diagnostics richness may change retained detail, but not writeback meaning.

## Scope

In scope:

- bridge-owned writeback-capable request declaration
- bridge-owned derived-effect artifacts preserved through authority transfer
- explicit admission into registered relational commit strategies
- closed idempotence/no-op equivalence contracts
- typed writeback failure and denial taxonomy
- replay-safe writeback artifacts spanning effect production through authority
  outcome
- certification for suites 19 through 21 in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)

Out of scope:

- new relational authority semantics beyond the existing commit-strategy system
- new merge ontology or invariant semantics
- ambient adapter UX around writeback
- scheduler replacement inside `worth-signal`

## Governing Design Rules

### 1. One Declaration Surface Starts Every Writeback Story

There must be one bridge-owned declaration that states:

- request identity and request kind
- read-only versus writeback-capable intent
- truth-view basis and source-capability basis
- lowered policy digest
- bridge effect family
- targeted relational strategy identity
- declared idempotence class

No scattered booleans. No post hoc "should we persist this?" judgment.

This declaration should mirror existing bridge patterns such as
`BridgePolicyDeclaration`, `SourceDeclaration`, and
`BridgePreviewSessionDeclaration`: one canonical ingress artifact, one digest,
one identity, one validation/admission story.

### 2. Bridge Effect And Relational Strategy Stay Separate

At minimum the bridge must distinguish:

- `BridgeWritebackDeclaration`
- `BridgeDerivedEffectArtifact`
- `BridgeWritebackStrategyContract`
- `ValidatedBridgeWritebackCandidate`
- `BridgeWritebackAuthorityOutcome`
- `BridgeWritebackReplayBundle`

The bridge effect is not itself a relational mutation plan.

It should map more closely to the crate's existing declaration/contract/result
pattern than to the current `BridgeSignalEvaluationRequest`, which is a
delivery-side read artifact rather than an authority-boundary artifact.

### 3. Idempotence Is A Closed Equivalence Contract

Writeback equivalence must include at least:

- bridge request identity
- bridge effect digest
- truth-view basis digest
- lowered policy digest
- targeted strategy descriptor digest
- writeback schema/version digest
- declared idempotence class
- triggering causality digest

Equality on this basis may classify a request as canonical no-op where the
contract requires it. Mismatch must fail closed or force fresh authority
evaluation. Adapter retry state is not part of the contract.

### 3.1 Causality Must Be First-Class, Not A Debug Attachment

Every writeback-capable bridge story must preserve a canonical causality chain
that can answer:

- which truth commit or snapshot/view basis triggered the derived work
- which bridge routing or source materialization artifacts participated
- which derived evaluation surface produced the effect proposal
- which writeback declaration and strategy contract consumed that effect
- which authoritative outcome, no-op outcome, or failure terminated the story

At minimum, the milestone should introduce a canonical writeback causality
artifact or digest basis rather than relying on free-form provenance strings.

Rules:

- causality identity is part of writeback artifact identity
- causality identity is part of idempotence classification
- replay must reconstruct the same causality digest from canonical artifacts
- diagnostics may elaborate causality, but may not define it

### 3.2 Loop Prevention Must Be A Contract, Not A Hope

Milestone 12 must define an explicit anti-loop contract for bridge-mediated
writeback.

The minimum contract surface must answer:

- what makes a writeback-origin commit distinguishable from an ordinary truth
  commit at the bridge boundary
- when a bridge-origin commit is allowed to re-trigger derived nodes
- when a re-triggered derived result must collapse to canonical no-op rather
  than producing another authority attempt
- how the bridge/runtime prove that the same causal chain is not widening into
  unbounded self-triggered publication

The first shipped rule should bias toward fail-closed safety:

- if the bridge cannot prove the re-triggered writeback is semantically a no-op
  or belongs to an explicitly admitted convergent writeback class, it must not
  keep publishing authority attempts

### 4. Authority Boundary Consumes Lowered Decisions Only

The authority boundary may consume only admitted request kind, canonical bridge
effect, explicit strategy contract, closed idempotence basis, and validated
writeback candidate. It may not rediscover legality, widen permissions, or
infer strategy identity from output shape.

### 5. Failure Topology Must Stay Structured

Required minimum failure classes:

- `BridgeWritebackNotRequested`
- `BridgeWritebackPolicyRejected`
- `BridgeWritebackStrategyUnavailable`
- `BridgeWritebackStrategyDescriptorMismatch`
- `BridgeWritebackIdempotenceBasisMismatch`
- `BridgeWritebackStaleTruthBasis`
- `BridgeWritebackInvariantRejected`
- `BridgeWritebackMergeAuthorityRejected`
- `BridgeWritebackStrategyFailed`
- `BridgeWritebackStrategyPanicked`
- `BridgeWritebackReplayMismatch`
- `BridgeWritebackAuthorityBypassRejected`

Every writeback-capable request yields exactly one canonical no-op, canonical
authoritative commit, or typed failure.

## Complexity Contracts

Milestone 12 must name and prove boundedness for:

- writeback declaration admission
- effect-to-strategy admission
- idempotence classification
- loop-prevention classification
- writeback validation
- replay

Minimum counters:

- `writeback_request_count`
- `writeback_effect_width`
- `writeback_strategy_contract_count`
- `writeback_strategy_rejection_count`
- `writeback_idempotence_check_count`
- `writeback_causality_match_count`
- `writeback_loop_prevention_check_count`
- `writeback_loop_prevention_rejection_count`
- `writeback_noop_count`
- `writeback_commit_count`
- `writeback_failure_count`
- `writeback_authority_bypass_rejection_count`
- `writeback_validation_rejection_count`
- `writeback_replay_request_count`
- `writeback_replay_mismatch_count`

No implementation may scan prior bridge requests, all retained writeback
artifacts, or ambient retry state to decide current writeback legality.

## Phases

### Phase 1: Writeback Authority Model And Effect Contract

Define:

- explicit read-only versus writeback-capable request kinds
- bridge-derived effect declarations and artifacts
- canonical writeback causality artifacts and digest bases
- targeted relational strategy contracts
- closed idempotence basis
- explicit anti-loop contract and admitted convergent writeback-class vocabulary
- minimum writeback failure taxonomy
- authority-boundary rules separating bridge effect production from truth
  mutation authority

Phase 1 is complete only when read-only and writeback-capable requests are
mechanically distinct, bridge effects are first-class, causality is
first-class, loop-prevention assumptions are explicit, and idempotence meaning
is typed rather than narrative.

Current read: this phase is effectively implemented. Remaining work is mainly
regression pressure from later phases rather than missing substrate.

### Phase 2: Strategy Admission, Validation, And Authoritative Outcome

Implement:

- admission of canonical bridge effects into targeted relational strategies
- validation against truth-view basis, lowered policy, and strategy descriptor
- explicit no-op classification where required
- loop-prevention classification against triggering causality and declared
  convergent writeback class
- authoritative commit handoff through the ordinary relational strategy pipeline
- typed containment for strategy failure, panic, stale basis, invariant
  rejection, and merge denial

Phase 2 is complete only when identical admitted effect inputs lower to
identical candidate digests, no-op and commit outcomes are explicit, loop
checks are explicit, and failed strategies leave zero authoritative residue.

Current read: this phase is partially implemented. The runtime already carries
typed no-op, commit, rejection, failure, and panic boundaries, but the admitted
writeback strategy surface is still intentionally narrow.

Updated read: this phase is implemented for the first admitted strategy class.
The runtime now carries explicit strategy-class admission, validated
writeback-candidate artifacts, retry disposition, typed no-op/commit/rejection
outcomes, and typed failure containment through the authority seam. Remaining
work in this area is post-Phase-2 surface expansion, not missing lifecycle
plumbing.

### Phase 3: Replay, Diagnostics, And Certification

Ship:

- replay-safe writeback bundles spanning effect production through authority
  outcome
- diagnostics derived from canonical writeback artifacts rather than live
  adapter state
- harness certification proving bounded self-triggered behavior under hostile
  feedback scenarios
- certification suites 19 through 21
- proof tests for the named complexity contracts

Phase 3 is complete only when replay reconstructs writeback meaning from
canonical artifacts alone and diagnostics-tier variation changes retained detail
only, not writeback meaning.

Current read: this phase is implemented. Replay-safe bundles, hostile harness
lanes, standardized certification evidence, bridge-owned writeback execution
and replay records, canonical counter artifacts, and diagnostics/explanation
surfaces now ship as part of the runtime rather than as harness-local folklore.

## Must Ship

- one bridge-owned writeback declaration surface
- one bridge-owned derived-effect artifact family preserved through replay
- one canonical writeback causality artifact or digest basis
- explicit targeted relational strategy contracts
- a closed idempotence/no-op equivalence contract
- an explicit anti-loop classification contract
- explicit canonical no-op outcomes
- explicit authoritative writeback outcome records
- typed failure and denial classes
- bounded counters for writeback declaration, admission, idempotence,
  validation, outcome, and replay
- replay-safe writeback bundles and explanations
- certification satisfying Milestone 12 suites 19 through 21

## Must Preserve

- serialized authority for final truth mutation
- bridge remains an effect producer and coordinator, not a second commit path
- invariant enforcement and merge semantics remain authoritative in
  `worth-relational`
- read-only and writeback-capable flows remain mechanically distinct
- causality remains canonical from truth trigger through writeback outcome
- diagnostics richness changes retained detail only, not canonical writeback
  meaning
- retries and replay do not fabricate duplicate authoritative commits where the
  contract requires idempotence
- self-triggered writeback paths fail closed or converge through an explicit
  canonical no-op boundary rather than looping by accident

## Acceptance Evidence

Milestone 12 is complete only when the bridge harness can prove:

- repeated equivalent writeback-capable requests emit canonical no-op or
  identical authoritative outcomes according to the declared idempotence class
- writeback artifacts preserve a canonical causality digest linking truth
  trigger, bridge effect, strategy admission, and authority outcome
- bridge-origin effect artifacts remain distinguishable from read-only bridge
  execution artifacts and final authoritative commit artifacts
- strategy failure, panic, stale basis, invariant rejection, merge denial, and
  authority bypass each fail explicitly at the correct boundary
- failed or rejected writeback attempts leave zero authoritative residue
- hostile self-triggered feedback scenarios either terminate in canonical no-op
  or fail closed without unbounded publication
- replay reconstructs the same writeback admission and no-op-versus-commit
  decision and the same causality digest from canonical artifacts alone
- diagnostics-tier variation changes retained detail only, not writeback
  identity or failure class
- exact counters prove bounded idempotence checks, causality checks, and
  loop-prevention checks for representative certification scenarios
- certification suites 19 through 21 pass with canonical machine-checkable
  bundles

## Architectural Notes

Milestone 12 should extend the bridge crate with subdomains such as:

- `writeback/declaration.rs`
- `writeback/effect.rs`
- `writeback/causality.rs`
- `writeback/strategy.rs`
- `writeback/idempotence.rs`
- `writeback/loop_prevention.rs`
- `writeback/validation.rs`
- `writeback/outcome.rs`
- `writeback/replay.rs`
- `diagnostics/writeback.rs`

The current
[policy](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/policy)
and
[speculation](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/speculation)
subsystems are prerequisites, not substitutes. Milestone 12 should consume
their outputs rather than duplicate them.

Just as importantly, the current builder/runtime shape suggests a concrete
constraint:

- writeback should not be smuggled into `SignalBridgeSink`
- writeback should not be hidden inside `BridgeSourceAdapter`
- writeback likely needs its own builder-owned authority surface, parallel to
  existing source/sink registration, with freeze-at-construction semantics

The current codebase already favors:

- one facade method family per subsystem
- one closed taxonomy per milestone surface
- one admitted registry or contract boundary where legality matters
- one canonical diagnostics record family for replay/explanation
- one explicit performance/counter story for every claimed hot path

Milestone 12 should follow that pattern directly.

Expected facade growth should therefore look more like:

- `validate_writeback_declaration(...)`
- `admit_writeback_declaration(...)`
- `lower_writeback_effect(...)`
- `classify_writeback_loop_prevention(...)`
- `validate_writeback_candidate(...)`
- `replay_writeback_bundle(...)`

rather than one oversized `execute_writeback(...)` entrypoint that hides the
phase chain.

## Test And Harness Model

Milestone 12 is not closed by feature tests. It must satisfy the certification
discipline defined in
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md),
including:

- `control_lane`, `hostile_lane`, and `replay_lane`
- equality assertions for semantically equivalent lanes
- inequality assertions for intentionally different semantic lanes
- typed-failure assertions for rejected lanes
- zero-or-absence assertions for forbidden duplicate authority effects,
  forbidden residue, forbidden fallback, and forbidden loop widening
- offline-sufficient canonical bundles rather than log-based judgments

The harness should vary:

- request kind
- truth-view basis
- lowered policy bundle
- triggering causality basis
- writeback strategy identity
- idempotence class
- no-op versus changed-output basis
- self-triggering feedback shape
- failure injection point
- replay boundary placement
- diagnostics richness

Minimum certification outputs:

- `writeback_digest`
- `bridge_effect_digest`
- `causality_digest`
- `mutation_plan_digest`
- `idempotence_report`
- `loop_prevention_report`
- `truth_integrity_report`
- `authority_boundary_matrix`
- `failure_digest`
- `replay_digest`
- `counter_snapshot`
- `counter_artifact`

### Certification Rules For Milestone 12

Every Milestone 12 certification suite must:

- emit a canonical bundle sufficient for offline pass/fail analysis
  including a canonical counter artifact rather than only a sibling digest
- derive certification counters from retained bridge-owned writeback execution
  and replay records rather than scenario-local bookkeeping
- compare independently produced lanes rather than checking one run in isolation
- prove what must stay the same, what must change, and what must fail
- include exact counter assertions for representative scenarios, including
  counters that must remain zero
- treat self-triggered feedback pressure as a first-class hostile perturbation,
  not as an optional integration test

At minimum, the bundle must make it mechanically answerable:

- whether two equivalent writeback-capable lanes produced the same canonical
  no-op or commit truth
- whether an intentionally different effect basis produced a different outcome
- whether a rejected lane failed at the correct strategy, invariant, merge, or
  authority boundary
- whether diagnostics richness changed retained detail only
- whether loop-prevention and idempotence counters remained within contract

### Required Lane Shapes

#### Suite 19: Bridge Writeback Idempotence And Diff Truth

Required lanes:

- `control_lane`
  - one canonical writeback-capable request over stable truth basis
- `hostile_lane`
  - repeated equivalent requests, retry perturbation, and self-triggered
    feedback pressure over the same semantic effect
- `replay_lane`
  - replay/restart reconstruction of the same writeback story from canonical
    artifacts

Required assertions:

- equality:
  - control and hostile lanes compare equal when the effect is semantically
    equivalent and the declared idempotence class requires suppression
- inequality:
  - changed-output lanes must produce different `writeback_digest`,
    `bridge_effect_digest`, or `mutation_plan_digest`
- zero/absence:
  - duplicate authoritative mutation count must remain zero in equivalent retry
    lanes
  - forbidden fallback mutation count must remain zero
  - forbidden loop-widening count must remain zero
- typed failure:
  - if the declared writeback class is not convergent and the feedback pattern
    is unsafe, the hostile lane must fail explicitly rather than looping

Required hostile scenarios:

- repeated identical writeback request after success
- repeated identical writeback request after restart
- writeback-origin commit re-entering the same bridge/derived path
- changed-output versus unchanged-output comparison

#### Suite 20: Strategy Failure Containment

Required lanes:

- `control_lane`
  - successful writeback through the targeted strategy
- `hostile_lane`
  - failure injection during effect-to-strategy handoff, strategy execution,
    validation, or commit handoff
- `replay_lane`
  - replay of the failed or later-retried strategy-bearing path

Required assertions:

- equality:
  - later successful retry must compare equal to the no-failure control basis
- typed failure:
  - failure must localize strategy failure versus truth-boundary rejection
- zero/absence:
  - authoritative residue after failed writeback must remain zero
  - partial commit publication count must remain zero
  - loop-propagated retry publication count must remain zero

Required hostile scenarios:

- strategy returns typed failure
- strategy panics
- strategy descriptor is missing during replay or retry
- stale truth basis causes writeback rejection before publication

#### Suite 21: Authority Bypass Rejection

Required lanes:

- `control_lane`
  - one legal writeback-capable request through the admitted authority path
- `hostile_lane`
  - bypass attempts that try to skip strategy admission, invariant validation,
    merge authority, or commit authority
- `replay_lane`
  - replay of rejected bypass attempts and legal controls

Required assertions:

- typed failure:
  - each bypass attempt must fail at the correct declared boundary
- zero/absence:
  - authoritative mutation count must remain zero for every bypass lane
  - canonical commit artifact count must remain zero for every bypass lane
  - adapter fallback publication count must remain zero
- inequality:
  - legal authority-path outcomes must remain distinct from rejected bypass
    bundles

Required hostile scenarios:

- direct mutation-plan injection without strategy contract
- merge-bearing writeback without admitted merge legality
- invariant-skipping writeback attempt
- â€œpublish because preview already proved itâ€ shortcut

### Required Loop-Resistance Certification

Milestone 12 must add an explicit loop-resistance certification shape inside its
suite coverage even if the named suite labels remain 19 through 21.

Minimum hostile feedback matrix:

- same-causality same-effect retrigger
- same-causality changed-effect retrigger
- bridge-origin commit observed as a fresh truth trigger after restart
- interleaved ordinary truth commit plus bridge-origin writeback commit

Minimum required outcomes:

- canonical no-op for admitted convergent same-effect retriggers
- typed fail-closed rejection for unsafe retriggers
- no unbounded sequence of authoritative writeback publications
- replay preserving the same loop-prevention classification

Minimum exact counters:

- `writeback_idempotence_check_count`
- `writeback_causality_match_count`
- `writeback_loop_prevention_check_count`
- `writeback_loop_prevention_rejection_count`
- `writeback_noop_count`
- `writeback_commit_count`

For representative convergent scenarios, the spec should require exact expected
values rather than presence-only assertions.

## Anti-Patterns Explicitly Rejected

- writeback as a loose flag on ordinary bridge execution
- direct adapter-owned mutation publication from bridge outputs
- erasing bridge-origin effect semantics too early
- idempotence based on request identity alone
- no-op represented only by silence
- strategy failure falling back to raw mutations or partial publication
- preview artifacts treated as already-authoritative writeback input
- loop prevention by scheduler luck, debounce folklore, or diagnostics policy
- replay that proves only final truth shape and not bridge-origin authority
  transfer

## Sequencing Notes

Milestone 12 builds directly on:

- Milestone 10 speculative/preview boundary work
- Milestone 11 policy propagation
- relational Milestone 8.5 commit-strategy infrastructure

Milestone 12 must land before Milestone 12b, because the bridge first has to
prove one canonical writeback family before it can generalize writeback-family
extensibility honestly.

Milestone 12b must then land before Milestone 13 end-to-end certification,
because production-grade certification should test the bridge against a
bridge-native extensible writeback surface rather than a single-family special
case or host-local mapper folklore.

## Self-Check

- This solves a real structural problem: bridge-derived execution still lacks an
  honest authority-boundary contract.
- The adversarial constraint is load-bearing: duplicate commits, failing
  strategies, retry drift, bypass attempts, and self-triggered feedback loops
  are the real failure mode.
- Authority boundaries are preserved: the bridge proposes; `worth-relational`
  decides and commits.
- The milestone defines proof obligations, not chores: effect preservation,
  causality preservation, idempotence, loop resistance, failure containment,
  bypass rejection, and replay sufficiency are all machine-checkable.

## Closeout Standard

Milestone 12 is complete only when read-only and writeback-capable requests are
mechanically distinct, bridge-derived effect artifacts remain first-class
through replay, every writeback flow targets an explicit registered relational
strategy, idempotence and no-op publication are canonical, failed and rejected
flows leave zero authoritative residue, replay reconstructs writeback meaning
from canonical artifacts alone, and certification suites 19 through 21 pass.

If the bridge can still mutate truth outside relational strategy authority, if
equivalent retries can still fabricate duplicate authoritative commits, if
bridge-origin effect meaning or causality still disappears before replay, if
self-triggered writeback still depends on luck rather than explicit convergence
or fail-closed no-op classification, or if authority bypass remains visible
only in host logs rather than typed artifacts, Milestone 12 is not complete for
its first admitted family.

If the bridge still needs host-local shadow protocol logic to admit a second
writeback family honestly, that is Milestone 12b work, not a defect in the
Milestone 12 closeout statement above.
