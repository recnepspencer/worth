# Milestone 7A: Merge-Ready History Certification

## Purpose

Milestone 7A is not a merge-execution milestone.

It is a certification and hardening milestone that proves the runtime can carry
canonical ordered multi-parent history through its real authority surfaces
without collapsing back into linear-history assumptions.

The exact definition for this milestone is:

> A history model is merge-ready when canonical ordered multi-parent commit
> shapes can be represented, persisted, recovered, replayed, queried, and
> certified without collapsing to linear-history assumptions.

That is the entire scope center for this document.

Milestone 7A does **not** certify:

- semantic merge correctness
- conflict resolution correctness beyond the current explicit history/conflict
  surfaces
- causal reconciliation semantics
- branch-intent preservation
- merge-result validity as a richer product behavior
- CRDT-style merge policy execution

Those are Milestone 7B-and-later concerns.

## Status and Architectural Starting Point

The current runtime already contains substantial merge-ready history shape:

- [`CommitReference`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/history/data/mod.rs)
  already carries `parents: Vec<CommitId>`
- commit construction already preserves deterministic parent ordering
- canonical commit envelopes already persist ordered parent lists
- replay already checks parent-list parity in several paths
- durability already persists and recovers merge-bearing histories
- history access already performs DAG ancestry traversal rather than assuming a
  strict single-parent chain
- tests already cover merge-shaped commits in history, replay, and durability

That means Milestone 7A should be approached honestly:

- not as greenfield history design
- not as â€œimplement mergesâ€
- not as a mini merge framework
- but as a whole-system certification pass plus targeted assumption removal

## Governing Constraints

This milestone is governed by the coding-guideline rules and by the runtime's
truth-grade goals.

The most important constraints are:

1. ordered parent lists are authoritative, not advisory
2. parent order is canonical and position-sensitive, not set-like
3. authority, replay, durability, diagnostics, and ancestry must agree on the
   same ordered history shape
4. certification artifacts must prove canonical runtime truth, not invent a
   second source of truth
5. cost must remain visible and bounded; merge-ready support must not quietly
   introduce broad rescans or graph-wide rediscovery in paths that claim
   narrower work

## Phase 1.1 Audit Baseline

Step 1.1 is the baseline classification pass over the current parent-order
surfaces in code.

The goal of this audit is to answer one question before semantic-tightening
work continues:

which surfaces are authoritative history truth, which are contextual merge
metadata, and which are derived or convenience views?

### Authoritative parent-order surfaces

These surfaces are authoritative in the current implementation and must remain
so in Milestone 7A:

- [`CommitReference.parents`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/history/data/mod.rs)
  is the sole authoritative ordered-parent surface
- [`CanonicalCommitEnvelope.commit.parents`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/replay/data/mod.rs)
  is the persisted/published authority carrier for ordered parent truth
- commit construction in
  [`authority/commit/plan_building.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/authority/commit/plan_building.rs)
  is the only phase that canonically assembles parent order
- history storage and branch-head state in
  [`logic/runtime/state/subsystems/history.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/logic/runtime/state/subsystems/history.rs)
  preserve committed ordered parent truth without reinterpretation

### Contextual merge metadata surfaces

These surfaces are real and important, but they are not parent authority:

- `merge_parent_branches`
- `merge_base_commits`
- `requested_merge_parent_count`
- `effective_merge_parent_count`
- `merge_parent_count`

These exist in history summaries, commit summaries, publication diagnostics,
and replay/durability artifacts as contextual or summary information. They may
support diagnostics, ancestry interpretation, or future merge work, but they
must never outrank `CommitReference.parents`.

### Derived and convenience surfaces

These surfaces are derived views, proof helpers, or convenience APIs:

- `branch_head(...)`
- `latest_commit(...)`
- `ancestor_chain(...)`
- `latest_common_ancestor_between_branches(...)`
- `inspect_merge(...)`
- replay `reconstructed_parent_chain`
- replay and durability parity bases built from parent-order truth
- diagnostics fields such as `parent_count` and `authoritative_parent_list`

