# Test-cost census and prebuilding

Scope: root, Query, Bank and Store. worth-ui is excluded. This implements Track B
step B3 under milestone-3's verification tiers; it changes test execution, never
Store's durability, recovery, authority, or sweep membership contracts.

## Boundary review

Cargo metadata and compiler JSON own package, artifact and diagnostic identity.
The runner owns compilation and prerequisite lifetimes. A running court consumes
prebuilt executables and compiler outcomes; it may not acquire the package lock
or create another build graph. Fixture diagnostics must be attributed by Cargo's
target identity, never by process exit alone. A missing prerequisite is a typed
refusal, never success. The source scanner reports candidates; only resolved
compiler invocations enter enforcement.

Existing competing compilation paths are trybuild sessions, Store's
`independent_executable` first-use builds, Store's per-fixture compiler boundary
driver, Proof's band-guard specimen builds and Bank's metadata call. The latter
two preserve real dependency-tree/IR and metadata observations respectively.
The Store certification-absence fixture requires a separate production graph.

The lane inventory must start from all discovered tests, ignoring default
filters, and compare every nonignored identity against the complete partition
set. Ignored tests stay visible as scheduled-only and are never executed here.
Legacy logs lack artifact/CPU/setup data: absence of measurements cannot become
a zero, a passed budget, or permission to remove a landing lane.

## Implementation sequence

1. `scripts/test_cost/` owns standard-library census, retained command logs,
   Cargo artifact accounting, lexical source sites, inventory multiplicity,
   compile-size inputs and non-gating CPU/setup telemetry. Fixture workspaces
   protect attribution, missing data, duplicate graphs and coverage failures.
2. Merge `codex/m3-courts` before execution-path changes. Establish shared
   prebuild/result reading in the existing gates, with complete Cargo JSON.
   Move uniform trybuild lists with a codemod; retain all fixture expectations.
   Prebuild executable prerequisites before any court starts.
3. Enforce no in-test compilation, one compilation per declared compiler unit,
   and exact lane multiplicity. Reserve slots from measured process CPU rather
   than test names. A whole-pool reservation requires measured parallel work.
4. Store: batch compile-fail inputs by required feature environment, preserve
   production/certification claims and lockfile versions, move pass cases into
   ordinary courts, pool only independent sweep cases, and fold formal builds.
   Drop serial workspace execution only after coverage passes. Keep TLC intact.
5. Retain before/after censuses and ranked residue under target/test-cost. Run
   affected nonignored suites once through prebuilt execution, repair moved
   selectors/snapshots, merge courts again, and run required constitution,
   generated-context, format and dirty line-cap checks. Obtain independent
   adversarial review of the final commit range and exact lane logs.

## QA considerations

An unrelated compiler error cannot satisfy a semantic denial. Batched failures
must still compile every fixture and retain successful counterparts. Child
selectors must name moved modules exactly, and process claims still use real
independently compiled executables. Sweep placement preserves every case,
continuation, panic and comparison; adaptive or shared-state loops stay serial.
Timings are telemetry, never merge gates. No optimization profile, system
configuration, Cargo feature, compatibility path or sampling is introduced.

## Requirement map

These names map the B3 brief and the widened workspace scope to executable
checks. Milestone-3 D17 governs typed refusals; iteration laws 7, 16 and 22
govern compile size, test names and non-gating measurements respectively.

