# Milestone 3.10.1 Phase 5 Implementation Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Objective

Condense the product surface to two honest audiences:

- `worth_ui::facade::app` owns the ordinary application lifecycle, mounted-frame request,
  exhaustive outcome, and affine continuation/recovery handles.
- `worth_ui::facade::inspection` owns compact read-only evidence.

Host implementers consume `worth-ui-host-contract` directly. Certification-only and
internal runtime vocabulary must not remain public merely because current tests use it.

## Boundary Review

### Current authority

- `WorthUiActiveApplicationSession::execute_mounted_frame` is already the only default
  product execution route.
- `facade::mounted` currently re-exports frame assembly, admission, attempt,
  presentation, retention, projection storage, host mechanics, identity views, and
  certification vocabulary as one broad public surface.
- `facade::runtime` currently re-exports lane executors, raw runtime launch, frame
  certification, Query ingress, and plan/runtime internals.
- `facade::host` mirrors host-contract mechanics through the product crate.
- `facade::registry` mixes authored descriptor inputs with frozen runtime snapshots.
- `facade::inspection` is read-only in authority but is not yet governed by an exact
  symbol manifest.

### Real callers

The caller inventory found no production workspace consumer of
`worth_ui::facade::{mounted,runtime,inspection,host,registry}`. Their current consumers
are product compile fixtures, certification scenarios, and tests. Certification usage
does not justify a product audience.

`execute_mounted_frame` currently has certification coverage but no downstream product
contract fixture that performs the complete ordinary journey and branches on every
outcome family.

### Destination authority

- Product lifecycle types required to call or exhaustively branch on
  `execute_mounted_frame` move to `facade::app`.
- Compact immutable evidence types required by a product inspector move to
  `facade::inspection`.
- Host protocol, adapter, registration-mechanics, and native completion types are
  consumed from `worth-ui-host-contract`.
- Authored descriptor inputs move to the declaration audience.
- Frozen registry/runtime snapshots move to inspection or certification according to
  their real caller.
- Lane execution, frame assembly, admission, presentation attempts, retention
  coordinators, raw runtime launch, and certification types are consumed from their
  internal or support owner and are absent from the product facade.
- No compatibility alias, root re-export, prelude, or generic request bag replaces a
  removed surface.

No crate extraction is part of Phase 5. The work changes audience publication, not
operational ownership.

## Public DX Contract

The Query-free ordinary path must compile with only the app audience:

```rust
use worth_ui::facade::app::{
    UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline, WorthUi,
};

let app = WorthUi::app().freeze()?;
let mut session = app.launch()?;
let outcome = session.execute_mounted_frame(
    UiMountedFrameRequest::all_bound_surfaces(),
    UiPresentationDeadline::at_tick(1),
    0,
    |_| {},
)?;

match outcome {
    UiMountedFrameOutcome::Published(receipt) => inspect(receipt),
    UiMountedFrameOutcome::Unchanged(receipt) => inspect(receipt),
    UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => recover(rejected),
    UiMountedFrameOutcome::InFlight(in_flight) => continue_from(in_flight),
    UiMountedFrameOutcome::PresentationIndeterminate(frame) => reconcile(frame),
    UiMountedFrameOutcome::AdmissionDenied(denial) => retry(denial),
    UiMountedFrameOutcome::RetentionDenied(denial) => retry(denial),
    UiMountedFrameOutcome::CompletionDenied(denial) => reject_foreign(denial),
    UiMountedFrameOutcome::Reconciled(receipt) => inspect(receipt),
}
```

The exact exhaustive variants remain those defined by Milestone 3.10. The fixture may
use a real published application source when empty bootstrap state cannot produce every
branch, but it may not import runtime, mounted, host, Query, or certification facades.

## Implementation Batches

### Batch 1 — Exact product API manifest and enforcement