These are consumers of authoritative history truth. Milestone 7A must keep
them honest, but they do not define truth.

### Current implementation findings

The audit found the following:

1. the runtime already stores ordered parents canonically in the correct
   authority surfaces
2. replay parity already compares ordered parent lists as order-sensitive
   history truth
3. durability already fails explicitly on missing authoritative parent-closure conditions
4. the biggest remaining risk is not â€œparents are unsupportedâ€; it is
   convenience logic, diagnostics wording, and helper naming drifting back
   toward linear-history assumptions
5. ancestor-selection naming was previously looser than the actual algorithm;
   Milestone 7A has already started tightening this by naming the current rule
   explicitly as `max_commit_id_common_ancestor`

### Step 1.1 close condition

Step 1.1 is considered complete when:

- the authoritative/contextual/derived classification is explicitly recorded
- no one needs to guess which parent-related field actually defines truth
- later milestone steps can target the non-authoritative surfaces without
  reopening the authority question

## Semantic Authority Rules

### Parent order authority

The authoritative parent-order surface is:

- [`CommitReference.parents`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/history/data/mod.rs)

Nothing else is allowed to compete with that authority.

The rules are:

- `CommitReference.parents` is the sole authoritative storage/publication
  surface for ordered parents
- parent-list canonicalization occurs exactly once at commit construction
- after publication, no helper may reorder, deduplicate, normalize, or
  reinterpret parent order outside explicit history-resolution logic
- any parity, replay, or durability comparison of parent lists is order
  sensitive
- `merge_parent_branches` remains contextual merge provenance, not parent
  authority

### Consumer-side order wrapper

Milestone 7A should add a narrow semantic wrapper:

- `OrderedParentList`

Its purpose is limited:

- it is a consumer-side semantic guardrail for order-sensitive logic
- it wraps authoritative parent order so helpers cannot casually compare parent
  vectors as sets
- it does not introduce alternate authority
- it must always be derived from `CommitReference.parents`, never authored as a
  separate competing source

### History-shape classification

Milestone 7A should also add:

- `HistoryShapeClassification`

Recommended variants:

- `Root`
- `Linear`
- `MergeReady`

This classification is intentionally coarse.

It is suitable for:

- assumption removal
- certification branching
- diagnostics wording where the runtime needs a literal shape category

It is **not** suitable for future merge semantic policy. Milestone 7B may need
finer distinctions, and this type must not be treated as the final merge
ontology.

## What Milestone 7A Must Prove

Milestone 7A is complete only when the runtime proves all of the following:

### 1. Canonical ordered multi-parent shape is real

The runtime must prove that:

- zero-parent roots remain zero-parent roots
- one-parent commits remain one-parent commits
- ordered multi-parent commits remain ordered multi-parent commits
- parent order is stable through publication, persistence, recovery, replay,
  and query surfaces

### 2. No observable surface quietly assumes linear history

The runtime must prove that these surfaces remain operational under ordered
parent lists:

- history access
- ancestry reasoning
- replay planning
- replay authority comparison
- durability planning
- durability recovery validation
- branch-head reasoning
- diagnostics emission
- certification digest generation

The failure mode this milestone is hunting is:

â€œthe type supports multiple parents, but some helper, replay path, or
diagnostic summary still behaves as though parent count is 0 or 1.â€

### 3. DAG ancestry reasoning remains mechanically honest

Milestone 7A must certify ancestry behavior over merge-bearing histories, but
the document must use precise language.

Do **not** use vague graph shorthand like â€œlatest common ancestorâ€ unless that
is exactly the algorithm being certified.

The current implementation should be described in terms of the actual runtime
behavior it computes:

