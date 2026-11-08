# Phase 7: long-running custody routes

This is R7's per-kind route selection and R9's decision inventory. It governs
the remaining H7 implementation; it is **not evidence that these routes have
landed**. The [delivery tracks](milestone-3.md#delivery-tracks) still govern
timing. Graceful close/reopen is track H; interrupted owner publication and
mid-step kills are track C. No D15 refusal is lifted by this document.

## Truth at the handoff

A checkpoint object, memory arena, installed provider, artifact registry or
erased commit authority is a live resource. It is never a serializable run.
Durable data identifies the original work and its acknowledged posture; a fresh
receiver supplies resources and checks current authority before use. Neither a
recorded identifier nor old authorization recreates a capability.

World's original occurrence and retry binding remain the authority for business
effects. Query's obligation must refer to that original when it exists. Before
World admits an attempt, Query owns the work identity; it must allocate and
record that identity without inventing a future World assignment. Reopen must
not mint another identity for the same retained work.

An existing terminal contract authorizes only its own transition. A managed
run's failure does not prove cleanup, a yielded-run release does not prove
business success, and disappearance of memory proves neither. The selected
terminal routes below execute the existing owner transition at the graceful
handoff and retain its real disposition. They do not infer that it happened
after process death. Pending or refused release stays a durable obligation.

## R7: per-kind routes

Paths in the source column are relative to Query execution's
[`domain_computation`](../../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/).
Bridge and Signal entries name their separate owning runtime explicitly.
"Data" means a new owner-defined versioned contract with cold readmission;
"terminal" means an existing owner transition, durably recorded. None of these
new durable contracts is implemented merely by selecting its route here.

| Kind and current owner | Selected route | Required durable handoff and reopen consequence |
|---|---|---|
| Admitted/running direct run, `managed_run/direct.rs` | Terminal: the existing `abandon()` produces `Failed` | Preserve logical work identity and actual terminal disposition. Execute cleanup through `WorthQueryDirectRunTerminal::cleanup`; retain a failure separately. A cold terminal inspection must never report `Completed`. |
| Admitted/running workflow managed computation, `managed_run/workflow.rs` and `workflow/running_operations.rs` | Terminal: the existing run abandonment, separate from the authored workflow instance | Retain the computation's failure and its actual artifact/cleanup disposition. Authored workflow progress, approvals and original operation keys remain graph truth; ending a computation does not cancel the workflow instance. |
| Paused direct provider execution, `managed_run/direct_graph_execution.rs` and `direct_graph_terminalization.rs` | Terminal: existing paused-execution abandonment | Release the actual live provider execution and transfer its memory into terminal cleanup. Preserve provider uncertainty and any failed release as obligations. This is not the yielded checkpoint-release route. |
| Paused workflow provider execution, `managed_run/workflow_graph_execution.rs` | Terminal: existing paused-execution abandonment | Release the actual live provider execution, retaining its memory and actual workflow artifact disposition for terminal cleanup. Artifact authority and projection accumulators are not durable data. |
| Yielded direct run, `managed_run/direct_yield.rs` and `direct_yield_cleanup.rs` | Terminal: existing yielded `cleanup()` | Persist the real cleanup receipt and its `Complete` or `RecoveryRequired` posture. Do not turn its same-runtime readmission authority into cold authority or re-run its provider. |
| Yielded workflow run, `managed_run/workflow_yield.rs` and `workflow_yield_cleanup.rs` | Terminal: existing yielded cleanup | Persist the actual `Complete`, `Pending` or `RecoveryRequired` result, retaining artifact release obligations. A pending artifact is not an empty cache. |
| Direct yield recovery, `managed_run/direct_yield_recovery.rs` | Terminal through the exact recorded state | `Running` still owns the paused execution: use its abandonment owner. Only `Terminalized` admits `cleanup_terminalized`. Preserve the distinction and the original failure; never reconstruct the paused trait object. |
| Workflow yield recovery, `managed_run/workflow_yield_recovery.rs` | Terminal through the exact recorded state | `Running` uses paused abandonment; `Terminalized` uses `release_terminalized`. Retain the real frozen-artifact release result, including `Pending`, without recreating an artifact authority from its diagnostics. |
| Direct/workflow readmission recovery, `managed_run/readmission/recovery/` and `readmission/direct_lane/preflight/preparation/progression/recovery/` | Terminal via the existing state-specific cleanup/terminal recovery owner | Finish or retain checkpoint release, restored-execution release, artifact rollback and terminal resources individually. A reassembled live yield remains subject to the yielded route. Cold admission cannot call a live checkpoint's restore method. |
| Provider restore recovery, `managed_run/provider_restore/recovery.rs` | Terminal via existing restore cleanup | Preserve `Retained`, `RetainedAfterRestoredRelease` and `Released` facts. Checkpoint release and restored-execution release are separate; a release panic never becomes success at reopen. |
| Direct terminal cleanup, `managed_run/direct_terminal/` | Data for outstanding terminal obligations | Record run identity and exact `ProviderMemoryRetained`, `QueueRelease` or `BridgeFinalization` failure. Cold discharge uses the owning fresh resource receiver and records its actual outcome; losing an old port cannot discharge it. |
| Workflow terminal cleanup, `managed_run/workflow/terminal/` | Data for outstanding terminal obligations | Preserve `Complete`, `Pending` and `RecoveryRequired`, exact terminal kind and artifact obligations. Query and Bridge reacquire their own live resources; installed artifact registries are not serialized. |
| Failed mutation cleanup, `primary_graph/application_attempt/compare_and_commit/cleanup_required/managed.rs` | Data in `query.obligations` | Preserve actual run identity, failure kind and original business evidence. `Native`, `AffinityInconsistent` and `CompletionInconsistent` remain distinct. Fresh authority must not repair inconsistent evidence by declaring release. |
| Conditional retained decisions, `primary_graph/conditional_operation/signal_decision_reentry/retained_decision.rs` | Data only for the obligation and retained-terminal classes below | Readmit original work and its exact operation/intent association. Recreate process-local eligibility with fresh host time and authority; never persist a clock lease or a provider session. |
| Mandatory owned-async request, Bridge `conditional_execution/owned_async/managed_request.rs` and Signal's owned-async request/queue owner | Data and owner readmission | Retain exact request identity, payload contract and contents, request generation, lineage, completion/retry posture and queue obligation under their existing owners. A dropped admission does not cancel or retire a request. Fresh runtime ports consume readmitted data. |
| Explicitly process-local owned-async request, Bridge `conditional_execution/owned_async/declaration.rs` and `process_local_release.rs` | Terminal: existing opted-in exact release | The declaration must already opt into `with_process_local_retention` and have managed queue capacity. Release through the genuine custodian and completion owner; retain a refusal. This route never applies to a mandatory request. |
| Erased retained commit input, `primary_graph/application_attempt/commit_authority_binding.rs` | Existing original-result DATA; fresh authority at use | Never encode `Arc<dyn Any>`, `TypeId`, or a capability. Refer to the original semantic result and its declared encoded input where applicable. Correction-disclosing original input/meaning membership and both terminal disposal paths remain H9, not an H7 serialization shortcut. |

The provider checkpoint's existing `restore(&self, ...)` requires the retained
live object. Its evidence includes a process-local checkpoint ordinal, provider
generation, call and session; that evidence is not a portable checkpoint
contract. The terminal route deliberately avoids promising provider continuation
after reopen without such a contract. It preserves the original computation's
outcome and cleanup instead of reporting a successful recomputation.

## R9: exhaustive retained-decision inventory

The names below are the actual variants, not aliases for a smaller enum.
The last column specifies the consuming owner; it does not claim a durable
consumer exists yet. Any implementation match is exhaustive, without `_`.

| Actual variant | Class | Required retained meaning / consumer |
|---|---|---|
| `OperationPreparationPending` | Durable obligation | Preserve its actual preparation/publication stage and existing World custody when present, including `PublicationRecoveryRequired`. Only work before World admission uses Query-only identity; never invent a future assignment or label every pending stage pre-effect. |
| `OperationPreparationRefused` | Durable obligation while its refused owner retains custody | Preserve the exact refusal and retained preparation. Close/reopen must not erase an owner merely because the attempt has no effect. Terminal disposal requires that owner's lawful no-effect/release transition. |
| `OperationCleanupRequired` | Durable obligation | Managed cleanup record, actual run and failure kind, separate from original business fate. |
| `OperationProductStale` | Durable obligation | Original effect and pending product advancement; not permission to execute another effect. |
| `OperationNoEffect` | Terminal | Retain the exact cause when the caller can still request it; bind to the original key/intent. |
| `Eligible` | Process-local | Re-derive against current durable truth and host time. |
| `Suppressed` | Process-local | Re-derive; omission is not settlement of another obligation. |
| `Deferred` | Process-local | Re-derive with a freshly acquired clock lease. |
| `OperationRetryable` | Durable obligation | Original key, operation intent and typed retry cause; fresh admission precedes continuation. |
| `OperationCommitRetryable` | Durable obligation | Original key/intent and commit denial; retry cannot publish a duplicate original. |
| `OperationExecutionControlRetryable` | Durable obligation | Original work and typed execution-control stop; fresh request controls are supplied at use. |
| `OperationBackpressured` | Process-local | Re-evaluate snapshot/retention/provider admission; its diagnostic never settles separately retained work. |
| `OperationControlStopped` | Terminal | Preserve typed cancellation, timeout or authorization stop when still caller-addressable. |
| `OperationTerminalFailure` | Terminal | Preserve the actual terminal failure when still caller-addressable. |
| `OperationProductUnpublished` | Durable obligation | Join the exact World unpublished original and retry binding, using its recovery owner. |
| `OperationIndeterminate` | Durable obligation; ambiguous-effect recovery is track C | Retain uncertainty and original association. Track H must never infer a fate for an unclean effect. |
| `OperationCommitted` | Terminal | The original World occurrence/idempotency record remains authoritative; retain an addressable Query association without a competing business result. |
| `OperationAlreadyCommitted` | Terminal | Same original occurrence as the retry; no new effect and no replacement identity. |
| `Failed` | Terminal | Preserve the Bridge denial when still caller-addressable; a diagnostic is not a cleanup receipt. |
| `InterruptedDuringReentry` | Durable obligation | Preserve work identity and last acknowledged posture. Do not convert absence of a live reentry frame into a no-effect claim. |

The extra preparation and committed variants are intentional entries. In
particular, `WorthQueryProductPreparationRefused` retains an owner, so treating
every refusal as an empty terminal would lose custody. `OperationCommitted`
does not require duplicating World's durable business result.

## Implementation boundaries

There is one Query obligation history in `query.obligations`. The present
`program_occurrence/support_lifecycle/journal` owns support records and their
fold. New run/decision records require an explicit typed envelope, codecs and
family folds; silently sending new bytes through a support-only decoder is not
an extension. Support status and custody counts must preserve their existing
cold admission. Store continues to transport opaque owner records.

The destination separates record families from live consumers:

```text
worth-foundational                         [future meaning]
  home-scoped run/obligation identities and versioned retained postures
worth-proof                                [future legality]
  acknowledged record / admitted readmission witnesses
worth-query-execution/domain_computation
  managed_run/                             [existing live owners]
    durable_custody/                        [future run record consumption]
  primary_graph/
    program_occurrence/support_lifecycle/   [existing support fold]
    query_obligations/                      [future shared envelope/routing]
      managed_cleanup/                     [future record/fold/discharge]
      conditional_decision/                [future record/fold/continuation]
    workflow/adoption/                     [existing shared disposition owner]
worth-runtime-bridge/conditional_execution/owned_async/
  readmission/                             [future Bridge association admission]
worth-signal                               [future request/queue data admission]
```

These are destinations, not empty modules to add now. Live counters, ports,
`Drop` and resource tables remain in their runtime. New public contracts pass
through their crate facade. Replay is not an ordinary recovery entrance.
No additional journal, Home-only workflow installer or provider-object codec is
introduced. R7/R9 work does not move H9's original-meaning membership into H7.

For each family the implementation must wire issuance, publication, cold fold,
fresh-receiver admission, continuation/terminal inspection and discharge in one
slice. Persist the obligation before handing custody to a caller, including a
cleanup refusal that would prevent graceful close. A frame or enum without its
ordinary consumer is unfinished. Original uncertainty and release failures stay
typed; a cold receiver cannot acknowledge release just because process-local
memory has disappeared.

## Required evidence and remaining dependencies

The real Host directory opener is the qualification boundary. A court retains
authentic work, closes gracefully, reconstructs with fresh authorities and
checks the same original identities, typed posture and absence of duplicate
effects. Terminal-route courts must prove the owner transition occurred before
close and the same terminal is visible after reopen. Cleanup cases must show the
persisted outstanding record, an actual discharge, then successful close.
Mandatory and opted-in process-local owned-async controls must be paired so the
release door cannot accidentally admit the mandatory case.

R9 evidence must distinguish unchanged eligibility, retained retry/cleanup,
terminal results and original business fate. The temporal court must retry its
original intent after filesystem reopen and observe exactly one occurrence;
canonical key-value tests alone do not prove that journey.

At the current branch, Host's H5-held `application_home/support.rs` still
refuses Phase 7 directory families, and temporal/owned-async definitions depend
on H5's Home Signal stage. These public directory courts remain unrun until
those seams merge. This document reports source inspection only. Workflow
dispositions, durable cleanup consumers, R9 cold consumption and the R7 routes
above still require implementation and qualification.
