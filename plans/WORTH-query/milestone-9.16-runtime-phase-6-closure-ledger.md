# Milestone 9.16 Runtime Phase 6 Closure Ledger

**Owner:** Runtime Hardening Track, Phase 6
**Canonical specification:** `milestone-9.16.md`
**Status:** Complete
**Policy:** A requirement is `PROVED` only when its production owner, public
consumer evidence, adversarial evidence, and residue posture agree. A finding
is `CLOSED` only when the root cause and every causally dependent guarantee
have been rechecked.

This ledger is the durable meaning of the `R6.*` and `Q6.*` identifiers used by
the milestone. Green broad tests alone do not change a row's status.

## Requirement ledger

| ID | Guarantee | Status | Current evidence and remaining closure |
|---|---|---|---|
| R6.1 | One canonical installed application-query identity covers parameters, scope, root paths and guards, result shape, cardinality, predicates, ordering, dependency ceiling, disclosure, basis, and lanes; descriptive digests remain separate from cryptographically rooted installed authority. | **PROVED** | Installation derives one fixed-width typed digest through the bounded Foundational SHA-256 slot and retains its complete canonical basis and work evidence. Query installation authority remains the separately keyed HMAC family; copied descriptive digest bytes open no authority path. |
| R6.2 | Application queries lower into the Milestone 9.10 canonical graph-read requirement chain. | **PROVED** | Guard fields and complex estate traversal lower through the canonical requirement, estimate, runtime-profile budget, inventory, and plan-review chain. |
| R6.3 | Runtime support review and budget admission happen before executable plan authority is minted. | **PROVED** | The execution-runtime installer owns the resource profile; request limits only narrow it, an undersized profile denies before plan authority, and the bank estate plan is admitted with exact budget evidence. |
| R6.4 | Non-live lanes share one bounded Query-owned execution kernel, typed projection, and governed receipt assembly. | **PROVED** | One-shot, continuation, historical, and preview use the shared application-query kernel. Projection retains installed identities and typed rows; the warm-path residue oracle and scale twins prove exact-zero execution and projection digest work. |
| R6.5 | Parameters, authenticated principal, typed scope, controls, and authorization are bound before execution authority. | **PROVED** | Parameters and typed preconditions are admitted under installed entry/encoded-byte ceilings. Runtime Bridge authorization consumes retained correspondence evidence, and the real preparation-through-retry oracle performs zero legacy digest calls after fresh admission. |
| R6.6 | Current, pinned, historical, and preview truth bases are concrete move-only authority with terminal release. | **PROVED** | Basis hostility, pin lifecycle, and terminal resource-baseline tests. |
| R6.7 | One canonical result meaning is preserved across every declared one-shot, continuation, historical, live, and preview lane. | **PROVED** | All declared lanes retain the installed query/result contract. Historical, preview, continuation, and 32-consumer live courtrooms pass; live delivery and every other warm phase report exact zero canonical work. |
| R6.8 | Mutation preconditions are typed at declaration, installed as an exact family/scope upper bound, bound into canonical attempt and idempotency intent, provider-recompared atomically at commit, and reported only from completed provider comparison evidence. | **PROVED** | The installed operation owns the six-entry and 256-KiB encoded-byte ceilings. Duplicate and oversized preconditions deny. Matching commit, relevant drift, unrelated drift, response-loss recovery, and exact retry retain provider-backed comparison evidence and exact-zero commit/retry/recovery digest work. |
| R6.9 | Plan, basis, result buffer, continuation, and live resources have non-wrapping lifecycle accounting and terminal release evidence. | **PROVED** | Resource-lifecycle and failure-path baselines. |
| R6.10 | Bank account, payment, estate, actor, and audit reads use installed domain queries with no local query authority. | **PROVED** | Account, payment, estate, actor-assignment, governance-boundary, and institution-audit families are installed application queries. Institution audit's local marker, authorization, projector, operation, outcome, and execution lane have been deleted. |
| R6.11 | Root limits, nested dependency work, and retained result-buffer bytes are distinct and fail closed. | **PROVED** | Nested-result, variable-width buffer, result-limit, and work-limit evidence. |
| R6.12 | Phase 6 cutover has zero legacy markers, projectors, filtering/sorting/pagination, repeated child reads, and warm-path canonicalization. | **PROVED** | The exact transitive warm source cone rejects private SHA, legacy digest helpers, canonical-basis preparation, and digest rendering. A dynamic lower-layer oracle additionally proves the real mutation path performs zero legacy digest calls through preparation, commit, response-loss recovery, and retry. |
| R6.13 | Receipts report exact planning, fallback, scans, lookups, ordering, work, and buffer evidence; covered paths report zero fallback and zero per-result neighbor lookup. | **PROVED** | Existing structural receipt evidence remains exact, and receipts now carry nine canonical-work phases. Independent result/graph, candidate, policy-fact, and live-consumer twins keep warm canonical work exact zero. |
| R6.14 | Every Phase 6 semantic digest is derived only at installation or bounded fresh admission, under exact entry and encoded-byte budgets owned by the governing installed contract. | **PROVED** | Foundational owns bounded canonical material encoding and SHA-256 derivation. Query installation and fresh admission supply exact entry/byte ceilings; execution, projection, commit, retry, live delivery, recovery, and publication retain typed results without rehashing. |
| R6.15 | Installation, admission, execution, provider commit, projection, live delivery, retry resolution, recovery inspection, and publication expose phase-separated basis preparation, canonical encoding/allocation, digest derivation, SHA-256 compression, and digest-text materialization evidence. | **PROVED** | `WorthQueryCanonicalWorkPhases` exposes all nine named stages. Each stage separates basis entries, encoded bytes, canonical allocation, SHA input, SHA compression, derivations, and digest-text materialization; warm stages are compile-fixed to exact zero. |
| R6.16 | Increasing roots, edges, candidates, result rows, projected fields, authorization facts, and live consumers while semantic identities remain fixed does not increase canonical or digest work; ordinary fan-out lanes remain exact zero. | **PROVED** | Three ordinary scale-axis twins independently grow result/graph, guarded-candidate, and policy-fact fan-out; a 32-consumer live twin grows delivery fan-out. A scheduled 512-query/32,768-row probe exercises sustained real operations. Every warm stage remains exact zero. |