| Requirement | Enforcement / named evidence |
|---|---|
| Build census, including nested gates | `test_nested_gate_processes_register_their_complete_evidence_logs`; `test_direct_cargo_uses_planned_runner_and_deduplicates_only_its_known_alias`; `test_direct_preflight_refusal_never_reuses_an_old_canonical_capture`; `test_fresh_artifacts_do_not_count_as_compilation` |
| No Cargo or trybuild in running tests | `census.py --all --check-sources`; `test_named_executable_and_imported_command_alias_cannot_bypass_the_rule`; `test_cargo_in_autodiscovered_binary_unit_tests_is_refused`; `test_external_unit_tests_keep_enclosing_alias_and_constant_bindings`; `test_literal_included_tests_keep_compiler_sleep_and_timeout_sites` |
| Real batched compiler fixtures and workspace lock | `test_each_diagnostic_is_attributed_to_its_fixture_not_the_aggregate_exit`; `test_generated_package_may_be_new_but_registry_versions_stay_workspace_locked`; `compiler_probe.py` |
| Missing prerequisites refuse through types | `CompilerFixtureRefusal`, `PrebuiltExecutableRefusal`, Store compiler-boundary refusal types; consumer courts assert the real outcomes |
| Complete compiler snapshots | `compiler_codes_labels_source_and_coordinates_are_not_normalized_away`; `compiler_warning_text_and_order_remain_part_of_the_snapshot` |
| Production certification absence | `production_denial_refuses_a_certification_capture`; real `certification_authority_is_absent_in_the_production_environment`; prebuilder rejects resolved test features |
| Successful fixtures compile and execute ordinarily | `test_ordinary_test_executes_the_program_and_preserves_a_moved_test_name`; emitted supported-program court tests |
| Independently built process executables | `test_only_independently_compiled_binary_can_be_retained`; existing moved process courts consume the manifest |
| Compile-count ratchet | `test_same_unit_recompiled_in_another_directory_exceeds_budget`; `test_identical_failed_unit_in_two_cargo_invocations_exceeds_unchanged_budget`; `test_multiple_error_messages_in_one_invocation_are_one_budgeted_attempt`; `test_unattributed_unsuccessful_outcome_is_refused_with_or_without_artifact_json`; `test_cached_different_libraries_refuse_even_when_no_compilation_occurs`; committed `scripts/test_cost/budgets.json` |
| Measured slot reservations, never timing verdicts | `test_recommendation_uses_peak_observed_cpu_parallelism_without_a_timing_gate`; `test_generated_config_preserves_timeouts_and_places_measured_rule_first` |
| Construction input fingerprints | Executable court tests under `setup_observation::`; only complete named recipes are instrumented |
| Exhaustive independent worker sweeps | Store sweep tests and changed oracle/process parents; `test_rerunning_the_cutover_does_not_pool_a_worker_again` |
| Exact lane coverage, including executable examples | `test_zero_and_multiple_selections_cannot_prove_exact_coverage`; `test_nextest_metadata_binds_identity_and_is_filtered_before_libtest_listing`; `transport_probe.py` |
| Ignored tests stay scheduled-only | `test_scheduled_scale_is_never_an_executable_landing_lane`; the transport probe inventories an ignored panic without executing it |

Boundary-check is being replaced under `orchestration/m3/boundary_triage.md`.
This slice does not edit that checker; its no-in-test-compilation rule is owned
by `scripts/test_cost` for the coordinator's subsequent port.

## Corrected historical reservation inventory

The coordinator's 2026-10-30 ruling establishes that the original seven
whole-pool entries were hypotheses. Retained Landing 1 configuration and
executions refute six: `branch_transitions_oracle`, `canonical_journal_oracle`,
`physical_checkpoint_oracle`, `limit_sweep`, `process_kill`, and
`recovery_interruption`. Only `anchored_record_members::oracle` has the claimed
matching whole-pool rule. No reservation is added to satisfy a false premise.

`known_cases.py` preserves the source loops, execution events and every examined
filter as counterevidence. A refuted hypothesis is resolved inventory, while
missing source/execution/configuration or unsupported selector grammar remains
unknown. Actual slot parallelism still requires CPU measurement; neither a
serial source loop nor a matching reservation measures it. The original failed
report remains byte-identical; the corrected verdict is a separately derived
report under `target/test-cost/b3-corrections/known-cases.{json,md}`.

## Incremental coordinator rulings after the initial handoff

The existing `a990632506` evidence remains historical; corrections publish new
bounded captures. No exhaustive sweep, ignored test, Query scale lane or UI
workspace is needed to validate this infrastructure correction.

- Compiler stages are declared before execution in `compiler_stages.json`.
  NullableOptional retains its exact source, production audience and E0080
  golden, using normal code generation. Codegen E0080 can carry its callsite
  in rustc's separate instantiation note; the reader requires that exact source
  for each error group and preserves every rendered error and note. Check-stage
  output cannot use that codegen attribution route. Named proofs include
  `test_nullable_optional_is_codegen_only_in_the_same_normal_production_audience`,
  `every_codegen_error_requires_its_own_attributed_boundary`, and
  `check_stage_cannot_use_a_codegen_instantiation_note_as_its_error_span`.