- common-ancestor selection rule
- merge-base result or merge-base candidate set
- maximal or otherwise canonical ancestor result under the runtime's reachability algorithm

The important milestone rule is:

- the spec must name the real algorithm, not a colloquial approximation

### 4. Branch reasoning semantics are explicit

Milestone 7A must state clearly which operations are order-sensitive and which
are reachability-sensitive.

Default rule:

- parent order is authoritative history truth
- ancestry, reachability, and branch-unique reasoning are graph-based unless a
  specific operation explicitly depends on ordered parent position

That distinction must be preserved so future code does not accidentally infer
branch meaning from parent index when the runtime only intended reachability.

## Minimal Proof Artifact Model

The original broad artifact family is intentionally trimmed here so Milestone
7A does not become a reporting-framework exercise.

Every artifact introduced by this milestone must be classified as one of:

- canonical proof artifact
- derived summary/view
- harness convenience wrapper

Only the minimum set of true proof-bearing artifacts should be added.

### Canonical proof artifacts

Milestone 7A should add only these new proof-bearing surfaces:

#### `ParentListSerializationArtifact`

Purpose:

- proves that ordered parent lists persist byte-for-byte / element-for-element
  through serialization and recovery-reconstruction boundaries

Must capture:

- commit id
- authoritative ordered parent list
- persisted/recovered ordered parent list
- parity result
- parent-count parity

#### `AncestryQueryMatrix`

Purpose:

- proves the runtime's certified ancestry/merge-base reasoning over merge-ready
  histories

Must capture machine-checkable cases for:

- root histories
- linear histories
- ordered multi-parent fixture histories
- branch ancestry queries
- merge-base/common-ancestor queries under the runtime's actual algorithm
- branch-unique commit reasoning

This matrix is a proof artifact because ancestry correctness is part of the
requirement, not merely a display concern.

### Harness convenience wrapper

Milestone 7A may add exactly one aggregation wrapper:

#### `MergeReadyHistoryCertificationBundle`

This bundle exists for certification runs and test/harness orchestration.

It is **not** runtime authority.

Its role is:

- aggregate digests from canonical runtime surfaces and the two proof artifacts
- provide machine-checkable certification output for the named requirement

Required digest fields:

- `parent_list_digest`
- `ancestry_query_digest`
- `replay_acceptance_digest`
- `diagnostics_digest`
- `branch_reasoning_digest`

Important rule:

- certification bundles are proof artifacts, not runtime authority
- runtime logic must never consult certification bundles to determine history
  truth

### Derived summaries/views

Anything beyond the two proof artifacts and the single bundle should be treated
as a derived view unless it proves something the existing set cannot.

Specifically:

- diagnostics summaries are views over canonical diagnostics artifacts
- replay reports are views unless replay parity requires a distinct proof
  artifact not already covered by certification digests
- generic â€œhistory shape artifactâ€ wrappers should be avoided unless they carry
  proof data not already represented above

## Phased Implementation Plan

### Phase 1: Freeze semantic boundaries

Goal:

- make ordered parent authority explicit and unambiguous before adding more
  certification machinery

Must ship:

- explicit milestone documentation of the 7A merge-ready-history definition
- explicit authority rule that `CommitReference.parents` is the sole ordered
  parent authority surface
- `OrderedParentList` as a consumer-side semantic guardrail
- `HistoryShapeClassification` as a coarse shape classifier for assumption
  removal and certification branching
- a structural rule in code-level docs and helper boundaries that parent
  canonicalization happens exactly once at commit construction

Must preserve:

- no alternate parent authority
- no helper-side reordering, deduplication, or reinterpretation of parent order
- no broadened merge semantics beyond history-shape certification

### Phase 2: Remove linear-history assumptions from runtime surfaces

Goal:

- audit and harden every relevant surface so multi-parent history is
  operationally real rather than merely representable in types

Must ship:

- targeted audit of:
  - history access
  - replay planning
  - replay authority
  - durability planning
  - durability recovery validation
  - branch reasoning helpers
  - diagnostics emission
  - certification digest generation
- replacement of any remaining single-parent assumptions with ordered-parent or
  DAG-reachability-aware behavior
- stricter diagnostics field usage that prefers literal history terms over
  merge-role language not yet formally defined

Diagnostics language rules:

- prefer:
  - `authoritative_parent_list`
  - `ordered_parent_index`
  - `parent_count`
  - `ancestry-derived merge-base candidates`
- avoid unless formally defined elsewhere:
  - `primary parent`
  - `target parent`
  - `source parent`
  - `merge intent parent`

Must preserve:

- no semantic inflation into merge-execution roles
- no hidden full-history rescans in paths that already claim narrower work

### Phase 3: Add proof artifacts and certification runner

Goal:

- turn merge-ready history into a certifiable runtime property rather than a
  cluster of scattered assertions

Must ship:

- `ParentListSerializationArtifact`
- `AncestryQueryMatrix`
- `MergeReadyHistoryCertificationBundle`
- a dedicated harness entrypoint such as:
  - `run_merge_ready_history_shape_certification(...) -> MergeReadyHistoryCertificationBundle`

The runner must construct certification cases for:

- root commit histories
- linear histories
- ordered multi-parent commit-envelope fixtures
- replay over ordered parent lists
- durability round-trip over ordered parent lists
- ancestry and branch-reasoning queries over merge-bearing histories
- diagnostics emitted from merge-bearing histories

Must preserve:

- proof artifacts derive from canonical runtime truth
- harness formatting never becomes a second authority surface

### Phase 4: Sharpen replay and durability drift semantics

Goal:

- make parent-order failures precise rather than vaguely â€œhistory driftedâ€

Milestone 7A should explicitly classify these drift modes:

#### `CanonicalHistoryDrift`

Meaning:

- authoritative ordered parents differ from the expected canonical history
  envelope shape

#### `ReplayAuthorityDrift`

Meaning:

- replay parent reconstruction or replay parity disagrees with authoritative
  ordered parent truth

#### `DurabilityParityDrift`

Meaning:

- recovered ordered parent order differs from persisted authoritative order

These may map onto existing mismatch/failure classes, but the milestone must
make the distinction explicit even if the final implementation encodes them
through existing replay/durability error families.

Must ship:

- explicit replay rejection/parity behavior for parent-order tampering
- explicit durability rejection/parity behavior for malformed or drifted parent
  order
- a fixture vs replay vs recovered certification matrix that proves parent order
  remains identical across all three surfaces

Must preserve:

- parent-order reversal is drift, not equivalence
- order-sensitive parity remains part of canonical history truth

### Phase 5: Cost visibility and 7B runway

Goal:

- make merge-ready certification cost-visible and leave behind only the
  narrowest safe runway for Milestone 7B

Must ship these counters:

- `merge_history_parent_comparisons`
- `merge_history_ancestry_traversals`
- `merge_history_replay_parent_checks`
- `merge_history_durability_parent_checks`
- `merge_history_ancestry_nodes_visited`
- `merge_history_replay_planning_nodes_visited`
- `merge_history_durability_validation_nodes_visited`

Optional only if needed by the actual audit:

- `merge_history_broad_scan_fallback_count`

Why the additional node-visited counters matter:

- event/check counters alone can hide radically different DAG breadth costs
- node-visited counters expose whether merge-history support stayed
  proportional or fell back to broad traversal

#### 7B runway rule

Milestone 7A may add one narrow internal seam for 7B, but it must remain
strictly bounded.

If introduced, `MergeCommitHistoryShape` must be:

- internal only
- derived helper state
- non-authoritative
- explicitly allowed to change or disappear in 7B

It must not be treated as proof that 7A already owns richer merge semantics.

If the implementation can complete 7A cleanly without this seam, omitting it is
preferable to speculative architecture.

