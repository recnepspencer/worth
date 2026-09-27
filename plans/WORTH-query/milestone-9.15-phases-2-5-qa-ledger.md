# Milestone 9.15 Phases 2-5 QA Closure Ledger

## Audit state

- Status: **CLOSED — PHASES 2-5 PROVED**
- Worktree branch: `worth-query`
- Audit starting commit: `ba210a31`
- Governing specification: `plans/WORTH-query/milestone-9.15.md`
- Governing laws: every document under `docs/coding-guidelines/`
- QA methods: `qa-loop`, `qa-tests`, and `code-quality-qa`
- Starting worktree: 305 dirty entries (140 modified, 58 deleted, 107
  untracked)

This ledger defines closure. Existing green tests are candidate evidence only
until their setup, authority path, oracle, consequential state, and fault
sensitivity have been inspected against the corresponding row.

## Status vocabulary

- `OPEN`: not yet proved under this ledger.
- `PROVED`: current source and independent evidence establish the claim.
- `DEFECT`: current evidence contradicts the claim.
- `BLOCKED`: proof requires an unavailable authority or external state.
- `N/A`: the claim is inapplicable, with a recorded governing reason.

A row becomes `PROVED` only when it records the production source trace, a
positive witness, a hostile twin, an independent oracle, consequential state,
fault sensitivity, and the exact verification command appropriate to the
claim.

## Phase 2: managed artifact ownership and workflow carriage

| ID | Status | Closure claim | Result and evidence |
| --- | --- | --- | --- |
| P2.1 | PROVED | Only installed contract -> production admission -> runtime owner -> move-only handle can create operational artifact authority. | The execution package keeps construction private and the certification suite rejects direct registration, direct production authority, forged authority parts, and handle cloning. `cargo test -p worth-query-execution` and compile certification pass. |
| P2.2 | PROVED | Runtime, generation, operation, run, stage, basis, family/version, payload owner, and lifecycle generation are exact affinity dimensions. | `authority_affinity_tests.rs` constructs real installed packages and opaque contract authorities, varies every affinity dimension independently, and asserts the exact denial. The former boolean matrix remains only as a totality test for denial precedence. |
| P2.3 | PROVED | Production requires the exact active stage execution, and one resource attempt cannot amplify into multiple live artifact runs. | Production admission is retained through the active stage; the live-run registry uses exact attempt identity plus `Weak` ownership and denies a second live artifact run until close/drop. |
| P2.4 | PROVED | Stage carriage consumes the predecessor handle unless the installed contract admits the exact borrow or lease. | `stage_admission.rs` now produces a private `WorthQueryAdmittedWorkflowStage` only after readiness, predecessor-edge, and contract checks. Transfer/lease mutation is downstream of that proof; the hostile completed-consumer retry test observes no second lease mutation. |
| P2.5 | PROVED | Move, borrow, lease, replacement, cancellation, and disposal leave exact owner/lease counts and destroy provider payload exactly once. | Lifecycle transition tests observe provider destruction and exact owner/lease state across movement, replacement, cancellation, and disposal. The package and installed artifact suites pass. |
| P2.6 | PROVED | Panic, denial, cancellation, and failure at every handoff preserve the declared recovery posture without orphan resources. | Artifact lifecycle and workflow tests exercise panic/failure/cancellation/drop with provider-observed destruction and no remaining live owner, except the pre-admission mutation defect isolated in P2.4/F01. |
| P2.7 | PROVED | Trace and replay carry semantic projection/disposition without operational payload or original authority. | Trace/replay types retain semantic projection and disposition only; compile tests deny authority reconstruction and ordinary lanes do not import replay authority. |
| P2.8 | PROVED | Query owns registration/lifecycle; providers own allocation/destruction; no public downcast, global bag, or invisible consumer resource exists. | Registration and lifecycle live in execution; allocation/destruction remain provider calls. Source/export search found no public payload downcast or global resource bag. |

## Phase 3: bulk and chunked native artifact access

