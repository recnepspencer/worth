# Store M4: composed split-barrier TLA+ model, witnesses and trace validator

`StoreM4ScaleReclaim.tla` composes `specs/tla/store_m3/StoreDurability.tla`
(`M3 == INSTANCE StoreDurability`, resolved by `tools/tla/run_tlc.py`'s
`-DTLA-Library`) with every M4 barrier of
`plans/database-foundation/milestone-4.md` section 9.2. Each durable barrier is
its own action, so `Crash` interleaves between any two, including between the
M4 recovery stages.

| Area | Actions |
| --- | --- |
| Commits and Home frontier | `PrepareBranch`, `BeginBranchCommit` (FIFO revalidation = M3 `BeginCommit`), `RefuseStaleCandidate`, `FlushJournal`, `PublishBranchCommit` (= M3 `SealManifest`; composes onto the current frontier) |
| Checkpoints | `BeginCheckpoint`, `SealCheckpoint` (canonical coverage), `CoverCancelCheckpoint` (fuzzy maintenance checkpoint covering a cancelled candidate without root publication), `RetireHistoryPrefix` (consolidation) |
| Branch lifecycle | `ArchiveBranch`, `RetireBranchRef`, `RefuseRetiredBranchName` |
| PinTable | `PublishForkPin`, `AttachChildPin`, `ReleaseChildPin`, `AcquireReaderLoan`, `AcquireGrandchildLoan`, `ReleaseReaderLoan`, `BeginExpireForkLease`, `FinishExpireForkLease` |
| P0 writeback fence | `WritebackIssue`, `WritebackLand`, `WritebackCancel`, `RefuseStaleWriteback` |
| Reclamation | Path A: `ReclaimIntent` -> `PublishFreeSpaceRoot` -> `ReclaimComplete` -> `AdmitToAllocator` (or after crash at `fsroot`, `fsroot_reopened` -> `RecoverResumeReclaimComplete` settles directly to `admitted` without a second `AdmitToAllocator`). Path B: `CancelCandidateExtent` -> `CoverCancelCheckpoint` (or `SealCheckpoint`) -> `AdmitToAllocator`. Then `AllocateExtent` (reuse) |
| Cursors | `IssueRetainedBasisToken`, `ResumeCursor`, `ResumeCursorRefused` (HistoryUnavailable), `ReleaseCursorLoan` |
| Crash and recovery | `Crash` (= M3 `CrashKill`; drops loans, unlanded writebacks, preparations, cursor loans, PinTable residency, staged coverage; transitions `fsroot` -> `fsroot_reopened`), `BeginRecover`, `RecoverWal` (= M3 `AdvanceRecoverStage`; decides the pending commit's fate), `RecoverRestorePins`, `RecoverResumeFreeSpaceRoot`, `RecoverResumeReclaimComplete`, `RecoverServing` (= M3 `FinishRecover`) |

## Commands

Run from the repository root. TLC exit codes: 0 no error; 11 deadlock (for a
trace, a record the model does not allow); 12 invariant violated; 13
temporal property violated.

```sh
M=specs/tla/store_m4
# Safety, both slices must pass (exit 0)
python tools/tla/run_tlc.py check $M/StoreM4ScaleReclaim.tla          # LifecycleSpec
python tools/tla/run_tlc.py check $M/StoreM4ScaleReclaimPhysical.tla  # PhysicalSpec
# Non-vacuity witnesses, each must exit 12 on its WitnessNNotReached
python tools/tla/run_tlc.py check $M/StoreM4Witness1GrandchildSurvival.tla
python tools/tla/run_tlc.py check $M/StoreM4Witness2WritebackFence.tla
python tools/tla/run_tlc.py check $M/StoreM4Witness3ChainedCrashReclaim.tla
python tools/tla/run_tlc.py check $M/StoreM4Witness4ConcurrentPrepArchive.tla
python tools/tla/run_tlc.py check $M/StoreM4Witness5RetainedCursor.tla
# Trace validation: hand-*.ndjson exit 0, hand-rejected-*.ndjson exit 11
python tools/tla/run_tlc.py trace $M/StoreM4ScaleReclaimTrace.tla $M/traces/<trace>.ndjson
```

Results on the work PC (32 workers, TLC 2.19), constants `MaxCommits = 2`,
`MaxCrashes = 2`, `MaxInflight = 1`:

| Check | Result |
| --- | --- |
| `StoreM4ScaleReclaim` (LifecycleSpec) | exit 0; 71,140,642 states generated, 10,459,094 distinct, depth 38, 72 s |
| `StoreM4ScaleReclaimPhysical` (PhysicalSpec) | exit 0; 78,199,509 generated, 9,174,404 distinct, depth 44, 73 s |
| Witness 1 to 5 | exit 12 each, on the named `WitnessNNotReached` |

## Safety invariants and properties

Checked in both slices: `TypeOK`, `M3NoRetiredHistoryGap`,
`M3DurabilityNoLoss`, `M3AtomicityCleanState`, `StoreGCPinSafety`,
`LeaseExpiryNoReaderLoss`, `AddressReuseGenerationAndWriteFenceSound`,
`ReclaimPreservesAllReachableRoots`, `CanonicalCoverageBeforePrune`,
`InterruptedReclaimIdempotent`, `NoBranchResurrection`,
`RetainedBasisCursorImmutable`, `HomeFrontierComposed`,
`PinsRestoredBeforeMaintenance`. Action properties: `M3MonotonicDurability`,
`GenerationMonotone`, `NoBranchResurrectionStep`, `HomeFrontierMonotone`.

## Why two safety slices

The fully interleaved `Spec` cannot be checked exhaustively in reasonable
time. During development it passed 27M distinct states with more than 9M still
queued when it was stopped. Safety is therefore checked over two sub-relations of
`Next`. Their union is `Next`, and each is sound for the invariants it owns:

- `LifecycleNext` drops `WritebackSteps` and `BarrierSteps`
  (`PublishFreeSpaceRoot`, `ReclaimComplete`, `AdmitToAllocator`,
  `AllocateExtent`). Those steps write only physical variables (`cellState`,
  `cellGen`, `inflight`, generations, and phases that are already past
  `"none"`). Every lifecycle guard and lifecycle invariant reads a phase only
  as `phase[e] = "none"`. Projected on lifecycle state, every `Spec`
  behaviour is therefore a `LifecycleSpec` behaviour. This slice owns the pin,
  lease, reach, coverage-at-intent, branch, frontier and cursor laws.
- `PhysicalNext` drops `PinSteps`, `CursorSteps` and `ArchiveSteps`. Those
  steps write no physical variable. They only add `Reach` holders (pins,
  loans, cursor loans, archived roots), which can only disable
  `ReclaimIntent("base")`. With "archived" mapped to "live", volatile `prep`
  projected away (just-in-time `PrepareBranch` immediately before
  `BeginBranchCommit` simulates every commit identically), and pins/cursors
  erased, every `Spec` behaviour projected on physical state is a
  `PhysicalSpec` behaviour. This slice owns the generation, fence, idempotent
  resume and admission-coverage laws.

Witnesses and trace validation always use the full `Spec` and its actions.

## Non-vacuity witnesses (milestone-4.md section 9.2)

Each witness module EXTENDS the model and adds witness-only ghost variables;
every witness step is a `Next` step. A witness cfg may add a `CONSTRAINT`
that only prunes the search (for example "no pin in this witness"). Any trace
TLC prints is still a behaviour of `Spec`.

| Witness | Module | What the printed trace shows |
| --- | --- | --- |
| 1 Grandchild survival and unrelated space return | `StoreM4Witness1GrandchildSurvival` | A is retired while B and a grandchild loan through the fork pin's child overlay share `base`; while the fork lease is `Expiring` and that grandchild loan still holds `base`, `gc` is reclaimed all the way to `admitted`; `base` enters reclamation only after the loan, child and lease drain and B moves on |
| 2 Paused writeback fence and cancel without root publication | `StoreM4Witness2WritebackFence` | Coalesced claim `{3,4}` at `<<0,0>>` in flight; `cand` cancelled and covered by `CoverCancelCheckpoint` with `AdmitToAllocator("cand")` NOT `ENABLED`, then admitted at `<<0,1>>` with zero root publications (`cellGen'[4][1] = cellGen[4][1]`, `cellGen'[4][2] > cellGen[4][2]`); `gc` completed with overlapping claim in flight and `AdmitToAllocator("gc")` NOT `ENABLED`, then admitted after drain; then a `<<0,0>>` submission is refused |
| 3 Chained crash during recovery reclaim resume | `StoreM4Witness3ChainedCrashReclaim` (`MaxCrashes = 3`) | Crash at `intent`, `RecoverResumeFreeSpaceRoot`, a second crash before `RecoverResumeReclaimComplete` (`fsroot -> fsroot_reopened`), next reopen settles `fsroot_reopened -> admitted` directly at `<<1,1>>` without a second `AdmitToAllocator` (`~ENABLED AdmitToAllocator("gc")`), reallocate and land a write, a third crash, and a reopen leaving the cells fresh at `<<1,1>>` |
| 4 Concurrent preparation and archive race | `StoreM4Witness4ConcurrentPrepArchive` | A and B prepared on one frontier; A publishes; B passes revalidation with a stale candidate and publishes onto A's frontier (`a1` kept); A prepares, is archived, and is refused |
| 5 Retained cursor across consolidation and HEAD advance | `StoreM4Witness5RetainedCursor` | Token on B at `base`; B's HEAD moves to `b1`; history consolidated; crash and reopen; `ResumeCursor` loans exactly `base`; after `base` is pruned, `ResumeCursorRefused` (HistoryUnavailable), never `b1` |

## Trace validation

`StoreM4ScaleReclaimTrace.tla` accepts a trace only if every record is a
transition the composed model allows from the previous state and reaches
exactly the recorded post-state. The trace cfg checks every safety invariant
and action property listed above, plus `TraceAccepted`.

| Trace | Expected | Covers |
| --- | --- | --- |
| `hand-chained-crash-reclaim` | exit 0 | Witness 3 path (`fsroot_reopened -> admitted`), idempotent resume, stale refusal after reuse |
| `hand-path-b-cancel-zero-root-pub` | exit 0 | Path B cancel -> `CoverCancelCheckpoint` -> `AdmitToAllocator("cand")` at `<<0,1>>` (zero root publications) -> `AllocateExtent({4})` -> stale refusal |
| `hand-writeback-fence-cancel-and-cursor` | exit 0 | Witness 2 and 5 paths |
| `hand-grandchild-survival` | exit 0 | Witness 1 path |
| `hand-concurrent-prep-archive` | exit 0 | Witness 4 path, and an indeterminate commit applied by `RecoverWal` |
| `hand-rejected-m5-reclaim-under-grandchild-loan` | exit 11 | M5: `base` reclaimed while a grandchild loan holds it |
| `hand-rejected-m6-branch-resurrection` | exit 11 | M6: retired branch live again after recovery |
| `hand-rejected-m7-cursor-rebinds-head` | exit 11 | M7: resumed cursor loans B's HEAD instead of the basis |
| `hand-rejected-m8-prune-before-coverage` | exit 11 | M8: `base` reclaimed before a canonical checkpoint covers it |
| `hand-rejected-m9-double-free-on-resume` | exit 11 | M9: recovery re-releases a reallocated `g'` range |
| `hand-rejected-m9-duplicate-admit-after-fsroot-reopen` | exit 11 | M9: duplicate `AdmitToAllocator` after `fsroot_reopened` already settled to `admitted` |
| `hand-rejected-m9-admit-before-complete` | exit 11 | M9: allocator admission before `ReclaimComplete` |
| `hand-rejected-m10-admit-under-inflight-writeback` | exit 11 | M10: admission while an overlapping claim is in flight |
| `hand-rejected-m10-stale-generation-writeback` | exit 11 | M10: stale-generation submission admitted |
| `hand-rejected-m10-cancel-without-fence-advance` | exit 11 | M10: candidate cancel that does not advance `extent_fence_seq` |

Each rejected trace is an accepted prefix plus one mutant record. The prefixes
were checked separately and pass, so each rejection comes from the mutant
record alone.

### NDJSON schema for `store_m4_dst` (M4.6b)

One JSON object per line, one line per observed barrier. A JSON list becomes a
tuple and `{"__set__": [...]}` becomes a set.

Action and parameters:

| `action` | Parameters |
| --- | --- |
| `PrepareBranch`, `BeginBranchCommit`, `RefuseStaleCandidate`, `ArchiveBranch`, `RetireBranchRef`, `RefuseRetiredBranchName`, `IssueRetainedBasisToken` | `branch`: `"A"` or `"B"` |
| `BeginCheckpoint` | `epoch` |
| `RetireHistoryPrefix` | `newFloor` |
| `BeginRecover` | `buildId` |
| `RecoverWal` | `resolvedHead` (the epoch recovery settled on) |
| `WritebackIssue`, `WritebackLand`, `WritebackCancel`, `RefuseStaleWriteback` | `range` (set of cells), `gen` (`[rootPubGen, fenceSeq]`) |
| `ReclaimIntent`, `PublishFreeSpaceRoot`, `ReclaimComplete` | `extent`: `"base"` or `"gc"` |
| `AdmitToAllocator` | `extent`: `"base"`, `"gc"` or `"cand"` |
| `AllocateExtent` | `range` |
| every other action | none |

Post-state, required on every record:

| Field | Type |
| --- | --- |
| `postDurableHead`, `postCheckpointEpoch`, `postStagedCheckpoint`, `postRetainedFloor`, `postRecoveredHead` | int |
| `postPendingWrite` | `"none"`, `"journal_unflushed"` or `"journal_flushed_pre_manifest"` |
| `postCrashed` | bool |
| `postRecoveryStage` | `"none"`, `"staging"` or `"manifest_switch"` |
| `postBranchState` | `{"A": s, "B": s}`, where s is `"live"`, `"archived"` or `"retired"` |
| `postHead` | `{"A": "base" or "a1", "B": "base" or "b1"}` |
| `postPrep` | `{"A": p, "B": p}`, where p is `"none"`, `"current"` or `"stale"` |
| `postFrontierSeq` | int |
| `postPendingBranch` | `"none"`, `"A"` or `"B"` |
| `postPinState` | `"None"`, `"Active"`, `"Expiring"` or `"Expired"` |
| `postPinnedRoots` | set of chunk names |
| `postReaderLoans`, `postChildPins` | 0 or 1 |
| `postPinsRestored` | bool (`false` after `Crash` until `RecoverRestorePins`) |
| `postPhase` | `{"base", "gc", "cand"}` mapped to `"none"`, `"intent"`, `"fsroot"`, `"fsroot_reopened"`, `"complete"`, `"cancelled"` or `"admitted"` |
| `postCov` | the same keys mapped to `"live"`, `"dropped"`, `"staged"` or `"covered"` |
| `postCellState` | list of 4 values: `"live"`, `"retired"`, `"free"` or `"fresh"` |
| `postCellGen` | list of 4 pairs `[rootPubGen, fenceSeq]` |
| `postRootPubGen`, `postFenceSeq` | int |
| `postInflight` | set of `{"range": set, "gen": pair}` |
| `postTokenBasis`, `postCursorLoan` | chunk name or `"none"` |
| `postM4Rec` | `"none"`, `"pins_restored"`, `"fsroot_resumed"` or `"complete_resumed"` |

### Binding production runs to the model

The model has a fixed finite topology: branches `A` and `B` over a shared base;
cells `1..4`, with extent `base = {1}`, `gc = {2,3}` and `cand = {4}`; claim
ranges `{3}` (split) and `{3,4}` (coalesced across `gc` and `cand`);
reallocation ranges `{3}`, `{4}`, `{2,3}` and `{3,4}`. A `store_m4_dst` scenario that emits
traces sets up exactly these roles. Its fixture maps the real
`ArenaByteRange`s to cells, emits `ExtentAllocationGeneration` as the pair, and
reports observed barrier outcomes, not oracle post-state.

`postRootPubGen` and `postFenceSeq` count the modelled events since the
scenario began. `rootPubGen` advances by one on each root publication (a
branch commit or a free-space root; a recovery batch counts as one), and
`fenceSeq` advances by one on each retirement completion or candidate cancel.
Each generation pair (`gen`, `postCellGen`) is the real
`ExtentAllocationGeneration` written in those counts.