- Cargo's preexecution unit graph declares required compiler units, including
  physical platform, output, full profile and resolved dependency feature
  graph. Artifact hashes cannot declare units. The unchanged maximum is one;
  identical units in different directories still collide, and undeclared
  command, flag, profile, feature or config drift refuses. Cold bounded real
  production/certification evidence must accompany synthetic graph tests;
  a warm zero cannot establish this claim.
  Error-attributed failed compiler attempts count once per declared unit per
  Cargo invocation, including targets that emit no successful artifact. Multiple
  diagnostics do not multiply the count; cached warnings do not establish an
  attempt. Ambiguous target-to-unit attribution refuses rather than passing as
  zero compiler work. Artifact census counts retain their fresh:false meaning.
  Unsuccessful compiler invocations without an attributable failed unit retain
  an explicit unknown/refusal, including process failures without error JSON.
  Successful build completion separates later test/run runtime failures from
  compilation. Non-compiling graph declarations never mint compiler attempts.
  Direct census Cargo commands use the same predeclaring runner. Markerless
  historical captures may supply a leading command/outcome, never unit identity;
  cached observations without a declaration cannot certify a compiler budget.
- Closed Root/Query/Bank testkits observe only complete named construction
  recipes, using the optional non-gating transport documented in
  `scripts/test_cost/setup_observation.md`. Opaque and uninstrumented recipes
  stay unknown. No fixture sharing follows from a fingerprint alone.
- The coordinator's `test_cost_pool_integration_ruling.md` supersedes the
  physical-Execution control-door proposal. The existing independent harness
  remains owned by `worth-store-test-support::sweep::run_jobs`; it introduces no
  ambient leased semantic context around a Store case. Its only normal-graph
  consumers must remain courts, certification harnesses and testkits; ordinary
  product callers may reach it only through their development/test graph.
- Register only that named function through the replacement performance law's
  local `expect(clippy::disallowed_methods)` declaration: independent
  certification harness, one width query, one scope and one spawn site. Keep
  queue membership, ordered outputs, catch-per-case panic handling and final
  canonical panic selection. Width zero/one drains the same queue directly on
  the calling thread, without constructing a worker scope. Do not introduce
  Execution doors, features, runtime dependencies or checker/config edits.
- Focused evidence covers empty input, repeated equal inputs, direct one-worker
  execution and complete draining after multiple panics at serial and parallel
  widths. Qualify those tests and a real Store serial/nested-stop case only after
  the supplied combined source and reviewed checker are integrated on the
  pinned compiler. Verify the integrated consumer graph, strict Clippy,
  boundaries, source caps, formatting and current nonignored membership;
  unchanged exhaustive sweeps are not rerun for a local site declaration.
  Earlier 1.94 captures remain references, not integrated 1.99.1 qualification.
- The coordinator's subsequent committed-profile ruling retires the old
  `test_environment_keeps_every_core_and_disables_debug_information` assertion
  that the runner must force debug zero. Its replacement preserves parallelism
  and build identity while asserting no injected profile/incremental/toolchain
  settings. Native, documentation-only, direct census and standalone prebuild
  qualification refuse external artifact-changing inputs before execution;
  committed profiles and the selected checkout pin govern the actual capture.
  Existing overridden captures retain their original inputs/results unchanged.


### Integrated pool owner and duplicate court mounts

The coordinator's physical-owner ruling after `bad2fed572` relocates the existing
independent sweep and its seven tests to the dependency-free
`worth-store-testkit-sweep` owner. Its three courts import it directly; Relational
loses only its scheduling-only TestSupport edge. Fixture dependencies, serial
fallback, complete panic draining, input order and every case remain unchanged.

The incoming `959ee77f37` court-layout decision supersedes forty unmounted old
paths reintroduced during integration. Their mounted counterparts retain all
seven named tests (three scheduled profiles, two structural-cost proofs, the
25-input sealed-artifact denial and the anchored replacement oracle). No claim
is retired. Physical-format oracle helpers live in `oracle/support/literal_vectors`;
phase-eight checkpoint/history helpers retain the canonical oracle/process
support mounts; incarnation children retain `writable_ownership` selectors.
The anchored replacement's independent-case collection and witness reduction
are transferred to its mounted `replacement_recovery` test before removing the
old copy. `category-path-audit.json` in the B3 integration evidence records every
old mount and named-test counterpart. This removes duplicate source copies,
not tests, continuations or assertions.


