## Cross-Milestone Query Support And Honesty Suites

These suites cut across milestone boundaries. They exist to prove that the
query subsystem's admitted support surface, fallback behavior, semantic
reference truth, artifact lifecycle, schema evolution behavior, diagnostics
sufficiency, and beta support claims remain honest as the feature surface
widens.

### 14. Admitted Query Family Boundary Test

Purpose

Prove that admitted query-family combinations execute canonically, while
non-admitted combinations fail explicitly before semantic drift, fallback
degradation, or silent widening occurs.

Scenario

- exercise a curated admitted/non-admitted matrix including cases like:
  - supported detail + live + policy mask + historical basis
  - supported preview-session basis + diff inspection + admitted merge workflow
  - unsupported grouped-view + lineage + CDC-shaped output + saved-query reload
  - supported rollup + tenant schema variant
  - unsupported writeback declaration + masked aspect trigger + denied tenant
    context
  - unsupported structured-content predicate inside an unshipped view-shape
    family
  - supported subscription declaration + policy mask + grouped view
  - unsupported subscription declaration + raw CDC fallback + durable restart
    request
  - supported temporal subscription + policy mask + time-only wake
  - unsupported temporal subscription + ambient host clock
  - supported async resource query + retry + stale completion denial
  - unsupported async resource family + fallback to host-local loading state
  - supported mixed truth/time/async delivery + preview discard
  - unsupported mixed-cause delivery + host-arrival-order fallback
- compare runtime capability advertisement against actual admission behavior

Must verify

- admitted combinations execute and preserve canonical meaning
- non-admitted combinations fail typed and early
- no unsupported combination sneaks through via fallback, widening, or partial
  degradation

Required verification output

- `query_digest`
- `failure_digest`
- `support_matrix_digest`
- `counter_snapshot`

Pass condition

Admitted combinations pass canonically, and non-admitted combinations fail
explicitly before semantic drift.

### 15. Fallback Non-Leakage / No Silent Widening Test

Purpose

Prove that unsupported or non-admitted query requests never widen, degrade, or
fall back silently into a semantically different execution path.

Scenario

- request unsupported projection shapes
- request unsupported view-shape/live combinations
- request unsupported policy/tenant/history combinations
- request unsupported subscription declaration, bridge lowering, basis binding,
  or activation combinations
- request unsupported temporal query basis, unbound host clock, raw timer, or
  time-only delivery fallback combinations
- request unsupported async resource family, stale completion fallback, or
  host-local loading/error state fallback combinations
- request unsupported mixed-cause delivery combinations that would require host
  event arrival order to decide result meaning
- request unsupported store-backed capabilities where runtime-backed execution
  would be semantically different

Must verify

- unsupported projection shapes do not widen to whole-entity reads
- unsupported view/live combinations do not degrade into misleading best-effort
  behavior
- unsupported policy/tenant/history combinations do not partially execute and
  redact later
- unsupported subscription declaration combinations do not degrade into raw
  CDC, host observer inference, generic subscription kinds, or direct
  activation
- unsupported temporal combinations do not degrade into ambient clocks, raw
  timers, raw signal wake events, or historical-basis mutation
- unsupported async combinations do not degrade into host-local loading/error
  state, stale completion acceptance, raw resource completion delivery, or
  transport-local retry policy
- unsupported mixed-cause combinations do not degrade into host-arrival-order
  semantics or raw event fanout
- unsupported store-backed capabilities do not silently fall to a semantically
  different path without explicit diagnostics

Required verification output

- `failure_digest`
- `counter_snapshot`
- `fallback_report`
- `forbidden_widening_zero_report`
- `forbidden_delivery_residue_zero_report`

Pass condition

Fallback is explicit, typed, diagnosable, and non-leaking.

### 16. Cross-Feature Composition Matrix Test

Purpose

Prove that the nastiest admitted cross-feature compositions remain canonical,
and that unsupported compositions fail explicitly instead of drifting.

