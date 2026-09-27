# WORTH Runtime Bridge DX Boundary Cleanup Spec

## Purpose

This document turns the bridge DX principles into a concrete cleanup target for
the current public API.

It is intentionally focused on one question:

How do we reshape the existing `RuntimeBridge`, builder, request, and
diagnostics surfaces so Milestone 13 can be implemented through real
integration-grade public flows rather than through direct phase stitching?

This is the concrete API-hardening spec that sits between:

- the bridge DX boundary docs
- the current subsystem-shaped facade
- the upcoming Milestone 13 implementation work

It should be read as one phase inside the full bridge DX program, not as the
whole program.

---

## Input Surfaces Audited

This cleanup spec is based on the current method inventory in:

- [`crates/worth-runtime-bridge/src/facade.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade.rs)
- [`crates/worth-runtime-bridge/src/facade/runtime.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade/runtime.rs)
- [`crates/worth-runtime-bridge/src/facade/runtime/routing_and_bulk.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade/runtime/routing_and_bulk.rs)
- [`crates/worth-runtime-bridge/src/facade/runtime/speculation.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade/runtime/speculation.rs)
- [`crates/worth-runtime-bridge/src/facade/runtime/writeback.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade/runtime/writeback.rs)
- [`crates/worth-runtime-bridge/src/facade/runtime/stream.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade/runtime/stream.rs)
- [`crates/worth-runtime-bridge/src/facade/runtime/source`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade/runtime/source)
- [`crates/worth-runtime-bridge/src/diagnostics/facade`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/diagnostics/facade)
- [`crates/worth-runtime-bridge/src/builder`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/builder)

---

## Adversarial Constraint

The cleanup must survive the following hostile condition:

> Milestone 13â€™s pricing-shock reference workload must be implementable through
> a small number of bridge-owned request/session flows over the public facade,
> without direct calls into low-level validation, lowering, replay, mapper, or
> canonicalization phases unless the test is explicitly certifying those
> specialist phases themselves. If the ordinary reference workload still needs
> to call phase-level methods such as `validate_*`, `admit_*`, `lower_*`,
> `canonicalize_*`, and `replay_*` directly to perform ordinary bridge jobs, the
> bridge boundary remains structurally dishonest.

This is the hard problem first.

The point of the cleanup is not prettier names.
The point is to prevent Milestone 13 from baking internal phase decomposition
into the public testing contract and to prevent the wider bridge DX program
from freezing todayâ€™s accidental shape.

---

## Current Boundary Problems

### 1. The facade is subsystem-first, not job-first

The bridge surface currently exposes a large number of strong types and methods,
but the shape is still primarily:

- stream subsystem
- source subsystem
- structural subsystem
- policy subsystem
- merge subsystem
- speculation subsystem
- writeback subsystem

Now that `milestone-12b` is complete, that writeback subsystem is even richer:

- family admission
- mapper envelopes
- mapped family inputs
- family-aware replay
- family-aware diagnostics

That depth is architecturally correct.
It is not yet boundary-correct.

That is honest internally.
It is not yet the right everyday memory shape.

### 2. Phase-level verbs dominate public execution

Current public methods include many raw phase verbs:

- `validate_*`
- `admit_*`
- `lower_*`
- `canonicalize_*`
- `replay_*`
- `publish_*`
- `reduce_*`

These are architecturally legitimate and should remain available for specialist
work.
But ordinary bridge jobs should not require callers to assemble these phases by
memory.

### 3. The diagnostics surface is rich but not yet job-shaped

`BridgeDiagnosticsFacade` has many record-specific query and explain methods.

That richness is valuable.
The problem is that the first-question story is still weak.

A caller should start from a question like:

- explain this route
- inspect this speculative session
- compare these two branches
- export this certification bundle

and only descend into record-specific methods when necessary.

### 4. The builder is explicit but still too inventory-shaped

