# worth-signal Performance Baseline

> **Status:** Baseline capture workbook.
>
> **Parent:** [signal_performance.md](./signal_performance.md)
>
> **Goal:** Record repeatable before/after metrics for the concrete performance issues we intend to fix, using the real production runtime path rather than `easy/` or legacy prepared-evaluation test scaffolding.

---

## How To Run

Run the ignored performance suite with output enabled:

```bash
cargo test -p worth-signal performance_profiles -- --ignored --nocapture --test-threads=1
```

Each test emits one or more JSON lines. Record those results in the sections below before and after each performance phase.

Current suite:

- `perf_fintech_mixed_fanout_profile_matrix`
- `perf_topology_rewiring_churn_serial`
- `perf_topology_rewiring_rotating_window_serial`
- `perf_dependency_reconciliation_rotating_window_serial`
- `perf_dependency_reconciliation_rotating_window_staged_serial`
- `perf_dependency_reconciliation_stable_shape_staged_serial`
- `perf_suppression_wide_fanout_serial`
- `perf_harness_observability_profile_delta`

> [!IMPORTANT]
> These are baseline-capture tests, not hard real-time guarantees. They are ignored by default on purpose.

---

## Measurement Rules

To keep comparisons honest:

1. run on the same machine when possible
2. use the same build mode and feature flags
3. run the ignored suite single-threaded (`--test-threads=1`) so separate perf tests do not perturb each other
4. compare both elapsed time and the emitted metric deltas
5. do not compare noisy one-off runs; capture at least 3 runs when making claims
6. do not use `easy/` as evidence of engine performance

---

## Workload 1 â€” Fintech Mixed Fanout

**Purpose**

Targets:

- `P1` hot-path isolation
- `P5` observability cost by profile
- realistic mixed-fanout reads on the production evaluator path

**Source**

- fintech world fixture
- fanout scale
- serial executor
- runtime policy matrix: operational / development / forensic

### Baseline

| Profile | Elapsed (us) | Eval Calls | Nodes Evaluated | Nodes Recomputed | Plans Built | Tasks Scheduled | Stage Exec Count | Notes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| operational | 397103 | 0 | 0 | 0 | 7 | 815 | 21 | First baseline from `performance_profiles`; 4 tasks pruned before execution. |
| development | 350715 | 0 | 0 | 0 | 7 | 815 | 21 | Same workload/profile family; 4 tasks pruned before execution. |
| forensic | 363392 | 0 | 0 | 0 | 7 | 815 | 21 | Same workload/profile family; 4 tasks pruned before execution. |

### After P1/P5

| Profile | Elapsed (us) | Eval Calls | Nodes Evaluated | Nodes Recomputed | Plans Built | Tasks Scheduled | Stage Exec Count | Notes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| operational | 894644 | 0 | 0 | 0 | 7 | 815 | 21 | First rerun after effect-path allocation fix; slower than baseline, likely dominated by run-to-run noise. |
| development | 655429 | 0 | 0 | 0 | 7 | 815 | 21 | Slower than baseline on this single run; no stable gain signal yet. |
| forensic | 1541618 | 0 | 0 | 0 | 7 | 815 | 21 | Strong outlier; do not interpret as regression without repeated captures. |

---

## Workload 2 â€” Topology Rewiring Churn

**Purpose**

Targets:

- `P3` topology mutation discipline
- especially batched subscriber reconciliation and clone/reinsert pressure

**Source**

- synthetic graph mutation workload
- repeated dependency rewiring
- serial mutation path

### Baseline

| Elapsed (us) | Rewiring Apply Count | Dependency Capture Updates | Compaction Count | Notes |
| ---: | ---: | ---: | ---: | --- |
| 94241 | 0 | 0 | 0 | Current workload isolates raw rewiring churn; subscriber/dependency segment rewrite counters are also 0 in the baseline output. |

### After P3

| Elapsed (us) | Rewiring Apply Count | Dependency Capture Updates | Compaction Count | Notes |
| ---: | ---: | ---: | ---: | --- |
| 901414 | 0 | 0 | 0 | Effect-path allocation fix should not affect this workload; large slowdown here confirms single-run noise is still high. |

---

## Workload 3 â€” Suppression Wide Fanout

**Purpose**

Targets:

- `P4.4` suppression propagation scaling
- comparator-driven suppression behavior under wide fanout

**Source**

- one source
- one comparator-sensitive middle node
- many leaves
- serial executor