## Finding ledger

Finding IDs before Q6.43 were transient review identifiers and are not cited
as durable evidence. New findings are appended here and reopen the earliest
affected requirement plus every causal dependent.

| ID | Impact | Finding | Status | Closure evidence |
|---|---|---|---|---|
| Q6.43 | Medium | The application-query execution-count assertion became stale and could conceal unexecuted coverage. | **CLOSED** | The execution suite count was corrected and the full application-query execution suite rerun. |
| Q6.44 | High | Proof-buffer bytes were incorrectly charged against the inline index subtotal, denying a lawful payment-detail query. | **CLOSED** | Memory accounting now keeps index, proof, result, and total bytes distinct; exact-budget and one-byte-under tests plus the real payment consumer pass. |
| Q6.45 | Critical | Bank account discovery retained host-local operation authority and needed a generic typed union of bounded principal-to-account paths. | **CLOSED** | The strengthened guarded-path substrate preserves the original union/dedup behavior; the full ordinary-read suite, dynamic work test, inventories, and residue oracle pass. |
| Q6.46 | Critical | Pending-payments migration requires fixed typed equality guards on an intermediate authorization role and terminal payment status; plain root paths would over-authorize viewers or force forbidden host filtering. | **CLOSED** | Guards are typed and identity-bearing, schema-closed, planned as exact predicate fields, batch-executed at current or pinned truth, work-accounted, and proved by approver/viewer/status/result/work hostility. The legacy operation/projector/admission lane is absent. |
| Q6.47 | Medium | The first bounded guard batch reacquired the authoritative state handle once per frontier entity, adding avoidable hot-path overhead inside an otherwise bounded API. | **CLOSED** | The batch captures one authoritative state view and materializes every candidate against that view; Relational and consumer evidence were rerun. |
| Q6.48 | Low | The ledger omitted root paths and guards from R6.1 and routed Q6.45 audits to the declaration file that existed before the root-selection directory split. | **CLOSED** | R6.1 now names the complete identity surface and the Q6.45 ownership list names the current source files. |
| Q6.49 | Medium | Indexed root execution trusted an optional equality-index identifier with a warm-path panic even though the public denial lattice already represented missing predicate support. | **CLOSED** | Indexed root selection now maps the absent identifier to `PredicateIndexUnavailable`; the focused root-selection suite and full execution suite pass. |
| Q6.50 | High | Application-query admission always used one fixed inline-index budget. The honest Phase 6.7 estate plan exceeded it, while no execution-runtime installation surface could admit a larger bounded operating-world profile; raising caller work/results could not lawfully help. | **CLOSED** | A typed installer-owned profile independently bounds index, per-root result, and intermediate capacity. The runtime retains it, requests only narrow it, one-byte capacity denies despite broad caller work, a larger profile admits the same query identity, and the estate receipt proves its estimate exceeds the old default. |
| Q6.51 | Critical | Estate graph bootstrap inferred `EstateAssignment` by selecting the first case sharing an employee assignment's branch and fabricated an ID-zero target when no case matched. Multiple estates in one branch could therefore publish false case authority. | **CLOSED** | `BankEstateWorld` now owns an explicit `(EstateCaseId, EmployeeAssignmentId)` relation. The split graph adapter publishes only declared pairs; branch equality mints no assignment edge. The specialist and executor consumer paths both pass through installed authorization. |
| Q6.52 | Critical | A preview declaration pinned an exact snapshot, but `BridgeSpeculativeComparison` reconstructed a branch-head request when execution began. A preview could therefore drift after it was opened. | **CLOSED** | The comparison retains and reuses the exact declared truth-view selector. Consumer evidence opens an estate preview, advances main, and proves preview returns the old value while current returns the new value under the same canonical query identity. |
| Q6.53 | High | The Query preview facade accepted a caller-constructed Bridge request and exposed Bridge session identity, residue, and outcome types, allowing infrastructure authority to leak across the application-query boundary. | **CLOSED** | Query now derives the primary branch head internally, mints an opaque Query-owned session identity, lowers it privately to Bridge, owns discard and release receipts, and exposes typed Query denials. |
| Q6.54 | High | Bridge preview validation did not prove that the session-basis selector branch matched the speculative branch binding. | **CLOSED** | Declaration validation rejects mismatched branch identities before preview authority is minted; a hostile cross-branch test proves the denial. |
| Q6.55 | High | Institution audit remained an operation-shaped local marker, authorization, projector, and outcome lane rather than an installed application query. | **CLOSED** | Institution audit is a schema-installed application query with typed scope, ability, graph shape, ordering, resource ceiling, governed receipt, and public consumer evidence. The entire legacy lane and its exports are absent. |
| Q6.56 | Medium | Preview session identity allocation used wrapping atomic increment and could eventually reuse a supposedly unique authority identity. | **CLOSED** | Allocation uses checked atomic update, returns a typed exhaustion denial, and has a non-wrapping boundary test. |
| Q6.57 | Medium | Frozen schema and ability inventories omitted the estate-view ability, employee-assignment identity field, and installed application-query inventory. | **CLOSED** | The inventories now freeze those declarations and all ten installed bank application-query names; the schema and ability inventory suites pass. |
| Q6.58 | Low | Unit tests still consumed the deleted `bind_bank_world` compatibility wrapper, concealing incomplete cutover. | **CLOSED** | Tests use the canonical estate-aware binding entry point directly and the old wrapper has zero repository matches. |
| Q6.59 | Medium | The bank query execution helper required seven independent semantic arguments, making scope identity, parameters, and controls easy to misbind at every consumer. | **CLOSED** | `BankApplicationQueryInvocation` packages the typed query request, scope, parameters, and controls; current and preview execution consume that single invocation. |
| Q6.60 | High | The initial R6.8 evidence plan grouped declaration, installation, binding, provider comparison, idempotency, receipt provenance, and consequential state into one happy-path claim, allowing missing negative-layer proof to hide behind a green bank race. | **CLOSED** | Declaration, installation, binding, provider comparison, idempotency, receipt provenance, and consequential state now have distinct negative evidence; the exact causal chain is recorded below and in R6.8. |
| Q6.61 | Critical | Mutation-precondition identity was included in the operation-scope fingerprint used to derive the idempotency-key identity. Reusing one client key with a changed expectation could therefore address a new provider key and commit the money movement twice. | **CLOSED** | Operation scope excludes mutable expectations; key identity is stable for the same client key while normalized intent changes. Exact retry returns the original receipt, changed intent is denied, and the bank balance oracle proves one movement. |
| Q6.62 | High | Provider-recomparison evidence was constructed before the provider performed its final atomic comparison, allowing a receipt-shaped certificate to precede the event it claimed. | **CLOSED** | Recomparison evidence is minted only from committed comparison or authoritative receipt recovery. Relevant drift denies, unrelated drift commits, and response-loss recovery returns the original provider-backed receipt. |
| Q6.63 | Medium | The first relevant-drift race could fail through the ordinary read set even if expected values were ignored, so it did not independently prove typed precondition-value enforcement. | **CLOSED** | A request with the correct entity version but wrong typed `AccountStatus` denies and an independent balance read proves no mutation; matching, unrelated-drift, and retry twins pass. |
| Q6.64 | High | Query prepared some Foundational value or basis material but then owned private SHA-256 grammars and raw digest derivation for cross-crate semantic identity. Foundational's existing digest front door was overlooked, but its only admitted implementation is explicitly fixture-grade and therefore cannot simply replace the operational identities. | **CLOSED** | Foundational now owns the production SHA-256-only bounded digest slot, exact entry/encoded-byte denial, and separated canonical-allocation/SHA work evidence. Query semantic families consume that slot only at installation or bounded fresh admission. |
| Q6.65 | Critical | Installed package "authority nonces" were deterministic hashes of public runtime, generation, package, and admission meaning. Child authority identities used secret-prefix SHA-256, compared MAC-like text with ordinary equality, encoded absent ability policy as a collidable sentinel string, and retained raw secret arrays in derived-`Debug` authority artifacts. | **CLOSED** | The fallible installed-index build now obtains an OS-random redacted root key, derives generation/package keys and family seals with typed domain-separated HMAC-SHA-256, verifies seals in constant time, and zeroizes secret key storage on drop. Exact rebuilds retain lineage, successors rotate package keys, and independently rooted same-ordinal indexes are foreign. RFC-vector, entropy-denial, redaction, wrong-key, cross-family, changed-field, option-collision, runtime/Bridge/bank consumer, strict Clippy, line-cap, and residue evidence pass. |
| Q6.66 | Medium | The Authentik callback extracted CSRF secrets as strings and compared them normally, bypassing the dependency's enabled timing-resistant secret comparison contract. | **CLOSED** | Callback admission compares `CsrfToken` values directly with `timing-resistant-secret-traits` enabled. Protocol residue freezes the protected comparison, malformed callbacks remain typed denials, and the real Authentik adapter compiles under strict all-target Clippy. |
| Q6.67 | Critical | Ordinary graph projection hashes every returned row and chunk into stream evidence, so result fan-out directly increases SHA-256 work and creates the exact per-result hashing posture forbidden for future high-fan-out kernels. | **CLOSED** | Ordinary application-query projection retains installed call/result authority and typed rows without content hashing. Result and graph fan-out change structural counters while all warm canonical-work fields remain exact zero. |
| Q6.68 | High | Foundational materializes an unbounded canonical `String` only after digest readiness, while Phase 6 installation/admission callers have no exact encoded-byte denial or allocation evidence. | **CLOSED** | Bounded encoding checks entry count and encoded bytes before work escapes and reports canonical allocation separately from SHA input and compression. Oversized Query precondition material denies under the installed byte ceiling. |
| Q6.69 | High | The ledger declared Phase 6 closed without representing the phase-separated canonical-work and independent fan-out guarantees added to the governing specification. | **CLOSED** | R6.14-R6.16 now carry the contract and this final audit rechecked every causally reopened row against source, targeted hostility, broad suites, and real consumer scale evidence. |
| Q6.70 | High | Canonical-work receipts initially collapsed provider commit, projection, live delivery, retry resolution, and recovery inspection into broader phases, allowing unmeasured warm work to hide behind an execution zero. | **CLOSED** | The phase evidence type now names all nine lifecycle stages, and bank read, mutation, retry, recovery, and live tests assert each relevant stage independently. |
| Q6.71 | Critical | The original warm residue cone stopped at application code. A real bank commit crossed operation binding and generic provider-session helpers that performed 34 hidden execution-time hashes while the public receipt reported zero. | **CLOSED** | The transitive authority/session/decision/provisional/invariant chain now carries retained semantic identities and checked occurrence labels without hashing. The static oracle covers those files, and the dynamic real-path oracle reports exactly zero calls through preparation, commit, response-loss recovery, and retry. |
| Q6.72 | Medium | The first 512-query speed proof ran in the ordinary edit-loop suite and added roughly 22 seconds to every targeted test rerun. | **CLOSED** | The sustained probe is an explicit scheduled ignored test. The three active scale twins finish in under a second, while the heavy probe is run deliberately with `--ignored --nocapture`. |
| Q6.73 | Critical | Replacing runtime hashes by reusing upstream text initially collapsed distinct attempts, sessions, plans, and overlays onto the same identity, weakening substitution and readmission authority. Several lifecycle counters also used wrapping increment. | **CLOSED** | Execution attempts and managed runs regain checked O(1) occurrence identities; provider-session identity is minted from its checked generation while physical identity remains separate; session binding retains the unique resource attempt. All substitution, interleaved-session, provisional, and 31 readmission tests pass, and boundary tests prove counters do not wrap. |

