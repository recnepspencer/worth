# WORTH Relational DX Boundary Cleanup List

## Purpose

This is the cleanup list for the public seam methods that currently sit in a
bad middle state.

They are public.

They expose real or half-real helper lanes.

But they are not all properly integrated into the explicit facade story.

This is exactly the kind of thing that becomes sad later if we leave it alone
while bridge work starts hardening around it.

So this doc answers one simple question:

- for each leaky seam, do we `Promote` it or `Remove` it?

Not in theory.

As a concrete pre-bridge cleanup call.

---

## Decision Rule

Use the repo standards, not social heuristics.

We promote a seam when:

- it exposes real architectural power
- it has a coherent public job
- the capability would be worse if forced through some unrelated lane

We remove a seam when:

- it returns an almost-empty helper
- it exists mostly as internal decomposition leakage
- it weakens the facade without teaching a distinct public concept

We do not hide hard things just because they are hard.

We only remove seams that do not currently earn public boundary status.

---

## Cleanup Table

| Seam | Current State | Call | Why |
| --- | --- | --- | --- |
| `RelationalRuntime::publication_authority()` | public accessor to a helper whose useful verbs are mostly `pub(crate)` internals | `Remove` | there is not a real public authority lane here yet |
| `RelationalRuntime::storage_authority()` | public accessor to a helper whose verbs are almost entirely internal machinery | `Remove` | this is internal substrate authority leaking through the boundary |
| `RelationalRuntime::retention_authority()` | public accessor to a real helper with real verbs | `Promote` | retention is a real authority lane with a coherent public job |
| `RelationalRuntime::visibility_reads()` | public accessor to a public helper not represented in the explicit facade story | `Promote` | visibility reads are real read semantics, not accidental support trivia |
| `RelationalRuntime::lineage_access()` | public accessor to an almost-empty helper shell | `Remove` for now | lineage is real, but this particular seam is not yet a real public lane |
| `RelationalRuntime::lineage_authority()` | public accessor to an almost-empty helper shell | `Remove` for now | same as above |
| `RelationalRuntime::simulation_access()` | public accessor to a helper with real public verbs | `Promote` | compiled artifact reads are architecturally real and distinct |
| `RelationalRuntime::simulation_authority()` | public accessor to a helper with a real public verb | `Promote` | compiled artifact creation is a real specialist authority lane |
| `RelationalRuntime::invariant_access()` | public accessor to a helper with meaningful public verbs, but not part of explicit facade story | `Promote` | validation and certification are real architecture and deserve an honest lane |
| `MergeAccess::runtime()` | specialist helper handing back the whole runtime | `Remove` | this is a pure boundary backdoor |

---

## The Removals

## `publication_authority`

Source:
[`authority.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/publication/logic/authority.rs)

Call:
`Remove`

Why:

- the helper mostly exposes internal publication machinery through `pub(crate)`
- the public seam makes it look like there is a real public publication write
  lane when there basically is not
- keeping the accessor public weakens the facade contract more than it helps

Cleanup move:

- make `RelationalRuntime::publication_authority()` non-public
- keep top-level public publication verbs where they actually belong
- only re-promote later if a real public publication authority workflow
  materializes

## `storage_authority`

Source:
[`authority.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/storage/logic/authority.rs)

Call:
`Remove`

Why:

- this helper is basically internal substrate mutation and retention machinery
- it does not represent a coherent product-facing public concept
- leaving it public teaches the wrong boundary

Cleanup move:

- make `RelationalRuntime::storage_authority()` non-public
- keep storage as a read or support lane, not a public mutation lane

## `lineage_access`

Source:
[`mod.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/lineage/logic/access/mod.rs)

Call:
`Remove` for now

Why:

- the helper currently has no actual public verbs
- the public seam claims a lineage access lane exists, but the lane is still a
  shell
- lineage itself is real, but this seam is not yet an honest product door

Cleanup move:

- make `RelationalRuntime::lineage_access()` non-public for now
- if we later expose lineage jobs, do it as a real facade lane with named
  read verbs

## `lineage_authority`

Source:
[`mod.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/lineage/logic/authority/mod.rs)

Call:
`Remove` for now

Why:

- same problem as `lineage_access`
- public seam exists before the real public lane exists

Cleanup move:

- make `RelationalRuntime::lineage_authority()` non-public for now
- reintroduce only when there is an actual public authority workflow to teach

