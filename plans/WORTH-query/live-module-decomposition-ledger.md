# Live Module Semantic Decomposition Ledger

Status: scoped decomposition verified 2026-08-03; unrelated repository-red
baselines are recorded in the completion log

Source under correction:
`workspaces/worth-query/crates/worth-query/src/live/mod.rs` (5,979 lines)

## Governing truth

This ledger implements the already-committed topology in Milestones 5 and 5.1.
It does not redesign live-query semantics. The public contract remains the
existing Query facade exports; `crate::live` remains an internal aggregation
surface.

The adversarial constraint is structural: a future live-family, locality
strategy, delivery contract, diagnostic, or certification lane must have one
predictable insertion point without enlarging a catch-all module or creating a
second live authority. The behavioral constraint remains convergence with
fresh canonical query execution from the same basis.

## Invariants

- Preserve every existing public type, function, enum variant, method, error,
  visibility boundary, and facade export unless compilation proves an internal
  path was never part of the contract.
- Preserve Query ownership of live meaning. Signal scheduling and bridge
  routing remain external authorities.
- Keep query-shaped patches; do not introduce raw CDC or transport-shaped
  payloads.
- Keep promotion, locality, delivery, replay, and certification as one proof
  chain, not parallel implementations.
- `live/mod.rs` becomes an aggregation facade only. It may declare modules and
  re-export symbols; it may not implement live behavior.
- Every new or touched Rust file must be at most 400 lines. Remove the old
  `live/mod.rs` line-cap exemption only after the complete subtree passes the
  guard.
- Use the narrowest internal visibility that permits collaboration. Cross-file
  fields may use `pub(in crate::live)`; no extraction may widen a public
  constructor or proof boundary.
- Existing tests must be moved by proof obligation, not deleted or weakened.

## Destination tree

```text
live/
  mod.rs                              # internal facade: module declarations + re-exports only
  promotion/
    mod.rs                            # promotion family facade only
    family.rs                         # LiveQueryFamily vocabulary
    descriptor.rs                     # LivePromotionDescriptor
    plan.rs                           # LiveQueryPlan state and basic accessors
    admission.rs                      # preflight-to-live promotion boundary and errors
  relevance/
    mod.rs                            # relevance family facade only
    query_contract.rs                 # QueryFieldKey and QueryRelevanceContract
    bridge_change.rs                  # bridge delta/slice/transition vocabulary
    classification.rs                 # relevant/irrelevant classifications and suppression
  refresh/
    mod.rs                            # refresh policy facade only
    admission.rs                      # refresh admission matrix, fallback, typed denials
    delivery_width.rs                 # patch-width assessment and overflow resolution
    coalescing.rs                     # coalescing admission decisions and errors
  identity/
    mod.rs                            # live identity facade only
    subscription.rs                   # subscription and change-sequence identities
    progress.rs                       # ordinal, start/progress basis, replay digest progression
  patches/
    mod.rs                            # patch family facade only
    detail.rs                         # detail delta and outcome construction
    ordered_collection.rs             # membership/order patch and outcome construction
    bounded_materialization.rs        # bounded-scope patch and outcome construction
    envelope.rs                       # payload, patch identity, canonical envelope
  locality/
    mod.rs                            # locality family facade only
    scope_contract.rs                 # scope identity, predicate, locality-aware relevance
    admission.rs                      # admitted classes, cost/breadth/widening policies
    matching.rs                       # region/partition match and widening outcomes
    planning.rs                       # RegionScopedLivePlan and planning report
    execution.rs                      # region execution report/envelope/error translation
  delivery/
    mod.rs                            # delivery-contract facade only
    query_contract.rs                 # query-shaped delivery identity and locality outcome
    stream_admission.rs               # request, admitted consumer, stream contract identity
    stream_lowering.rs                # lowered contract construction and cost posture
    member_projection.rs              # member projection and window compatibility
    replay_record.rs                  # delivery-contract replay record and regional bundle
  telemetry/
    mod.rs                            # live telemetry facade only
    counters.rs                       # counter storage and public scalar observations
    digest.rs                         # activity, absorption, and canonical digest projection
    evidence.rs                       # outcome/error/locality/stream-to-counter translation
  execution/
    mod.rs                            # execution family facade only
    report.rs                         # live execution report and envelope
    change.rs                         # execute_live_change orchestration
    replay.rs                         # replay_live_sequence orchestration and run artifacts
    digest.rs                         # result/delivery/replay digest construction mechanics
  certification/
    mod.rs                            # certification facade only
    lanes.rs                          # success and rejection lanes
    artifact.rs                       # MilestoneFiveLiveArtifact
    adapter.rs                        # named certification adapter entrypoints
  tests/
    mod.rs
    promotion.rs
    progress.rs
    detail_patch.rs
    collection_patch.rs
    materialization_patch.rs
    delivery_policy.rs
    artifact.rs
    replay.rs
    locality.rs
    stream_delivery.rs
  region_scoped.rs                    # existing test-only region implementation; unchanged unless required
```

