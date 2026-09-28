# WORTH Relational DX Condensation Map

## Purpose

This document says where raw public capability should collapse into a smaller
number of guided workflows.

This is the Phase 3 map.

It exists because "condense" is too vague on its own.

For each workflow family, we need to say:

- what the current raw flow is
- what hurts about it
- what the target canonical flow is
- which raw APIs stay visible
- which raw APIs get contained behind the guided story

---

## Workflow Families To Condense

The Relational families that need explicit condensation are:

- runtime setup and configuration
- write truth
- read truth and query
- inspect what happened
- history and replay escalation
- merge execution
- compiled artifacts and validation
- retention and durability

---

## Required Fields Per Family

For each family, record:

- current raw flow
- current pain
- target canonical public flow
- target abstraction shape
- raw APIs retained
- raw APIs contained
- migration notes

---

## 1. Runtime Setup And Configuration

### Current raw flow

- `RelationalRuntimeApi::builder()`
- `profile(...)`
- optional flat builder knob stream
- `schema_registry(...)`
- `build()`

The builder already has a real spine.

The problem is everything around that spine arrives as a long flat list of
policy and tuning verbs.

### Current pain

- setup is technically clear, but still too bag-like
- config is real architecture, but the builder does not teach grouped intent
- an agent can succeed, but it still has to guess which knobs matter first

### Target canonical public flow

- `RelationalRuntimeApi::builder()`
- `profile(...)`
- `schema_registry(...)`
- bounded refinement sections only when needed
- `build()`

The main memory should be:

- pick profile
- provide schema
- refine a few important runtime sections
- build

### Target abstraction shape

- guided builder with section-shaped refinement

### Raw APIs retained

- `profile`
- `schema_registry`
- `execution_model`
- `durability_mode`
- `diagnostics`
- grouped config knobs that remain architecturally real

### Raw APIs contained

- flat policy pile with no job grouping
- capacity, storage-layout, lane-policy, and budget tuning as first-memory
  setup knowledge

### Migration notes

- do not remove real knobs just because there are many of them
- regroup and document them as setup sections
- the builder should feel profile-first, not policy-first

---

## 2. Write Truth

### Current raw flow

- get runtime
- begin transaction
- push one or more batches
- optional savepoints
- optional bulk-mutation planning
- optional `admit_*` phase verbs
- `commit()`

### Current pain

- the real write path is good, but the surface still leaks its internal phase
  decomposition
- `plan_bulk_mutation_batch(...)` is helpful, but the `admit_*` trio teaches
  too much internal staging too early

### Target canonical public flow

- begin transaction
- push batch
- commit

For bigger writes:

- begin transaction
- plan bulk mutation batch
- commit

### Target abstraction shape

- transaction session with layered raw + guided split

### Raw APIs retained

- `push_batch`
- `create_savepoint`
- `rollback_to_savepoint`
- `plan_bulk_mutation_batch`
- `commit`

### Raw APIs contained

- `admit_naming_stable_bulk_mutation_batch`
- `admit_lineage_safe_bulk_mutation_batch`
- `admit_provenance_complete_bulk_mutation_batch`
- `merged_plan`
- `inspect_staging`

### Migration notes

- do not flatten the internal phases out of existence
- keep them available for power users and bridge work
- make the transaction story feel like one write-truth workflow first

---

## 3. Read Truth And Query

### Current raw flow

- current-truth reads live behind `visibility_reads()`
- query has its own public surface
- inspection, history, and storage also expose read-shaped doors

### Current pain

- there is no one obvious "read truth" memory yet
- current truth, query, and inspection are all real, but the escalation order is
  not obvious enough

### Target canonical public flow

- `read_truth`
- query when traversal or selection gets bigger
- inspection when the question becomes operational
- history when the question becomes historical

Current implementation reality:

- the primary read lane is still `visibility_reads()`

### Target abstraction shape

- guided runtime-owned read lane plus adjacent query lane

### Raw APIs retained

- `visibility_reads`
- query surface
- `history_access`
- `inspection_access`

### Raw APIs contained

- storage helpers as read-truth discovery
- history and inspection as rival first-contact doors for ordinary reads

### Migration notes

- Phase 3 should teach `visibility_reads()` as the implementation seam for the
  product direction `read_truth`
- query should stay prominent, but as the "bigger read" door, not a rival
  everyday lane

---

## 4. Inspect What Happened

### Current raw flow

- `inspection_access()`
- `publication_access()`
- publication diagnostics helpers
- retention inspection methods
- structural identity methods

### Current pain

- the verbs are pretty good
- the problem is that operator questions are spread across inspection and
  publication nouns instead of being taught as one nearby job family

### Target canonical public flow

- `inspect_what_happened`
- inspect recent commits
- inspect branch head
- inspect publication outputs
- inspect retention state