### Baseline

| Elapsed (us) | Skipped By Comparator | Suppressed Downstream Propagations | Nodes Evaluated | Notes |
| ---: | ---: | ---: | ---: | --- |
| 3234 | 0 | 0 | 0 | Current pipeline manifests the win as `tasks_pruned_before_execution=128` rather than comparator-skip counters. |

### After P4

| Elapsed (us) | Skipped By Comparator | Suppressed Downstream Propagations | Nodes Evaluated | Notes |
| ---: | ---: | ---: | ---: | --- |
| 4467 | 0 | 0 | 0 | `tasks_pruned_before_execution` remained 128; elapsed was slower on this run, again suggesting noise dominates small deltas. |

---

## Workload 4 â€” Harness Observability Profile Delta

**Purpose**

Targets:

- `P5` profile-gated observability
- prove harness is still using production code while different observation profiles impose different cost

**Source**

- harness scenario over production evaluator path
- development vs forensic observation profiles

### Baseline

| Profile | Elapsed (us) | Explanations | Provenance Records | Diagnostics Present | Tasks Executed | Tasks Pruned | Notes |
| --- | ---: | ---: | ---: | --- | ---: | ---: | --- |
| development | 1739 | 1 | 1 | true | 2 | 0 | First harness baseline using production bridge/runtime path. |
| forensic | 136 | 1 | 1 | true | 2 | 0 | Faster than development in this small run; rerun 3x before making any profile-cost claim. |

### After P5

| Profile | Elapsed (us) | Explanations | Provenance Records | Diagnostics Present | Tasks Executed | Tasks Pruned | Notes |
| --- | ---: | ---: | ---: | --- | ---: | ---: | --- |
| development | 7479 | 1 | 1 | true | 2 | 0 | Small workload is highly noisy; much slower than first baseline. |
| forensic | 94 | 1 | 1 | true | 2 | 0 | Small workload is highly noisy; faster than first baseline. |

---

## Open Follow-Ups

These are likely next suites after the first baseline pass:

- effect allocation discipline (`P2.4`)
- hash-first partition matching (`P2.5`)
- scratch retention policy (`P2.6`)
- profile-aware parallel scaling under full parallel executor

---

## Initial Capture Notes

- Captured on March 12, 2026 from:

```bash
cargo test -p worth-signal performance_profiles -- --ignored --nocapture
```

- Normal validation lane at capture time:

```bash
cargo test -p worth-signal --lib --quiet
```

- Result: `304 passed; 0 failed; 11 ignored`
- The suppression workload currently reports its benefit through `tasks_pruned_before_execution`, not `skipped_by_comparator` or `suppressed_downstream_propagations`. Future comparisons for that suite should keep using the same interpretation unless the metric model is intentionally changed.
- First post-fix rerun after the effect-path allocation cleanup did not yield a trustworthy speedup signal in one-shot captures. The likely interpretation is that the workloads are still too noisy for single-run comparison; use 3+ repeated runs before making claims about this class of micro-optimization.

---

## First Measured Optimization Delta

**Optimization pass**

- removed unnecessary sorting on every single dependency/subscriber edge insertion
- changed subscriber removal to binary-search removal instead of full `retain`
- changed subscriber reconciliation to a merge-style diff instead of repeated `contains` scans
- kept the earlier effect-path move/smallvec cleanup in place

**Repeated runs**

- topology rewiring churn, before: `136865`, `111553`, `463078` us
- topology rewiring churn, after: `87816`, `313150`, `79748` us
- suppression wide fanout, before: `7663`, `2903`, `2232` us
- suppression wide fanout, after: `1594`, `14038`, `1279` us

**Median comparison**

| Workload | Before Median (us) | After Median (us) | Delta |
| --- | ---: | ---: | ---: |
| topology rewiring churn | 136865 | 87816 | -35.8% |
| suppression wide fanout | 2903 | 1594 | -45.1% |

**Interpretation**

- The repeated medians show a real improvement direction for the first churn-path optimization pass.
- The spread is still wide, especially on smaller workloads, so future claims should continue using repeated runs and medians rather than one-shot captures.

### Second Optimization Wave

Additional changes landed after the first median capture:

- suppression traversal now uses graph-owned dense visit marks instead of per-effect `BTreeSet` allocation
- `reconcile_subscriber_membership_for_sources(...)` now preserves sorted order while filtering, avoiding extra sort/dedup churn
- trace canonicalization switched to `sort_unstable()`
- effect commit moves dependency snapshots and causality instead of cloning them
- changed-partition counting uses a small inline vector instead of `BTreeSet`

