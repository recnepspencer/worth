# WORTH Relational DX Boundary Spec

## Purpose

This document translates the Relational DX work into a concrete public boundary
target.

It answers:

- what should be first-class
- what should stay public but be clearly contained
- what should be promoted into honest lanes
- what should leave the public story

This is the code-shaping boundary doc.

It is not just philosophy.

And it is not optimized for what a patient engineer can eventually infer after
reading a lot of code.

It is optimized for what a human or AI agent can reach for fast and use
correctly.

---

## Daily-Use Principle

The public boundary should optimize for the normal jobs first.

Those jobs are:

1. build the runtime
2. write truth
3. read truth
4. inspect what happened
5. go back in time when needed
6. escalate to merge, replay, durability, or strategy work only when needed

If those jobs are not smooth, the API is not ready, no matter how strong the
deep specialist surfaces are.

That is the real standard here.

---

## Agent-Use Principle

Relational should be designed so an AI agent can use it without learning the
internal crate decomposition first.

That means:

- one obvious door per job
- stable top-level verbs
- minimal fake seams
- specialist lanes that are explicit instead of ambient

Bad boundary outcome:

- the agent sees six similarly plausible helper accessors and picks one by
  vibes

Good boundary outcome:

- the agent sees one obvious job-shaped door and only escalates deeper when the
  task really requires it

---

## Canonical Public Layers

## Layer 1: Primary Runtime Story

This is what should define the product.

Required characteristics:

- short import story
- one obvious setup flow
- one obvious truth-mutation flow
- one obvious truth-read flow
- obvious inspection door

This is where names like these make sense:

- `build_runtime`
- `write_truth`
- `read_truth`
- `inspect_what_happened`

Those do not have to be the literal final method names everywhere.

But they are the right standard.

If a name does not make the job obvious at about that level, it should be
viewed suspiciously.

## Layer 2: Guided Operational Readback

This is for:

- history
- publication
- diagnostics
- inspection
- validation readback
- config inspection

Required characteristics:

- easy to discover after Layer 1
- job-shaped
- not mixed with substrate or specialist internals

## Layer 3: Contained Specialist Power

This is for:

- merge
- replay
- durability
- commit strategies
- simulation / compiled artifact workflows
- retention authority
- visibility-specialized read or pin workflows

Required characteristics:

- real
- public
- clearly specialist
- not competing with the first five minutes of usage

## Layer 4: Not Public Product Boundary

This is not part of the published product story.

Includes:

- harness support
- public backdoors
- empty helper shells
- substrate mutation helpers
- accidental internal seams

---

## Final Intended Top-Level Identity

The stable public identity should converge toward:

- `worth_relational::facade`
- `RelationalRuntimeApi` as the obvious construction door
- job-shaped runtime lanes underneath

Everything else should either:

- reinforce that shape

Or:

- get contained
- get renamed
- get removed

---

## Target Boundary Shape

## Primary Daily-Use Surface

These are the things that should feel primary.

### Setup

Should feel like:

- `RelationalRuntimeApi::builder()`
- `profile(...)`
- `schema_registry(...)`
- `build()`

Boundary policy:

- setup is primary
- profile-first setup is primary
- builder flattening is a problem to condense, not a reason to hide power

### Write Truth

Should feel like:

- transaction entry
- batch push
- commit

Target public story:

- `begin_transaction(...)`
- `push_batch(...)`
- `commit()`

Boundary policy:

- transaction is the canonical write-truth door
- commit results stay prominent
- deeper planning and admission phases stay available but not primary

### Read Truth

Should feel like:

- current truth first
- query when needed
- immutable visibility semantics underneath the read path

Boundary policy:

- ordinary current-truth reads should not push users toward storage helpers
- if `visibility_reads()` is promoted, it should become part of this official
  read-truth lane

### Inspect What Happened

Should feel like:

- inspect commit
- inspect recent commits
- inspect branch head
- inspect diagnostics or publication outputs

Boundary policy:

- inspection and publication should answer operator questions directly
- raw artifact nouns should not define the user-facing boundary

---

## Contained Operational Surface

These are real and public, but should not define the first impression.

## History

Should stay public.

Should be taught as:

- the first escalation after current truth

Not as:

- a rival first-class day-one surface

## Publication

Should stay public.

Should be taught as:

- published outcomes
- patch and subscriber streams
- diagnostics readback

Not as:

- a random pile of CDC nouns

## Inspection

Should stay public.

Should be taught as:

- a question-shaped operational lane

Not as:

- a giant inspection noun bucket

## Validation Readback

`invariant_access()` should be promoted into a contained validation lane.

It should be taught as:

- validate current state
- inspect mutation-sensitive state
- inspect certification state

Not as:

- a leaked internal helper

## Config Inspection

Resolved config should stay public and easy to inspect.

But the config story should be grouped by subsystem intent instead of being
taught as one flat knob heap.

---

## Specialist Surface

These remain public and real.

They just stop pretending to be normal first-path usage.

## Merge

Public.

Specialist.

Guided.

Preferred public story:

- inspect planning scope
- prepare merge execution
- execute prepared merge

Boundary rule:

- no backdoor like `MergeAccess::runtime()`

## Replay

Public.

Specialist.