The builder already has strong explicit registration methods.
That is good.

But the setup story still reads more like:

- here is the list of things the bridge might need

than:

- here is the normal way to create a real bridge for routing, evaluation, and
  speculation work

---

## Cleanup Target

The bridge should converge toward six canonical everyday flows:

1. setup
2. route truth change
3. evaluate against truth view
4. open speculative session
5. discard or promote speculative session
6. inspect or export bridge diagnostics

Every other public surface should be classified relative to those flows:

- supports one of them directly
- is advanced but contained
- is specialist and intentionally raw

---

## Required Public Shapes

## Phase 1: Guided Everyday Requests

### Goal

Introduce or promote guided request/session objects for ordinary bridge jobs.

### Must Ship

- one guided route request path
- one guided truth-view evaluation request path
- one guided speculative session request path
- one guided discard or promote path
- one guided diagnostics inspection or bundle-export path

### Must Preserve

- raw specialist methods remain available where the phase decomposition matters
- bridge authority boundaries remain explicit
- no hidden ambient defaults

### Architectural Shape

The bridge should prefer objects like:

- `BridgeRouteRequest`
- `BridgeTruthViewEvaluationRequest`
- `BridgeSpeculativeSessionRequest`
- `BridgeSessionDispositionRequest`
- `BridgeDiagnosticsRequest` or `BridgeCertificationBundleRequest`

over requiring ordinary callers to manually sequence:

- validate
- admit
- lower
- prepare
- deliver
- canonicalize
- replay

for one normal bridge story.

### Existing API Mapping

The current routing path has enough raw pieces but needs a more guided wrapper
over methods such as:

- `ingest_committed_patch(...)`
- `plan_committed_patch(...)`
- `deliver_invalidation(...)`
- `prepare_signal_evaluation(...)`

The current speculation path has enough raw pieces but needs a more guided
session flow over methods such as:

- `validate_preview_session_declaration(...)`
- `declare_preview_session(...)`
- `admit_preview_session(...)`
- `activate_preview_session(...)`
- `discard_preview_session(...)`
- `promote_preview_session(...)`

The current writeback path has enough raw pieces but should stay specialist
unless ordinary bridge promotion flows need a narrower guided story that
accounts for the Milestone 12b family boundary without exposing all of it.

---

## Phase 2: Boundary Containment Of Raw Specialist Phases

### Goal

Keep phase-structured methods public only where they are legitimately
specialist.

### Must Ship

- a clear containment policy for raw phase verbs
- a public distinction between ordinary bridge flows and specialist bridge
  authoring/certification flows
- facade grouping or naming that reinforces that distinction

### Must Preserve

- specialist tooling remains real and powerful
- certification and replay work remain possible through explicit surfaces
- advanced users can still target phase boundaries when they mean to

### Boundary Rules

These method families should be considered specialist by default:

- `validate_*`
- `admit_*`
- `lower_*`
- `canonicalize_*`
- `replay_*`
- `publish_*`
- `reduce_*`

They should stay public only when one of the following is true:

- the method corresponds to a real externally meaningful protocol boundary
- the method is itself the target of a certification suite
- removing it would force dishonest hidden behavior into a guided flow

Otherwise, guided paths should absorb them.

---

## Phase 3: Diagnostics Jobs Over Record Inventory

### Goal

Make diagnostics begin with user jobs instead of record classes.

### Must Ship

- one primary diagnostics entrypoint story
- job-shaped convenience methods for the Milestone 13 workload
- explicit downgrade path from job-shaped queries into raw record queries

### Must Preserve

- existing record richness
- record-level explainers for specialist work
- deterministic diagnostics semantics

### Required Job Surfaces

The bridge diagnostics surface should support job-first access like:

- inspect latest route story
- inspect speculative session story
- inspect discard residue story
- inspect promotion story
- inspect writeback story
- export certification bundle

