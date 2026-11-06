# K1c final review evidence

This ledger records the new certification repair attempts after4adcea7b55.
Long Cargo commands run hidden with logs/exit receipts under
target/k1c-repair-logs. Python uses PYTHONUTF8=1. Query builds use the prescribed
all-ones implementation build ID. No Store binary is authorized or run.

Initial sync fetch and both explicitly messaged merges succeeded; both sync
branches were already current. The supplied independent Execution log reports
2230 passed,8 failed,1 ignored,308.20s. This is external baseline evidence,
not a claim of an agent-run full baseline. All eight failures remain in scope.

## Finding 2: production custody admission

Initial `cargo fmt -p worth-relational`: exit 0. Initial focused Native run
`cargo test -p worth-relational --lib storage::residency::kind_admission::tests:: -- --nocapture`:
4 passed, 0 failed/ignored/measured, 2274 filtered, 0.00s (compile 1m10s).
The subsequent fixture refinement replaces an invented fallback cause with a
loud positive-fixture assertion; no production refusal is changed.

`cargo fmt -p worth-relational` rerun: exit 0. Compiled mutant changes the real
admit prior selection to `prior.filter(|_| false)`. Exact private custody test:
0 passed, 1 failed, 2277 filtered, 0.00s (compile 2m04s), exit 101. Failure is
literal allocation identity inequality at kind_admission_tests.rs:74; a fresh
allocation cannot masquerade as growth of the original ticket. Mutation script
restored source byte-exactly and refreshed its timestamp (receipt exit 0).

Restored focused command: 4 passed, 0 failed/ignored/measured, 2274 filtered,
0.00s (compile 1m43s), exit 0. `cargo fmt --check -p worth-relational`: exit 0.
`cargo clippy -p worth-relational --all-targets`: exit 101, one excluded error,
`returning the result of a let binding from a block` at
mvcc/retained_entity_selection/payload.rs:34, and eight warnings. No action per
ruling; four independent lint-file scope assertions pass, excluding dirty overlap.
Boundary-check exit 0, repository boundaries valid. Agent-context check exit 0.
Dirty line caps exit 0, 4 Rust files checked. Both `git diff --check
origin/master...HEAD` and `git diff --check` exit 0. No full binary is claimed
for this intermediate slice; all four remain required after final source.
`/root/reviewer` independently passed the staged slice under qa-loop, qa-tests and code-quality-qa; no remaining in-scope finding.

## Required sync after finding 2

Finding2 committed as b48cebd529. Fetch advanced master to65e5548d14 (two
incoming authored-retirement/restoration commits). Master merge produced six
content conflicts. They are resolved additively with the actual29 mandatory
wire fields plus optional authored-index block. Provider-lifecycle merge was
attempted too early while conflicts remained and refused; it must be rerun.

Initial Native `cargo test -p worth-relational --lib
history::commit::envelope_residency_tests::` failed to compile, exit101, two
errors: E0592 duplicate lineage_event_ids after upstream moved it, and E0063
missing authored_cascade_retirement/restoration in K1c controlled decode.
The duplicate parent accessor was removed; both new seed fields now have
explicit decoding, duplicate rejection and mandatory missing-field checks.
No defaults erase them. Native and Query formatting initial commands exited0.
A corrected Native focused run remains required before merge commit.

## Reduced remaining round: scoped evidence

The orchestrator kept b48cebd529 and reduced the remainder to one commit.
No dropped implementation was partial: InlineBytes ceiling, duplicated nested
precedence, and Invalid/control-shell naming have no edits in this round.
The provider sync branch is already an ancestor of HEAD (merge-base exit0).
The master merge is included in the one remaining commit, not committed alone.

Intermediate sync Query invariant-failure filter: 6 passed,0 failed/ignored,
2235 filtered,0.00s (compile7m48s). Intermediate Query all-target Clippy exit0,
44 lib-test warnings (42 duplicates). Native Clippy exit101: the excluded
payload.rs:34 let_and_return error and8 warnings. Intermediate boundary/context
both exit0; dirty caps140 Rust files, exit0; both whitespace checks exit0.
These intermediate checks do not replace final-source qualification.
The queued merge-only Native rerun was cancelled before it had any Cargo child
(receipt125); no active build was killed. It is superseded by the scoped Native
runs and final full lib. No Store binary has run.