### Strict consumer stage and sanctioned test sites

The Store reader accepts the producer's required `proof_stage` field and checks
its Store check-only contract at the untrusted observation boundary. It retains
`deny_unknown_fields`; codegen, unknown and missing stages refuse. The named
`check_stage_observations_preserve_the_strict_wire_contract` and
`code_generation_unknown_and_missing_stages_cannot_satisfy_store_check_claims`
tests cover this protocol invariant. Actual compiler-reader owner cases consume
the previously completed production/certification batches without Cargo in tests.

The coordinator's coordinated storage-owner unit-test ruling permits exactly
39 existing thread calls in 22 named functions, with function-local categories,
purposes, source items and lexical method counts. Its positive owner-admission
ruling permits function-local dead-code expectations on 15 compile-only consumer
witnesses. No signatures, bodies, barriers, joins, limits, assertions, selectors
or main behavior change. These are distinct from the independent-case sweep;
shared-state race proofs retain their original physical coordination.

The positive-program cutover also gives the durability specimen and bounded
record declarations one physical court mount. Ordinary positive tests call that
same owner; no program entry, compiler claim or runtime assertion is removed.
Compile-only consumer functions and outcome payload fields carry local
`expect(dead_code)` declarations where they intentionally accept values that
the positive program cannot fabricate. Their original signatures, bodies and
typed payloads remain checked. This does not confer runtime evidence on them.

The outer-prebuild decision retires the unreachable
`compiler_boundary/bounded_process.rs` execution implementation. Repository
search of Store and `scripts/test_cost` found only its own declarations and its
module mount; it contained no tests. Delete that dead implementation rather
than retaining an unused second compiler-process owner. The result reader
continues to check every attributed denial from the real outer compiler batch.

Prebuilt nextest runs stream result events into their existing evidence log.
`test_result_is_flushed_while_the_child_is_alive_and_is_not_duplicated` uses a
real child that cannot finish until its first result is visible in the log.
The normal exit footer remains the only indication of a completed lane; an
interrupted capture is never rewritten into a completed capture.

### 2026-10-30 landing disposition

The user authorized landing the reviewed integration without the remaining
Store verification gate and made finishing Track B and reducing test cost the
next priority. This is a landing exception, not a claim that Store qualification
or Milestone 3 is complete.

The reviewed implementation is `8514c21769`. Its strict static checks, repository
boundaries, agent-context, formatting, source caps and lock alignment passed.
The separate Query/Bank qualification and independent source reviews passed.
All 88 failures from the earlier Store capture now have passing observations.

The Store follow-up finished during landing preparation. Its 1,049 selected
cases produced 1,041 passes and eight failures: owner units 213/0, ordinary
46/0, process 725/0, cost 4/0, determinism 1/0, oracle 27/0, and compiler-proof
readers 25/8. The eight failures report `WrongDiagnostics`; they remain open.
The actual logs are retained under
`target/test-cost/b3-pool-integration/store-composed-correction/`, including
`run-correction-compile-fail.log`. Earlier failed and interrupted captures retain
their original dispositions. This Store follow-up is not green.

Ignored tests, Query scale, WORTH UI and exhaustive Store certification were not
run. The warm prebuild and follow-up measurements do not establish a complete
cold-landing speedup. Track B must resolve the remaining proof-reader failures
and complete its outstanding cost work with honest end-to-end evidence.

### Next B3 slice: pinned compiler proof snapshots

Boundary review: the completed production/certification batches own the actual
compiler observations. The existing public result reader validates their
fixture attribution before comparing complete diagnostics. Its audit of 136
snapshot fixtures finds 35 stale goldens; eight court groups fail comparison.
The operations, source identities, audiences and primary denials remain intact.
Error-code inventories match except the compiler's omitted secondary E0282
after the preserved missing-method E0599. No boundary was opened by that change.
The pinned 1.99.1 compiler ruling supersedes the old diagnostic wording, optional
private-field and macro-origin notes, suggestion layout and type rendering.
No test or denial claim is retired; literal goldens must describe actual output.

