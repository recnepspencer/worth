# Milestone 9.5 Engineering Spec: Query Productization Debt Cleanup For Reuse, View Shapes, And Typed Consumption

> **Status:** Draft
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Primary predecessors:** [milestone-9.4.md](./milestone-9.4.md), [milestone-9.3.4.md](./milestone-9.3.4.md), [milestone-9.3.8.md](./milestone-9.3.8.md)
>
> **Purpose:** close the remaining runtime-backed Query productization debt in reusable scope/template composition, core view-shape families, grouped composition, retained-artifact projection consumption, preserved temporal/async reuse, and raw runtime bootstrap before store-backed and durable milestones freeze those lanes as public truth.

## Goal

Turn the remaining admitted-but-debt Query productization lanes into honest
runtime-backed production surfaces so ordinary Query consumers can rely on
scope/template reuse, core view families, grouped composition, retained
artifact fact consumption, preserved temporal/async reuse, and a simple valid
read-runtime bootstrap without falling back to special-case seams,
custom bridge-backed scaffolding, or
support-profile debt markers.

## Why This Milestone Exists

The old roadmap split temporal and async work across `9.4` through `9.7`, but
that semantic closure now lives inside one merged [milestone-9.4.md](./milestone-9.4.md).
What remains is a different kind of problem: Query's ordinary productization
surface still exposes several lanes that are admitted, documented, and useful,
but still marked as `debt` or left structurally unfinished.

The concrete evidence is already visible:

- the same support/profile tests still mark the core view families `table`,
  `detail`, `inspector_detail_observed`, `inspector_detail_focused`, and
  `kanban_grouped` as `debt`
- grouped composition docs still call out grouped template/composition profile
  debt
- projection consumption still says retained derived-artifact and live-artifact
  bindings are not yet first-class source families
- scopes/templates/saved-query/view-shape docs still show that runtime-backed
  temporal/async reuse does not yet carry through the full preserved
  inspector/grouped reuse surface
- hostile Phase 28 read/runtime work still needed a custom minimal
  bridge-backed runtime/testing seam because the raw runtime layer does not yet
  expose a simple public "give me a valid read runtime" bootstrap

Without this milestone:

- store-backed and durable milestones will inherit half-hardened reuse and
  view-shape semantics as if they were final
- product consumers will keep using special-case pack/bind/decode seams where
  Query claims a typed fact-consumption lane
- hostile runtime-backed read tests will keep spending their effort on bridge
  and adapter assembly instead of the read-seam behavior they are supposed to
  certify
- support/profile output will keep advertising core product surfaces as
  admitted-but-debt instead of actually closed

## Governing Summaries

- `MENTALITY.md`: the hard part is not adding one more helper. The hard part is
  preventing public Query product lanes from claiming closure while still
  depending on special-case seams or structurally unfinished neighbors.
- `arch_laws.md`: composition, planning, reuse, support, and certification must
  each have one explicit authority path. Query cannot close debt by smearing
  logic into docs, helpers, and tests separately.
- `composition_laws.md`: scopes, templates, view families, grouped planning,
  retained-artifact consumption, preserved reuse, and raw runtime bootstrap are
  different products and need separate boundary homes.
- `domain_structure_laws.md`: reusable composition, view-shape meaning,
  retained-artifact fact extraction, and temporal/async reuse posture must stay
  explicit instead of disappearing into generic runtime support or helper bags.
- `perf_laws.md`: debt closure cannot hide broad rescans, reopen lower-source
  artifacts, or make retained-artifact fact extraction more expensive just to
  look cleaner. The common path must remain bounded and honest, and hostile
  runtime testing must not require large assembly tax just to reach an
  ordinary valid read-runtime state.
- `AI_README.md`: Query is the ordinary runtime and product facade. Reusable
  composition, inspection, support/admission, projection consumption, and
  continuation/reuse must extend the existing Query categories rather than
  inventing sidecar semantics.
- `worth_query_roadmap.md`: merged `9.4` closes temporal/async semantics first.
  This milestone belongs immediately after it so productization debt is removed
  before store-backed and durable milestones freeze it.
