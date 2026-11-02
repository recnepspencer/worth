# Slice 3 certification corrections

Scope: the seven findings on aaab286d3e. No Slice 4 work or types are included.
The same lane reviewer applies qa-loop, qa-tests and code-quality-qa.

## Representation and readings I chose

No new executable, durable, evidence or authority shape is added. Source refusals
retain the exact authorization kind in the existing source-denial enum. Private
concrete teardown slots temporarily own the same backing values at their original
positions around typed registries; they do not recreate state or authority.
Exclusive graph authority and source meanings move once from live Option slots;
shared single-Arc owners retain their existing backing until the correct position.
Empty BTreeMap/route defaults allocate nothing. All ordinary getters remain loans
from the same concrete core. Caller input and public Decl/Host signatures stay typed
and unchanged. No public `'static`, `Send` or `Sync` bound is added.

An immutable comparison such as validate_history_execution_predecessor is not a
live re-entry: it compares carried predecessor proof only, performs no read or
mutation, and returns no authority. Its original None behavior remains. The actual
history-operation entry gates before occurrence admission and head work.

The required order is runtime/brand ownership, installed seal/definition/type,
input type, lapse, then reads/work. Outer transaction application/branch custody
and redispatch handle gates precede that inner admission order. Issuing recovery
authority preserves its existing lack of lapse refusal. Opening funded advancement
is work and therefore follows lapse checking; the original report is corrected.

## Corrections and independent evidence

1. Transaction commit removes both redundant pre-checks. Two literal owner courts
   exercise fresh and authentication-expired programs against foreign runtimes and
   wrong branches. Actual authorization expiry is independently observed before
   asserting ApplicationMismatch and BranchMismatch.
2. History validation, checked source binding, source preparation and pending source
   consumption gate before source/model/head/retention work and admission mutation.
   Real query output and history source/model custody are retained. Literal stale,
   input and cancellation refusals plus unchanged partition and independent consult,
   charging and fact counters prove the early boundary; fresh controls reach the
   intended work. The access owner in publication_boundary mints its own access
   and runs the complete constraint-owned scenario with a loan. No issuer or
   allowlist is widened.
3. Redispatch checks actual handle freshness/recovery posture before admission.
   A real committed-outbox handle expires under the owned clock on a cancelled
   admission and returns literal FreshAuthority(Expired).
4. Both recovery issuance paths keep ownership/seal/type and remove the added lapse
   check. A cancelled admission issues usable fresh authority for its real live
   handle; this does not authorize an effect after cancellation.
5. A cancelled commit produces zero funded-advancement observations and zero new
   reads. A genuine fresh control opens advancement and performs reads. The observer
   records even refused advancement openings, independently of admission policy.
6. Execution drops first; installed schema and concrete prefix/bridge follow; live
   conditionals precede provider/authority/source/transport/inbound/recovery custody;
   handlers, projection and producers precede output custody; workflow coverage
   remains last. The owner court records actual provider and handler destruction,
   including a provider-destructor panic with the original payload. This is not a
   tested bridge-close panic: exact field/slot order and pre-cleanup staging prove
   that remaining unwind seam structurally. No fake bridge hook was added.
7. Genuine foreign-installed seals exercise commit validation, migration, redispatch,
   managed recovery through the actual discovered World-retained interrupted owner row,
   workflow definition, consuming elevation-request readmission,
   history work, source binding and pending-source consumption. Each court targets
   its local gate; the central comparison remains intact in these mutations.

## Verification

The complete integrated correction source is the checked tree. During the full
Execution run, an extra explicit formatter found two touched mounted test files
that Cargo fmt did not traverse. Their only subsequent edits split retained field
paths across lines; the complete non-whitespace bytes were compared and identical.
The restored forgery run and final Clippy rebuild after that formatting. No court,
policy or Rust token changed during the broad run.
All evidence applies to the integrated correction tree, not independently rebuilt
intermediate commit SHAs. Per-finding commits are audited as causally self-contained.

## Development ledger

Final Python verification runs set PYTHONUTF8=1; all Query runs set the required all-ones build ID
and use CARGO_TARGET_DIR=C:/worth-build-cache/target-cut-pub-mono. Cargo output goes
to target/pub-mono-logs and each background process is awaited. No workspace build
or test ran. Failed development probes are reported separately from final gates.

Before work the named sync branches were fetched/merged. The conflict-free merge
9f48ef7bb3 used Git's default merge message; it lacks the required coauthor/footer.
That is a message-rule deviation. It was not amended, and correction commits use
the specified full-sentence messages and coauthor.

