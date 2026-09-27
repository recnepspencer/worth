# WORTH Query Runtime Generic Graph Authoring Closeout

## Status

The runtime generic graph authoring hardening gate is closed as of 2026-05-01
for the runtime-backed admitted surface in `worth-query`.

This closeout covers:

- identity-preserving existing-target relation update
- bridge-verified existing-target update, retirement, retarget, and
  supersession lanes
- first-class same-batch graph composition
- typed graph-composition denial and denied-path admission traces
- domain-invariant denial that stays distinct from runtime support denial
- geometry-pressure certification for:
  - `LoopSuccessorRewire`
  - `FailedNonManifoldAdmission`
  - `FaceInnerLoopInsertion`
  - `EdgeSplit`

This closeout does not claim:

- temporal, async/resource, or mixed-cause mutation semantics
- store-backed restart, durable replay, or persisted graph-authoring artifacts
- topology-specific domain semantics living inside Query
- any mixed-shape capability family that the support rows do not explicitly
  admit

## Governing Source Summary

- `MENTALITY.md`: protects hostile closure over "good enough" feature shape.
  This closeout only claims what is machine-checked through receipts,
  inspection, support rows, denial artifacts, and named hostile suites.
- `arch_laws.md`: protects typed phase boundaries, sealed proof artifacts,
  self-describing envelopes, and fail-closed denial. The graph authoring
  surface now closes only where receipts, denial traces, invariant summaries,
  assumption summaries, and lineage summaries are all explicit runtime
  artifacts.
- `domain_laws.md`: protects domain-agnostic substrate ownership and honest
  decomposition. Query now owns one generic graph-authoring runtime contract
  without absorbing topology-specific semantics from downstream domains.
- `perf_laws.md`: protects one lowering story, bounded counters, and honest
  execution posture. The admitted graph path now exposes explicit composition,
  resolution, lifecycle, assumption, and lineage counters rather than
  re-deriving graph meaning from generic batch residue.
- [runtime-authoritative-mutation-evidence-plan.md](./runtime-authoritative-mutation-evidence-plan.md):
  protects the broader target-evidence and authority-lane contract that this
  graph-authoring hardening extends.
- [runtime-generic-graph-authoring-plan.md](./runtime-generic-graph-authoring-plan.md):
  is the governing hardening spec that required mixed-shape composition,
  backend-verified existing-truth work inside composition, admission traces,
  domain-invariant denial separation, lifecycle taxonomy, assumption/read-set
  evidence, and geometry-pressure hostile suites.
- [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md):
  protects the named acceptance contract for the
  `Runtime Mixed-Shape Graph Authoring And Identity-Preserving Mutation Test`.

## Adversarial Constraint Closed

This gate had to survive the condition where one downstream graph domain wants
to use Query as the ordinary mutation runtime for:

- symbolic same-batch entity and relation declaration
- mixed symbolic and existing-target authoring in one canonical program
- identity-preserving relation rewires
- lineage-preserving supersession
- bridge-verified precondition checks on existing truth
- denied-path diagnostics that distinguish substrate failure from domain
  invalidity

The closed runtime contract now enforces that by giving one public proof chain:

1. one composition-local authoring closure through
   `workspace.compose_graph(...)` or
   `workspace.compose_graph_with_invariant_pack(...)`
2. one canonical lowered graph-composition program
3. one symbolic-to-resolved resolution map
4. one lifecycle-outcome surface
5. one assumption/read-set summary when verified lanes participate
6. one lineage summary when continuity-carrying lanes participate
7. one typed denial plus admission trace when execution never begins
8. one distinct domain-invariant denial plus attempted-shape summary when the
   substrate admits the program but the domain rejects the topology

No public path is allowed to reconstruct that contract from generic
`workspace.batch(...)` folklore, local graph glue, or host-specific verification
helpers.

## Shipped Scope

The closed surface includes:

- graph-authoring entry points:
  - `workspace.compose_graph(...)`
  - `workspace.compose_graph_with_invariant_pack(...)`
- symbolic same-batch authoring inside the composition closure:
  - entity declaration
  - relation declaration
  - symbolic relation declaration
  - symbolic entity follow-up mutation
  - symbolic relation follow-up mutation
  - symbolic relation retirement
- mixed existing-target authoring inside the same canonical program:
  - `update_existing(...)`
  - `retarget_existing(...)`
  - `supersede_existing(...)`
  - `delete_existing(...)`
  - `update_existing_verified(...)`
  - `retarget_existing_verified(...)`
  - `supersede_existing_verified(...)`
  - `delete_existing_verified(...)`