## Test and Certification Requirements

Milestone 7A closes against the roadmap requirement:

- `Merge-ready history shape test`

That test must prove all of the following as machine-checkable outputs:

### Required scenarios

- zero-parent root commits
- one-parent linear commits
- ordered multi-parent commit-envelope fixtures
- replay processing over ordered parent lists
- durability persistence and recovery over ordered parent lists
- branch comparisons and ancestry reasoning on merge-bearing histories

### Required verification points

- ordered parent lists persist through durability, replay, diagnostics, and
  branch reasoning
- APIs do not assume â€œsingle parent or noneâ€
- parent order is canonical and stable
- ancestry queries remain correct under the runtime's actual DAG reasoning rule

### Required certification outputs

- `parent_list_digest`
- `ancestry_query_digest`
- `replay_acceptance_digest`
- `diagnostics_digest`
- `branch_reasoning_digest`

### Required regression cases

Milestone 7A should also add focused regressions for:

- reversed ordered parent list is rejected as drift
- parent-list tampering causes replay parity failure
- recovered durable order differs from persisted order only through explicit
  parity failure, never silent acceptance
- branch-unique commit reasoning remains correct in merge-bearing histories
- diagnostics preserve literal ordered-parent reporting

## Relationship to Milestone 7B

Milestone 7A is intentionally narrower than merge execution.

What 7A gives 7B:

- authoritative ordered parent history truth
- certified replay/durability/ancestry behavior over DAG-shaped commit history
- no silent linear-history assumptions in core system surfaces
- cost-visible ancestry and parity paths
- a narrow guardrail wrapper (`OrderedParentList`) and coarse history-shape
  classifier (`HistoryShapeClassification`)

What 7A explicitly does **not** settle for 7B:

- semantic merge-role meaning
- causal metadata
- conflict-free merge policies
- merge-execution authority pipeline
- richer merge explanation artifacts
- policy-driven reconciliation behavior

7B must build on the same certified history truths. It must not invent a
parallel history-certification model.

## Completion Standard

Milestone 7A is complete only when:

- the runtime can represent, persist, recover, replay, and query ordered
  multi-parent commit history without linear-history collapse
- the named certification outputs are emitted as machine-checkable proof
  artifacts
- parent order is treated as authoritative canonical history truth on every
  relevant surface
- diagnostics and branch reasoning remain semantically honest
- cost counters expose the actual breadth of merge-history ancestry and parity
  work
- the resulting implementation leaves behind no accidental alternate history
  authority and no premature 7B semantic commitments

This is the bar for calling the runtime merge-ready in the Milestone 7A sense.

## Milestone 7A Implementation TODO List

This checklist is the linear execution spine for the milestone.

Rules for using it:

- items should be completed in order unless a later item is explicitly blocked
  on none of the earlier ones
- no item should be marked complete unless the corresponding code, tests, and
  diagnostics are in place
- if an item reveals a structural flaw in an earlier phase boundary, reopen the
  earlier item instead of patching around it
- the checklist is not done when implementation compiles; it is done when the
  certification and QA gates are green

### Phase 1 TODOs: Freeze semantic boundaries

- [ ] Audit every current parent-order surface in history, commit publication,
  replay, durability, and diagnostics; record which are authoritative,
  contextual, or derived
- [ ] Tighten the milestone text and code comments so `CommitReference.parents`
  is explicitly the sole authoritative ordered-parent surface
- [ ] Define `OrderedParentList` as a consumer-side semantic guardrail and make
  its non-authoritative role explicit in docs and type comments
- [ ] Identify all code paths that currently compare parent lists and classify
  each one as order-sensitive parity, reachability-only reasoning, or
  diagnostic/view logic
- [ ] Introduce `HistoryShapeClassification` with coarse `Root`, `Linear`, and
  `MergeReady` variants for assumption removal and certification branching only
