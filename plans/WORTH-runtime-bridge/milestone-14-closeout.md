# Milestone 14 Closeout: Bridge-Native Subscription Declaration Families, Admission, And Lifecycle

## Status

Milestone 14 is complete.

As of 2026-04-21, `worth-runtime-bridge` has one bridge-owned subscription
protocol shell that a manual host can use without reconstructing bridge
meaning from host-local callback glue.

The semantic center that shipped is:

one frozen bridge-native subscription declaration-family registry can
canonicalize equivalent declarations, admit them against explicit
snapshot-bound or branch-head truth basis, lower them into canonical admitted
`worth-signal` strategy descriptors, materialize activation-ready and
deactivated retained lifecycle artifacts, explain admitted and rejected paths
from bridge-owned diagnostics records, and replay retained subscription meaning
from canonical bundle artifacts without ambient host state.

Milestone 14 is therefore closed as a bridge-native declaration and admission
substrate, not as a delivery or fanout milestone.

## Milestone Objective

Milestone 14 existed to turn subscriptions from an implicit assembly of slices,
streams, and observer handles into one bridge-owned protocol surface with:

- canonical declaration families
- explicit basis-bound admission
- explicit lowering into admitted `worth-signal` strategies
- bridge-owned retained diagnostics
- bridge-owned retained replay and lifecycle artifacts

The objective was not to ship live delivery, sharing, continuation, checkpoint
resume, or store durability. Those remain later-milestone work.

## Phase-By-Phase Implementation Summary

### Phase 1: Canonical Declaration Families And Registry Freeze

Phase 1 shipped:

- a dedicated `subscription/` subdomain
- closed Milestone 14 declaration families:
  - `DetailExact`
  - `CollectionMembership`
- canonical family registry freeze with stable registry identity
- canonical declaration artifacts with normalized slice-intent ordering and
  deduplication
- typed declaration rejections
- compile-time privacy for internal construction paths

The declaration boundary now proves that semantically equivalent declarations
collapse to one canonical declaration artifact independent of builder order.

### Phase 2: Basis Admission And Signal Strategy Lowering

Phase 2 shipped:

- explicit subscription basis requests for:
  - `Snapshot`
  - `BranchHead`
- validated basis bindings with typed basis-resolution failures
- branch-head and snapshot proof checks that reject misbinding explicitly
- admitted subscription artifacts with canonical admitted identity
- closed-world lowering into admitted signal strategy descriptors:
  - `ExactFieldLensObservation`
  - `CollectionMembershipObservation`

This phase closed the boundary between declaration meaning and runtime truth
authority without inventing a second basis universe or a second observation
runtime.

### Phase 3: Lifecycle Shell, Diagnostics, And Replay

Phase 3 shipped:

- activation-ready retained lifecycle handles
- deactivated retained lifecycle handles
- canonical lifecycle records
- retained explanation artifacts for:
  - activation-ready subscriptions
  - deactivated subscriptions
  - admission rejections
- retained subscription bundles for replay
- replay summaries and typed replay mismatches
- compile-fail privacy tests for lifecycle and replay construction

This phase intentionally stops before live observer registration and shared
consumer delivery. The lifecycle ceiling for Milestone 14 is:

- declaration artifact
- admitted artifact
- activation-ready handle
- deactivated handle
- typed rejection artifact

## Major Design Decisions

- Subscription meaning is family-aware. Milestone 14 does not pretend one
  universal subscription shape is sufficient.
- The bridge owns declaration and admission semantics. `worth-signal` still
  owns execution semantics.
- Family registration is frozen before declaration admission. No admitted
  declaration depends on mutable runtime registration order.
- Basis binding is explicit. Milestone 14 binds either:
  - explicit snapshot identities
  - branch-head requests
- Historical retained snapshots can participate only when presented as explicit
  snapshot identities. Milestone 14 does not yet ship a distinct historical
  subscription basis kind.
- Preview basis admission is deferred. The docs were corrected at closeout so
  the shipped milestone does not overclaim preview subscription support.
- Lifecycle is retained and protocol-shaped, not delivery-shaped. Activation
  readiness is a bridge artifact, not hidden signal registration.

## Adversarial Constraints Addressed

The shipped implementation now survives the main naive-failure modes that made
Milestone 14 necessary:

- declaration order variation does not change canonical declaration identity
- duplicate slice intents normalize away without semantic drift
- family semantics are frozen and canonicalized before admission
- snapshot basis proof rejects misbound snapshot readers
- branch-head basis proof rejects misbound branch-head sources
- unsupported basis combinations fail before strategy lowering
- selected signal strategy is retained as a bridge-visible artifact
- replay reconstructs retained subscription meaning from canonical bundle
  artifacts without host-local callback state
