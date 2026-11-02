# Execution retained causes

## Landing and scope

The lane starts at `52e7dba3dc` on `codex/cut-pub-causes-20261101`.
Slice 1 is `c4b61d817783cb8e4f4ee973e414a3df73feb642` (items 1 and 2).
The original continuation is `a26581c5aae28b02da91f5fba9a6d42e1d11e208` (items 3 through 6).
The continuation implements items 3 through 6 and the remaining warnings in
Execution-owned source outside pub-mono. Its completed evidence is recorded below.
Both required sync branches were fetched and merged before each slice; both
merges were already current. The governing external plan was read only.
No consumer in this lane requires a change in pub-mono's reserved files.
The same independent reviewer remains `/root/lane_reviewer` throughout.

## Field dispositions

Every retained field below has outcome **a**. No retained cause or proof field
is deleted. Custody is a real retention/release consumer; names make that role
explicit. Courts P1-P4, C, W, I and N are named in the next section.

| Field | Outcome | Actual consumer | Court |
|---|---|---|---|
| DiagnosticPreparationDenial::Governance.0 | a | Public borrowed Governance refusal | I: diagnostic-preparation governance |
| InspectionAdmissionDenial::Authorization.0 | a | Public exact Authorization refusal | I: query authorization |
| InspectionAdmissionDenial::CurrentAuthorization.0 | a | Public exact CurrentAuthorization refusal | I: current authorization |
| InspectionAdmissionDenial::Capability.0 | a | Public exact Capability refusal | I: capability |
| InspectionAdmissionDenial::Scope.0 | a | Public exact Scope refusal | I: scope |
| InspectionAdmissionDenial::Security.0 | a | Public exact Security refusal | I: security |
| InspectionAdmissionDenial::Governance.0 | a | Public exact Governance refusal | I: governance |
| InspectionCause::Query.0 | a | Same borrowed Authorization projection | I: outer query |
| InspectionCause::Scope.0 | a | Same borrowed Scope projection | I: outer scope |
| InspectionCause::Selection.0 | a | Same borrowed Security projection | I: outer selection |
| InspectionCause::Principal.0 | a | Borrowed Principal refusal | I: principal |
| InspectionBindingDenial::Installation.0 | a | Borrowed Installation refusal | C's genuine uninstalled-binding precondition |
| DiagnosticContractDenial::Contract.0 | a | Borrowed DiagnosticContract refusal | diagnostic_contract_denial_survives_public_inspection |
| InspectionPlanDenial::Graph.0 | a | Borrowed Graph admission refusal | I: graph admission |
| InspectionReportDenial::Input.0 | a | Borrowed ReportingInput refusal | I: reporting input |
| InspectionAdmissionDenial::Session.0 | a | Borrowed exact Session refusal | I: session start |
| InspectionAdmissionDenial::Target.0 | a | ForeignTarget projection preserves the binding mismatch | I: foreign target |
| ProducerOutputEvaluationState::CaptureUnavailable.0 | a | Real require_compared classifies the retained cause into ConnectionRefusal::Capture | I: original capture state |
| ConnectionRefusal::Capture.0 | a | Borrowed ProducerCapture public projection | I: original capture state |
| DecisionQueryPreparationStop::Native.0 | a | HandlerExecutionDenial::native_decision_refusal | N: ordinary decision |
| IndexDecisionStop::Native.0 | a | Same public getter through managed-index error owner | N: managed-index decision |
| OriginalSettledProducerSource::Committed.settlement | a | Real current-settlement reader compares original immutable owner against receipt | P1 |
| OriginalSettledProducerSource::Committed.publication | a | Same reader compares observed publication identity | P1 |
| OriginalSettledProducerSource::Committed.disclosure | a | Same reader compares original disclosure owner, including legitimate absence | P4 |
| OriginalSettledProducerSource::StableReused.settlement | a | Same reader compares actual stable publication's settlement owner | P2 |
| OriginalSettledProducerSource::Restored.checkpoint | a | Same reader compares complete restored checkpoint identity | P3 |
| WorthQueryInspectionReadCompletion.managed_run | a | matches_report compares the report's actual managed run | C |
| WorthQueryInspectionReadCompletion.basis | a | matches_report compares the report's actual basis | C |
| WorthQueryInspectionReadCompletion.basis_release | a | Exact basis identity; local custody validated by sealed completion construction | C |
| WorthQueryInspectionReadCompletion.release | a | Exact owner-minted reservation identity; GraphWork scope/count guaranteed by its private terminal | C |
| MaintainedWatchSelection.association | a | Refresh compares fresh context with retained selection association before poll/ACK | W |
| MaintainedWatchSelection.scope | a | Refresh compares retained scope with watch audience | W |
| MaintainedWatchSelection.native | a | native_custody keeps immutable Native backing alive after row loan ends; release at close | W: separate Weak observation |
| MaintainedWatchSelection.source | a | source_custody retains completed original read | W: separate Weak observation |
| MaintainedWatchSelection.coverage | a | coverage_custody retains admitted coverage | W: separate Weak observation |
| MaintainedWatchSelection.dependencies | a | dependencies_custody retains original dependencies | W: separate Weak observation |
| predicate::tree::Retired.empty_backing | a | empty_backing_custody releases backing outside tree gate | Existing 127-node removal/drop-count courts |
| predicate::tree::Retired.record | a | record_custody releases actual removed record after backing | Same independent drop-count courts |
| RetainedIndexImage::Maintained backing | a | native_custody releases image outside cache gate | Existing maintained-index release/allocation courts |
| predicate::preparation::Child::Refused.round | a | round_custody drops through RoundLink outside Native retirement gate | Existing predicate retirement/allocation courts |
| WorthQueryLiveCommitBatchCell allocation | a | allocation_custody releases after batch payload | Existing live delivery and resource-conservation courts |
| WorthQueryRetiredApplicationAttempt entry | a | entry_custody drops outside attempt gate | Existing application attempt/resource courts |
| WorthQueryRetiredApplicationAttemptPhase phase | a | phase_custody drops outside phase gate | Existing phase/resource courts |

Both requested atomics use explicit compare_exchange_weak loops with Relaxed
success/failure orderings and the original checked arithmetic. Their local
courts check literal starting values, terminal MAX values, refusal without state
change, and the 256-value lineage sequence. The new reservation identity's
counter also refuses exhaustion precisely; six admission paths preserve
IdentityExhausted instead of classifying it as capacity unavailability.

## Courts and independent oracles

- **P1:** real_settlement_seed_preserves_producer_a_and_later_query_basis_b_without_read_custody.
- **P2:** genuine_input_cutoff_retains_stable_supplier_without_native_execution.
- **P3:** tracked_aspect_absence_survives_query_checkpoint_then_invalidates_on_other_field.
- **P4:** native_serial_capture_uses_effective_ancestor_funding_and_releases_its_last_payload.
- **C:** cancelled_after_consuming_completion_cannot_publish_and_keeps_original_payer.
- **W:** retained_watch_coordinates_refuse_refresh_and_keep_original_custody.
- **I:** causal_inspection::application_inspection::stop::owner's literal/pointer courts.
- **N:** handler_denial_preserves_native_decision_preparation_cause and handler_denial_preserves_native_managed_index_decision_cause.

P1-P4 use genuine owner-issued settlements and disclosure from the existing
runtime fixtures. They alter only retained descriptive provenance and call the
actual current-settlement reader through an existing fixture's checking closure.
The independent oracles are deliberately absent publication/settlement facts,
a distinct earlier genuine settlement, an absent/present disclosure substitution (not two present owners), and a
flipped checkpoint idempotency byte. Every corruption must return the existing
typed ForeignSettlement refusal, while each unmodified counterpart succeeds.
The comparison algorithm is not reproduced in a test oracle.