## Q6.50 exact evidence

Production ownership:

- `worth-query-execution/execution_runtime/application_query_resources.rs`
- `worth-query-execution/execution_runtime/runtime_root.rs`
- `worth-query-execution/primary_graph/application_query/admission.rs`
- `bank-server/identity_runtime.rs`

Public and adversarial evidence:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution application_query::planning_budget

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution execution_runtime::tests

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_reads estate
```

## Q6.45 exact evidence

Production ownership:

- `worth-query-declaration/application_query/root_selection/path.rs`
- `worth-query-installation/application_query/root_selection.rs`
- `worth-query-execution/.../read_execution/root_selection/path_union.rs`
- `bank-domain/queries/account_discovery.rs`

Public and adversarial evidence:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution application_query::root_selection

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_reads

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_mutations \
  public_consumer_executes_every_typed_mutation_family

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-domain --test ability_inventory

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-domain --test schema_inventory
```

Residue oracle:

```text
rg "DiscoverAccountsOperation|project_account_discovery_read|\
authorize_account_discovery|struct AccountDiscovery\b|\
queries::AccountDiscovery\b" workspaces/worth-query-bank-world/crates
```

Expected result: no matches.

## Q6.46 exact evidence

Production ownership:

- `worth-query-declaration/application_query/root_selection/guard.rs`
- `worth-query-installation/application_query/root_selection.rs`
- `worth-query-execution/.../read_execution/root_selection/path_union.rs`
- `worth-relational/.../reader/truth_frontier_field_equality.rs`
- `bank-domain/queries/pending_payments.rs`

