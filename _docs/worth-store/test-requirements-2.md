# WORTH Store Test Requirements

## Purpose

Tests exist to expose defects in Store behavior and compiler-visible
boundaries. They are not a second implementation, a status database, or a
certification protocol.

The governing rule is:

> A source rearrangement that preserves behavior must not fail certification.
> A forged authority, illegal transition, checker counterexample, real
> crash/reopen defect, or resource-envelope violation must fail directly.

## Test Classes

### Owner tests

Owner tests exercise one production responsibility through its real API. They
assert exact returned values, state transitions, denials, counters, and
resource release. Private-state mutation is allowed only when the production
owner explicitly exposes a test boundary for that responsibility.

### Integration scenarios

Integration scenarios cross real crate or runtime boundaries. They use typed
plans and authorities, production encoders/decoders, real files or admitted
backend substitutes, and independently derived expected values.

### Fresh-process scenarios

Claims involving crash, process identity, runtime identity, reopen, durable
namespace state, or cleanup after death require separate executables. A helper
thread or same-process reconstruction cannot satisfy such a claim.

### Compiler-boundary tests

Use compile-fail tests when the guarantee is visibility, ownership, linearity,
authority, feature isolation, or dependency direction. Compile-fail tests must
compile representative consumer code; they must not inspect source text.

### Formal checks

Formal checks invoke the pinned checker directly and return one of the typed
verdicts defined by the formal runner. Focused owner tests map concrete owner
outcomes to model actions where that comparison adds value. Checker output and
counterexamples are transient diagnostics.

### Release-scale tests

Large-store, multi-gigabyte blob, recovery-size-independence, and long-running
pressure claims run as explicit ignored or scheduled commands. They assert
actual memory, allocation, I/O, latency, cleanup, and persisted-state behavior.

## Evidence Rules

- Assertions consume facts produced by the execution under test.
- Oracles are independent of fixture labels and production decisions.
- Expected bytes, digests, and state are derived by a meaningfully independent
  implementation when parity is the claim.
- Exact structural counts remain exact. Performance and capacity claims use
  the weakest honest bound.
- Randomized or scheduled tests print the replay seed on ordinary failure.
- A passing command with zero selected tests is a failure. Exact CI selectors
  use `--exact`; nextest lanes use `--no-tests=fail`.
- Unsupported capabilities and exhausted formal bounds are explicit outcomes,
  never silently promoted to success.
- Tests clean up files, processes, and other resources they create.

## Required Harnesses By Sequence

| Sequence | Direct test infrastructure |
| --- | --- |
| S.0 | workload generator and owner assertions |
| S.1 | adversarial backend, storage interposer, offline verifier |
| S.2 | allocation/memory pressure and workload generator |
| S.3 | corruption injection and independent verification |
| S.4 | fault scheduler, crash/reopen, recovery determinism |
| S.4.5 | deterministic scenarios, production yieldpoints, observers, independent oracles, counters, replay |
| S.5 | deterministic physical interleavings and stable-read/reclaim scenarios |
| S.5.1 | direct scope, authenticity, custody, and wrong-scope denial scenarios |
| S.6 | backend qualification and I/O-pressure scenarios |
| S.7 | blob-scale streaming, corruption, and bounded-memory scenarios |
| S.8 | workload, corruption, access-path, and offline-verifier scenarios |
| S.9 | direct checker execution and focused owner-to-model mappings |
| S.10 | backup/PITR/repair workflows and fresh-process recovery |
| S.11 | tenant, key, custody, audit, and cross-backend security scenarios |
| S.12 | direct hostile-scale, soak, performance, backend, and hazard tests |

## Prohibited Test Machinery

Do not add or retain:

- requirement or closure ledgers;
- source inventories, per-file fingerprints, or source-text assertions;
- generated test catalogs, coverage matrices, maturity ladders, or mutation
  catalogs;
- evidence packages whose purpose is proving that tests or other evidence were
  assembled;
- reviewer receipts, adjudication protocols, or current-status fields stored
  in Markdown;
- preflight commands that list and count tests before running them;
- same-run self-comparison, fixture labels as verdicts, or logs as proof;
- controlled defects whose only consumer is another test protocol.

Git records the reviewed historical revision. Current validity is computed by
the current compiler, direct tests, formal checker, boundary checker, and
independent review.

## Required Gates

For a scoped Store change, run in proportion to risk:

- formatting and dirty Rust line-cap checks;
- focused owner tests;
- affected integration, UI, formal, and fresh-process lanes;
- workspace `cargo check --all-targets` for broad architectural changes;
- the workspace boundary checker;
- the agent-context checker;
- direct release-scale commands when the claim depends on scale.

A historical phase does not become a live software system. Later defects are
fixed in code and protected by the narrowest useful direct regression test.