C issues two genuine completions with the same provider label, distinct managed
runs and distinct Native bases. It exchanges one completion fact at a time and
calls the real validate_report entry, requiring ForeignIssuedAccess. The oracle
is the issuance history and independently checked distinct owner identities;
it does not copy matches_report. Session, plan, target and all four requested
retained facts are attacked. A provider label alone is deliberately insufficient.

W keeps the fresh request lawful and exchanges only the stored scope or
association. Each must return typed ForeignContext before availability or ACK;
restoring the original field returns Pending. Four independent Weak observations
check actual backing lifetimes before and after close. The existing prefix-watch
court installs 256 unrelated probes, requires fewer than 128 visited regions for
one changed row, checks full-parent refresh and exact allocation conservation.
No timing threshold is added; elapsed times are telemetry, not correctness.

I and N use independently selected literal owner refusals and exact pointer/full
value assertions. Installation's court obtains a real refusal by requesting an
uninstalled binding from a genuine installed schema; it does not fabricate an
installation error or introduce another runtime fixture. These diagnostic courts
claim projection integrity, not independent end-to-end authorization issuance.

## Composition, shapes and bounded work

No AspectShape form, migration, decoder, shim or alternate representation is
added. The borrowed public WorthQueryApplicationInspectionOwnerRefusal now has
14 diagnostic cases; slice 1 supplied six and this continuation adds the remaining
retained owner causes. Existing diagnostic types become read-only public names
through the owning inspection facade and Host alias. HandlerExecutionDenial adds
the borrowed Native-stop getter. Query Admission owns a new opaque process-local
WorthQueryGraphCapacityReservationIdentity and its release receipt getter; the
identity is provenance, not execution authority, and has no public constructor.
The public Decl/Host compile-reference court checks diagnostic names/re-exports.

Provenance checks compare immutable settlement/disclosure owners in constant time.
Read/publication names and restored producer, role and fact bytes are funded
before semantic comparisons; no unrelated lineage is traversed. The report funds
three times its basis clone bytes plus twice the shared Product branch-name
bytes before cloning and before its two later basis comparisons. Memory retains
only the original clone-byte quote. Arithmetic overflow is a typed refusal.
No cost enters a production API type or signature.

Unused entries_for_entity, predicate_factory, source program and completion
session_identity view methods are deleted because no owner has a consumer for
those facade views; the underlying routing, factory, program and identity remain
consumed by their actual owners. The invalidate_predicates wrapper is cfg(test)
because its only caller is its owner test; production uses its metered owning
service. Visibility of source facts now matches their enclosing domain boundary.
No allow, underscore rename, dummy read or unused getter is introduced.

## Independent review

The same reviewer uses qa-loop, qa-tests and code-quality-qa. It caught and closed:
provider labels being insufficient reservation provenance; unpaid variable-width
provenance/basis comparisons; a new runtime-bearing test helper; a court that
called matches_report without the real validator; missing facade re-exports;
coarse identity-exhaustion classification; and unnecessary changes to untouched
capacity and consumer-value tests. The helper now receives a reader closure,
completion corruption reaches validate_report, identity exhaustion is precise,
facade names are compiled, and both unrelated files are restored exactly.
The reviewer accepted all nine mutation invocations and independently checked their exact assertions and counts. The reviewer independently accepted the final restored-source evidence, diagnostic ownership and this report before committing; no material in-scope QA finding remains.

## Readings I chose

1. The clarified warning ruling reserves admission re-entry, application_runtime*
   and Installation warnings for pub-mono. No needed consumer crosses that scope.
2. In-memory original provenance identifies the immutable owner retained by the
   same seed/settlement. Restored checkpoint identity compares semantically.
   General RecordedSettlementIdentity semantic equality is unchanged.
3. The watch fixture belongs to its standalone consumer_entry manifest; package
   selection uses that manifest with -p, never a workspace-wide command.
4. Untouched physical-source debt stays with its owner under AGENTS.md, including
   the fixture MIN constant that Cargo attributes to Execution through a path
   mount, and the old Admission capacity_tests.rs thread-spawn Clippy error.
   Actual emitted diagnostic counts are reported without claiming literal zero.
5. Custody already existed and automatic Drop was its real consumer. Weak lifetime
   assertions and owner review certify the clarification. The isolated Native=None
   mutation trips the observer's required live-custody precondition; it does not
   demonstrate a post-close Weak-lifetime mutation or four separate field mutations.
6. The footer's package-scoped checks replace wider skill defaults. Its explicitly
   required downstream court binaries are affected proof targets for this lane.

## Verification and mutation ledger

All counts below are **passed/failed/ignored/measured/filtered**. Cargo uses the
lane-isolated `C:/worth-build-cache/target-cut-pub-causes`; boundary/context use
its `-tools` sibling. WORTH_IMPLEMENTATION_BUILD_ID is fixed to
11111111-1111-1111-1111-111111111111. Long Cargo commands run in hidden background
processes with complete logs under `%TEMP%/worth-pub-causes`; none is killed by
a tool timeout. Later final builds use CARGO_BUILD_JOBS=1. No workspace-wide
Cargo build/test or make test runs. Runtime durations are telemetry only.

### Final restored source

| Log stem | Exit | Counts or result |
|---|---:|---|
| `remaining-final-fmt` | 0 | 0 formatting differences |
| `remaining-final-lib` | 0 | 2256/0/1/0/0 |
| `remaining-final-decl-host` | 0 | 326/0/2/0/0 |
| `remaining-final-host-inspection` | 0 | 47/0/0/0/295 |
| `remaining-final-query-inspection` | 0 | 3/0/0/0/61 |
| `remaining-final-admission-clippy` | 101 | worth-query-admission: 1 error diagnostics |
| `remaining-final-court-clippy` | 101 | worth-query-court-decl-host: 64 error diagnostics; worth-query-court-decl-host: 1 warning diagnostics; worth-query-execution: 6 warning diagnostics; worth-query-testkit-document-retention: 1 warning diagnostics; worth-relational: 7 warning diagnostics |
| `remaining-final-execution-clippy` | 0 | worth-query-execution: 13 warning diagnostics; worth-relational: 7 warning diagnostics |
| `remaining-final-consumer-fmt` | 0 | 0 formatting differences |
| `remaining-final-watch` | 0 | 1/0/0/0/268 |
| `remaining-final-watch-growth` | 0 | 1/0/0/0/268 |
| `remaining-final-consumer-clippy` | 0 | worth-query-consumer-root: 129 warning diagnostics; worth-query-execution: 6 warning diagnostics; worth-query-topology-entry: 2 warning diagnostics; worth-relational: 7 warning diagnostics |
| `remaining-final-boundary` | 0 | 0 violations; repository boundaries are valid |
| `remaining-final-context` | 0 | 0 context violations |
| `remaining-final-lines-gitbash` | 0 | 84 Rust files checked; 0 cap violations |
| `remaining-final-diff-head` | 0 | 0 whitespace errors |
| `remaining-final-diff-work` | 0 | 0 whitespace errors |
| `remaining-final-diff-cached` | 0 | 0 whitespace errors |

Exact final commands, in the same order:

