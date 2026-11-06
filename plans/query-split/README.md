# worth-query-execution split: measures, audit and stop rulings

Produced by the scripts in `scripts/test_topology/src_*.py` and `scripts/query_split/`.
Measured on master `038ff2869d`. Every number here is deterministic for that tree and the
committed commit list; wall time appears only under Telemetry.

## Outcome

| Stage | Result |
|---|---|
| 1. Move in-crate tests to courts | **Stopped after the audit.** 1,457 runtime-root tests (163,557 lines) need private items or `cfg(test)` hooks. No court can move without owner rulings. `[lib] test = false` is unreachable. |
| 2. Break the `primary_graph` cycle | **Stopped after measurement.** With re-exports attributed, the cycle is roughly 15 times what the brief estimated. A ratchet now refuses new upward edges. |
| 3. Split along domain layers | **Stopped.** 98.6% of production lines are in one component. The only valid cuts reuse a median 3,891 lines per commit. |
| 4. Cache configuration (optional) | **Not prepared.** Pinned sccache cannot share first-party Rust entries across checkout paths. Verified behaviours below. |

Each stop follows the brief's rule: "If a premise is wrong, say so with structural evidence and stop that stage."

## Structural measures on master (`measures/master/`)

- Production lines (own text, test-gated code excluded): **454,268**.
- In-crate test code: 505 families, **2,266 tests, 204,611 lines**.
- Source lines under `src/`: 659,993.
- The library has a test target in every workspace that builds it.
- Edit reach (packages recompiled when the crate changes):

  | Workspace | Packages |
  |---|---:|
  | Query | 25 |
  | bank world | 18 |
  | root | 9 |
  | `consumer_entry` fixture | 6 |

  Fan-in × size in the Query workspace is 11,356,700.
- Query workspace: 21 test targets, 13 court crates, 14 testkit crates. Four of them exceed 15,000 raw lines; they are pre-existing and governed by `court-frontier-splits.toml`.
- LLVM IR lines (`cargo llvm-lines`) were **not** recorded. It needs a second full codegen build of the crate, which this environment's disk budget did not allow.

## Stage 1: in-source test family audit (`stage1/`)

| Decision | Families | Tests | Lines |
|---|---:|---:|---:|
| public surface (move as is) | 23 | 45 | 1,260 |
| runtime root, public needs (must move) | 4 | 11 | 205 |
| runtime root, private needs or hooks (blocked) | 107 | 1,457 | 163,557 |
| private logic (stays a unit test) | 302 | 732 | 31,680 |
| owner-admitted (stays) | 12 | 21 | 2,696 |
| support, no tests | 57 | 0 | 5,213 |

**Blocked courts by cause.**

| Blocked by | Lines |
|---|---:|
| `cfg(test)` hooks only | 13,127 |
| private items only | 5,065 |
| both | 145,365 |

**What blocks them.** The ranking in `stage1/src-family-audit.md` lists the needs that block the most lines. It is led by the test-host request doors compiled only under `cfg(test)`:

| Hook | Lines blocked | Families |
|---|---:|---:|
| `.test_host_request` | 130,921 | 38 |
| `.fixture` | 127,185 | 52 |
| `.borrowed_request_for_test` | 115,862 | 31 |
| `with_test_advancement` | 111,451 | 8 |
| `TestHostRequest` | 104,978 | 7 |
| `test_product_world_resources` | 93,428 | 8 |

These are the private items that block the most lines:

| Private item | Lines blocked |
|---|---:|
| `EqualityTypeId` | 99,891 |
| `WorthQueryApplicationObservedFact` | 95,048 |
| `primary_truth_branch_identity` | 93,949 |
| `WorthQueryCheckpointOutputRole` | 84,267 |

A court crate links the library without `cfg(test)`, so none of these exist there.

**Rulings the owner must make before any court moves.** AGENTS.md: "Unit-test-only hooks use `cfg(test)`; other controls and observers compile and use owned handles."
- **Test-host request family.** Should `TestHostRequest`, `with_test_advancement` and `test_host_request` become a compiled control with an owned handle, listed in `control-callers.txt`? Or should a testkit-facing door be ruled?
- **Narrow public owner APIs, or a testkit, for the private items above.** No internals were made `pub`.

**Private-access decisions.** They are recorded per family in `stage1/src-family-audit.json` as `decision`, `private_needs`, `test_hooks`, `runtime_roots` and `support_from`.

