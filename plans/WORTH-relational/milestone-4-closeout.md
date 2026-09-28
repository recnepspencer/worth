# Milestone 4 Closeout: Relation Integrity and Schema Contracts

## Status

Milestone 4 is closed as of 2026-03-21.

The runtime now treats relation legality as schema-declared, invariant-enforced
truth rather than host-side validation folklore or post-commit aftercare.

The semantic center shipped in this milestone is:

relation legality is authoritative commit-time truth, lowered from generic
schema contracts into runtime-owned invariant work, with deterministic failure,
replay, recovery, rollback, and diagnostics behavior.

## Shipped Scope

Milestone 4 delivered:

- schema-declared relation-integrity contracts nested under
  `RelationKindRegistration`
- distinct contract families for endpoint legality, cardinality, uniqueness,
  symmetry, and endpoint-deletion integrity
- explicit separation between declaration model, lowered rule model, and
  commit-time execution packets
- invariant-pipeline enforcement as the single semantic authority for relation
  legality
- planner/executor proof-boundary metadata for relation-integrity work
- typed relation-integrity failures and conflict surfaces with structured
  localization fields
- durable relation-integrity compatibility checks with contract-family-aware
  mismatch reporting
- branch-local, replay, recovery, and savepoint certification for accepted and
  rejected relation-integrity outcomes
- cost-honest relation-integrity counters and proof tests for narrowed hot
  paths
- publication-facing diagnostics that preserve specific invariant failure codes,
  localization fields, and proof-boundary summaries

Before closeout, the implementation also removed or tightened several paths
that would have undermined milestone honesty if left in place:

- the dead legacy invariant execution path was removed so packet-backed
  execution is the only runtime-owned path
- `Replace` now participates in relation-integrity applicability and
  endpoint-deletion enforcement rather than slipping past narrowed planning
- `RequireRelationRetirement` was tightened so cascade delete no longer silently
  satisfies an audit-retention contract
- relation-integrity diagnostics at the transaction boundary no longer collapse
  entirely to code/detail; structured invariant fields now survive into commit
  conflicts and published failure artifacts
- the relation-integrity complexity contract moved from debt to verified only
  after narrowed-path proof tests existed for entity-only skip, uniqueness,
  symmetry, and endpoint-deletion execution

## Acceptance Mapping

Milestone 4 is considered closed against the roadmap because the required
acceptance surfaces are now directly covered by code and certification lanes.

### `Savepoint rollback fracture test`

Covered by:

- `tests::publication::cdc::savepoint_residue::rolled_back_illegal_relation_work_leaves_zero_cdc_and_diagnostic_residue`
- `tests::publication::cdc::savepoint_residue::rolled_back_endpoint_deletion_work_leaves_zero_cdc_and_diagnostic_residue`
- `tests::publication::cdc::savepoint_residue::nested_savepoint_abandoned_aspect_work_leaves_zero_patch_cdc_history_and_lineage_residue`

What is proven:

- illegal relation-integrity work abandoned behind nested savepoints leaves zero
  authoritative truth residue
- rejected or rolled-back relation-integrity work does not leak into patch,
  CDC, history, or diagnostics
- endpoint-deletion integrity follows the same rollback-fracture contract as
  relation creation/deletion legality

### `Hostile commit/replay equivalence test`

Covered by:

- `tests::history::replay::replay_contract_preserves_relation_integrity_declared_schema`
- `tests::history::replay::replay_contract_preserves_branch_local_relation_integrity_truth_after_rejected_feature_attempt`
- `tests::durability::contracts::durability_contract_recovery_ignores_rejected_relation_integrity_attempts`
- `tests::transactions::core::relation_integrity::relation_integrity_rejected_branch_local_commit_does_not_advance_truth_or_leak_to_main`

What is proven:

- accepted relation-integrity histories replay canonically from committed
  artifacts
- rejected relation-integrity attempts never become authoritative truth
- branch-local rejected attempts do not leak across branches or pollute replay
  truth
