# Milestone 10 Sequential Phase Ledger

This ledger is the certification record for Phases 1–9 of
[Milestone 10](./milestone-10.md). The implementation accumulated in one
working candidate, so this is a sequential certification run over the current
tree—not a claim that the historical edits were reviewed phase by phase as
they were written.

Milestone 10 is closed. Store durability, WAL, and crash courts remain a
later Store-owned milestone. Phase 8 closed the courtroom. The duplicate
`throughput()` constructor was removed; `operational()` is the elevated
public Throughput + OnDemand constructor. The idle-versus-introspective
packet remains observation-cost evidence.

## Gate law

The next phase is locked until the current phase has all of the following:

1. a phase-local boundary review and exact implementation scope;
2. the phase's focused tests, source search/coverage manifest, and relevant
   compile-fail or runtime evidence;
3. a frozen phase-local source fingerprint and command record;
4. three fresh, separate, read-only reviewers: `qa-loop` for production
   correctness and authority, `qa-tests` for fixture/oracle/adversarial
   evidence, and `code-quality-qa` for ownership/topology/composition;
5. a reconciliation entry for every finding, including the root repair and
   rerun evidence; and
6. a gate decision recorded below.

A finding about a later phase is recorded as deferred and does not silently
expand the current gate. A current defect in the phase under review blocks the
gate. Any material repair reopens the affected phase and requires fresh
reviewers before the phase can close.

The reviews are proportional: they validate the current M10 surfaces and
compile-valid acceptance mutations. They do not attempt to become a general
Rust compiler, macro expander, public-API census, or name-resolution engine.

## Sequential gate status

| Phase | Certified scope | Required evidence | Gate status |
| --- | --- | --- | --- |
| 1 | Boundary, pre-change baseline, red controls, and source/owner coverage | `milestone-10-baseline.md`; named red-control record; M13 boundary/source-edge tests; source fingerprint; three fresh reviewers | **Closed — accepted after final repair review** |
| 2 | Eight-axis Foundational composition, identity/difference, resolution progression, and compile-fail boundaries | profile certification; identity/difference and progression adversarial twins; compile-time boundary suite; three fresh reviewers | **Closed — accepted after final repair review** |
| 3 | Observation disposition, typed absence, and all five optional-work disclosure classes | disposition matrix; absence legality; included/excluded work-class tests; three fresh reviewers | **Closed — accepted after profile-only OnDemand denial and public-front-door repair** |
| 4 | Facades, readiness inventory, documentation, and scoped facade exception | facade/trybuild tests; content-sensitive readiness evidence; docs links/examples; three fresh reviewers | **Closed — accepted after final proportional review** |
| 5 | Signal request → admitted → resolved → installed policy and all current reader/restore/reconfiguration seams | policy compiler tests; installed-policy consumer search; restore/rollback/fork tests; default/parallel checks; three fresh reviewers | **Closed — accepted after final repair review and independent Sol acceptance** |
| 6 | On-demand observation sessions, zero optional work, performed receipts, and lifecycle/restore gates | exact zero/nonzero twins; session lifecycle and cancellation tests; timer/counter mutation probes; default/parallel, WASM, workspace, and constitutional gates; three fresh reviewers plus independent Sol | **Closed — accepted after timer-gate repair and final bounded reviews** |
| 7 | Branch/snapshot/restore lifecycle boundaries, persisted observation activation, and explicit historical absence | restore/branch transition matrix; active-session interruption and failure-atomicity twins; activated-versus-never-activated historical availability; default/parallel, WASM, and constitutional gates; three fresh reviewers plus independent Sol | **Closed — accepted after lifecycle, absence, and provenance-gate repairs** |
| 8 | Signal certification courtroom, measured performance packet, serial/parallel operational parity, and deletion of tier-coupled compatibility paths | six-profile 1,024-output matrix under a 10-minute budget; recorded 4,096 scheduled bound; 120-batch warm median/p95/throughput; nodes/edit-width/fanout slopes; deletion probes; three fresh qa-loop, qa-tests, and code-quality reviewers | **Closed — accepted after restore-history, packet-honesty, and composition repairs.** |
| 8 remainder | Public production constructor | Remove `throughput()` synonym; elevate `operational()`; keep idle-versus-introspective packet as observation-cost evidence | **Closed — constructor removed; `operational()` elevated; independent review accepted** |
| 9 | Eight-axis consumer cutover | Every remaining six-axis `FoundationalProfileSet` construction names both new axes; Store operational mapping is Throughput + OnDemand; durability stays Store-owned | **Closed — eight-axis cutover accepted** |

## Phase 1 review packet

Phase 1 reviewers must inspect only the current baseline and boundary claim:

- `plans/WORTH-foundational/milestone-10-baseline.md` and the Phase 1 section
  of the M10 spec;
- the named M13 boundary/source-edge inventory and its mutation-sensitive
  controls;
- the pre-M10 red-control record and canonical digest parity evidence;
- the current Signal/Foundational ownership boundary relevant to the baseline.

They must not use Phase 6 zero-work/session claims to close Phase 1. The phase
closes only when the baseline is reproducible, the red control is genuinely
pre-cutover, and the inventory fails for the named compile-valid deletion,
misrouting, and bypass mutations.

## Phase 2–7 packets

Each later packet is opened only after the prior row is **Closed**. The packet
must name the exact symbols and files under review, the later-phase exclusions,
the focused command set, the source fingerprint, and the three reviewer
identities. A final aggregate QA pass is supplemental and cannot replace these
phase-local rows.

