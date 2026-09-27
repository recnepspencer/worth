# WORTH Signal Exposure Cleanup Strategy

## Goal

We need a cleanup strategy that is safe under both futures:

- `worth-signal` stays mostly private/internal for a while
- `worth-signal` later becomes publicly exposed or open sourced

Because that future is still undecided, the strategy must be conservative.

The wrong move would be to overexpose now and try to claw APIs back later.

The right move is:

- keep the product story narrow
- keep specialist power available where justified
- avoid baking internal review/certification/test machinery into the public identity
- make future open-sourcing easier by reducing accidental commitments now

This document originally framed some decisions in terms of external user
stories. That framing is incomplete for `worth-signal`.

`worth-signal` is intended to support:

- web development
- geometry kernel development
- DSL and compiler-style systems
- ML and analysis pipelines
- runtime bridge integrations

That means exposure decisions cannot depend too heavily on a narrow
domain-by-domain story. The better frame is:

> what control surfaces belong to the runtime's ideal architecture, and what
> surfaces would expose implementation details, create fragmentation, or grant
> control over things users should not need to steer directly?

---

## Core Rule

Until a surface has a clear architectural reason to exist as public control,
assume it is **not part of the main public API**.

That does not always mean "make it private immediately."

It means:

- do not put it in the main facade unless there is a strong reason
- do not let it define the docs hierarchy
- do not optimize naming around internal architecture exposure

More precise version:

- expose capabilities
- avoid exposing maintenance ceremony
- avoid exposing proof artifacts as if they were everyday steering surfaces
- avoid exposing internal decomposition purely because it exists internally

---

## Architectural Exposure Model

The ideal architecture should expose only the layers a user can legitimately
control without destabilizing the runtime model.

## Layer A: Semantic authoring surface

This is the surface for describing computation.

Legitimate controls:

- graph construction
- dependency declaration
- invalidation inputs
- node contracts
- evaluation results
- runtime policy selection
- transactions
- diagnostics access

This should be compact, elegant, and stable.

## Layer B: Execution and optimization policy surface

This is the surface for steering behavior where different domains genuinely need
different tradeoffs.

Legitimate controls:

- executor selection
- comparator policies
- condition policies
- tiers
- checkpoints
- batch operations
- snapshot/branch behavior

This should remain available, but should be condensed into coherent policy
objects and specialist namespaces instead of being scattered across many
parallel concepts.

## Layer C: Integration-author surface

This is the surface for bridge and framework authors.

Legitimate controls:

- context threading
- event/subscriber integration
- reuse/equivalence contracts
- merge/reconciliation integration
- reconstructability and replay integration
- proof-bearing performance forms where the bridge truly consumes them

This may need real power, but it should not distort Layer A.

## Layer D: Internal enforcement and certification surface

This is not product API.

Examples:

- harnesses
- parity/certification helpers
- internal contract marker types
- deployment/certification presets
- support scaffolding for architectural verification

These should not shape the visible library identity.

---

## Decision Classes

Every currently exported symbol should land in one of four action buckets.

## `Keep`

Remain in the main public story.

Use when:

- the symbol is central to normal usage
- it has a clear product narrative
- it is likely to remain stable

## `Contain`

Remain public, but move out of the main facade or into a narrower specialist
namespace.

Use when:

- expert users may need it
- runtime bridge may need it
- the surface is real but too specialized for everyday users

## `Hide`

Remove from the main facade and product docs, but do not necessarily make
private yet.

Use when:

- the surface is mostly internal
- we may still need it across internal crates
- we are not ready to break it fully or redesign it yet

## `Internalize`

Make crate-private, test-only, feature-gated, or move to another crate.

Use when:

- the surface is internal certification or harness machinery
- there is no legitimate external user story
- keeping it public creates future compatibility debt

---

## Conservative Exposure Rules

These rules should govern the cleanup:

