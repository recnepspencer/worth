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

## Slice results

Baseline only: 160,719 symbols and 674,508,286 archive bytes. Optimization results
must retain the same fixture, profile, instrument and successful artifact selection.
