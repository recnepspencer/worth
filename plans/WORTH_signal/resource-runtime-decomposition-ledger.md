# Resource Runtime Semantic Decomposition Ledger

Status: scoped decomposition verified 2026-08-03; unrelated repository-red
baselines are recorded in the completion log

Source under correction:
`crates/worth-signal/src/logic/transaction/runtime/state/resource.rs`
(5,220 lines)

## Governing truth

This ledger realizes the decomposition already required by Signal Milestones B
and C. Those specifications explicitly forbid request, completion, retry,
timeout, cancellation, revalidation, retention, diagnostics, and replay policy
from remaining collapsed in one `resource.rs`.

The adversarial constraint is lifecycle convergence: overlapping generations,
out-of-order completion, cancellation, retry, revalidation, timeout, branch
restore, retention truncation, and replay must preserve one generation-safe
derived resource truth with the same denials and diagnostics. Physical
decomposition must not create competing state owners or bypass the existing
`ResourceRuntimeState` authority.

## Invariants

- `ResourceRuntimeState` remains the single runtime owner of resource-derived
  lifecycle state. Extraction creates multiple inherent `impl` blocks, not
  multiple state authorities.
- Preserve every existing method signature and visibility used by runtime,
  transaction, facade, tests, and certification.
- Resource state remains derived and reconstructible; no extracted module may
  acquire domain truth, external transport, persistence, or host-fetch
  authority.
- Validation, policy resolution, and admission precede effects. Completion,
  cancellation, rejection, timeout, retry, and revalidation retain their typed
  success and denial topologies.
- `resource.rs` becomes an internal facade only. It may declare modules and
  re-export `ResourceRuntimeState` plus the exact internal plan types consumed
  by sibling runtime modules.
- State fields shared inside the resource subtree may be `pub(super)` from
  `resource/state.rs`; no extraction may widen them beyond the resource parent.
- Every new or touched Rust file must be at most 400 lines. Remove the old
  `resource.rs` line-cap exemption only after the complete subtree passes.
- No `policy.rs`, `lifecycle.rs`, `operations.rs`, `types.rs`, `helpers.rs`, or
  similar file may absorb multiple independent policy or lifecycle families.

## Destination tree

```text
state/
  resource.rs                          # internal facade: declarations + narrow re-exports only
  resource/
    state.rs                           # ResourceRuntimeState storage and Default
    identity_issuance.rs               # monotonic descriptor/request/generation/ordinal issuance
    observation/
      mod.rs                           # observation facade only
      summary.rs                       # runtime and node summary reads
      retained_availability.rs         # retained history/denial/retry availability reads
      output_continuity.rs             # terminal visibility and continuity recording
    policy/
      mod.rs                           # policy runtime facade only
      registry.rs                      # frozen registry install, validation, freeze, lowering
      restore_compatibility.rs         # compatibility classification and readmission
      diagnostics.rs                   # effective diagnostics decision and budget posture
      replay.rs                        # replay digest bases and reconstruction report
    restore/
      mod.rs                           # restore facade only
      epoch.rs                         # branch epoch bump and in-flight rekeying
    safe_point.rs                      # safe-point observation and counters
    managed_queue.rs                   # queue binding, enqueue/dequeue, bounded mutation
    timeout/
      mod.rs                           # timeout facade only
      plan.rs                          # resolved timeout and scheduled wake admission types
      wake.rs                          # active wake lookup, binding, stale-after wake ownership
      heartbeat.rs                     # extension candidate/admission/denial
      admission.rs                     # timeout lifecycle admission
      denial.rs                        # typed timeout denial construction
    request/
      mod.rs                           # request facade only
      declaration.rs                   # resource-node declaration
      admission.rs                     # request admission from lowered descriptor
      coalescing.rs                    # equivalent-intent coalescing
      supersession.rs                  # active request supersession
    retry/
      mod.rs                           # retry facade only
      backoff.rs                       # deterministic retry delay resolution
      schedule.rs                      # schedule request and prepared retry
      admission.rs                     # prepared retry admission
      denial.rs                        # retry schedule/admission denial recording
    revalidation/
      mod.rs                           # revalidation facade only
      proof.rs                         # typed cause-specific proof minting/validation
      preparation.rs                   # cause-specific preparation and shared preview
      admission.rs                     # prepared admission and coalescing
      denial.rs                        # revalidation denial recording
    completion/
      mod.rs                           # completion facade only
      admission.rs                     # scalar envelope validation/admission
      batch.rs                         # batch admission orchestration
      staging.rs                       # admitted/denied staging and rollback
      commit.rs                        # staged completion commit
      denial.rs                        # completion denial classification and record creation
    retention/
      mod.rs                           # retention facade only
      availability.rs                  # retained/pruned availability classification
      compaction.rs                    # budgeted lifecycle-history compaction
    cancellation/
      mod.rs                           # cancellation facade only
      admission.rs                     # cancellation request boundary
      application.rs                   # cancellation footprint and state effects
      denial.rs                        # cancellation denial construction
    rejection/
      mod.rs                           # rejection facade only
      admission.rs                     # rejection boundary and state transition
      denial.rs                        # rejection denial construction
```