Root fmt (Foundation, Native, Bridge, both affected Foundation courts, Native
certification), Query fmt (Query, Execution, Archive, installed-operation fixture)
and Store aspect-native fmt each exit0. The latter formats the single allowed
fingerprint compile fix; it is not a Store build/test.

`cargo test -p worth-foundational --doc AspectShape`: initial compile-fail fence
1 passed,0 failed,3 filtered; separate merged-doc harness0 tests,5 filtered,
exit0 (compile11.09s, fence0.13s). Restoring the deleted Foundational form and its
old arms/builders makes the same fence fail:0 passed,1 failed,3 filtered,0.46s,
exit101 (compile9.64s). Literal compiler oracle: `Test compiled successfully,
but it is marked compile_fail.` Source restored byte-exactly (receipt0).
Restored command:1 passed,0 failed,3 filtered,0.07s; merged harness0/5 filtered,
exit0 (compile7.19s). This is the API soundness mutant; the compiler, not an
implementation mirror, certifies absence of the old form.

Native normalization filter:5 passed,0 failed/ignored,2276 filtered,0.00s,
exit0 (compile3m12s). Includes the real-admission custody tests and new digest
callback court. Symbol-page filter:1 passed,0 failed/ignored,2280 filtered,
0.00s,exit0 (compile0.90s). These ordinary typed-cause courts use literal
SourceUnavailable with calls[(0,65536),(1,0)] and Content::SizeLimit{97,31};
the private callback does not pretend to return SDK WorkExhausted. The real
driver's separate first-stop latch preserves that original SDK cause.

First content-court filter:68 passed,1 failed,153 filtered,20.44s,exit101
(compile5.95s). Failure at carried_export.rs:152 was `unwrap_err()` on Ok
RelationalNativeCheckpoint{byte_len:18583}: its corruption stimulus wrote outer
artifact version2, which the merged master now supports. Refresh that stimulus
to retired version1 and the exact expected UnsupportedVersion{found:1,supported:2}.
This does not accept corruption or add an old-format decoder. A version-guard
mutant and restored whole content family remain pending.

Foundational compatibility rejection filter:9 passed,0 failed/ignored,
153 filtered,0.00s,exit0. The deleted-shape-only case is retired; adjacent
scalar-reference rejection remains. Foundational contract-validation filter:
7 passed,0 failed/ignored,102 filtered,0.00s,exit0. Its descriptive digest scalar
fixture claims value-family validation, not Native byte custody.

The four new full-binary logs will use round5-full-* names. Prior final-*.log
files belong to earlier rounds and are not evidence for this source state.

## Ordering and regression probes

Ordering court baseline:1 passed,0 failed/ignored,2241 filtered,0.00s,
compile6m19s,exit0. Compiled mutant changes preparation_stop's
PreEffectDenied to Denied:0 passed,1 failed,2241 filtered,0.00s,
compile1m49s,exit101; literal panic is `pre-transaction retention must refuse
without commit recovery` at preimage_retention/tests.rs:40. Byte-exact restore
receipt0. Restored court:1 passed,0 failed/ignored,2241 filtered,0.00s,
compile1m30s,exit0. Independent oracle is the expected pre-effect enum and exact
resource/content payload, not a duplicate mapper.

First six Query regressions pass, each1 passed,0 failed/ignored,2241 filtered:
real_pending_refusal (0.05s), bootstrap (0.14s), restoration (0.06s),
inbound_completion (0.12s), inbound_publication (0.12s), duplicate_root_work
(11.96s). The first five now literally retain WorkExhausted/partition None
because Native content directory work precedes Head residency. Consumer stages
remain exact. The sixth preserves the attempt's original bill/ceiling and funds
only its separate full-checkpoint observer.

The seventh (authority real-door) was manually interrupted after over eight
minutes with no result. Its test process was verified as the exact owned
worth_query_execution-8bdceb521757d9ce.exe; no Cargo/rustc process was killed.
Cargo receipt is -1; output: `process did not exit successfully ... (exit code:
0xffffffff)` and `test exited abnormally`. It had started1 test with no summary;
this is neither a pass nor an assertion failure. Added progress telemetry to
its existing prefix loop, preserving every assertion, to distinguish a large
sweep from a nonreturning attempt. Diagnostic rerun and eighth regression are
pending. No full binary has run in this round yet.

