# Milestone 12b Engineering Spec: Bridge-Native Extensible Writeback Families And Mapper Containment

> **Status:** Complete
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-12.md](./milestone-12.md)
>
> **Next milestone:** [milestone-13.md](./milestone-13.md)
>
> **Bridge certification companion:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** turn writeback family extensibility into a bridge-owned protocol surface so domains can admit multiple authority-bearing writeback families through one causal, replay-safe, loop-safe bridge contract without teaching domain semantics to the bridge and without pushing writeback protocol logic into host-local mappers

## Summary

Milestone 12 proved one certified writeback family. That is necessary, but it
is not the summit.

The bridge vision is not "one special writeback works." The vision is that
truth and derived computation can form one causal loop across many domains
without turning the bridge into a domain runtime.

Without Milestone 12b, the bridge still has a production-grade weakness:

- a second writeback family risks becoming a bridge-core edit rather than a
  bridge-native admission
- host mappers can quietly become shadow writeback protocols
- family-specific no-op, retry, replay, and loop semantics can drift outside
  the bridge boundary
- domain extensibility in `worth-relational` and `worth-signal` can exist while
  bridge-mediated writeback extensibility remains folkloric

Milestone 12b exists to close that gap before Milestone 13 asks the bridge to
be production-grade ready for end-to-end reference certification.

## Goal

Make extensible writeback families a first-class bridge protocol surface so
multiple domain-shaped writeback families can be admitted, replayed, diagnosed,
and certified through one bridge-owned causal contract while keeping domain
semantics in parent runtimes and keeping host mappers as translation rather
than shadow authority protocol.

## Why This Milestone Exists

Milestone 12 established the bridge-owned writeback protocol for one admitted
family. Milestone 13 expects the bridge to be production-grade ready for a
reference workload. That expectation is too strong if the bridge can still
write back only through one family-shaped lane or if a new family requires
host-local protocol invention.

Milestone 12b therefore belongs between 12 and 13 because it establishes the
missing truth:

- writeback-family identity must be explicit and bridge-admitted
- family-specific semantics must plug into bridge phases without bridge core
  rewrites
- host mappers must translate into family contracts, not smuggle in parallel
  protocol semantics
- replay, diagnostics, idempotence, and loop prevention must remain
  bridge-visible across families

Without this milestone, Milestone 13 risks certifying a single-family special
case rather than a production-grade bridge.

## Adversarial Constraint

Milestone 12b must survive the following hostile condition:

> Two or more domain-distinct writeback families with different effect shapes,
> candidate rules, retry dispositions, no-op semantics, merge or invariant
> boundaries, mapper implementations, diagnostics tiers, and replay boundaries
> must produce family-correct admission, family-correct no-op-versus-commit
> classification, family-correct failure localization, and family-correct replay
> identity without requiring bridge-core edits for each family and without
> allowing host mappers to redefine causality, idempotence, loop prevention,
> authority boundaries, or replay meaning outside the bridge contract.

If any path:

- encodes family meaning as opaque host payload rather than a bridge-admitted
  family contract
- requires bridge-core branching for each newly admitted family
- lets a mapper redefine no-op or retry semantics outside bridge-visible
  artifacts
- allows two families with different semantics to alias into the same replay or
  idempotence meaning
- hides family-specific authority or invariant denial inside host strings
- reopens the loop-prevention problem because family identity disappears at the
  authority boundary

then Milestone 12b has failed.

## Explicit Assumptions

Milestone 12b makes these assumptions explicit:

- the bridge should remain domain-agnostic, but not family-agnostic
- a writeback family is a bridge protocol concept, not just a relational or
  signal implementation detail
- host mappers are allowed to translate domain effects into bridge family
  contracts, but not to define bridge protocol semantics
- family-specific validation and lowering may differ, but causality, replay,
  diagnostics, and authority ownership remain bridge-owned
- the first multi-family milestone should prove at least two materially
  different admitted families rather than merely re-spelling one family twice
- family extensibility must remain compile-time and registry-admitted; it must
  not collapse into stringly typed or trait-object late binding