Public and adversarial evidence:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution application_query::root_selection

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution root_path_guard_reads_its_pinned_truth_version

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_reads

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_mutations \
  public_consumer_executes_every_typed_mutation_family -- --exact

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-domain --test ability_inventory

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-domain --test schema_inventory
```

Residue oracle:

```text
rg "ReadPendingPayments|project_pending_payments_read|\
authorize_pending_payments|struct PendingPayments\b" \
workspaces/worth-query-bank-world/crates
```

Expected result: no matches.

## Q6.52-Q6.59 exact evidence

Production ownership:

- `worth-runtime-bridge/facade/standard_path.rs`
- `worth-runtime-bridge/speculation/validation.rs`
- `worth-query-execution/primary_graph/application_branch.rs`
- `worth-query-execution/primary_graph/application_query/basis/preview_authority.rs`
- `worth-query-execution/primary_graph/application_query/basis/preview_session_open.rs`
- `bank-domain/queries/institution_audit/`
- `bank-server/application_query/request.rs`
- `bank-server/application_query/execution.rs`

Public, hostile, and inventory evidence:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --lib --test ordinary_reads

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-domain --test schema_inventory
```

Residue oracles:

```text
rg "ReadInstitutionAudit|BankReadOutcome|bind_bank_world\b" \
  workspaces/worth-query-bank-world/crates

rg "BridgeSpeculativeSessionRequest|BridgeSpeculativeSessionIdentity" \
  workspaces/worth-query-bank-world/crates
```

