# WORTH Store test runner

`store-test-runner` selects ordinary Cargo tests for the WORTH Store workspace.
Run commands from `workspaces/worth-store`.

The supported products are owner tests, developer smoke tests, UI compile
tests, and CI lanes. They are convenience selections over real Cargo targets;
their exit status is the verdict.

```text
cargo run -q -p store-test-runner -- owner -p worth-store
cargo run -q -p store-test-runner -- smoke
cargo run -q -p store-test-runner -- ui
cargo run -q -p store-test-runner -- ci --partition scenario
cargo run -q -p store-test-runner -- ci --partition process-scenario
```

Use `--list` to inspect the planned Cargo commands without running them.
`--target-root` selects an external Cargo target directory.

The runner is a direct dispatcher. Each product names stable Cargo packages,
targets, features, and test filters in source. It does not run Cargo discovery,
list tests, count tests, record Git revisions, or write generated reports.
Cargo and the selected test executables provide the verdict through their exit
status.

The structural partition requires Bash on `PATH`. On Windows, put Git Bash
before the WSL launcher; the runner resolves that `PATH` entry explicitly so
Windows' system-directory search does not select WSL instead.

The runner's owner, scenario, UI, and formal partitions require `cargo-nextest`
on `PATH`. The full Cargo lanes also run directly:

```text
cargo test --workspace --no-fail-fast
cargo test -p worth-store-recovery-runtime --features certification-test-authority --no-fail-fast
cargo test -p worth-store-relational --features certification-test-authority --no-fail-fast
cargo clippy --workspace --all-targets
```

Cargo and nextest default to eight concurrent tests in this workspace. Process
scenarios each run complete Store instances with bounded checkpoint seams;
using the host's logical CPU count can oversubscribe those fixtures. An explicit
`RUST_TEST_THREADS` override takes precedence for Cargo runs.

Commands using the CI profile and diagnostic compiler commands disable
incremental sessions explicitly; the repository's global Cargo setting otherwise
overrides their profiles. Compiled dependencies remain cached in each lane's
target directory.

The diagnostic profile keeps every dependency unoptimized: isolated compiler
workspaces would otherwise apply the ordinary dependency optimization wildcard
to Store's own crates. Compiler denials still use the declared profile and
retain their exact semantic checks.

Nextest runs compiler-boundary fixtures one at a time because trybuild shares
Cargo's artifact lock. On Windows their aggregate limit is thirty minutes,
including the cold dependency build and every pass/fail fixture.
Each diagnostic Cargo invocation allows fifteen minutes for a cold Windows
metadata build (five minutes elsewhere). A timeout retains Cargo's progress;
it never supplies the expected semantic compiler denial.

On Windows, the five scale and exhaustive interruption proofs named in
`.config/nextest.toml` reserve the entire nextest fixture pool and have a
thirty-minute aggregate limit. Their own concurrent readers and writers still
run, and Store operation deadlines remain enforced. The long pending-WAL
producer reports completed operations to its parent; four minutes without
progress fails fixture setup.

`relational_home` requires the certification feature because its crash oracle
uses the compiled media and interruption controls. The scenario CI partition
selects it and enables `worth-store-relational/certification-test-authority`.
The six home media sweep tests cover eight target matrices, reserve the entire
eight-slot nextest pool and run eight independent cases internally. Libtest
also serializes these sweeps within its process. Each case rebuilds its own Store
because physical file identities forbid directory cloning; it then opens,
injects one cut, recovers, compares the acknowledged/in-flight prefix, finishes
the script, reopens and checks history, branch state and fresh identities.
Schedules that reject a directive are now admitted before building that home.
The final reopen declares the completed script's known schema, avoiding an
unnecessary old-schema recovery/refusal.

