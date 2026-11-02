# Engine-once Slice 3: runtime core, installation views and installed re-entry

Implementation parent: `52e7dba3dcbe257a209c69878438e7627867f328`.
The governing engine-once design and Slice 2 rulings were read in full. Both named
sync branches are incorporated. The CAD plan remains read-only. This slice stops
before Slice 4.

## Authority and representation

The former application's 43 runtime fields are split into thirteen typed
registries/application-type inventories and thirty original fields in
`WorthQueryApplicationRuntimeCore`. The concrete core also retains the
installation-issued operation catalog. The existing private field name `runtime`
is kept for the core, preserving the seven reserved pub-causes files without a
new public Deref or duplicate runtime state. Certification found that grouping
fields had changed automatic destruction order. The correction places the core
before the conditional and typed registries, moves execution into a local drop
guard, and stages concrete custody in its original positions around typed
registries before cleanup can unwind. The actual backing order is preserved,
including the live conditional registry between prefix/bridge and provider
custody. The original explicit bridge close, conditional clear and resource
clear sequence remains unchanged. Details and the actual provider/handler
sentinel court are recorded in mono-runtime-core-corrections.md.

`InstalledQueryView<'a>`, `InstalledOperationView<'a>` and
`InstalledSchemaView<'a>` are borrowed `#[doc(hidden)]` installation views, with
private construction from genuine installed state. They are not new persisted
or executable authority forms. Query preserves its optional binding observation;
operation exposes its genuine seal/definition/contracts and recorded type values;
schema exposes its actual compiled backings and shared catalog.

The existing installed operation's concrete fields now live in one private
embedded `InstalledOperationBacking`, with the original typed handle's
PhantomData unchanged. Its field order and all public signatures, including const
getters, are preserved. The catalog's existing retention clone and view assembly
now have concrete bodies. The backing is the storage of the same installed
operation, not a second owned authority representation. Only that private backing
implements the sealed retained-source trait. The typed handle gains no public
Clone/from-parts constructor. The immutable compiled/native backings remain
Arc-shared; retention still performs its existing cold metadata clones.

The canonical operation catalog belongs to the schema index record, so separate
loans from the same registered schema share it. It is issued before later
operation registration and retains genuine installed state after typed loans
drop. Repeated declaration identities must agree in binding, seal, definition,
recorded type values and contracts, or get the existing exact installation
refusal. A hot lookup clones an Arc under a BTreeMap lock, then borrows its view
after unlock; it allocates or recompiles nothing and calls no callback under the
lock. Expected seals are never inferred from the admission being checked.

Every typed re-entry validates runtime/brand ownership, the genuine installed
seal/definition and operation type, input type, then lapse, before source work.
Transaction application/branch custody and redispatch handle gates keep their
original precedence. Admission lapse precedes funded advancement; this deliberately
retains the ruled pre-funding refusal, so the former claim that all outer phase
gates were unchanged was too broad. Recovery-authority issuance checks ownership,
seal and type without adding lapse refusal; actual use still checks currentness.
History work and source binding/consumption now gate admission before reads or
mutation. Pure carried-proof comparison helpers remain immutable predicates. Mismatched installed facts produce `StaleInstalledOperation`; input mismatch
retains mutation's exact `InputNotAdmitted` and adds that exact general
authorization refusal. Redispatch/recovery preserve its original kind through
`AdmissionAuthorization`. Requested/approved/mandatory lifecycle bindings return
through the existing outcome converters on refusal or lapse.

All new comparison and mutation/AdmissionAuthorization conversion bodies are
concrete. Runtime commit validation receives shared borrows of the existing
admitted core and brand tag; their fields/constructors stay private in the
admission owner. No owned core extraction, rewrap door or caller input crosses
this seam. Caller `ApplicationSharedInput<Input>` stays in the brand, with no new
public `'static`, Send or Sync bound. The remaining generic methods borrow and
forward arguments, or preserve genuinely typed result/binding custody.