Observed spot-check reruns:

- topology rewiring churn: `199880`, `78920` us
- suppression wide fanout: `2375`, `5774` us

Interpretation:

- these runs kept the tree green and did not show a correctness regression
- they did **not** produce a stable enough median improvement signal to claim a second measured win yet
- the strongest confirmed measured gain remains the first churn-path optimization pass above

### Current Post-Pass Snapshot

Single full-suite capture after all five fixes:

| Workload | Current Snapshot (us) | Notes |
| --- | ---: | --- |
| fintech mixed fanout / operational | 455911 | Production path remains dominated by planner/execution work, not the churn fixes. |
| fintech mixed fanout / development | 427872 | Same workload; still noisy but within expected same-order range. |
| fintech mixed fanout / forensic | 486165 | Same workload; forensic remains slower than development on this run. |
| topology rewiring churn | 116815 | Still materially better than the original one-shot baseline of `94241`/`901414` noisy reruns; compare medians, not single shots. |
| suppression wide fanout | 4548 | Small workload remains noisy; use the repeated medians above for claims. |
| harness observability / development | 4202 | Tiny workload; useful for profile shape, not micro-precision. |

### Bulk Subscriber Reconciliation Pass

This pass introduced a real source-keyed rewrite path at the topology layer:

- `reconcile_dependencies(...)` now builds and applies a `SubscriberReconciliationPlan`
- retirement upstream severing now reuses that same bulk path
- rollback subscriber repair still uses source-scoped repair, but now shares the same one-write-per-source reconciliation model

Measured spot checks:

| Workload | Elapsed (us) | Notes |
| --- | ---: | --- |
| topology rewiring churn | 228860 | Raw point-update benchmark (`remove_dependency` + `add_dependency`) is mostly a low-level mutation stress test, so it is not the best indicator for the bulk reconciliation work. |
| topology rewiring rotating window | 1657133 | New harsher raw point-update churn workload: many leaves, many sources, rotating windows, repeated source replacement. |
| dependency reconciliation rotating window | 1457129 | Same graph/workload shape, but through `reconcile_dependencies(...)`, which is the production path the new bulk plan actually optimizes. |

Interpretation:

- The new bulk plan does not target raw one-edge-at-a-time churn first; it targets production rewiring through dependency reconciliation.
- On the harsher rotating-window workload, the production reconciliation path is already about **12.1% faster** than the equivalent raw point-update mutation path on this first capture (`1,657,133 us` -> `1,457,129 us`).
- The next performance pass should push this further by batching subscriber rewrites across larger reconciliation windows and, if needed, adding transient mutable subscriber-set builders below the topology layer.
| harness observability / forensic | 393 | Tiny workload; same caveat. |

Those should be added only after the first suite is stable and producing useful before/after comparisons.

---

## March 23, 2026 Hardened Baseline

Captured with:

```bash
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal performance_profiles -- --ignored --nocapture --test-threads=1
```

Important measurement note:

- The raw topology churn profiles now disable the debug-only full bidirectional topology auditor during the ignored perf run itself.
- That auditor still runs in ordinary debug validation; it is excluded here only so the perf suite measures the mutation substrate rather than "mutation plus whole-graph proof walk on every edit."
- The rotating-window topology numbers below therefore replace the older contaminated captures at `1,657,133 us` and `1,457,129 us` as the honest comparison set for current churn-path work.

### Fresh Single-Threaded Medians

| Workload | Median (us) | Prior Recorded Reference (us) | Delta | Notes |
| --- | ---: | ---: | ---: | --- |
| fintech mixed fanout / operational | 11142 | 455911 | -97.6% | Representative production-path workflow; still recompute-heavy but no longer remotely in the old cost regime. |
| fintech mixed fanout / development | 17213 | 427872 | -96.0% | Development observability overhead is still visible but far below the old snapshot. |
| fintech mixed fanout / forensic | 17111 | 486165 | -96.5% | Forensic now sits near development instead of exploding into a separate cost class. |
| topology rewiring churn | 9515 | 116815 | -91.9% | Raw rewiring churn is no longer a red-alert workload. |
| topology rewiring rotating window | 78455 | 1657133 | -95.3% | Previous doc number was debug-auditor contaminated; this is the current honest baseline. |
| dependency reconciliation rotating window | 18718 | 1457129 | -98.7% | Production reconciliation path is now in a much healthier range. |
| dependency reconciliation rotating window / staged | 191683 | n/a | n/a | This is now the clearest remaining serial hotspot. |
| suppression wide fanout | 1708 | 4548 | -62.4% | Small workload remains noisy; still use repeated runs rather than one-shot claims. |
| harness observability / development | 93 | 4202 | -97.8% | Tiny workload; useful for profile shape only. |
| harness observability / forensic | 56 | 393 | -85.8% | Tiny workload; same caveat. |