All nine previously repaired Native regressions pass individually, each1/0/0,
2280 filtered. In brief order their test runtimes are0.41,1.48,0.26,0.31,0.33,
0.40,0.04,0.05,0.03 seconds. Leased-preparation family:15 passed,0 failed/ignored,
207 filtered,3.80s,exit0 (compile6.63s). Exact filter names and invocations are
in round5-known-regressions.ps1 under the ignored logs directory.

The reviewer completed its one source pass under all three QA skills. Code was
clear; its only required finding was two active Foundational guides advertising
the removed API. Both guides were corrected and its finding-specific recheck
closed them. Final gate is pending evidence; no unrelated source re-review.

The eighth Query regression, the mandatory-schema snapshot, passes1/0/0,
2241 filtered,0.37s (compile2m20s). The real authority diagnostic retains
all2,132 unique charged-prefix attempts and every original assertion. Its
only instrumentation is probe/witness count logging. Do not replace this
with a prefix sample: the public denial does not identify every internal
preimage checkpoint, so a smaller sample would miss the later ordering fault.

Authority diagnostic completed:1 passed,0 failed/ignored,2241 filtered,
686.23s (compile5.65s),exit0. All2,132 unique positive-charge prefixes were
attempted, with96 exact DecisionReadSet authority witnesses; no `Aborted` or
unexpected successful below-budget attempt. All eight supplied Query
regressions are now individually green. This long sweep explains the original
silent interval; it was progress, not a nonreturning test.
Refreshed version/size corruption baseline:1 passed,0 failed/ignored,
221 filtered,0.47s (compile1.02s),exit0. Guard-disabled mutant is pending.

Corruption mutant compiled35.25s and failed literally at carried_export.rs:152:
`called Result::unwrap_err() on an Ok value: RelationalNativeCheckpoint
{ byte_len:18583, .. }`. Counts0 passed,1 failed/0 ignored,221 filtered,0.23s,
exit101. It disables only the outer version guard. Source restoration is
byte-exact,receipt0. Restored whole content family:69 passed,0 failed/ignored,
153 filtered,22.87s (compile24.37s),exit0. Oracle is the literal retired wire
version and exact UnsupportedVersion refusal; it does not call the encoder to
compute an expected version or imitate the guard.

Final-run prerequisite checker passes34 exact exit receipts,28 positive test
runs and three compiled failing mutants with source restored and staged.
The initial interrupted authority attempt is explicitly excluded from green
receipts. Four complete binaries are now running sequentially on this final
Rust source state; no workspace-wide or Store test is invoked.

## Final-source full binaries

`cargo test -p worth-relational --lib`:2250 passed,0 failed,31 ignored,
0 measured/filtered,317.38s,exit0 (compile30.18s). No filtered substitute:
all2281 tests in the full lib harness were selected.

`cargo test -p worth-court-relational-certification --test
worth_court_relational_certification`:221 passed,0 failed,1 ignored,
0 measured/filtered,65.59s,exit0 (compile2.14s). Full222-test harness selected.

`cargo test --manifest-path workspaces/worth-query/Cargo.toml -p
worth-query-court-decl-host --test worth_query_court_decl_host`:358 passed,
0 failed,2 ignored,0 measured/filtered,51.96s,exit0 (compile7m16s, including
reported package-cache lock waits). Full360-test public harness selected.

## Full-run discovery: preserve semantic cleanup

The full Execution run reported two additional failures before finishing:
right_record_wrong_field_retention_denial_commits_nothing and
two_field_cross_record_retention_denial_commits_nothing. Their existing real
commit-door assertions require Aborted for malformed semantic retention demands.
The new preparation_stop default was too broad: it made those failures
PreEffectDenied as well as typed content/resource stops. Narrowed the decision
to Some(Ok(typed)) only; None delegates to the existing CommitStop::from abort
and cleanup owner. Cancellation retains its distinct control arm. Existing
real-door expectations are unchanged. The new value court's semantic negative
case now asserts the pre-existing Denied branch. Scoped fmtcheck exit0.

The in-flight full Execution binary contains the pre-correction code and must
finish with its failures recorded. Followup focused courts and affected full
Query binaries will qualify the correction. Native and certification sources
are unchanged, so their complete green runs remain final-source evidence.
This is additional testing justified by a newly discovered regression, not a
repeat of green unaffected binaries. The reviewer recheck is restricted to
this final-evidence finding in the already-reviewed owner.

