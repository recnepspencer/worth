# Query monomorphization baseline

## Reproduction and scope

Baseline source: `05a9cb6c8c63500e38eaa27c378c90660440e00e` (Slice 0 plus the
required master/provider sync). Windows x86_64-pc-windows-msvc; rustc 1.99.1
(58cf8f9c3 2026-10-13), Cargo 1.99.1 (5f94df478 2026-08-27), LLVM
23.1.1-rust-1.99.1-stable from llvm-tools-preview, Python 3.12.10.

The largest public generic application fixture in the required Decl/Host court's
successful dependency build is `worth-query-testkit-certification-canonical`:
674,508,286 bytes, versus document-retention 307,246,716 and
certification-operation 258,265,094. This is a measured cohort, not an exhaustive
build of every application. The fixture exercises real installation, one-shot
queries, producer and checkpoint paths through the public facades.

Use the repository's existing **test profile** (opt-level 0, debug information 0,
debug assertions and overflow checks enabled; no fixture features). No profile,
role, or compiler setting changed. A preliminary dev-profile archive with line
tables was 994,727,852 bytes; it is not this baseline and is never a savings claim.
The court was already built for required focused tests; the no-run command below
certifies its exact artifact without another large build.

From the repository root in PowerShell (logs directory must exist):

```powershell
$env:WORTH_IMPLEMENTATION_BUILD_ID='11111111-1111-1111-1111-111111111111'
# This session's existing CARGO_TARGET_DIR is C:/worth-build-cache/target-cut-pub-mono.
cargo test --manifest-path workspaces/worth-query/Cargo.toml `
  -p worth-query-court-decl-host --test worth_query_court_decl_host `
  --no-run --message-format=json *> target/pub-mono-logs/baseline-court-build.jsonl
python scripts/quality/query_monomorphization.py `
  --cargo-messages target/pub-mono-logs/baseline-court-build.jsonl `
  --package worth-query-testkit-certification-canonical `
  --llvm-bin C:/Users/Esther/.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/bin `
  --output target/pub-mono-logs/mono-baseline-court.json
python -m unittest discover -s scripts/quality -p test_query_monomorphization.py
```

Run long Cargo commands in the background, wait, and inspect their logs. The
instrument selects the named archive only from successful Cargo JSON output;
it refuses failed builds and ambiguous artifacts. Reuse these commands and this
profile for comparisons. Build logs and complete generated rankings are local
ignored evidence; the committed tables below retain the baseline.

## Baseline totals

| Metric | Baseline |
| --- | ---: |
| Archive bytes | 674,508,286 |
| Defined external symbols | 160,719 |
| COFF objects | 258 |
| Physical code-section bytes | 48,544,077 |
| Unattributed code bytes | 53,757 |
| Mixed-family shared-alias bytes | 0 |

Archive: `debug/deps/libworth_query_testkit_certification_canonical-15d7b7f23c793455.rlib`
in the target directory above. SHA-256:
`1b0e4ae8584201e0b4fe385909436e1025ecceb7eb6545052d88e02b3f3d3fb7`.
Independent LLVM nm and readobj counts agree at 160,719; LLVM size and readobj
code totals agree at 48,544,077 bytes. Disagreement is a loud measurement failure.

## Namespace counts

Namespace ownership is inferred from demangled definitions or qualified Self
types. Reference/tuple/primitive namespaces are retained verbatim rather than
misassigned to a crate. These are defined external archive symbols, not final
DLL exports. The leading crate namespaces are:

| Namespace | Symbols |
| --- | ---: |
| `core` | 69,066 |
| `worth_query_execution` | 46,363 |
| `alloc` | 13,595 |
| `worth_execution` | 10,550 |
| `worth_query_declaration` | 5,766 |
| `worth_query_installation` | 4,052 |
| `worth_query_testkit_certification_canonical` | 2,327 |
| `std` | 1,310 |
| `worth_query_publication` | 1,088 |
| `worth_relational` | 859 |
| `&mut core` | 578 |
| `worth_runtime_world` | 574 |
| `stacker` | 468 |
| `worth_foundational` | 447 |
| `hashbrown` | 439 |
| `serde_stacker` | 434 |
| `worth_query_admission` | 370 |
| `&mut <>` | 170 |
| `serde_json` | 167 |
| `bool` | 166 |