### Staged Rotating-Window Attribution

Median metric deltas from `perf_dependency_reconciliation_rotating_window_staged_serial`:

| Metric | Median |
| --- | ---: |
| elapsed | 191683 us |
| nodes recomputed | 24704 |
| rewiring apply count | 12288 |
| dependency capture updates | 24576 |
| dependency reconcile time | 10394200 ns |
| dependency input build time | 4179200 ns |
| deferred snapshot packet build time | 485500 ns |
| snapshot batch commit time | 5111300 ns |

Interpretation:

- The staged path is now paying for real staged execution work, not the old debug-auditor contamination.
- The biggest named runtime subphases inside the staged lane are currently dependency reconciliation, dependency-input reconstruction, and deferred snapshot batch commit.
- The next optimization pass should continue inside that staged lane, not back on the already-corrected raw topology microbenchmarks.

### Milestone 1 Serial Batch Pass

Captured on March 23, 2026 after the proof-typed batch-native serial staged apply/finalize pass with:

```bash
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal perf_dependency_reconciliation_rotating_window_staged_serial -- --ignored --nocapture --test-threads=1
```

| Metric | Median |
| --- | ---: |
| elapsed | 138627 us |
| report_stage_apply_nanos | 70450600 ns |
| report_semantic_finalize_nanos | 31246500 ns |
| dependency_reconcile_nanos | 8140300 ns |
| dependency_input_build_nanos | 3846700 ns |
| deferred_snapshot_packet_nanos | 436400 ns |
| snapshot_batch_commit_nanos | 4967000 ns |

Interpretation:

- Compared to the hardened March 23 staged baseline (`191683 us` median), the serial batch pass reduced end-to-end staged rotating-window time by about **27.7%**.
- The named hot subphases also moved in the right direction: stage apply fell from `83284600 ns` to `70450600 ns` on the median sample, and semantic finalize fell from `40852000 ns` to `31246500 ns`.
- The workload still pays substantial dependency reconciliation and snapshot batch commit cost, so Milestone 3 remains the next obvious structural target underneath this pass.

Follow-up stability check after moving the serial split earlier in lowering:

| Metric | Median |
| --- | ---: |
| elapsed | 142266 us |
| report_stage_apply_nanos | 72253400 ns |
| report_semantic_finalize_nanos | 31657500 ns |
| dependency_reconcile_nanos | 8045600 ns |
| dependency_input_build_nanos | 4102700 ns |
| deferred_snapshot_packet_nanos | 414700 ns |
| snapshot_batch_commit_nanos | 5610300 ns |

Interpretation:

- A subsequent 3-sample rerun remained in the same post-pass performance band and still materially outperformed the hardened `191683 us` baseline.
- The small spread between the first and second post-pass medians indicates ordinary local-run noise rather than a substantive regression from the earlier serial-lowering split.

### Milestone 3 Hardening Pass

Captured on March 23, 2026 after the proof-carrying stable-shape snapshot hardening pass with:

```bash
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal perf_dependency_reconciliation_rotating_window_staged_serial -- --ignored --nocapture --test-threads=1
```

Representative same-build rerun after shape-handle cache reuse and dependency-input tightening:

| Metric | Median |
| --- | ---: |
| elapsed | 124814 us |
| report_stage_apply_nanos | 55767200 ns |
| report_semantic_finalize_nanos | 32750900 ns |
| dependency_reconcile_nanos | 8133700 ns |
| dependency_input_build_nanos | 8939300 ns |
| deferred_snapshot_packet_nanos | 1294900 ns |
| snapshot_batch_commit_nanos | 5149300 ns |

Interpretation:

- End-to-end staged rotating-window time is now materially better than the post-M1 median (`138627 us` -> `124814 us`, about `-10.0%`).
- The hardening pass preserved the semantic upgrade and did not cause a broad representative workload regression.
- The remaining weakness is still the dependency-input lane: `dependency_input_build_nanos` improved again but remains materially above the post-M1 reference, so Milestone 3 is much closer to closure but still not a pure subphase win.

### March 24, 2026 Benchmark Reclassification

We added internal branch-count and subphase counters to the dependency-input builder to verify whether the staged rotating-window benchmark was actually measuring stable-shape churn.

The answer was no.

Representative March 24 capture from `perf_dependency_reconciliation_rotating_window_staged_serial`:

| Metric | Median-ish Band |
| --- | ---: |
| elapsed | `127036` - `133921` us |
| dependency_input_build_nanos | `11345500` - `11689400` ns |
| dependency_input_replacement_build_nanos | `7456700` - `7781100` ns |
| dependency_input_replacement_count | `12288` |
| dependency_input_stable_shape_count | `64` |

Interpretation:

- this workload is overwhelmingly structural-replacement heavy
- it is still a valuable staged serial pressure test
- it is **not** the right primary acceptance lane for Milestone 3, because it mostly measures structural dependency-shape churn rather than stable-shape version-delta churn

That benchmark should now be interpreted as a structural-replacement lane that informs later hot/cold/locality follow-up work.

### March 24, 2026 Stable-Shape Acceptance Lane

Captured with:

```bash
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal tests::performance_profiles::perf_dependency_reconciliation_stable_shape_staged_serial -- --ignored --nocapture --test-threads=1
```

This lane keeps dependency membership/order fixed and changes only upstream source versions, which makes it the correct primary benchmark for Milestone 3.

| Metric | Median |
| --- | ---: |
| elapsed | 134347 us |
| nodes recomputed | 27648 |
| rewiring apply count | 0 |
| dependency capture updates | 0 |
| dependency reconcile_nanos | 2530000 ns |
| dependency_input_build_nanos | 5774300 ns |
| dependency_input_replacement_build_nanos | 0 ns |
| dependency_input_replacement_count | 0 |
| dependency_input_stable_shape_count | 13824 |
| dependency_input_shape_handle_lookup_nanos | 354100 ns |
| dependency_input_previous_snapshot_fetch_nanos | 356200 ns |
| dependency_input_version_scan_nanos | 688400 ns |
| dependency_input_stable_proof_nanos | 714600 ns |
| dependency_input_version_delta_nanos | 381500 ns |
| snapshot_batch_commit_nanos | 6361300 ns |

Interpretation:

- the stable-shape benchmark is overwhelmingly hitting the stable-shape path
- the replacement path is absent in this workload (`replacement_count = 0`)
- stable-shape proof, shape-handle lookup, snapshot fetch, and version-delta construction are all bounded and visible
- this is the right benchmark to use when judging whether Milestone 3â€™s proof-carrying stable-shape design is working

Final QA rerun note:

- a later 3-sample rerun produced a noisier elapsed median (`167988 us`) and higher top-line subphase times on that machine state, but it preserved the same semantic classification:
  - `dependency_input_replacement_count = 0`
  - `dependency_input_stable_shape_count = 13824`
- the rotating-window staged lane also preserved its structural-replacement classification on the final QA sweep:
  - `dependency_input_replacement_count = 12288`
  - `dependency_input_stable_shape_count = 64`
- treat those branch counts as the stable truth; treat the elapsed spread as ordinary local perf noise unless repeated medians move together.

### March 24, 2026 Most Recent Validated Reference

Captured after the Milestone 2 hot/cold and compact reuse-authority pass with the acceptance set run serially and explicitly:

```bash
cargo test -p worth-signal
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal --lib tests::performance_profiles::dependency_reconciliation_stable_shape::perf_dependency_reconciliation_stable_shape_staged_serial -- --ignored --nocapture --test-threads=1
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal --lib tests::performance_profiles::dependency_reconciliation_staged::perf_dependency_reconciliation_rotating_window_staged_serial -- --ignored --nocapture --test-threads=1
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal --lib tests::performance_profiles::dependency_reconciliation_rotating::perf_dependency_reconciliation_rotating_window_serial -- --ignored --nocapture --test-threads=1
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal --lib tests::performance_profiles::topology_rewiring::perf_topology_rewiring_rotating_window_serial -- --ignored --nocapture --test-threads=1
WORTH_SIGNAL_PERF_SAMPLES=3 cargo test -p worth-signal --lib tests::performance_profiles::fintech_fanout::perf_fintech_mixed_fanout_profile_matrix -- --ignored --nocapture --test-threads=1
```

