# Milestone 3.11 Phase 5 Implementation Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Objective

Close Milestone 3.11 on exact source by making the visual-snapshot resource
contract enforceable, projecting one immutable snapshot evidence object,
documenting the public and human workflows, and closing VS-01 through VS-09
without creating another product path, test target, or truth source.

Phase 4 is the entry boundary. Its closed contract and 20-row ledger prove the
canonical overlay and permanent product pulse. Phase 5 may add evidence,
admission, cost, documentation, and audits; it may not redesign snapshot
identity, mounted authority, host capture, spatial truth, overlay lifecycle, or
the executable world.

## Boundary Review

### Existing Authority

- `worth-ui-inspection/query/visual_snapshot/` owns caller-declared policy and
  non-authoritative disclosure meaning.
- `worth-ui-runtime/inspection/visual_snapshot/` owns grants, live receipts,
  registries, immutable indexes, and managed resource lifetime.
- `worth-ui-host-contract/visual_snapshot/` owns mechanical presentation-bound
  capture evidence.
- `worth-ui-runtime/facade/entry/visual_snapshot.rs` is the only public capture
  admission path.
- `worth-ui-runtime/facade/entry/visual_overlay.rs` is the only public overlay
  publication path.
- the existing compile-contract sessions, consolidated application-contract
  target, topology-contract target, and sole executable-world target own all
  milestone proof.

### Gaps Found During Phase 5 Planning

1. `UiVisualSnapshotEvidence`, required by the milestone, has no production
   type or receipt projection.
2. disclosure audience is retained, but redaction is not an explicit policy or
   artifact field; `Redacted` is an unreachable retention variant.
3. `maximum_retained_structural_bytes` is copied into a grant but is not
   consumed by capture admission or retained-resource accounting.
4. the declared 65,536 visible and hit-test record ceilings are not represented
   in policy or enforced before host effects.
5. overlay lease count is tested, but published and cleared overlays expose no
   structural cost receipt.
6. the developer visual-inspection document does not exist, the lifecycle
   document stops before the 3.11 pulse, and `ai-diagnostics.md` currently
   phrases identity-aware comparison as present even though 3.12 owns it.

These are implementation gaps. The ledger must not close them with prose-only
evidence.

## Compile-Time and Runtime Enforcement

The following stays compiler-enforced:

- geometry, optional-pixel, and required-pixel artifact postures remain sealed
  marker types;
- grants remain three distinct concrete, non-serializable types;
- request coordinates remain generatively snapshot-scoped;
- pending/completed capture and pending/published/cleared overlay states remain
  linear;
- `UiVisualSnapshotEvidence` is immutable and carries only derived correlation
  values, never authority;
- pixel artifacts expose redaction and retention as separate typed postures.

The following remains runtime-adjudicated because it depends on application or
host state:

- request disclosure matching the session-sealed grant;
- visible/hit region count admission;
- per-receipt and per-session retained structural bytes;
- pixel redaction transformation;
- expiry, capacity, host capability, deadline, and presentation affinity.

No generic authority marker, serializable grant, raw identity constructor, or
boolean redaction flag is permitted.

## Batch 5A: Evidence and Disclosure Closure

### Production Work

- add a typed `UiVisualInspectionDisclosure` carrying audience and explicit
  pixel redaction policy;
- require every visual snapshot request to declare that disclosure and compare
  it with the concrete grant before effects;
- separate pixel redaction posture from retention disposition;
- transform redacted native bytes deterministically and label them as derived,
  never as the original native capture;
- add immutable `UiVisualSnapshotEvidence` containing schema version, exact
  affinity, coordinate observation, visible/hit index identities, artifact
  posture, disclosure posture, query budget, and structural cost;
- make every live snapshot receipt expose exactly one sealed evidence
  projection.

### Proof

- public compile twins retain sealed artifact/grant/coordinate behavior;
- application contracts prove matching unredacted synthetic disclosure,
  matching redacted disclosure, mismatch denial before host request, and
  redaction provenance;
- disposal and shutdown invalidate bytes without changing immutable
  correlation evidence;
- topology tests reject a constructor or runtime-authority field in the
  immutable evidence type.