## Reconciliation record

| Phase | Reviewer role | Reviewer identity / revision | Findings and repairs | Rerun evidence | Decision |
| --- | --- | --- | --- | --- | --- |
| 1 | qa-loop | `m10_p1_qa_loop_final` @ `6c9a4f277` | Rejected cumulative lineage count; repaired to mutation-window sequence delta. Rejected a zero-test benchmark selector; repaired to the nested `--lib` path. Final fresh reread accepted. | `locality_red_controls` 7/7; exact digest 1/1; boundary inventory 20/20; benchmark selector lists 1 ignored test; final rerun ACCEPT | **Closed** |
| 1 | qa-tests | `m10_p1_qa_tests_fresh` @ `6c9a4f277` | Confirmed the lineage false-green repair, corrected benchmark command, and narrowed digest claims to implemented axes. Historical benchmark guard regression recorded as later performance work, not a Phase 1 gate failure. | Same focused suite; benchmark selector resolves to 1 real ignored test; final read-only ACCEPT | **Closed** |
| 1 | code-quality-qa | `m10_p1_code_quality_fresh` @ `6c9a4f277` | Rejected 418-line locality execution owner; split red-observation payload/delta construction into `locality_execution/red_observation.rs`. Deferred `profiles/progression.rs` 418-line debt to Phase 2 as scoped. | Parent 311 lines, child 118; fmt, diff, boundary-check, agent-context, scrutinizer (126 files/47 advisories/0 scan errors); final ACCEPT | **Closed** |
| 2 | qa-loop | `m10_p2_qa_loop_final` @ `6c9a4f277` | Confirmed canonical resolution ledgers are the sole stored progression authority; legacy narrowing is a compatibility projection; requested→admitted and admitted→materialized transitions validate family, relation, duplicate, omission, and materialization requirements. | Profile certification 35/35; compile-time boundaries 66/66; `cargo check -p worth-foundational`; fmt, diff, boundary-check, and agent-context green; final fresh ACCEPT | **Closed** |
| 2 | qa-tests | `m10_p2_plan_critic_a` (reassigned final qa-tests reviewer) @ `6c9a4f277` | Confirmed the 3×2 objective/activation matrix, exact objective and activation token coverage, front-door duplicate denial, relation mismatch twins, reverse-order canonicalization, exact ledger propagation, omission denial, and old six-axis/raw-string/duplicate/default compile-fail boundaries. | Profile-focused certification 35/35; profile compile-fail groups 8/8; full compile-time boundary suite 66/66; final fresh ACCEPT | **Closed** |
| 2 | code-quality-qa | `m10_plan_critic_b` (reassigned final code-quality-qa reviewer) @ `6c9a4f277` | Confirmed canonical-ledger ownership, wrapper delegation, coherent family/progression/resolution placement, and line-cap repair; no duplicate stored narrowing authority remains. Lower-lane public exports are explicitly deferred to Phase 4. | Certification 424/424; fmt, diff, boundary-check, agent-context, and scrutinizer green; `facade.rs` 673-line exemption is the only over-cap file; final fresh ACCEPT | **Closed** |
| 3 | qa-loop | `m10_p2_qa_loop_final` @ `6c9a4f277` (fresh repaired review) | Rejected loss of observation context from canonical identity, comparison, and reports; repaired context propagation through canonical basis, mismatch comparison, report sources, materialized reports, and report canonicalization. Rejected profile-only planning silently inferring `Inactive` for activation-sensitive `OnDemand` targets; all profile-only planner/front-door paths now return typed `ObservationDispositionRequired`. Fresh review accepted the public front-door denial/explicit-disposition twin. | Focused observation/materialization 5/5; optional work disclosure 4/4; certification 429/429; compile-time boundaries 66/66; library 22/22; formatting, diff, boundary-check, and agent-context green | **Closed** |
| 3 | qa-tests | `m10_p2_plan_critic_a` @ `6c9a4f277` (fresh repaired review) | Rejected incomplete optional-work evidence: added independent active/inactive/continuous/explicit claim gates for all five optional classes, exact context assertions, report/canonical twins, and an exclusion-side loop proving every optional class can be disclosed as excluded without observation context. Required and accepted the public front-door OnDemand denial/explicit Inactive twin. | Focused observation/materialization 5/5; optional work disclosure 4/4; certification 429/429; compile-time boundaries 66/66; library 22/22; final ACCEPT | **Closed** |
| 3 | code-quality-qa | `m10_plan_critic_b` @ `6c9a4f277` (fresh repaired review) | Accepted sole production and test ownership after the disposition/absence test split and public front-door coverage; no duplicate stored authority or mixed Phase 3 test owner remains. Deferred the lower-lane materialization docs/export mismatch to Phase 4, where facade and documentation closure is owned. | Certification 429/429; focused observation 5/5; library 22/22; scrutinizer 137 files / 55 candidates / 0 scan errors; fmt, diff, boundary-check, and agent-context green; only allowlisted `src/facade.rs` exceeds 400 lines | **Closed** |
| 4 | qa-loop | `m10_p4_qa_loop_final` @ `6c9a4f277` | No concrete runtime defect, false claim, or required wrong-reason mutation. Confirmed lower-lane re-export parity, descriptive/proof-bearing readiness separation, exact readiness inventories, and Phase 5 exclusion. | Certification 432/432; compile-time boundaries 67/67; focused facade 3/3; fmt, diff, boundary-check, agent-context green; final fresh ACCEPT | **Closed** |
| 4 | qa-tests | `m10_p4_qa_tests_final` @ `6c9a4f277` | Confirmed exact typed facade signature, intended compile-fail readiness substitution, exact readiness vocabulary, distinct stronger lane, and documentation match. No fixture or oracle blocker. | Same certification and compile-time suites; final fresh ACCEPT | **Closed** |
| 4 | code-quality-qa | `m10_p4_code_quality_final` @ `6c9a4f277` | Confirmed lower-lane facade is a projection, no duplicate authority, distinct Phase 4 test owners, and no non-allowlisted line-cap violation. | Scrutinizer 143 files / 55 advisories / 0 errors; only allowlisted `src/facade.rs` over cap; final fresh ACCEPT | **Closed** |
| 5 | qa-loop | `m10_p5_qa_loop_slice` + `m10_p5_builder_qa_loop` + `m10_p5_restore_qa_loop` @ `6c9a4f277` | Repaired stale parallel reference; installed-policy accessor collapse; builder compile/install boundary; and restore recompile from diagnostics request. Each fresh review accepted its repaired slice. | Policy 6/6 default+parallel; builder 7/7; checkpoint rebuild 4/4; active-policy restore 1/1; final bounded ACCEPTs | **Slices accepted; Phase 5 open** |
| 5 | qa-tests | `m10_p5_qa_tests_slice` + `m10_p5_builder_qa_tests` + `m10_p5_restore_qa_tests` @ `6c9a4f277` | Strengthened independent policy projections, denied-installation preservation, builder boundary, checkpoint-carried authority, and active-vs-captured restore evidence. Final fresh reviews accepted each slice. | Policy 6/6 default+parallel; builder 7/7; restore 4/4 plus active-policy 1/1; final bounded ACCEPTs | **Slices accepted; Phase 5 open** |
| 5 | code-quality-qa | `m10_p5_code_quality_slice` + `m10_p5_builder_code_quality` + `m10_p5_restore_code_quality` @ `6c9a4f277` | Established distinct installed authority, migrated request-collapse consumers, centralized builder installation, carried installed policy in checkpoint authority, and removed restore/fork request reconstruction. | Scrutinizer 147 files / 58 advisories / 0 errors; no non-allowlisted over-cap file; final bounded ACCEPTs | **Slices accepted; Phase 5 open** |
| 5 | qa-loop | `m10_p5_consumer_qa_loop` @ `6c9a4f277` | Migrated execution-flow, tracing, lineage, summary, snapshot, fork, and merge retention readers to installed policy projections; added legacy checkpoint compatibility default. | Policy 7/7 default+parallel; checkpoint rebuild 4/4; invalidation restore 3/3; full-library regressions repaired; fresh ACCEPT | **Consumer slice accepted; Phase 5 open** |
| 5 | qa-tests | `m10_p5_consumer_qa_tests` @ `6c9a4f277` | Added request-mirror mutation twin proving snapshot lineage follows carried installed authority; verified consequential retention/tier/tracing assertions in default and parallel lanes. | Phase 5 state 34/34 default; 34/34 parallel; policy 7/7 default+parallel; fresh ACCEPT | **Consumer slice accepted; Phase 5 open** |
| 5 | code-quality-qa | `m10_p5_consumer_code_quality` @ `6c9a4f277` | Added direct installed tracing/lineage projections and migrated remaining snapshot/fork/merge retention paths; no duplicate operational reader remains in the reviewed consumer scope. | Scrutinizer 154 Rust files / 60 advisories / 0 errors; no new hard composition defect; fresh ACCEPT | **Consumer slice accepted; Phase 5 open** |
| 5 | qa-loop | `m10_p5_close_qa_loop` @ `6c9a4f277` | Rejected and repaired rollback reconstruction from the diagnostics request mirror by carrying the installed policy in the rollback packet; later independent Sol review reopened Phase 5 for orthogonality and resolved-authority defects. | Phase 5 state 34/34; policy 8/8; full Signal library 1260 passed / 26 ignored; superseded by final Sol REJECT | **Superseded — Phase 5 reopened** |
| 5 | qa-tests | `m10_p5_close_qa_tests` @ `6c9a4f277` | Confirmed the prior slice’s evidence, but final independent Sol review found the policy request still acts as unresolved operational authority. | Phase 5 state 34/34 default and parallel; policy 8/8 default and parallel; superseded by final Sol REJECT | **Superseded — Phase 5 reopened** |
| 5 | code-quality-qa | `m10_p5_close_code_quality` @ `6c9a4f277` | Removed the unused request-mirror recompile path; final independent Sol review nevertheless found unresolved request-shaped authority and diagnostics ownership. | Scrutinizer 158 Rust files / 64 advisories / 0 errors; dirty line-cap pass; superseded by final Sol REJECT | **Superseded — Phase 5 reopened** |
| 5 | qa-loop | `m10_p5_final_qa_loop` @ `6c9a4f277` | Found the concrete full-parallel admission gap: grouped apply could bypass the installed full-parallel threshold. Repaired the apply-stage guard and added the under-threshold serial-fallback twin. The stale grouped-parallel fixture was then configured with an explicitly admitted aggressive threshold. | `m10_policy` 13/13 default+parallel; `phase5_state` 34/34 default+parallel; adversarial parallel 12 passed/1 ignored; policy boundary 4/4; telemetry contract 7/7; fresh bounded ACCEPT | **Closed** |
| 5 | qa-tests | `m10_p5_final_qa_tests` @ `6c9a4f277` | Found a compile-fail false green in the diagnostics-policy boundary fixture because it referenced a nonexistent type and could pass on E0425 alone. Repaired the fixture to use the existing `runtime_policy::SignalRuntimePolicy` and refreshed the snapshot to assert only the intended E0603 private-module denial. | `phase_5_policy_boundaries` 4/4 default+parallel; adversarial parallel 12 passed/1 ignored; policy 13/13 and Phase 5 state 34/34 in both lanes; fresh bounded ACCEPT | **Closed** |
| 5 | code-quality-qa | `m10_p5_final_code_quality` @ `6c9a4f277` | Found the public `adjust_runtime_policy` closure path could panic on invalid compiler input. Added `try_adjust_runtime_policy` with typed `SignalRuntimePolicyCompilationDenial`, documented the infallible method as known-valid convenience, and added invalid-adjustment preservation evidence. | Focused policy boundary 4/4; boundary-check; agent-context; formatting; scrutinizer 179 files / 79 candidates / 0 errors; only allowlisted Foundational facade over cap; fresh ACCEPT | **Closed** |
| 5 | independent Sol | `m10_p5_final_sol` (GPT-5.6 Sol, high) @ `6c9a4f277` | Independently re-read the post-repair runtime policy, installed-policy consumers, grouped apply threshold, restore/fork/merge/checkpoint/rollback seams, and public adjustment boundary. Found no remaining concrete runtime defect, false Phase 5 claim, or compile-valid acceptance mutation blocker. | `m10_policy` 13/13 default+parallel; `phase5_state` 34/34 default+parallel; adversarial parallel 12 passed/1 ignored; policy boundary 4/4; fmt, boundary-check, agent-context, dirty line-cap inventory green | **Closed — ACCEPT** |
| 6 | qa-loop | `m10_p6_qa_loop_recheck3` @ `6c9a4f277` | Repaired the concrete optional-timer construction defect in GC epochs and snapshot-batch commits; bounded reread found no remaining Phase 6 runtime or authority defect. | `mutation_probes` 9/9 default+parallel; `runtime_telemetry` 11/11; full Signal lib 1283/0/26 default and 1316/0/28 parallel; final bounded ACCEPT | **Closed — ACCEPT** |
| 6 | qa-tests | `m10_p6_qa_tests_recheck3` @ `6c9a4f277` | Confirmed typed absence on OnDemand restore, foreign-session denial with active foreign sessions, and gate-before-timer construction in both named owners. | `branch_restore`, `previous_value_access`, `session_tokens`, `mutation_probes` 6/6, 6/6, 6/6, 9/9 in both lanes; `runtime_telemetry` 11/11 in both lanes; final bounded ACCEPT | **Closed — ACCEPT** |
| 6 | code-quality-qa | `m10_p6_code_quality_recheck4` @ `6c9a4f277` | Confirmed observation-session, telemetry mutation, checkpoint/serde rebind, and public facade ownership after the timer repair; no composition, topology, line-cap, or public-contract defect. | workspace check, Signal check, WASM, fmt, diff, boundary-check, agent-context, line cap, scrutinizer (387 files / 232 advisories / 0 errors); final bounded ACCEPT | **Closed — ACCEPT** |
| 6 | independent Sol | `m10_p6_final_sol_bounded` (GPT-5.6 Sol, high) @ `6c9a4f277` | Found and required repair of two concrete pre-gate optional timers in `epoch.rs` and `snapshots.rs`; after the repair, the bounded final reread accepted the Phase 6 claim. | Full default/parallel Signal libraries green; timer mutation probes and runtime telemetry twins green; final Sol-high ACCEPT after repair | **Closed — ACCEPT** |
| 7 | qa-loop | `phase7_qa_loop_final` @ `6c9a4f277` | Repaired snapshot/restore and branch-boundary ordering so managed observation sessions are interrupted only after successful preflight/reconstruction, with failure-atomic denial and target-policy preservation. | Phase7 lifecycle focused lane 10/10; branch/restore, checkpoint, snapshot, and safe-point twins green; final fresh ACCEPT | **Closed — ACCEPT** |
| 7 | qa-tests | `phase7_qa_tests_final` @ `6c9a4f277` | Rejected a retained-target test that passed for the wrong reason; removed it and retained runtime-observed noncurrent replay, unknown-target failure atomicity, and activated-versus-never-activated absence twins. | Phase7 lifecycle 10/10; default/parallel focused lanes and WASM green; final fresh ACCEPT | **Closed — ACCEPT** |
| 7 | code-quality-qa | `phase7_code_quality_final` @ `6c9a4f277` | Confirmed coherent session, diagnostics-state, branch/restore, and temporal test ownership; no duplicate authority, mixed owner, or non-allowlisted line-cap defect. | Workspace/Signal checks, fmt, diff, boundary-check, agent-context, and dirty line-cap green; final fresh ACCEPT | **Closed — ACCEPT** |
| 7 | independent Sol | `phase7_sol_high_final` (GPT-5.6 Sol, high) @ `6c9a4f277` | Found and required three concrete repairs: interrupt active sessions on noncurrent restore, carry the target branch installed policy, and distinguish never-activated OnDemand history. A reciprocal provenance gate was corrected from `DescriptiveLineage` to the actual `DescriptiveFacts` owner. Final exact-tree reread found no remaining Phase 7 runtime or claim defect. | Phase7 lifecycle 10/10; full Signal default 1293/0/26 and parallel 1326/0/28; WASM, workspace, fmt, diff, boundary-check, agent-context, and line-cap gates green | **Closed — ACCEPT** |
| 8 remainder | qa-loop | `m10_p8_rem_qa_loop_a/b/c` (grok-4.6) @ working tree on `6c9a4f277` | No remaining public `throughput()` constructor or builder helper; `operational()` still installs Throughput + OnDemand through `for_tier(Operational)`; remainder was not pre-closed. | `m10_policy` 13/13 then 14/14; source search of `worth-signal` constructors | **Closed — ACCEPT** |
| 8 remainder | qa-tests | `m10_p8_rem_qa_tests_first` (grok-4.6) | Two ACCEPT; one REJECT of leftover `"installed throughput policy"` expects, missing deletion scan, and a duplicate Throughput assert. | Superseded by repair | **Superseded — repaired** |
| 8 remainder | qa-tests | `m10_p8_rem_qa_tests_recheck_a/b/c` (grok-4.6) | Confirmed `include_str!` deletion scan, renamed operational expects, unique constructor install test, and idle-lane wiring through `operational()`. | `m10_policy` 14/14 including `presets_source_does_not_reintroduce_a_throughput_constructor` | **Closed — ACCEPT** |
| 8 remainder | code-quality-qa | `m10_p8_rem_code_quality_a/b/c` (grok-4.6) | Constructor ownership remains `presets.rs`; no leftover `throughput_policy` helper; remainder Rust files under 400 lines. | scrutinizer 3 remainder files / 0 candidates; `m10_policy.rs` 310, `presets.rs` 142, `builder/policy.rs` 231 | **Closed — ACCEPT** |
| 9 | qa-loop | `m10_p9_qa_loop_a/b/c` (grok-4.6) | Consumer constructions name both new axes; Store operational evidence is Throughput + OnDemand; durability stays Store-owned; no second `throughput()` constructor. | `s6_operational_evidence_source_names` ok; `support_widening` ok; `worth-query` check ok; `physical-certification` check ok; `bank-server` check ok | **Closed — ACCEPT** |
| 9 | qa-tests | `m10_p9_qa_tests_first` (grok-4.6) | One ACCEPT; two REJECT of reconstructed getter oracle missing retention source lock. | Superseded by repair | **Superseded — repaired** |
| 9 | qa-tests | `m10_p9_qa_tests_recheck_a/b/c` (grok-4.6) | Confirmed `include_str!` lock of S6 factory for OperationalMinimal, Retained, Throughput, OnDemand; S6 `cfg(test)` remains the runtime oracle. | `s6_operational_evidence_source_names_throughput_on_demand_and_retained` 1/1 | **Closed — ACCEPT** |
| 9 | code-quality-qa | `m10_p9_code_quality_first` (grok-4.6) | One ACCEPT; two REJECT of mixed-level Query support export and misnamed/misplaced operations source lock. | Superseded by repair | **Superseded — repaired** |
| 9 | code-quality-qa | `m10_p9_code_quality_recheck_a/b/c` (grok-4.6) | Named `consumer_support_export_profile`; lock lives in `operational_profile_lock.rs`; remainder files under 400 lines. | `worth-query` check ok; source-lock test 1/1 | **Closed — ACCEPT** |