- `milestone-9.4.md`: preserved temporal/async meaning is now closed at the
  runtime-backed semantic layer. This milestone must keep that meaning intact
  when composition, view-shape reuse, and retained artifacts participate.
- `test-requirements.md`: Query closes only when hostile certification proves
  canonical meaning, typed rejection, exact counters, and narrow artifact
  comparison. This milestone must add full-build certification rather than
  treating docs or support markers as self-certifying.

## Adversarial Constraint

For the same canonical query declaration, scope/template expansion,
view-shape family, grouped composition posture, retained result artifact,
basis/remask posture, and preserved temporal/async reuse posture, Query must
produce the same canonical declaration identity, the same support/admission
posture, the same typed fact-consumption contract, and the same delivery/reuse
meaning regardless of whether the caller reaches the lane through direct
composition, grouped composition, inspector/grouped reuse, or retained
artifact consumption.

This milestone fails if any covered path:

- treats scope or template composition as string substitution instead of
  canonical declaration composition
- treats view shape as display-only sugar instead of planning, delivery, and
  reuse semantics
- forces consumers back onto special-case runtime-owned pack/bind/decode seams
  where Query claims a typed projection-consumption lane
- preserves temporal/async vocabulary but erases the runtime-backed meaning
  closed in [milestone-9.4.md](./milestone-9.4.md)
- or leaves any covered public lane unfinished at milestone close

## Product Decision Lock

- This is a debt-cleanup milestone, not a capability-expansion milestone.
- The goal is to close runtime-backed productization debt in already-visible
  Query lanes, not to invent store-backed or durable semantics early.
- Scope/template expansion remains canonical query composition, not host-local
  rewrite folklore.
- View-shape families remain part of planning, delivery, identity, and reuse
  semantics, not presentation-only sugar.
- Projection consumption remains the typed fact lane. Retained derived/live
  artifacts must participate through that ordinary product path for the covered
  families in this milestone.
- Preserved temporal/async reuse must carry forward merged `9.4` semantics; it
  may not erase time/async posture during saved-query, scope/template, or
  view-shape reuse.
- Raw runtime bootstrap must expose a simple public ordinary lane for a valid
  bridge-backed read runtime. Hostile certification should not need custom
  one-off runtime assembly just to reach the read seam.
- This milestone fully builds the runtime-backed product surface it names.
  Milestones `10` and `11` own additional store-backed and durable capabilities;
  they do not excuse unfinished runtime-backed work here.

## Phase Plan

This milestone stays one milestone, but the execution program is intentionally
more granular than the original draft. Each phase below closes one explicit
authority slice rather than bundling several closure jobs under one heading.

### Phase 1: Named Scope Expansion Identity Closure

Close `named_scope_expansion:debt` as a canonical composition problem, not a
doc-label problem.

- Relevant subsystems: `composition::scopes`, composition reporting, application
  support/profile reporting.
- Relevant source surfaces:
  [composition/scopes/expansion.rs](../../crates/worth-query/src/composition/scopes/expansion.rs),
  [composition/report/support_profile.rs](../../crates/worth-query/src/composition/report/support_profile.rs),
  [application/support/report.rs](../../crates/worth-query/src/application/support/report.rs).
- Test requirements: prove named-scope expansion normalizes to the same
  canonical declaration identity as equivalent direct construction.

### Phase 2: Template Instantiation Identity Closure

Close `template_instantiation:debt` as ordinary canonical declaration
composition.

- Relevant subsystems: `composition::templates`, composition reporting,
  application support/profile reporting.
- Relevant source surfaces:
  [composition/templates/instantiation.rs](../../crates/worth-query/src/composition/templates/instantiation.rs),
  [composition/report/support_profile.rs](../../crates/worth-query/src/composition/report/support_profile.rs),
  [application/support/report.rs](../../crates/worth-query/src/application/support/report.rs).
- Test requirements: prove template instantiation normalizes to the same
  canonical declaration identity as equivalent direct construction.

### Phase 3: Composition Support-Profile And Docs Closure

Flip the public composition support posture from debt to closed runtime-backed
truth for the covered scope/template lanes and remove matching doc debt text.