## `MergeAccess::runtime`

Source:
[`mod.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/merge/logic/mod.rs)

Call:
`Remove`

Why:

- this is a classic boundary backdoor
- once you are in a specialist helper, handing back the whole runtime defeats
  the point of the boundary
- it is not a capability, it is an escape hatch

Cleanup move:

- remove `MergeAccess::runtime()`
- keep merge workflows expressed through explicit merge verbs instead

---

## The Promotions

## `retention_authority`

Source:
[`retention_authority.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/visibility/retention/retention_authority.rs)

Call:
`Promote`

Why:

- this is not fake power
- it has real public verbs with real jobs:
  - `inspect_plan`
  - `run_pass`
- retention is already part of the runtime architecture
- this should become an explicit contained authority lane instead of a weird
  accidental runtime seam

Promotion move:

- make retention an explicit contained public lane in the facade story
- place it in a dedicated `retention` lane that sits next to inspection and
  durability instead of disappearing inside either one
- teach it as visibility-retention authority, not as random runtime trivia

## `visibility_reads`

Source:
[`mod.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/visibility/materialization/read_records/mod.rs)

Call:
`Promote`

Why:

- immutable visibility reads are not support trivia
- they are one of the core architectural truths of the system
- a public read helper here makes sense, but only if the facade admits it on
  purpose

Promotion move:

- promote this into the official current-truth read lane under runtime
- connect it to immutable-read semantics in the docs
- keep the current method spelling as implementation detail for now, but treat
  the product lane as `read_truth`

## `simulation_access`

Source:
[`access.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/simulation/logic/access.rs)

Call:
`Promote`

Why:

- this helper actually has real public verbs:
  - `compiled_artifact`
  - `compiled_artifact_compatibility`
- compiled execution artifacts are not random internals
- they are a real specialist subsystem

Promotion move:

- surface this as an explicit contained `compiled_artifacts` lane
- keep it specialist
- do not leave it as a runtime side seam

## `simulation_authority`

Source:
[`authority.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/simulation/logic/authority.rs)

Call:
`Promote`

Why:

- `compile_execution_artifact` is a real public authority verb
- if compiled execution artifacts matter architecturally, this should be honest
  about it

Promotion move:

- give simulation authority the write side of the `compiled_artifacts` lane
- keep it clearly separate from the main transaction or merge story

## `invariant_access`

Source:
[`invariant_access.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/validation/logic/invariant_access.rs)

Call:
`Promote`

Why:

- this helper has real public verbs with clear jobs:
  - `mutation_sensitive_state`
  - `snapshot_publication_state`
  - `certification_state`
- validation and certification are central architectural truths here
- keeping only `certify_current_state()` public while hiding the read-side lane
  is not actually more honest

Promotion move:

- promote invariant access as a contained `validation` lane
- keep `certify_current_state()` as the top-level authority verb
- document the difference between read-side invariant inspection and authority
  enforcement

---

## Resolved Lane Ownership

These placement calls are now resolved for the DX program:

1. `visibility_reads()` belongs to the primary runtime read lane
   - product direction: `read_truth`
2. `invariant_access()` belongs to a contained validation lane
   - product direction: `validation`
3. `simulation_access()` and `simulation_authority()` belong to a contained
   compiled-artifact lane
   - product direction: `compiled_artifacts`
4. `retention_authority()` belongs to a dedicated contained retention lane
   - product direction: `retention`

These names are the product-story owners even where the literal Rust method
names are not final yet.

---

## Suggested Cleanup Order

If we want the best sequencing before bridge work:

1. remove `MergeAccess::runtime`
2. remove `publication_authority`
3. remove `storage_authority`
4. remove `lineage_access`
5. remove `lineage_authority`
6. define promoted lanes for:
   - `retention_authority`
   - `visibility_reads`
   - `simulation_access`
   - `simulation_authority`
   - `invariant_access`

That order shrinks the fake boundary first, then gives real power lanes a clean
place to live.

---

## What This Unlocks

Once this cleanup list is accepted, the canonical surface spec can stop being
vague.

It can say:

- these are the real primary doors
- these are the real contained specialist lanes
- these seams are gone
- these other seams graduated into explicit facade concepts

That is the point where the public shape becomes stable enough to build bridge
work against without teaching the wrong thing.
