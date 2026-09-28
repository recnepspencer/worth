# WORTH Store Physical Reconstruction C.1: Direct Test Execution

## Status

C.1 defines the small execution surface used to run Store tests. It is not a
certification authority and does not maintain a model of the test repository.

The governing rule is:

> Tests prove behavior. The runner only starts tests.

## Scope

C.1 owns:

- stable owner, smoke, UI, scenario, process-scenario, formal, and structural
  commands;
- direct Cargo or nextest argument construction;
- bounded sharding for expensive integration targets;
- propagation of the selected Cargo target directory and CI profiles; and
- process exit status.

C.1 does not own:

- source identity, which belongs to Git;
- package or target truth, which belongs to Cargo;
- behavioral verdicts, which belong to test executables;
- required-job completion, which belongs to GitHub Actions; or
- milestone or phase validity, which is a code-review judgment over current
  source and current test results.

## Product Contract

The supported entry points are:

```text
cargo store-owner -p <package>
cargo store-smoke
cargo store-ui
cargo store-ci --partition <lane> [--shard-index N --shard-count M]
```

`--list` prints the exact commands without running them. `--target-root`
selects an explicit Cargo target directory. Successful execution writes no
custom report, ledger, certificate, manifest, digest, or status artifact.

Each nextest command uses `--no-tests=fail`. A stale target or selector is
therefore a failed command, not a green empty lane.

## Selection Model

The runner uses direct, reviewable dispatch tables for the integration targets
that need dedicated scenario, UI, or formal lanes. This is intentional.

- Owner commands select a package's ordinary Cargo targets directly.
- Smoke commands select a deliberately small set of named behavioral tests.
- Scenario, UI, and formal commands select named integration targets directly.
- C.8 fresh-process recovery uses its direct process dispatcher because it
  manages child roles and failure seams rather than a normal libtest process.
- Structural checks invoke the repository-owned boundary, context, and line-cap
  commands directly.

The dispatch tables are not a complete inventory of every Cargo test target
and do not claim exactly-once classification. New ordinary tests are covered by
owner CI. A new dedicated integration target is added to the appropriate table
in the same change that introduces it. Code review and a real runner invocation
verify that change; the repository does not build a second Cargo metadata
catalog to certify it.

## Execution Rules

1. Every planned execution unit has one stable identity.
2. Duplicate unit identities are rejected before execution.
3. A requested shard must have a valid zero-based index and nonzero count.
4. Scenario sharding lowers to nextest's stable hash partition.
5. CI products apply the repository's `ci` nextest profile and `ci-test` Cargo
   profile.
6. A command stops with failure when its selector matches no tests.
7. The runner forwards Cargo and test-process diagnostics without translating a
   green exit into a stronger claim.
8. The runner does not parse Rust source, count assertions, inspect test names
   for semantic coverage, or launch Cargo metadata/test-list preflights.
9. The runner does not retain run history or compare the current run with a
   prior revision.

## Verification

The runner is protected by proportionate tests:

- argument parsing and invalid-shard rejection;
- exact command planning for representative owner, smoke, scenario, UI,
  formal, process-scenario, and structural products;
- duplicate execution-unit rejection;
- `--no-tests=fail` on every nextest product;
- target-root and CI-profile propagation; and
- CLI smoke tests for `--list` and invalid input.

Those tests inspect the command the runner owns. They do not attempt to prove
that other tests are adequate, enumerate all repository tests, or certify the
source tree.

## CI Topology

GitHub Actions invokes the stable products directly. Expensive release-scale,
destructive-recovery, soak, fuzz, mutation, and hardware qualification cases
remain named scheduled commands unless orchestration provides a concrete
runtime benefit.

CI logs are ordinary process output. They may be retained by the CI provider,
but no checked-in or generated WORTH Store evidence bundle is required to
interpret a successful job.

## Prohibited Machinery

C.1 must not contain or recreate:

- progressive proof or closure ledgers;
- source hashes, source manifests, file inventories, or Git-tree mirrors;
- Cargo-derived completeness catalogs or exactly-once lane matrices;
- assertion parsers, source-text architecture checks, or test-count baselines;
- mutation catalogs or mutation reports used as ordinary CI prerequisites;
- run seals, plan seals, readiness witnesses, handoff tokens, or phase status;
- generated JSON/CSV/Markdown reports consumed by another test stage; or
- tests whose subject is the completeness of another test registry.

If a target selection becomes stale, the correct fix is to update the direct
dispatch and run it. If a test is inadequate, the correct fix is to improve or
replace that behavioral test. Neither problem justifies another certification
system around the suite.

## Handoff

Later phases consume the repository, stable commands, and current CI
results directly. They do not consume a C.1 certificate, catalog, report,
ledger, or readiness token.