- Relevant docs:
  [Scopes, Templates, Saved Queries, And View Shapes](../../crates/worth-query/docs/authoring/scopes-templates-saved-queries-and-view-shapes.md),
  [Read Composition](../../crates/worth-query/docs/authoring/read-composition.md),
  [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md).
- Warnings: do not hide statuses from the profile; do not let docs get ahead
  of implementation.
- Test requirements: add the support/profile parity assertions for scope and
  template closure.

### Phase 4: Table And Detail View Closure

Close `table` and `detail` as core view families with canonical planning,
delivery, and reuse semantics rather than happy-path row rendering.

- Relevant subsystems: `view_shape`, `view_shape_live`, application
  support/profile reporting.
- Relevant source surfaces:
  [view_shape/planning.rs](../../crates/worth-query/src/view_shape/planning.rs),
  [view_shape/delivery.rs](../../crates/worth-query/src/view_shape/delivery.rs),
  [application/support/report.rs](../../crates/worth-query/src/application/support/report.rs).
- Test requirements: prove `table` and `detail` move from `debt` to closed
  runtime-backed product readiness with canonical artifacts.

### Phase 5: Inspector Detail View Closure

Close `inspector_detail_observed` and `inspector_detail_focused` as real
inspector product lanes with stable planning, delivery, and identity posture.

- Relevant subsystems: `view_shape`, identity-evolution inspector support,
  `view_shape_live`.
- Relevant source surfaces:
  [view_shape/family.rs](../../crates/worth-query/src/view_shape/family.rs),
  [identity_evolution/inspector.rs](../../crates/worth-query/src/identity_evolution/inspector.rs),
  [view_shape_live/family.rs](../../crates/worth-query/src/view_shape_live/family.rs).
- Warnings: do not let focused and observed inspector variants drift into
  local semantics under retained or live use.

### Phase 6: Core View Support-Profile And Docs Closure

Flip the public view-family rows for non-grouped core views and remove the
remaining doc wording that still teaches them as admitted debt.

- Relevant docs:
  [Scopes, Templates, Saved Queries, And View Shapes](../../crates/worth-query/docs/authoring/scopes-templates-saved-queries-and-view-shapes.md),
  [Collections, Cursors, Ordering, And Aggregations](../../crates/worth-query/docs/authoring/collections-cursors-ordering-and-aggregations.md).
- Test requirements: prove exact status movement for `table`, `detail`,
  `inspector_detail_observed`, and `inspector_detail_focused`.

### Phase 7: Kanban Grouped View Closure

Close `kanban_grouped` as a view-family product lane with honest grouped
planning, grouped delivery posture, and no residual refresh-only debt policy.

- Relevant subsystems: grouped view-shape planning/binding, grouped live
  execution.
- Relevant source surfaces:
  [view_shape/grouped_planning.rs](../../crates/worth-query/src/view_shape/grouped_planning.rs),
  [view_shape/grouped_binding.rs](../../crates/worth-query/src/view_shape/grouped_binding.rs),
  [view_shape/grouped_policy.rs](../../crates/worth-query/src/view_shape/grouped_policy.rs).
- Warnings: do not mark closure merely because grouped rows render.

### Phase 8: Grouped Composition Closure

Close grouped reusable composition as its own boundary rather than routing
everything through hidden ordinary collection plans.

- Relevant subsystems: `grouped_authoring`, grouped support/profile reporting.
- Relevant source surfaces:
  [grouped_authoring/declaration.rs](../../crates/worth-query/src/grouped_authoring/declaration.rs),
  [grouped_authoring/orchestration.rs](../../crates/worth-query/src/grouped_authoring/orchestration.rs),
  [grouped_authoring/posture.rs](../../crates/worth-query/src/grouped_authoring/posture.rs),
  [grouped_authoring/support.rs](../../crates/worth-query/src/grouped_authoring/support.rs).
- Test requirements: prove grouped declarations preserve canonical grouped
  identity across direct and reusable composition paths.

### Phase 9: Grouped Support-Profile And Docs Closure

Remove the explicit grouped template/composition debt wording from public docs
and support/profile surfaces after grouped implementation closure is real.