### Exit

Batch 5A closes only when disclosure denial and redaction are reachable through
production entry points and `UiVisualSnapshotEvidence` is public through the
curated facade.

## Batch 5B: Resource and Cost Closure

### Production Work

- add explicit visible-region and hit-test-region capacities to the immutable
  inspection policy and grant scope;
- acquire the mounted capture basis before host request construction, calculate
  a checked structural reservation from the retained frame/trace basis and
  exact spatial-index representation, and reject excess before host effects;
- track reserved and retained structural bytes beside pixel bytes in the
  session registry;
- require actual sealed structure to fit the admitted reservation;
- release structural and pixel accounting together on disposal, drop, and
  shutdown;
- expose overlay publish and clear cost receipts with exact emitted-region and
  lease counters;
- keep ordinary-frame visual counters exactly zero.

### Proof

- deterministic spatial tests cover 1, 1,024, and 65,536 records;
- maximum-overlap and candidate exhaustion remain typed incomplete at the
  4,096 default query budget;
- application contracts prove visible-count, hit-count, per-receipt structure,
  per-session structure, pixel, snapshot, and overlay capacity denials;
- unchanged frames prove all eleven visual counters remain zero across repeated
  ordinary execution;
- overlay publication reports four emitted border strips and one retained lease;
  clear reports zero retained overlay leases;
- shutdown reports both disposed pixel and structural bytes.

### Exit

Batch 5B closes only when every declared default bound has an enforcing owner,
a typed denial, structural counter evidence, and cleanup evidence.

## Batch 5C: Public DX and Human Workflow

### Documentation Work

- create `workspaces/worth-ui/docs/visual-inspection.md`;
- revise `workspaces/worth-ui/docs/application-lifecycle.md` with the exact
  target, trace, overlay, clear, edit, recovery, and certification workflow;
- revise only the visual-snapshot section of `ai-diagnostics.md` so 3.11's
  implemented receipt names are current and identity-aware comparison remains
  explicitly owned by 3.12;
- bind the visual-inspection Rust example to the existing
  `governed_visual_snapshot_lifecycle.rs` compile-pass source and the existing
  real application-contract witness;
- add no compile session, integration target, binary, runner, or harness.

### Proof

- a topology audit compares the documented Rust fence with its exact compiled
  source;
- the existing compile matrix compiles the example;
- the existing application-contract target runs its production-path witness;
- the lifecycle document contains the exact human and executable-world
  commands and explains which frame each trace describes.

### Exit

Batch 5C closes only when a fresh reader can run the pulse, identify the inset
target, observe the overlay and clear, perform valid and malformed edits, and
run the same certification command without reading test internals.

## Batch 5D: Ledger, QA, and Successor Handoff

### Closure Work

- close every VS-01 through VS-09 row with exact fixture provenance, typed
  result, mutation control, structural cost, teardown, and command;
- add a Phase 5 contract audit that requires exactly nine unique proved rows
  and all required evidence fields;
- set the milestone spec and Phase 5 contract to closed only after all rows
  pass;
- reconcile the roadmap with the exact closed 3.11 contract and 3.12
  inheritance boundary.

### Required Commands

- focused inspection, spatial, overlay, protocol, documentation, and ledger
  tests;
- consolidated application and topology targets;
- canonical two-session compile contracts;
- the sole Windows executable-world target with `--features executable-world`;
- `cargo fmt --check`;
- workspace/all-target/all-feature `clippy -D warnings`;
- full workspace/all-target/all-feature tests;
- official scoped WORTH UI line-cap guard and fail-closed live-source audit;
- `boundary-check`;
- `agent-context check`;
- `git diff --check`.

### Review Order

1. `implementation-batch` closes 5A.
2. `qa-loop` and `qa-tests` challenge 5A before 5B begins.
3. `implementation-batch` closes 5B.
4. `qa-loop`, `qa-tests`, and `code-quality-qa` challenge 5B before 5C.
5. `implementation-batch` closes 5C and 5D.
6. all three QA disciplines rerun across the entire dirty milestone.

Any failed gate creates a new written plan before repair. Later batches remain
locked until the earlier batch ledger rows are proved.
