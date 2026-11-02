# Public Query monomorphization lane evidence

The baseline and reproduction commands are in [mono-baseline.md](mono-baseline.md).
All Query Cargo commands below use
`--manifest-path workspaces/worth-query/Cargo.toml` and the environment value
`WORTH_IMPLEMENTATION_BUILD_ID=11111111-1111-1111-1111-111111111111`.
No workspace Cargo build/test, profile change, migration, compatibility path,
push, or amendment was used.

## Slice 0

Commit `a9c20fc4407abfdce5be76a135bceffa9a0fd6b7` clears the lint debt and
implements the owner's ordered source-program identity ruling. The identity is
minted on an actual cache miss, uses checked monotonic allocation, and remains
memory-only. Equality and ordering use it; Hash is removed. Installation uses
BTreeMap. Exhaustion remains an exact typed denial through the consumer boundary.

The final accepted source ran these tests. Each command starts `cargo test -p`
with the package below, then the displayed arguments (and the Query manifest
above where applicable). All runs had zero failures, ignored or measured tests.

| Package | Arguments | Passed | Filtered |
| --- | --- | ---: | ---: |
| worth-relational | `--lib validation::candidate_source::program::bindings::tests` | 5 | 2260 |
| worth-relational | `--lib mvcc::publication::companion::cell::public_cache::tests` | 3 | 2262 |
| worth-relational | `--lib derived_relation::evaluation::memo::` | 7 | 2258 |
| worth-relational | `--lib runtime::state::subsystems::indexing::generation_catalog::incremental_retirement::tests` | 1 | 2264 |
| worth-relational | `--lib branch_transition_excludes_settled_owner_inventory_until_release` | 1 | 2264 |
| worth-execution | `--lib storage::ordered_tree::tests` | 4 | 268 |
| worth-execution | `--lib backend::meter::kernel_receipt::tests` | 7 | 265 |
| worth-proof | `--lib managed_access::realized_footprint::tests` | 2 | 106 |
| worth-query-execution | `--lib domain_computation::primary_graph::application_query::managed_index` | 23 | 2175 |
| worth-query-execution | `--lib source_program_identity_exhaustion` | 2 | 2196 |
| worth-query-installation | `--lib application_query::inspection::tests` | 1 | 362 |
| worth-query-court-query-decl-host | `--test worth_query_court_query_decl_host host::application_runtime_surface` | 5 | 337 |
| worth-query-court-query-decl-host | `--test worth_query_court_query_decl_host dependent_dispatch::` | 14 | 328 |

Total: **75 passed in 13 nonempty runs**. An earlier checkpoint filter omitted
the `history` path segment and selected **0 tests**, with 2265 filtered; the
literal test-name run above corrected it. A provisional 24-test managed-index
run included a later removed mock and is not final-tree evidence.

Formatting passed for Query Execution, Installation, Publication,
testkit-certification-correction and court-query-decl-host, and root Relational,
Execution and Proof, using `cargo fmt --check -p` for those exact packages.
`cargo clippy -p` for the same Query five packages with `--all-targets
--message-format=json` passed with **0 errors, 105 warning diagnostics**;
root Relational/Execution/Proof passed with **0 errors, 16 warnings**.
Warning counts include repeated upstream diagnostics and are not clean-warning
claims. Boundary-check, agent-context, and both working/staged and
`git diff --check origin/master...HEAD` passed. The dirty line-cap guard passed
for **58 Rust files**.

Independent evidence: the source binding court compares literal bind order,
including inverse declaration order and reverse collection insertion; it does
not derive expected ordering from the comparator. It also checks cache equality
and literal terminal counter exhaustion. Query denial tests inject the exact
exhaustion variant and verify its translation. The ordered-tree court uses an
independent standard BTreeSet oracle. Host and dispatch courts exercise genuine
installed routes, proving application surfaces and dependent producer dispatch
still work. Other owner tests protect the touched allocation/custody mechanics.

The reviewer rejected a vector lookup alternative as quadratic. The user's
ordered identity ruling closed that concern with logarithmic lookup. A Query
literal disposal-order mock required forbidden ExecutionRequest test references;
boundary-check reported **BC9002**. The mock and mount were removed. No Query
literal disposal-order court is claimed. Reviewer closure covered all 58 files;
its independent composition scan had 50 advisories and 0 scan errors.

Earlier failed gates were repaired, not suppressed: Execution's initial 46
library/37 library-test lint errors; Installation's two errors; three court fmt
hunks; non-exhaustive denial translation (`E0004`); two redundant closures;
Publication's four type-complexity errors; one correction-fixture collapsible
condition; four court constant-empty-loop errors; and root Proof's one,
Execution's two and Relational's eight library/five library-test lint errors.
The exact lint expectations retained are local immutable-key/equality decisions,
with a reason at the line. No blanket allow was added.

## Required sync and gate preservation

