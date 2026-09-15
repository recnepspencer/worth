# Decisions — fix/worth-signal-wasm-hearth-issues

Branch from `origin/master` (0111969816). Each entry records a call made
without asking, why, and how it is enforced.

## Field issues from Hearth (`_docs/worth-signals-wasm-issues.md`)

1. **Mutation response digest no longer throws on `Date`.** Line values are
   JSON values, so a loaded or mutation-response value is canonicalized once to
   the JSON it would serialize to (`toJSON` honored like `JSON.stringify`,
   `undefined` members dropped, non-finite numbers `null`) before it is
   committed (`package-src/product/resource/lines/state/line_value_canonical_json.ts`).
   Values JSON would misrepresent (`Map`, `Set`, typed arrays, boxed
   primitives, functions) are refused with the reason and the line rejects
   with that reason instead of hanging pending. A rejected line keeps the
   message from both `Error` instances and `{ code, message }` boundary
   refusals (`line_rejection_message.ts`).
2. **`awaitSettlement` resolves `timedOut` instead of rejecting.** The deadline
   is the waiter's result, not the line's: the line stays pending and settles
   later. The result carries `resultKind: "timedOut"` with the pending
   operation and continuity as the types always advertised; `drainAuthoredWork`
   never extends a deadline.
3. **Runtime-wide invalidation.** `resources.invalidateAll()` and
   `resources.refreshAll()` sweep every materialized line of every family,
   keyed entries included, and report scope `runtimeAll` / freshness
   `manualRuntimeInvalidateAll` so readers can tell the sweep from a family or
   line invalidation. Visible values stay. Both return the line count.
4. **Form action debug reads are lazy.** `form.debugAction(id)` computes
   `verification` and `digest` on first access and memoizes them, so a React
   render that only reads `pending` or `latestExecution` no longer pays
   O(total form history). The React form hook builds action bindings through a
   dedicated `signals_form_actions.ts` module instead of rebuilding every
   binding on each summary change.

## Runtime and history

5. **Exact restore tokens are a bounded pending registry.** Each realm keeps
   its 64 most recent pending exact artifacts; minting the 65th evicts the
   oldest. Redeeming an evicted, consumed, or discarded token fails
   `restoreTokenNotPending`; a token the realm never issued fails
   `invalidInput`. `discardRestoreToken(token)` / `pendingRestoreTokenCount()`
   on the raw module, `history.discard_exact_artifact(...)` and
   `adapters.discardExactRuntimeEnvelope(...)` on the product surface, release
   artifacts a caller abandons. Unbounded growth was the alternative.
6. **Restoring a runtime snapshot reactivates its branch.** A snapshot belongs
   to the branch it was captured on; a refused restore leaves the active
   branch unchanged.
7. **Exact runtime envelope exports are root-branch evidence.** The native
   runtime refuses them on a child branch. The worker-first root import
   context now only mints exact artifacts while the root branch is active and
   exposes `requireRuntimeEnvelopeArtifact()`, which throws the same
   `invalidInput` denial the compatibility deployment throws, so
   `adapters().exportRuntimeEnvelope()` and `graph.exportSnapshot()` behave
   identically on both deployments. Definitions have no branch preflight and
   stay cached. Fixed alongside: `hostCapabilityTransportReport` read
   `.definitions` off a value that already was the definition envelope.
8. **Snapshot capture is an observation and never consumes the merge ledger.**
   Native `worth-signal` cleared the live branch mutation ledger on every
   snapshot capture (active branch, stored branch, owner cell). The
   worker-first root mints snapshots after every mutation, so every later
   merge adopted nothing. Now only the stored snapshot packet carries the
   cleared ledger (so restoring it still reinstates the boundary, per
   `active_restore_reinstates_branch_merge_ledger_boundary_for_later_fast_forward_merge`);
   the live ledger keeps its pending records. Enforced by native
   `snapshot_capture_keeps_pending_source_mutations_merge_visible` and wasm
   `branch_merge_after_observation` (three tests, including the
   observe-after-every-step flow the worker-first root performs).