First full Execution run (pre-correction) ended exit101:2239 passed,2 failed,
1 ignored,0 measured/filtered,698.99s (compile41.84s). Both failures are at the
unchanged preimage_retention.rs:28 `Aborted` assertion. Literal outputs:
`Denied(WorthQueryApplicationCommitDenial { kind:ProviderRejected,
stage:ProviderCommit, detail:Some("recorded inverse demands an unobserved exact
mutated field"), cause:None })` and the same shape with detail
`recorded inverse has several admitted observations for one exact mutated field`.
The authority sweep and the eight supplied regressions passed in that full run.

Reviewer restricted recheck approves the narrow correction, and explicitly
closes its earlier overly broad pre-effect recommendation. It confirmed both
real-door semantic expectations and unchanged-head assertions remain unchanged.
Only the typed-stop branch is re-mutated; no fresh broad review is required.

Final scoped fmtcheck commands pass:Root6 packages,Query4 packages,Store1
already-authorized compile-fix package. Root all-target Clippy exits101:
payload.rs:34 let_and_return,1 error/8 warnings, expressly excluded debt.
Court all-target Clippy --no-deps exits101:8 errors/2 warnings. Four are the
expressly excluded existing diagnostics (public_only.rs:38 int_plus_one;
managed_source_work_account.rs:292 int_plus_one and:104 bool_then;
managed_source/nested.rs:110 err_expect). FOUR additional needless_borrow
errors are incoming master authored_retirement/restoration.rs:58,79,81,106.
They are not claimed to be expressly excluded by the user's prior list.
Independent reviewer proved its indexed blob830b79b26be37d95f648ebcff783e5e259469f70
is exactly origin/master's, with no cached-vs-MERGE_HEAD or working-vs-index
difference. A separate current/index/master value check also passes. Those
four errors are unrelated untouched incoming sync debt under AGENTS; K1c did
not change their signatures or reference handling. Do not expand this lane
into authored-restoration fixture maintenance. Both Clippy runs are FAILED,
not green. Query lint and mechanical final checks remain pending.

Final Query all-target Clippy (worth-query, worth-query-execution,
worth-query-package-archive, worth-query-testkit-installed-operation-fixture)
passes exit0 in4m22s; Execution lib-test reports44 warnings,42 duplicates.
This checks the narrowed semantic correction, not the earlier broad mapper.
All three scoped fmtcheck commands and the additional Execution fmtcheck
pass exit0. No Store test/build or workspace-wide command was run.

Final boundary-check exit0:repository boundaries valid. Agent-context check
exit0. Dirty line caps exit0:151 Rust files checked; existing Foundational
facade.rs719-line allowlist honored. Branch, unstaged and staged whitespace
checks each exit0 (only Git newline conversion warning in unstaged evidence).
Four excluded lint-file scope assertions pass. Additional incoming-court
current/index/master equivalence check passes. All Python checks use PYTHONUTF8=1.
The narrowed correction's focused/mutation/restoration and full Query reruns
start only after these checks, serially, with temporary mutations restored.

