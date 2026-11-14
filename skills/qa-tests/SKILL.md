---
name: qa-tests
description: Use when tests fail or need review during a production code change, or when asked to review public WORTH test quality.
---

# QA Tests

The production change is the work. Tests are evidence for it. Code is cheap;
certification (builds, slow suites, snapshot regeneration, review) is not.
CAD kernel tests use cad-qa-tests.

Leave unaffected tests alone. When an affected fixture obstructs the change or
makes its certification fragile, simplify or replace it within the task. Defer
unrelated suite improvements.

## Preproduction

WORTH is not in production. Do not preserve obsolete APIs, stored formats,
fixtures or golden bytes for backward compatibility. Within the change's
scope, delete displaced implementations, adapters, migrations and
compatibility-only tests; use one current representation. Recreate development
artifacts instead of migrating them. Preserve required current-version replay,
undo and recovery behavior. Add backward compatibility only when the user
explicitly requests it.

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