1. Add `plans/worth-ui/milestone-3.10.1-phase-5-product-api.toml`.
2. Record every product symbol with:
   - symbol;
   - owning audience;
   - stability posture;
   - authority posture;
   - named real caller journey; and
   - source facade file.
3. Add a syntax-aware certification audit that compares the manifest with actual
   public modules and re-exports.
4. Reject:
   - an unmanifested public export;
   - a manifest row without a real caller;
   - duplicate audience ownership;
   - root/prelude wildcard publication; and
   - a host-contract or certification symbol in the app audience.
5. Add hostile fixtures for unmanifested growth and duplicate ownership.

### Batch 2 — Ordinary app journey

1. Re-export the minimal mounted request, deadline, exhaustive outcome, stop, and
   continuation/recovery types through `facade::app`.
2. Keep constructors private when a value must originate from a runtime transition.
3. Add one downstream compile-pass fixture covering prepare, activate, replacement,
   mounted execution, exhaustive branching, and compact inspection.
4. Add a second Query-free headless fixture proving no dummy Query, adapter,
   inspection, or recovery setup is required.

### Batch 3 — Mounted and runtime audience removal

1. Classify every current `facade::mounted` and `facade::runtime` export as:
   - app outcome/input;
   - inspection;
   - host contract;
   - certification/support; or
   - internal only.
2. Migrate certification callers to the direct owner or a gated support extension.
3. Remove `pub mod mounted` and `pub mod runtime` from the product facade once no
   documented product caller remains.
4. Do not retain an advanced mounted/runtime audience without a named non-test caller.

### Batch 4 — Host, registry, and inspection audience split

1. Migrate adapter-facing callers from `facade::host` to
   `worth-ui-host-contract`.
2. Move authored registry descriptor inputs to `facade::declaration`.
3. Move immutable frozen snapshot evidence to `facade::inspection` only when an
   ordinary inspector has a real journey; otherwise move it to support/internal
   ownership.
4. Remove the product `host` and `registry` facade modules after migration.
5. Narrow `facade::inspection` to references, requests, denials, and immutable
   receipts; reject mutable stores, builders, materializers, and reconstruction.

### Batch 5 — Mid-protocol and outcome hostility

1. Compile-fail construction of prepared frames, publication attempts,
   reconciliation input, mounted identity indexes, and cost evidence.
2. Compile-fail publication, reconciliation, completion, or retry without the exact
   sealed handle returned by the ordinary route.
3. Compile-fail ordinary imports of raw host mechanics, certification authority, raw
   runtime, and removed mounted/runtime modules.
4. Behaviorally prove every public mounted outcome remains distinguishable and carries
   only lawful next actions.

### Batch 6 — Closure

1. Run the canonical compile-contract runner without adding Cargo sessions.
2. Run the ordinary journey, Query-free, replacement, mounted, headless, egui,
   inspection, and topology suites.
3. Run default and certification builds, strict clippy, boundary-check, and
   agent-context checks.
4. Audit every dirty WORTH UI code/test file for the 400-line cap, one semantic
   responsibility, function decomposition, and argument advisories.
5. Mark the Phase 5 proof ledger only after every row has fresh evidence.

## Causal Reopen Rules

- Any app export change reopens manifest exactness, ordinary journey, audience
  hostility, and no-ceremony guarantees.
- Any continuation or outcome change reopens mid-protocol hostility and mounted
  outcome behavior.
- Any host/registry/inspection migration reopens audience ownership and inspection
  non-authority.
- Any compile-fixture topology change reopens consolidated build-cost evidence.
- Any runtime or mounting change reopens Milestone 3.10 publication, predecessor,
  failure-preservation, and cost guarantees.

## Non-Goals

- Phase 6 predecessor-source and dead-route cleanup beyond what is required to remove
  the Phase 5 public audiences.
- New snapshot, rebind, intent, service, or appearance semantics.
- A new prelude, compatibility facade, or broad advanced audience.
- Crate extraction or a new compile-contract workspace.
