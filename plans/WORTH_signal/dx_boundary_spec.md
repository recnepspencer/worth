# WORTH Signal DX Boundary Spec

## Purpose

This document translates the DX plan into a concrete public-boundary target.

It answers:

- what the published public surface should look like
- what the everyday developer should reach for first
- what specialist surfaces should remain available without dominating the
  product
- what should leave the visible product boundary

This is the first code-shaping document, not just a cleanup philosophy.

---

## Daily-Use Principle

The public API should optimize for mundane day-to-day use first.

That means a developer should be able to do the common jobs without thinking
about:

- proof-bearing pipeline forms
- merge witnesses
- artifact retention internals
- internal contract taxonomy
- low-level diagnostics plumbing

The daily-use jobs are:

1. define a graph or computation
2. update inputs or invalidate work
3. read or evaluate derived outputs
4. configure runtime behavior at a high level
5. debug why something changed
6. batch work safely

If those jobs are not smooth, the library is not ready regardless of how strong
the deep specialist surfaces are.

---

## Canonical Public Layers

## Layer 1: Everyday Product Surface

This is what ordinary users should learn first and keep using daily.

Required characteristics:

- short import story
- strong defaults
- low ceremony
- obvious escalation path

## Layer 2: Advanced Runtime Control

This is for users who need deliberate control over scheduling, batching,
policies, or state history.

Required characteristics:

- explicit
- structurally coherent
- discoverable after Layer 1, not before it

## Layer 3: Integration And Specialist Infrastructure

This is for runtime bridge authors and deep specialists.

Required characteristics:

- available
- narrow
- clearly specialist

## Layer 4: Internal Support

Not part of the product boundary.

---

## Final Intended Top-Level Surface

The top-level public identity should converge toward:

- `worth_signal::facade`
- `worth_signal::easy`
- optionally `worth_signal::diagnostics`, but only if it is kept consistent with
  the curated diagnostics story

Everything else should either be subordinate to those entry points or not be
publicly emphasized.

---

## Target `facade` Shape

The facade should converge toward a small set of strong namespaces.

## `facade::core`

Daily-use semantic vocabulary and the shortest path to ordinary work.

Should contain:

- `SignalGraph`
- `NodeBuilder`
- `NodeId`
- `Aspect`
- `AspectMask`
- `AspectVersion`
- `DependencyEdge`
- `NodeState`
- `ChangedRegion`
- `NodeEvaluationResult`
- `OutputChange`
- `OutputIdentity`
- `EvaluationCondition`
- partition primitives needed in normal use
- core errors

Why:

- everyday work should not force users to guess between `types`, `graph`, and
  `evaluation`

## `facade::runtime`

The canonical production entry surface.

Should contain:

- `SignalRuntime`
- `SignalRuntimeBuilder`
- `SignalRuntimeConfig` only if it remains part of the guided setup story
- `SignalTransaction`
- `TransactionResult`
- `TransactionOutcome`
- `TransactionTiming`
- `EvaluationSummary`
- `SignalRuntimePolicy`
- guided runtime setup and computation-definition entry points
- batch invalidation entry points

Why:

- mundane day-to-day usage should revolve around â€œthe runtimeâ€ as one coherent
  thing

## `facade::diagnostics`

Job-oriented diagnostics surface.

Should contain:

- guided diagnostics access
- explanation
- comparison
- health/summary
- history/replay/lineage entry points

Why:

- diagnostics is a moat, but it must feel like one subsystem with a few clear
  jobs

## `facade::advanced`

Advanced runtime control that is legitimate but not needed every day.

Should contain:

- executor control
- comparator control
- condition resolver control
- tier policy
- checkpoint policy
- snapshot/branch/restore advanced surfaces

Why:

- many of these are real and important, but they should not compete with the
  main daily-use story

## `facade::integration`

Specialist integration-author surface.

Should contain:

- merge/reconciliation orchestration
- event/subscriber integration
- reuse/equivalence machinery that external integrations truly need
- proof-bearing forms only if they remain externally necessary
- reconstructability and specialized history/support forms where justified

Why:

- this keeps runtime bridge and specialist infrastructure available without
  making normal use feel infrastructural

## `facade::easy`

Not needed as a nested namespace if `worth_signal::easy` remains top-level.

Policy:

- do not duplicate the product boundary inside the facade if it creates
  confusion

## Namespaces That Should Disappear From The Main Facade

The following current facade shapes should not survive unchanged:

- `facade::types`
- `facade::graph`
- `facade::evaluation`
- `facade::planning`
- `facade::performance`
- `facade::proof`
- `facade::harness`

Reason:

