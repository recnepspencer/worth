---
name: cad-qa-tests
description: Use when tests fail or need review during a CAD kernel production
  code change, or when asked to review CAD kernel test quality.
---

# CAD QA Tests

The production change is the work. Tests are evidence for it. Code is cheap;
certification (builds, slow suites, snapshot regeneration, review) is not.

Leave unaffected tests alone. When an affected fixture obstructs the change or
makes its certification fragile, simplify or replace it within the task. Defer
unrelated suite improvements.

## When a test fails after your change, classify it first

1. The change intentionally alters the contract: update obsolete expectations.
   Stop only for a contradiction or a decision nobody has made.
2. The change unintentionally violates the contract: fix the production code.
3. The test reaches into internals and behavior is unchanged: make the smallest
   coherent correction, or delete it if another test protects the same behavior
   through the appropriate public interface.
4. A fixture broke, or the test failed in setup: make the smallest correction
   that removes the cause. If the correction preserves the cause of recurring
   breakage, replace the affected fixture with a simpler direct test. Complete
   removal of displaced support code while preserving relevant guarantees.

## New and replacement tests

- Use the smallest set covering distinct failure mechanisms. Reuse existing
  protection; add a regression test when the bug would otherwise escape.
- Name the defect each test would catch and justify expectations independently
  of the behavior under test. If you cannot, do not write it.
- Keep setup, action, and observation clear. Existing factories and small local
  setup functions are appropriate. Avoid new fixture frameworks and unnecessary
  abstraction.
- Prefer the real object when it is fast and deterministic.
- Avoid tests that restate the implementation, assert incidental argument
  passing or formatting, or duplicate another. Protocol arguments and byte
  formats remain valid subjects when contractual.
- Ensure setup reaches the intended action and observation. Consider affected
  error paths.

## Cost

- Use the cheapest sufficient affected evidence. Include compile, link, and
  setup cost. Expensive verification needs a concrete unresolved concern or an
  explicit repository requirement. Batch necessary expensive work and reuse
  valid results. Remove incidental snapshot churn when practical.
- Derived values are not automatically harmless golden changes. Confirm that
  the protected guarantees remain correct; retain independent known-answer and
  contractual byte checks. Explain intentional expectation changes.
- After two failed attempts at the same test, diagnose the cause or change
  approach. Stop and report only when a concrete blocker remains.
- Stop when affected behavior is protected and required checks pass, with no
  known material in-scope defect remaining. Do not tidy unrelated tests or add
  certification machinery to certify the tests themselves.

## Report

Report material test changes and removals, changed or skipped assertions,
checks actually run, and concrete limitations. Do not weaken required protection
merely to regain green.

## CAD kernel

A CAD test proves the CAD consequence of the change under review. Apply one
test to every test: could this fail without any kernel code being wrong? If
yes, it is testing the platform. Remove it, or move it to a reproducer at
the platform's public owner.

- Assert the CAD result that follows from a platform guarantee, not the
  guarantee itself. Do not rebuild platform machinery in a test or fixture.
- Expected values come from a source independent of the code in dispute: an
  analytic result, a hand-computed value or an external CAD oracle. Known
  answers, not a second solver written in the test.
- No test-local epsilon. Compare exactly, against the declared tolerance
  policy, or against a bound the test states.
- Inputs enter through the production admission the claim depends on. A
  typed dimension is admitted as typed, not built from a float.
- A test proves its claim only at the boundary it exercises. Name it for
  what it observes.
- A refusal test asserts the typed outcome, not merely that something failed.
- Byte and digest reproduction is the determinism court's claim. Elsewhere
  they are derived values: assert meaning.
- Before adding an installed world, inspect the compile and link closure and
  use the narrowest existing surface. No new test binary per scenario.
