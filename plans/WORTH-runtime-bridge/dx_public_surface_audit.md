# WORTH Runtime Bridge DX Public Surface Audit

## Purpose

This document records the current public-surface classification for
`worth-runtime-bridge` after the main DX hardening passes on the facade and
standard-path workflows.

It exists to answer one concrete question:

What, exactly, is the bridge currently treating as:

- canonical
- advanced
- specialist
- compatibility-only baggage

This is the bridge's public memory audit.

It should be updated whenever we materially change the facade shape.

---

## Audit Input

This audit is based on:

- [`crates/worth-runtime-bridge/src/facade.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade.rs)
- [`crates/worth-runtime-bridge/src/facade/request.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade/request.rs)
- [`crates/worth-runtime-bridge/src/facade/standard_path.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade/standard_path.rs)
- [`crates/worth-runtime-bridge/src/facade/runtime`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade/runtime)
- the current public docs spine under
  [`crates/worth-runtime-bridge`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge)

---

## Bottom-Line Read

The bridge now has a real canonical surface.

That is new and important.

The public shape is no longer only:

- a flat export wall
- a subsystem inventory
- a protocol-phase transcript

It is now substantially closer to:

- one authoritative facade path
- one standard-path category inside that facade
- one explicit-control category inside that facade
- one specialist category inside that facade

But the top-level `facade` still remains broad for compatibility.

That means the bridge is in a mixed state:

- product shape is now deliberate
- compatibility sprawl still exists underneath it

That is acceptable during transition.
It is not the final desired state.

---

## Class A: Canonical Surface

These are the surfaces that now define the bridge product memory model.

### Imports

- `worth_runtime_bridge::facade`
- `worth_runtime_bridge::facade::RuntimeBridge`
- `worth_runtime_bridge::facade::RuntimeBridgeBuilder`

### Setup

- `RuntimeBridge::builder()`
- builder aliases such as `with_truth_source(...)`
- builder aliases such as `with_compute_sink(...)`
- `build()`

### Route And Evaluate

- `BridgeRouteRequest`
- `RuntimeBridge::route(...)`
- `RuntimeBridge::evaluate_current(...)`
- `RuntimeBridge::evaluate(...)`
- `BridgeTruthViewEvaluationRequest`
- `BridgeRoute`
- `BridgeEvaluationTarget`
- `BridgeTruthViewEvaluation`

### Speculate

- `RuntimeBridge::speculate(...)`
- `BridgeSpeculativeSessionRequest`
- `BridgeSpeculativeSessionHandle`
- `BridgeSpeculativeSessionHandle::compare_to_main()`
- `BridgeSpeculativeSessionHandle::discard(...)`
- `BridgeSpeculativeSessionHandle::promote(...)`
- `BridgeSpeculativeComparison`
- `BridgeSpeculativePromotionRequest`

### Diagnostics

- `RuntimeBridge::diagnostics()`
- `BridgeDiagnostics`
- `BridgeDiagnostics::explain_last()`
- `BridgeDiagnostics::explain_route(...)`
- `BridgeDiagnostics::explain_evaluation(...)`
- `BridgeDiagnostics::explain_session(...)`
- `BridgeDiagnostics::explain_promotion(...)`

### Verdict

This Class A surface is now strong enough to be treated as the public bridge
identity.

Milestone 13 everyday integration lanes should continue to target this class.

---

## Class B: Advanced But Supported

These surfaces are real product lanes, but they should not define first-read
memory.

### Policy And Runtime Control

- policy declaration admission and lowering
- policy provenance and replay-bundle summarization
- route planning policy projection
- explicit runtime policy inspection

### Truth-View And Historical Control

- explicit truth-view selectors
- snapshot and historical commit targeting
- continuity delivery and lineage resolution

### Source Materialization

- source declaration admission
- source packet planning
- source packet materialization
- source packet-set canonicalization

### Stream Coordination

- stream declaration validation
- consumer contract resolution
- window planning
- checkpoint publication
- checkpoint resume
- replay-audit delivery

### Structural Comparison

- structural declaration admission
- structural fingerprint materialization
- structural packet planning
- structural reduction
- remap publication
- branch comparison publication

### Merge

- merge declaration admission
- merge lowering
- merge routing reduction
- continuity/remap/explanation publication
- merge replay bundle reconstruction

### Writeback

- admitted writeback contracts
- writeback effect lowering
- idempotence and loop-prevention classification
- strategy compatibility classification
- explicit authority execution

### Verdict

This Class B surface is now coherent enough to document as advanced.

It should be taught after the standard path, and it should gain compile-checked
examples before we call bridge DX finished.

---

## Class C: Specialist Surface

These remain public because they are genuinely useful for replay, proof,
certification, and adapter-authoring work.

### Raw Phase Verbs

- `validate_*`
- `admit_*`
- `lower_*`
- `canonicalize_*`
- `replay_*`
- `publish_*`
- `reduce_*`

### Canonical Records And Replay Artifacts

- canonical route/history/continuity/merge/structural records
- replay bundles
- retained diagnostics records
- certification bundle internals

### Proof And Identity Machinery

- ontology mapping and precedence structures
- structural fingerprint details
- writeback mapper envelopes and mapped family inputs
- stream replay record internals
- continuity authority internals

### Debug/Registry Inspection

- direct registry accessors
- direct host-adapter accessors
- direct writeback-authority accessors

### Verdict

These are legitimate specialist surfaces.

The DX problem is no longer that they exist.
The remaining DX requirement is that they stay clearly secondary in docs,
examples, and tests.

---

## Class D: Compatibility-Heavy Top-Level Surface

The broad top-level `facade` re-exports are still the main compatibility
pressure point.

They currently keep many advanced and specialist nouns visible at the same
level as the canonical path.

This is why the bridge can still feel denser than Laravel/Angular-grade API
products even though the underlying layering is now much better.

### Current Decision

Keep the broad export surface for now.

Reason:

- tests
- examples
- internal consumers
- transition safety

### Required Constraint

Do not let these broad re-exports redefine the bridge story.

The product identity should continue to be taught through:

- `everyday`
- `advanced`
- `specialist`

not through flat `facade::*` memory.

---

## Remaining DX Gaps

The main remaining gaps are now specific.

### 1. Advanced Examples Need To Catch Up

The bridge has compile-checked everyday examples.

It still needs stronger compile-checked examples for:

- policy control
- source materialization
- stream checkpoint/resume
- structural remap and branch comparison
- merge replay or explanation flow

### 2. Compatibility Surface Is Still Too Loud

The top-level `facade` remains broad.

This is survivable, but it still means a user can discover the bridge in the
wrong order.

### 3. Some Raw Phase Families Are Still Publicly Dense

This is acceptable if they stay specialist.

It becomes a problem again if:

- Tier 1 docs use them
- ordinary integration tests use them
- new examples teach them first

---

## Completion Test

The bridge DX hardening can be considered substantially complete when:

1. everyday flows stay stable under Milestone 13 workload pressure
2. advanced flows have compile-checked examples and coherent guide coverage
3. specialist flows remain public but clearly secondary
4. no new ordinary tests are added against raw phase APIs
5. the broad top-level facade is no longer the dominant teaching surface

We are close to that state.
We are not fully there yet.