Expected result: no matches.

## Q6.60-Q6.64 exact evidence

Production ownership:

- `worth-query-declaration/application_schema/mutation_precondition.rs`
- `worth-query-installation/application_operation/precondition_contract.rs`
- `worth-query-execution/primary_graph/application_attempt/precondition_binding/`
- `worth-query-execution/primary_graph/authorization/scope_identity.rs`
- `worth-query-execution/primary_graph/application_attempt/idempotency/`
- `worth-query-execution/primary_graph/application_attempt/provider_execution/`
- `worth-query-execution/primary_graph/application_attempt/provider_recomparison.rs`
- `worth-query-admission/authenticated_principal/adapter_identity.rs`
- `worth-foundational/canonicalization/digest_slots/`
- `worth-query/evidence_identity/foundational.rs`

Layered precondition and consumer evidence:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-certification --test mutation_precondition_compile_fail

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution \
  domain_computation::primary_graph::tests::application_attempt

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_mutations

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-domain proposals::idempotency_tests
```

Expected result: two compile denials, 33 execution courtroom passes, three
public bank mutation passes, and four idempotency identity passes.

Foundational slot and semantic-family evidence:

```text
cargo test -p worth-foundational sha256
cargo test -p worth-foundational \
  algorithm_slot_admission_denies_unsupported_version_domain_and_algorithm