**Proposed admissions and runtime-root registry changes.** None. Nothing moved, and the 302 private-logic families construct no runtime root, so they need no admission. No tests were renamed. `src-court-baseline.json` is unchanged, because shrinking it requires moving a family.

**The 27 movable families were not moved.** Moving them changes no gate:
- the library keeps its test target;
- the baseline sheds at most 4 entries.

The audit is also item-level and can't see method visibility; the compiler confirms that at apply. Move them in the regenerate run, after the rulings, with `court_codemod.py` extended to source families.

**`[lib] test = false` is unreachable.** The 302 private-logic families (732 tests, 31,680 lines) legitimately stay, so the second `cfg(test)` compile of the whole crate remains. It shrinks only when the crate is split, because each unit test then recompiles only its own crate.

## Stage 2: `primary_graph` coupling (`stage2/`)

These are counts of distinct (source file, defining item) references. Re-exports from the `primary_graph` root are attributed to the defining module, which addresses the brief's caveat. Test-gated families and items are excluded on both ends.

| Measure | Brief | Measured |
|---|---:|---:|
| Top-level modules (files and directories) | 56 | 79 |
| Largest strongly connected component | 32 | **56** |
| Cross-module references | 520 | 7,072 |
| Upward edges against the greedy layering | 27 | **145** |
| References on those edges | 86 | **2,035** (1,936 type, 92 function) |

**Lower bound, independent of the layering.** Breaking only the 102 two-cycles needs at least 1,346 file-level references removed. Counted as distinct items, that is **417**: 280 types and 32 functions.

**The heaviest mutual pairs.**

| Pair | References each way |
|---|---|
| `application_attempt` ↔ `workflow` | 351 / 191 |
| `application_contribution` ↔ `application_output_demand` | 174 / 177 |
| `application_query` ↔ `product_operation` | 118 / 75 |
| `application_attempt` ↔ `application_contribution` | 126 / 69 |

One example: `application_attempt/workflow_definition_program` defines the workflow publication types that `workflow` consumes, while `application_attempt` also depends on `workflow`. That is a disputed ownership boundary in the sense of Domain Structure Law 12, not a misplaced type. Each such pair needs an owner ruling: one responsibility, or a split through a contract.

**Ratchet.**
- `scripts/query_split/primary_graph_layering.json` freezes today's layering and its 145 upward edges.
- `module_coupling.py check` refuses:
  - a new upward edge;
  - an unplaced new module;
  - a stale baseline edge, which must be removed in the same change.
- It runs as plain Python in about 30 seconds.
- It is **not wired into CI.** It would refuse new upward edges on in-flight branches, so where it runs is the owner's call.

**`domain_computation` level.** 12 of 20 modules form one component with `primary_graph`, so Stage 3's acyclic premise fails too:

| Module | References into `primary_graph` | References back |
|---|---:|---:|
| `authorization` | 287 | 556 |
| `provider_session` | 52 | 537 |
| `managed_run` | 18 | 26 |

## Stage 3: candidate splits and churn replay (`stage3/`)

**Commit list.** `stage3/churn-commits.txt` holds the last 400 non-merge commits on master that touch the crate. 23 mass edits (more than 150 files) were skipped.

**Graph.** 107 module nodes: crate-root modules, `domain_computation` children and `primary_graph` children.
- The largest component covers 71 nodes and **447,833 of 454,268 production lines**.
- Below it: 27 nodes, 3,891 lines.
- Above it: 8 nodes, all facade-side.

| Candidate | Valid | Crates | Mean reusable lines | Median reusable lines | Mean reusable crates | Mean recompiled crates |
|---|---|---:|---:|---:|---:|---:|
| today | yes | 1 | 0 | 0 | 0 | 1.00 |
| vocabulary below | yes | 2 | 3,658 | 3,891 | 0.94 | 1.06 |
| vocabulary below, facade above | yes | 3 | 5,904 | 3,891 | 0.94 | 2.06 |
| 3 layers | hypothetical: 2,045 refs removed first | 3 | 87,438 | 0 | 0.67 | 2.33 |
| 5 layers | hypothetical: 2,516 refs removed first | 5 | 99,233 | 48,681 | 1.33 | 3.67 |
| 8 layers | hypothetical: 2,708 refs removed first | 7 | 106,085 | 72,886 | 2.28 | 4.72 |