Commit `05a9cb6c8c63500e38eaa27c378c90660440e00e` incorporates master
`9d3e872a30` and provider `64b97c65f2`. The single conflict preserves the new
sealed ordinary source-reader constructor and the direct lint-correct read
expression. The reviewer inspected this resolution and all local gate repairs,
without claiming review of the unrelated 963-file upstream integration.

Clippy first exposed **2 Installation diagnostics**, then **1 Archive error**,
then **76 Execution error diagnostics at 50 unique positions**, then
**16 Publication errors**. Exact failure messages included incompatible MSRV
for `AtomicU64::try_update`, type-complexity, checked `unwrap`, and dropping a
non-Drop owner. Mechanical fixes preserve behavior. The ordinal owner uses
checked CAS under the supported MSRV; a new test checks MAX-1 followed by two
refusals without wrapping. Its lifetime tests also remain intact.

Final command:

```text
cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution -p worth-query-installation -p worth-query-package-archive -p worth-query-publication -p worth-query-court-query-decl-host --all-targets --message-format=json
```

It passed with **0 errors, 116 warning diagnostics**. Fmt passed for the four changed Query owners.
Boundary-check, context, and all diff checks passed; the dirty line-cap guard
passed **890 Rust files**, including the incoming merge input.

Tests on the final repaired source:

| Package | Arguments after `cargo test` and Query manifest | Passed | Filtered |
| --- | --- | ---: | ---: |
| worth-query-execution | `-p worth-query-execution --lib document_history` | 5 | 2207 |
| worth-query-court-decl-host | `-p worth-query-court-decl-host --test worth_query_court_decl_host certification::document_history::group::` | 15 | 312 |

Both had zero failed/ignored/measured tests. The 15 real history-group courts
protect touched publication completion/group/handoff behavior using installed
operations and literal resource/outcome assertions. The conflict's dependent
dispatch court also passed **14/0**, 328 filtered, before the subsequent
mechanical repairs; that timing is not final-tree dispatch evidence.

## Baseline slice

Commit `2fee93f1a8` adds the record and instrument. The 11 Python owner tests pass;
the hand-authored reports and deliberately corrupt independent totals provide
an oracle independent of the parser. Boundary-check, context, dirty line caps
(0 Rust files), and diff checks pass. No Rust owner changed, so fmt/Clippy had
no touched package to run. The court no-run build and real archive measurement
both exit 0; independent totals are recorded in the baseline document.

Reviewer findings closed: ambiguous alias ownership is now reported as shared;
nested qualified owners and inherent qualified owners preserve their namespace;
negative/out-of-section coordinates refuse. The reviewer independently reran
the 11 tests and matched every baseline table to the actual report and successful
Cargo artifact.


## Concrete source-meaning slice

Shape: one private `one_shot/outcome/source_meanings.rs` module owns concrete
row/set meaning completion. Its context only borrows the original registry,
selection, execution request, allocation and query identity. Public generic
signatures, typed capture/projection and observation headers remain unchanged.
The interner removes generic error/callback bodies and returns the exact existing
relational Cause. No numerical loop gains dynamic dispatch; the comparator
crosses the existing semantic footprint boundary.

Commands (Query manifest/environment from above):

