# WORTH Relational DX Diagnostics Product Map

## Purpose

Relational diagnostics and inspection should be taught around user jobs, not a
pile of export families.

This map records the Phase 3 product story for operator-facing readback.

---

## Core Jobs

The operator-facing product should be organized around:

- inspect what happened
- inspect what published
- inspect what is wrong
- inspect what is retained
- go deeper into history or replay when needed

---

## Canonical Entry Shape

Relational does not currently have one honest `runtime.diagnostics()` door.

So the product story should not pretend otherwise.

The two nearby doors are:

- `runtime.inspection_access()`
- `runtime.publication_access()`

Those should be taught together as the operator readback surface.

The escalation door after that is:

- `runtime.history_access()`

And then:

- `runtime.replay_access()`

That is the honest shape today.

---

## Job 1: Inspect What Happened

Primary output:

- recent commit understanding
- branch-head understanding
- graph and connectivity understanding
- structural identity understanding

What should be discoverable first:

- `inspect_commit(...)`
- `inspect_recent_commits(...)`
- `inspect_branch_head(...)`
- `graph_summary()`
- `connectivity_summary()`
- `structural_identity(...)`

What should be secondary:

- lower-level noun families around historical views and structural detail

Decision:

- inspection verbs already do a decent job here
- Phase 3 should teach them as the first operator door instead of letting the
  noun surface speak for itself

---

## Job 2: Inspect What Published

Primary output:

- latest publication bundle
- latest patch
- latest replay publication
- ongoing patch and subscriber streams

What should be discoverable first:

- `latest_bundle()`
- `latest_patch()`
- `latest_replay()`
- `read_patch_stream()`
- `read_subscriber_stream()`

What should be secondary:

- raw diagnostic artifact collections
- artifact-level helper methods as first-memory knowledge

Decision:

- publication should be taught as "what got published" first
- diagnostics artifacts stay public, but as a deeper slice of the same lane

---

## Job 3: Inspect What Is Wrong

Primary output:

- publication diagnostics
- certification and invariant state
- runtime contract trouble

What should be discoverable first:

- `publication_access().diagnostics()`
- `publication_access().diagnostics_since(...)`
- `invariant_access()`
- `certify_current_state(...)`

What should be secondary:

- raw artifact views
- certification-only test surfaces

Decision:

- there is not one unified diagnostics object yet
- the honest product story is split:
  - publication diagnostics for emitted trouble
  - validation for truth-contract trouble

---

## Job 4: Inspect What Is Retained

Primary output:

- retention summary
- record retention detail
- snapshot pinning state
- retention execution detail

What should be discoverable first:

- `retention_summary()`
- `inspect_record_retention(...)`
- `inspect_snapshot_pinning(...)`
- `inspect_retention_execution(...)`

What should be secondary:

- retention control
- durability control

Decision:

- retention readback belongs in the operator readback story
- retention control belongs in the contained `retention` lane

---

## Job 5: Go Deeper Into History Or Replay

Primary output:

- past truth reads
- commit envelopes
- replay outcomes
- verification and reconstruction

What should be discoverable first:

- `history_access()`
- `latest_commit()`
- `branch_head(...)`
- `entity_aspect_history(...)`
- `relation_aspect_history(...)`
- `replay_access()`
- `compare_outcome(...)`

What should be secondary:

- trace-heavy history helpers
- replay ranges
- merge-adjacent history helpers

Decision:

- history is the first escalation after operator readback
- replay is the deeper verification lane after history

---

## Product Rule

The operator-facing product should feel like:

- one nearby inspection lane
- one nearby publication lane
- one clear history escalation
- one deeper replay escalation

Not:

- a taxonomy of helper families

---

## Design Requirement

The user or AI agent should not need to first understand:

- diagnostics artifact nouns
- publication bundle internals
- validation helper naming
- replay record internals

Instead, the product story should lead with the jobs above and let the raw
types show up only after the right door has already been chosen.

---

## Phase 3 Outcome

Phase 3 is done when all of these are true:

- `inspection_access()` is the obvious first operator door for "what happened?"
- `publication_access()` is the obvious nearby door for "what published?" and
  "what is wrong in publication?"
- `invariant_access()` is taught as the contained validation lane, not a random
  helper seam
- retention readback is easy to discover from inspection
- `history_access()` is the obvious next door after current readback
- `replay_access()` is the obvious deeper verification door
