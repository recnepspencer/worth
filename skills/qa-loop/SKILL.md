---
name: qa-loop
description: Review and correct a completed WORTH implementation slice against its specification, relevant engineering laws, tests, and code-review risks. Use after coherent implementation work when Codex should find material defects, fix root causes, and verify the final change without creating a progressive proof ledger.
---

# QA Loop

Review the implementation, not a certification system around it. Requirements
come from the governing specification, public contracts, repository laws, and
the behavior the change necessarily promises. Tests are evidence for those
requirements; code review decides whether that evidence is adequate.

Read the repository instructions, governing specification, relevant engineering
laws, and `docs/coding-guidelines/qa_review_guide.md`. Inspect the task's diff,
the affected production paths, adjacent owners and consumers, and the tests that
claim to protect the change.

## Establish scope

State the implementation slice and the material requirements under review in
plain language. Select only the QA categories that are causally relevant. Do not
manufacture concerns to fill every category or create requirement identifiers,
status rows, predecessor portfolios, or phase-reopening records.

Include adjacent authority, lifecycle, persistence, integration, or facade
boundaries when they can invalidate the change. Preserve unrelated user work and
report unrelated repository debt as a caveat rather than expanding the task.

## Independent review

Use the reviewer arrangement selected by the user or repository. Do not replace
it with a hard-coded model or create new reviewer instances merely because a
small correction occurred. Persistent reviewers may follow the correction cycle
as long as they inspect the final diff and current test results before clearing
the work.

Give reviewers the specification, relevant laws, current diff, affected source
and tests, and test results. Ask for material correctness, authority, security,
lifecycle, recovery, performance, integration, DX, test, and structural defects
that are relevant to the change. Reviewers provide independent judgment; the
primary agent verifies findings against the repository and owns corrections.

If outside review is required but unavailable, report that limitation. Do not
relabel the primary pass as independent review.

## Review and correct

Trace real success, denial, failure, cancellation, partial-effect, recovery, and
teardown paths where they matter. Look for wording-compliant implementations
that defeat the specification's intent, competing authority paths, stale state,
dishonest fallbacks, resource escape, and tests that can pass for the wrong
reason.

For each supported finding, state:

1. the affected requirement
2. the concrete defect and evidence
3. the required root-cause correction
4. the test or review evidence needed after correction

Fix root causes rather than padding tests. Search the affected semantic family
for the same defect, then rerun only the causally affected tests and repository
gates. Use focused checks during correction, ordinary CI checks for the final
change, and scheduled expensive suites only when they are relevant or explicitly
required.

## Completion

QA is complete when:

- the affected requirements are implemented on the final source
- appropriate focused and required repository checks pass
- code review finds the tests adequate for the risk
- required independent reviewers have inspected the final diff
- no known in-scope material defect remains

Report the requirements reviewed, material findings and corrections, tests and
repository gates run, reviewer conclusions, unrun environment-dependent checks,
and any scoped caveats. Do not create evidence to certify the evidence, reopen
historical phases, or continue adding proof after review finds the residual risk
acceptable.