## Later-phase exclusions

Phases 1–9 are closed. This ledger does not reopen leftover idle-clock or
stage-report gating, renaming `ExecutionObjectiveProfile::Throughput`,
changing Operational's default away from Throughput + OnDemand,
parallelization, or a Store crash/WAL courtroom.

## Phase 8 remainder freeze record

- Working-tree remainder on candidate `6c9a4f277`.
- Packet: `crates/worth-signal/src/runtime_policy/presets.rs`,
  `crates/worth-signal/src/logic/transaction/runtime/state/builder/policy.rs`,
  `crates/worth-signal/src/tests/m10_policy.rs`,
  `crates/worth-signal/src/tests/performance_profiles/{named_scale_slopes,scheduled_node_bound}.rs`,
  and the caller/spec docs that elevate `operational()`.
- Focused command: `cargo test -p worth-signal --lib tests::m10_policy --
  --test-threads=1` (14/14), including
  `operational_installs_throughput_on_demand_objective` and
  `presets_source_does_not_reintroduce_a_throughput_constructor`.
- Boundary: `operational()` is the public Throughput + OnDemand constructor;
  `throughput()` is deleted; Operational still defaults to that pair;
  `ExecutionObjectiveProfile::Throughput` remains. Idle-versus-introspective
  evidence stays an observation-cost packet, not a synonym.