- [ ] Add a structural rule in code-level docs/helpers that parent-list
  canonicalization occurs exactly once at commit construction
- [ ] Audit helper paths for illegal parent-list normalization behaviors:
  sorting, deduplication, role reinterpretation, or synthetic reconstruction
- [ ] Seal or redirect any helper APIs that allow parent-order misuse after
  publication
- [ ] Confirm `merge_parent_branches` and similar fields are documented and used
  as contextual provenance only, never as parent authority

### Phase 2 TODOs: Remove linear-history assumptions from runtime surfaces

- [ ] Audit history access for any â€œsingle parent or noneâ€ assumptions hidden
  behind convenience logic
- [ ] Audit replay planning for any parent-chain logic that assumes linear
  history rather than ordered multi-parent reachability
- [ ] Audit replay authority comparisons so parent-order parity is enforced as
  canonical history truth
- [ ] Audit durability planning and recovery validation for implicit linear
  assumptions, especially around parent-chain reconstruction and completeness
- [ ] Audit branch-head reasoning helpers and branch-unique commit reasoning for
  correctness on merge-bearing DAGs
- [ ] Audit diagnostics emission for vague or semantically unsafe role language
  such as â€œprimary parentâ€ or â€œtarget parentâ€
- [ ] Replace diagnostics wording with literal history language:
  `authoritative_parent_list`, `ordered_parent_index`, `parent_count`, and
  `ancestry-derived merge-base candidates` where appropriate
- [ ] Replace any open-coded `parents.len() > 1` branches that are really
  shape-classification logic with a single classification/helper surface
- [ ] Identify and tighten the runtime's current common-ancestor / merge-base
  terminology so the spec and code name the actual algorithm honestly
- [ ] Add or update tests covering branch reasoning semantics where parent order
  is authoritative but reachability remains graph-based

### Phase 3 TODOs: Add proof artifacts and certification runner

- [ ] Define `ParentListSerializationArtifact` as a canonical proof artifact
- [ ] Define `AncestryQueryMatrix` as a canonical proof artifact
- [ ] Define `MergeReadyHistoryCertificationBundle` as a harness aggregation
  wrapper and document clearly that it is not runtime authority
- [ ] Decide the minimal file/module ownership for these artifacts so they live
  with certification/harness responsibility rather than leaking into unrelated
  runtime domains
- [ ] Implement a dedicated certification runner such as
  `run_merge_ready_history_shape_certification(...)`
- [ ] Make the runner build root-history certification cases
- [ ] Make the runner build linear-history certification cases
- [ ] Make the runner build ordered multi-parent fixture certification cases
- [ ] Make the runner capture replay acceptance/parity evidence for those cases
- [ ] Make the runner capture durability round-trip evidence for those cases
- [ ] Make the runner capture ancestry/branch-reasoning outputs for those cases
- [ ] Make the runner emit machine-checkable digests for:
  `parent_list_digest`, `ancestry_query_digest`, `replay_acceptance_digest`,
  `diagnostics_digest`, and `branch_reasoning_digest`
- [ ] Verify every field in the certification bundle is derived from canonical
  runtime truth rather than test-local ad hoc formatting
- [ ] Keep any diagnostics/replay summaries as derived views unless they prove
  something not already carried by the canonical artifacts and bundle digests

### Phase 4 TODOs: Sharpen replay and durability drift semantics

- [ ] Define the exact drift taxonomy for ordered-parent failures across
  history, replay, and durability
- [ ] Add explicit handling/documentation for `CanonicalHistoryDrift`
- [ ] Add explicit handling/documentation for `ReplayAuthorityDrift`
- [ ] Add explicit handling/documentation for `DurabilityParityDrift`
- [ ] Decide whether these map onto existing mismatch/error classes or require
  narrower typed wrappers while preserving current public semantics
- [ ] Add replay tests where reversed parent order is treated as drift, not
  equivalence