These may be implemented as wrappers over existing record accessors, but the
public story must begin there.

### Existing API Problem

The current diagnostics facade is strong but record-heavy:

- dozens of `explain_*_record(...)`
- dozens of `last_*_record(...)`
- dozens of `*_record_for_identity(...)`

Those should remain available.
They should not be the only obvious bridge diagnostics story.

---

## Concrete Method Decisions

## Keep Primary

These should stay or become primary:

- `RuntimeBridge::builder()`
- `RuntimeBridgeBuilder::build(...)`
- `RuntimeBridge::diagnostics()`
- one guided route-execution method family
- one guided truth-view evaluation method family
- one guided speculation method family

## Keep But Contain

These should remain public but should not define day-one bridge usage:

- bulk planning
- policy validation/admission/lowering
- source declaration/admission/materialization planning
- structural declaration/admission/planning/publication
- merge declaration/admission/lowering/publication
- raw writeback declaration/admission/lowering/validation/replay
- raw writeback family admission, mapper, execution, and replay detail

## Promote Into Guided Flow

These raw existing sequences should gain guided wrappers:

### Routing

Current raw sequence:

- `ingest_committed_patch(...)`
- `plan_committed_patch(...)`
- `deliver_invalidation(...)`
- `prepare_signal_evaluation(...)`

Target public shape:

- one route or route-and-prepare flow

### Speculation

Current raw sequence:

- `validate_preview_session_declaration(...)`
- `declare_preview_session(...)`
- `admit_preview_session(...)`
- `activate_preview_session(...)`
- `discard_preview_session(...)`
- `promote_preview_session(...)`

Target public shape:

- one open-session flow
- one discard flow
- one promote flow

### Diagnostics

Current raw surface:

- many record-specific `explain_*` and `*_record_for_identity` methods
- many new family-aware writeback explanation and query methods after
  `milestone-12b`

Target public shape:

- one job-first diagnostics layer above those records

---

## What Milestone 13 Tests Are Allowed To Call Directly

For ordinary end-to-end reference workload tests, direct calls should be
limited to:

- builder setup
- guided route flow
- guided evaluation flow
- guided speculation flow
- guided discard or promote flow
- diagnostics entrypoint

Direct calls into raw specialist methods are acceptable only when the test is
specifically certifying:

- a specialist boundary
- replay parity of a canonical artifact
- raw failure localization at a named phase boundary

This rule is what keeps the end-to-end tests from becoming fake seam tests.

---

## Must Not Fake

This cleanup must not:

- hide authority boundaries behind one giant `execute_everything(...)` method
- remove specialist raw methods merely to make the surface smaller
- keep the facade giant and call it clean because the types are well named
- pretend the diagnostics problem is solved by adding more record accessors
- let the reference workload depend on raw phase sequencing for ordinary jobs

---

## Self-Check

- This solves a real structural problem: Milestone 13 currently risks
  certifying subsystem seams instead of the bridge boundary.
- The adversarial constraint is precise: ordinary bridge jobs must not require
  phase-by-phase stitching.
- Authority boundaries are preserved: guided flows may compose phases, but they
  must not hide truth or compute ownership.
- The cleanup defines proof obligations, not just style goals: the reference
  workload itself becomes the public-boundary audit.
- A competent engineer can map this into concrete request/session types,
  wrappers, and diagnostics helpers.

---

## Implementation Gate

This cleanup spec is complete only when the next implementation pass can answer
all of these concretely:

1. What exact guided request types will ordinary route/evaluate/speculate jobs
   use?
2. Which existing raw methods remain public but specialist?
3. Which existing raw method sequences are wrapped by guided flows?
4. What exact diagnostics job methods will Milestone 13 call first?
5. Which current Milestone 13 tests would fail the new â€œordinary jobs must use
   canonical flowsâ€ rule?

If those answers are still fuzzy, the bridge is not ready to start the main
Milestone 13 implementation work.
