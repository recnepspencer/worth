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