The implementer may add a more specific leaf when a listed responsibility
cannot stay below 400 lines. Adjacent leaves may be merged only if they share
authority, lifecycle, failure topology, and replacement fate and the resulting
name still predicts exclusions. Bucket substitutions are forbidden.

## Extraction ledger

| ID | Current source responsibility | Current range | Destination owner | Required boundary | Status |
| --- | --- | ---: | --- | --- | --- |
| SR-01 | prepared revalidation/retry and timeout plan types | 65-179 | owning `revalidation/`, `retry/`, and `timeout/plan.rs` leaves | Preparation types are phase proofs, not generic option bags | Verified |
| SR-02 | effective diagnostics policy | 180-216 | `policy/diagnostics.rs` | Diagnostics policy controls richness, never lifecycle legality | Verified |
| SR-03 | resource runtime storage and defaults | 217-296 | `state.rs` | One state owner; fields visible only inside resource subtree | Verified |
| SR-04 | replay digest bases | 297-434 | `policy/replay.rs` | Digest records are derived projections, not replay authority | Verified |
| SR-05 | output continuity, terminal visibility, summary/availability reads | 435-748 | `observation/*` | Reads observe lifecycle truth without mutation authority | Verified |
| SR-06 | policy restore compatibility and declaration lowering | 749-971 | `policy/restore_compatibility.rs`, `policy/registry.rs` | Policy is frozen/lowered before hot-path consumption | Verified |
| SR-07 | diagnostics posture and replay reconstruction | 972-1266 | `policy/diagnostics.rs`, `policy/replay.rs` | Cold reconstruction cost remains separate and explicit | Verified |
| SR-08 | restore epoch and safe-point observation | 1267-1404 | `restore/epoch.rs`, `safe_point.rs` | Restore rekeys generations; safe points expose typed evidence | Verified |
| SR-09 | managed queues | 1405-1554 | `managed_queue.rs` | Queue capacity and mutation denial stay bounded and explicit | Verified |
| SR-10 | timeout/stale-after wake lookup and heartbeat extension | 1555-1746 | `timeout/wake.rs`, `timeout/heartbeat.rs` | Temporal wakes remain framework-owned | Verified |
| SR-11 | node declaration and request admission | 1747-2014 | `request/declaration.rs`, `request/admission.rs` | Descriptor/policy admission precedes in-flight effects | Verified |
| SR-12 | retry delay and revalidation proof validation | 2015-2507 | `retry/backoff.rs`, `revalidation/proof.rs`, owning denial leaves | Each revalidation cause retains a distinct typed proof | Verified |
| SR-13 | revalidation preparation, admission, and coalescing | 2508-2936 | `revalidation/preparation.rs`, `revalidation/admission.rs` | Preview/classification completes before state mutation | Verified |
| SR-14 | retry scheduling and prepared retry admission | 2937-3229 | `retry/schedule.rs`, `retry/admission.rs` | Budget charge and timeout plan are consumed once | Verified |
| SR-15 | scalar and batch completion admission | 3230-3483 | `completion/admission.rs`, `completion/batch.rs` | Stale/hostile envelopes fail before commit effects | Verified |
| SR-16 | retention availability and budgeted compaction | 3484-3815 | `retention/availability.rs`, `retention/compaction.rs` | Compaction preserves typed availability and reconstruction posture | Verified |
| SR-17 | completion staging, rollback, and commit | 3816-3974 | `completion/staging.rs`, `completion/commit.rs` | Staged proof types keep invalid commit order uncallable | Verified |
| SR-18 | cancellation and timeout admission | 3975-4169 | `cancellation/admission.rs`, `timeout/admission.rs` | Terminal transition cause stays explicit | Verified |
| SR-19 | cancellation/rejection/timeout/retry/revalidation/completion denials | 4170-4900 | owning family `denial.rs` leaves | No universal denial builder; each family owns its topology | Verified |
| SR-20 | cancellation effects, supersession, and equivalent-intent coalescing | 4330-5095 | `cancellation/application.rs`, `request/supersession.rs`, `request/coalescing.rs` | Effects remain after admission; coalescing requires exact equivalence | Verified |
| SR-21 | identity and ordinal issuance | 5096-5164 | `identity_issuance.rs` | Runtime alone mints identities; callers cannot reconstruct them | Verified |
| SR-22 | error translation and internal facade | 5165-end, whole file | nearest owner, `resource.rs` | Facade aggregates only and preserves exact visibility | Verified |

## Function-level composition findings to close

The following existing functions are not cleared by merely moving them:

- `reconstruct_replay_summary`: separate retained-state collection, canonical
  digest projection, compatibility decision, and report assembly.
- `admit_resource_request_with_descriptor`: expose eligibility, equivalence,
  supersession, identity issuance, state insertion, timeout binding, and report
  construction as named steps.