If any of these assumptions prove false, the bridge vision and roadmap should
be corrected before implementation continues.

## Product Decision Lock

- The bridge remains an effect producer and coordinator, never a domain
  authority.
- Writeback family identity is a first-class bridge concept.
- A host mapper may translate domain semantics into an admitted writeback
  family, but may not invent causality, idempotence, loop, or replay semantics
  outside bridge artifacts.
- A second admitted family must not require bridge-core special casing that
  bypasses the family registry or family contract path.
- Replay, diagnostics, and failure surfaces must remain family-aware and
  bridge-native.
- The bridge must distinguish family identity from strategy identity; a family
  may contain multiple strategies.
- Family-specific semantics must remain visible at the authority boundary; they
  may not disappear into opaque blobs before request/receipt recording.
- Invalid family lifecycle states must be unrepresentable: external code must
  not be able to construct admitted family contracts, admitted family
  candidates, or replay-valid family bundles without passing through the bridge
  proof chain.
- The bridge must emit native family admission, mapper, execution, and replay
  records as first-class decision-log artifacts; certification must consume
  those records rather than scenario-authored summaries.

## Scope

In scope:

- bridge-owned writeback-family taxonomy and admission
- sealed compile-time family registry and proof-bearing admission surface
- family-aware effect, candidate, outcome, replay, and diagnostics artifacts
- mapper-containment rules that keep host translation separate from bridge
  protocol ownership
- native family decision-log and record retention surfaces
- certification for multi-family parity, cross-family separation, and
  shadow-protocol rejection

Out of scope:

- teaching domain semantics to the bridge
- shipping all future writeback families for all domains
- replacing `worth-relational` commit authority or `worth-signal` derivation
  authority
- broad UI or workflow surfaces

## Phases

### Phase 1: Family Identity And Registry Boundary

Define and implement:

- a bridge-owned writeback-family taxonomy
- explicit family declaration and admission surfaces
- a sealed family registry and compile-time admitted family boundary
- family-aware effect and contract identity
- family-aware replay and diagnostics identity
- proof-bearing family types that make out-of-order or skipped family
  admission phases uncompilable
- lifecycle propagation such that adding family N+1 fails to compile until
  every required bridge boundary is updated

Phase 1 is complete only when a second admitted family can exist without
rewriting the bridge pipeline phases and without collapsing into stringly typed
host folklore.

### Phase 2: Family-Specific Lowering, Validation, And Authority Handoff

Implement:

- family-aware effect lowering and candidate validation
- family-specific no-op, retry, and compatibility semantics through one bridge
  phase chain
- authority request and receipt surfaces that preserve family identity
- typed failure mapping that keeps family-specific denials bridge-native
- mapper-containment rules ensuring host lowering remains translation, not
  shadow protocol
- a sealed mapper-output envelope whose fields cannot define replay identity,
  idempotence identity, loop disposition, or failure class
- one canonical family execution artifact at authority handoff, with all
  diagnostics and counters derived from it

Phase 2 is complete only when multiple admitted families can cross the same
bridge pipeline honestly while preserving family-distinct no-op, commit,
failure, and replay meaning.

### Phase 3: Multi-Family Certification And Shadow-Protocol Rejection

Ship:

- hostile certification suites for multi-family admission
- hostile certification for cross-family replay, idempotence, and loop parity
- hostile certification for shadow-protocol rejection and mapper containment
- canonical bundles proving that family identity remains visible from admission
  through authority outcome and replay
- compile-fail proof tests for undeclared families, skipped family admission
  phases, and illegal mapper outputs
- exact tracing and cost assertions for family lookup, mapper lowering, family
  dispatch, replay validation, and decision-log retention boundaries, derived
  from runtime-owned family records rather than scenario-authored totals

Phase 3 is complete only when the bridge can prove that new writeback families
enter through bridge-native contracts rather than through host-local folklore.

## Must Ship

