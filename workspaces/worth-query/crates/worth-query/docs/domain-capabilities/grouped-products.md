# Grouped Products

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

Grouped products are the per-member route, receipt, and envelope projections
that hang off one retained grouped declaration.

Use them when you need to inspect how each member of the same neighborhood
would route, cross, or publish without losing the grouped aspect witness that
admitted the neighborhood.

Start from one `WorthQueryGroupedDeclarationArtifact` and then choose:

- `grouped_route_checked(...)` or the matching helper variant
- `grouped_receipt_checked(...)` or the matching helper variant
- `grouped_envelope_checked(...)` or the matching helper variant

Each grouped product retains:

- the original grouped declaration
- member index
- member role
- member-local aspect record
- the canonical single-member checked or proof-visible product

That lets you answer:

- did one member route differently from the rest?
- did one member defer while others issued receipts?
- what envelope or receipt truth belongs to this specific member?

These are inspection-heavy surfaces. They reuse the existing declaration-entry
product stack instead of creating grouped-local route, receipt, or envelope
semantics.