| ID | Status | Closure claim | Result and evidence |
| --- | --- | --- | --- |
| P3.1 | PROVED | One access admission binds handle, layout, basis, affinity, borrow generation, fields, bounds, alignment, lifetime, and provider session. | Native admission now binds a typed `WorthQueryArtifactNativeAccessBound` for row batch, field slice, chunk, projection, or scalar fallback. Every access evidence value exposes the exact admitted bound and installed tests assert it for every lane. |
| P3.2 | PROVED | Borrowed rows, field slices, and chunk views cannot escape lifetime or cross thread/session/provider boundaries. | Runtime mismatch tests reject foreign provider/session/layout/generation; trybuild rejects escaping chunk views and cross-thread cursors. |
| P3.3 | PROVED | Bulk row, field-slice, projection, and admitted scalar lanes preserve identical semantic values and basis evidence. | The native-access matrix compares all lanes against independently mounted foundational values and exact basis evidence. |
| P3.4 | PROVED | Chunking is physically bounded and incremental rather than eager full materialization followed by paging. | First-chunk provider contacts and physical allocation observations scale with chunk width rather than total result size. |
| P3.5 | PROVED | Opaque/reference/content families use declared native projection or deny; generic cloned-row widening is impossible. | Family parity/rejection tests cover declared native mappings and deny undeclared/foreign selection without a generic cloned-row lane. |
| P3.6 | PROVED | Scalar fallback exists only when explicitly admitted with visible complexity and call amplification. | Scalar access requires the installed scalar posture and records provider call amplification; unsupported fallback is denied. |
| P3.7 | PROVED | Chunk width controls independently measured peak scratch/result memory. | `projection_sink/tests.rs` uses `stats_alloc` in an isolated subprocess and proves independently measured live-allocation peak changes with chunk width, including nested variable-width values. |
| P3.8 | PROVED | Physical counters distinguish lanes and respond only to causally relevant scale axes. | `artifact_native_access_causality.rs` executes real installed bulk, field, chunk, scalar, and narrow/wide projection worlds while holding payload work constant. It asserts exact changes only in the causally relevant contact, width, chunk, value, and call counters. |

## Phase 4: installed structural and decision evidence

| ID | Status | Closure claim | Result and evidence |
| --- | --- | --- | --- |
| P4.1 | PROVED | Equivalent schemas converge and every semantic drift dimension conflicts atomically. | Installation conflict tests vary unit, aggregation, requiredness, replay, classification, and retention and assert clean index residue after denial. |
| P4.2 | PROVED | Counter aggregation rejects unknown sources, cycles, incompatible units, incompatible scope/reset boundaries, and invalid requiredness. | Structural-counter validation and its hostile matrix cover every named relationship before installation commit. |
| P4.3 | PROVED | Receipt admission enforces required rows, uniqueness, monotonicity, aggregation relationships, and declared schemas. | The receipt denial suite now attacks undeclared and duplicate counters, missing provider certification, and missing, duplicate, and undeclared decision summaries. Every case asserts the exact denial and zero later publication/consumption. |
| P4.4 | PROVED | Mandatory interpretive/cost core survives every policy; optional material has an exact typed sidecar state. | Publication preserves mandatory counter/decision summaries while sidecars encode materialized or digest-only state; inspection/cert copies retain the core. |
| P4.5 | PROVED | Each decision schema's classification and retention constrain raw records and every derived copy, including mixed-schema sidecars. | Decision admission now supports `PartiallyMaterialized`: eligible same-retention records remain materialized, shorter-lived raw records are omitted, and the digest covers the complete record set. Inspection copies downgrade partial material to digest-only; mixed-governance tests prove both sides. |
| P4.6 | PROVED | Candidate summaries cannot overstate completeness, feasibility, optimality, termination, or incumbent posture. | Candidate validation rejects dishonest posture combinations and does not infer claims from omitted optional evidence. |
| P4.7 | PROVED | Transformation/loss summaries retain mandatory posture without granting repair, admission, identity, or publication authority. | Transformation/loss material is descriptive; no authority constructor consumes it, and compile certification rejects domain evidence as authority. |
| P4.8 | PROVED | Counters, records, summaries, digests, and free-form material cannot mint authority. | Public-door inventory plus compile tests reject evidence authority construction, private binding parts, and admitted-evidence cloning. |

## Phase 5: execution resource request and admission