Escalates naturally from history.

Preferred public story:

- replay after history when verification or reconstruction is needed

## Durability

Public.

Specialist.

Operational.

Preferred public story:

- read durability state through access
- act through checkpoint, recover, compact

## Commit Strategies

Public.

Specialist.

Pipeline-shaped.

Preferred public story:

- canonicalize
- execute
- lower
- validate
- commit

## Simulation / Compiled Artifact Lane

Promote this.

It has coherent public jobs:

- read compiled artifact
- check compiled artifact compatibility
- compile execution artifact

Boundary rule:

- if this stays public, it needs to be a named contained lane, not a runtime
  side seam

## Retention Authority

Promote this.

It has coherent public jobs:

- inspect retention plan
- run retention pass

Boundary rule:

- this must become an explicit lane if it stays public

---

## Boundary Promotions

These are the seams that should graduate into explicit public lanes.

## `visibility_reads`

Promote into the canonical read-truth story.

Reason:

- immutable visibility semantics are not support trivia
- they are part of the architecture the product should actually teach

Resolved product direction:

- this belongs to the primary runtime read lane
- the product story should teach this lane as `read_truth`
- the current implementation seam is still `visibility_reads()`

## `invariant_access`

Promote into a contained validation lane.

Reason:

- it has real public jobs
- validation is part of the architecture
- hiding the read side while keeping certification public is weird

Resolved product direction:

- this belongs to a contained validation lane
- the product story should teach this lane as `validation`
- the current implementation seam is still `invariant_access()`

## `simulation_access` / `simulation_authority`

Promote into a contained specialist lane.

Reason:

- these are real compiled-artifact jobs, not fake power

Resolved product direction:

- these belong to a contained specialist compiled-artifact lane
- the product story should teach this lane as `compiled_artifacts`
- the current implementation seams are still `simulation_access()` and
  `simulation_authority()`

## `retention_authority`

Promote into a contained operational lane.

Reason:

- retention planning and execution are real public jobs

Resolved product direction:

- this belongs to a dedicated contained retention lane
- it should sit next to inspection and durability, not disappear inside either
  one
- the current implementation seam is still `retention_authority()`

---

## Boundary Removals

These should leave the public story.

## `publication_authority`

Remove from public boundary.

Reason:

- there is no real public publication authority lane here yet
- the public seam implies more coherence than actually exists

## `storage_authority`

Remove from public boundary.

Reason:

- this is internal substrate control leaking outward

## `lineage_access`

Remove for now.

Reason:

- current public shell is basically empty
- lineage is real, but this seam is not yet an honest lane

## `lineage_authority`

Remove for now.

Reason:

- same story as `lineage_access`

## `MergeAccess::runtime`

Remove.

Reason:

- pure boundary backdoor

## `facade::harness`

Remove from the main product boundary.

Reason:

- support and certification scaffolding should not shape product identity

---

## Module-Level Boundary Calls

This is the compact layer map.

| Surface | Boundary Role | Policy |
| --- | --- | --- |
| `runtime` | `Primary` | center of product, condense hard |
| `transactions` | `Primary` | center of write-truth story |
| `query` | `Primary` | center of read-truth story |
| `identity` | `Primary` | keep visible and stable |
| `schema` | `Primary` | keep central, condense heavily |
| `payloads` | `Primary` | keep simple and visible |
| `config` | `Primary` | keep central, regroup by subsystem intent |
| `errors` | `Primary` | keep stable |
| `history` | `Contained` | first escalation after current truth |
| `inspection` | `Contained` | operator question lane |
| `publication` | `Contained` | publication outcome lane |
| `diagnostics` | `Contained` | should become more job-shaped |
| `snapshots` | `Contained` | compact operational support |
| `indexes` | `Contained` | real support lane |
| `storage` | `Contained` | keep as support, not center |
| `symbols` | `Contained` | support vocabulary |
| `merge` | `Specialist` | guided specialist lane |
| `replay` | `Specialist` | guided specialist lane |
| `durability` | `Specialist` | operational specialist lane |
| `commit_strategies` | `Specialist` | strategy pipeline lane |
| `lineage` | `Specialist` | keep noun surface, remove fake empty helper seams |
| `harness` | `Removed` | not part of product boundary |

---

## Naming Test

We should pressure-test candidate names with this question:

- if an AI agent sees this name in autocomplete with no extra context, will it
  pick the right thing?

Good direction:

- `read_truth`
- `write_truth`
- `inspect_what_happened`
- `prepare_merge_execution`
- `execute_prepared_merge`
- `certify_current_state`

Weak direction:

- names that only make sense after internal architecture context
- names that imply a coherent public lane when the helper is actually a shell
- names that hide the job behind subsystem jargon

This is not about dumbing things down.

It is about making the boundary legible.

---

## Release Test For Boundary Quality

The boundary is good enough only if a user or agent can answer these instantly:

1. Where do I build the runtime?
2. Where do I write truth?
3. Where do I read truth?
4. Where do I inspect what happened?
5. Where do I go when I need history or replay?
6. Where do I go when I need merge or strategy work?
7. Which public seams are real lanes, and which ones are gone?

If the answer is still:

- "it depends which helper you happened to discover first"

Then the boundary is still wrong.
