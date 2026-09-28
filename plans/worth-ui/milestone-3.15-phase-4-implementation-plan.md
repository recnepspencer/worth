# WORTH UI 3.15 Phase 4 Implementation Plan

This plan is subordinate to `milestone-3.15.md`. It divides Phase 4 into
coherent proof-bearing batches without changing the milestone destination.

## Boundary Brief

The semantic inputs are the committed `UiCommittedMotionTrack`, the exact
already-presented mounted-frame identity and surface binding, and admitted
monotonic `Tick` evidence. Motion sampling may derive presentation geometry,
opacity, visibility, and damage. It may not publish semantic facts, mint a
mounted frame, retarget or terminate the semantic track, or commit hit-test
truth before physical presentation settlement.

`UiMountedPresentationCoordinator` remains the ordinary issuer and settlement
owner for host presentation work. The existing host presentation protocol is
extended with a distinct same-frame sample work kind; semantic `Delta` work
continues to require a real predecessor and successor. Native and headless
hosts preserve semantic command storage and apply derived, discardable sample
overrides keyed by exact retained command identities.

A sample is transactional. The sampler prepares a bounded candidate. A
before-effects rejection discards it, a presented completion commits it, an
in-flight completion retains it under the existing completion-token protocol,
and effects-indeterminate settlement makes presentation truth explicitly
unavailable. Hit testing and terminal Motion settlement consume only committed
samples.

The ordinary cost is proportional to active motion targets, commands already
indexed under those targets, and emitted damage regions. An inactive sampler
does not allocate sample work, request readiness, or enter a host boundary.

## Batch 4.3a — Same-frame sample authority and headless settlement

### Result

A real admitted `Tick` can prepare presentation-only motion work, present it
through the authority-sealed mounted host path, and commit the sample only
after successful settlement. The headless host proves the path without
creating a semantic frame transcript.

### Module shape

- `worth-ui-host-contract/mounted_frame/presentation_work/sample.rs` owns
  branded sample transforms, opacity, command overrides, and exact same-frame
  affinity.
- Runtime `motion_sampling` owns staged sampler candidates and commit/discard
  transitions.
- Runtime presentation work production converts sampled targets to exact
  retained command identities using the existing per-instance index.
- Runtime presentation coordinator modules own sample admission, initial
  outcome settlement, and retained in-flight completion state.
- Headless retained presentation owns bounded derived sample overrides and a
  latest-sample certification observation; semantic transcripts remain
  unchanged.

### Ordered work and proof

1. Add the authority-sealed `Sample` work contract and runtime lease issuance.
   Contract tests prove same predecessor/successor affinity, finite branded
   values, unique exact command identities, and rejection of forged work.
2. Change the sampler from eager mutation to prepare/commit. Sampling tests
   prove rejection leaves current geometry, terminal requests, and hit-test
   truth unchanged.
3. Produce sample work from the coordinator's retained presentation state.
   Tests prove command lookup is target-indexed, no semantic command mutation
   occurs, and production cost names active targets/commands/damage.
4. Settle synchronous and in-flight sample outcomes through the existing host
   effect port and completion-token protocol. Tests prove commit, discard,
   pending, cancellation, and indeterminate behavior.
5. Consume `Sample` in the headless adapter as derived retained state. An
   integration test proves the mounted frame identity and semantic transcript
   count do not change while the presentation epoch and committed sample do.

### Verification

- Focused host-contract presentation-work tests.
- Focused runtime motion-sampling and presentation-coordinator tests.
- Focused headless presentation tests and certification Tick scenario.
- `cargo fmt --all -- --check`.
- Dirty Rust line-cap guard.
- Workspace boundary check and agent-context check.

### Out of scope

Native raster consumption, native readiness production, operating-system
reduced-motion observation, and portal exit-retention lifecycle are not claimed
by this batch. They remain required below before Phase 4 can close.

## Batch 4.3b — Native retained presentation consumption

Apply the same `Sample` work to a derived native override table. Validate every
identity and affinity before effects, update damage indexing transactionally,
transform rect/portal/text raster geometry and alpha, and preserve the existing
physical pending-completion path. Pixel, damage, portal-anchor, clipping, and
same-frame epoch tests must use the production adapter.

## Batch 4.4 — Readiness, reduced motion, and exit retention

Reserve an internal application-readiness owner only when Motion support is
installed. Reuse the native physical clock and existing level-triggered
readiness protocol to emit one admitted `Tick` per ready batch while active;
inactive Motion requests no work. Observe the system reduced-motion posture and
snap through the sampler policy without inventing a Motion host capability.

Keep a dismissed portal in `Closing` while its exit presentation is retained.
When the committed exit sample settles terminally, route one ordinary terminal
proposal/publication to `Closed`, remove retained presentation once, and release
the bounded exit-retention census. Shutdown cancels all retained exits and
leaves no readiness or sampling owner alive.

## Batch 4.5 — Authored transient-surface composition

### Result

An open portal is a complete product surface rather than an anonymous host
rectangle. Its visible title, icon, body, and action row are authored by the
product through qualified mounted mechanics. The runtime owns portal
lifecycle, placement, clipping, focus, shielding, and routing; neither the
host nor a Pulse-specific renderer invents product copy or controls.

### Contract and honesty

- Portal content is associated with one exact portal owner and has a bounded,
  typed role and portal-relative allocation.
- Portal content is absent from base-frame paint, semantics, and hit testing.
  It becomes mounted only while that exact portal is presented, translated
  into the portal's qualified bounds and raised into its presentation layer.
- Title, icon, explanatory copy, primary action, and secondary action use the
  same qualified text, paint, and hit-test paths as ordinary mounted content.
- Action labels describe real admitted behavior. A product may present Save
  only when activation commits an actual editable value; a decorative Save
  that merely dismisses the surface is forbidden. Platform Pulse therefore
  uses truthful live-action and Cancel semantics unless this batch introduces
  a real saved value.
- Escape, Cancel, the primary action, outside shielding, focus acquisition,
  focus restoration, motion retention, and terminal retirement remain one
  coherent portal lifecycle rather than parallel UI state.

### Proof

1. Runtime contract tests prove portal children are suppressed while closed,
   translated and layered while their exact owner is open, clipped within the
   surface, and removed without leaving invisible hit targets.
2. Interaction tests route primary and secondary controls through real intent
   admission and prove exact close/focus-restoration behavior.
3. The native executable-world journey proves title/icon/body/action pixels,
   qualified text mechanics, keyboard Escape, pointer Cancel, primary action,
   exit motion, and restored base hit testing through the production adapter.
4. Full-window captures at 960-by-600 and 1120-by-700 receive the milestone's
   designated visual review; passing geometry alone cannot close the batch.

## Phase 4 gate

After all batches pass focused and constitutional verification, run the three
persistent Grok 4.6 QA sessions for `qa-loop`, `qa-tests`, and
`code-quality-qa`, repair material findings, rerun them, and then use the
Sol-high subagent as the final Phase 4 gate. Phase 5 begins only after that gate
passes.
