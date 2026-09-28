# WORTH UI 3.11 Phase 4 Batch 4C Implementation Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Closure Claim

The ordinary checked-in Platform Pulse product captures one bounded pixel
snapshot of its first mounted presentation, independently adjudicates the
fixed target and background points, traces the target to its stable authored
semantic name, publishes and visibly clears the canonical mounted overlay,
then continues replacement, malformed preservation, recovery, normal close,
and zero-residue shutdown in the same native child.

Protocol schema v2 carries only bounded, derived correlation evidence.
It cannot construct or re-enter runtime authority. The executable-world
runner accepts the claim only when typed product observations and independent
process-bound native pixels agree.

## Authority and Truth Owners

- Declaration artifacts retain the authored semantic name from which their
  existing declaration identity is derived. The runtime visual-trace
  projection may disclose that stable name; the wire may only copy it.
- The active native application shell owns mounted inspection, concrete visual
  grants, snapshot capture, point adjudication, overlay publication, clear,
  disposal, replacement, and shutdown.
- The visual snapshot receipt alone owns exact frame, surface, binding,
  coordinate, region, trace, pixel, cost, and retained-snapshot affinity.
- The observation stream owns output-only lifecycle ordering and correlation.
  It receives concrete public receipts and derives private wire structs; it
  never accepts caller-authored raw identity values.
- The product binary owns one finite visual exercise in ordinary code. It
  retains real linear handles between updates and has no runner or direct egui
  drawing dependency.
- The executable world owns independent fixed scenario constants, native
  captures, same-child typestate, mutation controls, deadlines, budgets, and
  teardown.

## Batch 4C.1 — Public Trace Truth and Protocol v2

1. Retain `key_basis` as the declaration artifact's authored semantic name and
   expose it read-only through the visual declaration reference.
2. Add native-shell mounted-frame inspection delegation so product code uses
   the same public facade as downstream applications.
3. Add the missing snapshot-affinity accessors required for non-authoritative
   wire correlation.
4. Introduce private-field protocol-v2 snapshot, point-trace,
   overlay-published, overlay-cleared, and snapshot-retired observation types.
   Float observations use exact bit representations; no screenshot bytes enter
   stdout.
5. Extend the lifecycle stream with a typed visual progression:
   `AwaitingSnapshot -> SnapshotCaptured -> IdentityTraced ->
   OverlayPublished -> OverlayCleared -> SnapshotRetired`.
6. Require the initial replacement to follow overlay clear, and require
   retirement to cite the replacement's current successor before later
   lifecycle observations.
7. Project wire evidence only from concrete snapshot, adjudication, target,
   published-overlay, cleared-overlay, supersession, disposal, and replacement
   receipts.
8. Reject schema v1 before any outcome can be adjudicated.

Gate: focused protocol round-trip, version rejection, positive progression,
out-of-order, foreign-affinity, and bounded-payload tests; public compile
contracts remain green.

## Batch 4C.2 — Ordinary One-Shot Product Exercise

1. Set an explicit product visual policy for exactly one capture of the fixed
   160-by-96 logical page, a declared maximum 4x physical scale and
   983,040-byte ceiling, bounded point results/candidates, and bounded retained
   structure.
2. Add a private product visual-execution module with real state-bearing
   phases. It begins only after first publication and a short native-settlement
   dwell.
3. Capture the current presented surface through the public shell, poll the
   asynchronous host readback, and accept only
   `UiVisualSnapshotOutcome::Captured<UiPixelsRequired>`.
4. Adjudicate the checked-in target and background points separately. Require
   target visible and hit results to agree with each other, require the
   background not to resolve to the target, and require the target declaration
   name to equal the checked-in authored name.
5. Publish snapshot and point-trace observations from those concrete receipts.
6. Derive the overlay target from the exact selected hit target, publish the
   mounted successor, emit its observation, and retain it long enough for an
   external capture.
7. Clear through a distinct mounted successor, emit its observation, and
   retain the original snapshot and selected target through the first
   replacement.
8. After green replacement, prove the retained target is explicitly
   superseded, dispose the snapshot, emit retirement evidence, and require
   shutdown to report zero remaining visual capture and overlay resources.
9. Map every visual failure to a typed terminal family and close the native
   window without claiming later lifecycle success.

Gate: product protocol/unit tests, focused runtime integration, warnings denied,
source topology, and no-cfg-test/no-direct-egui audit.

## Batch 4C.3 — Same-Child Executable Courtroom

1. Extend executable typestate exactly:
   `Published<InitialBlue> -> Published<SnapshotCaptured<InitialBlue>> ->
   Published<IdentityTraced<InitialBlue>> ->
   Published<OverlayPublished<InitialBlue>> ->
   Published<OverlayCleared<InitialBlue>>`.
2. Make green source application available only from
   `Published<OverlayCleared<InitialBlue>>`; visual events cannot be skipped by
   the cumulative journey.
3. At each transition consume the next schema-v2 event, preserve the same
   process/window/client binding, and carry the prior evidence forward.
4. Capture native pixels after overlay publication and after clear. Check the
   fixed two-pixel magenta target border, unchanged yellow interior, unchanged
   blue background control, and complete magenta removal after clear.
5. Before each monitor-region capture, have the sealed Windows adapter raise
   the already process-bound client to the top of the normal z-order, cross a
   DWM compositor barrier, mint a private one-use exposure witness, and consume
   that witness in the exact no-resampling capture. Product code owns none of
   this courtroom operation.
6. Adjudicate product target and background evidence independently against
   checked-in names, points, and expected target bounds. Product events alone
   and pixels alone each remain insufficient.
7. Consume both green replacement and snapshot-retired observations before the
   existing malformed-preservation and recovery progression resumes.
8. Extend shutdown adjudication to prove zero visual capture and overlay
   residue, then preserve successful exit, watcher cleanup, and sandbox
   deletion.
9. Keep the inherited single executable target, single child/window, two Cargo
   compile sessions, at most eight native captures, 256 events, one MiB
   encoded observations, five-second transition deadlines, and thirty-second
   total ceiling.

Gate: `cargo test --manifest-path workspaces/worth-ui/Cargo.toml
-p worth-ui-platform-pulse --features executable-world --test
executable_world` passes the full cumulative
journey and explicit event-only, pixel-only, sole-node, wrong-target, and
restored-pixel mutation controls.

## Phase 4 Closure Evidence

- VS-01: exact ordinary product entry, one child, typed snapshot-to-overlay
  progression, independent native consequence, inherited lifecycle, and
  teardown.
- VS-06: retained subordinate overlay authority, explicit clear successor,
  supersession, disposal, and zero shutdown residue.
- VS-07: public compile twins, schema-v1 rejection, output-only wire topology,
  production-source audit, and no runtime-internal runner imports.
- P4-02 and P4-15 through P4-19 close only from their exact compile, protocol,
  product, runner, pixel, and cleanup evidence.
- P4-20 remains open until focused and broad tests, warnings, composition,
  line-cap, ledger, boundary, and agent-context gates all pass on the same
  source.

## Failure Replanning Rule

Any failed compile, test, topology audit, pixel predicate, timing gate, or
ledger audit reopens every causally affected claim. Before editing again,
record a new bounded plan that identifies the failed guarantee, its real truth
owner, the narrowest root-cause repair, and the evidence that must be rerun.
Phase 5 remains locked until every Phase 4 row is proved.
