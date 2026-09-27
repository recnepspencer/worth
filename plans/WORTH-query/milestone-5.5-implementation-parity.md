# Milestone 5.5 Implementation Parity

## Purpose

This note records the current implementation parity for Milestone 5.5 after the
workflow hardening passes. It exists to distinguish:

- shipped milestone substance
- shipped but under-proved areas
- explicit debt that should not be silently mistaken for completion

## Shipped

- Query-owned workflow taxonomy, basis binding, and declaration admission exist
  in `worth-query`.
- Lowering into relational mutation, relational merge, and bridge writeback
  authorities exists and remains authority-distinct.
- Query-shaped conflict inspection and post-merge inspection exist as proof-
  bearing artifacts rather than raw lower-crate passthrough.
- Conflict inspection now requires both an admitted query-owned conflict
  inspection declaration and explicit lower-authority merge-class evidence via
  a relational merge inspection artifact, and preserves denied merge ontology
  for deletion and topology-denied lanes rather than fabricating a generic
  admissible merge class.
- `worth-relational` now seals merge inspection minting behind a proof-bearing
  `RelationalMergeInspectionInput` plus authority-owned `inspect_execution_surface`
  access, so external callers cannot synthesize inspection proof directly from
  raw lowered summary data.
- Authority outcome shaping and replay bundle construction exist.
- Compile-fail boundaries exist for workflow declarations, lowered declarations,
  inspection artifacts, outcome artifacts, and replay artifacts.
- Production workflow code is now decomposed into domain-aligned submodules:
  `foundation`, `lowering`, `inspection`, and `performance`.
- Workflow tests are now decomposed into responsibility-specific submodules:
  `binding`, `lowering`, `inspection`, and `replay`.
- Workflow certification is now decomposed into `lane`, `matrix`, `row_catalog`,
  and `tests`.
- Certification now includes explicit rows for:
  - prediction-width explicitness
  - realized-width explicitness
  - zero-rediscovery parity on admitted lanes
  - denied merge-class explicitness for deletion/topology conflict inspection
  - unsupported merge-family lowering rejection
  - unsupported writeback-family lowering rejection
  - stale-workflow-denied rejection
  - compile-fail authority-override boundary as a required row

## Shipped But Still Narrow

- Workflow prediction and realized-width artifacts are present and certified, but
  the currently admitted lanes are still intentionally narrow and mostly width-1.
  The architecture is in place; richer width differentiation will come from
  broader admitted workflow families.
- Budget posture is explicit on declarations and authority outcomes, but the
  current implementation surface mostly proves `WithinBudget` and typed denial
  rather than a wider variety of admitted budget transitions.
- The workflow counter contract is now materially broader, including merge and
  writeback denials, staleness checks and denials, explicit rebind, replay
  bundle count, budget-cross count, and work avoided by query lowering. The
  admitted runtime still exercises only a narrow subset of interesting values.
- The workflow counter contract is now lane-explicit as well as phase-visible:
  mutation lowering, merge lowering, writeback declaration, writeback
  causality binding, conflict inspection, and post-merge inspection each have
  dedicated counters instead of relying only on aggregate lowering/inspection
  totals.
- Freshness policy is now lowering-significant instead of advisory-only:
  preview-origin writeback and merge declarations now distinguish exact-basis
  stale denial from allow-explicit-rebind behavior, and certification proves
  those denial surfaces separately.
- The spec names a broader budget-cross and broadening-denial counter matrix
  than the current admitted workflow families exercise. The current milestone
  is now honest about that narrowness: the counter surfaces and typed denials
  are shipped, and the unexercised variety belongs to later admitted workflow
  families rather than hidden 5.5 implementation debt.

## Explicit Debt

- None currently recorded for the Milestone 5.5 implementation boundary.

## Next Implications For 5.6

- 5.6 should build a unified facade over these proof-bearing workflow surfaces
  rather than re-deciding workflow admission or lowering.
- 5.6 should not invent convenience APIs that collapse declaration, lowering,
  inspection, and authority outcome boundaries back into one surface.
- If 5.6 wants stronger performance claims, it should either:
  - deepen the certified range of budget-cross and replay counter scenarios, or
  - explicitly narrow the public claim to the exercised workflow families.