- Gate decision: **Closed — three qa-loop, three qa-tests (fresh after the
  expect-string and deletion-scan repair), and three code-quality reviewers
  accepted.**

## Phase 9 freeze record

- Working-tree remainder on candidate `6c9a4f277`.
- Packet: remaining Store/Query/Relational/bank-world/UI
  `FoundationalProfileSet` construction sites, S6
  `operational_evidence_profile()`, operations
  `operational_profile_lock.rs`, Query `consumer_support_export_profile()`,
  and Store vision/roadmap mapping docs.
- Focused command: `cargo test --manifest-path
  workspaces/worth-store/Cargo.toml -p worth-store-operations --features
  certification-test-authority s6_operational_evidence_source_names --
  --test-threads=1` (1/1).
- Broader: `cargo check -p worth-query`; `cargo check --manifest-path
  workspaces/worth-store/Cargo.toml -p worth-store-physical-certification`;
  `cargo check --manifest-path workspaces/worth-query-bank-world/Cargo.toml
  -p bank-server`.
- Scoped caveats: `worth-store-certification` remains blocked by
  pre-existing formal-models errors in untouched files;
  `worth-ui-runtime` remains blocked by a pre-existing query-binding
  `Regroup` match in an untouched file.
- Boundary: eight-axis consumer cutover only. Durability stays Store-owned.
- Gate decision: **Closed — three qa-loop, three qa-tests (fresh after the
  source-lock repair), and three code-quality reviewers (fresh after the
  export-split and lock-placement repair) accepted.**