Query Execution module counts (first three namespace components):

| Module | Symbols |
| --- | ---: |
| `worth_query_execution::domain_computation::primary_graph` | 40,599 |
| `worth_query_execution::domain_computation::authorization` | 3,917 |
| `worth_query_execution::domain_computation::execution_runtime` | 972 |
| `worth_query_execution::domain_computation::provider_session` | 631 |
| `worth_query_execution::equality_type_id::EqualityTypeId` | 209 |
| `worth_query_execution::domain_computation::application_aftermath` | 27 |
| `worth_query_execution::basis::product_branch` | 5 |
| `worth_query_execution::domain_computation::product_publication` | 2 |
| `worth_query_execution::domain_computation::runtime_time` | 1 |

## Ranked Query Execution bodies

Type arguments and trait qualification are normalized to group instantiated
Self/method families. These are diagnostic families, not compiler-exact source
function identities. Bytes are physical section extents from function offsets
to the next offset or section end, including padding and any intervening local
code. Alias names at one address count once; mixed-family aliases are reported
separately and never assigned an arbitrary owner. Archive size also contains
metadata and relocations. Do not extrapolate these numbers to downstream savings.

| Rank | Normalized family | Code bytes | Copies |
| --- | --- | ---: | ---: |
| 1 | `worth_query_execution::domain_computation::primary_graph::application_query::one_shot::outcome::complete_one_shot_with_capture::<>` | 790,976 | 32 |
| 2 | `worth_query_execution::domain_computation::primary_graph::application_installation::construction::in_memory_with_contributions_in_advancement::<>` | 569,692 | 23 |
| 3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::source_identity::checkpoint_commitment::append_footprint::<>` | 409,056 | 96 |
| 4 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry::intern_with_storage::<>::{closure#0}` | 401,280 | 96 |
| 5 | `worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::source_readmission::ready_on_selected::certify_ready_on_selected_with_disclosure::<>::{closure#5}` | 344,134 | 28 |
| 6 | `worth_query_execution::domain_computation::primary_graph::application_attempt::read_set::WorthQueryApplicationReadAttempt<>::complete_native` | 340,488 | 24 |
| 7 | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>` | 313,329 | 23 |
| 8 | `worth_query_execution::domain_computation::primary_graph::application_attempt::read_set::WorthQueryApplicationReadAttempt<>::complete_full` | 277,896 | 24 |
| 9 | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_binding::registration::register_provider_attempt::<>` | 258,842 | 23 |
| 10 | `worth_query_execution::domain_computation::primary_graph::application_attempt::read_set::completed_facts::admit_completed_facts::<>` | 235,333 | 103 |
| 11 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::finish_application_query_admission_with_basis::<>` | 165,620 | 28 |
| 12 | `worth_query_execution::domain_computation::primary_graph::workflow::instance::transition::settlement::visit_transition_facts_from_basis::<>` | 160,563 | 23 |
| 13 | `worth_query_execution::domain_computation::primary_graph::workflow::instance::facts::visit_start_facts::<>` | 160,092 | 18 |
| 14 | `worth_query_execution::domain_computation::authorization::application_disclosure::contract::compilation::admit_rule::<>` | 153,203 | 23 |
| 15 | `worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::unpublished::ContinuationState::progress::<>` | 146,272 | 14 |
| 16 | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryProviderAttemptPreparation::stage_local_workflow_settlement::<>` | 145,521 | 23 |
| 17 | `worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::post_authorization::preparation::prepare_authorized::<>` | 145,516 | 14 |
| 18 | `worth_query_execution::domain_computation::primary_graph::application_attempt::read_set::WorthQueryApplicationReadAttempt<>::complete_projected_dependencies_with_request` | 140,737 | 23 |
| 19 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry::intern_with_storage::<>` | 140,256 | 96 |
| 20 | `worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution::run_admitted_advancement::<>` | 139,955 | 100 |

## Instrument evidence and growth policy

The instrument's 11 fast owner tests use hand-counted COFF reports, not generated
expectations copied from its algorithm. They cover aliases, independent symbol
and size disagreement, corrupt offsets, qualified owner normalization, and
successful/failed Cargo artifact selection. The real scan independently checks
two LLVM reports for symbols and two for bytes.

No automatic application-size growth test is added: reproducing this fixture
requires the large court dependency build, outside a cheap owner test. This
baseline and per-slice measurements are the growth record allowed by the brief.
Instrument tests are measurement tests, not a claim of a symbol-growth guard.

## Readings I chose

- The explicit Slice 0 lint cleanup permits necessary nongeneric lint repairs;
  the later generics-only restriction governs optimization slices. Newly synced
  gate failures were repaired at their owners before measuring.
- The footer's scoped checks replace wider skill gates. No workspace build ran.
- Source-program identity is in-memory bind order only, with checked exhaustion;
  it never enters persistence, content identity, or evidence.
- A Query literal traversal mock required forbidden ExecutionRequest references
  in its test module (BC9002). It was removed rather than hiding the reference.
  The Relational literal bind-order owner court and BTreeMap implementation are
  the available evidence; no Query literal traversal court is claimed.
- Measurement begins after Slice 0 and sync, using the existing test profile.
  There is no dev-to-test-profile reduction claim.
- Workflow formatting retains the existing exact Error parameter. Its dynamic
  emitter removes application specialization without inventing an erased error
  channel or widening a refusal. The final application archive verifies that
  the large formatter bodies are no longer instantiated there.

## Slice results

| Slice | Symbols | Delta | Archive bytes | Delta | Physical code bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 160,719 | 0 | 674,508,286 | 0 | 48,544,077 |
| Concrete source meanings | 157,807 | -2,912 (-1.81%) | 659,337,342 | -15,170,944 (-2.25%) | 46,914,499 |
| Concrete completed facts | 156,880 | -3,839 (-2.39%) | 655,841,648 | -18,666,638 (-2.77%) | 46,647,854 |
| Shared workflow emission | 156,909 | -3,810 (-2.37%) | 654,806,940 | -19,701,346 (-2.92%) | 46,360,124 |

The source-meaning slice keeps the public generic entry and typed observations,
but uses the exact existing relational denial and a dynamic comparator inside
the concrete registry. Row and set completion use a borrowed non-generic owner
context. The original comparison-state custody extent and phase ordering remain.
Query Execution namespace symbols fall from 46,363 to 44,506; primary_graph
falls from 40,599 to 38,742. One-shot completion falls from 790,976 to 667,168
code bytes (still 32 typed copies). The 96-copy interner/checkpoint families
are absent from the application archive; their bodies now compile in the owner.
Independent totals agree at 157,807 symbols and 46,914,499 code bytes;
unattributed code is 52,405 bytes and shared-alias code remains zero.
Archive SHA-256:
`b795f2ee986eb1a5dbd27c5917509211bfa24859872cfd0d56ee504584e8429d`.

Reproduction substitutes `source-meanings-court-build.jsonl` for the baseline
build log and `mono-source-meanings.json` for the measurement output, with the
same exact build/instrument commands, fixture and existing test profile.
Source parent is `2fee93f1a8`, plus the four-file source-meaning diff in its
following commit. Upstream sync fetched before the slice was unchanged.


Completed-fact admission saves a further 927 symbols, 3,495,694 archive bytes
and 266,645 code bytes relative to the preceding slice. Query Execution now
has 44,094 namespace symbols; primary_graph has 38,330. The 103-copy generic
entry remains as the ownership shell, but its body falls from 235,333 to
111,549 code bytes. Concrete begin/append/finish compile in the engine owner.
Independent totals agree at 156,880 symbols and 46,647,854 code bytes;
unattributed code is 56,216 and shared-alias code is zero. SHA-256:
`a413fdc5393aecc0feff84f8e0102b3b612d585526c8a3b37e7f435c99e58440`.
Reproduce with `completed-facts-court-build.jsonl` and `mono-completed-facts.json`
in the same commands. Source parent is `e55d089b77`, plus its following
completed-fact commit; fetched sync branches were unchanged.


Workflow emission saves a further 1,034,708 archive bytes and 287,730 physical
code bytes, while **adding 29 external symbols** relative to completed facts.
This is a byte reduction, not a per-slice symbol reduction. Query Execution
namespace count is 44,102 and primary_graph is 38,338. The 23-copy transition
formatting body and new common start-record body are absent from the application
archive; the 18-copy start ownership shell is 26,190 bytes instead of 160,092.
Existing Error specialization is retained exactly, while application callback
specialization leaves the large bodies. No erased error container was added.
Independent totals agree at 156,909 symbols and 46,360,124 code bytes;
unattributed code is 58,481 and shared-alias code is zero. SHA-256:
`b601d3320681e37f6e9b525418aefe3225ebd85b84d70cc302fe55be68c3f598`.
Reproduce with `workflow-facts-court-build.jsonl` and `mono-workflow-facts.json`
in the same commands. Source parent is `e9dfbfa019`, plus its following workflow
commit; fetched sync branches were unchanged.

## Stopping point and remaining ranking

The measured final fixture is 19,701,346 bytes (2.92%) and 3,810 symbols (2.37%)
smaller than baseline; physical code falls 2,183,953 bytes (4.50%). No downstream
size reduction or controlled build-time/throughput improvement is claimed.

These are the final leading remaining Query families, using the same normalization:

| Rank | Normalized family | Code bytes | Copies |
| --- | --- | ---: | ---: |
| 1 | `worth_query_execution::domain_computation::primary_graph::application_query::one_shot::outcome::complete_one_shot_with_capture::<>` | 667,168 | 32 |
| 2 | `worth_query_execution::domain_computation::primary_graph::application_installation::construction::in_memory_with_contributions_in_advancement::<>` | 569,692 | 23 |
| 3 | `worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::source_readmission::ready_on_selected::certify_ready_on_selected_with_disclosure::<>::{closure#5}` | 344,134 | 28 |
| 4 | `worth_query_execution::domain_computation::primary_graph::application_attempt::read_set::WorthQueryApplicationReadAttempt<>::complete_native` | 340,488 | 24 |
| 5 | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>` | 313,329 | 23 |
| 6 | `worth_query_execution::domain_computation::primary_graph::application_attempt::read_set::WorthQueryApplicationReadAttempt<>::complete_full` | 277,896 | 24 |
| 7 | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_binding::registration::register_provider_attempt::<>` | 258,842 | 23 |
| 8 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::finish_application_query_admission_with_basis::<>` | 165,620 | 28 |
| 9 | `worth_query_execution::domain_computation::authorization::application_disclosure::contract::compilation::admit_rule::<>` | 153,203 | 23 |
| 10 | `worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::unpublished::ContinuationState::progress::<>` | 146,272 | 14 |

Boundary review of the next candidates found:

- Remaining one-shot completion constructs typed observations and invokes typed
  capture/projection/results. Its common source-meaning work is already shared.
  Turning the remaining aggregate bytes into a saving needs another result or
  observation representation change, rather than another large concrete loop.
- Installation construction carries Schema/Contributions configuration, typed
  installed schema and bootstrap, initial-state callback, program admission,
  checkpoint transition, and publication. Its common installer already exists;
  the small setup remainder does not justify another slice on this evidence.
- Ready-on-selected disclosure owns prepared typed Query permission and its
  one-shot disclosure continuation across current/pending/rebound states. An
  erased view here would need to preserve that lifecycle across several owners.
- Native/full read completion mix exact admitted operation/scope contracts,
  authority checks, expected-fact custody, and typed completed read sets. The
  large completed-fact admission loop is shared. Smaller patch/validation
  fragments could be extracted, but require additional phase/drop boundaries
  for a smaller uncertain marginal archive saving.
- Commit preparation, registration, and admission finish bind the exact admitted
  operation/query to provider, effect or result phases. Their aggregate family
  sizes are not evidence that all bytes are removable common work.
- Disclosure rule compilation is a possible later independent optimization:
  153,203 bytes/23 copies. Its remaining query type crosses admission,
  result-rule and influence compilers. Sharing it needs a borrowed installed
  query view across those owners; no benefit for that change was measured here.

I stopped after the concrete completion/formatting cuts. The last low-risk
extraction saved only 0.16% of the preceding archive and added 29 symbols. The
next ranked opportunities require new erased lifecycle/query views or smaller
multi-phase helpers; that proof-interface/maintenance cost is not justified as
another slice by the remaining measured common bodies. This is a deliberate
marginal-benefit judgment, not a hard-rule blocker or a claim that the remaining
engine cannot be optimized. The record identifies where to resume with a new
boundary review. No court or refusal was weakened to obtain these numbers.
