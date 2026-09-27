# worth-signal Milestone 4 Interior Heat Audit

> **Status:** Phase 1 audit artifact
>
> **Parent milestone:** [milestone-4.md](./milestone-4.md)
> **Access matrix:** [milestone-4-access-matrix.md](./milestone-4-access-matrix.md)

## Purpose

This document records the Phase 1 interior heat audit required by Milestone 4.
It exists to answer a stricter question than "is this field touched often?"

The question is:

```text
If this field moves into the hot lane, is its internal representation honestly
hot-shaped?
```

If the answer is no, the field does not become hot as a whole object. It either:

- remains warm with explicit hot-lane escalation
- is split into a hot header and warm side structure
- or is redesigned before promotion

## Audit Results

### `PartitionVersionMap`

Source:

- [version.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/aspect/version.rs)

Current shape:

- `global: AspectVersion`
- `partitions: BTreeMap<PartitionToken, AspectVersion>`

Heat verdict:

- `global` is hot-shaped
- `partitions` is not hot-shaped
- the type as a whole is **not** honest hot-lane storage under partition-heavy workloads

Reasoning:

- `AspectVersion` is fixed-width and cache-friendly
- `BTreeMap` introduces pointer chasing, branch-heavy traversal, and variable-size interior shape
- a node with no partition-local versions and a node with many partition-local versions currently share the same outer field type, which hides a real locality boundary

Phase 1 decision:

- do **not** treat `PartitionVersionMap` as a permanently admissible hot-lane object
- current hot-lane reads may continue to use it through narrow accessor seams while the storage split is not yet landed
- Phase 3 must either:
  1. split it into `global hot version header + warm partition side map`, or
  2. prove that representative hot workloads are overwhelmingly global-only and the warm side never pollutes those lanes

### Dirty Partition Scope Lane

Source:

- [entry.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/node/entry.rs)
- [output.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/output.rs)

Current shape:

- `SmallVec<[(Aspect, PartitionSubscription); HOT_VEC_INLINE_CAPACITY]>`
- `PartitionSubscription` contains `PartitionToken`, optional `String` detail, and `PartitionMatchMode`

Heat verdict:

- inline cardinality helps for small sets
- element payload is not fully hot-shaped because it carries owned partition/detail strings
- the lane is conditionally hot at best, not unconditionally hot

Reasoning:

- `SmallVec` is a useful cardinality optimization
- `PartitionSubscription` still embeds string-bearing partition and detail data
- frequent reads of "whether any scoped dirtiness exists" are much cheaper than frequent reads of full scoped payloads

Phase 1 decision:

- do not promote the current dirty-scope collection wholesale into a permanent hot-lane payload
- treat the likely future split as:
  1. hot scoped-dirty presence/header fact
  2. warm scoped-dirty payload collection
- representative suppression/invalidation workloads must justify any stronger promotion

### `DependencySetId` / `SubscriberSetId`

Source:

- [handles.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/handles.rs)
- [segmented.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/segmented.rs)

Current shape:

- compact `Option<NonZeroU32>` handles
- dereference into segmented stores

Heat verdict:

- the handles themselves are hot-shaped
- dereference cost depends entirely on whether the hot lane needs segment traversal

Reasoning:

- the handle representation is compact and locality-friendly
- the segmented stores are honest side structures, but following the handle into segment storage is not free
- this means the handle can live hot while the dereference remains a separate cost boundary

Phase 1 decision:

- keep the handles as candidate hot-lane residents
- do not treat handle compactness as proof that downstream segment traversal is also hot-safe
- the access matrix must distinguish "read handle only" from "dereference segment payload"

### `RuntimeArtifactState` Hot Candidate Fields

Source:

- [trace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs)

Candidate hot facts:

- `output_hash`
- `output_change`
- `recomputed`
- `dependency_count`
- `meaningful_input_changes`
- `changed_partition_count`
- `propagation_suppressed`
- `changed_scopes`

Heat verdict:

- the scalar counters and flags are hot-shaped
- `changed_scopes` is only conditionally hot-shaped because it wraps partition-scope payloads

Reasoning:

- the scalar fields are compact and stable
- `changed_scopes` is still semantically narrow, but its backing payload should be treated with the same caution as dirty partition scopes

Phase 1 decision:

- scalar runtime artifact facts remain valid hot-lane candidates
- `changed_scopes` may remain in the hot artifact only if representative workloads show it remains compact enough and avoids cold-rich payload drag
- otherwise split into `changed_scope_summary` hot fact plus warm side payload

## Closure Decisions

The following conclusions are now locked for Phase 1 closure:

- `PartitionVersionMap` is **not** approved as a permanent hot-lane object in its current shape
- dirty partition scope payloads are **not** approved as unconditional hot-lane payloads in their current shape
- set handles are approved as hot candidates, but segment dereference remains a separate locality boundary
- scalar runtime artifact facts are approved as hot candidates

## What This Blocks

These audit results block the following bad moves:

- moving `PartitionVersionMap` wholesale into a future hot store and calling the audit done
- moving dirty partition scope payloads wholesale into hot storage because they are "already inline"
- claiming that hot-lane handles prove hot-lane segment traversal

## What This Enables

These audit results enable the next correct moves:

- design `aspect_versions` as a hot header plus warm partition side map if needed
- split scoped-dirty state into hot presence/header and warm payload
- continue migrating hot read paths onto narrow accessors without prematurely freezing final physical layout