```text
cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-admission -p worth-query-execution -p worth-query-court-decl-host --check
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query-decl-host --test worth_query_court_query_decl_host inspection
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query --test worth_query_court_query inspection
cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-admission --all-targets --message-format=json
cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --all-targets --message-format=json
cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets --message-format=json
cargo fmt --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root --check
cargo test --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root --bin worth-query-consumer-root retained_watch_coordinates_refuse_refresh
cargo test --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root --bin worth-query-consumer-root predicate_watch_prefix_refreshes_full_parent_and_prunes_unrelated_probes
cargo clippy --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root --all-targets --message-format=json
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
"C:/Program Files/Git/usr/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty
git diff --check origin/master...HEAD
git diff --check
git diff --cached --check
```

The final library's existing ignored subprocess cleanup probe remains ignored;
its parent supplies the payload. Decl/Host retains its two scheduled large-history
qualification ignores. Neither is claimed as executed. Formatting was applied
to the standalone consumer package before its final check (exit 0, no test run).

### Focused positive baseline

All following commands exited 0 on the same production source, before mutations.
The one-test atomic filter selected only node lineage; the extra run explicitly
selected both atomics and the contract court.

| Log stem | Command suffix after cargo test | Counts |
|---|---|---|
| `positive-producer` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib producer_observation` | 2/0/0/0/2255 |
| `positive-restored` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib tracked_aspect_absence_survives_query_checkpoint` | 1/0/0/0/2256 |
| `positive-disclosure` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib native_serial_capture_uses_effective_ancestor_funding` | 1/0/0/0/2256 |
| `positive-completion` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib cancelled_after_consuming_completion` | 1/0/0/0/2256 |
| `positive-owner` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib causal_inspection::application_inspection::stop::owner` | 17/0/0/0/2240 |
| `positive-native` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib handler_denial_preserves_native` | 2/0/0/0/2255 |
| `positive-atomic` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib refuses_exhaustion_without_reusing_or_wrapping` | 1/0/0/0/2256 |
| `positive-admission-identity` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-admission --lib graph_reservation_identity_exhaustion` | 1/0/0/0/124 |
| `positive-admission-graph` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-admission --lib graph_obligation` | 5/0/0/0/120 |
| `positive-watch` | `--manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root --bin worth-query-consumer-root retained_watch_coordinates_refuse_refresh` | 1/0/0/0/268 |
| `positive-watch-growth` | `--manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root --bin worth-query-consumer-root predicate_watch_prefix_refreshes_full_parent_and_prunes_unrelated_probes` | 1/0/0/0/268 |
| `positive-extra` | `--manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib -- exhaustion_without_reusing_or_wrapping diagnostic_contract_denial_survives_public_inspection` | 3/0/0/0/2254 |

### Mutation results

Each invocation compiled successfully and then exited 101 at the intended test
assertions. Independent filters were combined after Cargo's `--` delimiter where
the same package/binary could serve them; libtest's actual help confirmed multiple
filters. Each failed name and assertion was checked independently. These are not
compiler failures. Source bytes were restored in finally blocks with fresh writes;
no source modification time was restored. The final suite rebuilds restored code.

| Log stem | Counts | Specific lost consumer and assertion |
|---|---|---|
| `mut-A-execution` | 0/8/0/0/2249 | Committed settlement, reused settlement and restored checkpoint accepted corrupted provenance; validator accepted foreign managed_run; both Native getters returned None; both atomics returned Some(MAX) instead of None. |
| `mut-A-watch` | 0/1/0/0/268 | Removed scope guard: coordinates court accepted foreign original scope (expect failed). |
| `mut-B-execution` | 1/2/0/0/2254 | Removed publication and basis comparisons: publication accepted; validator accepted foreign basis. Unmodified reused-settlement control passed. |
| `mut-B-watch` | 0/1/0/0/268 | Removed association guard: coordinates court accepted foreign original association (expect failed). |
| `mut-C-execution` | 0/2/0/0/2255 | Removed disclosure and basis-release identity comparisons: disclosure accepted; validator accepted foreign basis_release. |
| `mut-C-watch-custody` | 0/1/0/0/268 | Native custody=None: observer required live custody and panicked at selection.rs:269. This proves required-slot sensitivity, not a post-close Weak-lifetime mutation. |
| `mut-D-execution` | 0/6/0/0/2251 | Removed actual provenance-reader call: P1/P2/P3 accepted corrupt seeds and P4 stopped at committed-settlement corruption; coarse real capture projection lost exact cause; validator accepted foreign capacity release. |
| `mut-E-execution` | 1/18/0/0/2238 | Shared owner consumer=None: 16 owner positives, contract and genuine installation cause failed; unrelated-stop negative control passed. Installation failure precedes completion exchanges and is not completion sensitivity. |
| `mut-E-identity` | 0/1/0/0/124 | Wrapping reservation mint returned Ok(MAX) where precise exhaustion Err was expected. |

Exact mutation commands:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib -- producer_observation tracked_aspect_absence_survives_query_checkpoint cancelled_after_consuming_completion handler_denial_preserves_native exhaustion_without_reusing_or_wrapping
cargo test --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root --bin worth-query-consumer-root retained_watch_coordinates_refuse_refresh
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib -- producer_observation cancelled_after_consuming_completion
cargo test --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root --bin worth-query-consumer-root retained_watch_coordinates_refuse_refresh
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib -- native_serial_capture_uses_effective_ancestor_funding cancelled_after_consuming_completion
cargo test --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root --bin worth-query-consumer-root retained_watch_coordinates_refuse_refresh
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib -- cancelled_after_consuming_completion producer_observation tracked_aspect_absence_survives_query_checkpoint native_serial_capture_uses_effective_ancestor_funding inspection_preserves_original_capture_state_cause
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib -- causal_inspection::application_inspection::stop::owner diagnostic_contract_denial_survives_public_inspection cancelled_after_consuming_completion
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-admission --lib graph_reservation_identity_exhaustion
```

The nine invocations produce 40 intended assertion failures and 2 passing controls.
Positive W separately observes all four real backings alive after runtime custody
ends and all four dead after closing the last watch. Three other custody fields
were not individually mutated; their independent Weak assertions and owner review
are the evidence claimed. Existing destruction-custody regressions also run in the
full library and existing watch allocation court.

### Earlier candidate checks and failures

These runs are not substituted for the final restored-source checks above.
Slice 1's full historical record remains in commit c4b61d817783cb8e4f4ee973e414a3df73feb642.
Its library passed 2245/0/1/0/0, Decl/Host 325/0/2/0/0, host inspection 47/0/0/0/295
and Query inspection 3/0/0/0/61. Its public-owner=None mutations produced 1/7/0/0/2235
then1/10/0/0/2235; final slice 1 owner courts all passed. Its final Clippy emitted 71
Execution and 7 Relational warnings, 0 errors. Boundary/context/fmt/diff passed;
line caps checked 8 Rust files. An initial overlapping candidate Clippy failed
E0583 (`file not found for module tests`), with 39 Execution and 7 Relational
warnings; that incomplete snapshot was never presented as a final pass.

| Continuation log stem | Exit | Exact result/failure |
|---|---:|---|
| `remaining-first` | 101 | No tests ran; 3 E0277 errors (report resource conversion and two test Debug bounds) |
| `remaining-second` | 101 | 1/1/0/0/2246; assertion failed: disclosure.is_some(); corrected using genuine foreign disclosure |
| `remaining-completion` | 0 | 1/0/0/0/2247 |
| `remaining-watch` | 101 | 0/1/0/0/268; BindingQueryNotInstalled for undeclared alias; corrected using a genuine second world |
| `remaining-clippy` | 101 | worth-query-admission: 1 error diagnostics; worth-query-execution: 3 error diagnostics; worth-query-execution: 16 warning diagnostics; worth-relational: 7 warning diagnostics |
| `remaining-clippy4` | 0 | worth-query-execution: 12 warning diagnostics; worth-relational: 7 warning diagnostics |
| `remaining-clippy5` | 101 | worth-query-admission: 1 error diagnostics; worth-query-execution: 13 warning diagnostics; worth-relational: 7 warning diagnostics |