Plan: update only those 35 complete observed goldens, retaining existing external
coordinate masks. Do not change the renderer, fixture Rust, production behavior,
audiences, features or compiler stages. Reuse the already-built public reader to
audit all 136 snapshots, then run the eight failed court groups with the existing
native binaries and real compiler outcomes. Keep the failed 1,049-case capture
and every expected/observed pair. No compiler batch or passed durability/process
case needs another execution for this data correction.

Separately retain the completed capture's generated Store slot observations.
Validate bounded catalog framing, immutable historical samples, current native
identity filtering and the generated nextest configuration. Timing remains
telemetry. Commit snapshot correction and measured policy separately. Obtain
fresh independent review of these changes and the affected evidence; prior
source approval does not cover the generated catalog. A comparable full cold
landing and the ranked runtime/setup/generic residue remain separate work.

### B3 affected proof closeout

Commits `9195629f86` and `083ab6f50f` complete the preceding correction.
The existing public reader audits all 136 captured snapshot fixtures with zero
mismatches. Nine actual court cases pass: the eight failed groups and the
production certification-absence control. No Rust, fixture, audience, stage,
manifest, lockfile or renderer changed. The original 1,049-case capture remains
1,041 passes and eight failures; its later repair is separate evidence.
All prior source/sample/baseline values survive in the measured catalog:
5,824 samples, 66 current rules and 451 retired identities. Independent review
approves the exact `465478e955..083ab6f50f` range without findings. Raw evidence
and the review are under `target/test-cost/b3-pool-integration/`.
This closes the affected correction, not a whole cold landing or Milestone 3.

### B4.0 cadence cutover: superseding owner decision

The latest user decision in `orchestration/m3/track_b.md` supersedes the prior
B4 loop/freshness proposal and the intermediate per-edit/checkpoint proposal.
There will be ZERO automatic boundary/constitution checks after writes, edits,
patches, shell calls or batches, including real source changes. A planned phase
completion and landing explicitly run the existing full checks. Individual
edits and tool calls are not phases. Broader iteration commands are deferred;
no classifier, dirty tracking, debounce, watcher, cache or preflight replaces
this automatic hook. The superseded proposal remains in Git history only.

Boundary review: both `.claude/settings.json` and `.codex/hooks.json` actively
register the same constitution PostToolUse command. The dedicated
`scripts/check-constitution-post-tool-use.ps1` has no other live repository
caller in the inspected hook/script/workflow paths. User/local Claude and
Codex settings contain no equivalent WORTH entry; active Git-hook inspection
found none. Unrelated setting values were not exposed. SessionStart preparation
builds the tools but does not execute boundary/context validation; preserve it.
The explicit `scripts/check-constitution.ps1` and CI landing invocation remain
the gate. Real launcher testing exposed a readiness bug: JSON emitted into the
PowerShell return pipeline made a missing-tool result truthy. Emit that failure
through standard output without contaminating the boolean readiness result;
retain the typed refusal and existing diagnostic projection.

Authorized files: remove only the identified PostToolUse registrations in the
two repository hook configurations; retire their unused dedicated script; fix
the launcher readiness-output seam; add focused cadence/launcher tests; update
this plan. No checker rule/configuration,
profile, registry, product, constructor or Phase 3 file changes are needed.

Implementation and acceptance:

1. Remove the two constitution registrations, preserving other hooks/settings.
   Do not replace them with automatic checks of any kind. Delete the dedicated
   post-tool script after the caller audit; retain preparation and explicit gate.
2. Test actual event wiring with invocation counters: writes, edits, multi-edits,
   patches and known/unknown shell calls each cause zero automatic constitution
   invocations. Preserve unrelated hooks and existing session preparation.
3. Run the real prepared checker and context executable through the unchanged
   explicit launcher on an isolated Git/Cargo fixture. Its valid counterpart
   must pass; an actual governed source violation and forbidden dependency must
   fail for their intended diagnostics, not missing inputs or mocked exits.
   Check generated-context refusal and prepared-tool failure handling as well.
   Reuse existing executables; no fixture compilation or new Cargo target tree.