- Relevant docs:
  [Collections, Cursors, Ordering, And Aggregations](../../crates/worth-query/docs/authoring/collections-cursors-ordering-and-aggregations.md),
  [Scopes, Templates, Saved Queries, And View Shapes](../../crates/worth-query/docs/authoring/scopes-templates-saved-queries-and-view-shapes.md).
- Warnings: documentation wording counts as part of the closure surface.

### Phase 10: Retained Artifact Source-Family Admission Closure

Close the admission gap where retained derived-artifact and live-artifact
bindings are not yet first-class projection-consumption source families.

- Relevant subsystems: `projection_consumption`, retained/live artifact binding
  seams.
- Relevant source surfaces:
  [projection_consumption/source.rs](../../crates/worth-query/src/projection_consumption/source.rs),
  [projection_consumption/contracts.rs](../../crates/worth-query/src/projection_consumption/contracts.rs).
- Warnings: do not wrap the same escape hatch in a new name and call it done.

### Phase 11: Projection Fact Extraction Unification Closure

Close the typed declaration, contract, extraction, receipt, and envelope path
for the covered retained/live artifact families.

- Relevant source surfaces:
  [projection_consumption/facts/mod.rs](../../workspaces/worth-query/crates/worth-query/src/projection_consumption/facts/mod.rs),
  [projection_consumption/extraction/mod.rs](../../crates/worth-query/src/projection_consumption/extraction/mod.rs),
  [projection_consumption/extraction/grouped.rs](../../crates/worth-query/src/projection_consumption/extraction/grouped.rs),
  [projection_consumption/extraction/query_context.rs](../../crates/worth-query/src/projection_consumption/extraction/query_context.rs),
  [projection_consumption/receipt.rs](../../crates/worth-query/src/projection_consumption/receipt.rs),
  [projection_consumption/envelope.rs](../../crates/worth-query/src/projection_consumption/envelope.rs).
- Test requirements: prove one typed fact path when admitted.

### Phase 12: Projection Escape-Hatch Eradication Closure

Prove the ordinary product path no longer needs the special-case runtime-owned
retained-artifact seams named in this milestone.

- Relevant APIs and product surfaces: `workspace.read(...)`,
  `workspace.observe(...)`, `workspace.materialize(...)`, and the forbidden
  special-case helper path.
- Test requirements: require exact zero assertions for fallback to
  `consume_scalar_fields(...)`, `terminal_json_decode_row_pair(...)`,
  `terminal_json_decode_row_triple(...)`, `verify_scalar_alignment(...)`,
  `read_live_artifact_bundle(...)`, `bind_live_artifact(...)`, and
  `read_live_artifact_binding(...)`.

### Phase 13: Saved-Query Temporal/Async Reuse Matrix Closure

Close the saved-query legality matrix so preserved reuse remains semantically
strict about temporal/async posture.

- Relevant subsystems: saved-query reuse, policy-basis saved reuse.
- Relevant source surfaces:
  [saved_query/reuse/matrix.rs](../../crates/worth-query/src/saved_query/reuse/matrix.rs),
  [saved_query/future_support.rs](../../crates/worth-query/src/saved_query/future_support.rs),
  [policy_basis/saved_reuse.rs](../../crates/worth-query/src/policy_basis/saved_reuse.rs).
- Warnings: do not preserve the vocabulary while erasing the meaning closed in
  [milestone-9.4.md](./milestone-9.4.md).

### Phase 14: Inspector And Grouped Preserved-Reuse Propagation Closure

Close preserved temporal/async posture across inspector and grouped reuse
neighbors so they carry the same runtime-backed meaning rather than downcast.

- Relevant subsystems: preview/runtime preserved reuse, view-shape and grouped
  preserved reuse.
- Relevant source surfaces:
  [runtime/tests/preview/temporal_async.rs](../../crates/worth-query/src/runtime/tests/preview/temporal_async.rs),
  [query_context/scoped.rs](../../crates/worth-query/src/query_context/scoped.rs),
  [view_shape/family.rs](../../crates/worth-query/src/view_shape/family.rs),
  [grouped_authoring/posture.rs](../../crates/worth-query/src/grouped_authoring/posture.rs).