Six remaining and_then adapters in migration, read_set, managed recovery (two),
workflow preparation and retirement only sequence calls to concrete validation
owners. The reviewer read all six: they inspect no fields, assemble no view and
convert no AdmissionAuthorization kind. This is a thin-shell limitation, not a
claim of literally zero generic forwarding instructions.

The old generic bodies replaced here are deleted. No Decl or Host source or
signature changed. No AspectShape form, persisted format, migration, decoder,
fallback, dual path, crate, feature, profile or build role was added. No new
HashMap, HashSet, unsafe or placeholder was introduced. Existing application-type
inventories remain in the typed registries.

## Warning ruling

The user explicitly assigns these two meaningful unread causes to pub-causes for
its next round after this commit lands, and exempts exactly them from Slice 3's
touched-file warning gate:

- `DecisionQueryPreparationStop::Native(CompanionPreflightStop)` in
  `application_entry/mutation/execution/decision_query.rs`.
- `InspectionPlanDenial::Graph(WorthQueryGraphWorkAdmissionDenial)` in
  `causal_inspection/application_inspection/admission/plan.rs`.

Both payloads remain intact. No allow, fake read, unused getter or underscore
rename was used to silence them. The final all-target check has zero new warnings
and no other warning in a touched file. MSRV remains 1.94; the two fetch_update
sites and all seven originally reserved files remain untouched.

## Commands and courts

All Query commands use:

```text
WORTH_IMPLEMENTATION_BUILD_ID=11111111-1111-1111-1111-111111111111
CARGO_TARGET_DIR=C:/worth-build-cache/target-cut-pub-mono
PYTHONUTF8=1
```

Every Python check in this follow-up uses PYTHONUTF8=1. Long Cargo commands ran
in the background with output redirected to `target/pub-mono-logs/`; no tool
timeout killed a build. No workspace Cargo command or make test ran.

### Installation and central seal owners

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-installation --lib
```

**365 passed, 0 failed, 0 ignored, 0 measured, 0 filtered**, 4.46 s. Two new installed-view courts use real schema/index owners, literal OperationMeaningChanged, independent owner validation and Arc identity. They prove shared rebind catalog, retained lifetime, certified duplicates and refusal of a genuinely installed foreign operation. Existing schema-affinity cases moved unchanged for the line cap. Five cryptography courts include an independent published RFC 4231 case-one HMAC vector, wrong-key/family/field/option separation, explicit entropy refusal, generation keys and redacted Debug; the query owner checks the central seal family. Log: `slice3-engine-once-installation-owner`.

### Restored forgery court

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib erased_core_forgery -- --test-threads=1
```

**19 passed, 0 failed, 0 ignored, 0 measured, 2222 filtered**, 5.84 s. Five new foreign-installed-seal cases cover projection, idempotency, read attempt, mutation and real decision-query entry. The door copies another genuinely installed operation seal into both core and tag while leaving target names/types/definition intact, isolating comparison with installation. Literal StaleInstalledOperation, independent source/fact/request counters and reader sentinels are the oracles. A separate input-type case expects InputNotAdmitted. The allocation court now measures full re-entry plus its other probes and still asserts zero allocations/reallocations. Log: `slice3-engine-once-restored-forgery`.

### Execution library

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib
```

**2240 passed, 0 failed, 1 ignored, 0 measured, 0 filtered**, 644.10 s. The unchanged courts exercise advancement/panic payload/poisoning, authority order, exact failures, lifecycle binding custody, cancellation, recovery, resource evidence and scale. The existing ignored refused_maintenance_cleanup_probe is a subprocess probe whose parent supplies its required payload. No test was newly ignored or weakened. Log: `slice3-engine-once-execution-owner`.

### Publication recovery owner

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-publication --lib application_entry::mutation::performed::denial::recovery_posture::query
```

**5 passed, 0 failed, 0 ignored, 0 measured, 23 filtered**, 0.00 s. The exhaustive caller explicitly maps InputNotAdmitted to Terminal for reuse of the same required-output source. The new court asserts literal Terminal through both actual admission and execution wrappers rather than repeating the classifier. Log: `slice3-engine-once-publication-owner`.