Current implementation reality:

- the operator lane is split across `inspection_access()` and
  `publication_access()`

### Target abstraction shape

- two nearby contained lanes organized around operator jobs

### Raw APIs retained

- `inspect_commit`
- `inspect_recent_commits`
- `inspect_branch_head`
- `connectivity_summary`
- `graph_summary`
- `retention_summary`
- `latest_bundle`
- `latest_patch`
- `latest_replay`
- `read_patch_stream`
- `read_subscriber_stream`

### Raw APIs contained

- artifact-first diagnostics discovery
- raw noun clouds as the primary operator story

### Migration notes

- do not invent a fake unified runtime diagnostics door if the code does not
  have one yet
- instead, teach inspection and publication as two adjacent doors inside the
  same operator workflow

---

## 5. History And Replay Escalation

### Current raw flow

- `history_access()`
- branch and version graph reads
- merge-adjacent history helpers
- `replay_access()`
- `replay_authority()`

### Current pain

- history is a legitimate next escalation after current truth
- replay is also real, but should not feel equally first-contact
- merge-adjacent history helpers make the history lane look noisier than its
  main jobs really are

### Target canonical public flow

- read truth now
- go to history when you need past truth
- go to replay when you need reconstruction or verification

### Target abstraction shape

- narrow escalation ladder

### Raw APIs retained

- `latest_commit`
- `branch_head`
- `branches`
- `entity_aspect_history`
- `relation_aspect_history`
- `canonical_commit_envelope`
- `compare_outcome`
- `replay_commit`

### Raw APIs contained

- trace-heavy history verbs
- merge-adjacent history planning verbs
- `replay_range` as default replay memory

### Migration notes

- history should be the first specialist escalation after read truth
- replay should stay clearly one step deeper

---

## 6. Merge Execution

### Current raw flow

- `merge_access()`
- inspect planning scope
- prepare merge execution
- execute prepared merge
- verify prepared merge execution

### Current pain

- the top-level merge story is actually decent
- the main risk is letting raw merge vocabulary dominate before a user even
  knows whether they need merge

### Target canonical public flow

- inspect merge scope
- prepare merge execution
- execute prepared merge

### Target abstraction shape

- guided specialist prepared operation

### Raw APIs retained

- `inspect_history_scope`
- `inspect_planning_scope`
- `prepare_merge_execution`
- `execute_prepared_merge`
- `verify_prepared_merge_execution`

### Raw APIs contained

- broad merge noun cloud as first-memory knowledge

### Migration notes

- keep merge powerful and specialist
- the bar for promoting more merge vocabulary into the everyday runtime story
  should stay brutal

---

## 7. Compiled Artifacts And Validation

### Current raw flow

- `simulation_access()`
- `simulation_authority()`
- `invariant_access()`
- `certify_current_state()`

### Current pain

- these lanes are architecturally real now
- but they still read like helper seams unless the docs teach them as owned
  product lanes

### Target canonical public flow

- use `validation` when you need to inspect or certify truth contracts
- use `compiled_artifacts` when you need compiled-lane reads or execution
  control

### Target abstraction shape

- contained adjacent specialist lanes

### Raw APIs retained

- `invariant_access`
- `certify_current_state`
- `simulation_access`
- `simulation_authority`

### Raw APIs contained

- internal implementation naming as the main mental model

### Migration notes

- the important Phase 3 win is not removing these seams
- it is teaching them as intentional lanes with honest jobs

---

## 8. Retention And Durability

### Current raw flow

- retention reads live under inspection
- retention control lives behind `retention_authority()`
- durability reads and writes live under durability access and authority
- snapshot pinning is nearby but separate

### Current pain

- these are all real operational jobs
- but retention in particular can get lost between inspection, history, and
  durability if we do not give it a crisp product place

### Target canonical public flow

- inspect what is retained
- go to `retention` when you need retention control
- go to `durability` when you need checkpoint, recovery, or storage-state work

### Target abstraction shape

- contained operational lanes with a clear split between readback and control

### Raw APIs retained

- inspection retention verbs
- `retention_authority`
- `recovery_plan`
- `checkpoint`
- `recover`
- `compact_store`
- snapshot authority verbs

### Raw APIs contained

- mixing retention control into generic inspection language
- making durability look like the default answer to every operational question

### Migration notes

- retention should stay its own lane
- durability should stay clearly specialist and operational

---

## Phase 3 Outcome Test

Phase 3 is done only if the crate can be taught in this order without walking
module by module:

1. build runtime
2. write truth
3. read truth
4. inspect what happened
5. escalate to history
6. escalate to replay
7. escalate to merge, validation, compiled artifacts, retention, or durability

If the explanation still sounds like a tour of subsystems, condensation is not
done.