Before the residency integration, the three 176-case commit matrices retained
the ordinary 300-second/default and 600-second/CI guards. Checkpoint replacement
had 337 admitted cases (two seeds), so its allowance scaled by 337/176:
574 seconds locally and 1,149 in CI, rounded to 600 and 1,200 seconds. Each seed discovers its actual
media-operation count, and each sweep prints its counts and elapsed time.
The budget funds the same per-cut workload allowance, rather than assuming
that the fastest focused measurement predicts a full lane under build load.
No cuts, continuation steps or oracle checks are removed.
These historical measurements used Windows 11, Rust 1.94.0 and unoptimized `ci-test` on a
32-logical-processor, 128-GiB host during continuous boundary-check rebuilds
with `CARGO_BUILD_JOBS=8`.

| Sweep | Admitted cases | Before wall seconds (approx.) | Initial focused after seconds |
| --- | ---: | ---: | ---: |
| First commit | 176 | 316.3 | 83.92 |
| First fork commit | 176 | 325.4 | 87.97 |
| Schema transition | 176 | 347.8 | 85.33 |
| Checkpoint replacement | 337 | 452.1 | 164.81 |

Baseline ran four six-worker sweeps concurrently (24 homes); optimized
nextest runs one eight-worker sweep at a time. These timings measure the
combined fixture and capacity change, not an isolated product speedup.
The same 865 admitted cases and 189 refused schedules remain. The baseline
wall observations include process startup and have about one-second resolution.

Mean seconds per admitted case, before / after:

| Sweep | Prepare | Open | Inject | Recover | Compare | Continue/reopen |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| First commit | 0.567 / 0.291 | 0.281 / 0.120 | 0.214 / 0.110 | 0.461 / 0.153 | 0.0021 / 0.0013 | 8.510 / 3.006 |
| First fork commit | 2.235 / 1.027 | 0.365 / 0.147 | 0.220 / 0.137 | 0.481 / 0.211 | 0.0037 / 0.0030 | 6.561 / 2.283 |
| Schema transition | 3.869 / 1.432 | 0.424 / 0.170 | 0.210 / 0.082 | 1.051 / 0.380 | 0.0056 / 0.0045 | 4.533 / 1.580 |
| Checkpoint replacement | 2.434 / 1.257 | 0.326 / 0.197 | 0.277 / 0.163 | 0.363 / 0.205 | 0.0035 / 0.0031 | 3.732 / 1.943 |

Checkpoint work in that focused run was 337 * 3.77 / 8 = 159 seconds plus
setup/teardown, matching 164.81 observed. Each commit matrix was 176 * about
3.8 / 8 = 84 seconds plus setup/teardown. That measurement did not establish
a uniform full-lane cost: a later complete scenario run passed 936 tests but
timed out checkpoint replacement at 600.35 seconds. This was a budget failure,
not a symlink failure or a green scenario run.

A second diagnostic kept the scenario runner's Cargo targets, features and
profile during continuous builds. It passed all four sweeps, with the same
865 admitted cuts. These mean seconds per case expose the changing cost of
preparation and continuation; comparison remains milliseconds:

| Sweep | Prepare | Open | Inject | Recover | Compare | Continue/reopen | Sweep seconds |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| First commit | 0.118 | 0.085 | 0.050 | 0.161 | 0.0012 | 1.726 | 49.05 |
| First fork commit | 0.578 | 0.145 | 0.072 | 0.163 | 0.0025 | 1.517 | 57.55 |
| Schema transition | 5.344 | 0.558 | 0.300 | 1.371 | 0.0036 | 5.735 | 305.95 |
| Checkpoint replacement | 0.702 | 0.170 | 0.112 | 0.167 | 0.0028 | 0.987 | 94.84 |

For example, the loaded schema matrix cost 176 * 13.31 / 8 = 293 seconds
plus setup/teardown, against 305.95 observed. The checkpoint matrix cost
337 * 2.14 / 8 = 90 seconds plus setup/teardown, against 94.84 observed.
The shared host's build and I/O load varies during a lane; the checkpoint
guard therefore uses its case count relative to the commit matrices, rather
than this faster diagnostic alone. Each case still creates its own real home,
executes the nine-commit script around one fault, recovers, finishes the script,
reopens and verifies all history, branch states and fresh identities.
Phase profiling is temporary; remove it and invalidate the compiled package
before qualifying complete CI runs, then check the executed artifact for residue.