| Command | Result |
| --- | --- |
| `cargo clippy -p worth-query-execution --all-targets --message-format=json` | 0 errors, 114 warning diagnostics |
| `cargo fmt --check -p worth-query-execution` | exit 0 |
| `cargo test -p worth-query-execution --lib observed_source::source_identity` | 15 passed, 0 failed/ignored/measured, 2197 filtered |
| `cargo test -p worth-query-court-decl-host --test worth_query_court_decl_host --no-run --message-format=json` | exit 0; exact artifact selected for measurement |
| `cargo test -p worth-query-court-decl-host --test worth_query_court_decl_host certification::application_graph::canonical_admission::` | 10 passed, 0 failed/ignored/measured, 317 filtered |
| `cargo test -p worth-query-court-decl-host --test worth_query_court_decl_host certification::application_graph::source_only::` | 30 passed, 0 failed/ignored/measured, 297 filtered |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | exit 0 |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | exit 0 |
| `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 4 Rust files pass |
| `git diff --check` and `git diff --check origin/master...HEAD` | exit 0 |

The source-identity owner court injects literal PolicyMemoryExhausted fields
(requested 97, admitted 53, ancestor 11), requires exact propagation, and checks
that refusal preserves ordinal, retry identity, live coordinates and eventual
removal. This injected expectation is independent of production comparison.
The 15 owner tests also retain row/set distinction and checkpoint/currentness
identity checks. The real canonical admission courts exercise original request
cancellation, swallowed decode refusals, successful reads and recovery. The
source-only courts exercise actual retained source authority, stale/retry,
managed cleanup and capability/resource behavior. Their literal native outcomes,
read-back state and contact/resource counts are external assertions; they do not
copy the moved interner algorithm. No court was weakened.

Reviewer inspection found no material source-meaning defect. It independently
traced the guard/unmatched/retired destruction order, ordinal advancement,
original grant custody, source comparison state extent and unchanged phase
positions. Composition scan: 4 Rust files, 7 advisories, 0 scan errors. The reviewer independently checked the final
measurement and all gates, and approved the slice with no open finding.


## Concrete completed-fact admission slice

Shape: the existing generic entry retains its signature and owns both its
builder and converted iterator. Concrete begin/append/finish functions own the
existing admission/append/seal work. Append borrows a dynamic iterator and
callback; it does not acquire authority or change which fact can be expressed.
The explicit iterator scope preserves conversion after initial admission and
destruction before final checks (also before builder destruction on refusal).

Commands use the same Query manifest/environment:

| Command | Result |
| --- | --- |
| `cargo clippy -p worth-query-execution --all-targets --message-format=json` (first run) | exit 101, one E0423 compile error, 0 tests executed |
| Same Clippy command after correction | exit 0, 0 errors, 114 warning diagnostics |
| `cargo fmt --check -p worth-query-execution` | exit 0 |
| `cargo test -p worth-query-execution --lib read_set::completed_facts::tests` | 3 passed, 0 failed/ignored/measured, 2212 filtered |
| `cargo test -p worth-query-court-decl-host --test worth_query_court_decl_host --no-run --message-format=json` | exit 0; successful artifact selected |
| `cargo test -p worth-query-court-decl-host --test worth_query_court_decl_host certification::application_graph::canonical_admission::` | 10 passed, 0 failed/ignored/measured, 317 filtered |
| `cargo test -p worth-query-court-decl-host --test worth_query_court_decl_host certification::application_graph::source_only::` | 30 passed, 0 failed/ignored/measured, 297 filtered |
| Boundary-check and agent-context commands above | both exit 0 |
| `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 2 Rust files pass |
| Working and origin/master...HEAD diff checks | exit 0 |

The failed run said `error[E0423]: cannot find function denial in this scope`:
a test imported the denial module. It now uses the existing denial constructor;
no visibility change or production repair was needed.

The three new owner tests use independently written literal event sequences,
ordered entity IDs, injected exact denial kind/subject at all four authority
checks, and original cancellation before iterator conversion. These are
SystemAllocation builder mechanics, not a synthetic installed admission claim.
The real installed admission/source-only courts retain their external outcomes,
read-back/resource/contact assertions described above. They were rerun on this
slice without changes. Reviewer independently traced all ownership/check phases;
2 Rust files, 3 advisories, 0 scan errors. It accepted the test-only constructor
correction, independently matched final measurements and gates, and approved
the slice with no open finding.


## Workflow fact emission slice

Shape: two existing workflow formatting owners now borrow a dynamic emitter.
The start entry keeps the original owned subject and emitter, and emits the
created-subject relation only after the shared four record/link effects.
Transition formatting consumes the same earned basis. Error remains exact and
generic; the large bodies no longer specialize on the application callback.
There is no boxed error, mutable error slot, new authority, or allocation.

Commands use the same Query manifest/environment:

| Command | Result |
| --- | --- |
| `cargo clippy -p worth-query-execution --all-targets --message-format=json` | exit 0, 0 errors, 114 warning diagnostics |
| `cargo fmt --check -p worth-query-execution` | exit 0 |
| `cargo test -p worth-query-execution --lib workflow::definition::compilation::reconstruction::start_provenance_tests` | 5 passed, 0 failed/ignored/measured, 2210 filtered |
| `cargo test -p worth-query-court-decl-host --test worth_query_court_decl_host --no-run --message-format=json` | exit 0; successful artifact selected |
| `cargo test -p worth-query-court-decl-host --test worth_query_court_decl_host certification::native_workflow_operations::capability::workflow::` | 2 passed, 0 failed/ignored/measured, 325 filtered |
| Boundary-check and agent-context commands above | both exit 0 |
| `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 2 Rust files pass |
| Working and origin/master...HEAD diff checks | exit 0 |

The five existing owner courts read genuine native provenance for existing and
newly created subjects, distinguish immutable start provenance from mutable
current revision, retain data after original payer drop, and inject corrupted or
missing native truth to require refusal. Those native reconstruction/read checks
are independent of the moved effect formatter. The two installed public courts
check fresh selected capability at workflow completion and captured actor meaning
against the original proposal rather than replacement input. No tests changed.

Review caught the proposed naive callback borrow's owned-subject drop seam
before implementation. The accepted shell retains subject ownership and final
transfer, avoiding an Option/error-slot abstraction. The reviewer independently
traced first-error propagation, effect order and capture-before-subject drop;
2 Rust files, 8 advisories, 0 scan errors. The reviewer independently matched
final measurements, all gates and the remaining ranking, inspected the named
continuation seams, and approved both the slice and the documented marginal
stopping judgment with no open material finding.