- compile-time boundaries prevent external code from fabricating admitted,
  lifecycle, or replay artifacts

## Tests Added Or Strengthened

The milestone now has focused declaration/admission/lifecycle coverage in
[subscription.rs](/Users/shepworth/Documents/programming/WORTH/crates/worth-runtime-bridge/src/facade/tests/subscription.rs)
plus direct replay-tamper coverage in
[replay.rs](/Users/shepworth/Documents/programming/WORTH/crates/worth-runtime-bridge/src/subscription/replay.rs),
suite-shaped certification coverage in
[subscription_certification.rs](/Users/shepworth/Documents/programming/WORTH/crates/worth-runtime-bridge/src/harness/tests/subscription_certification.rs),
and compile-fail privacy coverage in
[/Users/shepworth/Documents/programming/WORTH/crates/worth-runtime-bridge/tests/ui](C:\Users\shepworth\Documents\programming\WORTH\crates\worth-runtime-bridge\tests\ui).

Key proof lanes include:

- canonical declaration equivalence across ordering variation
- family registry identity stability
- typed rejection for unsupported family/slice combinations
- snapshot-basis admission success
- branch-head-basis admission success
- typed rejection for missing snapshot basis
- typed rejection for snapshot misbinding
- typed rejection for branch-head misbinding
- declaration equivalence parity across diagnostics-tier variation
- explicit suite-shaped certification for Milestone 14 suites 28 through 30
- activation-ready lifecycle preparation
- deactivation lifecycle transition
- retained bundle replay success
- retained bundle replay rejection under registry drift
- retained bundle replay rejection under admitted-declaration tampering
- retained bundle replay rejection under lifecycle/admitted mismatch
- compile-fail denial of external lifecycle/replay construction

Verification baseline at closeout:

- `cargo test -p worth-runtime-bridge subscription -- --nocapture`
- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail -- --test-threads=1`
- `cargo test -p worth-runtime-bridge`

## Major QA Findings And How They Were Resolved

The hostile QA loop found and resolved several real structural problems before
closeout.

Resolved declaration-phase findings:

- dead caller-supplied declaration identity input removed from the public API
- family slice-kind semantics canonicalized and deduplicated before registry
  identity is computed
- malformed family capability metadata rejected at the freeze boundary
- explicit structural counters added for declaration and registry work

Resolved admission-phase findings:

- snapshot basis proof now verifies the bound snapshot identity instead of
  trusting acquisition alone
- the fake `CurrentSnapshot` versus `HistoricalSnapshot` split was collapsed
  into one honest `Snapshot` basis kind
- basis-resolution failures are typed and preserved, not collapsed into
  strings
- branch-head admission now verifies that the returned patch belongs to the
  requested branch
- counter constructors were corrected so stage accounting reflects actual
  work performed

Resolved lifecycle-phase findings:

- rejection explanations now retain typed admission rejection kind rather than
  dropping denial topology on the diagnostics path
- milestone spec and certification docs were corrected so Milestone 14 no
  longer overclaims preview, quiescent, or distinct historical-basis behavior

## Residual Risks Or Deferred Items

Milestone 14 is complete, but several deliberate deferrals remain:

- no live observer registration
- no shared subscription fanout
- no continuation across identity evolution
- no subscription checkpoint/resume
- no preview-scoped subscription basis admission
- no distinct quiescent or paused lifecycle handle
- no store-backed durability for retained subscription bundles
- no Query-owned lowering yet

These are not open defects in Milestone 14. They are Milestone 15 and 16 work.

The main boundary to preserve going forward is:

- Milestone 15 may add delivery, sharing, continuation, and preview behavior
  on top of the retained artifacts from Milestone 14
- it must not reopen declaration identity, basis admission honesty, or
  signal-strategy selection as ambient runtime behavior

## Overall Assessment

Milestone 14 meets its implementation spec once the spec is read with the
correct shipped ceiling:

- declaration families are canonical and replay-safe
- basis admission is explicit and typed for snapshot-bound and branch-head
  truth views
- signal strategy lowering is bridge-visible and closed-world
- lifecycle artifacts are retained, phase-typed, and facade-owned
- diagnostics explain both success and denial paths from bridge-owned records
- replay reconstructs retained meaning without ambient host state
- compile-time privacy boundaries hold
- performance posture is honest enough for this milestone because the key
  control-plane counters, lookup boundaries, and freeze boundaries are now
  explicit

Milestone 14 is therefore closed as the bridge-native declaration, admission,
and retained lifecycle substrate for first-class subscriptions.