After integrating residency phase 1, all 33 home tests remain in the scenario
selection. The nine-commit, three-checkpoint script now includes all six
archive/deletion lifecycle steps as well as the feature fork. The eight
matrices admit 1,822 fault cases and reject 406 unsupported schedules before
world construction. Checkpoint member publication expands its matrix from
337 to 590 admitted cuts. The archive/deletion test runs three 176-cut
matrices under one test timer; none of their cases or assertions is removed.

The merged diagnostic passed all six tests during continuous eight-job
boundary-check rebuilds and a concurrent full Store workspace run, alongside
other builds on the shared host. It used the same Windows host and Rust
1.94.0, with `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` and
`CARGO_PROFILE_CI_TEST_DEBUG=0`. Every case builds its own home; the two seeded
memory-only reference scripts are shared immutably within each matrix.

Mean seconds per admitted case after integration (recovery includes aborting
the faulted home; continuation includes the remaining script, final reopen and
fresh-identity probes):

| Matrix | Cuts | Prepare | Open | Inject | Recover | Compare | Continue/reopen | Wall seconds |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| First commit | 176 | 0.344 | 0.183 | 0.123 | 0.273 | 0.001 | 7.376 | 189.76 |
| Fork publication | 176 | 3.265 | 0.335 | 0.252 | 0.575 | 0.005 | 13.142 | 400.88 |
| First fork commit | 176 | 3.091 | 0.289 | 0.187 | 0.414 | 0.004 | 8.670 | 289.85 |
| Archive | 176 | 5.412 | 0.485 | 0.264 | 0.623 | 0.005 | 12.316 | 435.38 |
| Deletion request | 176 | 1.945 | 0.220 | 0.100 | 0.286 | 0.007 | 3.644 | 141.91 |
| Deletion | 176 | 2.011 | 0.230 | 0.095 | 0.292 | 0.009 | 3.039 | 131.83 |
| Checkpoint replacement | 590 | 5.738 | 0.410 | 0.466 | 0.494 | 0.012 | 4.621 | 894.77 |
| Schema transition | 176 | 5.533 | 0.289 | 0.126 | 0.623 | 0.014 | 3.313 | 235.43 |

Preparation and continuation dominate; recovery remains below 0.63 seconds
per case on average, and mean comparison is at most 14 milliseconds. For example,
checkpoint work is 590 * 11.741 / 8 = 866 seconds, plus setup/teardown, against
894.77 seconds observed. These are loaded fixture measurements, not an
ordinary-operation latency contract or an isolated product speedup.

Keep the ordinary reference allowance of 300 seconds locally and 600 seconds
in CI per 176 admitted cuts. Round each aggregate to the next whole minute:

| Test | Admitted cuts | Default seconds | CI seconds | Diagnostic seconds | Final CI seconds |
| --- | ---: | ---: | ---: | ---: | ---: |
| First commit | 176 | 300 | 600 | 189.76 | 88.24 |
| First fork commit | 176 | 300 | 600 | 289.85 | 104.64 |
| Schema transition | 176 | 300 | 600 | 235.43 | 145.31 |
| Fork publication | 176 | 300 | 600 | 400.88 | 279.97 |
| Archive/deletion request/deletion | 528 | 900 | 1800 | 709.13 | 472.59 |
| Checkpoint replacement | 590 | 1020 | 2040 | 894.77 | 465.46 |

Only the two larger aggregate tests need overrides. Checkpoint replacement
gets ceil(300 * 590 / 176) and ceil(600 * 590 / 176), rounded to 1,020 and
2,040 seconds; the loaded diagnostic left 56% of its CI guard unused. The
three lifecycle matrices get 900 and 1,800 seconds, with 61% unused in the
diagnostic. The four 176-cut tests retain the ordinary limits; the slowest
measured one left 33% of the CI limit unused. The full-pool reservation stays
in force for every sweep. Temporary profiling must be removed and its package
fingerprint invalidated before final CI qualification and artifact inspection.