Scenario

- run a curated adversarial composition matrix including rows like:
  - scope + template + saved-query reload
  - scope + policy mask + historical basis
  - lineage + correspondence + diff
  - preview-session basis + conflict inspection + merge intent lowering
  - inspector view + live promotion + aspect-focused projection
  - rollup + tenant schema variation
  - structured content + policy mask + live maintenance
  - CDC-shaped output + diff + branch basis
  - query-triggered writeback + policy mask + branch workflow basis
  - relationship-proof denial + saved-query artifact reload
  - subscription declaration + policy mask + grouped view
  - subscription declaration + saved-query exact reuse + tenant schema drift
  - subscription declaration + inspector view + relationship-proof denial
  - unsupported subscription declaration + durable restart request
  - temporal query basis + historical snapshot + time-only wake
  - temporal subscription + policy mask + rolling window
  - async resource query + retry + tenant remask
  - async completion + branch promotion + supersession
  - mixed truth/time/async delivery + preview discard
  - unsupported ambient clock + saved query reload
  - unsupported mixed-cause ordering + raw event fallback request

Must verify

- semantically equivalent rows produce the same canonical meaning
- intentionally different rows produce distinct digests
- out-of-support compositions fail typed and early

Required verification output

- `query_digest`
- `result_digest`
- `delivery_digest`
- `failure_digest`
- `composition_matrix_digest`

Pass condition

Cross-feature composition remains canonical where admitted and fail-closed
where not admitted.

### 17. Reference Semantics Test

Purpose

Prove that a bounded set of load-bearing admitted query families agrees with an
independent, deliberately slow, obviously correct semantic oracle rather than
only agreeing with the main planner/executor pipeline.

Scenario

- build a reference executor for a bounded admitted subset covering at least:
  - detail queries
  - collection queries with ordering and pagination
  - bounded traversal/materialization
  - policy-masked results
  - diff output for admitted shapes
  - live end-state convergence for admitted live families
  - temporal end-state convergence for admitted time-aware families
  - async resource state convergence for admitted resource families
  - mixed-cause final delivery convergence for admitted cause-ordering
    families
- compare canonical system results against the reference executor

Must verify

- planner and executor results agree with the independent semantic oracle
- result shapes match the oracle's declared semantics
- live end-state converges to the same truth as the oracle's re-executed end
  state
- diff and policy-masked results remain oracle-equivalent for the admitted set
- temporal, async, and mixed-cause end states agree with the oracle's canonical
  cause sequence rather than host event arrival order

Required verification output

- `query_digest`
- `result_digest`
- `reference_result_digest`
- `oracle_parity_report`
- `counter_snapshot`

Pass condition

The main query system agrees with the independent semantic oracle for the
bounded admitted subset.

### 18. Saved Artifact Semantic Freeze Test

Purpose

Prove that saved queries and related query artifacts retain canonical semantic
identity across reload, export/import, and admitted parameter rebinding.

Scenario

- create a saved query from direct construction
- reload it
- export/import it
- re-bind admitted parameters
- execute it across admissible bases and policy contexts

Must verify

- artifact reload preserves canonical query identity
- export/import preserves semantic meaning rather than "close enough" behavior
- admitted parameter rebinding changes only the semantics the bound context is
  supposed to change
- artifact identity changes when semantic meaning actually changes

Required verification output

- `query_digest`
- `replay_digest`
- `artifact_freeze_digest`
- `artifact_binding_matrix`
- `counter_snapshot`

Pass condition

Saved query artifacts remain semantically frozen unless an intentionally
meaning-changing rebinding occurs.

### 19. Schema Evolution Compatibility Test

Purpose

Prove that query artifacts remain legal and semantically stable under
compatible schema evolution, and fail typed and early under incompatible schema
evolution.

Scenario

- evolve schemas through compatible and incompatible changes
- execute ordinary queries, saved queries, templates, and scopes across those
  schema boundaries