- durable recovery preserves the same accepted/rejected boundary as live
  execution

### `Durable recovery and schema mismatch test`

Covered by:

- `tests::durability::contracts::durability_contract_failure_relation_integrity_plan_mismatch_is_explicit`
- `tests::durability::contracts::durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories`
- `tests::durability::contracts::durability_contract_recovery_ignores_rejected_relation_integrity_attempts`

What is proven:

- recovery rejects incompatible relation-integrity plan revisions explicitly
- durable compatibility reporting preserves relation-integrity contract-family
  context rather than flattening to a generic mismatch
- accepted endpoint-deletion and retirement histories recover canonically
- rejected relation-integrity work does not survive into recovered truth

### `Missing-twin / nonmanifold corruption localization test`

Covered by:

- `tests::validation::logic::invariant_access::commit_boundary_symmetry_failure_fields_localize_missing_twin_endpoints`
- `tests::validation::logic::invariant_access::commit_boundary_cardinality_failure_fields_localize_nonmanifold_like_overflow`
- `tests::publication::observability::invariant_failure_artifact_preserves_specific_code_localization_and_proof_boundary`
- `tests::transactions::core::relation_integrity::relation_integrity_commit_boundary_requires_paired_twin_edge`
- `tests::transactions::core::relation_integrity::relation_integrity_commit_boundary_rejects_source_cardinality_overflow`

What is proven:

- missing-twin symmetry failures localize contract identity, relation kind, and
  offending endpoints
- nonmanifold-like cardinality overflow localizes the contract, entity, count,
  and violated boundary
- the same localization survives from invariant execution into transaction
  conflicts and publication-facing diagnostic artifacts
- proof-boundary metadata is observable alongside localization, so widened or
  narrowed validation scope is inspectable rather than implicit

## Additional Hardening Added Before Close

Milestone 4 closeout also includes these extra hardening lanes beyond the bare
roadmap labels:

- explicit request/planner narrowing so unrelated relation kinds are not
  scheduled for relation-integrity execution
- replace-driven applicability for endpoint legality and endpoint-deletion
  integrity
- direct contract-mode tests for canonical undirected, paired inverse,
  inverse-prohibited, paired twin, normalized symmetric uniqueness, and all
  endpoint-deletion modes
- branch-local divergence coverage for retained-vs-deleted endpoint histories
- helper modularization for relation-integrity schema fixtures, recovery
  harnesses, inspection requests, and savepoint hostile-sequence tests
- proof-boundary observability on invariant execution metadata and published
  invariant diagnostics
- explicit cost proofs for relation-integrity hot paths with the complexity
  contract registry updated to reference those tests

The closeout expectation here was runtime-authoritative legality plus
certification-grade diagnostics and replay honesty, not merely schema fields
and a few validation callbacks.

## Explicit Deferrals

Milestone 4 intentionally does not claim ownership of:

- general schema evolution or subscriber schema transition
- host-defined custom rule DSLs
- authoritative merge/reconciliation completion
- domain-complete CAD or chip product workflows beyond the generic
  relation-integrity certification surfaces

Those remain deferred to later roadmap milestones, especially
`Milestone 5: Schema Evolution, CDC Contract Evolution, and Schema
Reconciliation`, `Milestone 7B: Authoritative Merge Execution`, and later
domain certification work.

Milestone 4 still guarantees the prerequisite truth foundation that later work
must consume:

- schema-declared generic relation legality
- invariant-owned authoritative enforcement
- deterministic rollback, replay, recovery, and rejection behavior
- structured, localizable invariant failures
- inspectable planner/executor proof boundaries
- cost-honest relation-integrity execution

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-relational --lib`
- 324 tests passing

That baseline includes the relation-integrity transaction, savepoint,
replay/recovery, durability mismatch, complexity-contract, publication
observability, and hostile certification lanes added during Milestone 4
closeout.

## Operational Conclusion

Milestone 4 can be treated as closed.