The implementer may merge adjacent leaves only when the resulting file still
has exactly one reason to change and remains comfortably below 400 lines. It
may introduce a more specific leaf when a listed file would exceed the cap. It
may not replace named leaves with `types`, `model`, `common`, `helpers`,
`shared`, or another bucket.

## Extraction ledger

| ID | Current source responsibility | Current range | Destination owner | Required boundary | Status |
| --- | --- | ---: | --- | --- | --- |
| QL-01 | family, projected fields, relevance contract | 18-248 | `promotion/family.rs`, `relevance/query_contract.rs` | Relevance consumes validated query meaning; it does not revalidate it | Verified |
| QL-02 | bridge field/relation/locality deltas and transitions | 249-567 | `relevance/bridge_change.rs` | Bridge evidence is input, never Query authority | Verified |
| QL-03 | relevance/suppression/refresh classifications | 568-714 | `relevance/classification.rs`, `refresh/admission.rs`, `refresh/coalescing.rs` | Classification finishes before patch construction | Verified |
| QL-04 | live promotion descriptor | 715-809 | `promotion/descriptor.rs` | Descriptor derives only from admitted one-shot planning | Verified |
| QL-05 | subscription, sequence, basis, and replay identities | 810-969 | `identity/subscription.rs`, `identity/progress.rs` | Progress is monotonic and basis-bound | Verified |
| QL-06 | live plan state, delivery-width policy, and family outcome dispatch | 970-1469 | `promotion/plan.rs`, `refresh/delivery_width.rs`, `patches/*` | Plan fixes policy; execution consumes it without rediscovery | Verified |
| QL-07 | locality vocabulary, budgets, plan, reports, and envelope | 1470-2048 | `locality/*` | Locality extends the admitted live plan and cannot form a second live lane | Verified |
| QL-08 | stream consumer and query delivery contracts | 2049-2473 | `delivery/*` | Consumer contract remains query-shaped through lowering | Verified |
| QL-09 | regional counters and typed live errors | 2474-2536 | `telemetry/*`, owning error modules | Errors preserve denial kind; counters remain derived evidence | Verified |
| QL-10 | `LivePolicyCounters` storage, digesting, aggregation, and evidence constructors | 2537-3469 | `telemetry/counters.rs`, `telemetry/digest.rs`, `telemetry/evidence.rs` | Counter mapping cannot become semantic decision authority | Verified |
| QL-11 | patch payload families and patch envelope | 3470-3813 | `patches/*` | Each family owns its own patch semantics; envelope only composes | Verified |
| QL-12 | replay inputs/runs, execution reports/envelopes, errors | 3814-3998 | `execution/replay.rs`, `execution/report.rs`, owning error modules | Replay advances sealed progress and preserves exact digests | Verified |
| QL-13 | certification lanes, artifact, and adapter | 3999-4414 | `certification/*` | Certification consumes production execution; no shadow semantics | Verified |
| QL-14 | promotion, execution, replay, artifact construction, digest mechanics | 4415-4810 | `promotion/admission.rs`, `execution/*`, `certification/artifact.rs` | Orchestrators read as named phases; digest mechanics do not decide policy | Verified |
| QL-15 | inline tests | 4811-5979 | `tests/*` by proof family | Every moved test retains the same setup, cause, and independent observation | Verified |
| QL-16 | internal facade and line-cap debt | whole file | `live/mod.rs`, line-cap allowlist | Facade aggregates only; exemption is deleted after all files comply | Verified |

## Function-level composition findings to close

These existing functions require explicit review during extraction. Moving them
unchanged is insufficient if the same mixed responsibility remains:

- `detail_live_outcome`, `ordered_collection_live_outcome`, and
  `bounded_materialization_live_outcome`: separate classification, payload
  construction, width assessment, and final outcome assembly where those steps
  are currently interleaved.
- `LivePolicyCounters::digest_parts`: split canonical counter projection into
  named counter families; preserve one deterministic terminal ordering.