- one bridge-owned writeback-family taxonomy
- one family admission boundary
- one family-aware writeback contract surface
- family-aware replay and diagnostics artifacts
- mapper-containment rules and typed rejection surfaces
- one canonical family execution artifact and one canonical family replay
  artifact, with all explanatory surfaces derived from them
- one native family decision-log record family:
  `WritebackFamilyAdmissionRecord`, `WritebackFamilyMapperRecord`,
  `WritebackFamilyExecutionRecord`, and `WritebackFamilyReplayRecord`
- at least two materially distinct admitted writeback families in harness-grade
  certification
- certification satisfying Milestone 12b suites 22 through 24

## Must Preserve

- truth authority remains in `worth-relational`
- derived execution authority remains in `worth-signal`
- bridge-owned causality, idempotence, loop, and replay semantics remain
  visible and canonical
- host mappers remain translation layers rather than shadow authority protocol
- adding a new family does not require per-family bridge-core orchestration
  branches
- adding a new family must fail to compile at each unwired lifecycle boundary
  until the boundary is explicitly updated
- family tracing must remain queryable from retained native records alone

## Acceptance Evidence

Milestone 12b is complete only when the bridge harness can prove:

- two distinct admitted writeback families can coexist without aliasing replay
  or idempotence identity
- family-specific no-op, commit, retry, and failure meaning remain typed and
  bridge-visible
- host mapper variation that preserves family semantics yields equal canonical
  bundles
- host attempts to smuggle undeclared or opaque family semantics fail
  explicitly before authority execution
- cross-family lookalike outputs remain mechanically distinct where semantics
  differ
- loop-prevention and authority-boundary evidence remain family-aware and
  replay-safe
- compile-fail tests prove that undeclared families, skipped proof phases, and
  illegal mapper outputs are unrepresentable or uncompilable
- offline observers can reconstruct family admission, mapper lowering,
  authority handoff, replay validation, and exact decision boundaries from
  retained native records alone
- the Milestone 12b certification suites in `test-requirements.md` pass with
  canonical machine-checkable bundles

## Architectural Notes

Milestone 12b should extend the bridge crate with subdomains such as:

- `writeback/family.rs`
- `writeback/family_contract.rs`
- `writeback/family_registry.rs`
- `writeback/family_mapper.rs`
- `writeback/family_compatibility.rs`
- `writeback/family_record.rs`
- `writeback/family_replay_record.rs`
- `diagnostics/writeback_family.rs`

The bridge should own the family protocol, not the family domain semantics.
That means the registry and contract surfaces should carry proofs about family
identity, effect shape, and replay identity while leaving actual domain meaning
to admitted strategies and mappers outside the bridge core.

The registry must be sealed and compile-time exhaustive. "Extensible" in this
milestone does not mean "string-keyed plugin map at runtime." It means the
bridge can admit multiple materially different family implementations through
one proof-bearing family protocol without rewriting bridge orchestration.

Mapper outputs must also be structurally constrained. A mapper may supply
domain-local payload, bridge-declared family inputs, and domain evidence needed
for family lowering. A mapper must not be able to author:

- replay identity
- idempotence identity
- loop-prevention disposition
- failure class
- authority-boundary classification

Those remain bridge-owned derivations from admitted family contracts and
bridge-owned phase outputs.

Canonical artifact ownership must be explicit:

- one `WritebackFamilyExecutionRecord` is the canonical authority-boundary
  artifact for executed family writeback
- one `WritebackFamilyReplayRecord` is the canonical replay-validation artifact
- explanations, digests, counters, and certification bundles are all derived
  views over retained native records rather than parallel sources of truth

Tracing requirements are first-class. Family admission, mapper lowering,
family candidate validation, authority handoff, authority receipt validation,
and replay validation must all emit span-aware decision entries with O(1)
lookup by canonical family record identity.

The implementation should map to an explicit proof chain rather than a bag of
helpers. The intended family-bearing phase sequence is:

