# WORTH Relational Canonical Surface Spec

## Purpose

This document defines the positive canonical public shape of
`worth-relational`.

This is not just about what a human will memorize.

It is also about what an AI agent will reliably reach for when it is trying to
use the crate without getting lost in internal seams.

That means the optimization target is:

- obvious top-level doors
- clear job-shaped verbs
- stable escalation paths
- fewer weird side seams
- less accidental runtime spelunking

If an agent can use the crate correctly after a fast read of the public docs,
the surface is probably doing its job.

If the agent has to infer architecture from a pile of helper accessors and
internal-feeling names, the surface is still bad.

---

## Product Center Of Gravity

`worth-relational` should present itself as:

- authoritative truth runtime for transactional graph state
- with strong history, inspection, publication, and replay semantics

That is the center.

Not:

- a bag of subsystems
- a certification scaffold
- a merge lab
- a bridge substrate
- a storage toolkit

Those things are real.

They just are not the first thing the public shape should teach.

---

## Canonical Agent-Friendly Shapes

The surface should be optimized for an agent doing something like:

1. construct a runtime
2. mutate truth
3. read truth
4. inspect what happened
5. escalate into history, replay, merge, or specialist lanes only when needed

The public shape should make that progression obvious.

The agent should not have to guess:

- whether to start from `RelationalRuntimeApi` or some lower constructor
- whether publication, history, and inspection are separate worlds
- whether a public accessor is a real lane or just leaked internals
- whether a specialist helper is the official path or a trap

---

## Canonical Public Memory Shapes

These are the shapes the docs and facade should teach as the default usage
story.

## 1. Canonical Import Path

Primary path:

- `worth_relational::facade`

Policy:

- the facade is the real public contract
- users and agents should not need to import through internal crate structure
- `RelationalRuntimeApi` may stay as a convenience door, but not as a rival
  boundary

Target memory shape:

```rust
use worth_relational::facade::*;
```

Not:

- mixed imports from `facade`, `presentation`, and runtime internals
- guessing which helper module is secretly the real API

## 2. Canonical Production Setup Flow

The normal setup story should revolve around:

- `RelationalRuntimeApi::builder()`
- `profile(...)`
- essential builder refinement
- `build()`

Target production shape:

```rust
let runtime = RelationalRuntimeApi::builder()
    .profile(RelationalRuntimeProfile::Default)
    .schema_registry(schema_registry)
    .build();
```

Target property:

- one obvious construction path
- profile-first defaults
- schema registration is explicit
- deeper knobs remain available, but do not crowd the first path

Architectural rule:

- config must mirror architecture
- builder power stays real
- setup should still read like one declaration, not like a scavenger hunt

AI-agent rule:

- an agent should be able to create a normal runtime by following one obvious
  setup pattern
- it should not need to discover hidden setup conventions by bouncing across
  modules

## 3. Canonical Truth-Mutation Flow

The normal mutation story should revolve around:

- begin transaction
- push one or more intent batches
- commit
- inspect commit result

Target shape:

```rust
let mut tx = runtime.begin_transaction(TransactionOptions::default());

tx.push_batch(worker_batch);

let result = tx.commit()?;
```

Target property:

- mutation feels like one coherent truth-authority workflow
- transaction, commit result, history effect, and publication effect feel like
  one product
- deeper staging and admission phases stay available underneath, but do not own
  the main story

Condensation policy:

- `plan_bulk_mutation_batch(...)` stays visible because it helps condense a hard
  workflow
- the `admit_*` trio stays available, but clearly deeper than the normal path

## 4. Canonical Current-Truth Read Flow

The normal read story should revolve around:

- runtime read access
- query when needed
- immutable visibility semantics underneath the surface

Target shape to drive toward:

```rust
let current = runtime.visibility_reads();
let query = runtime.query(...);
```

What matters here is not the exact final method spelling yet.

What matters is the boundary shape:

- current-truth reads should feel distinct from:
  - history
  - replay
  - inspection
  - merge
- immutable read semantics should be explicit in the mental model
- agents should not need to reach into storage helpers for ordinary reads

Canonical rule:

- current truth first
- historical truth only when explicitly requested

## 5. Canonical History And Replay Escalation Path

History and replay should be taught as an escalation path, not as ambient noise.

Canonical escalation:

1. read current truth
2. ask history what changed
3. ask replay to verify or reconstruct

Target shape:

```rust
let history = runtime.history_access();
let latest = history.latest_commit();

let replay = runtime.replay_authority().replay_commit(request);
```

Target property:

- history access is the normal door into the past
- replay is a specialist escalation after history, not a rival first-class
  everyday path
- agents should learn one sequence:
  current truth -> history -> replay

## 6. Canonical Inspection And Diagnostics Flow

Inspection and diagnostics should be organized around questions, not artifact
vocabulary.

The normal operator flow should revolve around:

- inspect a commit
- inspect recent commits
- inspect a branch head
- inspect graph or connectivity state
- inspect structural identity
- inspect retention state
- read publication diagnostics when needed

Target shape:

```rust
let inspection = runtime.inspection_access();
let commit = inspection.inspect_commit(commit_id)?;
let recent = inspection.inspect_recent_commits(window)?;

let publication = runtime.publication_access();
let diagnostics = publication.diagnostics_since(cursor);
```

Target property:

- inspection answers operator questions directly
- publication exposes publication-facing outputs directly
- diagnostics are productized as jobs, not just raw artifact piles

AI-agent rule:

