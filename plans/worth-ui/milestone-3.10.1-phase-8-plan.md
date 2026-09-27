# Milestone 3.10.1 Phase 8 Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Status

Planned from the exact Phase 7 closing source. Phase 7 is closed with all
twelve proof-ledger claims marked `PROVED`, the canonical 23-fail/12-pass
compiler matrix using two Cargo sessions, and the exact-source full lane green.

## Objective

Close Milestone 3.10.1 by making the documented architecture, public developer
journey, migration posture, future insertion map, roadmap, generated context,
and mechanical closure evidence agree with the implementation.

Phase 8 is not a prose cleanup. Documentation examples are public-surface
claims. The closeout must mechanically reject predecessor APIs, missing owner
maps, open proof rows, and a roadmap that hands Milestone 3.11 a transitional
architecture.

## Opening Authority Review

The implemented ordinary lifecycle is:

```text
WorthUi::app()
-> WorthUiApplicationBuilder
-> freeze()
-> WorthUiApp
-> launch()
-> WorthUiActiveApplicationSession
-> execute_mounted_frame(...)
-> typed publication, denial, in-flight, or indeterminate outcome
```

The application source closure supplied to `execute_mounted_frame` is the
ordinary input collection seam. `execute_framework_turn` is crate-private
runtime machinery and must not appear as a downstream entry point.

File and Rust authoring converge before runtime:

```text
file bytes -> worth-ui-dsl compiler ----\
                                        -> WorthUiWatchedCandidateSubmission
Rust authored atoms -> worth-ui-dsl ----/
-> WorthUiApplicationBuilder::with_candidate_submission(...)
-> runtime preparation
```

Inspection borrows prepared or active truth and returns receipts. It does not
provide plan, publication, mutation, reconstruction, or execution authority.

The runtime subsystem map already names seven owners and exact future homes for
Milestones 3.11, 3.12, 3.17, and 3.18. Phase 8 documents and cross-checks that
map; it does not invent a parallel topology.

## Required Documentation

The public documentation set will be:

1. `workspaces/worth-ui/README.md`
   - compact workspace entry and canonical reading order;
2. `workspaces/worth-ui/docs/architecture.md`
   - source-to-mounted owner flow and allowed dependency direction;
3. `workspaces/worth-ui/docs/authored-composition.md`
   - file and Rust authoring, DSL ownership, convergence, denials, and current
     support limits;
4. `workspaces/worth-ui/docs/application-lifecycle.md`
   - ordinary application journey plus typed outcome and recovery posture;
5. `workspaces/worth-ui/docs/inspection.md`
   - prepared/active inspection without operational authority;
6. `workspaces/worth-ui/docs/runtime-subsystems.md`
   - seven runtime owners, allowed dependencies, failure preservation, cost
     posture, and future insertion points;
7. `workspaces/worth-ui/docs/migration-3.10.1.md`
   - removed routes and their intended named audience replacements;
8. `workspaces/worth-ui/docs/query-binding.md`
   - Query-backed use through the ordinary mounted-frame path;
9. `workspaces/worth-ui/AI_README.md`
   - compact AI discovery using only the closed route; and
10. `workspaces/worth-ui/docs/worth-ui-readme.md`
    - architecture orientation reconciled with the canonical builder and
      mounted-frame entry.

Product feature docs follow the feature-documentation shape: plain-language
problem statement, stable entry points, mental model, execution order, small
and realistic examples, adjacent features, inspection, anti-patterns, limits,
and related docs.

## Mechanical Closeout Authority

Add `plans/worth-ui/milestone-3.10.1-phase-8-closeout.toml` as the exact
documentation and closeout manifest. It must name:

- every required document and required heading;
- the source file and witness for each executable example claim;
- forbidden predecessor or internal-route tokens;
- the exact seven runtime subsystem owners;
- the exact Milestone 3.11, 3.12, 3.17, and 3.18 insertion owners;
- every Phase 3 through Phase 8 proof ledger;
- the generated-context command;
- the authoritative verification commands; and
- the roadmap and milestone completion markers.

Extend the existing Milestone 3.10.1 topology owner rather than adding an
integration target or a parallel script. The audit must reject:

- a missing required document or section;
- a stale predecessor/internal route in public docs;
- a documentation example without its named production compile/behavioral
  witness;
- a subsystem or future-insertion mapping that differs from the Phase 4
  authority ledger;
