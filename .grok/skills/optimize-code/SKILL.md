---
name: optimize-code
description: Produce a concrete optimization plan for existing WORTH code by finding missed assumptions, unnecessary work, obsolete code, duplicated responsibilities, weak abstraction boundaries, and demonstrated scalability opportunities. Use when the user wants to plan code deletion, consolidation, simplification, or optimization without implementing the changes.
---

# Optimize Code

Produce an architecture-grounded plan for making existing code smaller, clearer,
more coherent, and more efficient. Do not implement the plan or edit production
code, tests, specifications, or repository state during this turn.

Plans and specifications express intended behavior, but they may contain
incomplete assumptions about real consumers, scale, lifecycle, failure, or the
cheapest coherent structure. Inspect the actual producers, consumers, data flow,
lifecycle, and workload rather than treating the existing plan as proof of the
best implementation.

## Establish scope

Read the repository instructions, governing specification, relevant engineering
laws, current implementation, tests, and available performance evidence. State:

- the behavior and contracts that must be preserved
- the complete production journey under review
- the directly affected owners and consumers
- the expected workload and scale
- the boundaries outside the optimization scope

Trace at least one real journey from public entry through preparation, effects,
observation, recovery, and cleanup. Keep unrelated repository debt outside the
plan.

## Challenge assumptions

Verify inherited assumptions against the actual system, including:

- cardinality, expected scale, and locality of changes
- ordinary versus reconstructive operation frequency and cost
- concurrency, ordering, cancellation, rejection, retry, and partial effects
- ownership, cleanup, maximum work, capacity, and backpressure
- cache validity, retained history, identity, and revision stability
- real consumer behavior and hot-path placement
- typed evidence crossing boundaries
- whether sibling consumers truly share semantics
- whether current abstractions still match their real consumers

Report missed assumptions even when the implementation passes its tests.
Distinguish demonstrated defects, structurally inevitable scale problems,
concerns requiring measurement, and speculative possibilities. Do not turn
speculation into required work.

## Find deletion and consolidation opportunities

Look for superseded or duplicate public routes, repeated preparation or recovery,
decisions made by multiple authorities, flattened and reconstructed values,
wrappers without authority or meaning, marker types without substantive evidence,
obsolete adapters and compatibility surfaces, production-bypassing test routes,
redundant fixtures, accidental sibling differences, single-consumer abstractions,
repeated authentication or observation, and orchestration layers that only
forward calls.

Verify callers and semantic purpose before proposing deletion. For each proposed
deletion, name the exact path, its callers, why it is obsolete or duplicated, the
surviving owner, and the evidence needed before removal.

## Find scalability and efficiency opportunities

Inspect the actual cost structure for:

- global scans serving local changes
- unsuitable nested growth or arbitrary polling loops
- repeated parsing, allocation, cloning, formatting, or serialization
- N+1 queries and unnecessary boundary crossings
- recomputation of prepared evidence
- reconstructive work leaking into ordinary paths
- broad invalidation despite available dependency evidence
- unbounded queues, histories, caches, or retained revisions
- eager work that could remain demand-driven
- unnecessary synchronization or lock duration
- missing batching or backpressure at a real workload boundary
- cleanup cost tied to historical rather than live ownership
- large intermediate collections and duplicated conversions
- abstractions that conceal expensive work

Use existing measurements when available and direct complexity or lifecycle
analysis otherwise. Recommend measurement when evidence is insufficient. Do not
prescribe theoretical micro-optimizations without a meaningful workload.

## Evaluate abstraction boundaries

For each abstraction to introduce, retain, collapse, or remove, answer:

1. What semantic responsibility does it own?
2. Which real producers and consumers use it?
3. What authority or evidence crosses it?
4. What duplicate decision, sequencing, or cost does it eliminate?
5. What code becomes deletable?
6. Can an ordinary caller bypass it?
7. Does it preserve locality and bounded work?
8. Would it still make sense for another real consumer?

Do not recommend generic pipelines, marker wrappers, parallel authority lanes,
configurable frameworks, or speculative extension points without a demonstrated
integration need. Unify paths only when authority, lifecycle, recovery, and
performance responsibilities match.

## Form the optimization plan

Prioritize changes in this order when causally appropriate:

1. Delete obsolete paths and fixtures.
2. Remove unnecessary work.
3. Collapse duplicated responsibilities and sequencing.
4. Correct misplaced authority or evidence.
5. Carry existing results instead of recomputing them.
6. Localize or bound work.
7. Correct abstraction boundaries.
8. Add an abstraction only when real consumers justify it.

Plan around a complete production endpoint. The first implementation slice must
produce an observable end-to-end improvement and delete the path it replaces.
Include prerequisites only when they directly enable that endpoint, then return
immediately to it. Group work by causal behavior rather than file or crate.

For each plan item, include the problem and evidence, any missed assumption, the
affected producers and consumers, exact code to delete, collapse, move, or
introduce, preserved behavior and authority, expected cost or locality
improvement, discriminating verification, and dependencies. Separate required
optimization from optional hardening.

## Evaluate tests and fixtures

Identify tests and fixtures that protect distinct behavior, duplicate stronger
evidence, bypass production, repeat implementation logic in their oracle, require
unrealistic setup, conceal scale defects, or impose disproportionate cost.

Preserve the smallest honest evidence set. Do not weaken acceptance requirements.
Recommend new evidence only when an optimization changes or exposes a material
property such as locality, bounded work, capacity, recovery, cancellation,
concurrency, or cleanup.

## Output

Produce a concise report containing:

- **Protected behavior:** production journey and contracts that remain intact.
- **Findings:** category, concrete code and callers, evidence, consequence, and
  confidence for each material finding.
- **Optimization destination:** smallest coherent final structure, surviving
  owners, and surviving public routes.
- **Implementation sequence:** short causal sequence that reaches an end-to-end
  improvement early and deletes replaced machinery as it proceeds.
- **Required verification:** focused production-path evidence, measurements, and
  repository gates.
- **Deferred opportunities:** concrete items excluded because evidence is
  insufficient, scope is unrelated, or another milestone owns them.

Do not produce a progress ledger, generic best-practices essay, speculative
rewrite, or implementation diff. Make the next coding pass direct and difficult
to misinterpret.
