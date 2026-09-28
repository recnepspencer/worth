# Authoritative Mutation Evidence

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Authoritative mutation evidence is the **runtime surface for proving what a write batch bound, executed, and retained** on bridge-backed and graph-composition paths: target identity, existing-truth assertions, causality/provenance digests, and session/batch aggregates. Inspection surfaces expose **read-side** retained evidence; **write receipts** and `public_authoritative_mutation_evidence_support()` define what mutation claims are contractually supported.

## Why You Use It

- verify bridge-backed writes carried causality and provenance digests
- audit batch/session-level mutation counts and backend-verified assertions
- distinguish symbolic target resolution from retained authoritative assertions
- read `WorthQueryMutationSurfaceReport` / closeout support without implying store-backed durable mutation logs

## Core Mental Model

Evidence layers:

| Layer | Content |
|-------|---------|
| Target / existing truth | Identity binding, assertions before/after write |
| Provenance | Contract, effect intent, feedback, causality, strategy, execution digests |
| Batch / session | Aggregates, component counts, backend-verified update/delete counts |
| Support profile | `public_authoritative_mutation_evidence_support()` — verified vs deferred rows |

`workspace.inspections()?.inspect` may surface related **read** evidence; authoritative mutation evidence is the **write-path contract** certified under `authoritative-mutation-evidence-certification` matrix rows.

This evidence proves the local mutation lane. It does not by itself prove that
an external effect completed. Co-committed outbox observations, external-owner
dispatch posture, and receipt-bound recovery belong to application aftermath.

## Main Entry Points

- `WorthQueryWorkspace::public_authoritative_mutation_evidence_support()`
- `WorthQueryWorkspace::public_authoritative_mutation_evidence_closeout()`
- `WorthQueryWorkspace::public_mutation_surface_report()`
- `runtime/surface/mutation_evidence/` — `WorthQueryMutationProvenanceEvidence`, batch aggregates
- Matrix: `runtime/support_matrix.rs` row `authoritative-mutation-evidence-certification`

Tests: `runtime/tests/mutation/batch.rs`, `mutation_evidence/batch/`, `bridge_backed_verification_support.rs`.

## Typical Flow

1. Admit and execute a mutation batch through the workspace mutation surface.
2. Collect per-target provenance evidence (digests, outcome/failure classes on bridge writeback).
3. Read batch/session aggregate counters for certification or diagnostics.
4. Call `public_authoritative_mutation_evidence_support()` before UX or agents claim “backend verified” semantics.
5. Use inspection for **follow-up read** of retained state—not as a substitute for write receipts.

If the operation declared an external effect, continue from the sealed commit
receipt into the application-aftermath surface. Do not derive dispatch or
recovery authority from mutation digests or aggregate counters.

## How It Relates

- [Inspection](inspection.md) — per-target retained read evidence
- [Effects](../execution/effects.md) — authoring/DX; execution receipts in [authority-scoped effect execution](../execution/authority-scoped-effect-execution.md)
- [Cross-runtime causal inspection](cross-runtime-causal-inspection.md) — causal **explanation** reads vs write provenance
- [Declaration bridge continuation routing](../domain-capabilities/declaration-bridge-continuation-routing.md) — bridge writeback neighbors

## Good to Know

- `WorthQueryMutationProvenanceEvidence` exposes digest accessors (`contract_digest`, `causality_digest`, `authoritative_artifact_digest`, etc.)—stable for tests and agents.
- Batch types expose counts (`backend_verified_update_count`, `existing_truth_binding_count`, …) for aggregate auditing.
- Support varies by runtime posture; use `public_authoritative_mutation_evidence_support_for_posture` in tests as a pattern.

## Anti-Patterns

- Asserting full durable mutation audit trails while matrix/store rows remain deferred.
- Using inspection-only reads to prove a write was backend-verified without write receipts.
- Conflating naming/continuity mutation counts with authoritative retained assertions.
- Treating a committed mutation receipt or outbox row as proof that the
  external owner completed its consequence.

## Current Limits

Consult `public_authoritative_mutation_evidence_support()` and `authoritative-mutation-evidence-certification` in `runtime/support_matrix.rs` for the live row set. Representative honesty:

| Concern | Posture |
|---------|---------|
| Runtime-backed target identity + existing-truth binding | **Verified** on certified paths |
| Bridge-backed provenance digests | **Verified** where certification applies |
| Store-backed durable mutation archive | **Deferred** / blocked neighbors |
| Inspection as write proof | **Not a substitute** for mutation evidence receipts |

## Related Docs

- [Inspection](inspection.md)
- [Effects](../execution/effects.md)
- [Authority-scoped effect execution](../execution/authority-scoped-effect-execution.md)
- [Support matrix and admission](../foundations/support-matrix-and-admission.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