Narrowed correction's focused `cargo test --manifest-path
workspaces/worth-query/Cargo.toml -p worth-query-execution --lib
preimage_retention::`:15 passed,0 failed/ignored,2227 filtered,0.35s,
exit0 (compile2m19s). This includes both unchanged real-door semantic refusal
courts and the value-only typed cause/posture court. The typed-only ordering
mutant and restored family are pending; the original broad mapper's mutant
is not being substituted for this corrected-branch evidence.

Corrected typed-stop mutant compiles37.20s, then fails at the exact
preimage_retention/tests.rs:40 pre-effect assertion:0 passed,1 failed,
0 ignored,2241 filtered,0.00s,exit101. It changes only Some(Ok(_)) from
PreEffectDenied to Denied; semantic None remains on the existing protocol
owner. Byte-exact restoration receipt0. Restored focused family is rebuilding.

Restored focused family:15 passed,0 failed/ignored,2227 filtered,0.61s,
exit0 (compile36.04s). Both semantic real-door assertions remain unchanged and
pass. Typed content/resource and cancellation value cells pass. Corrected
full decl-host and Execution reruns now qualify only the affected Query code;
no unchanged Native full rerun or workspace-wide build is added.

Corrected full public rerun: `cargo test --manifest-path
workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test
worth_query_court_decl_host`:358 passed,0 failed,2 ignored,0 measured/filtered,
65.11s,exit0 (compile2m23s). Full360-test harness selected. Corrected full
Execution started all2242 tests (compile3.23s); final result remains pending.

## Readings I chose

Work0 now stops at content-directory work before Head residency, so exact
partition attribution is None; no partition-read evidence is discarded.
The private normalization callback can name StorageUnavailableCause, not SDK
WorkExhausted. Its zero-Work digest stop is therefore SourceUnavailable,
while the existing driver retains the original SDK WorkExhausted separately.
Merged outer artifact version2 is current; writing version1 is the corrupt
retired-version stimulus. The snapshot authority now always emits seven
blocks, including the empty content_formats block. Semantic protocol failures
retain their established abort/cleanup path; only checked read/content stops
use the pre-effect denial arm. No new access door or refusal vocabulary is
invented for these decisions.

Known deferred groups: the InlineBytes ceiling; duplicated nested-stop
precedence; string Invalid refusals and control-shell fallback naming.
Older derived_relations/projected_member_domains defaults and the historical
merge trailer stay untouched. The retained64MiB/8,000,000Work reconstruction
fixture funds actual content/root admission and retained custody work rather
than asserting obsolete cost-free reconstruction figures.

Corrected full Execution: `cargo test --manifest-path
workspaces/worth-query/Cargo.toml -p worth-query-execution --lib`:2241 passed,
0 failed,1 ignored,0 measured/filtered,1027.62s,exit0 (compile3.23s). Full2242
selected; all eight supplied regressions plus both newly discovered semantic
courts pass. Four final-source harness totals are5070 passed,0 failed,
35 ignored; these totals use only corrected public/Execution results.

Final receipt checker passes,exit0. It matches all six full summaries including
the first failed Execution run, exact mechanical receipts, both known failing
lint summaries, compiled corrected-branch mutation, byte-exact restoration,
no unstaged Rust and no unresolved index entries. Most recent normal Execution
rlib is libworth_query_execution-b7016cf434752ce1.rlib,320,186,710 bytes.
The earlier4adcea measurement was319,361,570 bytes; the825,140-byte difference
includes synced upstream changes and is not attributed solely to K1c. The new
preimage mapper and stop classifier are non-generic; no new generic engine
body was added.

## Court ownership and independent oracles

- Private kind admission calls real retained_admission::admit. Independent
  SerialMemoryBudget charge, allocation identity, factory count and final-drop
  accounting catch ignoring prior custody; baseline/restoration4/4 pass.
- Private normalize digest court uses literal callback stop SourceUnavailable
  and exact calls[(0,65536),(1,0)], proving streamed hashing preserves a stop.
  Symbol-page value court uses literal Content::SizeLimit{97,31}, proving exact
  cause transport. Neither computes its expected cause with the mapper.
- Private preimage court calls the production classifier/receiver. Literal
  typed Work/content/cancellation and phase enums certify pre-effect routing;
  literal semantic Denied protects existing cleanup. Checked-stop ordering
  mutants fail; restored family15/15 passes. Existing real-door wrong-field
  and cross-record courts independently require Aborted and unchanged Head.
- Foundational compile-fail fence asks rustc whether AspectShape::Content
  exists. Restoring the variant makes the compiler-based oracle fail; removing
  it again passes. Existing family and compatibility courts retain scalar
  reference checks, without claiming Native admission from shape validation.
- Native certification's direct artifact byte overwrite uses retired literal
  version1 against supported2. It expects literal UnsupportedVersion, never a
  serializer-derived expectation; disabling the decoder guard admits the bad
  frame and fails the court. Restored content family69/69 passes.
- Schema install keeps literal refusal for an unbound top-level reference and
  the positive format-bound Struct witness; the removed shape-only case has
  no replacement decoder. Query real-door regressions check literal refusal
  stage/cause and SDK-funded observer independence. Snapshot expected bytes
  are hand-written seven-block authority data, including content_formats.

No new AspectShape form was added. The obsolete Content form, its builders,
portable/archive representation and match arms were deleted. ContentRef inside
a format-bound Struct remains the content field representation.

Final independent reviewer gate:PASS under qa-loop,qa-tests,code-quality-qa.
The same reviewer independently matched all six raw full summary lines,
including the failed first Execution run, final receipt checker and cached
whitespace; no unstaged Rust or unresolved index remained. Its active-guide
finding and narrowed semantic-cleanup correction are closed. Native/Court
Clippy failures remain reported debt, not passing checks. No remaining
in-scope finding. Appending this evidence and rechecking cached whitespace
requires no new source review. Final staged and unstaged whitespace rechecks
both pass after the ledger update.