The first continuation compiler failures preserved no passing claim. Resource
conversion was corrected through the existing Bridge resource wrapper; tests
inspect typed results without requiring unrelated Debug implementations. The
initial Clippy failure included three items_after_test_module errors in changed
files; their modules moved to the end. Its other error is the unchanged Admission
thread-spawn debt below. The first boundary run exited 1 with
BC9002_TEST_TOPOLOGY for a runtime-bearing provenance helper; the helper now takes
a checking closure. Corrected boundary runs 2 and 3 exited 0; context runs 1 and 3
exited 0; dirty cap runs checked 28 then 76 Rust files, both exit0. Intermediate
format applications (remaining-fmt, fmt3, fmt4, fmt5, fmt6, consumer-fmt and
values-fmt) exited 0. The unrelated value constant change was restored exactly.

The first final line-cap command, `bash scripts/ci/check_workspace_rust_line_caps.sh dirty`,
checked 0 files and exited 1 because Windows'
WSL launcher returned `execvpe(/bin/bash) failed: No such file or directory`.
The verified Git Bash rerun executes the unchanged dirty guard and passes 84 files.
No guard or allowlist is bypassed or changed.

### Warning and pre-existing-error ownership

`remaining-final-execution-clippy` unique diagnostics (multiplicity is within this invocation):

- 2x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/progression/retained_read.rs:320`: function `query_execution_denial` is never used.
- 2x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/progression/retained_read.rs:339`: function `work_denial` is never used.
- 2x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/execution/source_readmission/ready.rs:5`: function `ready_work_denial` is never used.
- 2x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/application_query/admission/permission/current_ready.rs:83`: method `seal_current_ready` is never used.
- 2x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/entity_resolution/admitted_scope.rs:25`: method `resolve_encoded_entity_admitted` is never used.
- 1x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/output_lineage/ownership_oracle/../../../../../../worth-query-certification/fixtures/consumer_values/src/integer_orientation.rs:13`: associated constant `MIN` is never used.
- 2x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/navigation/output_correspondence.rs:18`: method `into_original` is never used.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:15`: type `companion::cell::CompanionBranchCellCore` is more private than the item `companion::cell::managed_source::ManagedIndexSourceObservation::core`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:27`: type `companion::cell::CompanionBranchCellCore` is more private than the item `companion::cell::managed_source::ManagedSourceRootImage::core`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:29`: type `companion::cell::CompanionRootImage` is more private than the item `companion::cell::managed_source::ManagedSourceRootImage::image`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:207`: type `companion::cell::CompanionBranchCellCore` is more private than the item `ManagedSourceReadConditions::matches_reserved`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:207`: type `companion::cell::CompanionRootImage` is more private than the item `ManagedSourceReadConditions::matches_reserved`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/validation/candidate_source/authored_history.rs:54`: fields `records` and `incidence` are never read.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/validation/candidate_source/preparation.rs:225`: field `closure` is never read.

`remaining-final-admission-clippy` unique diagnostics (multiplicity is within this invocation):

- 1x **error**, `worth-query-admission`, `crates/worth-query-admission/src/domain_computation/execution_resource_admission/capacity_tests.rs:81`: use of a disallowed method `std::thread::spawn`.

`remaining-final-court-clippy` unique diagnostics (multiplicity is within this invocation):

- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/cell_history/checkpoint_cases.rs:52`: unnecessary use of `clone` to create a slice from a reference.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/capability/grant_commands/mod.rs:40`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/capability/grant_commands/mod.rs:41`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/mod.rs:63`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/mod.rs:64`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/mod.rs:120`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/mod.rs:122`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/retained_capacity/mod.rs:24`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/retained_capacity/mod.rs:27`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/native_postures/mod.rs:59`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/native_postures/mod.rs:60`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/retained/native_postures/mod.rs:20`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/retained/native_postures/mod.rs:21`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/context/handoff.rs:19`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/context/handoff.rs:20`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/context/handoff.rs:57`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/context/handoff.rs:58`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/cancellation.rs:158`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/cancellation.rs:160`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/handoff.rs:58`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/handoff.rs:60`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/handoff.rs:118`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/handoff.rs:120`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/mod.rs:154`: using `clone` on type `CommitId` which implements the `Copy` trait.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/recovery.rs:87`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/recovery.rs:90`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/resources.rs:48`: called `.err().expect()` on a `Result` value.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/cancellation.rs:98`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/cancellation.rs:99`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:45`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:46`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:98`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:99`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:165`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:166`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/recovery.rs:82`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/recovery.rs:83`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:332`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:333`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:337`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:351`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:352`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:356`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/handoff.rs:26`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/handoff.rs:27`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/handoff.rs:60`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/handoff.rs:61`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/capture_race.rs:40`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/capture_race.rs:41`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/handoff.rs:29`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/handoff.rs:31`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/recovery.rs:35`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/recovery.rs:36`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/redo/handoff.rs:35`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/redo/handoff.rs:36`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/retention/cancellation.rs:92`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/retention/cancellation.rs:93`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/retention/cancellation.rs:167`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/retention/cancellation.rs:168`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/native_workflow_operations/capability/grant_commands/mod.rs:40`: use of a disallowed method `std::thread::scope`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/native_workflow_operations/capability/grant_commands/mod.rs:41`: use of a disallowed method `std::thread::Scope::spawn`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/cost/document_history/group.rs:180`: the loop variable `member_index` is only used to index `samples`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/cost/document_history/growth/scenario.rs:72`: manual implementation of `.is_multiple_of()`.
- 1x **error**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/cost/document_history/growth/scenario.rs:217`: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its contained lifetimes.
- 1x **warning**, `worth-query-court-decl-host`, `crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/retention/governed.rs:14`: unused import: `document_history::GovernedRetentionHistory as History`.
- 1x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/progression/retained_read.rs:320`: function `query_execution_denial` is never used.
- 1x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/progression/retained_read.rs:339`: function `work_denial` is never used.
- 1x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/execution/source_readmission/ready.rs:5`: function `ready_work_denial` is never used.
- 1x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/application_query/admission/permission/current_ready.rs:83`: method `seal_current_ready` is never used.
- 1x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/entity_resolution/admitted_scope.rs:25`: method `resolve_encoded_entity_admitted` is never used.
- 1x **warning**, `worth-query-execution`, `crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/navigation/output_correspondence.rs:18`: method `into_original` is never used.
- 1x **warning**, `worth-query-testkit-document-retention`, `crates/worth-query-testkit-document-retention/src/certification/application_graph/cell_history/checkpoint.rs:6`: unused import: `application_schema`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:15`: type `companion::cell::CompanionBranchCellCore` is more private than the item `companion::cell::managed_source::ManagedIndexSourceObservation::core`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:27`: type `companion::cell::CompanionBranchCellCore` is more private than the item `companion::cell::managed_source::ManagedSourceRootImage::core`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:29`: type `companion::cell::CompanionRootImage` is more private than the item `companion::cell::managed_source::ManagedSourceRootImage::image`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:207`: type `companion::cell::CompanionBranchCellCore` is more private than the item `ManagedSourceReadConditions::matches_reserved`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:207`: type `companion::cell::CompanionRootImage` is more private than the item `ManagedSourceReadConditions::matches_reserved`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/validation/candidate_source/authored_history.rs:54`: fields `records` and `incidence` are never read.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/validation/candidate_source/preparation.rs:225`: field `closure` is never read.