9. **A restored store completes its uninitialized recipes.** A merged branch
   state resets every recipe to recompute from the merged inputs, but a read
   only invalidated the node it asked for, so a published output read through
   a natively-clean intermediate computed returned `Null` (and stayed `Null`
   after later writes). `restore_runtime_store_snapshot` now marks every
   uninitialized recipe dirty and evaluates the standing demand (published
   outputs) in the same step, so a switched-to, merged, or restored branch is
   complete before anything observes it; other computeds recompute on their
   next read. Evaluating at restore rather than at first read keeps the
   worker-first root's cached diagnostics honest: its refresh reads values
   after it captures summaries, so a read-time recompute would have been
   invisible to that refresh (the diagnostics history would instead have been
   wiped by reordering the refresh, which is why that alternative was
   rejected). O(recipes) plus one evaluation of the demanded nodes per restore
   with uninitialized recipes; restores of complete stores do nothing.
10. **Published outputs are standing demand.** A transaction that reaches a
    published output recomputes it at commit so worker delivery and the
    committed truth include it. Callback dependency patches are evaluated
    topology (`set_evaluated_dependencies`), not structural replacements, so a
    node keeps the value the transaction computed.
11. **Host tip ingress exists on `mainThreadCompatibility`.** Same surface
    shape as worker-first (`compatibility_host_tip.ts`): writes applied in one
    synchronous transaction, empty `projectedReadableIds`, worker batch only
    validates epochs, `settleAuthoredWork()` resolves immediately. Line
    bindings and forms use one path in both deployments.
12. **Diagnostics and history views refresh after a portable import.** The
    native runtime exposes `diagnostics_views` refresh so replay, lineage, and
    diagnostics describe the imported runtime rather than the pre-import one.

## Parity evidence

13. **Run counters are excluded from worker/compat parity.** The comparators
    in `worker_first_root_surface_comparators.mjs` skip executor usage counts
    and `plans_built` in the graph summary (and executor usage in the
    performance summary): they count execution and planner runs a runtime has
    performed, and a worker-first root runs both for every cached artifact it
    refreshes while a compatibility runtime runs them per direct read.
    `graph_storage_dependency_segments_rewritten` joins the compaction count
    and subscriber segments already excluded: all three move only when storage
    compaction runs, which depends on the run history. Every evaluation
    (`fresh_compute_count`, `nodes_recomputed`), invalidation, stage, task,
    and value counter still compares exactly; the worker-first refresh costs
    roughly 30 extra planner runs per mutation on a three-node graph, which is
    the price of its refresh-everything design and not changed here.
15. **React store snapshots are stable and still enter root read.** Since
    `f9b8129728` ("React attach honesty") `getSignalSnapshot` entered
    `signals.read` on every call so superseded or foreign handles throw, but
    the unsubscribed path (React renders before it subscribes, twice in
    development) replaced the snapshot with each fresh read, so object-valued
    signals handed `useSyncExternalStore` a new reference per call, and
    `react/store.runtime.test.mjs` still asserted the pre-August single cached
    read and had failed since. Now every call still enters root read; a
    subscribed, current entry serves its cached reference (the watch is the
    change channel); an unsubscribed or invalidated entry adopts the fresh
    value but keeps the cached reference when the value is structurally equal
    (`react/signal_snapshot_equality.ts`, O(value), only on that path). The
    test enforces the read count, reference stability, the foreign-handle
    throw, and adoption after a watch notice.
14. **Public surface policy.** `package/types/host_tip_surface.d.ts` belongs to
    `core.callable` with `compatibility_host_tip_commit.test.mjs` as evidence;
    every drifted baseline was regenerated after reviewing the docs.

## Pre-existing state of master (not introduced here)

- Native `cargo test -p worth-signal --lib` has 59 failing tests on master;
  the set is unchanged on this branch (compared by name).
- On master, 6 of 8 signals-runtime suites had failures, including
  `worker_first_callable_root_surfaces.test.mjs` failing at its first
  assertion; commits e736170e24 / f9b8129728 / 221861e0d4 are where those
  regressions landed.