- they reflect internal decomposition and export inventory more than the product
  memory model we want users to keep

This does not mean all the underlying capability disappears. It means the
published grouping must change.

---

## Everyday Canonical Flows

These are the flows that must feel obvious in ordinary use.

## 1. Define Graph Structure

Canonical surface:

- `SignalGraph`
- `NodeBuilder`

Desired feel:

- create nodes
- declare conditions and output policy where needed
- wire dependencies
- done

Anti-goal:

- needing to consult several namespaces to assemble one node definition

## 2. Set Up A Production Runtime

Canonical surface:

- `SignalRuntime::builder(graph)`
- `SignalRuntimePolicy`

Desired feel:

- choose a policy preset
- optionally refine advanced sections
- build

Anti-goal:

- scattered runtime configuration with several equally primary control points

## 3. Define Durable Computations

Canonical surface to drive toward:

- guided computation declaration from the runtime surface

Desired feel:

- one declaration for identity, behavior, and important policy

Anti-goal:

- remembering a coordination sequence across graph setup, runtime registration,
  and read-time closures

## 4. Invalidate Or Update Inputs

Canonical surface:

- batch-first invalidation on the runtime/transaction surface
- scalar helpers as convenience only

Desired feel:

- mutation and invalidation feel like one coherent operation

Anti-goal:

- production users learning scalar invalidation as the main mental model

## 5. Read Or Evaluate Outputs

Canonical surface:

- runtime reads/evaluation for production
- graph-level prepared execution for explicit advanced control

Desired feel:

- day-to-day reading and evaluation feel easy
- advanced execution still exists without becoming the default path

Anti-goal:

- making every serious read/evaluation feel like manual orchestration

## 6. Explain And Debug

Canonical surface:

- runtime/graph diagnostics access point

Desired feel:

- â€œwhy did this change?â€ should be near at hand

Anti-goal:

- having to know whether to start with inspect, compare, render, diff, summary,
  lineage, or replay helpers

---

## Positive Design Rules For Mundane DX

## Rule 1: The Common Path Must Require Very Few Concepts

A normal user should not need more than these mental anchors:

- graph
- runtime
- transaction
- diagnostics

If ordinary use requires substantially more nouns than that, the surface is
still too fragmented.

## Rule 2: Everyday Actions Should Live On Everyday Objects

Examples:

- graph construction on `SignalGraph`
- production setup on `SignalRuntimeBuilder`
- batched invalidation on transaction/runtime helper objects
- explanation on diagnostics access objects

Not:

- everyday actions scattered across utility namespaces and raw support types

## Rule 3: The API Should Prefer Intent Over Mechanics

Good:

- define computation
- invalidate batch
- explain node
- compare runs

Bad:

- manually carry intermediate artifacts unless the user explicitly wants the
  lower-level path

## Rule 4: Raw Power Must Escalate Naturally

The user should be able to start simple and move deeper without relearning the
library from scratch.

## Rule 5: Advanced And Specialist Must Not Pollute Daily Autocomplete

If a user doing mundane work sees more merge/proof/certification machinery than
runtime and diagnostics ergonomics, the boundary is wrong.

---

## Surfaces To Remove From Daily View

These may remain public somewhere, but should leave the default daily-use
boundary:

- proof-bearing forms
- performance contract taxonomy
- raw reuse/equivalence machinery
- merge witnesses and conflict records
- lineage internals
- harnesses
- certification/deployment support
- internal architectural contract markers

---

## Concrete Boundary Moves

## Move Out Of Main Daily Surface

- specialist proof and performance forms
- merge and reconciliation raw forms
- artifact/history internals
- event/subscriber plumbing
- telemetry/meta plumbing

## Keep But Repackage

- planning/execution control
- checkpoint and tier policy
- snapshot/branch/restore surfaces
- comparator and condition control

## Remove From Product Boundary

- harness and certification support
- deployment support
- internal contract marker types

---

## `easy` Doctrine For Daily Use

Provisional doctrine:

- `easy` is the first-15-minutes path
- it should be explicitly guided
- it must not teach a wrong mental model

Implication:

- `easy` should help users understand the graph/runtime/diagnostics model
- it should not become a parallel long-term architecture with different core
  concepts

---

## Release Test For Boundary Quality

The boundary is good enough only if a day-to-day user can answer these
questions instantly:

1. Where do I start?
2. How do I set up a runtime?
3. How do I batch invalidations or updates?
4. How do I define a computation?
5. How do I explain why something changed?
6. Where do I go when I need deeper control?

If the answer to any of those is still â€œit depends which namespace you learned
first,â€ the boundary is not ready.