## Per-call-site mutation record

The shared installed comparison in authorization/operation_admission/installed_reentry.rs
has an empty diff. Eight independent caller omissions share one build; they are
not represented as eight separately built mutants. Commit validation is mutated
separately because its helper is also used by consuming lifecycle readmission.
Every test obtains genuine custody before altering its admission seal. Setup for
one case does not depend on another removed gate: pending source binding prepares
a genuine admission before corruption, and the managed interruption commits a
genuine unaltered program while the central commit gate remains intact.

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib foreign_installed_seal -- --test-threads=1
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib foreign_installed_seal_is_stale_at_commit_validation_before_work -- --test-threads=1
```

First mutant: exit 101, 6 passed, 8 failed, 0 ignored/measured, 2244 filtered, 4.11 s;
build 59.57 s. Separate commit mutant: exit 101, 0 passed, 1 failed,
0 ignored/measured, 2257 filtered, 0.25 s; build 1 m 02 s. Both compiled with zero
errors. The managed fixture's deliberate World unwind is caught by the ordinary
application boundary; the mutation failure is the later literal seal-refusal probe,
not that setup unwind. Finally blocks restore exact original bytes with fresh
mtimes. The same reviewer checked all nine hashes and the intended failure probes.

| Local gate omitted | Court failure with its gate removed |
| --- | --- |
| Commit core validate_commit_installation call | Local validation returns Ok(()). |
| Migration program.validate_installed_entry | Expected stale candidate refusal is absent. |
| Redispatch admission.validate_installed_operation | Reaches Err(TransportNotInstalled). |
| Managed recovery delivery admission.validate_installed_operation | Reaches the owner recovery path instead of the authorization refusal. |
| Workflow definition admission.validate_installed_entry | Reaches the separate binding gate instead of the projection authorization refusal. |
| Consuming elevation-request readmission validate_commit_program | Refusal is absent; the stale program returns through readmission. |
| History admitted.validate_installed_operation | History validation returns Ok(()). |
| Checked source-binding validate_source_admission | Binding returns Ok(()). |
| Pending source consumption validate_source_admission | Consumption returns Ok(()). |

After the access-owner harness correction, the history-only local-gate mutant
was repeated: compile succeeded in 4 m 19 s, exit101, 0 passed/1 failed,
0 ignored/measured, 2,257 filtered, 1.21 s. Its exact failure is
`a stale installed seal must precede history work: Ok(())`. Original and restored
production-file hashes match; the shared comparator remains unchanged.

The exact omitted spans and restoration SHA256 values are retained in
mono-runtime-core-corrections-mutations.json. Hashes describe bytes at mutation
restoration in this Windows working tree; Git stores its normal normalized text.

## Closed development probes

These are failures encountered during development, not passing gates:

| Probe prefix in target/pub-mono-logs | Result/output | Closure |
| --- | --- | --- |
| slice3-correction1 | Owner compile101,1E0308; zero tests ran | on_branch receives the real branch selector, not its identity. |
| slice3-correction1-fixed | 2passed,2241filtered,2.50s; subsequent fmt-check1 | Final integrated checks supersede the unfinished-draft formatting check. |
| slice3-corrections-preflight | fmt1: mounted cancellation module could not be resolved | Explicit owner-relative path mounts. |
| slice3-corrections-preflight2 | Clippy101,5errors:2E0599,3E0596; zero tests ran | Exact Stopped(Cancelled) variant and owned mutable execution loan. |
| slice3-corrections-preflight3 | Clippy101,6errors:3E0432,E0603,E0308,E0716; zero tests ran | Correct owner imports, genuine observation lease and non-temporary request borrow. |
| slice3-corrections-preflight4 | Clippy101,2errors:E0308,E0716; zero tests ran | The remaining lease/request fixture corrections. |
| slice3-corrections-entry | 13passed,1setup failure,2244filtered,0.83s | Snapshot fault actually yields Indeterminate(CommitRecoveryRequired,ActiveSnapshotCapacityExhausted), so use the real post-effect World interruption/readmitted row. |
| slice3-corrections-entry-fixed | 14passed,2244filtered,0.82s; Clippy101,err_expect+let_and_return | Direct Result matching and direct expression return; no allowances. |
| slice3-corrections-final-boundary | exit1, BC2001_BAND_DEPENDENCY_VIOLATION: history court named program_publication_access outside the access owner or exact registered native courts | Mint access in publication_boundary owner; loan it to the complete constraint scenario. No raw accessor forwarder or allowlist expansion. |
| slice3-corrections-rustfmt-all-touched | explicit rustfmt-check1 on55files;2mounted files need field-chain wrapping | Whitespace-only formatting; all non-whitespace bytes identical. |

The selected successful custody run passes 93 tests, 0 failed/ignored/measured,
2165 filtered, 8.01 s. The actual provider/handler teardown run passes 1 test,
0 failed/ignored/measured, 2257 filtered, 0.46 s; its two scenarios cover normal
and provider-destructor-panic teardown. The installation final run passes 371,
0 failed/ignored/measured/filtered, 3.02 s. The complete final checks are:

| Check (Query commands use --manifest-path workspaces/worth-query/Cargo.toml) | Result |
| --- | --- |
| cargo test -p worth-query-execution --lib | 2,257 passed; 0 failed; 1 existing ignored; 0 measured/filtered; 666.49 s. |
| cargo test -p worth-query-execution --lib erased_core_forgery -- --test-threads=1 | 24 passed; 0 failed/ignored/measured; 2,234 filtered; 6.79 s. |
| cargo test -p worth-query-execution --lib foreign_installed_seal_is_stale_at_history_constraint_before_work -- --test-threads=1 | Final access-owner harness: 1 passed; 0 failed/ignored/measured; 2,257 filtered; 2.11 s; restored build 3 m 20 s. |
| cargo test -p worth-query-court-decl-host --test worth_query_court_decl_host | 325 passed; 0 failed; 2 existing ignored; 0 measured/filtered; 24.43 s. |
| cargo test -p worth-query-court-query-decl-host --test worth_query_court_query_decl_host | 340 passed; 0 failed; 2 existing ignored; 0 measured/filtered; 134.37 s. |
| cargo fmt -p worth-query-execution --check | Exit 0. |
| cargo clippy -p worth-query-execution --all-targets --message-format=json | Exit 0; 87 Execution warning diagnostic emissions, 38 distinct; unchanged from certified Slice 3; 0 new/removed, 0 in touched files. 7 existing Relational warnings. |
| rustfmt --check --edition 2024 --config skip_children=true <every touched Rust file> | Exit 0; all 56 final touched files, including the access-owner scenario. |
| python scripts/quality/scrutinize_rust_functions.py --dirty . | Reviewer ran three times: two scans of 55 Rust files and the final scan of 56 files; each exit 0, 45 advisory candidates, 0 scan errors. Relevant composition review passed. |

The full suite preceded only a test-harness correction: the history scenario's
access is now minted by its owning publication_boundary test and loaned to the
same complete constraint assertions. Production logic is unchanged. Focused
restored and local-gate mutation checks certify that final harness below.

Final repository gates all exit 0:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
scripts/ci/check_workspace_rust_line_caps.sh dirty
git diff --check
git diff --check origin/master...HEAD
```