`remaining-final-consumer-clippy` unique diagnostics (multiplicity is within this invocation):

- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/authentication.rs:22`: associated function `issued_for_sibling_owner` is never used.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/candidate_index.rs:2`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\mod.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/candidate_index.rs:5`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\directory.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/candidate_index.rs:7`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\host.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/candidate_index.rs:10`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\mutation.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/candidate_index.rs:12`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/candidate_index.rs:14`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\query.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/candidate_index.rs:16`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\schema.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/installation.rs:33`: field `producer_domain_denial` is never read.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/access.rs:4`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\access\contract.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/access.rs:6`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\access\grants.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:2`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\access.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:3`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\affinity.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:5`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\authority.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:6`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\candidate_allocation.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:8`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\coordinates.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:9`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\field.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:10`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\geometry.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:11`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\lifecycle.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:13`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\provider.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:14`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\query.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:15`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\seed.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:16`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\status.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/mod.rs:17`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\summary.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/boxes/provider.rs:12`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\boxes\provider\storage.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/directory.rs:10`: variant `Retire` is never constructed.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/directory.rs:10`: variants `Retire` and `Create` are never constructed.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/host.rs:61`: variant `Incompatible` is never constructed.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/host.rs:82`: function `try_install_prefix_version` is never used.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/../prefix.rs:20`: unused import: `installed_version::IncompatibleVersion`.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/../prefix.rs:49`: field `omit_refinement` is never read.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/../prefix.rs:97`: method `totals` is never used.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/../prefix/cost_stages.rs:18`: method `replace` is never used.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/../prefix/installed_version.rs:4`: struct `IncompatibleVersion` is never constructed.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/../prefix/structure.rs:8`: variants `OmitSource` and `DuplicateSource` are never constructed.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/../prefix/structure.rs:24`: unneeded `return` statement.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/authoring/handler.rs:11`: this `if` statement can be collapsed.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/correction/motion.rs:4`: unused import: `application_schema::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/correction/motion.rs:5`: unused imports: `application_entry::*` and `primary_graph::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/correction/mover.rs:3`: unused import: `application_schema::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/correction/mover.rs:4`: unused imports: `application_entry::*` and `primary_graph::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/correction/note.rs:8`: unused import: `application_schema::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/correction/note.rs:9`: unused imports: `application_entry::*` and `primary_graph::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/correction/recorded.rs:4`: unused imports: `*`, `application_aftermath::*`, and `application_schema::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/correction/state.rs:3`: unused import: `application_schema::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/correction/state.rs:4`: unused import: `worth_query_host::facade::primary_graph::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/court.rs:8`: unused import: `application_contribution`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/court.rs:295`: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its contained lifetimes.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/court/exact_refinement.rs:52`: redundant pattern matching.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/court/prefix_growth/work_attribution.rs:24`: this `if` statement can be collapsed.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/events.rs:2`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\maintained\nested\authoring.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/events.rs:14`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\maintained\nested\query.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/events.rs:18`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\maintained\nested\shape.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/events/composition.rs:2`: unused import: `query`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/events/court.rs:4`: unused import: `schema`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/events/rule.rs:49`: field `scope` is never read.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/generated/indexed/operation.rs:8`: unused import: `*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/generated/indexed/program.rs:6`: unused import: `application_contribution::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/generated/indexed/request.rs:4`: unused import: `*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/generated/indexed/request.rs:63`: struct `Request` is never constructed.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/generated/program.rs:5`: unused import: `application_contribution::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/court.rs:9`: unused import: `self`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/generated/program.rs:5`: unused import: `application_contribution::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/handler.rs:3`: unused import: `application_entry::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/nested/authoring.rs:5`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\maintained\nested\authoring\handler.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/nested/authoring.rs:20`: variants `Name`, `Enabled`, `Link`, `Unlink`, `Share`, and `Retire` are never constructed.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/nested/composition.rs:2`: unused import: `query`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/protected/expectation.rs:5`: unused import: `*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/refinement/mutation.rs:5`: unused import: `*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/rule.rs:2`: unused import: `query`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/rule.rs:7`: unused import: `worth_query_host::facade::application_invariants::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/rule.rs:38`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\maintained\rule\evaluation.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/rule.rs:39`: unused imports: `Record` and `Rule`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/rule/evaluation.rs:3`: unused import: `super::ReservedName`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/rule/evaluation.rs:6`: unused import: `application_schema`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/maintained/source_authorization/expectation.rs:5`: unused imports: `*`, `application_query::*`, and `application_schema::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/mutation.rs:7`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\mutation\handler.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:13`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix\cost_stages.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:16`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix\installed_version.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:18`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix\storage.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:20`: unused import: `installed_version::IncompatibleVersion`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:21`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix\definition.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:23`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix\predicate.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:25`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix\refinement.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:27`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix\structure.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:29`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix\summary.rs`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:31`: file is loaded as a module multiple times: `consumer_root\src\application_invariant_acceptance\managed_index\prefix\text_read.rs`.
- 3x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:49`: field `omit_refinement` is never read.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs:97`: method `totals` is never used.
- 3x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix/cost_stages.rs:18`: method `replace` is never used.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix/installed_version.rs:4`: struct `IncompatibleVersion` is never constructed.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix/structure.rs:8`: variants `OmitSource` and `DuplicateSource` are never constructed.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix/structure.rs:9`: variant `DuplicateSource` is never constructed.
- 4x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/prefix/structure.rs:24`: unneeded `return` statement.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/query.rs:110`: function `source_definition` is never used.
- 2x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/query.rs:127`: function `guarded_source_definition` is never used.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/query.rs:208`: struct `FindPrefixAlias` is never constructed.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/query.rs:221`: function `prefix_alias_scope` is never used.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/refusing/../maintained/rule/evaluation.rs:3`: unused import: `super::ReservedName`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/refusing/../maintained/rule/evaluation.rs:6`: unused import: `application_schema`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/refusing/../maintained/rule/evaluation.rs:25`: fields `scope` and `updates_before_callback` are never read.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/refusing/court.rs:8`: unused import: `admission::authenticated_principal::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/refusing/rule.rs:2`: unused import: `query`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/refusing/rule.rs:6`: unused import: `worth_query_host::facade::application_invariants::*`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/managed_index/refusing/rule.rs:38`: unused imports: `Record` and `Rule`.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/proof/application_program/document_history/aggregate/group.rs:137`: using `clone` on type `CommitId` which implements the `Copy` trait.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/proof/application_program/document_history/aggregate/group/disposal.rs:37`: called `.err().expect()` on a `Result` value.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/proof/application_program/document_history/navigation_outputs/profile.rs:48`: called `.err().expect()` on a `Result` value.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/resources.rs:85`: function `application_limits_with_policy` is never used.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/resources.rs:91`: function `application_limits_with_observations` is never used.
- 1x **warning**, `worth-query-consumer-root`, `consumer_root/src/application_invariant_acceptance/seed.rs:12`: function `seed_cycles` is never used.
- 1x **warning**, `worth-query-execution`, `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/progression/retained_read.rs:320`: function `query_execution_denial` is never used.
- 1x **warning**, `worth-query-execution`, `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/progression/retained_read.rs:339`: function `work_denial` is never used.
- 1x **warning**, `worth-query-execution`, `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/execution/source_readmission/ready.rs:5`: function `ready_work_denial` is never used.
- 1x **warning**, `worth-query-execution`, `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/admission/permission/current_ready.rs:83`: method `seal_current_ready` is never used.
- 1x **warning**, `worth-query-execution`, `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/entity_resolution/admitted_scope.rs:25`: method `resolve_encoded_entity_admitted` is never used.
- 1x **warning**, `worth-query-execution`, `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/navigation/output_correspondence.rs:18`: method `into_original` is never used.
- 1x **warning**, `worth-query-topology-entry`, `topology_entry/src/island_coupling/handler.rs:48`: the `Err`-variant returned from this closure is very large.
- 1x **warning**, `worth-query-topology-entry`, `topology_entry/src/owner_discovery/query.rs:5`: unused import: `application_schema::*`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:15`: type `companion::cell::CompanionBranchCellCore` is more private than the item `companion::cell::managed_source::ManagedIndexSourceObservation::core`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:27`: type `companion::cell::CompanionBranchCellCore` is more private than the item `companion::cell::managed_source::ManagedSourceRootImage::core`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:29`: type `companion::cell::CompanionRootImage` is more private than the item `companion::cell::managed_source::ManagedSourceRootImage::image`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:207`: type `companion::cell::CompanionBranchCellCore` is more private than the item `ManagedSourceReadConditions::matches_reserved`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/mvcc/publication/companion/cell/managed_source.rs:207`: type `companion::cell::CompanionRootImage` is more private than the item `ManagedSourceReadConditions::matches_reserved`.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/validation/candidate_source/authored_history.rs:54`: fields `records` and `incidence` are never read.
- 1x **warning**, `worth-relational`, `crates/worth-relational/src/validation/candidate_source/preparation.rs:225`: field `closure` is never read.


The six Execution functions belong to pub-mono's admission/re-entry work and
are untouched: query_execution_denial, work_denial, ready_work_denial,
seal_current_ready, resolve_encoded_entity_admitted and into_original.
The mounted MIN warning belongs to untouched consumer_values source even though
Cargo labels its library-test diagnostic Execution. There are zero warnings in
other Execution-owned source; no claim of zero total emitted diagnostics is made.
The seven upstream Relational diagnostics are outside this lane.

Admission's all-target Clippy error is pre-existing in untouched
execution_resource_admission/capacity_tests.rs:81:

```text
error: use of a disallowed method std::thread::spawn
Threads and async executors belong only to the reviewed owning sites;
computation belongs to worth-execution.
```

The file was restored exactly after removing an unnecessary new test; the genuine
completion court already proves reservation uniqueness. AGENTS.md explicitly
reports pre-existing failures in untouched files as debt without expanding or
blocking a scoped task. The required failing command is recorded, not called green.
Decl/Host's all-target Clippy command also exits 101 with 64 errors, all in
untouched physical files: 58 disallowed thread methods and one each of
cloned_ref_to_slice_refs, err_expect, clone_on_copy, needless_range_loop,
manual_is_multiple_of and drop_non_drop. Its 15 emitted warnings and all exact
locations are listed above. None is a diagnostic in this slice's changed files.
These untouched court failures follow the same AGENTS.md debt ruling; the full
court binary nevertheless passes independently. No Clippy failure is called green.

Standalone consumer all-target Clippy exits 0 with 144 warning diagnostics:
129 from worth-query-consumer-root, 2 from worth-query-topology-entry, 6 from
Execution's reserved functions, and 7 upstream Relational warnings. The fixture
warnings are pre-existing in untouched physical files; no final diagnostic from
any of the four Clippy invocations points to a changed file.

No item 3-6 consumer required changing a reserved pub-mono file. No scope stop
remains; the listed untouched debt is not hidden or claimed repaired.

## Independent certification follow-ups

The certification review found no blocking production refusal defect in
52e7dba3dc..a26581c5aa, and identified five in-scope corrections. Each correction
is separately reviewed and committed. Python verification sets PYTHONUTF8=1.
The first sync merged origin/master without conflicts in
512c521a514dcc258539394782071bd638ed63f0. Its automatic Git merge message omitted
the required co-author trailer; this is a process mistake, not a compliant commit
message. History is not rewritten. Subsequent merges and fix commits use explicit
full-sentence messages and the required trailer.
The two causes handed over from pub-mono remain deferred until its slice 3;
this round does not modify DecisionQueryPreparationStop::Native or
InspectionPlanDenial::Graph.

### 1. Present, distinct provenance owners

The earlier P4 claim was too strong: a genuine disclosure payload was placed
against an absent expected disclosure. That was not evidence for two present
owners, and the historical description above is corrected.

The settlement court committed_provenance_rejects_present_foreign_settlement_owner
uses two actual producer completions from independently installed runtimes. It
exchanges only the retained descriptive settlement owner, keeping both sides
present, and reaches the real current-settlement reader. Issuance history and
distinct live Arc owners are the independent oracle.

P4 now issues two separate genuine source-inspection capture payloads. Its new
local court calls require_disclosure_owner, the named typed comparison used by
the actual provenance checker. The same-owner positive succeeds; two present,
distinct owners must refuse ForeignSettlement. This is local comparison evidence,
not a claim that a producer receipt with present disclosure was issued. Existing
P4 currentness coverage still checks legitimate absence against foreign present
capture; the previously accepted reader-removal mutation proves that wiring.

An initial opt-in disclosed-producer fixture compiled but failed both tests at
its required present-disclosure precondition (0 passed, 2 failed, 2257 filtered).
Declaring source inspection did not capture it on that producer path. That fixture
was removed completely; no producer implementation was changed to stage a world.
The narrower real-owner comparison court avoids fabricated receipt custody.

The initial corrected positive and final restored runs each pass 2/0/0/0/2256
(passed/failed/ignored/measured/filtered). Weakening same_owner to
left.is_some() == right.is_some() compiles, then fails 0/2/0/0/2256 at precisely
"present foreign committed settlement: ()" and "present foreign disclosure: ()".
The final source restores pointer identity and rebuilds the cleaned harness.
An unexercised present-disclosure seed branch was removed while the mutant build
was active; mutation observations cover the unchanged exercised paths, and the
restored positive/Clippy compile the final harness. No extra production shape is
introduced. Exact per-commit checks follow.


Slice 1 verification (all commands use the lane target/build identity above):

| Exact command | Exit | Result |
|---|---:|---|
| cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib committed_provenance_rejects_present_foreign | 101 | Initial removed fixture: 0/2/0/0/2257; missing real disclosure precondition |
| cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution | 0 | Formatting applied; both candidate invocations |
| cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib -- committed_provenance_rejects_present_foreign native_serial_capture_uses_effective_ancestor_funding | 0 / 101 / 0 | Positive 2/0/0/0/2256; presence-only mutant 0/2/0/0/2256; restored 2/0/0/0/2256 |
| cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --check | 0 | 0 differences |
| cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets --message-format=json | 0 | 13 Execution-labelled warnings, 7 Relational warnings, 0 errors; unchanged ownership inventory above |
| cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root . | 0 | 0 boundary violations |
| cargo run --manifest-path tools/agent-context/Cargo.toml -- check | 0 | 0 context violations |
| "C:/Program Files/Git/usr/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty | 0 | 4 Rust files checked, 0 violations |
| git diff --check origin/master...HEAD | 0 | 0 whitespace errors |
| git diff --check | 0 | 0 whitespace errors |

No new broad runtime worlds were added for disclosure. The same reviewer accepted
the corrected ownership boundary and independently verified the two intended
mutation assertions and restored run; final gate review precedes the commit.
Per-slice focused library courts and footer gates run before each fix commit.
Full Execution library, full Decl/Host and both inspection binaries run at final
round closure; they are not claimed re-run for this first correction.


### 2. Consume the sealed completion proof

Finding 1 landed in d0dde53d7b0a2cfd061165881ed7bfb39dc2e957. Both sync branches
were fetched and merged before this correction; both were already current.

matches_report no longer re-checks basis_release.custody_released(), graph release
scope or released reservation count. complete_inspection_read already validates
local basis custody before constructing its private completion, and the private
WorthQueryReservedGraphProviderCapacity::release constructor always creates a
GraphWork receipt with exactly one released reservation. No other path can
construct the matching owner-issued graph reservation identity. These immutable
construction invariants are not three independently forgeable completion facts.
The matcher still checks the exact basis-release identity and graph reservation
identity, as well as session, managed run, plan, target and basis.

No new court is added for guaranteed shape. The existing genuine-completion C
court remains the independent issuer-history oracle and reaches validate_report.
The reservation-identity removal mutation compiles and fails C at exactly
"foreign release": 0/1/0/0/2257, exit 101. Positive and freshly restored runs
both pass 1/0/0/0/2257, exit 0. All footer gates pass; Clippy retains only the
known 13 Execution-labelled and 7 Relational warnings. The same reviewer independently accepted
these results and this correction before committing. No new shape is added.


Exact s2 commands and results:

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | 0 | Formatting applied |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib cancelled_after_consuming_completion` | 0 | 1/0/0/0/2257 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib cancelled_after_consuming_completion` | 101 | 0/1/0/0/2257 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib cancelled_after_consuming_completion` | 0 | 1/0/0/0/2257 (passed/failed/ignored/measured/filtered) |
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --check` | 0 | 0 formatting differences |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets --message-format=json` | 0 | 13 warning diagnostics from worth-query-execution; 7 warning diagnostics from worth-relational |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | 0 boundary violations |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | 0 context violations |
| `"C:/Program Files/Git/usr/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | 1 Rust files; 0 cap violations |
| `git diff --check origin/master...HEAD` | 0 | 0 whitespace errors |
| `git diff --check` | 0 | 0 whitespace errors |


### 3. Actionable public diagnostics

Finding 2 landed in 614045a007ec32478792ebbd6749b0e3582ad280. Both required sync
branches were fetched and merged before this correction, both already current.

OwnEffectOnReads remains public because its information changes the diagnosis:
its existing real at_own_publication reader is now public, with the already
existing typed FactlessCurrentness answer nameable through Host. Actual rebase
courts read the genuine unmoved/moved effects and assert Current/Superseded;
the undecidable family retains its existing coverage. This is a read-only
conditional diagnostic, with existing runtime consumers and no new authority.

The public RebaseVerificationReason no longer exposes two unusable ordinals.
Its private RebaseVerificationFailure retains both exact positions, and the
existing FullVerificationReason path consumes them against genuine retained facts
in currentness.rs. Only duplicate public positional slots are deleted. Native
fact versus general revision and indexed fact versus general indexed denial stay
distinct public cases; exact indexed denial values also survive. No protected
predicate value, raw fact payload, substitute digest or opaque token is published.

| Field | Outcome | Reason or consumer | Court |
|---|---|---|---|
| OwnEffectOnReads.0 | a | Public existing at_own_publication returns typed FactlessCurrentness, also used by runtime currentness | Genuine failed-rebase effect court and undecidable family |
| Public NativeFactRevisionUnavailable ordinal | b | Private failure/full-verification carrier already resolves the position against retained fact identity; public slot is not an actionable identity | Private native ordinal court retained; public cause-stage court |
| Public IndexedSelectionFactDenied ordinal | b | Same real private fact-reader ownership; preserve public fact-stage and exact indexed refusal | Existing nonzero indexed ordinal court; public indexed cause-stage court |

inspection_capacity_identity stays public: Execution calls it across the Admission
crate boundary to bind a report to its live session before consuming the plan.
The user's conditional narrowing rule therefore does not apply. Its integration
purpose is documented; no wrapper or unused alternative is introduced.

The new Host court installs the existing cell-history program, executes a real
bound query, then requests inspection with a genuinely uninstalled binding. It
reads owner_refusal through Host and checks the literal BindingQueryNotInstalled
cause and exact declared binding subject. It constructs no public refusal enum.
The installation diagnostic and kind become nameable through the same facade.
docs/capability-index.md documents the retained public vocabulary and reader.

The focused library positive run passed 20/0/0/0/2240. The combined public
reader/stage mutant compiled and failed 16/4/0/0/2240: native fact versus general
revision, indexed fact versus general denial, Superseded, and Undecidable.
These are literal expected public outcomes; the genuine effect courts derive
their source histories independently of the public reader under test.

The genuine Host positive run passed 6/0/0/0/323. Removing only Installation's
owner projection compiled and failed 5/1/0/0/323 at exactly "Host must preserve
the original installation cause". The other five inspection courts passed.
This is behavioral facade evidence, beyond the retained name-compilation court.
Saved source bytes are restored before clean verification; final restored runs,
inspection courts and footer results follow below. This correction leaves the
two explicitly deferred pub-mono handover causes unchanged.


Exact s3 commands and results:

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-admission -p worth-query-execution -p worth-query-court-decl-host` | 0 | Formatting applied |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib postcommit_currentness` | 0 | 20/0/0/0/2240 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host inspection` | 0 | 6/0/0/0/323 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib postcommit_currentness` | 101 | 16/4/0/0/2240 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host inspection` | 101 | 5/1/0/0/323 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib postcommit_currentness` | 0 | 20/0/0/0/2240 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host inspection` | 0 | 6/0/0/0/323 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query-decl-host --test worth_query_court_query_decl_host inspection` | 0 | 47/0/0/0/295 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query --test worth_query_court_query inspection` | 0 | 3/0/0/0/61 (passed/failed/ignored/measured/filtered) |
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-admission -p worth-query-execution -p worth-query-court-decl-host --check` | 0 | 0 formatting differences |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-admission --all-targets --message-format=json` | 101 | 1 error diagnostics from worth-query-admission |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets --message-format=json` | 0 | 13 warning diagnostics from worth-query-execution; 7 warning diagnostics from worth-relational |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --all-targets --message-format=json` | 101 | 64 error diagnostics from worth-query-court-decl-host; 1 warning diagnostics from worth-query-court-decl-host; 6 warning diagnostics from worth-query-execution; 1 warning diagnostics from worth-query-testkit-document-retention; 7 warning diagnostics from worth-relational |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | 0 boundary violations |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | 0 context violations |
| `"C:/Program Files/Git/usr/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | 23 Rust files; 0 cap violations |
| `git diff --check origin/master...HEAD` | 0 | 0 whitespace errors |
| `git diff --check` | 0 | 0 whitespace errors |

The fresh restored library and Host runs reproduce their positive counts.
Both inspection binaries pass on this restored source. All new production and
test files are free of Clippy diagnostics. Execution's exact 13/7 warning
ownership inventory above is unchanged.

Admission and Decl/Host all-target Clippy are **failed checks**, not green gates.
Admission emits exactly `error: use of a disallowed method std::thread::spawn`
at untouched execution_resource_admission/capacity_tests.rs:81. Decl/Host emits
64 errors across 25 untouched physical files: 58 disallowed-method errors and
one each of cloned_ref_to_slice_refs, err_expect, clone_on_copy,
needless_range_loop, manual_is_multiple_of and drop_non_drop. Its warning counts
are 7 Relational, 6 reserved Execution, 1 document-retention testkit and 1
untouched court import. The reviewer independently compared primary spans with
the complete tracked/untracked dirty set: zero matches. These are pre-existing
repository debt under AGENTS.md; no exception or suppression is added.

No AspectShape form or authority door is added. The public change is actionable
read-only diagnostics. The same reviewer independently checked source, literal
oracles, all five semantic mutation failures, restored bytes, and test results;
final footer/report gate precedes this separate commit.

The same reviewer issued the final slice-3 PASS under qa-loop, qa-tests and
code-quality-qa after checking all source, evidence, debt and report entries.


### 4. Exhaustive retained-cause projection

Finding 3 landed in 2d8243c6521b7bd16adb33ab2767b7b3d5163f0b. The final
post-report `git diff --check` also passed with zero errors. Both required sync
branches were fetched and merged before this correction; both were current.

Boundary review followed the actual inspection admission, read/terminal,
diagnostic, comparison, source, plan and report owners and their existing
public readers. Every catch-all None arm in owner.rs is removed. An added
variant now requires an explicit projection decision from the compiler.
The private generic domain helper is replaced by an exhaustive capture reader.
Only descriptive denials and the diagnostic stage vocabulary become public;
session, admission and completion proof construction stays private.

The plan preserves exact existing dedicated readers and adds descriptive
projection where no such reader exists. New pure enum matches borrow existing
objects or return fixed cases; they allocate nothing and perform no graph work.
Common kernel/storage failures carry the actual owner stage rather than merge
owners. Report identity and bridge remain distinct stages. Parameter comparison's
unit domain cause means the two original parameter-binding identities differ;
its distinct ComparisonParametersUnavailable case preserves that cause.

| Retained field/discriminant | Outcome | Real consumer | Court |
|---|---|---|---|
| InspectionCause::Advancement | a | owner_refusal borrows the original advancement, including non-resource cases | Original advancement pointer court |
| InspectionAdmissionDenial::Read | a | Read borrows the exact managed graph-read denial | Nine literal read cases, including index identity and native refusal payload, plus pointer identity |
| InspectionAdmissionDenial::Terminal | a | Terminal borrows the exact terminal wrapper and its request/read cause | Both literal terminal cases and pointer identity |
| InspectionAdmissionDenial::DiagnosticAdmission | a | DiagnosticAdmission borrows the exact diagnostic-comparison denial | All six owner cases and pointer identity |
| InspectionCause::Unavailable | a | Distinct Unavailable owner case | Literal unavailable-stage court |
| ForeignIssuedAccess | a | Distinct public owner case | Literal foreign-access court |
| MissingGraph | a | Distinct public owner case | Literal missing-graph court |
| MissingInspection | a | Distinct public owner case | Literal missing-inspection court |
| FacetDenied | a | Distinct public owner case | Literal facet court |
| MissingOriginalProjection | a | Distinct public owner case | Literal missing-projection court |
| DiagnosticContract NotDeclared/UnsupportedShape | a | Separate fixed diagnostic-contract cases | One literal court per distinction |
| ComparisonParameters domain unit | a | Distinct parameter-binding mismatch case | Literal parameter-stage court |
| RichSource UnresolvedField/MissingOriginalValue | a | Separate fixed source-materialization cases | One literal court per distinction |
| Comparison DisclosureInterpretationChanged/OriginalDisclosureIncomplete/NestingExceedsDeclaredShape | a | Separate fixed comparison cases | One literal court per distinction |
| Plan Overflow | a | Distinct plan-accounting case | Literal plan-overflow court |
| Report ForeignComparison/ForeignMaterialization | a | Separate fixed report cases | One literal court per distinction |
| Report Identity Overflow/ForeignSession/ProductRequired | a | Separate fixed report-identity cases | One literal court per distinction |
| Report Bridge AccountingOverflow/InvalidReportingAssociation | a | Separate fixed bridge cases | One literal court per distinction |
| RetainedStorageDenial::Overflow | a | StorageAccountingOverflow carries its exact owner stage | Literal binding/target/storage/contract/plan/comparison/report/identity/bridge and nested canonical route courts |
| Captured MapKernelFailure::Panic | a | KernelPanic carries its exact capture stage | One literal court per actual capture owner |
| Captured MapKernelFailure::ResultCapacityExceeded | a | KernelResultCapacityExceeded carries its exact capture stage | One literal court per actual capture owner |

No retained cause is deleted. The remaining named None branches delegate to
real exact readers: Request/RequestControl to resource_refusal and
request_interruption; Connection General/Output to producer_connection_refusal
and producer_output_refusal; Rich's two domain cases to materialization_refusal;
array/allocation/memory/request/lease/kernel stops to resource_refusal; immutable
canonical Basis/Digest causes to canonical_refusal. Storage overflow is now
projected explicitly rather than mistaken for a delegated memory refusal.

The new tests inject descriptive private denial carriers and read the public
owner projection. Their oracle is the independently written literal public
case/stage and, for borrowed owners, pointer identity with the retained object.
They are projection courts, not claims that fabricated denial inputs constitute
admission, completion, canonical readiness or actual producer receipts.

Readings I chose: a meaningful unit discriminant is information, so naming a
None arm does not justify erasing it. Keep real existing dedicated consumers;
project the other distinctions. The same reviewer approved focused owner and
actual Host tests before this slice's commit, with the full Execution/DeclHost
and both inspection binaries on final slice-5 source, avoiding repeated broad
court compilation for consecutive diagnostic-only changes. All footer gates
still run before every commit. The two deferred pub-mono handover causes remain
unchanged. No AspectShape form is added.

The same reviewer passed source and court review. Completed mutation, restored
runs and footer evidence follows.


Exact slice-4 commands and results:

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | 0 | Formatting applied |
| `"C:/Program Files/Git/usr/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | 17 Rust files; 0 cap violations |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib stop::owner` | 0 | 81/0/0/0/2243 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib stop::owner` | 101 | 16/65/0/0/2243 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib stop::owner` | 0 | 81/0/0/0/2243 (passed/failed/ignored/measured/filtered) |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host inspection` | 0 | 6/0/0/0/323 (passed/failed/ignored/measured/filtered) |