Final qualification used the same zero-debug profiles and eight case workers,
with the full workspace lane and continuous eight-job builds running alongside.
All 943 scenario tests passed, including all 33 home tests. Every media sweep
left at least 53% of its CI guard unused; checkpoint replacement left 77% and
the lifecycle matrices left 74%. The actual CI home executable was scanned after
execution and contained no temporary profiling token.

The weekly `Store certificate roster scale` workflow runs the explicitly
ignored `maximum_certificate_roster_checkpoints_recovers_and_rejoins` test.
It ingests 64 chunks at the production minimum of 64 KiB each (4 MiB total).
A receiver-owned retirement pause keeps all 63 real partial V3 drops pending
until the fixture checkpoints the full
63-Batch-plus-Accumulator roster, recovers through C8 and rejoins Serving,
then checks every certificate before a further checkpoint folds the Batches
into the same cumulative custody. Its separate
30-minute guard funds that complete scale world; it is excluded from ordinary
scenario runs. Under continuous builds it passed in 732.58 seconds: 195.99
for ingest, 525.67 for 63 drops (8.34 per drop), 1.75 for checkpoint,
0.76 for C8, and 6.77 for rejoin/carry. The 1,800-second guard left 59% unused.
Run the scheduled selection locally with the exact nextest
command in `.github/workflows/store-certificate-roster-scale.yml`.

Windows confinement tests create real file and directory symlinks. Run with
Windows Developer Mode enabled
or an account holding the symbolic-link creation privilege; fixture creation
must succeed before those tests can exercise confinement.

The C9 courtroom script runs its owner, independent graph, and root process
phases even when an earlier phase fails, and returns failure if any phase fails.

The separate `scripts/ci/verify_worth_store_formal_toolchain.ps1` (or `.sh`)
check requires Java on `PATH` and the checksum-pinned TLC jar described in
`worth-store-formal-models/formal-toolchain.toml`. It downloads the jar into
`target/formal-tools` when absent; offline runs must provision that jar first.

## Fresh-process recovery

The C8 process suite runs the recovery scenarios against a production writer,
offline observer, and recovery executable that the suite builds itself, each
independently of the test image:

```text
cargo run -q -p store-test-runner -- ci --partition process-scenario
```

C8 writer setup reports its phase and completed durable operations to the
parent. Setup includes all 96 operations and three checkpoints, not merely
process creation. A 60-second inactivity guard diagnoses stalled work; an
aggregate ceiling allows 120 seconds of fixed setup plus five seconds per
submitted operation. The history, exact crash seams, and recovery comparisons
are unchanged.

The checkpoint crash writer uses the same thirty-second checkpoint deadline
as its ordinary setup checkpoints. A loaded qualification found capture still
pending when one five-second pause probe expired. The writer now observes the
full operation envelope in bounded probe windows, fails early on a terminal
outcome, and reports the requested seam and stalled progress. It still pauses
at that exact seam; the parent marker guard and recovery assertions are unchanged.

Before milestone integration, under continuous eight-job boundary-check rebuilds,
the complete C8 process target passed 40 tests in 495.53 seconds; its largest
recorded writer setup was 39.38 seconds. The aggregate target includes all sixteen checkpoint crash
scenarios and the independent writer, observer and recovery builds.

After integration, the complete C8 process target passed all 40 tests in
760.98 seconds (1,118.56 seconds including the runner and builds) while
workspace and tooling verification ran alongside. The size-independence
reproducer also passed twenty consecutive loaded runs after correcting its
checkpoint-seam wait. The one
ignored focused diagnostic remains covered by the required sixteen-case matrix.

The same suite runs under plain `cargo test`; the runner only bounds it. The
suite's builds use their own target directory beside the test's, so repeated
runs keep the normal compilation cache. There is no source inventory,
executable digest report, proof report, or retained certification bundle. Git
identifies the source revision, Cargo builds the binaries, and the process
tests decide whether the behavior passes.