- `DeclaredWritebackFamily`
- `ValidatedWritebackFamily`
- `AdmittedWritebackFamily`
- `MappedWritebackFamilyInput`
- `LoweredWritebackFamilyEffect`
- `ValidatedWritebackFamilyCandidate`
- `ExecutedWritebackFamily`
- `ReplayedWritebackFamily`

Each phase type should consume the prior proof-bearing type and produce the
next. Constructors for these types should be crate-sealed, fields should remain
private, and the facade should expose only read-only views over established
proofs. External code must not be able to synthesize later family phases
without traversing the proving functions.

Family identity should also be represented as a sealed bridge-native type, not
as a string descriptor. The intended shape is a closed family declaration
surface with family-local strategy space attached beneath it. In practice that
means:

- one sealed `BridgeWritebackFamilyKind`
- one sealed `BridgeWritebackFamilyDeclaration`
- one family-aware `BridgeWritebackFamilyContract`
- one family-aware `BridgeWritebackFamilyStrategyClass`

The bridge should be able to distinguish:

- "this is the projected-state-diff reconciliation family"
- "this family has strategy classes A and B"
- "this request belongs to strategy A of that family"

without flattening those meanings into one descriptor field.

Mapper containment should also be encoded in types. The intended split is:

- a domain-owned mapper input type
- a bridge-owned `BridgeWritebackFamilyMapperOutput`
- a crate-sealed bridge-owned witness proving that the output came from an
  admitted family mapper path

The mapper output should carry only:

- admitted family kind
- admitted family strategy class
- domain payload needed for bridge family lowering
- domain evidence declared by the family contract

It should not carry any bridge-owned equivalence or authority classifications.
Those must be derived later by bridge phases so the type system preserves the
"mapper translates, bridge classifies" boundary.

Native record ownership should also map to concrete code surfaces rather than
free-floating JSON. The intended record family is:

- `WritebackFamilyAdmissionRecord`
- `WritebackFamilyMapperRecord`
- `WritebackFamilyExecutionRecord`
- `WritebackFamilyReplayRecord`

Each record should contain:

- canonical record identity
- family kind and strategy class
- causality linkage
- parent record linkage where applicable
- typed decision entries
- boundary-specific counters
- diagnostics-policy-independent semantic digests

The diagnostics layer should derive explanations and bundles from these native
records instead of re-summarizing execution state through separate ad hoc
structs.

Compile-time enforcement should be part of the design, not just the test plan.
This milestone should expect:

- `pub(crate)` constructors for all proof-bearing family types
- sealed traits for family registration and mapper admission
- compile-fail tests for skipped family phases
- compile-fail tests for missing lifecycle propagation when a new family is
  added
- compile-fail tests for mapper outputs that attempt to set bridge-owned
  semantic fields

Performance and traceability must stay explicit at the family boundary. The
family admission and execution hot path should therefore declare exact counter
surfaces for:

- family lookup count
- family dispatch count
- mapper lowering count
- family candidate validation count
- family execution record append count
- family replay validation count
- decision-log entry count

These counters belong in the native records and in the derived certification
bundles. They should be derived from the runtime-observed execution and replay
artifacts, not authored as scenario bookkeeping.

Expected facade growth should therefore look more like:

- `validate_writeback_family(...)`
- `admit_writeback_family(...)`
- `lower_family_writeback_effect(...)`
- `validate_family_writeback_candidate(...)`
- `replay_family_writeback_bundle(...)`

rather than ad hoc per-domain execution entrypoints.

### Implementation Map Against Current Crate Shape

Milestone 12 already established a real writeback chain in the current crate:

- `writeback/declaration.rs`
- `writeback/validation.rs`
- `writeback/contracts.rs`
- `writeback/effect.rs`
- `writeback/idempotence.rs`
- `writeback/loop_prevention.rs`
- `writeback/strategy.rs`
- `writeback/strategy_compatibility.rs`
- `writeback/candidate.rs`
- `writeback/outcome.rs`
- `writeback/execution.rs`
- `writeback/replay.rs`
- `writeback/replay_record.rs`
- `facade/runtime/writeback.rs`

