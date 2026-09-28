# WORTH Runtime Bridge Docs Information Architecture

## Purpose

This document defines the publish-facing docs architecture for
`worth-runtime-bridge`.

It is the bridge equivalent of the docs discipline used by strong framework
docs such as Next.js, Angular, and Laravel:

- start with the product promise
- get users to a working result quickly
- organize guides around real jobs
- separate concepts from tasks
- make diagnostics and troubleshooting first-class
- contain deep protocol reference instead of forcing it into onboarding

This is not the same thing as the bridge DX cleanup docs.
Those define the API shape.
This document defines how that shape should be taught.

---

## Governing Documentation Principles

### 1. Start With Jobs, Not Subsystems

The first docs should answer:

- how do I set up the bridge
- how do I route truth into computation
- how do I evaluate against a truth view
- how do I open a speculative session
- how do I discard or promote it
- how do I inspect what happened

They should not start with:

- routing planner internals
- writeback family taxonomy
- mapper containment mechanics
- raw record inventories

### 2. The Bridge Must Read Like One Product

The public docs must present the bridge as:

- one causal protocol boundary between `worth-relational` and `worth-signal`

They must not present it as:

- a loose pile of milestone capabilities
- a re-export inventory
- a certification harness wearing a library costume

### 3. Concepts Should Support Guides, Not Replace Them

Architecture and protocol docs matter, but they should come after a reader can
already picture the ordinary workflows.

The public docs should teach in this order:

1. what the bridge is for
2. how to use it for normal work
3. how to inspect and trust it
4. how advanced protocol surfaces work

### 4. Diagnostics Are First-Class

For the bridge, diagnostics are not a support appendix.

They are part of the product promise:

- deterministic routing
- explicit truth-view basis
- explicit speculative versus authoritative basis
- family-aware writeback explanation
- replay and certification evidence

That means `DIAGNOSTICS.md` belongs in the first-read set.

### 5. Certification Must Be Visible But Contained

The bridge must be able to prove itself.

That proof story should be documented publicly, but it should not dominate
day-one onboarding.

The docs should expose certification as:

- a trust surface
- an advanced workflow
- a deeper reference area

not as the first thing ordinary users have to memorize.

---

## Primary Reader Journeys

The docs should optimize for these reader journeys first.

### Journey 1: "What is this?"

Reader goal:

- understand why the bridge exists
- understand its role between relational truth and signal computation
- understand what it owns and what it does not own

Docs:

- `README.md`
- `API_OVERVIEW.md`

### Journey 2: "Get me to a real result"

Reader goal:

- build a bridge
- wire sources and sinks
- route a truth change
- run an evaluation
- inspect a result

Docs:

- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`

### Journey 3: "Now I need speculation"

Reader goal:

- open a branch-local or preview session
- compare it to the main branch
- discard it or promote it safely

Docs:

- `BRANCHING_AND_SPECULATION.md`
- `WRITEBACK_AND_PROMOTION.md`

### Journey 4: "I need to understand what happened"

Reader goal:

- explain routing
- explain truth-view selection
- explain writeback outcome
- inspect replay and certification artifacts

Docs:

- `DIAGNOSTICS.md`
- `HISTORY_AND_REPLAY.md`
- `CAUSAL_BUNDLES_AND_GUARANTEES.md`

### Journey 5: "I am integrating something non-trivial"

Reader goal:

- use advanced source contracts
- understand mapping and continuity
- work with merge and structural comparison
- author or adapt host integrations

Docs:

- `ROUTING_AND_EVALUATION.md`
- `CHANGE_STREAMS_AND_SOURCES.md`
- `MAPPING_CONTINUITY_AND_REMAP.md`
- `MERGE_AND_STRUCTURAL_COMPARISON.md`
- `HOST_ADAPTERS.md`

---

## Recommended Published Docs Set

## Tier 1: First-Read Product Docs

### `README.md`

Purpose:

- landing page
- bridge thesis
- when to use it
- what it integrates
- shortest possible example

This should feel like:

- Next.js landing page plus a tiny first success path

### `QUICKSTART.md`

Purpose:

- smallest honest bridge setup
- source registration
- sink registration
- route truth change
- evaluate and inspect result

This should feel like:

- "five minutes to first real bridge run"

### `DAILY_WORKFLOWS.md`

Purpose:

- task-oriented guide for the six canonical jobs
- ordinary route, evaluate, speculate, discard or promote, inspect, export

This should feel like:

- Laravel-style task cookbook

### `API_OVERVIEW.md`

Purpose:

- explain the public API shape
- show the top-level types and namespaces
- clarify everyday versus advanced versus specialist surfaces

This should feel like:

- Angular-style "mental model of the public surface"

### `DIAGNOSTICS.md`

Purpose:

- explain the one diagnostics door
- teach explanation, replay, comparison, and certification readback
- serve as both observability guide and troubleshooting entry

This should feel like:

- a first-class product guide, not a support afterthought

## Tier 2: Important Specialist Docs

### `ROUTING_AND_EVALUATION.md`

Purpose:

- truth-change routing
- truth-view selection
- current, historical, and branch-local evaluation
- bulk-path and deterministic-routing mental model

### `BRANCHING_AND_SPECULATION.md`

Purpose:

- preview sessions
- speculative branch coordination
- split-view comparison
- discard semantics

### `WRITEBACK_AND_PROMOTION.md`

Purpose:

- authoritative handoff
- no-op versus commit
- strategy-bearing promotion
- family-aware promotion basics

### `HISTORY_AND_REPLAY.md`

Purpose:

- replay
- historical truth views
- offline diagnosis
- branch-local auditability

### `RUNTIME_POLICY.md`

Purpose:

- diagnostics tiers
- runtime profiles
- policy selection and refinement
- when policy changes behavior versus explanation granularity

## Tier 3: Deeper Reference Docs

### `CHANGE_STREAMS_AND_SOURCES.md`

Purpose:

- change feed contracts
- source protocols
- multi-consumer and backpressure-oriented integration detail

### `MAPPING_CONTINUITY_AND_REMAP.md`

Purpose:

- aspect mapping
- lineage continuity
- structural remap surfaces
- identity-preserving versus identity-rejecting transitions

### `MERGE_AND_STRUCTURAL_COMPARISON.md`

Purpose:

- merge-aware bridge semantics
- structural comparison
- speculative-versus-authoritative comparison machinery

### `CERTIFICATION_AND_HARNESS.md`

Purpose:

- what the bridge certifies
- how the pricing-shock reference workload proves the architecture
- how integration and end-to-end suites are organized

### `CAUSAL_BUNDLES_AND_GUARANTEES.md`

Purpose:

- canonical bundle story
- typed failures
- zero-residue guarantees
- replay equivalence and trust claims

### `HOST_ADAPTERS.md`

Purpose:

- adapter-authoring guidance
- mapper containment principles
- what host integrations may translate versus what the bridge must own

---

## Suggested Navigation Model

If we later build a docs site or crate-doc structure, the sections should group
like this:

### Getting Started

- `README.md`
- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`

### Core Concepts

- `API_OVERVIEW.md`
- `ROUTING_AND_EVALUATION.md`
- `BRANCHING_AND_SPECULATION.md`
- `WRITEBACK_AND_PROMOTION.md`

### Observability And Trust

- `DIAGNOSTICS.md`
- `HISTORY_AND_REPLAY.md`
- `CAUSAL_BUNDLES_AND_GUARANTEES.md`
- `CERTIFICATION_AND_HARNESS.md`

### Advanced Integration

- `RUNTIME_POLICY.md`
- `CHANGE_STREAMS_AND_SOURCES.md`
- `MAPPING_CONTINUITY_AND_REMAP.md`
- `MERGE_AND_STRUCTURAL_COMPARISON.md`
- `HOST_ADAPTERS.md`

---

## Documentation Order Of Attack

### Priority 0

Write first:

- `README.md`
- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`
- `API_OVERVIEW.md`
- `DIAGNOSTICS.md`

Reason:

- without these, the bridge still lacks a publishable spine

### Priority 1

Write next:

- `ROUTING_AND_EVALUATION.md`
- `BRANCHING_AND_SPECULATION.md`
- `WRITEBACK_AND_PROMOTION.md`
- `HISTORY_AND_REPLAY.md`

Reason:

- these complete the major user journeys and the Milestone 13 narrative

### Priority 2

Write after that:

- `RUNTIME_POLICY.md`
- `CHANGE_STREAMS_AND_SOURCES.md`
- `MAPPING_CONTINUITY_AND_REMAP.md`
- `MERGE_AND_STRUCTURAL_COMPARISON.md`
- `CERTIFICATION_AND_HARNESS.md`
- `CAUSAL_BUNDLES_AND_GUARANTEES.md`
- `HOST_ADAPTERS.md`

Reason:

- this is the deeper protocol and integration layer

---

## Bottom Line

The bridge docs should feel like a serious framework product:

- clear first success
- clear daily jobs
- clear diagnostics and trust story
- clear advanced lanes
- no requirement to learn milestone history before becoming productive

That is the standard this information architecture is setting.