### Phase 15: Preserved-Reuse Parity And Downcast-Rejection Closure

Add the hostile proofs showing grouped and inspector preserved neighbors do not
silently collapse to plain non-temporal or non-async reuse.

- Shared crate usage:
  `worth_proof::facade::TransitionReadiness`,
  `worth_proof::facade::TransitionOutcome` only where preserved reuse already
  crosses typed reuse/readmission progression boundaries.
- Test requirements: add the `Preserved Temporal Async Reuse Closure Test`.

### Phase 16: Runtime Builder Read-Bootstrap Authority Closure

Close the raw runtime builder debt so valid read-runtime bootstrap remains one
authority path through the real builder rather than custom harness folklore.

- Relevant subsystems: runtime builder and backend-parts assembly.
- Relevant source surfaces:
  [runtime/builder.rs](../../crates/worth-query/src/runtime/builder.rs),
  [runtime/backend/parts.rs](../../crates/worth-query/src/runtime/backend/parts.rs),
  [runtime/error.rs](../../crates/worth-query/src/runtime/error.rs).
- Warnings: do not solve this only inside test support.

### Phase 17: Public Bridge-Backed Read Bootstrap Surface Closure

Expose the simple public valid bridge-backed read-runtime bootstrap as the
ordinary lane for hostile testing and examples.

- Relevant source surfaces:
  [tests/support/public_bridge_runtime/mod.rs](../../crates/worth-query/tests/support/public_bridge_runtime/mod.rs),
  [runtime/tests/support/bridge/runtime_support.rs](../../crates/worth-query/src/runtime/tests/support/bridge/runtime_support.rs),
  [runtime/tests/support/stateful_bridge_runtime/mod.rs](../../crates/worth-query/src/runtime/tests/support/stateful_bridge_runtime/mod.rs).
- Relevant product surfaces: `WORTHQueryRuntime::builder()`, `runtime_bridge(...)`,
  `build_backend_from_parts()`.

### Phase 18: Hostile Harness Migration To Public Bootstrap Closure

Move hostile runtime-backed read certification off custom minimal scaffolding
and onto the public bootstrap lane.

- Relevant docs:
  [Workspace Overview](../../crates/worth-query/docs/foundations/workspace-overview.md),
  [Reads, Observation, and Materialization](../../crates/worth-query/docs/runtime-surfaces/reads-observe-materialize.md).
- Test requirements: add the `Raw Runtime Read Bootstrap Simplicity Test`.

### Phase 19: Residual Support/Profile Debt-Marker Closure

Remove the remaining covered debt markers from support/profile publication once
the underlying runtime-backed lanes are truly closed.

- Relevant subsystems: application support/profile reporting, runtime support
  profile output.
- Relevant source surfaces:
  [application/support/report.rs](../../crates/worth-query/src/application/support/report.rs),
  [application/support/registry.rs](../../crates/worth-query/src/application/support/registry.rs),
  [runtime/support/profile.rs](../../crates/worth-query/src/runtime/support/profile.rs).
- Warnings: do not remove debt text before the implementation is actually
  closed.

### Phase 20: Public Docs Debt-Wording Closure

Perform the milestone-wide doc closeout pass so the covered public product
lanes stop advertising admitted debt after implementation closure.

- Relevant docs: the authoring, capability, and foundations docs touched in
  Phases 1 through 18.
- Engineering decision: docs are authority surfaces here, not afterthoughts.

### Phase 21: Public Doc Coverage Assertion Closure

Close the documentation-coverage proof so public docs, support profiles, and
runtime-backed truth agree exactly.

- Relevant source surface:
  [public_doc_coverage/tests/support.rs](../../crates/worth-query/src/public_doc_coverage/tests/support.rs).
- Shared crate usage:
  `worth_foundational::facade::DiagnosticRichnessProfile` for debt-closeout
  reporting richness.

### Phase 22: Lane-Local Hostile Certification Closure

Close the lane-local hostile suites for composition, view-shape,
projection-consumption, preserved reuse, and bootstrap semantics.

