# Structural Correspondence and Historical Materialization

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Structural correspondence and historical materialization cover **identity evolution queries**, correspondence resolution, and historical envelopes that preserve payload plus support posture—implemented across `correspondence/`, `historical/`, and `correspondence_history.rs`. [Lineage and correspondence](lineage-and-correspondence.md) documents the **public query API** for correspondence/lineage; [historical diff and basis](historical-diff-and-basis.md) owns **diff/basis** semantics. This doc owns **structural materialization and history-shaped execution** boundaries.

## Why You Use It

- materialize historical views with explicit support posture in envelopes
- resolve correspondence evidence with structural (not merely textual) equality postures
- traverse `correspondence_history` artifacts without duplicating lineage DX entry points
- stay honest about store-backed historical replay deferred rows

## Core Mental Model

```text
Identity / correspondence request
  → admit (identity evolution or correspondence evaluation)
  → execute with structural correspondence rules
  → historical materialization envelope (payload + posture)
  → optional correspondence_history linkage
```

**Lineage/correspondence queries** (facade types like `CorrespondenceEvaluationRequest`) — see lineage doc for stable names. **Historical materialization** — envelope and structural neighbors here and in historical module tests.

## Main Entry Points

- `correspondence/` — evaluation, structural outcomes
- `historical/` — materialization, historical execution
- `correspondence_history.rs` — history-shaped linkage
- Lineage facade (cross-link): `admit_identity_evolution_query`, `execute_admitted_identity_evolution_query`
- Tests: `correspondence/tests.rs`, `historical/tests.rs`, `correspondence_history/tests.rs`

## Typical Flow

1. Choose lineage/correspondence API from [lineage doc](lineage-and-correspondence.md) for identity questions.
2. When historical **materialization** is required, admit historical basis ([basis lifecycle](basis-capability-lifecycle.md)) then historical execution path.
3. Resolve correspondence evidence; read structural outcome (unique, ambiguous, denied).
4. Attach or read correspondence history artifact when continuity across revisions matters.
5. Inspect envelope support posture before UX claims “full historical replay.”

## How It Relates

- [Lineage and correspondence](lineage-and-correspondence.md) — stable facade entry points and outcomes
- [Historical diff and basis](historical-diff-and-basis.md) — diff operators and basis for diffs
- [Basis capability lifecycle](basis-capability-lifecycle.md) — historical basis phases
- [Policy narrowing](../foundations/policy-tenant-and-relationship-proof-narrowing.md) — policy-aware historical diff **deferred**

## Good to Know

- Structural correspondence is stronger than naive field equality—outcomes encode disagreement and ambiguity.
- Historical envelopes carry **support posture** alongside payload so agents do not over-claim.
- `correspondence_history` tests document linkage invariants separate from one-shot evaluation.

## Anti-Patterns

- Duplicating entire lineage doc tables in app READMEs—link instead.
- Using historical diff APIs for pure identity correspondence without basis lifecycle receipts.
- Assuming policy-aware historical materialization because policy narrowing artifacts exist (execution parity deferred).

## Current Limits

| Concern | Status |
|---------|--------|
| Runtime-backed correspondence evaluation / identity evolution | **Verified** on certified paths |
| Historical materialization envelopes (runtime-backed) | **Verified** neighbors per tests |
| Store-backed historical replay | **Deferred** |
| Policy-aware historical diff | **Deferred** (policy narrowing profile) |

Consult module tests and runtime support matrix for row-level updates.

## Related Docs

- [Lineage and correspondence](lineage-and-correspondence.md)
- [Historical diff and basis](historical-diff-and-basis.md)
- [Basis capability lifecycle](basis-capability-lifecycle.md)
- [Support matrix and admission](../foundations/support-matrix-and-admission.md)