### Full Decl/Host consumer binary

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host
```

**325 passed, 0 failed, 2 ignored, 0 measured, 0 filtered**, 38.03 s. The real consumer boundary exercises ordinary public behavior, workflow, authorization, lifecycle, bounded work and cost cases. The two existing ignored tests are scheduled qualifications for 1,000 sibling futures and 1,000 authored chain steps; they were not run. No public court expectation was weakened. Log: `slice3-engine-once-decl-host-court`.

## Mutation proof and restoration

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib foreign_installed_seal -- --test-threads=1
```

Only `self.operation_authority_identity_bytes != installed.authority_identity_bytes()`
was removed. Runtime ownership, tag agreement, definition and type checks remained.
The current-tree run compiled successfully, then exited 101 with **0 passed,
5 failed, 0 ignored/measured, 2236 filtered**, 1.16 s. Decision and mutation violated
execution.did_nothing(); idempotency returned unexpected Ok; projection reached
its reader sentinel; read-attempt returned no denial. Thus each new case fails
with precisely the owed check removed. The proof checker exited 0. The runner's
finally block restored exact original bytes with a fresh timestamp; saved and
restored SHA256 is
`95fad5e696edb91c302c4325c7febcc9db5cffb269b84ac8669cb9e84b43c3aa`.
The restored 19-test court above rebuilt afterward and passed. The same reviewer
independently verified the one-term mutation, every failure and byte restoration.

The earlier proposal's mutation also compiled and failed all five cases (1.10 s),
then restored SHA 05f3340fac9f39ed56d5330703acbdeb58fb8cd089375abfd84cd94cd6b22f41
and passed 19 cases (5.17 s). Those historical results are not substituted for the
current-tree rerun.

## Final scoped gates

```text
cargo fmt --manifest-path workspaces/worth-query/Cargo.toml --check -p worth-query-execution -p worth-query-installation -p worth-query-publication
cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution -p worth-query-installation -p worth-query-publication --all-targets --message-format=json
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
bash scripts/ci/check_workspace_rust_line_caps.sh dirty
git diff --check
git diff --cached --check
git diff --check origin/master...HEAD
```

Formatting, Clippy, boundary, context and dirty caps exit 0. Caps check 606 Rust files.
Final Clippy reports **0 errors, 87 Execution warning diagnostics and 7 upstream
Relational diagnostics; Installation 0, Publication 0**. Execution's parent baseline
is 91 diagnostics / 40 distinct; final 87 / 38, **0 new**. Exactly the two user-exempt causes
are in touched files; no other touched-file warning remains. The genuinely dead
ComparedOutputProfile::into_original accessor and the private-interface leak in
compare_and_commit_application_with_causality were removed/narrowed properly.

Clippy/tools logs use `slice3-engine-once-final-clippy`, `slice3-engine-once-fmt-check`,
`slice3-engine-once-boundary`, `slice3-engine-once-context` and
`slice3-engine-once-final-linecaps`. Final working/staged/range whitespace results
are recorded alongside the final review; range is checked again after commit.
The seven reserved files, consumer facades and measurement instrument have empty
diffs. No additional ignored qualification, doctest or workspace scope is claimed.

## Closed development failures and earlier successful runs