## Phase 1 freeze record

- Candidate revision: `6c9a4f277e770adb1c0ebd00801408abf0b6c40e`.
- Phase packet: 20 files covering the M10 boundary/baseline docs, the M13
  boundary/source-edge inventory, the Operational red control, and the
  tier-independent operational digest.
- LF-normalized packet fingerprint (sorted `path<TAB>sha256` rows):
  `33cd93afa86f70dc6333658f6d8c7c9c9c80f3e3b222e9f480621d1b35c0997e`.
- Gate commands: `cargo test -p worth-signal --lib
  tests::domains::fintech::invalidation::locality_red_controls --
  --nocapture --test-threads=1`; exact operational digest parity; M13
  boundary inventory; corrected ignored benchmark selector; `cargo fmt --all
  -- --check`; `git diff --check`; boundary-check; agent-context.
- The measured current benchmark is slower than the historical guard on this
  candidate. That is recorded performance debt for the later performance
  phases; it does not change the historical-baseline/red-control decision.

## Phase 2 freeze record

- Candidate revision: `6c9a4f277e770adb1c0ebd00801408abf0b6c40e`.
- Phase packet: 23 files covering the eight-axis profile composition and
  identity/difference laws, objective and activation families, public
  progression front doors, canonical resolution ledgers, phase-local
  adversarial tests, and compile-fail boundary fixtures. The packet excludes
  this ledger and the milestone specification so its fingerprint is stable.