Milestone 12b should not create a parallel writeback stack. It should extend
that existing stack so family proof becomes part of the same canonical phase
chain.

The intended implementation split is:

- `writeback/taxonomy.rs`
  Add `BridgeWritebackFamilyKind` and any family-local retry or lowering
  taxonomy that must remain bridge-owned rather than mapper-owned.
- `writeback/declaration.rs`
  Extend declarations so writeback-capable declarations must bind family kind
  as well as strategy class. Read-only declarations must remain unable to bind
  family identity.
- `writeback/validation.rs`
  Produce `ValidatedWritebackFamily` proof alongside declaration validation.
  This is where undeclared or illegal family bindings should fail before
  contract admission.
- `writeback/contracts.rs`
  Admit only sealed family declarations and emit the first bridge-owned
  family contract proof.
- `writeback/strategy.rs`
  Keep strategy identity subordinate to family identity. Strategy basis should
  become family-aware instead of being the highest writeback-specific identity.
- `writeback/effect.rs`
  Lower effects from admitted family contract plus mapper witness, not from raw
  host-provided semantics. Family-aware effect identity should be owned here.
- `writeback/idempotence.rs`
  Scope idempotence identity by family kind so lookalike effects from different
  families cannot alias.
- `writeback/loop_prevention.rs`
  Preserve family visibility in loop classification and feedback provenance.
- `writeback/strategy_compatibility.rs`
  Become explicitly family-aware so compatibility is checked first at the
  family boundary and then at the strategy boundary.
- `writeback/candidate.rs`
  Consume only family-aware effects and family-aware compatibility proofs, and
  emit a family-aware validated candidate.
- `writeback/execution.rs`
  Either absorb or be split into `family_record.rs` so the canonical execution
  record becomes the native `WritebackFamilyExecutionRecord` rather than a
  family-blind writeback record with extra fields.
- `writeback/replay.rs` and `writeback/replay_record.rs`
  Preserve family identity in replay semantics and native replay retention.
- `facade/runtime/writeback.rs`
  Remain the single orchestration path, but switch to family-bearing proof
  types and family-bearing native record emission.

The cleanest path is evolutionary, not revolutionary:

1. Introduce family-bearing proof types into the current modules.
2. Rename or wrap current execution/replay record types into explicit family
   record names if needed.
3. Add mapper witnesses and sealed mapper outputs.
4. Only after the proof chain compiles cleanly, widen to a second admitted
   family.

This milestone should also prefer extending current types over duplicating them
when the lifecycle is structurally identical. If an existing type already
represents a phase and only lacks family proof, the fix is to parameterize or
enrich it with family-bearing proof rather than creating a second type that
duplicates the same phase semantics under a new name.

## Sequencing Notes

Milestone 12b builds directly on:

- Milestone 12 first-family writeback protocol
- relational extensible commit strategy infrastructure
- signal-side extensible derived execution surfaces

Milestone 12b must land before Milestone 13 because end-to-end reference
certification should test a bridge-native extensible writeback architecture,
not a one-family special case plus host glue.

## Self-Check

- This solves a real structural problem: bridge-native writeback extensibility
  is missing even though parent runtimes are already extensible.
- The adversarial constraint is load-bearing: multi-family admission, mapper
  containment, and cross-family replay separation are the real production-scale
  failure modes.
- Authority boundaries are preserved: domains translate into admitted families;
  the bridge carries protocol; parent runtimes own semantics and authority.
- The milestone defines proof obligations, not chores: cross-family separation,
  mapper containment, replay parity, and shadow-protocol rejection are all
  machine-checkable.

## Closeout Standard

Milestone 12b is complete only when the bridge can admit multiple writeback
families through one bridge-owned protocol boundary, preserve family identity
through causality, validation, authority outcome, diagnostics, and replay, and
reject host-local shadow protocols that try to bypass that boundary.

If a second family still requires bridge-core special casing, if host mappers
still define bridge protocol semantics outside canonical artifacts, or if
family-specific writeback meaning still disappears before replay, Milestone 12b
is not complete.