4. Run the full explicit repository boundary/generated-context checks only at
   this planned slice's completion, plus required focused checks and independent
   review. Record invocation counts and elapsed observations as telemetry.
   Do not replay Store suites. B3/B3+ runtime/setup/generic work and the missing
   comparable cold landing remain separate; they do not block this cutover.

### Subsequent independently landable B4 slices

The current parent Track B brief additionally authorizes B4.1-B4.6. Land the
verified B4.0 cutover first; do not hold coherent slices for all of B4 or a new
full cold campaign. The fresh independent reviewer policy is unchanged.

| Slice | Boundary and first acceptance journey |
|---|---|
| B4.1 worker instructions | Update the existing external preamble/active callers in place: checkout pin and committed artifact configuration, available runners, no blanket serialization/cache deletion or per-module mandatory checks. Preserve reviewer policy. Prose-only evidence; external edits are not a Git landing. |
| B4.2 selection before build | Trace query_gate/CostGate/run_lane/nextest_build. Derive requested owner/binary scope before prebuild; reuse existing Cargo ownership. Prove one real owner target and required dependencies build while unrelated courts do not, with identical intended audience/features/profile. Full qualification retains full inventory/coverage. |
| B4.3 build identity | Trace application_build UUID issuance and shared testkit consumers. Agree a concrete write scope with Phase 3 before constructor/testkit changes. Preserve new application-build identity while reusing an issued verification-session/artifact identity lawfully; never substitute a global constant. |
| B4.4 compiler proof meaning | Use existing structured observations to assert intended denial/API/symbol/fixture and wrong-reason rejection. Preserve audiences, check/codegen and successful counterparts. Keep presentation-contract snapshots; retire incidental prose only with replacement semantic assertions. Reuse real captured outcomes. |
| B4.5 verification tiers | Separate running the checker from its self-tests and route local owner/risk categories with existing selections. Expensive campaigns require material reach or explicit full/scheduled selection. Prove routing and honest partial/full reporting without running matrices to test routing. |
| B4.6 CI | Reuse affected-workspace/dependency authority for scoped jobs and compatible Rust caches. Preserve full/manual qualification and safe cancellation semantics. Coordinate default meanings with B4.5; independent cache work may land earlier. |

No new scheduler, registry, daemon, persistent invalidation protocol or evidence
framework is authorized. Benchmark products/configuration/captures remain frozen
during the coordinator's controlled comparison. B3+ constructor/setup work and
B4.3 require Phase 3 scope agreement. The controlled-comparison timing hold has
been released. B4.0 runtime acceptance uses the focused real-tool fixture and
explicit phase-end checks.

### B4.2 focused native build selection

Boundary review: `CostGate` already owns suite predicates; Cargo metadata owns
package/target identity. Today `nextest_build.build` obtains that metadata but
unconditionally builds every workspace target before `SuiteQueries` applies
those predicates. The existing CargoRunner declares/checks required units and
must remain the only compilation boundary. No new ownership registry is needed.
Compiler fixture/executable prerequisites still derive from selected test cases.

The first bounded owner journey exposed a second broad step: the lexical source
census ran before Cargo metadata and spent minutes scanning unrelated Root
sources. It subsequently built the owner target and passed the selected case,
then was interrupted while repeating that broad scan for final aggregation.
Preserve the actual inner build/test/CPU observations and incomplete outer
capture; it is not a completed lane. Resolve metadata first, then check the requested packages and their
resolved transitive workspace dependencies before the only compiling command.
Check dependency testkits too; missing dependency resolution refuses. The final
census uses the same explicit source scope. Full qualification still checks all
workspace sources. This is scoped source selection, not a persisted cache or a
replacement enforcement lane.

Plan: resolve requested gate predicates against Cargo metadata before nextest's
binary prebuild; a library-only owner uses its package and `--lib`, other gate
requests use only their selected packages. Explicit full landing keeps the
existing workspace/all-target inventory and complete membership proof. Provide
the native runner an explicit package/target selection for direct owner requests;
an arbitrary case filter alone cannot infer a Cargo owner and retains its
existing build scope. Preserve default features, committed profiles/configuration
and the pre-execution unit graph declaration. Scope receipts describe selection,
never compiler output hashes. A scoped inventory must remain labeled partial
against the workspace; it must not retire unrelated scheduling identities or
replace the canonical complete census. Existing full qualification is unchanged.