| ID | Status | Closure claim | Result and evidence |
| --- | --- | --- | --- |
| P5.1 | PROVED | Request, contract, support, plan, and outcome represent every required independent resource dimension without collapsing memory or semantic axes. | Field-by-field trace accounts for 12 semantic axes and 19 resource dimensions through declaration, installation, support, admission, and outcome. |
| P5.2 | PROVED | Only the installed operation and exact provider support can lower an admitted plan; reconstructed or incomplete support opens no door. | Contract and admission identities use SHA-256 over length-framed canonical fields, identifiers reject trimmed/control input, and collision twins are hostile-tested. Raw lowering/reservation is root-internal; execution reaches it only after exact installed binding. |
| P5.3 | PROVED | Rejection precedes allocation, reservation, session mint, graph traversal, provider contact, and domain computation. | Rejection tests observe zero reservation/session/provider/graph/domain work for unsupported envelopes and postures. |
| P5.4 | PROVED | Provider or safe-point mismatch cannot fall through to a generic provider or weaker strategy. | Provider, access family, allocator family, and safe-point mismatches return typed denial without fallback. |
| P5.5 | PROVED | Strategy and envelope are immutable per attempt; any semantic change requires a newly admitted attempt identity. | Exact binding validation retains the admitted plan, and changed resource requests produce new plan and provider-session identities. |
| P5.6 | PROVED | Capacity reservation is physical, exact, atomic across subjects, deduplicated only by exact authority, and released once. | `capacity_tests.rs` proves rollback when a later subject fails and races eight threads against one slot, observing exactly one live holder. Drop releases occupancy to zero and permits the next exact reservation. |
| P5.7 | PROVED | Arrival pressure reaches typed reject/backpressure/degradation before bounds are exceeded. | A held live attempt exhausts fixed capacity and a concurrent-in-lifetime arrival receives typed saturation before session/provider work. |
| P5.8 | PROVED | Independent scale axes affect only causally relevant decisions and counters. | The admission causality matrix varies 12 semantic axes and 19 resource dimensions one at a time, asserts the exact decision/counter movement, and observes zero later work on denial. |
| P5.9 | PROVED | Async requirement, degradation, partial-effect, retained-progress, and yielded posture remain typed admission decisions without silent fallback. | The posture matrix now covers async, degradation, partial effect, retained progress, partial result, and yielded progress, including explicit-consent hostile twins and exact typed denials. |
| P5.10 | PROVED | No public plan, reservation, session, graph-call, or attempt door bypasses admission and the strongest predecessor proof. | Public reserve methods were removed from admitted plans; raw lowering/reservation moved to root-internal integration and execution calls it only after installed binding. Trybuild rejects direct plan reservation and host access to internal integration. |

## Cross-phase gates

| ID | Status | Closure claim | Result and evidence |
| --- | --- | --- | --- |
| X1 | PROVED | Phase progression is exact; later phases accept only the strongest predecessor proof and never reconstruct earlier authority. | The reopened P2 and P5 paths now require private typed stage admission and exact installed-operation binding before mutation or reservation. Runtime and compile attacks prove weaker/reconstructed inputs open no door. |
| X2 | PROVED | Destination topology realizes declaration, installation, admission, execution, publication, and certification ownership without flattening. | The five internal packages exist; the monolith is an integration package; `worth-query-host` depends on internal packages rather than the monolith; `road1.toml` recognizes the same topology. |
| X3 | PROVED | Audience facades expose capability without exposing internal authority packages or restoring the monolith. | Execution exposes the curated `facade::installed` namespace; `worth-query-host` is a pure audience reexport and cannot name `integration`. Trybuild rejects the attempted internal path, and boundary-check accepts the facade topology. |
| X4 | PROVED | Every public authority mint, retained capability, and recombination path is accounted for. | The unnecessary public reservation and integration doors are gone. Runtime authority-affinity attacks and compile-fail cases cover construction, retention, session minting, graph calls, reservation, and recombination. |
| X5 | PROVED | Failure, panic, cancellation, staleness, and drop preserve exact authority and consequential state. | Stage admission now precedes artifact mutation, partial capacity acquisition rolls back, reservation races admit only one holder, and lifecycle tests observe exact release/destruction after panic, cancellation, denial, staleness, and drop. |
| X6 | PROVED | Tests use causal worlds, real boundaries, independent oracles, intended-cause failures, hostile twins, and proportionate cost. | Reopened evidence now uses real installed authority worlds, one-axis native/resource causality matrices, no-residue receipt attacks, rollback/race tests, and explicit posture twins. Compiler-negative tests remain isolated in certification. |
| X7 | PROVED | Ordinary and warm paths remain structurally bounded and do not inherit replay, diagnostic, reconstruction, or compile-test inflation. | Warm package no-run paths measured 0.147s for execution and 0.138s for admission. The installed-world path was 23.633s after structural invalidation and 0.552s immediately warm; the full 320-test suite ran in 3.08s. Compiler certification remains isolated and ordinary packages do not import cert replay. |
| X8 | PROVED | One realistic workflow exercises Phases 2-5 through the public path without primitive/blob smuggling or Query domain vocabulary. | `artifact_phase_journey.rs` performs resource admission, managed allocation/move, native bulk access, and governed evidence receipt through the installed path. Its independent oracle checks 32 semantic values, exact bounds/counters, lifecycle effects, stage proof, distinct attempt identities, evidence binding, and governance posture. |
| X9 | PROVED | Formatting, functional suites, compiler certification, line caps, function scrutiny, Clippy, boundary-check, agent-context, and diff integrity are green. | Final verification is recorded below. All 234 dirty Rust files are within the 400-line rule or allowlisted (zero violations), and the dirty worth-query scope contains no unresolved implementation markers. |

