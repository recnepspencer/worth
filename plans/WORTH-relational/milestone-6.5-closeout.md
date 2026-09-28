# Milestone 6.5 Closeout: Invariant Authority Completion

## Status

Milestone 6.5 is closed as of 2026-03-23.

The runtime now treats structural legality as an explicit authority surface
rather than as a small set of legacy checks plus ad hoc extension points.

The semantic center shipped in this milestone is:

invariants are now represented as named authority contracts with stable rule
identity, schema-lowered native structural legality, packet-owned custom
execution, panic-safe evaluation, provenance-bearing diagnostics, and canonical
decision records consumed by commit/publication enforcement rather than by
best-effort after-the-fact analysis.

This is not "more validators were added."

The runtime now owns:

- native invariant completion for payload schema, partition isolation,
  acyclicity, and connectivity minimum
- custom structural invariant registration with stable semantic identity and
  frozen runtime registry ownership
- packet-owned custom preparation/execution so custom scope does not escape as a
  framework-level `Any` seam
- panic capture at custom preparation and execution boundaries
- structural custom execution context with bounded traversal and cumulative
  session budgets
- typed invariant failure fields for custom failures and new native structural
  contract families
- canonical per-result custom provenance summaries
- canonical invariant decision records derived from invariant execution results
- compile-fail coverage for key public boundary sealing rules

## Shipped Scope

Milestone 6.5 delivered:

- stable invariant identity surfaces through:
  - `InvariantRuleId`
  - `NativeInvariantRuleId`
  - `CustomInvariantRuleId`
  - `CustomInvariantSemanticIdentity`
  - `CustomInvariantSemanticVersion`
- public and internal descriptor separation for native and custom invariant
  rules
- frozen runtime-owned custom invariant registry with duplicate semantic
  identity rejection
- planner lowering of custom invariant registrations into real invariant work
  packets rather than metadata-only sidecars
- production custom invariant execution over structural truth surfaces:
  - touched visible entities and relations
  - planned entity and relation creates
  - payload access
  - relation metadata access
  - bounded cumulative traversal
  - structural count summaries
- panic-safe custom invariant preparation and evaluation with typed
  `CustomInvariantFailure` reporting
- custom invariant counters for:
  - preparation count
  - execution count
  - panic count
  - traversal frontier count
  - traversal step count
- custom provenance attached to invariant results and surfaced in detailed
  invariant diagnostics
- new schema-side structural invariant declaration surfaces for:
  - payload schema contracts
  - acyclicity contracts
  - partition isolation contracts
  - connectivity minimum contracts
- native rule and registration support for:
  - `PayloadSchemaContract`
  - `AcyclicityContract`
  - `PartitionIsolationContract`
  - `ConnectivityMinimumContract`
- real evaluator coverage for:
  - payload schema rejection
  - partition isolation rejection
  - planned-cycle rejection
  - publication-stage connectivity minimum rejection
- canonical invariant decision-log support through:
  - `InvariantDecisionKind`
  - `InvariantDecisionRecord`
  - `InvariantExecutionResult::decision_log()`
- compile-fail/UI coverage proving:
  - external callers cannot invoke the private `InvariantRegistration::for_rule`
    constructor
  - external callers cannot reach internal prepared custom execution surfaces

## Phase Completion Map

Milestone 6.5 is considered closed against the implementation work carried out
in this phase set.

### Phase 1: Structural Split and Type-System Foundation

Closed by:

- `src/validation/data/rule_id.rs`
- `src/validation/data/descriptor.rs`
- `src/validation/data/custom_rule.rs`
- `src/validation/logic/custom_registry.rs`
- `src/authority/commit/preparation/packets/invariant.rs`

What is proven:

- invariant identity is stable and explicit for both native and custom rules
- custom invariant semantic identity is separated from runtime execution
  ownership
- custom execution no longer enters the worker through a loose
  rule-id-plus-`Any` routing path

### Phase 2: Native Invariant Surface Completion

Closed by:

- `src/schema/data/structural_invariants.rs`
- `src/schema/data/relation_integrity.rs`
- `src/schema/logic/aspect_plans.rs`
- `src/logic/runtime/state/subsystems/aspect_semantics.rs`
- `src/validation/data/rules.rs`
- `src/validation/data/catalog.rs`
- `src/validation/engine/evaluator.rs`