Acceptance: focused command/selection tests, partial/full coverage and retained
history tests, then one real root owner-target build/run showing only that owner
test executable and required dependencies. Use the existing target directory;
no ignored/scale/UI or durability campaign. B4.3 identity and B4.5 default risk
tiering remain separate. Root owns the external B4.1 instruction cleanup.

### B4.3 verification artifact identity

Boundary review: genuine `application_build.py` calls own fresh application-build
issuance. Gate construction currently also issues a fresh UUID, even when a
focused invocation verifies the same existing output artifact. The two named
Query certification testkits embed it; changing it invalidates their compiler
inputs. Existing per-workspace `build/binaries-build-units.json` receipts already
carry the issued input. There is no need for another persisted identity record,
registry, constructor parameter or product/runtime environment door.

Root and Phase 3 agreed the Python issuance/session scope and named testkit
files are disjoint; this implementation needs no Rust edit. The reserved Host,
Execution construction and Store application-home files remain untouched.

Plan: `Gate.plan` resolves selected workspaces before any Cargo work. Without an
explicit caller ID, reuse one valid issued ID only when every requested workspace
artifact carries the same ID. Require the existing compiler-plan schema, the
expected workspace/manifest, native prebuild command and receipt container shape.
Do not add source/configuration/graph revalidation just to choose a descriptive
UUID; those checks remain exclusively in the existing compilation owner.
Missing, malformed, unrelated or mixed receipts keep the one newly
issued verification ID. Explicit caller identity retains precedence and must
parse as a nonzero UUID. Freeze the result within that Gate object.

The logical verification artifact is the existing gate output's per-workspace
native build, across focused invocations in that same output. A different output
starts a new artifact. This input reuse is never source/evidence freshness:
normal source checks, graph declarations, builds and result proofs still run.
Genuine `application_build.py` invocations continue issuing fresh UUIDs.

Write scope: `scripts/ci/query_gate.py`, `query_gate_environment.py`, their owning
Python tests and this plan. Prove coherent reuse, missing/malformed/mixed/unrelated
receipt handling and explicit precedence/refusal with
fixtures; retain the genuine-build issuance tests. Then use one bounded warm
compiler journey to observe repeated unchanged shared support without UUID-only
recompilation. No broad Query/compiler campaign or new target directory. Commit
and independently review this slice separately from the landed B4.2.

### B4.5 first cut: checker execution versus self-tests

Boundary review: the `boundary` command suite currently runs preparation,
constitution validation, the boundary checker's own Rust test suite, and dirty
line caps. The explicit constitution launcher already runs both boundary and
generated-context validation. The affected-lane tool separately selects checker
self-tests when their owning workspace is reached. No checker behavior changes.

First independent cut: move the existing checker-test command, byte-for-byte,
into an explicit `boundary-tests` command suite. `boundary` continues to run
preparation, the real prepared constitution entrypoint and line caps. Prove
command separation and unchanged failure propagation with routing tests using
the real Gate command owner and a recorded fake process boundary. Do not rerun
the checker's own tests merely to prove routing. Run the actual constitution
gate at phase completion, retain full evidence and obtain independent review.
Caller audit: the generic full `ORDER` now selects `boundary-tests` explicitly
once, preserving its prior implementation-test coverage. CI retains its direct
checker-test invocation. The external affected-lane tool retains its checker
workspace's direct Cargo self-test descriptor; source/config changes there still
reach it. Documentation, scheduling data and product changes reach the actual
global constitution check without creating a checker implementation-test reach.
Existing preparation-only SessionStart wiring and explicit terminal/CI launchers
are unchanged. No other generic `query_gate.py boundary` caller was found in the
scoped script/workflow/hook or active orchestration prompt/launcher searches.

The larger local-default cut remains separate. `landing_plan.py` explicitly
requires the Root gate and Store runner defaults to cover every normal partition;
preserve those entrypoints. The proposed generic gate default derives ordinary
owner/integration suites from existing workspace predicates, with named risk
categories and existing full landing paths still explicit. No new risk registry,
scheduler or source classifier. Resolve the concrete local/default reporting
contract before implementing that second cut. The accepted 4,408 Store membership
proof remains scoped to its earlier HEAD; four later admission tests have exact
targeted runtime evidence and do not retroactively enlarge that proof.