- an agent should know where to go when it wants to answer:
  - "what changed?"
  - "what is wrong?"
  - "what got published?"
  - "what is pinned or retained?"

## 7. Canonical Specialist Merge Flow

Merge should remain explicitly specialist.

But it should still feel guided.

Canonical shape:

```rust
let prepared = runtime.prepare_merge_execution(request)?;
let outcome = runtime.execute_prepared_merge(prepared)?;
```

And for planning-focused work:

```rust
let planning = runtime.merge_access().inspect_planning_scope(request)?;
```

Target property:

- planning and execution feel like one specialist workflow
- merge does not require agents to bounce through random history helpers and
  runtime backdoors
- `MergeAccess::runtime()` should not exist in the canonical shape

## 8. Canonical Specialist Strategy Flow

Commit strategies are real specialist power and should stay available.

Canonical shape:

```rust
let request = runtime.commit_strategies().canonicalize_request(&raw)?;
let execution = runtime.commit_strategies().execute(&request, &snapshot)?;

let mut authority = runtime.commit_strategies_authority();
let lowered = authority.lower_execution(&request, &execution, options)?;
let validated = authority.validate_lowered_plan(lowered)?;
let commit = authority.execute_validated_commit(validated)?;
```

Target property:

- there is one obvious strategy pipeline
- read and authority phases are explicit
- this remains specialist, but it is honest and usable

## 9. Canonical Durability Flow

Durability should be taught as a contained operational lane.

Canonical shape:

```rust
let plan = runtime
    .durability_access()
    .recovery_plan(RecoveryVerificationMode::AuditRecoveryVerification);

runtime.durability_authority().recover(plan)?;
```

Target property:

- durability reads and writes are explicit
- recovery is the canonical authority verb
- checkpoint and compaction remain available as contained operations

## 10. Canonical Validation And Certification Flow

Validation is real architecture and should not be left as a hidden helper seam.

Canonical shape to drive toward:

```rust
let validation = runtime.invariant_access();
let certification = validation.certification_state();

runtime.certify_current_state()?;
```

Target property:

- read-side invariant inspection and authority-side certification are distinct
- `certify_current_state()` stays as the top-level authority verb
- invariant access becomes an explicit contained lane instead of an accidental
  public seam

---

## Canonical Layering

This is the public shape we should be driving toward.

## Layer 1: Primary Daily-Use Runtime

Should own:

- runtime setup
- schema registration needed for normal use
- identity vocabulary
- transaction entry and commit
- current-truth read path
- query path

This is the layer an AI agent should reach for first almost every time.

## Layer 2: Guided Operational Readback

Should own:

- history access
- inspection access
- publication access
- config inspection
- validation readback

This is the layer for:

- what changed
- what published
- what is wrong
- what is retained

## Layer 3: Contained Specialist Power

Should own:

- merge
- replay
- durability
- commit strategies
- simulation / compiled artifacts
- visibility and retention authority

These remain public.

They just should not be mistaken for day-one runtime usage.

## Layer 4: Not Part Of The Public Story

Should not define the public product boundary:

- harness-first support scaffolding
- substrate mutation helpers
- empty public shells
- runtime backdoors

That includes the cleanup calls already made in
[`dx_boundary_cleanup_list.md`](./dx_boundary_cleanup_list.md).

---

## Canonical Door Policy

Every public lane should answer one of these questions cleanly:

- how do I build the runtime?
- how do I mutate truth?
- how do I read current truth?
- how do I inspect what happened?
- how do I go back in time?
- how do I recover or verify?
- how do I do specialist merge or strategy work?

If a public accessor does not clearly answer one of those, it probably should
not be public.

This is especially important for AI-agent usage.

An agent does much better with:

- one obvious door per job

Than with:

- five half-overlapping helpers that only make sense after internal context

---

## Role Of `RelationalRuntimeApi`

`RelationalRuntimeApi` should stay.

Its job is:

- to be the obvious construction entry
- to reduce setup ambiguity
- to give agents and humans a stable “start here” anchor

Its job is not:

- to become a second rival public boundary
- to own every workflow forever

Doctrine:

- `RelationalRuntimeApi::builder()` is the canonical setup entry
- the facade remains the canonical product boundary
- runtime verbs still live on the runtime

---

## Canonical Naming Direction

The public shape should prefer names that teach jobs and boundaries.

Good direction:

- `history_access`
- `inspection_access`
- `publication_access`
- `prepare_merge_execution`
- `execute_prepared_merge`
- `certify_current_state`

Bad direction:

- names that only make sense after reading internal architecture
- public seams that expose empty wrappers
- helper names that imply “official lane” without actually being one

Specific direction from this pass:

- keep specialist power, but do not let leaked helper shells pretend to be real
  lanes
- promote real specialist lanes explicitly if they have a coherent public job

---

## Canonical Summary

If this succeeds, the crate should feel like this:

1. import from `worth_relational::facade`
2. build a runtime through `RelationalRuntimeApi::builder()`
3. mutate truth through transactions
4. read current truth through the normal read path
5. inspect or diagnose through inspection and publication access
6. escalate to history, replay, merge, durability, validation, or strategies
   only when needed

And the agent-facing version of that same summary is:

1. there is one obvious setup door
2. there is one obvious mutation door
3. there is one obvious read door
4. there is one obvious inspection door
5. specialist lanes are explicit, not accidental

Anything that weakens those shapes should be treated as DX debt before bridge
work hardens around it.
