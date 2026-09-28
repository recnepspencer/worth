# WORTH Runtime Bridge DX Boundary Spec

## Purpose

This document defines the intended public boundary for `worth-runtime-bridge`
before the main Milestone 13 implementation phase.

It answers:

- what should feel primary
- what should remain available but contained
- what should count as specialist infrastructure
- what should not define the bridge product identity

This is the full bridge boundary target.

It is immediately relevant to Milestone 13, but it is not intentionally capped
at the minimum needed to unblock that milestone.

---

## Daily-Use Principle

The bridge public API should optimize for these jobs first:

1. build bridge
2. route truth changes
3. evaluate against a truth view
4. open a speculative session
5. discard or promote it
6. inspect what happened

If those jobs are not smooth, the bridge boundary is not ready, regardless of
how strong its deep subsystem artifacts already are.

---

## Canonical Public Layers

## Layer 1: Primary Bridge Story

This is what should define the product.

Required characteristics:

- short import story
- one obvious setup path
- one obvious route or evaluate path
- one obvious speculation path
- one obvious diagnostics door

## Layer 2: Advanced Runtime Control

This is for:

- policy refinement
- branch and historical view selection
- bulk planning
- structural comparison
- merge-aware reads
- advanced writeback configuration

Required characteristics:

- explicit
- coherent
- discoverable after Layer 1

## Layer 3: Specialist Infrastructure

This is for:

- certification bundle production
- replay and forensic inspection
- host-adapter authoring
- merge and structural proof surfaces
- detailed writeback family mechanics

Required characteristics:

- real
- public where required
- clearly specialist
- not the first thing users have to learn

## Layer 4: Not The Product Boundary

This does not define the bridge product.

Includes:

- test-only harness substrate
- support-only helpers
- milestone-local scaffolding that does not correspond to a real bridge job

---

## Final Intended Top-Level Identity

The bridge should converge toward:

- `worth_runtime_bridge::facade`
- `RuntimeBridgeBuilder` as the obvious setup door
- `RuntimeBridge` as the obvious execution door
- one diagnostics entrypoint for inspection and certification readback
- clear guided request and session types for ordinary bridge jobs

Everything else should either reinforce that shape or be explicitly contained.

---

## Boundary Policy For Existing Surface Families

## Primary

These should feel primary:

- `RuntimeBridgeBuilder`
- `RuntimeBridge`
- guided route requests
- guided truth-view evaluation requests
- guided speculative session requests
- guided discard and promote operations
- one diagnostics facade or entrypoint

## Advanced But Contained

These should remain public but should not define day-one bridge memory:

- policy declarations and lowering surfaces
- snapshot and truth-view selector detail
- stream protocol detail
- bulk planning detail
- structural comparison detail
- merge interpretation detail
- writeback family and strategy detail

## Specialist

These remain real and public where required, but they should be explicitly
specialist:

- canonical record schemas
- replay bundle detail
- fine-grained certification records
- proof-bearing or family-bearing writeback internals
- merge ontology and precedence detail
- structural fingerprint equivalence machinery

## Not The Product Boundary

These must not define the bridge product identity:

- raw test harness substrate
- fixture-only helper seams
- support-only certification assembly paths

---

## Concrete Boundary Rules

### 1. `RuntimeBridgeBuilder` Is The Setup Door

Users should not need to memorize multiple equally primary setup entrypoints.

The setup story should be:

- create builder
- register sources and sinks
- refine policy or diagnostics
- build

### 2. `RuntimeBridge` Is The Execution Door

Ordinary bridge work should pass through `RuntimeBridge`.

Milestone 13 tests should not bypass it for:

- route execution
- truth-view evaluation
- speculative session execution
- discard or promote operations

unless the spec explicitly says the called surface is specialist and the test is
certifying that specialist surface itself.

### 3. Diagnostics Must Have One Obvious Door

The bridge has many diagnostics records.
That is fine.

What is not fine is requiring users to choose their entrypoint by subsystem
guesswork.

The public story must have one diagnostics door that leads to:

- explanation
- replay
- comparison
- certification bundle access

### 4. Record Richness Must Not Dominate Everyday Usage

The bridge has a large number of valuable records, digests, and explanations.

They should remain available.
They should not be the first thing a normal caller sees when trying to route a
truth change or open a speculative session.

### 5. Milestone 13 Reference Tests Must Follow The Boundary

The pricing-shock reference workload should be treated as the boundary audit.

If the reference workload can only be written by reaching directly into raw
subsystem export surfaces for ordinary jobs, the boundary is not yet honest.

### 6. Milestone 12b Must Count As Baseline, Not As Specialist Noise

Bridge-native extensible writeback families and mapper containment are now part
of the real bridge product baseline after
[`milestone-12b.md`](./milestone-12b.md).

That means the boundary must make room for:

- family-aware promotion flows
- family-aware diagnostics readback
- family-aware specialist containment

without forcing ordinary users to learn the full family protocol to do common
bridge work.

---

## Release Test For Boundary Quality

The bridge boundary is good enough for the Milestone 13 implementation push
only if a user or AI agent can answer these immediately:

1. Where do I build the bridge?
2. Where do I route a truth change?
3. Where do I evaluate against a truth view?
4. Where do I open a speculative session?
5. Where do I discard or promote it?
6. Where do I inspect what happened?

If the answer to any of those is still:

- "it depends which subsystem exports you discovered first"

then the bridge boundary is still wrong.
