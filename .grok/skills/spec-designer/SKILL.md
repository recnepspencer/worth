---
name: spec-designer
description: Design or revise WORTH milestone specifications and roadmap entries. Use before implementation planning to decide the architectural destination, authority model, public contracts, phase progression, destination topology, documentation obligations, roadmap handoff, and proof that will expose dishonest or incomplete implementations.
---

# Spec Designer

Design the architecture that governs the milestone.

Do not implement production code unless the user also requests implementation.

A specification must settle the decisions that implementation is not allowed to improvise. Finish only when implementation planning can determine the edit sequence without rediscovering the architecture and QA can derive its guarantees without inventing intent.

## Ground the design

Read the repository instructions, every coding guideline, the target roadmap, and the existing target specification.

Inspect the real production boundary:

- authority producers and consumers
- public facades and composition roots
- types, effects, persistence, and lifecycle
- ordinary, diagnostic, migration, and reconstruction paths
- tests and mechanical enforcement
- predecessor guarantees and successor dependencies

Read adjacent documents when they govern an inherited contract or roadmap handoff. Do not impose an arbitrary reading boundary that leaves the architecture misunderstood.

Treat current code as evidence of present reality, not authority over the destination.

Recover:

1. the milestone's central claim
2. why it belongs here in the roadmap
3. what must already be true
4. what later work will rely on
5. the complete causal closure required to make the claim true

Exclude work with an independent telos. Include necessary foundations even when they cross the initially expected crate or subsystem boundary.

## Build the adversarial courtroom

When the milestone makes a runtime, integration, lifecycle, authority, recovery, compatibility, or performance claim, make its decisive proof genuinely adversarial.

Do not write "test under load," "handle failures," "verify recovery," or similar decorative adversity.

Identify the most plausible implementation that would satisfy the happy path while violating the milestone's real intent. Then design a production-valid world specifically to make that implementation fail.

Combine hostile conditions that attack the same architectural weakness:

- maximum relevant scale or fan-out
- worst lawful ordering or concurrency
- cancellation at the most damaging effect boundary
- crash after the most ambiguous partial effect
- duplicate, delayed, stale, or reordered delivery
- exhausted queues, memory, time, or admission budgets
- incompatible versions during coexistence
- authority loss, counterfeit authority, or scope widening
- destroyed derived state followed by reconstruction
- diagnostic, replay, or migration pressure against the ordinary path

Quantify the conditions whenever the claim permits it. Name the exact crash point, schedule, scale axis, resource bound, compatibility window, or amplification. State a cost condition as a growth class or as the equality of two ways of doing the same work, never as an exact count, and call for a counter only when it is proven necessary.

The courtroom must specify:

- the real production entry surface and composition root
- the causally valid world and authority provenance
- the hostile sequence
- the required typed outcomes
- the effects that may and may not occur
- the state that must survive
- the cost growth class, where a caller would feel it
- independent observations of the result
- the defective implementation the proof must convict

Make the proof mutation-sensitive: bypassing, inverting, deleting, weakening, misrouting, or stale-reusing the disputed mechanism must turn the evidence red.

If the claim crosses a real product boundary, use the real boundary. An in-memory reenactment is not end-to-end evidence.

Do not manufacture an end-to-end courtroom for a claim that is honestly local. Use model, property, exhaustive-transition, compile-fail, dependency-enforcement, deletion-inventory, or documentation-surface proof when that is stronger.

If the central claim cannot be made falsifiable, the milestone is not designed.

## Design backward from the proof

Derive the required architecture from what must survive the courtroom.

Freeze every decision whose omission would let implementation change meaning, authority, truth status, lifecycle, failure behavior, recovery, compatibility, or contractual cost.

Specify, where relevant:

- authoritative and derived truth
- authority grants, consuming proofs, and non-authorities
- constrained types and compiler-visible transitions
- public facades and dependency direction
- canonical artifacts and derived views
- effect ownership and commit posture
- typed denial, cancellation, partial, and indeterminate outcomes
- recovery and managed-resource lifecycle
- compatibility, migration, cutover, and deletion
- ordinary versus reconstructive or diagnostic cost
- caller and operator DX
- mechanical enforcement against recurrence

For every important type or responsibility, state what it proves, who constructs it, what it authorizes, what it cannot authorize, and what consumes it.

Show real code for caller-facing DX. Use the earliest honest enforcement boundary. Prefer unrepresentable states, compiler denial, and dependency enforcement over runtime checks; prefer typed runtime admission over convention; use tests to prove enforcement rather than substitute for it.

Leave product-equivalent private mechanics to implementation planning.

## Require the destination topology

Every specification must include a concrete destination directory and module tree for the current slice and its committed successors. A prose description alone is not a directory skeleton plan.

Anticipate growth from the roadmap, accepted specifications, and known responsibility families. Establish stable semantic and authority axes before growth arrives, not after the code has already been flattened around the first implementation.