- `coalesce_revalidation`: separate exact-equivalence proof, retained lineage,
  timeout/wake handling, counters, and report assembly.
- `admit_prepared_scheduled_resource_retry`: separate budget consumption,
  request progression, timeout binding, and report construction.
- `admit_resource_completion_with_boundary`: separate envelope validation,
  lifecycle admissibility, continuity decision, state transition, and report.
- `compact_lifecycle_history_with_budget`: separate compaction plan,
  availability preservation, application, and report/digest construction.
- `admit_resource_timeout`, `reject_resource_request`, and
  `apply_resource_cancellation`: make classification precede mutation and keep
  terminal effects legible.
- `try_coalesce_equivalent_request_intent`: name exact equivalence,
  coalescing eligibility, timeout transfer, and evidence construction.
- Existing signatures with five or more explicit arguments must be inspected
  for a missing proof-bearing input aggregate. Internal construction-basis
  structs are preferred where they encode one phase; public signatures must
  not change accidentally.

## Proof preservation ledger

| Production claim | Existing evidence family that must remain green |
| --- | --- |
| generation-safe in-flight admission and supersession | resource declaration/admission and out-of-order completion tests |
| typed completion admission/denial and transactional apply | scalar/batch completion, staging, rollback, commit, and UI compile-fail tests |
| framework-owned retry/timeout/cancellation | focused timeout-and-retry, cancellation, inherited deadline, and hostile race tests |
| cause-specific revalidation | explicit, forced, dependency, observer-demand, terminal, fulfilled, and stale-after tests |
| branch/replay reconstruction | branch restore, replay parity, retained availability, and certification bundle tests |
| bounded queues/history | managed queue, in-flight boundedness, retry budget, and retention compaction tests |
| compiler-visible authority boundaries | existing `tests/ui/resource_*` compile-fail suite |

This is a structural refactor. It must use existing product tests as the
behavioral oracle; it must not add alternate composition roots, test-only
constructors, or shadow lifecycle implementations.

## Ordered implementation

1. Create the resource subtree and facade skeleton.
2. Move state storage and phase-proof types to their owners; compile.
3. Extract read-only observation, policy, replay, and restore responsibilities;
   compile and run focused replay/restore tests.
4. Extract queue, timeout, request, retry, and revalidation responsibilities;
   compile and run focused lifecycle tests.
5. Extract completion, retention, cancellation, and rejection responsibilities;
   compile and run focused completion/retention tests.
6. Decompose the named long functions, reduce `resource.rs` to its facade,
   remove its allowlist entry, and run all closure gates.

## Closure evidence

- `cargo test -p worth-signal --lib resource_runtime`
- additional focused resource test filters selected from the touched families
- `cargo test -p worth-signal --test compile_fail` if visibility or proof
  boundaries changed
- `python scripts/quality/scrutinize_rust_functions.py crates/worth-signal/src/logic/transaction/runtime/state/resource.rs crates/worth-signal/src/logic/transaction/runtime/state/resource --relative-to .`
- `bash scripts/ci/check_workspace_rust_line_caps.sh`
- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`
- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`
- `cargo fmt --all -- --check`
- inspection proving every resource subtree Rust file is at most 400 lines and
  `resource.rs` contains no behavioral implementation

## Completion log

Verified from the integrated `ui` worktree on 2026-08-03:

- The 5,220-line source is now an 18-line behavior-free facade over 56
  resource Rust files, including the facade. Every file is at most 400 lines;
  the maximum is 330. The obsolete `resource.rs` allowlist entry is gone.
- `ResourceRuntimeState` remains a single definition in `resource/state.rs`.
  The facade re-exports only that state and the narrow timeout-plan type needed
  by its sibling runtime module.
- Independent QA reopened the work after extraction and corrected residual
  phase collapses in retry scheduling, heartbeat admission, cancellation,
  request coalescing/admission, declaration lowering, and restore rekeying.
  Typed candidates and preparation records now make classification-before-
  effect order visible, while inherited public signatures remain unchanged.
- The integrated full library lane passed 1,016 tests with 23 ignored and zero
  failures (1,039 total). The resource compile-fail boundary proof passed.
  Focused retry, timeout, cancellation, request-admission, revalidation,
  restore, policy-compatibility, and diagnostics lanes also passed.
- `cargo fmt`, `boundary-check`, and `agent-context check` pass. Function
  scrutiny reports zero scan errors and reduced the subtree advisory set from
  32 to 27. The remaining candidates are exhaustive validators/classifiers,
  canonical reductions, coherent single-operation orchestrators, inherited
  compatibility signatures, or narrow aggregate constructors.

Repository-red baseline:

- The workspace line-cap script remains red on unrelated repository debt (110
  failures in the integrated scan). Explicit scoped counting proves all 56
  resource files comply, and the guard reports no touched resource violation.

This baseline prevents a claim that the whole repository is constitution
green. It does not leave an open defect in this scoped decomposition.