1. Default to `Hide` or `Contain` when unsure.
2. Only `Keep` if the control belongs to the ideal public architecture.
3. Never expose a type just because it is architecturally interesting.
4. Never expose internal contracts merely because they help us reason about the codebase.
5. Do not let runtime bridge requirements force the entire library to feel like bridge infrastructure.
6. Keep room for future open-source hardening by minimizing accidental API promises now.
7. Prefer exposing one coherent control point over several overlapping knobs.
8. If two exposed concepts let users steer the same thing at different layers, the API is fragmenting.
9. If a type mainly exists to witness, prove, or certify an internal step, it should not be on the default public path.

---

## Anti-Fragmentation Rule

The public surface should not force users to choose between multiple equally
"official" ways to do the same job unless those ways represent genuinely
different layers.

Allowed:

- easy surface for fast start
- core runtime surface for production control
- specialist integration surface for bridge/framework authors

Not ideal:

- multiple overlapping orchestration APIs at the same layer
- raw proof forms mixed into ordinary authoring flows
- internal lifecycle helpers exposed alongside stable semantic controls

Practical implication:

- condense behavior into a few strong architectural surfaces
- keep namespace boundaries meaningful
- prefer progressive disclosure over flat abundance

---

## Condensation Rule

Some APIs are not merely overexposed. They are over-separated.

If two or more methods are almost always used together, or if one method is
unsafe, misleading, or incomplete without a companion method, then the public
API should prefer a higher-level object or builder that encodes the coupling.

This is a major DX lever.

The goal is not only to hide complexity. The goal is to collapse ceremony into
intentional operations.

### Good candidates for condensation

- multi-step setup flows that are nearly always executed in the same sequence
- policy bundles that currently require several independent knobs
- operations where one "raw" method is only correct when paired with follow-up
  validation, apply, commit, or reporting methods
- related runtime controls that represent one conceptual decision but are spread
  across several entry points

### Preferred shapes

- builders
- session objects
- prepared operation objects
- request/config structs
- scoped transactional helpers
- named presets that can still be refined

### Rule of thumb

If the docs need to say "call A, then usually B, and do not WORTHt C," that is
often a sign the surface should be condensed.

---

## Raw vs Guided Surface

For many capabilities, the ideal architecture may expose two layers:

### Guided surface

This should be the default path.

Properties:

- captures the common ceremony
- encodes the safe sequence
- gives strong defaults
- minimizes invalid partial states

### Raw surface

This should exist only where expert users genuinely need the control.

Properties:

- more explicit
- lower-level
- less ergonomic by design
- often specialist or integration-facing

This is different from fragmentation.

Fragmentation is when there are multiple competing "normal" APIs.
Good layering is when there is one clearly preferred guided path and one clearly
expert raw path.

---

## Companion-Method Smell

Treat the following as architecture smells during the audit:

- a method that should almost never be called alone
- a method whose output is only meaningful after passing through another method
- a method that exposes an intermediate representation as if it were a primary
  user-facing abstraction
- a method sequence that needs user memory rather than API structure to stay
  correct
- several policy setters that really represent one domain-level decision

When these appear, ask:

- should this become a builder?
- should this become a single request object?
- should this become a scoped session?
- should the raw method move down a layer while a guided method becomes primary?

---

## What We Should Probably Keep

These are the strongest candidates for stable product-facing exposure.

- `SignalGraph`
- `NodeBuilder`
- `DependencyEdge`
- `NodeEvaluationResult`
- `ChangedRegion`
- `OutputChange`
- `OutputIdentity`
- `mark_dirty`
- `mark_dirty_with_regions`
- `DirtyBatch`
- `mark_dirty_batch`
- `SignalRuntime`
- `SignalRuntimeBuilder`
- `SignalTransaction`
- `TransactionResult`
- `SignalRuntimePolicy`
- policy presets and the most important runtime config knobs
- `TierPolicy`
- `CheckpointPolicy`
- keyed computation surfaces that are part of the runtime value proposition
- a disciplined easy-mode story
- primary diagnostics and explanation surfaces

These form the product.

---

## What We Should Probably Contain

These are real capabilities, but should not dominate the default public story.