- `LivePolicyCounters::absorb`: keep aggregation mechanical and separate from
  semantic classification.
- `MilestoneFiveLiveAdapter::progress_advance_lane`: expose progression,
  execution, and lane construction as named steps.
- `execute_live_change`: expose family compatibility, progression, semantic
  execution, patch envelope construction, and report assembly as named steps.
- `patch_envelope_from_payload`: replace the six-argument mechanical signature
  with one named construction basis if that can be done without widening the
  public API.

## Test ledger

| Proof family | Existing evidence to preserve | Destination |
| --- | --- | --- |
| Promotion admission | detail/collection/materialization promotion and CDC rejection | `tests/promotion.rs` |
| Progress identity | monotonic advance and non-monotonic rejection | `tests/progress.rs` |
| Query-shaped patches | projected detail, reorder/membership, bounded-scope outcomes | family-specific patch test files |
| Suppression | irrelevant relation, no-op membership, off-region suppression | nearest owning relevance/locality test file |
| Width/coalescing/refresh | overflow outcomes and forbidden admission classes | `tests/delivery_policy.rs` |
| Artifact honesty | artifact summary, counter-bound digest, report/envelope content | `tests/artifact.rs` |
| Replay | step bundles and progress advance | `tests/replay.rs` |
| Locality | admission, widening, breadth budget, slice compatibility | `tests/locality.rs` |
| Stream delivery | admitted stream shapes, member/window overflow, replay record | `tests/stream_delivery.rs` |

No new integration-test crate is required for this structural change. Existing
crate tests and certification harnesses are the behavioral evidence; the
line-cap and boundary tools are the structural evidence.

## Ordered implementation

1. Create the module skeleton and empty aggregation facades.
2. Extract leaf vocabularies and identity types; compile.
3. Extract relevance, refresh, patch, locality, and delivery responsibilities;
   compile after each coherent family.
4. Extract telemetry, execution, replay, and certification; compile.
5. Split inline tests by proof family and run the focused crate suite.
6. Reduce `live/mod.rs` to declarations/re-exports, remove its allowlist entry,
   format, and run all closure gates.

## Closure evidence

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query`
- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --test compile_certification`
  only if public visibility or compile-fail boundaries changed
- `python scripts/quality/scrutinize_rust_functions.py workspaces/worth-query/crates/worth-query/src/live --relative-to .`
- `bash scripts/ci/check_workspace_rust_line_caps.sh`
- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`
- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`
- `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml --all -- --check`
- inspection proving every `live/**/*.rs` file is at most 400 lines and
  `live/mod.rs` contains no behavioral implementation

## Completion log

Verified from the integrated `ui` worktree on 2026-08-03:

- The 5,979-line source is now a 37-line behavior-free facade over 59 live
  Rust files. `LiveQueryPlan` has one definition, in `promotion/plan.rs`.
- Every non-exempt live file is at most 400 lines. The only over-cap file is
  the inherited test-only `region_scoped.rs` at 641 lines; its existing
  allowlist entry remains. The obsolete `live/mod.rs` entry is gone.
- The original inline live tests were moved without losing test names. The
  integrated focused lane passed 74 tests with zero failures.
- Independent QA reopened the work after extraction. It split the primary
  live counter projection into ordered semantic families and removed duplicate
  certification construction. A static comparison proves all 52 digest labels
  remain present in their original order, and the coalesced lane now constructs
  one patch envelope and one counter snapshot for both report and replay.
- `cargo fmt` for the Query workspace, `boundary-check`, and `agent-context
  check` pass. Function scrutiny reports zero scan errors; its nine remaining
  candidates are exhaustive typed classifiers, mechanical aggregation,
  inherited test-only region logic, or a scenario proof test.

Repository-red baselines, reproduced independently before integration:

- The full Query lane retains pre-existing lower-runtime routed-boundary
  failures involving `runtime/runtime_declarations.rs` and raw
  `worth_runtime_bridge` access. A representative exact failure was reproduced
  on the untouched branch before either decomposition commit.
- Compile certification passes 11 tests and retains one pre-existing
  `reference_consumer_residue` failure in WORTH UI raw bridge/signal imports.
  The same exact residue failure was reproduced on the untouched branch.
- The workspace line-cap script remains red on unrelated repository debt (110
  failures in the integrated scan). It reports no new live-subtree violation;
  `region_scoped.rs` is explicitly allowlisted.

These baselines prevent a claim that the whole repository is constitution
green. They do not leave an open defect in this scoped decomposition.
