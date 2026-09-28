## Milestone 1 Named Certification Suites

### 1. Canonical Query Normalization Parity Test

Purpose

Prove that equivalent query intent expressed through different admitted
construction paths produces the same canonical query artifact.

Scenario

- build equivalent detail and collection queries through at least:
  - direct construction
  - builder/combinator composition
  - scope/template expansion where admitted at this milestone
- vary helper ordering and host binding descriptors without changing query
  meaning

Must verify

- equivalent query construction yields identical canonical query digests
- result-shape meaning is preserved across equivalent construction paths
- host-local helper layering does not create alternate canonical meaning

Required verification output

- `query_digest`
- `result_shape_digest`
- `canonicalization_report`
- `counter_snapshot`

Pass condition

Equivalent construction paths normalize to identical canonical query meaning.

## Milestone 2 Named Certification Suites

### 2. Schema-Aware Rejection And Projection Legality Test

Purpose

Prove that invalid predicates, projection requests, traversal clauses, and
structured-content queries fail before execution.

Scenario

- attempt legal and illegal queries involving:
  - unknown aspects
  - incompatible field predicates
  - illegal traversal edges
  - invalid result-shape bindings
  - structured-content projections/predicates outside schema allowance
  - workflow-aware predicates with invalid context/shape

Must verify

- illegal queries fail during validation rather than planning/execution
- legal queries lower deterministically after validation
- no silent whole-entity widening occurs on invalid or unsupported requests

Required verification output

- `query_digest`
- `failure_digest`
- `validation_rejection_matrix`
- `counter_snapshot`

Pass condition

Schema-invalid and structurally illegal queries fail early, typed, and without
semantic widening.

## Milestone 3 Named Certification Suites

### 3. Planner / Executor / Binding Parity Test

Purpose

Prove that canonical query meaning survives planning, execution, and admitted
type-bound binding paths.

Scenario

- execute the same canonical queries through:
  - direct runtime-backed execution
  - independently re-planned runtime-backed execution
  - admitted type-bound binding descriptors
  - store-backed execution where the store path is already admitted

Must verify

- equivalent runs produce identical plan and result semantics
- executor does not rediscover planner-owned legality or scope decisions
- type-bound descriptors round-trip to the same canonical plan
- intentionally different admitted runtime route shapes produce distinct
  canonical plan/result evidence
- admitted runtime/store path pairs compare equal

Required verification output

- `query_digest`
- `plan_digest`
- `result_digest`
- `basis_digest`
- `counter_snapshot`

Pass condition

Planning and execution paths are parity-safe for the same canonical query and
basis.

## Milestone 4 Named Certification Suites

### 4. Collection, Cursor, Rollup, And CDC Shape Parity Test

Purpose

Prove that large-surface query behavior remains query-shaped, bounded, and
basis-honest.

Scenario

- run collection queries with:
  - ordering
  - cursor advancement
  - bounded traversal/materialization
  - aggregation/rollups
  - query-time derived fields
  - CDC-shaped output rendering

Must verify

- cursor advancement is stable for one basis
- traversal stays within declared scope
- rollups and derived fields remain basis-honest
- CDC-shaped output matches ordinary query meaning for the same query

Required verification output

- `query_digest`
- `result_digest`
- `delivery_digest`
- `cursor_progress_report`
- `counter_snapshot`

Pass condition

Collection semantics, derived result semantics, and CDC-shaped output remain
canonical and bounded.