- advanced planner and executor internals
- comparator resolver plumbing
- advanced condition resolver plumbing
- observer and materializer surfaces
- event bus and subscriber integration
- branch/snapshot/replay detail types
- merge/reconciliation surfaces
- proof-bearing performance forms
- reuse/equivalence contracts
- reconstructability proofs
- storage/core-profile tuning
- telemetry/meta plumbing

These should likely live under narrower specialist namespaces rather than
adjacent to day-one APIs.

---

## What We Should Probably Hide

These may remain public temporarily, but should leave the main facade first.

- architectural contract marker types
- deployment planning outputs
- profile catalogs used for certification-style workflows
- broad bundles of trace/proof shapes that do not have a direct app-developer story

Hiding them first lets us reduce public identity without prematurely committing
to a refactor we may later regret.

---

## What We Should Probably Internalize

These are the strongest candidates for eventual removal from the shipped public
surface.

- harness runtime
- parity suite helpers
- scenario fixtures
- assert helpers
- certification-facing bridge utilities
- internal transaction/runtime contract markers

If these remain needed, they likely belong in:

- a support crate
- a dev-only feature
- test support
- internal tooling only

---

## Cleanup Order

We should not do this in one sweep.

### Phase 1: Product Boundary Cleanup

Safe, high-value changes:

- remove `P3` surfaces from the main facade
- stop presenting harness/certification surface as part of the product
- narrow docs around `P0`

This gives immediate DX improvement with minimal architectural churn.

### Phase 2: Specialist Namespace Cleanup

After phase 1:

- move `P2` surfaces into narrower namespaces
- reduce overload inside `facade::types` and `facade::transaction`
- tighten direct `worth_signal::diagnostics` exposure if needed

This is where the API starts feeling elegant rather than merely less noisy.

### Phase 3: True Visibility Tightening

Only after we are confident:

- make some hidden surfaces crate-private
- move some support surfaces to separate crates/modules/features
- formalize the stable public contract

This is the step that matters most if open source becomes real later.

---

## Decision Heuristic For Each Symbol

For every export, ask:

1. Does this represent a legitimate control surface in the ideal architecture?
2. Does it let users express semantics or policy, rather than internal ceremony?
3. Is it usually used together with companion methods that should really be one object?
4. Would exposing it create overlap with another public control point?
5. Would exposing it now create compatibility pressure later?
6. Is it bridge/integration infrastructure rather than core library identity?
7. Is it mainly a proof, witness, or certification artifact?

Suggested interpretation:

- `yes, yes, no, no, no, no, no` â†’ `Keep`
- `yes, yes, yes, maybe, maybe, no, no` â†’ `Condense`
- `yes, yes, maybe, maybe, yes, no, no` â†’ `Contain`
- `maybe, no, maybe, yes, yes, maybe, maybe` â†’ `Hide`
- `no, no, maybe, yes, yes, no, yes` â†’ `Internalize`

---

## Immediate Practical Recommendation

Given current uncertainty, the safest near-term plan is:

1. keep the existing implementation surface mostly intact
2. aggressively prune the **main facade**
3. reorganize docs around product journeys instead of architecture
4. defer hard privacy changes until after we validate bridge needs

Refined framing:

- Layer A should feel minimal and powerful
- Layer B should feel coherent and intentional
- Layer C should exist without polluting Layers A and B
- Layer D should stop leaking into the product identity

That gives us the best tradeoff:

- better DX now
- less accidental public commitment
- less risk of hiding something bridge work will need

---

## What This Means In Practice

Short term, "cleanup" should mean:

- namespace cleanup
- exposure cleanup
- naming cleanup
- docs hierarchy cleanup
- default-path cleanup

It should **not** initially mean:

- sweeping internal refactors just to look cleaner
- large privacy changes before bridge requirements are clearer
- deleting specialist capabilities that may still be strategically important

---

## Bottom Line

We should behave as if every public symbol is a future promise, even if the
crate never becomes open source.

That means:

- narrow the visible product first
- contain specialist machinery second
- only harden visibility boundaries once we are confident