- LF-normalized packet fingerprint (sorted `path<TAB>sha256` rows):
  `f2506753f77979c8d19abf7f4f1c0a4469e61bf86609244428382ce788e4287b`.
- Gate commands: `cargo test -p worth-foundational --test certification
  -- --test-threads=1`; profile compile-fail groups and the full
  `compile_time_boundaries` suite; `cargo test -p worth-foundational --lib
  -- --test-threads=1`; `cargo check -p worth-foundational`; `cargo fmt --all
  -- --check`; `git diff --check`; boundary-check; agent-context; dirty Rust
  line-cap inventory.
- Boundary: Phase 2 certifies Foundational profile meaning, identity,
  difference, composition, and requested/admitted/materialized resolution
  progression. Observation disposition legality, public facade closure, and
  Signal runtime adoption remain later phase gates.
- Gate decision: **Closed — three fresh phase-local reviewers accepted after
  the final split and canonical-ledger repair.**

## Phase 3 freeze record

- Candidate revision: `6c9a4f277e770adb1c0ebd00801408abf0b6c40e`.
- Phase packet: 29 LF-normalized files covering the observation disposition
  and typed absence owners, materialization planner/disposition storage,
  optional-work ontology and claim gates, performance canonical/comparison/
  report propagation, the sole disposition/absence and work-disclosure test
  owners, and the Phase 3 test-requirement/documentation surface. The packet
  excludes this ledger and the milestone specification so its fingerprint is
  stable.
- LF-normalized packet fingerprint (sorted `path<TAB>sha256` rows):
  `411c30951a0c4ff0efe3fe55372717c5d6db977865c4bf53ca441baba04d8e55`.
- Gate commands: `cargo test -p worth-foundational --test certification --
  --test-threads=1` (429/429); `cargo test -p worth-foundational --test
  compile_time_boundaries -- --test-threads=1` (66/66); `cargo test -p
  worth-foundational --lib -- --test-threads=1` (22/22); focused observation
  disclosure/materialization tests (5/5); optional work disclosure (4/4);
  `cargo fmt --all -- --check`;
  `git diff --check`; boundary-check; agent-context; and dirty Rust
  line-cap/scrutinizer review (137 dirty Rust files, only the allowlisted
  `src/facade.rs` over 400 lines).
- Boundary: Phase 3 certifies descriptive observation disposition, typed
  observation-specific absence, all five optional observation work classes,
  active/inactive/continuous/explicit claim gates, independent included and
  excluded work disclosure, and preservation of exact profile identity plus
  disposition through canonical evidence and reports. Facade/docs closure,
  runtime observation sessions, Signal adoption, and lower-lane re-exports
  remain Phase 4 or later.
- Deferred reconciliation: the lower-lane profile materialization document
  advertises `_with_disposition`, but the lower-lane module does not yet
  re-export that function/type. This is recorded as Phase 4 facade/document
  debt and is outside the Phase 3 boundary.
- Gate decision: **Closed — three fresh proportional phase-local reviewers
  accepted after the profile-only OnDemand denial and public-front-door
  evidence repairs.**

## Phase 4 freeze record

- Candidate revision: `6c9a4f277e770adb1c0ebd00801408abf0b6c40e`.
- Phase packet: lower-lane materialization facade, exact M10 readiness
  inventory, Phase 4 facade parity and compile-boundary tests, readiness
  vocabulary tests, and the descriptive-surface/facade documentation.
