## Milestone 5 Named Certification Suites

### 5. Live Promotion Convergence And Suppression Test

Purpose

Prove that live-maintained query results converge to the same truth as fresh
query re-execution for the same basis.

Scenario

- promote admitted detail, collection, and bounded-materialization queries to
  live mode
- inject truth changes that are:
  - relevant
  - irrelevant
  - suppressible by declared live suppression policy
- compare live-maintained results to repeated fresh execution

Must verify

- live and fresh execution converge to the same result meaning
- irrelevant updates are suppressed
- query-shaped patches preserve ordering, membership, and projection semantics
- no raw CDC is exposed as the primary consumer contract

Required verification output

- `query_digest`
- `result_digest`
- `delivery_digest`
- `replay_digest`
- `counter_snapshot`

Pass condition

Live promotion preserves canonical query meaning and converges under churn.

## Milestone 5.1 Named Certification Suites

### 5.1. Region-Scoped Live Narrowing And Stream Contract Test

Purpose

Prove that region- or partition-scoped live invalidation and stream-backed
delivery contracts remain query-shaped, narrower than broad aspect invalidation
where admitted, and parity-safe with the same canonical live query meaning.

Scenario

- promote admitted live queries with locality-sensitive scope
- inject changes that:
  - hit a relevant region
  - miss the query's declared region
  - require stream-contract admission or typed denial
- compare region-narrowed live maintenance to fresh re-execution and to the
  broader aspect-level control surface

Must verify

- region-scoped invalidation narrows below broad aspect invalidation where the
  lower runtimes admit that narrowing
- irrelevant off-region changes suppress before visible delivery
- query-shaped live delivery can lower into formal stream contracts without
  semantic drift
- unsupported region/stream combinations fail typed and early

Required verification output

- `query_digest`
- `delivery_digest`
- `replay_digest`
- `counter_snapshot`

Pass condition

Region-scoped live narrowing and stream-backed delivery remain canonical,
explicit, and non-leaking.

## Milestone 5.2 Named Certification Suites

### 5.2. Preview Session Basis And Promotion Parity Test

Purpose

Prove that preview-session-bound query contexts preserve explicit basis and
lifecycle identity, and that preview-versus-promoted comparisons remain
query-native rather than ambient host orchestration.

Scenario

- execute the same canonical query against:
  - ordinary branch basis
  - admitted preview session basis
  - promoted-result comparison where the workflow admits it
- vary preview session lifecycle state without changing the declared canonical
  query shape

Must verify

- preview-session identity is explicit in the bundle
- preview-bound results preserve the same query meaning apart from the
  declared preview basis
- preview-versus-promoted comparison remains typed and explicit
- unsupported preview-session combinations fail typed and early

Required verification output

- `query_digest`
- `basis_digest`
- `result_digest`
- `replay_digest`
- `counter_snapshot`

Pass condition

Preview-session query contexts remain basis-explicit, lifecycle-explicit, and
parity-safe.

## Milestone 5.3 Named Certification Suites

### 5.3. Frontier Planning And Parallel Admission Parity Test

Purpose

Prove that frontier-aware planning and deterministic parallel admission alter
cost posture, not canonical query meaning.

Scenario

- plan and execute admitted bulk/live query families through:
  - frontier-aware serial route
  - frontier-aware parallel-admitted route
  - typed serial fallback where parallel admission is denied
- compare predicted breadth to realized breadth

Must verify

- serial and parallel admitted routes produce identical canonical query/result
  meaning
- planning emits explicit frontier and parallel-admission posture
- serial fallback remains explicit rather than hidden executor behavior
- breadth posture stays mechanically visible in counters

Required verification output

- `query_digest`
- `plan_digest`
- `result_digest`
- `counter_snapshot`

Pass condition

Frontier-aware planning and deterministic parallel admission remain
meaning-preserving and mechanically visible.

## Milestone 5.4 Named Certification Suites

### 5.4. Structural Correspondence And Historical Materialization Path Test

Purpose

Prove that structural correspondence and historical materialization-path
artifacts remain explicit about ambiguity, advisability, and how historical
truth was actually materialized.

Scenario

- run correspondence-aware queries over:
  - lineage-backed cases
  - structural-fingerprint-backed cases
  - ambiguous disagreement cases
- run admitted historical queries over:
  - retained snapshot path
  - delta replay path
  - full reconstruction path where admitted
- compare admitted lanes where:
  - structural candidate discovery stays within one bounded planner-owned
    discovery class
  - historical path execution stays within one admitted planner-owned cost
    posture
  - predicted breadth or span differs from realized work but remains
    explicitly reported as drift rather than silent executor mutation

Must verify

- structural correspondence never silently upgrades into authoritative
  continuity
- ambiguous correspondence remains explicit and typed
- historical result bundles expose materialization-path identity
- correspondence and historical result bundles expose planner-owned cost
  posture identity where admitted
- prediction drift between planned and realized correspondence/history work is
  explicit and typed
- execution never broadens structural candidate discovery into one successful
  broad-scan lane when the plan denied that posture
- execution never chooses replay versus reconstruction on its own after
  planning
- unsupported correspondence or historical-path cases fail typed and early

Required verification output

- `query_digest`
- `lineage_digest`
- `basis_digest`
- `result_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Correspondence and historical materialization-path semantics remain explicit,
typed, and ambiguity-honest.

## Milestone 5.5 Named Certification Suites

### 5.5. Query Workflow Lowering And Writeback Boundary Test

Purpose

Prove that query-authored mutation, merge, branch-workflow, and writeback
declarations lower into lower-crate authorities without `worth-query`
becoming a second mutation engine.

Scenario

- declare admitted query-authored workflows for:
  - mutation intent lowering
  - preview / compare / merge intent
  - conflict inspection
  - post-merge inspection
  - query-triggered writeback declaration
- compare lowered artifacts and outcomes against authoritative lower-crate
  control lanes

Must verify

- query-authored mutation intents lower into relational commit/merge surfaces
  without semantic drift
- query-triggered writeback declarations lower into bridge-owned writeback
  surfaces without hiding causality or idempotence semantics
- workflow bundles preserve explicit authority boundaries
- unsupported workflow families fail typed and early

Required verification output

- `query_digest`
- `plan_digest`
- `result_digest`
- `delivery_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Workflow lowering remains authority-preserving, typed, and non-duplicative.

## Milestone 5.6 Named Certification Suites

### 5.6. Unified Facade And Configuration Boundary Test

Purpose

Prove that the unified application facade and unified runtime configuration
make `worth-query` a real daily-driver surface without erasing subsystem
ownership or collapsing configuration into a bag.

Scenario

- exercise admitted application-facing surfaces through the unified facade
- resolve unified configuration for admitted runtime-backed capability mixes
- compare support metadata/capability advertisement to actual admission
  behavior

Must verify

- the unified facade preserves lower-crate authority boundaries explicitly
- unified configuration remains sectioned by subsystem ownership
- unsupported composed capabilities fail typed and early
- support metadata and executable admission behavior stay in sync

Required verification output

- `query_digest`
- `plan_digest`
- `support_matrix_digest`
- `capability_registry_digest`
- `counter_snapshot`

Pass condition

The unified facade/configuration surface is coherent for developers while
remaining structurally honest about ownership and support.

