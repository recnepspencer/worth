# Milestone 3.11 Phase 5D Implementation Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

Status: complete

## Destination

Phase 5D closes Milestone 3.11 only if every visual-snapshot claim, public
example, permanent product consequence, resource bound, topology rule, and
successor handoff agrees on one final source state. It adds no product
capability. Its work is proof adjudication, ledger closure, and documentary
transition.

## Honesty Rules

- A VS row remains `OPEN` until its exact command has passed on the final
  implementation source and its fixture, typed result, mutation control, cost,
  and teardown claims have been checked against that output and the owning
  tests.
- Rows with identical commands may share one execution on the same source;
  duplicate Cargo work is not additional evidence.
- A filtered command must report a nonzero expected test count. Compilation
  with zero executed tests is not evidence.
- Earlier phase ledgers are inherited evidence, not substitutes for the final
  VS courtroom.
- Documentation-only transitions after executable proof reopen topology,
  format, diff, and constitutional checks. Code changes reopen the causally
  relevant behavioral, compile, warning, and cost gates.
- A failed gate produces a new plan before correction. No ledger row is
  partially upgraded around a red result.

## Exact VS Command Matrix

Run the six unique commands represented by VS-01 through VS-09:

1. VS-01: the sole Platform Pulse executable-world target.
2. VS-02: consolidated `visual_snapshot::` application contracts.
3. VS-03: consolidated `visual_identity::` application contracts.
4. VS-04 and VS-08: runtime spatial visual-snapshot tests.
5. VS-05 and VS-06: the complete consolidated `visual_` application portfolio.
6. VS-07 and VS-09: the canonical two-session compile-contract runner.

Also run the exact cross-lane ordinary-frame test required by VS-08 because the
row's spatial command cannot itself prove `[0; 11]` ordinary cost. Each command
must retain its reported counts and all executable-world cost/cleanup values.

## Broad Final Gates

After the focused courtroom is green:

- run full WORTH UI workspace tests with all features;
- run full workspace/all-target/all-feature clippy with warnings denied;
- run the WORTH UI test-topology checker and its CI meta-tests;
- run `cargo fmt --all -- --check`;
- enforce the canonical tracked WORTH UI line cap and a fail-closed dirty/new
  Rust line-cap audit;
- run dirty-function scrutiny and review every candidate against composition
  and domain laws;
- run `git diff --check`;
- run `boundary-check` and `agent-context check`.

No retry, ignored test, warning, line-cap exemption, new binary, new integration
target, new compile target, or nested Cargo session is admissible.

## Ledger Closure

Populate each VS-01 through VS-09 evidence cell with:

- the executed test or compile count;
- the real fixture/boundary used;
- the typed outcome or compile rejection proved;
- at least one named mutation control;
- the exact bounded cost posture;
- the explicit disposal/shutdown result.

Then change every status to `PROVED` in one transition. Extend the existing
Phase 5 topology audit so it requires exactly nine unique ordered rows, every
status `PROVED`, every evidence field nonempty, exact command ownership, and no
extra row. Add a negative mutation showing an empty or reopened row makes the
closure audit red.

## Documentary Closure

Only after the ledger audit passes:

- change the Milestone 3.11 spec from `Planned` to a dated completed status;
- close the Phase 5 contract and 5D batch with exact evidence;
- update the roadmap's 3.11 section with completed status, the honest product
  pulse consequence, and the final proof posture;
- update the 3.12 predecessor paragraph to say it inherits a closed 3.11
  snapshot/overlay/trace/resource boundary without claiming comparison is
  already implemented;
- preserve 3.12 ownership of semantic observation admission, bounded hot
  rebind, and identity-aware predecessor/successor comparison.

The final topology audit must require the same closure statements in the spec,
contract, roadmap, and successor document.

## Ordered Execution

1. Run the focused VS command matrix and cross-lane supplement.
2. Run broad workspace, topology, quality, and constitutional gates.
3. Adjudicate and populate all nine VS rows.
4. Add closed-ledger and documentary topology enforcement.
5. Transition spec, contract, roadmap, and 3.12 handoff.
6. Rerun all document-sensitive and constitutional gates.
7. Run one final `qa-loop`, `qa-tests`, and `code-quality-qa` review across the
   complete dirty milestone.
