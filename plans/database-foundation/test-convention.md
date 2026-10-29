# Test layout convention v3: court crates under the iteration laws

**Authority.** `docs/coding-guidelines/iteration_laws.md` governs. This convention applies those laws to WORTH. Where it says more than the laws, it is marked **[adaptation]**. Adaptations are proposed amendments that the owner has not yet written into the laws; until then, they bind only this track. v3 supersedes v2 completely. In particular, v2's 120-second split trigger, its merge-upward rule, its class suffixes on crate names, its `-worlds` naming and its requirement-id test prefixes are all withdrawn.

## 1. Product crates (laws 2 and 4)

- **`src/` keeps only unit tests,** in `#[cfg(test)]` modules covering the module's own private logic.
- **A `src` test module may not construct a runtime root.** **[adaptation]** A runtime root is a type registered in `tools/boundary-check/config/runtime-roots.toml`, following law 2's check ("a registry of runtime roots that are forbidden in `src` test modules"). A test that constructs one is a court in disguise, and it moves to a court crate.
  - **A runtime root hosts or drives work.** Examples: the application runtime, installation runtimes, sessions, store, media and custody handles, and execution entry points submitted to an executor (`ExecutionRequest`, `SerialRequest`).
  - **Value and vocabulary types are not runtime roots, whatever crate owns them.** Examples: keys, identities, revisions, digests, enums, laws, budgets and cancellation tokens. A unit test may build these freely.
  - **Each registry entry carries a one-line reason.** Adding a type to the registry is a reviewed change, like an exemption.
- **No `#[path]` mounts into `tests/`.** A product package has no `tests/` directory.
- **A library with no unit tests sets `[lib] test = false`.**
- **A test edit never recompiles production code.**

## 2. Court crates (law 3)

- **A court crate is a leaf package.** Nothing depends on it. It sets `autotests = false` and has exactly one test target, `tests/<crate_snake_name>.rs`, with its modules under `tests/<crate_snake_name>/`. It contains no product code.
- **Name:** `<workspace>-court-<surface>`.
- **`<surface>` names the dependency set,** not a cost class and not a milestone. Examples: `decl`, `installation`, `host`, `package-archive`.
- **Grouping rule (mechanical).** Courts whose resolved normal and dev dependency sets are equal, as `cargo metadata` reports them, belong in one court crate. A court whose dependency set differs, because it needs more or fewer crates, gets its own crate named for what distinguishes it. Two examples:
  - `host-process` stays separate only if its dependencies differ from `host`'s, for example the rail executable or process-harness crates.
  - A small court with a narrower dependency set stays separate. Small is cheap when the surface is narrow. Never fold a narrow court into a wider one: that is surface collapse.
- **Adding a court** means placing a module in the court crate that matches its dependency set (law 20). A new crate is justified only by a new dependency set or by the size ceiling in §5.

## 3. Shared fixtures (law 3)

- **Fixtures live in testkit crates,** named `<workspace>-testkit`, or `<workspace>-testkit-<family>` once split (§5). Testkits replace v2's `-worlds` crates.
- **Production never depends on a testkit.** The boundary rule enforces this.
- **A testkit is compiled once.** Courts depend on it; it never depends on a court.

## 4. Categories are path segments (laws 15 and 16)

A test's category is a top-level module segment inside its court binary, never a word in its name. **[adaptation]** The category set for WORTH:

| Segment | Meaning | Selected by |
|---|---|---|
| *(none)* | Ordinary court: in-process, smallest world that proves the claim (law 17) | every profile, the inner loop |
| `process::` | Spawns processes or depends on real filesystem durability. Tests needing process isolation sit under `process::serial::`. | integration and landing profiles |
| `oracle::` | Compares against an independent oracle (in-process reference model or external), including exhaustive sweeps | landing profiles, per the oracle rule |
| `cost::` | Growth, count, timing, large scale; `#[ignore]` scale runs | scheduled only |
| `determinism::` | Determinism and replay proofs | landing profiles |
| `compile_fail::` | trybuild and compile-fail sessions | compile-boundary profile, serial |

- **Profiles and gates select by segment prefix** (for example `-E 'test(/^process::/)'`). Never by a name regex.
- **A test names its behaviour.** snake_case, a behaviour sentence, no `test_` or `it_` prefix, no milestone, phase, decision or ticket id. It is unique within its court crate.
- **Traceability lives in requirement maps,** never in names. Moved tests keep their names unless a name carries a milestone or ticket id; strip that id.

## 5. Size ceiling and splitting (laws 1 and 22) **[adaptation]**

- **Gates use deterministic measures only.** Wall time is telemetry, recorded once per landing as a trend, and never decides a split or a merge.
- **Court-crate ceiling:** a court crate or testkit crate splits when its Rust source exceeds **15,000 lines**. The boundary tool measures this on the diff.
  - The number is provisional. Calibration: the pre-split Query host court was about 61,000 lines and compiled in 977 s, which is costly per line because tests instantiate Query's generics.
  - Record each court's LLVM IR line total (`cargo llvm-lines`, law 7) once at the landing. That is the deterministic codegen measure the owner will use to replace this proxy.
- **Splits follow the next area:** owner module, requirement family or world family. Each resulting crate keeps the §2 naming, for example `worth-query-court-host-certification`.
- **No other split trigger.** A split never happens because a timing report said so.
- **No merges by test count.**

## 6. Enforcement (law 21)

The boundary-check test-topology rule enforces the following in governed workspaces (Query and the bank world in this track):

- no `tests/` directory in a product package;
- every court crate is a leaf with `autotests = false` and exactly one test target;
- court and testkit names follow the pattern;
- no production dependency on a court or a testkit;
- court-crate and testkit line ceilings;
- category segments drawn from the §4 set only;
- no filter in `query_gate.py` or the nextest config selects by name regex;
- no test name carries a milestone or ticket id.

Checks run on the diff by default and on the whole tree at the gate.

## 7. Hard rules for moving tests

- **No test is deleted, weakened or `#[ignore]`d.** Prove it with before and after multisets of test names per workspace. Module paths may change. Names may change only by stripping a milestone or ticket id; list each such rename.
- **A test that relied on being alone in its process** goes under `process::serial::`, or is fixed. Find these by reading.
- **Tests needing private access** stay unit tests, within §1's limits.
- **Record structural measures:** test-target count, court-crate count, lines per court and testkit crate, and the crate graph. One build-time sample per landing is telemetry only.

## Track sequencing and retired checker claims

The coordinator sequences source migration after the reviewed Slice 1 regroup.
The committed source baseline reports existing registered runtime-root tests;
new runtime-root constructions are refused. Slice 2 completes the macro-aware
inventory and private-access audit before moving source families.

The amended v3 section 1 retires the earlier claim that constructing any foreign
value or arbitrary opaque generator makes a court. Their checker tests now prove
that vocabulary enums and value-only generators pass. Runtime alias, helper,
struct-literal, macro-argument and factory proofs retain the work-host boundary.