What is proven:

- payload schema, acyclicity, partition isolation, and connectivity minimum now
  exist as first-class schema/native invariant concepts
- schema lowering and runtime registration account for the new native families
- invariant execution no longer needs milestone-specific one-off wiring to
  understand these rule kinds

### Phase 3: Production Custom Invariant Path

Closed by:

- `src/validation/data/custom_rule.rs`
- `src/validation/execution/planning.rs`
- `src/validation/execution/worker.rs`
- `src/validation/engine/state_view.rs`
- `src/validation/engine/engine.rs`

What is proven:

- custom invariants execute over real structural runtime state rather than test
  placeholders
- traversal budgets accumulate across the whole custom rule execution session
- custom prepare/evaluate panics are captured and turned into typed invariant
  failures instead of crashing the runtime

### Phase 4: Diagnostics, Provenance, and Decision Surfaces

Closed by:

- `src/validation/data/results.rs`
- `src/validation/data/execution.rs`
- `src/validation/engine/result.rs`
- `src/validation/logic/invariant_authority.rs`
- `src/transactions/data/outcomes.rs`

What is proven:

- native and custom failures now emit machine-readable structural violation
  fields
- custom execution provenance is preserved on invariant results
- invariant execution now exposes canonical decision records rather than only a
  bag of result verdicts
- commit-level validation summaries account for custom failures and custom panic
  captures

### Phase 5: Mechanical Boundary Enforcement

Closed by:

- `tests/ui.rs`
- `tests/ui/invariants/registration_constructor_is_private.rs`
- `tests/ui/invariants/prepared_custom_execution_is_not_public.rs`

What is proven:

- callers outside the crate cannot mint arbitrary invariant registrations using
  the internal registration constructor
- callers outside the crate cannot depend on or WORTH internal prepared custom
  execution surfaces

## Verification

Milestone 6.5 closeout was verified with the following passing lanes:

- `cargo test -p worth-relational validation::data:: --quiet`
- `cargo test -p worth-relational validation::engine:: --quiet`
- `cargo test -p worth-relational validation::logic::invariant_access:: --quiet`
- `cargo test -p worth-relational --test ui --quiet`

The implementation also added direct milestone-specific proof tests for:

- `engine_rejects_entity_payloads_that_violate_payload_schema_contracts`
- `engine_rejects_cross_partition_relations_under_partition_isolation_contracts`
- `engine_rejects_planned_cycles_under_acyclicity_contracts`
- `commit_publication_stage_rejects_sources_without_required_connectivity`
- `engine_executes_custom_packets_against_real_structural_surfaces`
- `engine_captures_custom_prepare_panics_as_typed_failures`
- `engine_captures_custom_evaluate_panics_as_typed_failures`
- `invariant_ui_boundaries_are_sealed`

## Important Semantics at Closeout

The implementation is explicit about the current publication-boundary behavior:

- connectivity minimum is enforced at the publication stage
- publication-stage invariant failure is surfaced as a publication failure in
  the current commit pipeline
- the current runtime does not persist a committed-but-unpublished head as a
  separate durable authority state

That behavior is now visible and typed rather than accidental or silent. It is
an important semantic fact for downstream milestone work and should be treated
as a current authority-pipeline property, not as an unstated assumption.

## Not in This Closeout

Milestone 6.5 closeout does not include:

- the Phase 10 causal metadata endcap that was appended to the 6.5 plan as a
  post-closeout follow-on
- distributed multi-writer reconciliation semantics
- domain-specific geometry legality libraries above the generic structural
  invariant runtime

Those are now set up on a stronger authority base than they had before this
milestone.

## Final State

`worth-relational` now has a materially more honest invariant subsystem:

- native structural legality is broader and explicit
- custom structural legality is runtime-owned, panic-safe, and provenance-rich
- invariant execution is observable through canonical decision records
- key public misuse paths are compile-time sealed

This closes the invariant-authority completion work needed before the next
phase of intent, causal, and domain-grade legality expansion.