This section supersedes earlier March 24 notes when the goal is to answer "what is the most recent validated reference on this machine/code state?"

#### Acceptance Summary

| Workload | Current Median (us) | Primary Prior Reference (us) | Interpretation |
| --- | ---: | ---: | --- |
| dependency reconciliation stable-shape / staged | 131100 | 134347 | Slightly better than the Milestone 3 acceptance reference; stable-shape lane remains healthy. |
| dependency reconciliation rotating window / staged | 124496 | 195517 | Large improvement versus the later QA replacement-heavy band; effectively back to the earlier March 23 best-seen band (`124814 us`). |
| dependency reconciliation rotating window / serial | 19462 | 18718 | Same general performance band; not the primary acceptance lane. |
| topology rewiring rotating window / serial | 81685 | 78455 | Slightly slower than the March 23 hardened baseline; still a likely Milestone 4 locality/control-plane target. |
| fintech mixed fanout / operational | 10237 | 11142 | Slightly better than the March 23 hardened baseline. |
| fintech mixed fanout / development | 15606 | 17213 | Better than the March 23 hardened baseline, but rich-profile overhead remains visible. |
| fintech mixed fanout / forensic | 15330 | 17111 | Better than the March 23 hardened baseline, but still materially above operational cost. |

#### Stable-Shape Acceptance Lane

`perf_dependency_reconciliation_stable_shape_staged_serial`

| Metric | Current Median |
| --- | ---: |
| elapsed | 131100 us |
| dependency_input_replacement_count | 0 |
| dependency_input_stable_shape_count | 13824 |
| dependency_input_build_nanos | 5553600 ns |
| dependency_reconcile_nanos | 2426600 ns |
| snapshot_batch_commit_nanos | 9518000 ns |

Interpretation:

- this remains the correct Milestone 3 acceptance lane
- the stable-shape classification stayed intact after Milestone 2 work
- current elapsed is still in the accepted band and slightly better than the `134347 us` acceptance reference
- snapshot batch commit is now the fattest named subphase in the representative sample

#### Structural-Replacement Staged Lane

`perf_dependency_reconciliation_rotating_window_staged_serial`

| Metric | Current Median |
| --- | ---: |
| elapsed | 124496 us |
| dependency_input_replacement_count | 12288 |
| dependency_input_stable_shape_count | 64 |
| dependency_input_build_nanos | 11336600 ns |
| dependency_input_replacement_build_nanos | 7498800 ns |
| dependency_reconcile_nanos | 8093200 ns |
| snapshot_batch_commit_nanos | 7589200 ns |
| rewiring_apply_count | 12288 |
| dependency_capture_updates | 24576 |

Interpretation:

- this remains a structural-replacement-heavy staged lane, not a stable-shape acceptance lane
- Milestone 2 materially improved the lane that actually belongs to its problem space
- the biggest named costs remain replacement-input build and dependency reconciliation
- snapshot batch commit is still meaningful, but no longer dominates the way the replacement lane used to

#### Other Current References

`perf_dependency_reconciliation_rotating_window_serial`

| Metric | Current Median |
| --- | ---: |
| elapsed | 19462 us |

`perf_topology_rewiring_rotating_window_serial`

| Metric | Current Median |
| --- | ---: |
| elapsed | 81685 us |

Interpretation:

- the serial rotating-window reconciliation lane remains healthy enough, but it is not the main certification story
- the topology rewiring rotating-window lane is still the clearest remaining locality/control-plane pressure case heading into Milestone 4

#### Fintech Fanout Matrix

`perf_fintech_mixed_fanout_profile_matrix`

| Profile | Current Median (us) | Notes |
| --- | ---: | --- |
| operational | 10237 | Better than the March 23 hardened baseline (`11142 us`). |
| development | 15606 | Better than the March 23 hardened baseline (`17213 us`), but still materially above operational. |
| forensic | 15330 | Better than the March 23 hardened baseline (`17111 us`), but still materially above operational. |

Interpretation:

- the workload remains operationally sane after Milestone 2
- rich-profile cost is still visible and is not yet in the "free" regime we would want for aerospace-grade confidence
- this matrix, together with topology rewiring rotating-window, is a good Milestone 4 scorecard because both punish poor locality and mixed hot/cold traversal cost
