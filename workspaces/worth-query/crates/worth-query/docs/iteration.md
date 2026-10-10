# Query Iteration

Choose the package that owns the edit. Declaration and installation are the
fastest loops and neither selects the main runtime package or cold
certification:

```text
cargo check -p worth-query-declaration
cargo test -p worth-query-declaration

cargo check -p worth-query-installation
cargo test -p worth-query-installation
```

For behavior owned by the main `worth-query` runtime package, run one command
that answers the current question. A check is useful while editing signatures;
a test command already performs the required build:

```text
cargo check -p worth-query --tests
cargo test -p worth-query
cargo test -p worth-query --lib domain_installation
```

Admission, execution, application-aftermath progression, publication, and host
facade behavior are separate package owners. Select the owner whose behavior
changed so its unit tests and doctests actually run:

```text
cargo test -p worth-query-admission
cargo test -p worth-query-execution
cargo test -p worth-query-publication
cargo test -p worth-query-host
```

Run the bounded cold portfolio at milestone closeout or when allocation,
hostile-world, reconstruction, or replay behavior changes:

```text
cargo test -p worth-query-execution --lib -- --test-threads=4
cargo test -p worth-query-certification -p worth-query-replay
```

Positive facade journeys belong in ordinary integration tests and doctests.
Legacy compiler matrices and test-harness self-certification were deleted;
do not recreate them. Scheduled CI gives each cold command a four-minute hard
timeout. A lane approaching that timeout is an iteration defect to fix, not a
budget to consume.

## Observed On 2026-07-18

These elapsed times are development observations, not CI budgets or golden
performance tests:

- the deleted compiler-fixture portfolio took about `399.2 s`, which exceeded
  a sane development loop and motivated its removal
- before the package extraction, the final ordinary library suite contained
  2,981 tests and took `118.14 s` of warm test time
- the Worth UI-owned Query binding suite takes `47.35 s` after a rebuild, of
  which `3.32 s` is test and doctest execution

Current owner-local and remaining-engine observations are recorded in
Milestone 9.13.1 Phase 8 closure evidence. They are review observations, not
portable thresholds. The remaining broad runtime boundary is a package-
selection problem owned by Milestone 9.13.2, not a reason to add a custom test
runner or another fixture-selection system.

The Phase 8 same-machine observations were:

- declaration warm check/test: `0.74 s` / `0.33 s`
- declaration package-invalidated check: `1.07 s`
- installation warm check/test: `0.36 s` / `0.22 s`
- installation package-invalidated check: `0.35 s`
- complete main runtime package after those invalidations: `43.55 s`
- explicitly selected cold hostile/replay lane after invalidation: `62.98 s`