Boundary output is `repository boundaries are valid`; context reports no failure;
the dirty cap checks 56 Rust files. The recorded 37-command runner ledger, exact argument
arrays, test result lines and failed-development output are retained in
mono-runtime-core-corrections-checks.json, alongside the explicit formatter and
reviewer advisory records. Per-call-site intentional failures are separately
retained in the mutation record. No workspace-wide build or test ran.
The final source-scope audit checks all 61 changed/new files against the lane's
Execution/docs ownership and confirms the seven reserved causes, Decl/Host surfaces
and build configuration have no correction diff.

The reviewer caught the incomplete conditional-cleanup unwind order during review;
all concrete post-conditional custody is now staged before cleanup can unwind.
The reviewer independently verified actual handler/provider teardown, local mutant
failure probes and restoration hashes. Measurement wording and the current Slice 2
ranking annotations were corrected. The final access-owner court preserves the
publication boundary and passes its repeated local mutation check.
Final independent approval: each of findings 1 through 7 passes qa-loop, qa-tests
and code-quality-qa. The same reviewer approves the seven causally self-contained
commits in that order. No implementation, test or measurement finding remains
open. This series stops before Slice 4.

## Measurement after the corrections

**The method sees a regression, not a saving.** The corrected current tree adds
31 exported symbols, 160,056 archive bytes and 53,518 physical code bytes against
certified Slice 3. Against Slice 2 it adds 153 symbols, 341,034 archive bytes and 72,702 code bytes.
The archive selection and scan preceded the final cfg(test)-only history-court
owner relocation. That later edit changes only unit-test placement, with no change
to production logic or the measured application entries. These numbers describe
the recorded archive, not a newly scanned archive after that harness edit.
These are measured whole-tree totals, including the 22 synchronized upstream files;
they are not isolated causal attribution to this correction series. No saving is
claimed or subtracted for unchanged generic source bodies or family-name churn.