**No cut was made.**
- The only valid cut moves 0.9% of the crate. It would reuse a median 3,891 lines per commit, against roughly 450,000 recompiled.
- Its cost:
  - a new crate;
  - profile entries in four manifests;
  - a `pub(crate)` → `pub` widening for every item the core consumes;
  - a full verification build.
- The value of the layered splits is real, but conditional on Stage 2. Breaking about 2,500 references would let a five-layer split reuse a median 48,681 lines per commit.

Re-measure with `churn_replay.py score` after each cycle-reduction step. Make the first cut when a valid candidate's median reusable lines is meaningful.

## Stage 4: cache configuration (verified behaviour; no commits)

- **Cargo 1.99.1 (pinned) honours a per-package override.** `incremental = false` in `[profile.dev.package.<crate>]` drops `-C incremental` for that crate only. Verified on a two-crate scratch workspace.
- **`[build] incremental = true` overrides it.** That line in `.cargo/config.toml` overrides the per-package setting, and both crates compiled incrementally again. Cargo's `dev` profile already enables incremental for workspace members, so removing it from `[build]` is the prerequisite for any per-crate cacheable layer.
- **sccache 0.18.0** (pinned candidate; the newest release at least two weeks old) has `basedirs` in its config file and `SCCACHE_BASEDIRS` in the environment.
  - Both are applied only in the C/C++ paths (`src/compiler/c.rs`, `preprocessor_cache.rs`).
  - For Rust, `generate_hash_key` hashes the raw working directory (step 9) and every `CARGO_*` variable, including `CARGO_MANIFEST_DIR`.
  - 0.19.0 is the same.
- **So first-party crates in checkouts at different absolute paths never share entries.** Registry dependencies, compiled from `~/.cargo/registry`, and rebuilds at the same path do share them.
- **Two-checkout telemetry: not run.** The mechanism it would demonstrate does not exist for Rust, and the owner asked to keep compiles rare.
- **Proposed law 14 amendment** (not edited):

  > Incremental state applies only to the volatile top crates, declared per role in the generated profiles. Stable lower crates compile non-incrementally so the cache can serve them. Cross-checkout reuse of first-party crates requires a cache wrapper whose Rust key normalizes the working directory and manifest paths. Until one is pinned, worktrees share registry dependencies and same-path rebuilds only.

## Regenerate on a newer master

```text
python scripts/query_split/structural_measures.py --out plans/query-split/measures/master
python scripts/test_topology/src_audit.py workspaces/worth-query/crates/worth-query-execution/src/lib.rs --out plans/query-split/stage1
python scripts/query_split/module_coupling.py report workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation::primary_graph --out plans/query-split/stage2/primary_graph
python scripts/query_split/module_coupling.py report workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation --out plans/query-split/stage2/domain_computation
python scripts/query_split/module_coupling.py baseline workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation::primary_graph --out scripts/query_split/primary_graph_layering.json
python scripts/query_split/churn_replay.py commits --ref master --count 400 --threshold 150 --out plans/query-split/stage3/churn-commits.txt
python scripts/query_split/split_candidates.py --out plans/query-split/stage3/candidates
python scripts/query_split/churn_replay.py score plans/query-split/stage3/churn-commits.txt plans/query-split/stage3/candidates/*.json --out plans/query-split/stage3
```

## Repository debt found (untouched, outside this scope)

**Production items used only by tests.** `worth-query-execution` warns about six items in its non-test build:
- `query_execution_denial` and `work_denial` in `retained_read.rs`;
- `ready_work_denial` in `source_readmission/ready.rs`;
- `seal_current_ready` in `permission/current_ready.rs`;
- `resolve_encoded_entity_admitted` in `entity_resolution/admitted_scope.rs`;
- `into_original` in `navigation/output_correspondence.rs`.

**Out-of-sync lockfile.** `tools/worth-query-release/Cargo.lock` is out of sync with its manifest: `cargo fetch` adds 76 lines.

**Offline cache for the court codemod.** `scripts/test_topology/test_court_codemod.py` needs `trybuild` in the offline registry cache before it can run.

## Telemetry (never a gate)

Machine: 4 cores, 15 GB RAM, Linux.

| Run | Wall time | Note |
|---|---|---|
| Cold `cargo build -p worth-query-execution --lib` (`dev`, with dependencies) | 8 min 30 s | 12 min 21 s user; leaves a 6.1 GB target directory |
| Source audit | about 27 s | |
| Coupling report | about 30 s | |
| Churn score for 6 candidates | about 44 s | |
