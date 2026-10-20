---
name: qa-tests
description: Review and correct WORTH tests and test harnesses for honest, proportionate evidence. Use when auditing fixture realism, boundary claims, oracle independence, adversarial value, redundancy, compile cost, or whether tests adequately protect a specification.
---

# QA Tests

Review whether the tests are credible evidence for the behavior they claim to
protect. Test adequacy is a code-review judgment; tests do not need a second
evidence system that certifies their adequacy.

Read the governing specification, relevant production code,
`docs/coding-guidelines/testing_laws.md`, and
`docs/coding-guidelines/qa_review_guide.md`. Trace setup, action, observation,
and teardown rather than trusting test names.

## Test behavior, not speculative resource budgets

Do not require work-counter, candidate-budget, retained-byte, mutation-quote,
reservation-ledger, or one-byte-short exhaustion matrices for ordinary modeling
and publication. Such tests need an explicit resource-admission product
requirement or a measured resource failure that actually needs accounting.
Hypothetical exhaustion and a desire for more complete accounting do not qualify.
Do not expand these suites or make them a milestone gate by default.

When unnecessary accounting is removed, remove or rewrite its causally affected
tests rather than preserving the mechanism to satisfy old assertions. Retain
meaningful evidence for geometry, currentness/invalidation, authority, atomic
publication, cancellation, and ordinary failure handling. Include a focused
positive case showing the real authoring operation succeeds without a caller
supplying resource estimates or an arbitrary work ceiling; do not replace this
with another bookkeeping certification system.

Measure iteration cost and investigate actual bottlenecks. Favor focused tests
of invalidation, reuse, partition independence, and concurrency when affected.
Semantic cardinalities, revisions, termination conditions, checked arithmetic,
and simple diagnostics are not speculative admission accounting. Preserve them
when they serve real correctness or diagnosis. Reuse accepted evidence and
avoid broad exhaustion campaigns or repeated full builds after each edit.

## Choose the execution lane

- **Focused:** the reproducer, affected owner tests, and the smallest honest
  boundary smoke during development.
- **CI:** the accepted requirement tests and mandatory repository gates for the
  final change.
- **Scheduled:** expensive mutation, fuzz, soak, compatibility, maximum-scale,
  destructive-recovery, and environment-specific suites.

Run an expensive lane during ordinary development only when it is the actual
reproducer or materially affected evidence. Run cheap discovery, configuration,
fixture, exact-name, and harness checks before expensive worlds.

## Independent review

Use the test reviewer selected by the user or repository. Do not hard-code a
review model or require a fresh instance after every correction. A persistent
reviewer may follow the work but must inspect the final production and test diff
and current results before clearing it.

Give the reviewer the specification, relevant laws, current diff, fixtures,
harnesses, assertions, production boundary, commands, results, timings, and
known environment constraints. Verify every finding directly.

Minimize ceremony and maximize pragmatic progress without relaxing substantive
review standards. Strictness belongs in correctness and evidence, not
coordination artifacts. Review the living branch and current diff; after a
finding, the same implementer corrects it and the same reviewer rechecks only
the correction and causally affected evidence. Do not create SHA checkpoints,
detached review snapshots, replacement worktrees, approval ledgers, or restart
the full review unless the correction invalidates earlier conclusions.

## Review lenses

- **World honesty:** Authority, identity, relationships, revisions, and state
  relevant to the claim have credible causal provenance.
- **Boundary honesty:** Integration and end-to-end labels match the real
  production boundaries exercised.
- **Oracle independence:** Expected results do not merely repeat the disputed
  production semantics.
- **Intended-cause failure:** Setup, routing, rejection, cleanup, and
  observation failures cannot counterfeit the expected result.
- **Risk pressure:** Denial, cancellation, exhaustion, concurrency, recovery,
  migration, and partial effects are tested when the production risk warrants
  them.
- **Proof economy:** Each test or parameterized family contributes meaningful,
  nonredundant evidence at an appropriate boundary.
- **Harness integrity:** Test support preserves relevant production invariants
  without introducing a test-only authority path.
- **Cost honesty:** Targets, compiler sessions, setup, external startup,
  retries, and retained artifacts are proportionate to the evidence produced.

Mutation probes, negative twins, and fault injection are optional tools for
high-risk or ambiguous claims. Do not require them when a test's sensitivity is
already clear. Do not expand test support into a compiler, API analyzer, source
fingerprint system, or proof ledger to guard hypothetical future mistakes.

## Correct and complete

Fix production defects in production. Repair fixture provenance and harness
boundaries before adding assertions. Consolidate redundant scenarios and
compiler sessions instead of accumulating green artifacts.

After correction, rerun the affected evidence and confirm the test reaches the
intended behavior. Review is complete when the relevant tests are honest,
appropriately scoped, reasonably economical, pass on the final change, and the
assigned reviewer finds them adequate. Report material findings, corrections,
tests run, expensive or environment-dependent checks not run, and accepted
residual risk.
