# WORTH Query Test Requirements

WORTH Query tests prove product behavior and real architectural boundaries.
They do not certify the existence, naming, inventory, or completeness of other
tests.

The governing review guidance is
[`qa_review_guide.md`](../../docs/coding-guidelines/qa_review_guide.md), together with
the repository testing and architecture laws. A milestone specification should
state its material QA considerations in ordinary prose.

## Evidence lanes

- **Focused development:** run the reproducer, affected owner tests, and the
  smallest honest boundary smoke.
- **Ordinary CI:** run accepted requirement tests and mandatory repository
  architecture, formatting, lint, and size gates.
- **Scheduled:** run expensive scale, fuzz, soak, compatibility, destructive
  recovery, and environment-specific suites.

An expensive test belongs in ordinary development only when it is the actual
reproducer for the change.

## Evidence worth keeping

- tests that execute real production behavior through the relevant owner or
  public facade;
- denial tests for plausible illegal operations;
- persistence and recovery tests when state crosses a durable boundary;
- concurrency tests when operations genuinely race;
- independent semantic oracles whose results are compared with production;
- one economical compile-fail case for each materially important public
  authority boundary; and
- measured cost tests for a declared ordinary-path or scale guarantee.

Each important test or family should have a clear answer to: what plausible
product defect would make this fail?

## Evidence to delete

Delete tests and support systems whose primary purpose is to certify other
evidence, including:

- closure ledgers, evidence registries, source fingerprints, and test
  inventories;
- manifests that merely enumerate required tests, phases, fixtures, or
  coverage rows;
- source scanners used as substitutes for compiler or runtime boundaries;
- mutation exercises whose only purpose is to prove that an already-clear test
  can fail;
- certification bundles that aggregate green results without exercising a new
  product boundary; and
- repeated reviewer or test-suite recertification after immaterial edits.

Do not reject a model merely because it is called a manifest or oracle. Keep it
when it independently predicts product behavior and is compared against real
execution.

## Completion

Testing is complete when the relevant behavior is exercised at an honest
boundary, the focused and required CI checks pass on the final source, code
review judges the evidence proportionate to risk, and no known in-scope material
defect remains. Record the commands and material caveats in the final review
summary; do not build another artifact to certify that summary.
