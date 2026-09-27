# Milestone 3.11 Phase 5C Implementation Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

Status: closed

## Destination

Phase 5C makes the governed visual snapshot lifecycle usable without requiring
architectural archaeology. Application developers get one continuing visual
inspection contract, humans get the exact permanent Platform Pulse workflow,
and future human/agent consumers keep one runtime-owned evidence substrate.

The documentation must describe only implemented 3.11 semantics. In
particular, it must not turn pixels into authority, imply that an overlay
mutates its base frame, or claim identity-aware predecessor/successor
comparison before Milestone 3.12.

## Authority Review

- `worth_ui::facade::inspection` is the only public visual inspection API
  surface documented for application callers.
- `WorthUiNativeApplicationShell` owns the running session and mints the
  concrete geometry, pixel, and overlay grants from its sealed policy.
- `UiVisualSnapshotReceipt<P>` owns immutable snapshot affinity, coordinate,
  spatial, pixel, identity-trace, cost, and retention evidence. It is evidence,
  not execution or publication authority.
- the canonical host contract owns capture and presentation mechanics; mounted
  runtime truth owns the meaning explained by visual evidence.
- `apps/platform-pulse` remains the single human and executable product
  composition root.
- `plans/worth-ui/ai-diagnostics.md` governs the shared human/agent model and
  must distinguish implemented snapshot tracing from the committed 3.12
  comparison successor.

## Exact Public Example

`workspaces/worth-ui/docs/visual-inspection.md` will contain one fenced
`compile-pass-source` block copied exactly from:

`crates/worth-ui/tests/ui/visual_snapshot/pass/governed_visual_snapshot_lifecycle.rs`

The Phase 5 topology audit will extract the block, compare it byte-for-byte to
that existing compile-pass source, and require its existing
`visual_snapshot_facade_compile.rs` row in both compile-contract CSV
inventories. The batch creates no new fixture, harness, Cargo session, or
integration target.

## Developer Contract

The visual inspection guide must explain:

1. why a visual snapshot is a receipt-bound evidence bundle while a screenshot
   is only its optional pixel artifact;
2. mounted authority versus host observation versus disposable pixel evidence;
3. frame, node, region, and coordinate-scoped point targeting;
4. distinct visible-contributor and total-ordered hit-test results;
5. client physical pixels, logical-to-physical transforms, half-open regions,
   clipping, and edge posture;
6. current, retained predecessor, historical, superseded, expired, omitted,
   denied, and indeterminate outcomes;
7. concrete grants, exact disclosure matching, redaction/artifact policy,
   deadlines, query/record/byte/snapshot bounds, and explicit disposal;
8. overlay publication as a successor mounted frame followed by a typed clear
   successor;
9. mounted receipt through authored provenance and evidence references;
10. cancellation, retry, indeterminate affinity, predecessor preservation, and
    shutdown recovery.

## Human Platform Pulse Contract

`docs/application-lifecycle.md` will identify the canonical 160-by-96 world:

- blue background;
- yellow inset target from `[48, 24]` through `[112, 72]`;
- target point `[80, 48]` and background point `[16, 16]`;
- temporary magenta identity border;
- version-2 `VisualSnapshotCaptured`, `VisualPointTrace`,
  `VisualOverlayPublished`, `VisualOverlayCleared`, and
  `VisualSnapshotRetired` observations.

It will retain the existing launch, valid edit, malformed edit, recovery,
normal close, in-process certification, and executable-world commands. It will
show how to read the target’s authored semantic name, mounted receipt,
snapshot/frame affinity, source path, and overlay successor frame from the
prefixed JSON observation without describing the observation stream as
authority.

## Diagnostic Architecture Correction

The visual snapshot section of `ai-diagnostics.md` will use implemented names
and split closure into:

- implemented in 3.11: capture, point/region adjudication, mounted-to-authored
  trace, overlay, bounded retention, and disposal;
- committed successor in 3.12: identity-aware predecessor/successor
  comparison.

Both human and agent consumers remain projections over the same receipts.

## Mechanical Proof

Extend the existing Phase 5 topology audit to require:

- every mandated developer topic;
- every exact human command and Pulse observation;
- the explicit 3.12 comparison reservation;
- byte-for-byte equality between the documentation fence and existing
  compile-pass source;
- the existing pass source’s presence in both compile-contract inventories;
- unchanged ordered batch states and still-open VS ledger rows.

Run the existing compile matrix, focused application witness, documentation
topology audit, `qa-loop`, `qa-tests`, and `code-quality-qa`. Phase 5D remains
locked until these proofs pass on the same source.

## Ordered Implementation

1. Create the continuing visual inspection guide with the exact compile source.
2. Revise the human Platform Pulse section against the implemented world.
3. Correct the governing diagnostic document’s 3.11/3.12 boundary.
4. Add documentation topology enforcement to the existing Phase 5 audit.
5. Run the existing compile and application witnesses plus static and
   constitutional QA.
6. Close 5C, keep every VS row open, and unlock only 5D.

## Closure Evidence

- The existing Phase 5 topology target passes 7 documentation, contract,
  ledger, and mutation tests. It compares the full documented program
  byte-for-byte with the existing compile-pass source and proves the small
  example is contained in that same source.
- The canonical compile matrix passes 35 fail and 13 pass targets in exactly 2
  Cargo sessions. The focused Platform Pulse application witness passes 4
  production-path tests.
- Product-document QA covers task-first purpose, stable entry points, lifecycle,
  small and real examples, adjacent features, debugging, anti-patterns, and
  current limits. The human workflow names the exact target, points, overlay,
  trace, edits, recovery source, certification command, and shutdown fields.
- Certification all-target/all-feature clippy passes with warnings denied. The
  parent and child topology files are 335 and 178 lines, and dirty-function
  scrutiny reports no Phase 5C candidate.
- `cargo fmt --check`, `boundary-check`, `agent-context check`, and
  `git diff --check` pass on the closing source.

Phase 5D is now the sole unlocked batch. Every VS-01 through VS-09 verdict
remains `OPEN`.