- composition proof surfaces on receipts and inspection:
  - `graph_composition_program()`
  - `graph_composition_resolution_map()`
  - `graph_composition_breadth()`
  - `graph_composition_lifecycle_outcomes()`
  - `graph_composition_evidence()`
  - `graph_composition_assumption_summary()` when verified lanes participate
  - `graph_composition_lineage_summary()` when continuity-carrying lanes
    participate
- denied-path graph artifacts:
  - typed `GraphCompositionDenied(...)`
  - `admission_trace()`
  - `failure_stage()`
- domain-invalid graph artifacts:
  - typed `GraphCompositionDomainInvariantDenied(...)`
  - `domain_invariant_summary()` including:
    - declared collections
    - declared symbols
    - attempted target-combination families
    - attempted lifecycle families
    - attempted program digest
    - attempted breadth digest
    - attempted counter snapshot
- machine-readable public support:
  - graph-composition capability rows
  - graph-composition extension-hook rows

## Stable Dependency Contract

Downstream domains may now rely on Query for:

- same-batch graph-shaped authoring without reconstructing symbolic resolution
  from final rows
- existing-target identity-preserving relation update when the lower runtime
  can preserve target identity honestly
- existing-target retarget when the requested continuity shape truly preserves
  one continuing identity
- existing-target supersession when the requested continuity shape truly
  carries lineage forward
- bridge-verified existing-target mutation lanes when the active runtime admits
  backend verification for that operation family and binding family
- composition-level assumption/read-set summaries instead of reconstructing
  verified preconditions from component rows one by one
- composition-level lineage summaries instead of reconstructing split or merge
  lineage from scattered continuity rows
- domain-invalidity evidence that answers "what supported graph shape was being
  attempted when the invariant pack rejected it?"

In practical terms, downstream graph kernels may delete:

- local same-batch symbolic-resolution folklore
- local relation-rewrite classification glue
- local verified-existing precondition reconstruction for admitted graph lanes
- local edge-split lineage reconstruction glue
- local graph-program rejection reconstruction once `admission_trace()` and
  `domain_invariant_summary()` cover the denied-path explanation contract

## Must Not Assume Yet

Downstream code must not assume:

- graph composition is merely a nicer spelling of generic `workspace.batch(...)`
- unsupported mixed-shape capability families are admitted just because a
  nearby lifecycle lane exists
- any plain existing-target update automatically counts as retarget or
  supersession without explicit continuity semantics
- split-successor continuity is a retarget lane
- bridge-backed verification support on scaffold runtimes implies
  production support on a runtime posture that still denies it
- domain-invariant rejection may be collapsed into `GraphCompositionDenied(...)`
- admitted runtime-backed graph authoring closes temporal, async/resource,
  store-backed, durable, or restart-stable mutation semantics
- Query owns topology validity, non-manifold semantics, or other domain-local
  invariant logic

## Acceptance Mapping

This gate is closed against:

- [runtime-generic-graph-authoring-plan.md](./runtime-generic-graph-authoring-plan.md)
- [runtime-authoritative-mutation-evidence-plan.md](./runtime-authoritative-mutation-evidence-plan.md)
- [worth_query_roadmap.md](./worth_query_roadmap.md)
- [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
- [runtime-authoritative-mutation-evidence-closeout.md](./runtime-authoritative-mutation-evidence-closeout.md)

because the admitted mixed-shape authoring, denial, support, and inspection
surfaces now exist directly and are certified by machine-checkable artifacts.

### `Runtime Mixed-Shape Graph Authoring And Identity-Preserving Mutation Test`

Covered by:

- [crates/worth-query/src/runtime/tests/mutation/graph_composition.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_lifecycle.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_lifecycle.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_followup.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_followup.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_mixed_existing.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_mixed_existing.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_verified_existing.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_verified_existing.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_retarget_existing.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_retarget_existing.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_supersede_existing.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_supersede_existing.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_denial.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_denial.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_existing_binding_denial.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_existing_binding_denial.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_retarget_existing_denial.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_retarget_existing_denial.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_boundary.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_boundary.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_lineage_summary_boundary.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_lineage_summary_boundary.rs)

What is proven:

- graph composition is one first-class public authoring family, not caller-owned
  scalar batch folklore
- canonical program ordering, resolution maps, breadth counters, lifecycle
  counters, lifecycle outcome taxonomy, and receipt/inspection parity are all
  retained as explicit runtime artifacts
- existing-target update, retarget, supersession, and retirement remain
  semantically distinct lanes
- verified existing-target lanes retain assumption/read-set evidence distinctly
  from target binding and mutation result evidence
- unsupported neighbors deny typed and early instead of degrading into
  best-effort graph mutation
- ordinary batches and reconstructed receipts fail closed on composition-only
  proof surfaces

### Geometry-pressure hostile programs

Covered by:

- [crates/worth-query/src/runtime/tests/mutation/graph_composition_geometry_pressure.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_geometry_pressure.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_face_inner_loop.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_face_inner_loop.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_invariant_pack.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_invariant_pack.rs)
- [crates/worth-query/src/runtime/tests/mutation/graph_composition_edge_split.rs](../../crates/worth-query/src/runtime/tests/mutation/graph_composition_edge_split.rs)

What is proven:

- `LoopSuccessorRewire` preserves relation identity under a verified retarget
  lane, binds the correct same-batch symbolic successor, and exposes aggregate
  assumption/read-set proof
- `FailedNonManifoldAdmission` keeps substrate support distinct from
  domain-invalid topology and preserves the attempted-shape summary on the
  denial path
- `FaceInnerLoopInsertion` exposes the full symbolic-to-resolved identity map
  for loop and half-edge construction
- `EdgeSplit` preserves split-successor lineage as a first-class continuity
  story rather than flattening it into retire-plus-create folklore

### Public support and closeout contract

Covered by:

- [crates/worth-query/src/runtime/tests/assembly/support_profile/graph_composition_capabilities.rs](../../crates/worth-query/src/runtime/tests/assembly/support_profile/graph_composition_capabilities.rs)
- [crates/worth-query/src/runtime/tests/assembly/support_profile/authority_evidence_closeout.rs](../../crates/worth-query/src/runtime/tests/assembly/support_profile/authority_evidence_closeout.rs)
- [crates/worth-query/src/runtime/authoritative_mutation_evidence_support.rs](../../crates/worth-query/src/runtime/authoritative_mutation_evidence_support.rs)
- [crates/worth-query/src/runtime/authoritative_mutation_evidence_closeout.rs](../../crates/worth-query/src/runtime/authoritative_mutation_evidence_closeout.rs)

What is proven:

- admitted graph-composition capability families are machine-readable and
  public
- extension-hook boundaries are explicit and non-bypassable
- fail-closed denial classes include graph-composition denial and
  domain-invariant denial
- the runtime authoritative mutation evidence closeout now literally includes
  graph-composition support, denied-path guidance, domain-invalid attempted
  shape summary, verified assumption summary, and lineage summary claims

## Documentation Closed

The public teaching surface is now updated in:

- [crates/worth-query/docs/graph-composition-authoring.md](../../crates/worth-query/docs/graph-composition-authoring.md)
- [crates/worth-query/docs/existing-truth-verified-updates.md](../../crates/worth-query/docs/existing-truth-verified-updates.md)
- [crates/worth-query/docs/existing-truth-verified-deletes.md](../../crates/worth-query/docs/existing-truth-verified-deletes.md)
- [crates/worth-query/docs/writes-and-intents.md](../../crates/worth-query/docs/writes-and-intents.md)
- [runtime-authoritative-mutation-evidence-closeout.md](./runtime-authoritative-mutation-evidence-closeout.md)

Those docs now teach the ordinary admitted graph-authoring story directly
instead of implying a narrower substrate or sending callers back to generic
batch glue first.

## Verification

This closeout was verified with:

- `cargo fmt -p worth-query`
- `cargo check -p worth-query --tests`
- `cargo test -p worth-query runtime_public_authoritative_mutation_evidence -- --nocapture`
- `cargo test -p worth-query graph_composition -- --nocapture`
- `cargo test --manifest-path crates/worth-query/Cargo.toml --test phase_boundaries_compile_fail`
- `cargo test -p worth-query`
- `git diff --check`

## Outcome

`worth-query` now has one ordinary admitted mixed-shape graph-authoring runtime
contract that serious downstream graph domains can cite directly.

The important architectural consequence is not just that Query can execute a
few graph-shaped workflows. It is that downstream domains no longer need to
rebuild:

- relation rewrite semantics
- same-batch symbolic resolution semantics
- verified existing-truth precondition glue
- edge-split lineage explanation
- domain-invalidity explanation for supported-but-rejected graph programs

That is the real closure condition this gate existed to enforce, and it is now
met.