The required full Decl/Host check already built the canonical fixture. The
artifact command below then finished in 1.40 s with cached artifacts; it introduced
no additional large fixture build. The unchanged COFF instrument independently
agrees with llvm-nm on exported-symbol totals and llvm-size on physical code bytes.
Both historical raw-report hashes match the committed Slice 3 measurement record.

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host --no-run --message-format=json
python scripts/quality/query_monomorphization.py --cargo-messages target/pub-mono-logs/slice3-corrections-measure-court.jsonl --package worth-query-testkit-certification-canonical --llvm-bin C:/Users/Esther/.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/bin --output target/pub-mono-logs/mono-slice3-corrections-certification-canonical.json
```

Both commands exit 0. The profile is unchanged: opt_level=0, debuginfo=0,
debug assertions and overflow checks enabled. The fixture is
worth-query-testkit-certification-canonical, the same fixture as both baselines.

| Metric | Slice 2 | Certified Slice 3 | Corrected tree | Delta from certified Slice 3 |
| --- | ---: | ---: | ---: | ---: |
| Exported symbols | 150,134 | 150,256 | 150,287 | +31 |
| Archive bytes | 619,715,596 | 619,896,574 | 620,056,630 | +160,056 |
| Physical code bytes | 44,852,822 | 44,872,006 | 44,925,524 | +53,518 |
| All-owner slope | 1,789,712.916065 | 1,792,398.212355 | 1,797,565.608913 | +5,167.396559 |
| Execution slope | 1,373,834.581767 | 1,375,552.419328 | 1,380,356.120015 | +4,803.700687 |

Slope remains sum(bytes/copies) over normalized families with at least 7 copies,
a single-fixture proxy rather than a measured multi-application derivative. Exact
Fractions, owner/module counts, both baseline deltas and all disappeared families
(up to 40) are retained in mono-runtime-core-corrections-measurement.json. Only 2
normalized families disappear against certified Slice 3: the concrete installed
semantic closure Clone (2,612 bytes / 1 copy), and Global::shrink_impl_runtime (743 bytes / 1 copy).
Six disappear against Slice 2; the JSON lists every one. Absent family bytes are
not net savings. Shared/unattributed bytes stay explicitly reported by the tool.

The largest new family is the typed source-admission forwarding shell. Its body
only borrows the existing core/tag and canonical schema identity and forwards to
WorthQueryApplicationRuntimeCore. The concrete body owns all installed/type/lapse
comparison and AdmissionAuthorization refusal construction. The old unit-only
source-denial enum now carries the exact original authorization kind. Unchanged
generic source functions therefore return a different concrete error representation;
their measured growth is not evidence of a newly duplicated comparison algorithm.
The measurement alone does not separate ABI/codegen effects from upstream changes.
No comparison, installed-view assembly or refusal conversion body was moved into a
new per-operation implementation in this round.

Top 20 positive byte changes against certified Slice 3, in measured order:

| Family | Before bytes/copies | After bytes/copies | Growth | Generic scale and source accounting |
| --- | ---: | ---: | ---: | --- |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts` | 76087/11 | 88231/11 | +12144 | **per application/query marker**. Existing Query-generic fact materialization body; no correction diff. It now returns the exact expanded source refusal representation. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::prepare_application_source_expectation::<>` | 52038/14 | 61894/14 | +9856 | **per mutation binding/operation**. Existing typed source-preparation body plus forwarding gate; comparison and refusal construction live in the concrete Core. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit::<>` | 31648/23 | 40112/23 | +8464 | **per operation**. Existing commit progression; correction diff only borrows the retained graph authority through its concrete live getter. |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::pending::WorthQueryPendingSourceExpectation<>::consume_into::<>` | 21033/9 | 28521/9 | +7488 | **per operation and query marker**. Existing pending source work plus an early thin forwarding call; the new validation/conversion body is concrete. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>` | 66211/23 | 73203/23 | +6992 | **per mutation binding/operation**. Existing source-affinity body, no correction diff; its concrete source error now carries exact authorization causes. |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::install_executable_operation` | 120588/26 | 127244/26 | +6656 | **per operation**. Existing typed installation compiler; no correction diff. The synchronization includes installed contract identity changes. |
| `worth_query_installation::application_schema::installed::WorthQueryInstalledApplicationSchema<>::installed_mutation_binding::<>` | 30900/25 | 36900/25 | +6000 | **per mutation binding/operation**. Existing typed installed-binding loan; no correction diff. The synchronization includes installed contract identity changes. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::bind_checked_source_expectation::<>` | 15705/9 | 21204/9 | +5499 | **per mutation binding/operation**. Existing typed checked binding plus the forwarding gate; no new comparison/conversion policy is duplicated here. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_source_admission::<>` | 0/0 | 3703/23 | +3703 | **per operation**. New thin shell borrows core/tag and schema binding, then calls the non-generic Core. 23 copies / 3,703 bytes are real forwarding cost; no claim of zero generic instructions. |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention` | 34540/11 | 38236/11 | +3696 | **per application/query marker**. Existing source-retention function, no correction diff; concrete source refusal representation changes the representation of its error. |
| `worth_query_installation::application_operation::binding::catalog::compile_application_mutation_catalog::<>` | 16919/7 | 20391/7 | +3472 | **per application/schema**. Existing Schema-generic mutation catalog compiler; no correction diff. Seven copies track schema catalogs. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::installation::assembly::assemble_application_runtime::<>` | 63217/7 | 65793/7 | +2576 | **per application/schema**. Existing typed registry assembly now packages empty concrete destruction custody. All custody Default/take/drop bodies are non-generic. |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization` | 17644/11 | 19756/11 | +2112 | **per application/query marker**. Existing source fact-admission body, no correction diff; concrete source refusal now carries the exact authorization cause. |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::from_installed_schema` | 22568/26 | 24648/26 | +2080 | **per operation**. Existing typed installed-operation entry, no correction diff; private backing/view/retention and comparison policy remain concrete. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::drop` | 105/7 | 1771/7 | +1666 | **per application/schema**. Typed Drop moves execution and calls three concrete custody owners before original typed cleanup. Required typed registry destruction remains per Schema. |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity` | 4965/5 | 6295/5 | +1330 | **per application/query marker**. Existing source-affinity function, no correction diff; returns the exact expanded source refusal. |
| `core::ptr::drop_glue::<>` | 702234/9415 | 703480/9415 | +1246 | **mixed support: per operation and per application**. Normalized drop glue aggregates all concrete value types. Copy count remains9415; its bytes cannot be assigned solely to operation brands. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_history_operation::<>` | 17757/9 | 18909/9 | +1152 | **per operation**. Existing history validation now forwards installed validation before lapse/head work. The installed comparison itself is concrete. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::teardown::ApplicationPostConditionalTeardown::default` | 0/0 | 724/1 | +724 | **non-generic**. One concrete post-conditional custody Default body, emitted once. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::prepare_mutation_handler_plan_with_source_access::<>` | 151712/32 | 152252/32 | +540 | **per mutation binding/operation**. Existing typed handler preparation body, no correction diff; its concrete source refusal now retains authorization causes. |