| Log prefix in `target/pub-mono-logs/` | Exit / diagnostic output | Correction |
| --- | --- | --- |
| `slice3-views-clippy` | 101; 4 E0515 | Borrowed getter lifetimes made explicitly `'a`. |
| `slice3-runtime-cutover-clippy` | 101; 1063 errors: E0615 412, E0609 601, E0308 10, E0599 30, E0614 10 | Routes changed to the actual core field and execution owner. |
| `slice3-runtime-references-clippy` | 101; 7 errors | Corrected remaining execution-runtime references. |
| `slice3-runtime-alltargets-clippy` | 101; 339 errors: E0609 228, E0615 90, E0614 6, E0308 4, E0599 11 | Test owners routed through the moved fields. |
| `slice3-reentry-clippy` | 101; 48 diagnostics: E0425 6, E0433 2, E0004 4, E0615 18, E0609 16, E0614 2 | Core type owner paths corrected. |
| `slice3-entries-lib-clippy` | 101; 5 diagnostics: E0061 3, E0004 2 | Lifecycle refusal paths return their original bindings. |
| `slice3-final-entries-lib-clippy` | 101; 4 diagnostics: E0603 1, E0599 1, E0004 2 | Narrow owner re-export corrected. |
| `slice3-complete-entries-lib-clippy` | 101; 2 E0004 | Exact InputNotAdmitted matches added. |
| `slice3-input-mapping-lib-clippy` | 101; 2 E0616 | Read-only catalog getter used at the owner boundary. |
| `slice3-forgery` | 101; 37 compile errors; zero tests ran | 34 moved-field references, 2 consequent dereference errors and the redispatch exhaustive court roster corrected. |
| `slice3-complete-alltargets-clippy` | 101; 8 collapsible_if diagnostics at 4 source sites (lib/test) | MSRV-safe let chains replaced the nested conditions. |
| `slice3-court-preflight-build` | 101; 1 E0004; zero court tests ran | Publication exact InputNotAdmitted terminal classification and literal two-entry owner court added. |
| `slice3-initial-linecaps` | 1; capability_revalidation 404 and application_schema_tests 401 lines | Concrete currentness body and existing schema-affinity courts moved into named owner files. |
| `slice3-engine-once-staged-diff` first stage |2; one trailing whitespace line in generated report | Strip trailing whitespace during report generation and restage; no Rust source changed. |
| `slice3-engine-once-installation-clippy` |101;1E0277 in owner test registering Arc of typed handle after sealed-source impl moved to backing | Test now obtains the independently installed foreign backing from its genuine schema catalog. Literal refusal and ptr oracle remain. |
| `slice3-write-final-report` first attempt |1; UnicodeDecodeError reading ASCII exit file as UTF16 | Read exit files using their BOM/UTF8 encoding; no source or evidence changed. |
| `slice3-engine-once-clippy` |0; two new distinct dead-wrapper warnings, so the warning gate did not pass | Delete obsolete generic ownership/commit-installation wrappers; retain concrete validators and strengthen allocation court to full re-entry. |

All ledger diagnostics are counted across the whole log, including lib/test
copies. These failed development probes were closed; they are not presented as
passing gates. The four collapsed-if source sites originally emitted 8 diagnostics.
The intermediate Clippy probes used the explicit Query manifest and scoped
Installation/Execution lib or all-target packages; no workspace check ran.

Earlier successful owner evidence: targeted installed views 2 passed / 363 filtered
(0.01s); original Installation 365 passed(3.29s); corrected original forgery 19 passed /
2222 filtered(5.05s); original full Execution 2240 passed / 1 ignored(598.84s); original
Publication 5 passed / 23 filtered(0.00s); original full Decl/Host 325 passed / 2 ignored
(24.32s). The updated pre-mutation forgery 19 passed / 2222 filtered(6.09s) and current
Installation 365 passed(4.46s). Current restored/integrated runs above supersede
earlier evidence. Earlier boundary/context/fmt/whitespace passes and 574/597/598
file-cap passes are superseded by final scoped checks.

## Measurement

**The final measurement still regresses against Slice 2. It certifies no net
saving for Slice 3.** It is smaller than the initial proposal after removing the
new generic bodies. Both points are retained in mono-runtime-core.json, including
the initial proposal's full raw hashes/counts/rankings.

