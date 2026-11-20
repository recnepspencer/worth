# Diagnostic Primitives And Categories

## What This Feature Is

This feature gives you the smallest shared pieces of a diagnostic artifact:
codes, scopes, severity, denial class, breach class, evidence posture, artifact
kind, delivery class, and availability.

Use these types when you want diagnostics that mean the same thing across
WORTH crates instead of local strings and booleans that only one runtime
understands.

## Why You Use It

Use this surface when you need to:

- define a diagnostic code and scope that are stable across producers
- distinguish policy denial from integrity breach
- describe whether evidence is retained, deferred, reconstructable, redacted,
  or unavailable
- say what kind of diagnostic artifact you are building before you build rows
  or bundles

Do not skip this layer and jump straight to rows or reports with plain strings.
That is exactly how local diagnostics dialects form.

## Stable Entry Points

The public entry points live in the diagnostics facade:

- `foundational_diagnostic_code(...)`
- `foundational_diagnostic_scope(...)`
- `diagnostic_*_definition()` helpers for artifact kinds
- `evaluate_diagnostic_materialization_legality(...)`

Key public types:

- `FoundationalDiagnosticCodeId`
- `FoundationalDiagnosticScopeId`
- `FoundationalDiagnosticSeverity`
- `FoundationalDiagnosticDenialClass`
- `FoundationalDiagnosticBreachClass`
- `FoundationalDiagnosticEvidencePosture`
- `FoundationalDiagnosticArtifactKind`
- `FoundationalDiagnosticDeliveryClass`
- `FoundationalDiagnosticAvailability`

## Core Mental Model

Treat primitives as the contract that later rows and bundles must honor.

- A code identifies the kind of fact being reported.
- A scope identifies where that fact belongs.
- Severity tells you how urgent or serious the fact is.
- Denial class tells you why a decision was rejected.
- Breach class tells you the reporting or integrity surface itself is broken.
- Evidence posture tells you what kind of evidence the row rests on.
- Artifact kind tells you whether you are looking at a support report,
  explanation bundle, comparison bundle, and so on.
- Delivery class and availability tell you what it costs to surface the
  artifact and whether the evidence is actually present.

If you collapse any of those into text, you lose the shared meaning this crate
exists to provide.

## How It Executes

Primitive constructors validate canonical token shape up front.

- Diagnostic codes and scopes reject malformed labels.
- Artifact-kind definitions are crate-controlled, not caller-invented.
- Materialization legality is explicit instead of ambient. A delivery class and
  an availability posture must be compatible with the artifact kind you claim.

## Small Example

```rust
use worth_foundational::{
    foundational_diagnostic_code, foundational_diagnostic_scope,
    FoundationalDiagnosticBreachClass, FoundationalDiagnosticDeliveryClass,
    FoundationalDiagnosticEvidencePosture, FoundationalDiagnosticSeverity,
};

let code = foundational_diagnostic_code("merge.denied")
    .expect("diagnostic code must use canonical tokens");
let scope = foundational_diagnostic_scope("transitions.merge")
    .expect("diagnostic scope must use canonical tokens");

let severity = FoundationalDiagnosticSeverity::Warning;
let evidence_posture = FoundationalDiagnosticEvidencePosture::RetainedDirect;
let delivery = FoundationalDiagnosticDeliveryClass::CanDefer;
let breach = FoundationalDiagnosticBreachClass::ConstructionBug;
```

## Real Example

Use primitives before you construct rows so the row family stays honest:

```rust
use worth_foundational::{
    foundational_diagnostic_code, foundational_diagnostic_scope,
    FoundationalDiagnosticDenialClass, FoundationalDiagnosticSeverity,
};

let code = foundational_diagnostic_code("support.coverage.incomplete")
    .expect("canonical code");
let scope = foundational_diagnostic_scope("diagnostics.certified")
    .expect("canonical scope");

let severity = FoundationalDiagnosticSeverity::Error;
let denial = FoundationalDiagnosticDenialClass::CoverageIncomplete;

// These values then feed a decision row or support row instead of becoming
// free-form strings inside a report bag.
let _ = (code, scope, severity, denial);
```

## How It Relates To Other Features

- [Diagnostic Outcomes, Subjects, And Rows](./diagnostic-outcomes-subjects-and-rows.md)
  builds row families on top of these primitives.
- [Diagnostic Materialization And Support Reports](./diagnostic-materialization-and-support-reports.md)
  uses artifact kind, delivery class, and availability to decide what kind of
  surface can be materialized honestly.

## Inspection And Debugging

If something fails early here, it is usually one of these problems:

- a code or scope label is not canonical
- the artifact kind does not match the API being used
- the chosen delivery class and availability posture are not legal together

Fix primitive drift here instead of compensating for it later in row or bundle
logic.

## Anti-Patterns

- Do not use raw strings as public stand-ins for codes or scopes.
- Do not treat denial class and breach class as interchangeable.
- Do not infer availability from the current runtime environment.
- Do not create a generic “diagnostic artifact” wrapper and push artifact-kind
  meaning into comments.

## Current Limits

- This layer gives you shared vocabulary. It does not give you rows, bundles,
  certification, or comparison by itself.
- Artifact-kind definitions are intentionally crate-controlled. If you need a
  new public artifact kind, add it here first instead of inventing a local one.

## Related Docs

- [Diagnostic Outcomes, Subjects, And Rows](./diagnostic-outcomes-subjects-and-rows.md)
- [Diagnostic Materialization And Support Reports](./diagnostic-materialization-and-support-reports.md)