cargo test -p worth-foundational --test compile_time_boundaries \
  canonical_digest_derivation_requires_admitted_input_shape

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-installation \
  phase_six_semantic_families_share_the_foundational_digest_slot

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-admission authenticated_principal::tests

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query domain_capabilities::canonical_runtime
```

The application-query declaration, installation, parameter, precondition, and
schema-identity bases are prepared only at declaration, installation, or fresh
admission. Execution, provider comparison, retry resolution, projection, live
delivery, and recovery carry their typed digests and perform no basis
preparation or digest derivation. Graph-read requirement, cost, inventory, and
plan-review digests remain descriptive summaries alongside the complete typed
rows; admission decisions inspect the rows, budgets, and support postures, and
copied digest text mints no plan authority. Legacy operation-aftermath product
contracts remain owned by Runtime Phase 8 and cannot satisfy Phase 6
application-query, precondition, or installed-authority proofs.

## Q6.64-Q6.73 bounded canonical-work closure

Production ownership:

- `worth-foundational/canonicalization/digest_slots/`
- `worth-query-installation/canonical_digest_derivation.rs`
- `worth-query-installation/canonical_work.rs`
- `worth-query-installation/application_query/tests/canonical_basis_residue.rs`
- `worth-query-execution/execution_digest.rs`
- `worth-query-execution/primary_graph/application_attempt/`
- `worth-query-execution/provider_session/`
- `worth-query-execution/managed_run/run_identity.rs`
- `bank-server/tests/ordinary_reads/canonical_work_scale.rs`
- `bank-server/tests/ordinary_mutations/preconditions.rs`
- `bank-server/tests/live_activity.rs`

Exact digest, budget, and authority evidence:

```text
cargo test -p worth-foundational -p worth-runtime-bridge

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-declaration -p worth-query-installation \
  -p worth-query-admission -p worth-query-publication

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution \
  preparation_commit_recovery_and_retry_perform_no_execution_digest_derivation

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-installation \
  phase_six_warm_consumers_cannot_hide_hashing_behind_a_helper

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-domain -p bank-server -p bank-http-adapter
```

The full execution suite proves 378 runtime tests and 12 compile-fail doctests.
The focused readmission family proves 31 direct/workflow recovery and
substitution cases. The installed precondition contract proves a six-entry and
256-KiB encoded-byte ceiling, including duplicate-entry and oversized-value
denials.

Scale and speed evidence:

```text
cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_reads canonical_work_scale::

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test live_activity \
  live_consumer_fanout_keeps_each_delivery_free_of_canonical_work

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_reads \
  high_operation_speed_probe_keeps_warm_digest_work_at_exact_zero \
  -- --ignored --nocapture