A one-file directory is explicitly permitted and required when appropriate if roadmap-backed future responsibilities will join it along the same stable semantic axis. Do not flatten it while waiting for a second file. Current population count is not structural evidence.

Do not create empty placeholders for uncommitted possibilities. Future-aware topology must be justified by known meaning, authority, lifecycle, failure, scale, ownership, replacement, or roadmap commitments, not generic extensibility.

Separate responsibilities when collapsing them would couple:

- meaning
- authority
- truth source
- lifecycle
- failure behavior
- scale
- ownership
- replacement

Recursively refine the tree until the remaining constituents genuinely share structural fate. Relatedness and similar representation are not cohesion. File count is not an optimization target.

For every significant directory or module boundary, identify:

- its dominant structural axis
- its semantic or authority owner
- what belongs there and what is excluded
- its truth and dependency direction
- its stable public facade
- the current responsibility that populates it
- the committed siblings, children, strategies, adapters, or versions expected to enter there
- the visibility, dependency, export, or automated enforcement that preserves it

Mark files and directories as existing, created, moved, replaced, removed, or committed-successor destinations. A committed successor may appear in the planned tree without requiring an empty placeholder to be created now.

Place stable meaning and authority above volatile mechanisms. Keep authoritative, derived, diagnostic, reconstructed, generated, migration, and legacy responsibilities structurally directional. Make external effect boundaries spatially locatable. Place cross-domain orchestration above participating domains rather than inside one of them.

Shared placement is lawful only when the contents share semantic authority, lifecycle, failure behavior, and replacement fate. Similar representation or convenient reuse is insufficient.

The destination topology must make anticipated growth additive. A committed successor must enter by adding a sibling, child, strategy, adapter, or versioned boundary, not by splitting a god file, renaming an established directory, moving the facade, reversing dependency direction, or reclassifying existing responsibilities.

Identify flat, bucket, and cross-authority placements the destination forbids. Use the file composition laws within the tree. Do not optimize for either fewer files or more files. Optimize for semantic predictability: the next correct responsibility must have an obvious home before it arrives.

## Treat documentation as a deliverable

Require documentation when the milestone changes a public capability, caller workflow, operator responsibility, architectural concept, security boundary, recovery procedure, compatibility contract, or migration path.

Do not write "update documentation" as an unnamed task. Name:

- the continuing audience
- the authoritative document to create, revise, or remove
- the public facade or operational surface it explains
- the examples, failures, lifecycle, recovery, security, or compatibility semantics it must cover
- how it will be checked against the real implementation

Prefer revising an existing authoritative document over creating a competing explanation. Remove or correct documentation made false by the milestone.

When the requested specification work includes authoring those documents, write them. Use `feature-doc-writer` for developer-facing feature documentation.

Do not create milestone residue, duplicate summaries, speculative guides, or closeout documents without a durable audience.

## Derive the phases

Order phases by proof and authority dependency.

Split a phase when:

- authority changes
- a stronger proof-bearing artifact appears
- effects begin
- compatibility or coexistence begins
- the public facade becomes real
- cutover or deletion becomes possible
- the proof boundary changes

Keep work together when splitting it would leave competing authority, a dishonest facade, or an unprovable intermediate state.

For each phase, state:

- what becomes true
- what proof or authority it consumes
- what architecture it establishes
- what it mechanically forbids
- what evidence it enables
- what the next phase may trust

Do not require fixed test counts or ceremonial subsections. Every phase must earn its existence by advancing the decisive proof, preserving an inherited guarantee, or establishing the successor handoff.

## Write one governing design

Write one decision, not a menu.

Put the adversarial courtroom before the phase plan. Let the reader see what the architecture must survive before explaining how it is built.

A useful specification usually contains:

1. Goal and roadmap placement
2. Current boundary
3. Adversarial constraint and decisive proof
4. Product decision lock
5. Architectural destination
6. Required directory and module skeleton
7. Ordered phase plan
8. Documentation deliverables
9. Must ship and must preserve
10. Acceptance evidence
11. Successor handoff

Follow local document style when it remains honest. Do not let a template dilute the design.

Update the roadmap when sequencing, ownership, or handoffs change. Revise or remove documents made false by the milestone.

## Refuse weak closure

Before finalizing, attack the specification itself:

- Can a fake authority still open the governed path?
- Can a happy-path implementation pass while recovery is false?
- Can derived state quietly become authoritative?
- Can replay, diagnostics, migration, or explanation tax the ordinary lane?
- Can concurrency or partial effects create an unowned state?
- Can the old path survive beside the new authority?
- Can the next milestone enter only by moving today's architecture?
- Does the planned tree flatten a known responsibility family while waiting for a second file?
- Does any location combine responsibilities that do not share structural fate?
- Can the proposed evidence pass for the wrong reason?
- Can implementation still make a product or architectural decision the specification should have made?
- Has a relevant documentation responsibility been reduced to an unnamed "update docs" task?

If any answer is yes, the specification is not done.