- Relevant source surfaces:
  [application/tests.rs](../../crates/worth-query/src/application/tests.rs),
  [projection_consumption/certification/mod.rs](../../crates/worth-query/src/projection_consumption/certification/mod.rs),
  [view_shape/tests.rs](../../crates/worth-query/src/view_shape/tests.rs),
  [saved_query/tests.rs](../../crates/worth-query/src/saved_query/tests.rs).
- Warnings: do not close on broad support-report equality alone.

### Phase 23: Cross-Lane Hostile Certification Matrix Closure

Close the combined milestone hostile matrix that varies composition path,
view-family, retained-artifact extraction, preserved temporal/async reuse, and
raw runtime bootstrap pressure together.

- Test requirements: add the `Milestone 9.5 Debt-Close Hostile Certification Matrix`.
- Require narrow canonical artifacts for composition, view family,
  projection-consumption contract, preserved temporal/async reuse, raw runtime
  read-bootstrap, and support-profile digests.

### Phase 24: Forbidden-Fallback Zero-Proof Closeout

Close the milestone only after exact zero assertions prove there is no
forbidden fallback to retained-artifact escape hatches and no forbidden custom
bootstrap scaffolding in the ordinary hostile read-runtime path.

- Shared crate usage:
  `worth_foundational::facade::FoundationalBoundaryArtifactCompileFailBoundary`,
  `worth_foundational::facade::FoundationalBoundaryEvidenceCompileFailBoundary`,
  `worth_foundational::facade::FoundationalPerformanceCompileFailBoundary`,
  and matching `...ProductionTestReadyArtifact` surfaces if milestone-close
  artifacts are emitted.
- Engineering decision: this milestone closes on hostile proof, not on the
  removal of `debt` strings.

## Must Ship

- runtime-backed closure for named-scope expansion and template instantiation
- runtime-backed closure for the admitted core view families `table`, `detail`,
  `inspector_detail_observed`, `inspector_detail_focused`, and
  `kanban_grouped`
- grouped composition closure with no remaining explicit composition debt in
  the admitted public lane
- first-class projection-consumption source-family closure for retained
  derived-artifact bindings and live-artifact bindings
- preserved inspector/grouped temporal/async reuse closure across the covered
  runtime-backed reuse surface
- simple public raw runtime bootstrap for a valid bridge-backed read runtime so
  hostile tests can focus on read-seam behavior instead of custom assembly
- support/profile, docs, and hostile certification closure for all covered debt
  families

## Must Preserve

- canonical declaration identity across direct construction and reusable
  composition
- view-family meaning as planning, delivery, identity, and reuse semantics
- projection consumption as the typed fact lane rather than row-bag folklore
- merged `9.4` runtime-backed temporal/async meaning across preserved reuse
- honest runtime-backed-first posture without leaking fake durable/store-backed
  claims early

## Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the relevant Milestone `9.5` certification suites added to
  [test-requirements.md](./test-requirements.md) all pass with narrow
  machine-checkable artifacts
- `application/tests.rs` no longer reports the covered scope/template and
  core view-family rows as `debt`
- grouped composition docs and support/profile surfaces no longer describe the
  admitted grouped planning lane as composition debt
- retained derived-artifact and live-artifact ordinary product paths no longer
  require the special-case runtime-owned seams listed in Phase 4
- preserved inspector/grouped temporal/async reuse neighbors are built with
  canonical runtime-backed semantics preserved across the covered reuse surface
- hostile runtime-backed read tests no longer need a custom minimal
  bridge-backed runtime harness just to obtain a valid raw runtime read lane

## Sequencing Notes

- This milestone belongs immediately after the merged
  [milestone-9.4.md](./milestone-9.4.md) closure so runtime-backed
  temporal/async semantics are already frozen before reuse and productization
  work tries to preserve them.
- It belongs before Milestones `10` and `11` because store-backed and durable
  milestones should not inherit half-hardened reuse, view-shape, and retained
  fact-consumption surfaces as if they were final.
- It should be implemented as an explicit per-family build program, not as
  scattered opportunistic cleanup.