```

The active scale gate runs three independent fan-out twins in under one second.
The explicit debug-profile probe executes 512 real queries returning 32,768
rows in 25.66 seconds (19.95 queries/second and 1,277 rows/second on the audited
machine). This is observational performance evidence, not a portable timing
threshold. Every query asserts exact-zero execution, provider-commit,
projection, live-delivery, retry-resolution, recovery-inspection, and
publication canonical work.

There is no geometry implementation in this milestone. The real bank
application is the current high-fan-out surrogate. The Milestone 9.19 handoff
therefore remains a prohibition and admission contract: future geometry may
consume the installed typed identities proven here, but ordinary kernels may
not hash per cell, feature, node, edge, candidate, projected field, or result.

## Q6.65 exact evidence

Production ownership:

- `worth-query-installation/authority_cryptography.rs`
- `worth-query-installation/installed_index/construction.rs`
- `worth-query-installation/installed_index/relation.rs`
- `worth-query-installation/installed_index/authority_validation.rs`
- `worth-query-installation/application_query/authority_seal.rs`
- `worth-query-installation/application_operation/installed.rs`
- `worth-query-installation/application_ability/installed_contract.rs`
- `worth-query-installation/application_principal_binding/installed_contract.rs`
- `worth-query-installation/installed_domain_operation.rs`
- `worth-runtime-bridge/correspondence/semantic_dependency_candidate.rs`

Cryptographic and authority-lineage evidence:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-installation authority_

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-installation --lib

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-execution --lib execution_runtime::tests

cargo test --manifest-path crates/worth-runtime-bridge/Cargo.toml \
  --lib correspondence

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_reads

cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml \
  -p bank-server --test ordinary_mutations
```

Strict changed-crate lint:

```text
cargo clippy --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-installation --all-targets -- -D warnings

cargo clippy --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-runtime-bridge --lib --no-deps -- -D warnings
```

Residue oracle:

```text
rg "authority_nonce|package_authority_nonce|authority_key:\s*\[u8|\
hash\.update\([^\r\n]*(nonce|authority_key)" \
  workspaces/worth-query/crates/worth-query-installation/src

rg "Sha256|sha2::Digest" \
  workspaces/worth-query/crates/worth-query-installation/src/application_ability/installed_contract.rs \
  workspaces/worth-query/crates/worth-query-installation/src/application_operation/installed.rs \
  workspaces/worth-query/crates/worth-query-installation/src/application_principal_binding/installed_contract.rs \
  workspaces/worth-query/crates/worth-query-installation/src/application_query/authority_seal.rs \
  workspaces/worth-query/crates/worth-query-installation/src/installed_domain_operation.rs
```

Expected result: no matches.

Remaining direct SHA owners are classified rather than hidden. Installed graph
participation hashes a descriptive affinity identity retained inside an opaque
recipe that also owns the exact provider `Arc`; every governed call still
requires that authority object. The application operation-scope fingerprint is
documented descriptive input retained inside the private move-only admitted
operation proof; copied bytes mint no admission. Cross-crate semantic digest
ownership remains separately open under Q6.64.

## Phase 6.9 certification closure

- strict all-target Clippy passes for Query declaration, installation,
  admission, execution, publication dependencies, the monolith, bank domain,
  bank server, and the Authentik HTTP adapter;
- `git diff --check` passes;
- no dirty non-allowlisted Rust file exceeds 400 lines;
- boundary-check passes after explicit snapshot regeneration for the two typed
  precondition macros;
- generated agent context is current;
- public bank read, mutation, schema, estate, and transcript suites pass; and
- the deleted bank-local query-lane oracle has no matches.

## Reopening rule

A change to root-path meaning, schema closure, canonical identity, planning
rows, cost dimensions, basis selection, traversal work accounting, ordering,
result limits, receipts, bank relations, or discovery authorization reopens
Q6.45 and R6.1-R6.6, R6.10-R6.13 as causally applicable. A change to payment
memory dimensions reopens Q6.44, R6.2, R6.3, R6.11, and R6.13. A change to
equality-index availability or indexed-root execution reopens Q6.49, R6.3,
R6.4, and R6.13. A change to execution-runtime query resource profiles,
budget intersection, or plan-review resource evidence reopens Q6.50, R6.2,
R6.3, R6.7, R6.11, and R6.13.
A change to preview selector retention, branch binding, Query-owned session
authority, or terminal discard reopens Q6.52-Q6.54, Q6.56, R6.6, R6.7, R6.9,
and R6.13. A change to institution audit or the bank invocation boundary
reopens Q6.55, Q6.57-Q6.59, R6.5, R6.10, R6.12, and R6.13.
A change to Foundational canonical encoding, Query digest-slot use,
canonical-work phase evidence, execution/session occurrence identity, or any
warm-path helper in the scanned transitive cone reopens Q6.64 and Q6.67-Q6.73
plus R6.1, R6.4, R6.5, R6.7, R6.8, and R6.12-R6.16 as causally applicable.