- Gate commands: certification 432/432; compile-time boundaries 67/67;
  focused facade parity 3/3; `cargo fmt --all -- --check`; `git diff --check`;
  boundary-check; agent-context; and dirty Rust composition review.
- Boundary: Phase 4 certifies public lower-lane/root/common parity, exact
  descriptive readiness evidence, typed separation from stronger proof-bearing
  readiness, and documentation alignment. Signal runtime policy adoption,
  runtime sessions, and Store implementation remain Phase 5 or later.
- Scoped exception: `crates/worth-foundational/src/facade.rs` remains the
  allowlisted re-export aggregator above 400 lines; no other dirty Rust file
  exceeds the cap.
- Gate decision: **Closed — three fresh proportional phase-local reviewers
  accepted. Phase 5 is now unlocked and current.**

## Phase 5 freeze record

- Candidate revision: `6c9a4f277e770adb1c0ebd00801408abf0b6c40e`.
- Phase packet: 98 content paths (93 Rust) covering the Signal runtime-policy
  request/admission/compiler/resolution/installation progression, every
  current execution/maintenance/diagnostic/tracing/lineage/restore/fork/merge/
  checkpoint/rollback consumer, parallel admission and grouped-apply policy,
  the public runtime adjustment front door, Phase 5 state and policy tests,
  adversarial parallel tests, compile-fail boundary fixtures, and the Phase 5
  policy inventory. The packet excludes this ledger, the M10 specification,
  and the separate Foundational Phase 1–4 packet.
- LF-normalized packet fingerprint (sorted `path<TAB>sha256(raw bytes)` rows,
  no trailing newline):
  `95fe69097dff9a763edd772fbc36dbcd7a216d00312c34c52259fa8551be210e`.
- Focused gate commands: `cargo test -p worth-signal --lib
  tests::m10_policy -- --test-threads=1` (13/13) and the same with
  `--features parallel`; `cargo test -p worth-signal --lib
  tests::phase5_state -- --test-threads=1` (34/34) and the same with
  `--features parallel`; `cargo test -p worth-signal --features parallel
  --lib tests::adversarial_parallel -- --test-threads=1` (12 passed, 1
  ignored); `cargo test -p worth-signal --test phase_5_policy_boundaries --
  --test-threads=1` (4/4) in both default and parallel lanes; and the
  telemetry-contract parallel selector (7/7).
- Broader gates: `cargo test -p worth-signal --lib --quiet` (1239 passed,
  26 ignored); `cargo test -p worth-signal --features parallel --lib --quiet`
  (1272 passed, 28 ignored); `cargo check -p worth-signal` and `--bins`;
  `cargo check -p worth-signal --target wasm32-unknown-unknown`; formatting,
  diff-check, boundary-check, agent-context, and dirty Rust composition
  scrutiny (179 files, 79 advisories, 0 errors). The dirty inventory contains
  180 Rust paths; the only file over 400 lines is the explicitly allowlisted
  673-line `crates/worth-foundational/src/facade.rs` aggregator.
- Boundary: Phase 5 certifies compiler-visible request → admitted → resolved
  → installed policy progression; installed-policy-only runtime consumption;
  typed public mutation denial and rollback preservation; checkpoint/fork/
  merge/restore authority; diagnostics and lineage retention; and full versus
  staged/serial parallel admission at the installed threshold. Observation
  sessions, zero-optional-work claims, performed receipts, production
  throughput slopes, Store adoption, and later certification-court work remain
  excluded to later phases.
- Gate decision: **Closed — qa-loop, qa-tests, code-quality-qa, and a fresh
  independent GPT-5.6 Sol high review all accepted the post-repair freeze.**

## Phase 6 freeze record

- Candidate revision: `6c9a4f277e770adb1c0ebd00801408abf0b6c40e`.
- Phase packet: 34 explicitly listed LF-normalized paths covering the managed
  observation-session request/admission/active/completion/graph/state owners,
  the optional telemetry mutation and graph telemetry owners, GC and snapshot
  timer owners, branch/restore and previous-value lifecycle seams, the
  performed/session/restore tests, and the public bridge/UI call sites. The
  packet excludes this ledger and the M10 specification.
- LF-normalized packet fingerprint (sorted `path<TAB>sha256(raw bytes)` rows,
  no trailing newline):
  `c99d602acf8f954627ff9d85f06e70a7709fbf32fb8367dfad76b9f0a89bcc57`.
- Focused gate commands: `cargo test -p worth-signal --lib
  tests::observability::mutation_probes -- --test-threads=1` (9/9) and the
  same with `--features parallel`; `tests::observability::runtime_telemetry`
  (11/11) in both lanes; branch restore, previous-value, session-token, and
  lifecycle selectors in default and parallel lanes; and the receipt/session
  integration selectors. The mutation probe checks that both `RuntimeInstant`
  constructions remain after the `OptionalTelemetry` gate.
- Broader gates: `cargo test -p worth-signal --lib -- --format terse`
  (1283 passed, 0 failed, 26 ignored); the same with `--features parallel`
  (1316 passed, 0 failed, 28 ignored); `cargo check --workspace --quiet`;
  `cargo check -p worth-signal --target wasm32-unknown-unknown --quiet`;
  `cargo fmt --all -- --check`; `git diff --check`; boundary-check;
  agent-context; dirty Rust line-cap guard; and scrutinizer (387 files,
  232 advisory candidates, 0 scan errors). Only the explicitly allowlisted
  `crates/worth-foundational/src/facade.rs` exceeds 400 lines.