Top 20 against Slice 2 (current corrected tree; source accounting is recorded for
every row in the measurement JSON):

| Family | Growth bytes | Generic scale |
| --- | ---: | --- |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::from_installed_schema` | +16302 | per operation |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts` | +12144 | per application/query marker |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::execute_mutation_handler_report_with_query_reads::<>` | +10512 | per mutation binding/operation |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::prepare_application_source_expectation::<>` | +9856 | per mutation binding/operation |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit::<>` | +8464 | per operation |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::pending::WorthQueryPendingSourceExpectation<>::consume_into::<>` | +7488 | per operation and query marker |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::install_executable_operation` | +7488 | per operation |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>` | +6992 | per mutation binding/operation |
| `worth_query_installation::application_schema::installed::WorthQueryInstalledApplicationSchema<>::installed_mutation_binding::<>` | +6000 | per mutation binding/operation |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::bind_checked_source_expectation::<>` | +5499 | per mutation binding/operation |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::begin_projected_application_read_attempt_with_request::<>` | +4784 | per operation and query/state |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>` | +4048 | per operation |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_source_admission::<>` | +3703 | per operation |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention` | +3696 | per application/query marker |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::installation::assembly::assemble_application_runtime::<>` | +3696 | per application/schema |
| `worth_query_installation::application_operation::binding::catalog::compile_application_mutation_catalog::<>` | +3472 | per application/schema |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_commit_program::<>` | +3174 | per operation |
| `worth_query_execution::domain_computation::primary_graph::application_installation::construction::in_memory_with_contributions_in_advancement::<>` | +2944 | per application and Contributions |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::resolve_admitted_application_idempotencies::<>` | +2576 | per operation and idempotency binding iterator |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization` | +2112 | per application/query marker |
