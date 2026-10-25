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

The same suite runs under plain `cargo test`; the runner only bounds it. The
suite's builds use their own target directory beside the test's, so repeated
runs keep the normal compilation cache. There is no source inventory,
executable digest report, proof report, or retained certification bundle. Git
identifies the source revision, Cargo builds the binaries, and the process
tests decide whether the behavior passes.