- [ ] Add replay tests where parent reconstruction disagreement fails with
  explicit evidence
- [ ] Add durability tests where persisted vs recovered parent order mismatch is
  rejected or flagged as parity failure explicitly
- [ ] Add a fixture-vs-replay-vs-recovered parity matrix proving ordered parent
  lists remain identical across all three surfaces
- [ ] Verify diagnostics emitted for drift stay literal and do not invent
  merge-role semantics not yet owned by 7A

### Phase 5 TODOs: Cost visibility and bounded 7B runway

- [ ] Add `merge_history_parent_comparisons`
- [ ] Add `merge_history_ancestry_traversals`
- [ ] Add `merge_history_replay_parent_checks`
- [ ] Add `merge_history_durability_parent_checks`
- [ ] Add `merge_history_ancestry_nodes_visited`
- [ ] Add `merge_history_replay_planning_nodes_visited`
- [ ] Add `merge_history_durability_validation_nodes_visited`
- [ ] Add `merge_history_broad_scan_fallback_count` only if the implementation
  audit reveals a real fallback path worth exposing
- [ ] Register the new counters in the correct performance/introspection
  surfaces and keep their names aligned with runtime vocabulary
- [ ] Add exact or structurally precise proof tests for the new counters where
  the workload shape is controlled enough to certify cost behavior honestly
- [ ] Audit whether any proposed 7B seam is actually necessary after the 7A
  hardening work is complete
- [ ] If `MergeCommitHistoryShape` is introduced, keep it internal-only,
  derived, non-authoritative, and explicitly disposable/changeable in 7B
- [ ] If `MergeCommitHistoryShape` is not necessary, omit it rather than
  landing speculative architecture

### Certification TODOs

- [ ] Add a named certification test for `Merge-ready history shape test`
- [ ] Ensure the certification run covers root, linear, and ordered
  multi-parent fixture histories
- [ ] Ensure the certification run covers replay processing over ordered parent
  lists
- [ ] Ensure the certification run covers durability persistence/recovery over
  ordered parent lists
- [ ] Ensure the certification run covers branch comparison and ancestry
  reasoning on merge-bearing histories
- [ ] Ensure the certification run proves APIs do not collapse to â€œsingle parent
  or noneâ€
- [ ] Ensure the certification run emits all required machine-checkable digests
- [ ] Ensure the certification run is deterministic across repeated execution

### Regression TODOs

- [ ] Re-run and keep green the existing merge-bearing history tests
- [ ] Re-run and keep green the existing merge-bearing replay tests
- [ ] Re-run and keep green the existing merge-bearing durability tests
- [ ] Re-run and keep green hostile commit/replay equivalence coverage that
  already touches merge-bearing histories
- [ ] Add focused regressions for reversed parent-order tampering
- [ ] Add focused regressions for parent-list serialization parity failure
- [ ] Add focused regressions for branch-unique reasoning under merge-bearing
  history
- [ ] Add focused regressions for diagnostics language and ordered-parent field
  reporting

### Closeout TODOs

- [ ] Re-read `milestone-7a.md` and confirm the implementation still matches the
  milestone's exact definition of merge-ready history
- [ ] Verify no new helper, artifact, or certification wrapper became accidental
  runtime authority
- [ ] Verify no diagnostics or type names overstate 7A as semantic merge
  correctness
- [ ] Verify no speculative 7B runway type became architecture cement without a
  current milestone need
- [ ] Produce closeout evidence that ordered parent truth survives publication,
  persistence, recovery, replay, ancestry reasoning, and certification output
- [ ] QA the full milestone to aerospace-grade and production-grade strictness:
  semantic honesty, deterministic observability, replay/durability parity,
  explicit failure taxonomy, compile-time boundary discipline, performance-cost
  visibility, hostile-edge-case coverage, and final cleanliness until the
  implementation reads as structurally correct enough to earn a green light
