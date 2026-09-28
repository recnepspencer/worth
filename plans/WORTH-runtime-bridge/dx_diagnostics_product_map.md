# WORTH Runtime Bridge DX Diagnostics Product Map

## Purpose

This document defines how bridge diagnostics should feel as a product.

The bridge already has deep diagnostics richness.
That is not the problem.

The problem is that richness is still too easy to experience as:

- a record warehouse
- a subsystem inventory
- a list of things the bridge remembers

instead of:

- one obvious inspection door for normal bridge work

This document exists to force the bridge diagnostics story into job-first
shape.

---

## Product Rule

The first question a caller should ask is:

- what just happened?

not:

- which retained record family do I need to query first?

The diagnostics surface must therefore begin with job-shaped questions and only
descend into record-family questions when the caller is doing specialist work.

---

## Primary Diagnostics Jobs

The diagnostics product must optimize for these jobs first:

1. explain the last bridge action
2. explain a route
3. explain a truth-view evaluation
4. explain a speculative session
5. explain a discard outcome
6. explain a promotion outcome
7. compare branch-local versus main outcomes
8. export or replay bridge evidence

If those jobs are not obvious, the bridge diagnostics product is not done.

---

## Canonical Diagnostics Door

The primary diagnostics door is:

- `RuntimeBridge::diagnostics()`

Everything ordinary should begin there.

That door should feel like:

- inspect
- explain
- compare
- replay
- export

not:

- browse raw records until something looks right

---

## Required Everyday Methods

The diagnostics surface should converge toward these everyday methods:

- `explain_last()`
- `explain_last_route()`
- `explain_route(...)`
- `explain_last_evaluation()`
- `explain_evaluation(...)`
- `explain_last_session()`
- `explain_session(...)`
- `explain_last_discard()`
- `explain_last_promotion()`
- `compare_main_and_speculative(...)`
- `export_certification_bundle(...)`

Some of these already exist in partial form.
The remaining work is to make the set complete and coherent.

---

## Product Grouping

Diagnostics should be taught and grouped in this order.

### Group 1: Explain What Just Happened

This is the default entry.

Questions:

- what was the last route?
- what truth view did I evaluate against?
- what happened to my speculative session?
- did I discard or promote?

### Group 2: Inspect A Named Bridge Object

This is for targeted follow-up.

Questions:

- explain route by route identity
- explain evaluation by record identity
- explain session by preview session identity
- explain promotion by preview session identity

### Group 3: Compare Outcomes

This is for Milestone 13-style bridge use.

Questions:

- compare main and speculative branch views
- compare two evaluations
- compare replayed and original bridge artifacts

### Group 4: Replay And Export

This is where diagnostics becomes a trust surface.

Questions:

- replay retained preview bundle
- replay canonical route record
- export certification evidence
- inspect causal bundle equivalence

### Group 5: Raw Record Access

This remains real and public.

But it is specialist.

It should be explained as:

- deeper forensic access
- adapter-authoring access
- certification internals

not as the default usage story.

---

## Everyday Explanation Shapes

The bridge should expose explanation objects that answer job questions first.

### Route

A route explanation should foreground:

- source commit
- source branch
- source snapshot
- invalidation target count
- routing mode
- fallback usage

### Evaluation

An evaluation explanation should foreground:

- selector or truth-view basis
- snapshot identity
- materialization path
- read packet shape
- retained record identity

### Speculative Session

A session explanation should foreground:

- preview session identity
- truth branch identity
- signal branch identity
- session outcome
- comparison basis

### Discard

A discard explanation should foreground:

- preview session identity
- residue summary
- cleanup outcome
- what remained non-authoritative

### Promotion

A promotion explanation should foreground:

- preview session identity
- authoritative commit-boundary digest
- authoritative artifact digest
- promotion proof linkage

---

## Comparison Product Requirements

The bridge docs and Milestone 13 story both require comparison as a
first-class experience.

That means the diagnostics product must support comparison questions without
forcing callers into structural-only APIs.

Required comparison stories:

- compare main and speculative evaluation snapshots
- compare retained route outcomes across interleaved churn
- compare replay bundle outcome to live session outcome
- compare diagnostics-tier variants without semantic drift

This does not necessarily require one giant comparison API.
It does require coherent comparison guidance and at least one obvious
ordinary-use entrypoint.

---

## Relationship To Replay

Replay is not a separate observability universe.

Replay is part of the diagnostics product.

The user journey should feel like:

- explain what happened
- retrieve the retained artifact
- replay it if needed
- compare replayed meaning to original meaning

That means diagnostics docs and APIs must link naturally to replay instead of
treating it as a disconnected specialist cave.

---

## What Must Stay Specialist

These remain public but should not dominate the diagnostics product story:

- `*_record_for_identity(...)`
- `last_*_record(...)`
- raw canonical record access
- raw replay helpers for individual record families
- specialist merge, structural, stream, and writeback record inspection

The rule is not to remove them.

The rule is:

- ordinary users should not need them first

---

## Milestone 13 Diagnostics Rule

The pricing-shock reference workload should use diagnostics as follows:

- explain route results through job-shaped access
- explain speculative session outcomes through job-shaped access
- use replay bundles as retained evidence
- only descend into raw record families when a test is explicitly certifying a
  specialist replay or record contract

If the pricing-shock workload still needs record-family spelunking for ordinary
assertions, the diagnostics product is still incomplete.

---

## Concrete Immediate Gaps

Based on the current bridge surface, the most important remaining diagnostics
gaps are:

- no equally obvious evaluation explanation door yet
- comparison helpers are still thin relative to the docs vision
- certification export still reads more specialist than everyday advanced
- naming across explanation methods is good but not yet complete enough to feel
  framework-grade

These should be the next implementation targets.

---

## Completion Test

The diagnostics product is only done when a new engineer or AI agent can do
all of the following without studying raw diagnostics state first:

1. explain the last route
2. explain the last evaluation
3. explain a speculative session
4. tell whether a session was discarded or promoted
5. compare main and speculative truth outcomes
6. retrieve retained replay evidence
7. know when to drop into raw records and when not to

If that still feels like expert spelunking, the bridge diagnostics product is
not finished.
