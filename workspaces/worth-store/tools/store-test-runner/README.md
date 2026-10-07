# WORTH Store test runner

`store-test-runner` selects ordinary Cargo tests for the WORTH Store workspace.
Run commands from `workspaces/worth-store`.

The supported products are owner tests, developer smoke tests, focused
iteration groups, UI compile tests, and CI lanes. They are convenience selections over real Cargo targets;
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

## Fresh-process recovery

The C8 process suite builds the production writer, offline observer, and
recovery executable independently, then runs the recovery scenarios with
those three executable paths:

```text
cargo run -q -p store-test-runner -- ci --partition process-scenario
```

The builds reuse Cargo's configured target directory so repeated runs keep the
normal compilation cache. There is no source inventory, executable digest,
proof report, or retained certification bundle. Git identifies the source
revision, Cargo builds the binaries, and the process tests decide whether the
behavior passes.


## Focused recovery iteration

From the repository root, choose one responsibility:

```text
cargo run --manifest-path workspaces/worth-store/Cargo.toml -p store-test-runner -- focus --group entry-custody
```

Replace `entry-custody` with a group below. `--list` prints its direct plan;
an unknown group or a selection matching no tests fails. These are selections
over `production_entry`, `phase_eight_process`, and `phase_eight_observer_cli`,
so adding focus groups adds no integration targets or link steps. A change to
either existing executable still compiles that executable once. Cargo's normal
cache supplies the process binaries on repeated runs.

| Group | Responsibility |
| --- | --- |
| `entry-admission` | Existing-root admission, generation zero, production custody |
| `entry-release` | Released generation reopen and subsequent reuse |
| `entry-custody` | Composite seal validation and serving rejoin |
| `entry-residency-policy` | Residency policy continuity through a full release ledger |
| `entry-capacity` | Release and checkpoint allocation admission |
| `entry-pool` | Recovery pool ownership, backing, and freshness transfer |
| `entry-pending` | Pending WAL fold and retained release controls |
| `entry-successor` | Per-object release successor chains |
| `entry-publication` | Publications above checkpoints and exact entry limits |
| `entry-media-denials` | Artifact damage classification and read ceilings |
| `entry-limits` | Entry, observation, manifest, and staging limit sweeps |
| `phase-oracle` | Independent persisted-media decoding and child lifecycle |
| `phase-checkpoint` | Checkpoint crash frontiers |
| `phase-agreement` | Cross-process observer agreement, fates, and terminal reports |
| `phase-recovery` | Recovery process death, cancellation, and deadline seams |
| `phase-mutation` | Mutation crash frontiers and rewrite recovery |
| `phase-successor` | Successor candidate admission, denial, and exact cost |
| `phase-observer` | Shipped offline observer CLI and report wire contract |

Focus commands limit nextest to four tests in flight. Independent crash matrix
scenarios also run at most four worlds at a time, each with its own mutable
root and child lifecycle. The full process-scenario lane builds the writer,
observer, and recoverer once, then runs its focus groups sequentially. It is
the heavy lane; use one focus command for iteration, with a five-minute budget
and a target of 150 seconds or less on a shared machine. The ordinary scenario
lane retains its complete entry and offline-observer selections.

Limit sweeps copy the exact files from one genuinely killed producer into
independent roots. Planning-only successor budget probes cancel before effects
and reuse the killed media. The residency policy group isolates its full
64-chunk, 63-partial-release fixture from unrelated tests.