Exact slice-4-footer commands and results:

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --check` | 0 | 0 formatting differences |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets --message-format=json` | 0 | 13 warning diagnostics from worth-query-execution; 7 warning diagnostics from worth-relational |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | 0 boundary violations |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | 0 context violations |
| `"C:/Program Files/Git/usr/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | 17 Rust files; 0 cap violations |
| `git diff --check origin/master...HEAD` | 0 | 0 whitespace errors |
| `git diff --check` | 0 | 0 whitespace errors |

The compiled projection-removal mutation changed exactly one Some consumer for
each of the 29 new public cases into None. Its 65 expected assertion failures
are 39 capture-stage courts, three lifecycle-carrier courts, 22 fixed-stage
courts and the original foreign-access court; 16 controls still pass. Every
mutated production byte was restored. The restored 81 owner courts and six
actual Host inspection courts pass. The oracle is each court's literal public
case/stage or the retained object's pointer, not a copied projection algorithm.

All footer checks exit zero. Execution Clippy retains the same 13 labelled
Execution and seven upstream Relational warning diagnostics inventoried above;
none comes from this slice's source. Six reserved pub-mono methods account for
12 duplicated all-target diagnostics; the thirteenth is the physically external,
path-mounted consumer-values fixture. No suppression or unrelated edit is made.
The dirty cap checks 17 Rust files with zero violations. No hard-rule or scope
stop occurred. Final clearance from the same reviewer follows the ledger.

The same reviewer independently reconciled all footer logs, zero dirty Clippy
primary spans, the mutation manifest and the report, and issued final PASS under
qa-loop, qa-tests and code-quality-qa. This fourth finding is cleared to commit.
