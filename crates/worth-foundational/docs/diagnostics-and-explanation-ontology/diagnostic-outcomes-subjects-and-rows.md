# Diagnostic Outcomes, Subjects, And Rows

## What This Feature Is

This feature gives diagnostics a typed subject, a typed locator, a typed
outcome, and a family-distinct row model.

It is the layer that lets you say “this denial belongs to this commit receipt
at this exact locator” without flattening everything into one generic event
record.

## Why You Use It

Use this surface when you need to:

- explain what entity a diagnostic is about
- point at a transition, boundary artifact, source, or mismatch locus
- distinguish decision rows from failure rows, support rows, comparison rows,
  and provenance-ready rows
- preserve locality and widened-fallout meaning explicitly

This is the line between “we have diagnostic words” and “we can actually tell a
blind consumer what happened.”

## Stable Entry Points

Subject and locator helpers:

- `foundational_diagnostic_branch_candidate_subject(...)`
- `foundational_diagnostic_merge_verdict_subject(...)`
- `foundational_diagnostic_committed_authority_subject(...)`
- `foundational_diagnostic_commit_receipt_subject(...)`
- `foundational_diagnostic_branch_discard_subject(...)`
- `foundational_diagnostic_boundary_artifact_subject(...)`
- `foundational_diagnostic_locator_transition(...)`
- `foundational_diagnostic_locator_boundary_artifact(...)`
- `foundational_diagnostic_locator_source(...)`
- `foundational_diagnostic_locator_mismatch(...)`

Row types:

- `FoundationalDiagnosticDecisionRow`
- `FoundationalDiagnosticFailureRow`
- `FoundationalDiagnosticComparisonRow`
- `FoundationalDiagnosticSupportRow`
- `FoundationalDiagnosticProvenanceReadyRow`
- `FoundationalDiagnosticRow`
- `FoundationalDiagnosticRowFamily`

## Core Mental Model

Think of a diagnostic row as:

- one subject
- one locator
- one row family
- one outcome kind
- one extra typed payload that belongs only to that family

The family matters:

- decision rows describe accepted, advisory, denied, deferred, unsupported, or
  mismatch-style decisions
- failure rows describe integrity or construction problems
- comparison rows describe parity or mismatch against another semantic surface
- support rows describe support posture and evidence posture
- provenance-ready rows describe where evidence came from without pretending to
  be a receipt or authority artifact

If you expose one public `DiagnosticRow { family, ... }` and make the rest
optional, you have already weakened the model.

## How It Executes

You build typed subjects and locators first, then construct the row family that
matches the semantic job.

The row constructors capture:

- common fields: code, scope, severity, subject, locator, outcome kind, labels
- family-specific fields:
  - denial class on decision rows
  - breach class on failure rows
  - mismatch locator and evidence posture on comparison rows
  - support evidence posture on support rows
  - evidence origin locator and evidence posture on provenance-ready rows

## Small Example

```rust
use worth_foundational::{
    foundational_diagnostic_commit_receipt_subject,
    foundational_diagnostic_locator_transition,
    foundational_diagnostic_code, foundational_diagnostic_scope,
    FoundationalDiagnosticDecisionRow, FoundationalDiagnosticLocalityClaim,
    FoundationalDiagnosticOutcomeKind, FoundationalDiagnosticSeverity,
    FoundationalDiagnosticSemanticLabelSet, FoundationalDiagnosticWidenedFalloutPosture,
};

let subject = foundational_diagnostic_commit_receipt_subject(commit_id, receipt_identity);
let locator = foundational_diagnostic_locator_transition(transition_locator);

let row = FoundationalDiagnosticDecisionRow::new(
    foundational_diagnostic_code("commit.accepted").expect("canonical code"),
    foundational_diagnostic_scope("transitions.commit").expect("canonical scope"),
    FoundationalDiagnosticSeverity::Info,
    subject,
    locator,
    FoundationalDiagnosticOutcomeKind::Accepted,
    FoundationalDiagnosticSemanticLabelSet::default(),
    None,
    FoundationalDiagnosticLocalityClaim::LocalOnly,
    FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
);
```

## Real Example

A real explanation bundle usually mixes row families while keeping each one
typed:

```rust
use worth_foundational::{
    FoundationalDiagnosticComparisonRow, FoundationalDiagnosticDecisionRow,
    FoundationalDiagnosticFailureRow, FoundationalDiagnosticProvenanceReadyRow,
    FoundationalDiagnosticRow, FoundationalDiagnosticSupportRow,
};

let rows = vec![
    FoundationalDiagnosticRow::Decision(decision_row),
    FoundationalDiagnosticRow::Support(support_row),
    FoundationalDiagnosticRow::Comparison(comparison_row),
    FoundationalDiagnosticRow::ProvenanceReady(provenance_row),
    FoundationalDiagnosticRow::Failure(failure_row),
];

let _family_distinct_rows: (
    FoundationalDiagnosticDecisionRow,
    FoundationalDiagnosticSupportRow,
    FoundationalDiagnosticComparisonRow,
    FoundationalDiagnosticProvenanceReadyRow,
    FoundationalDiagnosticFailureRow,
) = (decision_row_typed, support_row_typed, comparison_row_typed, provenance_row_typed, failure_row_typed);
```

## How It Relates To Other Features

- [Diagnostic Primitives And Categories](./diagnostic-primitives-and-categories.md)
  supplies the code, scope, severity, denial, breach, and evidence-posture
  vocabulary.
- [Diagnostic Materialization And Support Reports](./diagnostic-materialization-and-support-reports.md)
  turns row inventories into support reports and explanation bundles.
- [Diagnostic Canonical Basis And Comparison](./diagnostic-canonical-basis-and-comparison.md)
  depends on these row families staying semantically distinct.

## Inspection And Debugging

When a row looks wrong, check these questions in order:

1. Is the subject the right thing?
2. Is the locator pointing at the right semantic locus?
3. Is the row in the right family?
4. Is this a denial, a breach, or evidence absence?
5. Did locality widen, and was that recorded explicitly?

That sequence will catch most bad row construction quickly.

## Anti-Patterns

- Do not use a failure row for a policy denial.
- Do not use a decision row for a reporting-surface corruption bug.
- Do not hide mismatch location in row labels when a comparison row can carry
  it structurally.
- Do not collapse provenance-ready evidence-origin information into ordinary
  explanation rows.

## Current Limits

- Rows are descriptive only. They do not certify support, carry receipts, or
  become authoritative transitions.
- Provenance-ready rows are groundwork for later provenance work, not a
  full provenance system by themselves.

## Related Docs

- [Diagnostic Materialization And Support Reports](./diagnostic-materialization-and-support-reports.md)
- [Diagnostic Canonical Basis And Comparison](./diagnostic-canonical-basis-and-comparison.md)
