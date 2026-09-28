## Milestone 6 Named Certification Suites

### 6. Historical / Diff / Basis Parity Test

Purpose

Prove that current, branch-scoped, historical, and diff query contexts preserve
the same canonical query meaning apart from the explicitly declared basis.

Scenario

- run the same canonical query against:
  - current branch head
  - alternate branch head
  - historical commit/snapshot basis where admitted
  - diff/comparison between two declared bases
- compare runtime-backed and store-backed historical execution where admitted

Must verify

- basis identity is explicit in every lane
- historical results preserve the same result-shape meaning as current reads
- diff outputs remain query-shaped rather than raw storage deltas
- admitted runtime/store historical paths compare equal

Required verification output

- `query_digest`
- `basis_digest`
- `result_digest`
- `replay_digest`
- `counter_snapshot`

Pass condition

Historical and diff query execution remains basis-explicit and parity-safe.

## Milestone 7 Named Certification Suites

### 7. Lineage And Correspondence Query Parity Test

Purpose

Prove that lineage traversal and correspondence-aware queries remain explicit
about continuity, ambiguity, and rejection.

Scenario

- run lineage-aware queries over:
  - replacement
  - split
  - branch-local divergence
  - ambiguous correspondence candidates
  - explicitly rejected correspondence

Must verify

- authoritative lineage remains distinct from advisory correspondence
- ambiguous correspondence never silently becomes continuity
- branch-local identity evolution stays local unless the truth basis says
  otherwise
- replay preserves lineage/correspondence meaning

Required verification output

- `query_digest`
- `lineage_digest`
- `result_digest`
- `failure_digest`
- `replay_digest`

Pass condition

Identity-evolution query meaning remains typed, replay-safe, and ambiguity-
honest.

## Milestone 8 Named Certification Suites

### 8. Scope / Template / View-Shape Semantic Parity Test

Purpose

Prove that reusable query composition and admitted view shapes preserve the
same canonical meaning as direct construction while adding real planning and
live-maintenance semantics.

Scenario

- compare direct query construction to:
  - scope-composed queries
  - template-instantiated queries
- run admitted view shapes including:
  - table/detail
  - one grouped or temporal view
  - inspector-style detail if shipped

Must verify

- scopes/templates normalize to the same canonical query meaning as direct
  construction
- view shapes affect planning, invalidation, delivery, and patch semantics
- shipped view shapes do not exist only as cosmetic typing

Required verification output

- `query_digest`
- `plan_digest`
- `result_shape_digest`
- `delivery_digest`
- `counter_snapshot`

Pass condition

Composition and view-shape surfaces are semantic query artifacts, not sugar.