- `NOT_EVALUATED`, `ambiguous`, `transitional`, empty evidence, or a status
  other than `PROVED` in a closing proof ledger;
- a missing completion marker in the milestone or roadmap; and
- a changed closeout command set.

## Implementation Batches

### Batch 1 - Closeout manifest and hostile audit

1. Add this plan and the Phase 8 proof ledger.
2. Add the Phase 8 closeout manifest with exact documents, headings, witnesses,
   forbidden routes, owner mappings, ledgers, commands, and completion markers.
3. Add one bounded `phase8_closeout` topology owner under the existing
   Milestone 3.10.1 audit.
4. Add hostile unit mutations for stale routes, missing sections, wrong future
   insertion, open ledger rows, and missing roadmap completion.

### Batch 2 - Canonical architecture and authoring docs

1. Write the source-to-mounted architecture overview.
2. Write the file/Rust authored composition guide.
3. Document filesystem mechanics as runtime ingress and authored-language
   meaning as DSL-owned.
4. Explain steady-state source exclusion and current DSL support limits without
   promising later language features.

### Batch 3 - Lifecycle, recovery, and inspection docs

1. Rewrite the application lifecycle around
   `WorthUiApplicationBuilder` and `execute_mounted_frame`.
2. Remove ordinary `execute_framework_turn`, `facade::mounted`, and other
   predecessor/internal routes.
3. Provide a small ordinary example derived from the canonical downstream
   compile-pass fixture.
4. Provide advanced typed outcome/recovery examples that begin only from
   handles returned by the ordinary mounted-frame call.
5. Write an inspection guide that uses `facade::inspection` and session/app
   receipts without importing operational internals.
6. Reconcile Query binding and AI discovery with the same ordinary path.

### Batch 4 - Runtime map, migration, and roadmap handoff

1. Write the seven-owner runtime subsystem map with allowed directions,
   failure-preservation boundaries, cost posture, and future insertion homes.
2. Write migration notes mapping every removed predecessor route to its named
   audience replacement or explicit removal.
3. Reconcile the long runtime orientation without duplicating product guidance.
4. Mark Milestone 3.10.1 complete in its spec and roadmap.
5. State that Milestone 3.11 begins from the closed visual-snapshot insertion
   owner and cannot reopen source/runtime predecessor routes.

### Batch 5 - Generated context and exact-source verification

1. Run the agent-context generator/check; never hand-edit generated context.
2. Close every Phase 8 proof-ledger row with exact evidence.
3. Run the fresh-reader compiler fixture and stale-doc/insertion/ledger hostile
   tests.
4. Run the exact Milestone 3.10.1 inventory, full topology, application
   contracts, canonical compiler matrix, strict Clippy, formatting, WORTH UI
   line caps, boundary-check, agent-context, standalone test-topology gate,
   full lane, and diff hygiene after the last causal edit.

## Proof Strategy

- The existing downstream app-journey compile pass proves the documented
  ordinary call shape.
- Existing compile failures for predecessor builders, ordinary framework-turn
  access, and inspection authority prove stale routes are not aliases.
- The Phase 8 doc audit binds each guide to exact headings, source owners, and
  witnesses.
- Hostile in-memory manifest/document mutations prove the audit goes red for a
  stale route, missing section, wrong insertion owner, or open ledger.
- The Phase 4 runtime subsystem ledger remains the topology authority; the
  Phase 8 map must match it exactly.
- The Phase 7 closing evidence remains the cost authority; Phase 8 may cite but
  may not reinterpret its measurements.

## Causal Reopen Rules

- Editing a lifecycle example reopens the downstream compile-pass and
  predecessor-route hostility.
- Editing authoring or source-owner docs reopens file/Rust parity and
  DSL/runtime ownership audits.
- Editing inspection docs reopens inspection non-authority compile and
  behavioral evidence.
- Editing the subsystem map or roadmap reopens the Phase 4 insertion and cycle
  audits.
- Editing a proof ledger or closeout manifest reopens ledger-closure hostility.
- Editing generated context inputs reopens `agent-context check`.
- Any final Rust audit fix reopens strict Clippy, its focused hostility tests,
  the content fingerprint, and the exact-source full lane.

## Non-Goals

- New runtime, DSL, Query, host, mounted, inspection, or product behavior.
- Compatibility aliases for removed APIs.
- A new documentation framework, integration target, fixture workspace, Cargo
  session, or CI runner.
- Documentation of certification-only methods as product APIs.
- Beginning Milestone 3.11 implementation.