## Resolved findings

The findings remain here so closure cannot erase the audit history.

| ID | Severity | Affected rows | Proven defect and closure |
| --- | --- | --- | --- |
| F01 | High | P2.4, X1, X5 | **Resolved.** A private typed pre-transfer stage admission now precedes move/lease mutation; denied retry evidence observes unchanged lifecycle state. |
| F02 | Medium | P2.2, X6 | **Resolved.** Real installed authorities replace boolean surrogates for affinity proof, with each authority dimension varied independently. |
| F03 | High | P3.1, X1 | **Resolved.** Exact row, field, chunk, projection, or scalar access bounds are carried by admission and exposed in evidence. |
| F04 | Medium | P3.8, X6 | **Resolved.** Real installed native-access worlds prove counter causality and non-causal invariance one axis at a time. |
| F05 | High | P4.5 | **Resolved.** `PartiallyMaterialized` sidecars retain only governance-eligible raw records while digesting the complete record set; derived copies cannot recover omitted material. |
| F06 | Medium | P4.3, X6 | **Resolved.** Every counter/provider-certification/decision-summary denial branch has an exact hostile twin and no-later-work oracle. |
| F07 | High | P5.2, X1 | **Resolved.** Canonical identities are length-framed and hashed, identifiers reject non-portable spelling, and collision twins prove distinctness. |
| F08 | Critical | P5.2, P5.10, X1, X4 | **Resolved.** Raw lowering and reservation are root-internal and reachable only after installed binding; compile attacks reject the former public doors. |
| F09 | High | P5.6, X5, X6 | **Resolved.** Later-subject failure rolls back earlier reservations, an eight-thread/one-slot race admits exactly one holder, and drop returns occupancy to zero. |
| F10 | Medium | P5.8, X6 | **Resolved.** The admission causality matrix varies every semantic/resource axis independently and asserts exact operational counters plus zero downstream work on denial. |
| F11 | Medium | P5.9, X6 | **Resolved.** The posture matrix includes partial result and yielded progress alongside async, degradation, partial effect, and retained progress, with explicit-consent attacks. |
| F12 | High | X3, X4 | **Resolved.** Host now purely reexports the curated installed audience; internal integration is absent and compile-rejected. |
| F13 | High | X8 | **Resolved.** The integrated artifact journey crosses Phases 2-5 through one installed public path with independent semantic, physical, lifecycle, identity, and governance oracles. |

## Verification log

- `cargo fmt --all`: completed before final compilation.
- `cargo test -p worth-query-execution`: 33 passed.
- `cargo test -p worth-query-admission`: 72 passed.
- `cargo test -p worth-query-installation`: 88 unit tests and 2 doctests
  passed.
- `cargo test -p worth-query-publication`: package compiles; it currently
  declares no direct tests.
- `cargo test -q -p worth-query --test installed_operating_world`: 320 passed,
  0 failed, in 3.08s.
- `cargo test -p worth-query-certification --test compile_certification`: all
  12 harness tests and all registered trybuild cases passed. The host-facade
  correction was followed by a focused green
  `query_compiler_boundaries_hold` rerun.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed in the worth-query workspace.
- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`:
  `Road 1 Cargo topology is valid`.
- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`: passed.
- Dirty-scope line audit: 234 Rust files inspected; zero unallowlisted files
  exceed 400 lines.
- Dirty worth-query marker audit: zero `TODO`, `FIXME`, `todo!`, or
  `unimplemented!` markers.
- `git diff --check -- .`: clean. Git emitted only working-copy CRLF notices.

## Warm-path measurements

- `cargo test -p worth-query-execution --no-run`: 0.147s warm.
- `cargo test -p worth-query-admission --no-run`: 0.138s warm.
- `cargo test -p worth-query --test installed_operating_world --no-run`:
  23.633s after structural invalidation and 0.552s on the immediate warm rerun.
- The full installed-world runtime suite completed in 3.08s.

The workspace-wide line-cap script remains red on unrelated pre-existing files
outside this dirty review scope. This ledger therefore closes Milestone 9.15
Phases 2-5; it does not claim that unrelated repository-wide line-cap debt has
been remediated.