- Boundary: Phase 6 certifies compiler-visible observation-session admission,
  cancellation/stale/duplicate/foreign completion denial, typed optional
  absence and zero-work behavior under OnDemand, explicit selected-surface
  capture, performed receipt admission, checkpoint/serde rebind, branch
  restore filtering, and gate-before-construction for optional timers and
  counters. Phase 7 lifecycle courtroom work was intentionally excluded from
  this packet and is certified below; Phase 8 throughput/certification
  performance proof and deletion, and Phase 9 Store adoption remain open.
- Reconciliation: the independent Sol review found the two pre-gate timer
  constructions in `data/graph/lifecycle/epoch.rs` and
  `data/graph/storage/entries/snapshots.rs`. Both were moved behind the
  `OptionalTelemetry` gate, and the mutation probe plus default/parallel
  runtime twins were rerun. Fresh `qa-loop`, `qa-tests`, `code-quality-qa`,
  and GPT-5.6 Sol-high reviews accepted the repaired source.
- Gate decision: **Closed — all four bounded reviewers accepted after the
  timer-gate repair.**

## Phase 7 freeze record

- Candidate revision: `6c9a4f277e770adb1c0ebd00801408abf0b6c40e`.
- Phase packet: 32 explicitly listed paths covering the observation-session
  lifecycle, persisted activation state and typed historical absence,
  snapshot/restore and branch-boundary owners, failure-atomicity seams, and
  the phase-local temporal/observability tests:
  `crates/worth-signal/src/observation/session/{active,admission,completion,graph,mod,request,state}.rs`,
  `crates/worth-signal/src/data/graph/diagnostics_access/artifacts/{historical,tier}.rs`,
  `crates/worth-signal/src/diagnostics/policy/materialization.rs`,
  `crates/worth-signal/src/diagnostics/runtime/state.rs`,
  `crates/worth-signal/src/diagnostics/runtime/state/{lifecycle,retained,snapshot}.rs`,
  `crates/worth-signal/src/state/diagnostics.rs`,
  `crates/worth-signal/src/logic/transaction/runtime/state/branching/{fork_snapshot,lifecycle,snapshotting}.rs`,
  `crates/worth-signal/src/logic/transaction/runtime/state/branching/snapshotting/{capture,validation}.rs`,
  `crates/worth-signal/src/tests/temporal_runtime.rs`,
  `crates/worth-signal/src/tests/temporal_runtime/{branch_restore,previous_value_access,phase7_lifecycle}.rs`,
  `crates/worth-signal/src/tests/resource_runtime/safe_point.rs`,
  `crates/worth-signal/src/tests/observability/{access_counters,artifact_policy,materialization,retention_bounds,tier_truth}.rs`,
  `crates/worth-signal/src/tests/harness_bridge.rs`, and
  `crates/worth-signal/src/tests/phase5_state/retention.rs`. The packet
  excludes this ledger and the M10 specification.
- LF-normalized packet fingerprint (sorted `path<TAB>sha256(raw bytes)` rows,
  no trailing newline):
  `a401baeadd9eed6afc5ca4d5d09ee301bf7f93053657f79b447eef2d94734787`.
- Focused gate commands: `cargo test -p worth-signal --lib
  tests::temporal_runtime::phase7_lifecycle -- --format terse` (10/10) and
  the same with `--features parallel`; branch-restore (6/6), checkpoint
  rebuild (4/4), snapshot contract (3/3), and safe-point lifecycle selectors
  in default and parallel lanes. The phase-local suite covers current and
  noncurrent capture/restore, target-policy preservation, active-session
  interruption only after successful reconstruction, failure-atomic denial,
  branch switching, and explicit OnDemand activation versus never-activated
  historical absence for both explanation and provenance.
- Broader gates: `cargo test -p worth-signal --lib -- --format terse`
  (1293 passed, 0 failed, 26 ignored); the same with `--features parallel`
  (1326 passed, 0 failed, 28 ignored); `cargo check --workspace --quiet`;
  `cargo check -p worth-signal --target wasm32-unknown-unknown --quiet`;
  `cargo fmt --all -- --check`; `git diff --check`; boundary-check;
  agent-context; and dirty Rust line-cap scrutiny (398 dirty Rust paths,
  only the explicitly allowlisted 673-line
  `crates/worth-foundational/src/facade.rs` exceeds 400 lines).
- Boundary: Phase 7 certifies that snapshot, restore, branch capture, and
  branch transfer are observation-session boundaries; successful noncurrent
  restore carries the target branch's installed policy; persisted activation
  survives graph/checkpoint reconstruction; OnDemand history distinguishes
  never activated from policy omission; and explanation/provenance availability
  is gated by the actual `DescriptiveFacts` owner. Phase 8 throughput and
  certification performance proof, and Phase 9 Store adoption, remain open.
- Reconciliation: Sol's first reread found noncurrent restore did not interrupt
  an active session, restored the current branch policy into the target, and
  lacked an observation-specific historical absence outcome. The repairs added
  successful-boundary interruption, target-policy carriage, persisted
  activation masks, and `ObservationNotActivated`. Sol then found that
  provenance availability was checking `DescriptiveLineage` although the
  production recorder owns provenance under `DescriptiveFacts`; the gate and
  reciprocal facts-only restore test were corrected and rerun. The final
  exact-tree Sol review accepted with no remaining concrete Phase 7 defect.
- Gate decision: **Closed — qa-loop, qa-tests, code-quality-qa, and the fresh
  independent GPT-5.6 Sol-high reviewer all accepted after the listed repairs.**