Successful fresh artifact command and unchanged COFF instrument:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host --no-run --message-format=json > target/pub-mono-logs/slice3-engine-once-court-build.jsonl
python scripts/quality/query_monomorphization.py --cargo-messages target/pub-mono-logs/slice3-engine-once-court-build.jsonl --package worth-query-testkit-certification-canonical --llvm-bin C:/Users/Esther/.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/bin --output target/pub-mono-logs/mono-slice3-engine-once-certification-canonical.json
```

Both exit 0. The dependency artifact has the same opt_level=0 / debuginfo=0, debug-assertions and
overflow-checks profile as Slice 2. The script selects the one canonical artifact
from successful Cargo JSON. No filename is guessed, profile adjusted, release or
additional fixture built for this slice. Raw report/archive hashes and independent
llvm-nm / llvm-size totals are in the JSON record.

| Metric | Slice 2 | Initial proposal | Final Slice 3 | Final minus Slice 2 |
| --- | ---: | ---: | ---: | ---: |
| Exported symbols | 150,134 | 150,732 | 150,256 | +122 |
| Archive bytes | 619,715,596 | 621,699,534 | 619,896,574 | +180,978 |
| Physical code bytes | 44,852,822 | 44,994,729 | 44,872,006 | +19,184 |
| All-owner slope | 1,789,712.916065 | 1,795,642.930443 | 1,792,398.212355 | +2,685.296289 |
| Execution slope | 1,373,834.581767 | 1,377,426.419328 | 1,375,552.419328 | +1,717.837560 |

The follow-up removes 476 symbols, 1,802,960 archive bytes and 122,723 code bytes from
the initial proposal. This does not turn the final Slice 2 comparison into a saving.
Slope is sum(bytes/copies) for normalized families with at least seven copies: a fixture
proxy, not a measured multi-fixture derivative. Endpoints and deltas are rounded
separately from exact Fractions, retained in JSON. All owner/module counts and
qualifying family counts are also retained there. Shared/unattributed bytes are
reported as measured; no ambiguous code is assigned as a saving.

### Initial top 20 positive byte-growth families

Ranked by after-minus-before bytes, descending then name, using the hash-verified
raw reports referenced by mono-admitted-operation.json and the initial
mono-runtime-core.json. Generic scale comes from source signatures, not an
assumption that copies equal unique operations. Normalized core/alloc support
families combine many callers. Family-name churn is stated explicitly and is not
net semantic growth. Each new substantial per-operation clone/comparison/view/
conversion body identified here was moved into a concrete owner.

| Normalized family | Before bytes / copies | After bytes / copies | Byte growth | Instantiation and source accounting |
| --- | ---: | ---: | ---: | --- |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::retain_for_reentry` | 0 / 0 | 32136 / 26 | +32136 | **per operation**. Private retention clone of the typed installed handle; moved to concrete backing Clone/retain. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::resolve_admitted_application_idempotencies::<>::{closure#6}` | 0 / 0 | 30291 / 23 | +30291 | **per operation**. Closure ordinal churn: old closure#5 and new closure#6 each have 30291 bytes / 23 copies. This appearance is not 30291 bytes of new work. |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::from_installed_schema` | 8346 / 26 | 22568 / 26 | +14222 | **per operation**. Installation entry is typed by Schema, Operation and Input. New retention call now reaches a concrete body. |
| `core::result::Result<>::map_err::<>` | 1803383 / 9025 | 1815339 / 9103 | +11956 | **mixed support: per operation and per application**. Result map_err aggregates all concrete result/function types; its normalized family is not a count of operation brands. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_commit_program::<>` | 0 / 0 | 11155 / 23 | +11155 | **per operation**. New typed commit-entry validator; full body moved to RuntimeCore, typed shell forwards borrowed core/tag. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::execute_mutation_handler_report_with_query_reads::<>` | 9846 / 9 | 20502 / 9 | +10656 | **per operation/binding**. Existing generic decision-handler entry, specialized by Schema and mutation Binding. Added validation now calls concrete owners. |
| `worth_query_execution::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation<>::validate_mutation_entry::{closure#1}` | 0 / 0 | 8395 / 23 | +8395 | **per operation**. New typed mutation-refusal closure; replaced by named concrete mutation_denial. |
| `core::result::Result<>::and_then::<>` | 116802 / 578 | 124474 / 639 | +7672 | **mixed support: per operation and per application**. Result and_then aggregates many caller/result/closure types; new re-entry chains are now concrete. |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::installed_view` | 0 / 0 | 7592 / 52 | +7592 | **per operation**. New view assembly on typed installed handle; body moved to private concrete backing.view. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::begin_projected_application_read_attempt_with_request::<>` | 112608 / 23 | 118128 / 23 | +5520 | **per operation and query**. Existing admitted projection/read entry retains actual query/operation types. Its new validation body is concrete. |
| `worth_query_execution::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation<>::validate_installed_entry` | 0 / 0 | 4646 / 23 | +4646 | **per operation**. New brand ownership/seal chain; moved to existing concrete admitted core with borrowed tag. |
| `alloc::sync::Arc<>::new` | 34806 / 236 | 39408 / 262 | +4602 | **mixed support: per operation and per application**. Arc::new aggregates allocation for many concrete types; operation-retention allocation now takes concrete backing. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>` | 312961 / 23 | 317009 / 23 | +4048 | **per operation**. Existing typed commit preparation. New installed validation is forwarded to concrete owners. |
| `worth_query_execution::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation<>::validate_projection_authority` | 4324 / 23 | 8211 / 23 | +3887 | **per operation**. New installed re-entry chaining in existing projection gate; moved to concrete admitted core. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::resolve_admitted_application_idempotencies::<>::{closure#4}::{closure#0}` | 0 / 0 | 3473 / 23 | +3473 | **per operation**. Closure ordinal churn within existing idempotency body; old closure#3::{closure#0} was3312bytes, new closure#4::{closure#0}3473bytes. |
| `worth_query_execution::domain_computation::primary_graph::application_installation::construction::in_memory_with_contributions_in_advancement::<>` | 569692 / 23 | 572636 / 23 | +2944 | **per application/contributions**. Installer entry generic over Schema and Contributions, not an operation marker. Its added operation-catalog setup is concrete. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_commit_installation::<>` | 0 / 0 | 2576 / 23 | +2576 | **per operation**. New typed commit-installation validator; full validation/conversion now RuntimeCore. |
| `worth_query_execution::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation<>::validate_mutation_entry` | 0 / 0 | 2392 / 23 | +2392 | **per operation**. New typed mutation validation shell; chaining and conversion now concrete. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationEffectProgram<>::validate_installed_entry` | 0 / 0 | 2392 / 23 | +2392 | **per operation**. New typed program re-entry shell delegates to brand/core. No comparison policy remains in this shell. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::resolve_admitted_application_idempotencies::<>::{closure#1}` | 989 / 23 | 3266 / 23 | +2277 | **per operation**. Closure ordinal churn: old closure#0 and new closure#1 each have3266bytes; name-based growth2277 is not semantic growth. |

### Final top 20 positive byte-growth families

This is the final artifact against the same Slice 2 baseline. Existing typed
installers, reads, candidate/report and lifecycle custody retain their typed
values and result/drop envelopes. All new seal/brand comparisons, installed-view assembly, and mutation/
AdmissionAuthorization conversion bodies are concrete; thin borrowing/forwarding instructions still instantiate. The
remaining code growth of 19,184 bytes is reported without waiving it.

| Normalized family | Before bytes / copies | After bytes / copies | Byte growth | Instantiation and source accounting |
| --- | ---: | ---: | ---: | --- |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::from_installed_schema` | 8346 / 26 | 22568 / 26 | +14222 | **per operation**. Existing Schema/Operation/Input installation entry adds concrete retain/register calls plus typed Result/drop propagation. Retention clone, registration comparison and view assembly are concrete. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::execute_mutation_handler_report_with_query_reads::<>` | 9846 / 9 | 20502 / 9 | +10656 | **per mutation binding/operation**. Existing typed handler and report envelope. New installed/brand checks and denial-kind conversion have concrete bodies. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::begin_projected_application_read_attempt_with_request::<>` | 112608 / 23 | 118128 / 23 | +5520 | **per operation and query/state**. Existing typed projected read entry; new metadata validation calls the concrete admitted core. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>` | 312961 / 23 | 317009 / 23 | +4048 | **per operation**. Existing typed commit preparation; its added installation check reaches the concrete runtime/admission owners. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_commit_program::<>` | 0 / 0 | 3496 / 23 | +3496 | **per operation**. Thin borrowed Core+BrandTag adapter to concrete RuntimeCore validation; no comparison or outcome-mapping body in this shell. |
| `worth_query_execution::domain_computation::primary_graph::application_installation::construction::in_memory_with_contributions_in_advancement::<>` | 569692 / 23 | 572636 / 23 | +2944 | **per application and Contributions**. Existing typed installer/configuration entry; new catalog ownership is concrete. Installation construction is a later slice. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::resolve_admitted_application_idempotencies::<>` | 58259 / 23 | 61203 / 23 | +2944 | **per operation and idempotency binding iterator**. Existing typed idempotency orchestration; new validation/lapse policy is a single call into the concrete admitted core. |
| `core::result::Result<>::and_then::<>` | 116802 / 578 | 118632 / 593 | +1830 | **mixed support: per operation and per application**. Normalized Result::and_then aggregates different result/closure types. Remaining six adapters only sequence forwarding calls to concrete validation owners; they do not compare fields or convert AdmissionAuthorization kinds. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::compare_and_commit_application_in_advancement_with_history::<>` | 8050 / 14 | 9618 / 14 | +1568 | **per operation**. Existing typed program/history entry with a call to the concrete commit gate; actual candidate/program custody remains typed. |
| `worth_query_execution::domain_computation::primary_graph::invariant_projection::WorthQueryApplicationInvariantProjectionAuthority<>::project_admitted_operation_with_source_access::<>` | 57696 / 32 | 59232 / 32 | +1536 | **per operation, projection callback and Output**. Existing typed projection/read callback envelope; installed seal/type/lapse comparison body is concrete. |
| `worth_query_execution::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation<>::validate_mutation_entry` | 0 / 0 | 1495 / 23 | +1495 | **per operation**. Thin borrowed-argument adapter to concrete mutation validation and its named concrete denial conversion. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::compare_and_commit_application_for_program_output_producer::<>` | 8050 / 14 | 9394 / 14 | +1344 | **per operation**. Existing typed output-producer commit entry with a concrete gate call; typed program custody remains in its owner. |
| `worth_query_execution::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation<>::validate_installed_operation` | 0 / 0 | 1128 / 24 | +1128 | **per operation**. Thin core/tag/catalog forwarding method. The genuine installed-view comparison is concrete. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::installation::assembly::assemble_application_runtime::<>` | 62097 / 7 | 63217 / 7 | +1120 | **per application/Schema**. Existing typed runtime registry assembly now packages concrete runtime state and retains the installation-issued catalog. |
| `worth_query_execution::domain_computation::primary_graph::bootstrap::WorthQueryPrimaryGraphBootstrap<>::publish_with_resource_support` | 57533 / 7 | 58653 / 7 | +1120 | **per application/Schema**. Existing typed bootstrap retains/publishes the concrete catalog. No operation-generic comparison body is added. |
| `worth_query_execution::domain_computation::execution_runtime::installation_authority::WorthQueryExecutionInstallationAuthority::prepare_primary_graph_with_optional_checkpoint::<>` | 73472 / 7 | 74368 / 7 | +896 | **per application/Schema**. Existing typed primary-graph installation entry carries the new concrete catalog through preparation. |
| `worth_query_installation::application_schema::compilation::compile_application_schema::<>` | 26040 / 7 | 26936 / 7 | +896 | **per application/Schema**. Existing schema compilation shares its index-owned concrete operation catalog. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::execute_mutation_handler_report_with_query_reads::<>::{closure#0}` | 9 / 9 | 891 / 9 | +882 | **per mutation binding/operation**. Newly exported existing typed refused-report constructor, already present in the parent source. The old9-byte empty callback is final closure#1 with9bytes/9copies. This row combines ordinal reassignment with newly emitted existing report code, not new comparison policy. |
| `worth_query_installation::application_query::installed_contract::WorthQueryInstalledApplicationQuery<>::installed_view` | 0 / 0 | 851 / 23 | +851 | **per application and Query marker**. Thin borrow/constructor call into non-generic InstalledQueryView::new; view assembly is concrete. |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::install_executable_operation` | 119756 / 26 | 120588 / 26 | +832 | **per operation**. Existing typed operation compiler still captures install-time type values and wraps the private concrete backing. New comparison/view/retention bodies are not in this generic compiler. |

### Disappeared families

Only eight old normalized names are absent, so the requested top 40 is not padded.
Listed bytes/copies are old values, not net savings.

| Old family | Bytes | Copies |
| --- | ---: | ---: |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_active_capability_currentness` | 6622 | 7 |
| `worth_query_execution::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation<>::validate_runtime_ownership` | 4646 | 23 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_active_capability_currentness::{closure#0}` | 476 | 7 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_active_capability_currentness::{closure#1}` | 476 | 7 |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::idempotency::workflow_proposal_context::WorkflowProposalIdempotency::fmt` | 134 | 1 |
| `worth_query_execution::domain_computation::primary_graph::application_query::managed_index::source::initialization::SourceInitializationAuthority::clone` | 110 | 1 |
| `worth_query_execution::domain_computation::primary_graph::application_invariant::read_observation::InvariantReadObservation::default` | 61 | 1 |
| `worth_execution::backend::meter::nested_stop::NestedStopOwner::clone` | 3 | 1 |

No cheap growth test was added: producing the application archive requires the
large scoped consumer build. The permitted baseline and follow-up records supply
growth evidence; a static test of the committed numbers would mirror the record.

## Readings I chose

- Factor the same genuine installed state into a private concrete backing; do
  not invent an owned erased authority form or reconstruct expected seals from
  the admission. Public const getters and typed branding remain exact.
- Keep caller input in its brand and use shared existing core/tag metadata only
  for concrete validation. No owned extraction, public rewrap or new input bound.
- Keep the private runtime field name and typed inventories to preserve the seven
  reserved files and owner teardown; do not add a public Deref or new HashSet.
- The latest runtime/brand -> installed seal/type -> input -> lapse -> source order governs
  formerly lapse-first/occurrence-first paths; existing phase/handle ownership
  gates retain their roles and exact owner failures.
- Necessary exhaustive Publication caller propagation is in scope; no Decl/Host
  signature change is needed. No court is weakened to accommodate the cutover.
- The user explicitly exempts only the two named causes and assigns them to
  pub-causes; leave all meaningful payloads and both MSRV-sensitive fetch_update
  sites intact. No warning suppression or fake consumer is added.
- Keep one coherent commit for the causal authority change and the integrated
  tested tree, as the reviewer recommends. No extra build is spent on an
  artificial split. Stop here; Slice 4 and later construction work remain separate.

## Review and stop

The same independent lane reviewer applies qa-loop, qa-tests and code-quality-qa.
It closed missing re-entry paths, lifecycle return custody, exact exhaustive caller
mapping, line-cap owner splits, generic-body extraction, allocation coverage and
report accounting/rounding findings. It independently verified all fresh mutation
failures/restoration, initial and final rankings, actual archive/hash, independent
counts, exact Fraction deltas and generic-scale source classifications.

Final integrated review: **qa-loop PASS, qa-tests PASS, code-quality-qa PASS**.
The same reviewer independently confirmed the completed current-tree gates,
mutation proof/restoration, source classifications and final report, with no
unclosed finding, and approved one coherent Slice 3 commit. The two cause warnings
are exempt under the user's explicit ruling. This commit includes the measurement
record; its SHA is reported by the lane completion message. Slice 4 is not started.