- compare result-shape identity and artifact identity before and after
  evolution

Must verify

- compatible schema evolution preserves legal query meaning where it should
- incompatible schema evolution fails early and typed
- saved query/template/scope artifacts do not silently remap to new meaning
- result-shape evolution changes artifact identity when semantic meaning
  changed

Required verification output

- `query_digest`
- `failure_digest`
- `schema_compatibility_digest`
- `artifact_identity_drift_report`
- `counter_snapshot`

Pass condition

Schema evolution is compatibility-classified, semantically honest, and fail-
closed when meaning changes incompatibly.

### 20. Diagnostic Sufficiency Test

Purpose

Prove that canonical failure and drift bundles are not merely correct, but
sufficient to localize what failed and why without ambient debugging context.

Scenario

- run rejected or drifting cases covering:
  - legality failure
  - unsupported combination
  - policy denial
  - basis mismatch
  - temporal basis mismatch
  - async completion supersession
  - mixed-cause ordering denial
  - artifact portability failure
  - explicit fallback denial
- inspect only the emitted canonical bundles

Must verify

- bundles identify which clause failed
- bundles identify whether the failure class was legality, unsupported
  combination, policy denial, basis mismatch, or artifact portability
- bundles identify whether temporal, async, or mixed-cause failure came from
  query declaration, bridge lowering, signal/resource strategy, causality,
  ordering, support metadata, or certification coverage
- bundles identify whether fallback was considered and denied
- bundles identify which digest changed and why for drift cases

Required verification output

- `failure_digest`
- `diagnostics_sufficiency_report`
- `bundle_completeness_report`
- `counter_snapshot`

Pass condition

Rejected and drifting cases are mechanically localizable from canonical bundles
alone.

### 21. Beta Support Matrix Enforcement Test

Purpose

Prove that shipped beta surfaces, executable capability advertisement, support
metadata, and certification coverage stay in sync.

Scenario

- compare:
  - shipped beta support metadata
  - executable capability registry / admitted family registry
  - roadmap vision coverage appendix
  - named certification suite coverage
- include certified and non-certified query surfaces

Must verify

- every shipped beta surface maps to at least one certification row
- every non-certified surface is excluded from beta support metadata
- runtime capability advertisement matches actual admitted query families
- temporal/async support metadata excludes uncertified temporal basis,
  resource family, mixed-cause delivery, durable replay, and store-backed
  temporal replay claims
- documentation/support metadata and executable capability registry remain in
  sync

Required verification output

- `support_matrix_digest`
- `capability_registry_digest`
- `coverage_matrix_digest`
- `support_enforcement_report`

Pass condition

Beta support claims do not outrun certification or admitted runtime behavior.

## What These Tests Collectively Prove

Together, these tests prove that `worth-query` is:

- canonical about query meaning rather than builder-path dependent
- schema-aware before execution rather than repaired by runtime fallback
- snapshot- and basis-honest across runtime-backed and store-backed paths
- query-shaped across collection, live, diff, and delivery surfaces
- bridge-honest across query-owned subscription declaration and admission
  surfaces
- explicit about temporal query basis, time-only delivery, async resource
  causality, and mixed truth/time/async cause ordering
- incapable of accepting stale async completions, ambient clocks, raw timer
  events, or host-arrival-order delivery semantics as certified query behavior
- explicit about lineage, correspondence, policy, and tenant-boundary meaning
- durable and portable where it claims durable or portable artifact support
- explicit about admitted versus non-admitted query-family combinations
- incapable of silently widening, degrading, or advertising unsupported beta
  surfaces as certified support
- certifiable through canonical artifacts rather than by visual inspection

## Milestone Certification Rule

No query milestone should be considered closed until its named certification
suite emits canonical machine-checkable outputs and passes across:

- original execution
- an adversarial or hostile variation lane
- an independently produced equivalent or replay/resume lane where applicable

Without that, the query surface may still be promising, but it is not yet
trust-grade.

## Beta Support Rule
