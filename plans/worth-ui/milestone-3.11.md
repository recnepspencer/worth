# Milestone 3.11: Visual Snapshot Receipts and Hit-Test Identity Bridge

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

Status: Complete. Phases 1 through 5 closed on 2026-07-27.

Milestone 3.10.3 is complete. This milestone begins from its exact product
binary, permanent executable world, native client-area observation boundary,
typed lifecycle stream, published-world typestate, failure-artifact policy, and
residue-free teardown.

Closure preserves that same product world and adds presentation-bound visual
snapshots, separate visible and hit-test identity, mounted-to-authored traces,
managed identity overlays, bounded resource accounting, and public inspection
documentation. All nine VS rows are proved against final source.

## Placement

Milestone 3.10 closed the runtime-to-host contract around one canonical mounted
frame. Milestones 3.10.2 and 3.10.3 then proved that a real file-authored
application reaches an externally visible native window through the actual
product composition root.

Those milestones deliberately stop short of explaining a pixel. The product
can prove:

```text
source revision
  -> application generation
  -> mounted frame publication
  -> real native client pixels
```

It cannot yet prove:

```text
particular observed client pixel or region
  -> exact presented frame and surface binding
  -> visible mounted contributor set
  -> independently resolved hit-test target
  -> mounted receipt
  -> mounted instance and incarnation
  -> graph node
  -> declaration and authored provenance
  -> retained runtime evidence
```

Milestone 3.11 closes that causal join before hot rebind, Query projection,
intent, services, appearance, authored composition, human inspection, or agent
inspection depend on it. It is not a screenshot utility milestone. It is the
first runtime contract that can explain why a visible location belongs to the
UI meaning the product claims is current.

Milestone 3.12 may then relate predecessor and successor snapshots to admitted
rebind work without inventing visual identity. Milestones 3.15 and 3.16 may add
portal, focus, motion, and appearance evidence without redefining point
semantics. Milestones 3.19 through 3.22 may expose the same receipt substrate to
humans and agents rather than building a second visual tree.

## Goal

Produce one receipt-backed visual snapshot capability that:

- targets an exact current or retained mounted frame through production-minted
  identity and retention authority;
- binds host-observed client pixels, viewport geometry, surface binding, and
  presentation attempt to that frame without promoting pixels into runtime
  truth;
- indexes exact mounted visible and hit-test regions without reconstructing a
  parallel graph;
- distinguishes visible paint attribution from input hit-test resolution;
- traces every resolved mounted receipt through existing mounted, graph,
  declaration, provenance, and evidence indexes;
- exposes frame, node, region, point, and overlay workflows through the public
  Worth UI facade;
- remains explicitly requested, bounded, disposable diagnostic work with zero
  unchanged-frame cost; and
- extends the existing Platform Pulse binary and executable-world journey so a
  human and the external runner can see the identity trace reach the native
  product.

## Central Claim

A visual location is explained only when Worth UI can seal a host observation
to one exact presented mounted frame and separately answer:

1. which mounted receipts contributed admitted visible paint at that location;
2. which mounted receipt, if any, owns hit-test admission at that location; and
3. how each receipt reaches its mounted instance, mount incarnation, graph node,
   authored declaration, provenance, and runtime evidence.

The runtime owns the identity and explanation. The host owns mechanical capture
and geometry observation. Pixels corroborate presentation but never create or
recover semantic authority. A loose image, current-frame lookup performed after
capture, allocation-box guess, sole-node fallback, diagnostic integer, or
renderer-local overlay cannot satisfy this claim.

## Governing Summaries

- `MENTALITY.md` requires the plausible dishonest implementation to be designed
  out before the convenient screenshot API is built.
- `arch_laws.md` requires distinct semantic identities, owner-minted authority,
  compiler-visible capture progression, typed outcome topology, bounded
  asynchronous work, versioned boundary artifacts, explicit disclosure, and
  structurally separate authoritative and derived state.
- `composition_laws.md` requires capture admission, host observation, region
  projection, spatial indexing, paint adjudication, hit-test adjudication,
  identity tracing, overlay lifecycle, retention, and receipt projection to
  remain named responsibilities rather than one visual-inspection file.
- `domain_structure_laws.md` requires runtime truth, host observation,
  inspection projection, adapter mechanics, executable-world observation, and
  certification adjudication to remain physically directional.
- `perf_laws.md` forbids per-frame screenshot work, full node scans per point,
  unbounded pixel retention, and diagnostic cost on the ordinary presentation
  lane.
- `testing_laws.md` requires a nondegenerate product world, production-minted
  semantic handles, independent pixel and authored-target oracles, real product
  entry, mutation-sensitive controls, and no new integration target.
- `dx_laws.md` requires typed target and coordinate spaces, explicit capture
  and artifact policy, discoverable outcomes, managed overlay disposal, and
  executable public examples.
- `ai-diagnostics.md` establishes one evidence substrate shared by future human
  and agent inspection. Screenshots assist that substrate; they do not replace
  it.

## Existing Truth and Exact Gap

The existing implementation already provides the authority chain this
milestone must join:

- `UiMountedFrameIdentity`, `UiMountedPresentationAttemptIdentity`,
  `UiSemanticSurfaceIdentity`, `UiHostSurfaceIdentity`,
  `UiSurfaceBindingGeneration`, `UiMountedInstanceIdentity`,
  `UiMountIncarnation`, and frame-scoped
  `UiMountedNodeReceiptIdentity` are distinct opaque types;
- a mounted node receipt is minted only for an exact frame and mounted
  instance;
- mounted projection already carries separate paint, clip, layer, allocation,
  input, hit-test, focus, accessibility, motion, and diagnostic participation;
- runtime-owned mounted receipt state already retains the graph node and
  semantic-surface relationship that the host projection intentionally omits;
- graph inspection already maps graph nodes to declarations and authored
  provenance;
- mounted-frame inspection already distinguishes current and retained
  predecessor frames and owns bounded retention leases;
- native presentation already has typed complete, rejected-before-effects,
  in-flight, and indeterminate postures; and
- the Platform Pulse executable world already correlates one product process,
  process-bound window, native client-area capture, source revision,
  application generation, and published mounted frame.

The missing production contracts are:

- a host capture request and observation sealed to an admitted presentation
  basis;
- typed client-pixel and viewport coordinate conversion;
- exact visible-region and hit-test-region projections for a retained frame;
- a frame-scoped spatial index and bounded query cost;
- public point and region adjudication receipts;
- the mounted-receipt-to-graph/declaration/evidence trace;
- a canonical diagnostic overlay lifecycle;
- visual snapshot retention, disclosure, and pixel-artifact policy; and
- a versioned pulse observation that exposes the derived trace across the
  process boundary without exporting runtime authority.

The existing `worth-ui-inspection/src/receipt/snapshot/` production home and
`apps/platform-pulse/tests/executable_world/adjudication/identity_trace.rs`
executable-world home are commitments, not placeholders to route around.

## Adversarial Constraint

Assume the implementation is written by a careful engineer who makes the
ordinary happy path work but:

- returns the only mounted node for every point in the current pulse;
- treats a painted rectangle and the input hit target as the same fact;
- uses allocation boxes without applying clip, layer, visibility, or
  participation;
- resolves the node against the current frame when capture completes rather
  than the frame whose presentation was captured;
- stores a frame number beside independently obtained image bytes and calls
  that causal binding;
- accepts a node receipt from another frame, surface, binding generation,
  runtime world, or remount incarnation;
- lets a diagnostic `u64`, renderer widget ID, graph node, or source identifier
  stand in for a mounted receipt;
- builds a second visual tree inside the renderer or inspector;
- draws the selection rectangle directly in egui rather than publishing a
  diagnostic projection through the canonical mounted host contract;
- assumes one node has one rectangle and one region has one owner;
- ignores client origin, physical pixels, logical points, fractional scale,
  rounding, and half-open edge semantics;
- scans every mounted node for every point or allocates a full-frame
  per-pixel-identity buffer on every frame;
- retains unbounded screenshot bytes or widens diagnostic disclosure;
- reports no target, unsupported capture, stale frame, expired evidence,
  capacity denial, and indeterminate native state as the same `None`;
- publishes a plausible identity event without matching external pixels;
- observes matching pixels without a product-issued trace;
- uses the same runtime map to generate both the result and the expected test
  oracle; or
- adds an in-process screenshot target, a second pulse binary, a test-only
  capture branch, or a hidden product command channel.

The proof portfolio must make every one of those implementations red.

## Decisive Product Courtroom

### Real Entry Surface

The decisive scenario extends the existing Windows
`worth-ui-platform-pulse` executable-world journey. It launches the exact
Cargo-built binary through ordinary `main`, `eframe::run_native`, the
operating-system event loop, `PlatformPulseNativeFrame`, the public
application lifecycle, and `WorthUiHostEgui`.

It reuses:

- the checked-in `app/main.wui`;
- the isolated source sandbox;
- `PulseExecutableWorld<Published<InitialBlue>>`;
- the process-bound native window adapter;
- the existing lifecycle observation stream;
- the existing external source deltas;
- the existing failure bundle and teardown; and
- the one existing `executable_world` integration target.

No new binary, target, composition root, hidden command server, or in-process
egui capture may substitute for that entry.

### Nondegenerate Platform Pulse World

The canonical page must no longer be a world in which every point can honestly
return the same sole visible receipt.

Using the existing file-authored declaration and minimal static-paint path, the
checked-in pulse must contain:

- one background mounted visual region;
- one visibly distinct inset target region with a distinct mounted instance,
  graph node, and declaration;
- at least one background point outside the target; and
- one target point strictly inside every relevant edge so DPI rounding and
  anti-aliasing cannot accidentally decide the ordinary success case.

The target declaration has the stable family-qualified authored semantic name
`component:platform.pulse.component.identity_target`. The runner knows that
name, the fixed 160-by-96 logical page, and the two canonical logical points
from the checked-in scenario contract, not from runtime point resolution.
Product and runner independently project those points into client physical
pixels through the sealed capture transform. The runner does not copy or forge
the declaration, graph, mounted, frame, or snapshot identities.

The native client and screenshot are physical-pixel artifacts and therefore
scale with the real monitor DPI. The pulse admits at most a declared 4x scale
and 983,040 retained pixel bytes. A DPI change between pin and readback is a
typed capture failure; it is not retried or relabeled as the original frame.

The target point is constructed so its visible contributor and hit-test target
deliberately agree. Separate integration worlds prove cases where they diverge.
The pulse remains a small product page; it is not required to encode every
geometry adversary in its source.

If the current declaration-to-static-paint path cannot express these two
causally distinct regions, completing that narrow authored-to-mounted path is
inside this milestone. It may not introduce Milestone 3.17 expression meaning,
Milestone 3.18 module composition, or Milestone 3.16 appearance roles.

### Hostile Sequence

One cumulative ordinary-gate process performs this sequence:

1. Launch the exact binary against an isolated copy of the canonical source.
2. Await the exact first published mounted frame and process-bound native
   window.
3. Hold liveness and externally capture the physical client area corresponding
   to the admitted 160-by-96 logical page at the real monitor scale.
4. Let the ordinary pulse composition issue one explicit, one-shot visual
   snapshot request for that exact published frame. This is permanent product
   behavior, not a test feature or steady-frame action.
5. Require a product-issued snapshot observation carrying the snapshot,
   presentation, frame, surface, binding, viewport, and retention postures.
6. Project and adjudicate the canonical logical target as one exact
   client-physical pixel.
7. Require an exact visible-contributor result and an exact hit-test result.
   Both resolve to the target mounted receipt in this world, but remain
   separately represented.
8. Trace that receipt to mounted instance, mount incarnation, graph node,
   target declaration, authored provenance, and evidence.
9. Adjudicate the canonical background point and prove it does not resolve to
   the target receipt.
10. Publish a minimal target overlay derived from the target trace through the
    canonical diagnostic projection and mounted presentation path.
11. Require a subsequent mounted-frame publication whose overlay receipt cites
    the base snapshot, base frame, and target node receipt.
12. Externally capture the same native client area and independently prove the
    overlay became visible at the target region.
13. Clear and dispose the overlay through its managed lifecycle before
    continuing the inherited source journey.
14. Apply the existing valid blue-to-green whole-application replacement.
15. Prove the old snapshot remains explicitly historical while retained, or
    becomes explicitly expired. It must never be relabeled as the green current
    frame.
16. Apply the existing malformed source delta and prove the last admitted
    frame, point trace, and visible pixels remain predecessor truth.
17. Restore canonical bytes, observe the fresh successor, close the native
    window normally, and prove zero retained snapshot, overlay, process,
    watcher, surface, observation, or sandbox residue.

The existing journey may factor these steps into named scenario operations, but
it remains one child process and one cumulative product world.

Capture/replacement races are proved deterministically in the real host-contract
integration lane, not by timing the executable child. One schedule delays
completion after atomically copying the predecessor pixels and must return an
exact retained-predecessor receipt. Its negative twin advances the host
presentation epoch before readback and must return
`SupersededBeforeReadback` with no pixel artifact. The executable journey
proves that an already completed blue snapshot remains historical after green
replacement. No required evidence depends on winning an operating-system race.

### Independent Observations

The proof joins three evidence classes:

- **Runtime semantic evidence:** product-minted mounted, graph, declaration,
  provenance, evidence, snapshot, and overlay receipts.
- **Host observation evidence:** the capture observation, exact presentation
  basis, client bounds, scale, coordinate transform, and pixel artifact posture
  returned through the host contract.
- **External consequence evidence:** process liveness, process-bound window,
  client-area pixels before and after overlay, source writes, native close,
  exit, and cleanup.

No one class may certify itself:

- the runner's expected target declaration and canonical points come from the
  authored pulse scenario, not the runtime index;
- the product cannot claim native visibility from its own receipt alone;
- the external image cannot mint or reconstruct mounted identity;
- the host cannot supply graph or declaration identity; and
- the runtime cannot reinterpret pixels captured for another presentation.

### Mutation Sensitivity

The proof family must turn red when:

- all points return the sole or first mounted node;
- paint and hit-test results are aliased;
- clip or layer ordering is ignored;
- the target and background points return the same receipt;
- a capture is rebound to the frame current at completion;
- the presentation attempt, surface binding, viewport observation, or frame is
  removed from snapshot affinity;
- a foreign or expired node target is accepted;
- an identity trace skips the mounted receipt and joins graph or source
  directly from coordinates;
- a diagnostic integer is used to re-enter runtime authority;
- the overlay is drawn locally by egui or is visible without a matching
  successor mounted publication;
- the overlay identifies its own frame rather than citing the base snapshot;
- the product event exists without matching external pixels;
- matching pixels exist without the product event;
- the snapshot index is deleted and the test still obtains its expected result
  from the implementation under test;
- retained pixel or overlay resources survive disposal or shutdown; or
- the required executable lane is skipped or executes zero snapshot scenarios.

## Required Scenario Contracts

Every required scenario must record: production claim, plausible defect,
causally valid world and semantic handles, production entry and authority
boundary, exact action schedule, typed expected outcome, consequential state,
independent oracle, mutation control, cleanup disposition, structural cost, and
owning cost lane. Setup, action, observation, adjudication, and teardown failures
remain distinct. A scenario is incomplete if any field is absent.

The implementation plan and QA closure ledger must map every scenario ID below
to its existing test target/module, exact fixture world, and evidence command.
Scenarios may be parameterized or combined when they retain independent failure
diagnosis. They must not become separate Rust integration-test crates.

### VS-01: Real Product Pixel-to-Identity Pulse

- **Lane:** the sole Windows `executable_world` target and cumulative journey.
- **World:** the exact checked-in nondegenerate pulse installed in the isolated
  executable world, one child, one process-bound Windows client area, one
  production-issued pixel grant, and independently known target/background
  points.
- **Action:** launch, capture the first published target surface, adjudicate
  both points, trace the target, publish and externally observe the overlay,
  publish and externally observe its cleared successor, then continue valid
  replacement, malformed preservation, recovery, normal close, and teardown.
- **Required result:** target paint and hit-test results independently resolve
  to the target receipt; background does not; the trace reaches the expected
  authored target; overlay and clear cite their exact frames and base snapshot.
- **Oracle:** authored target name plus external target, background-control,
  overlay, and restored pixels. Runtime results do not generate expected
  points, source identity, or pixels.
- **Controls:** event-only, pixel-only, sole-node, direct-egui-overlay,
  wrong-target, skipped-platform, premature-exit, and residue mutations.
- **Cost:** one child/window, zero retries, each snapshot/overlay transition
  within 5 seconds, no more than 256 protocol observations or 1 MiB encoded
  observation bytes, at most 8 external native captures, and at most 30 seconds
  total ordinary journey time on the measured Windows posture.

### VS-02: Presentation-Epoch Readback Schedules

- **Lane:** existing consolidated application-contract integration target with
  the production host contract and deterministic contract adapter. The real
  egui adapter must pass the same ordinary exact-epoch contract; the
  deterministic adapter proves only the otherwise uncontrollable schedules.
- **World:** real runtime and host-contract integration with a deterministic
  contract-bound host adapter; production-minted frame, surface, binding, and
  presentation epoch.
- **Action A:** atomically copy predecessor pixels, delay completion, publish a
  successor, then deliver the old observation.
- **Result A:** exact retained-predecessor snapshot with predecessor pixels and
  epoch.
- **Action B:** publish the successor before any copy, then complete the old
  request.
- **Result B:** `SupersededBeforeReadback` and no pixel artifact.
- **Action C:** report bytes without a provable copy epoch.
- **Result C:** `CaptureAffinityIndeterminate` and unusable pixels.
- **Controls:** request-echo-only, current-at-completion, missing epoch check,
  and mislabeled old/new bytes.

### VS-03: Paint and Hit-Test Divergence

- **Lane:** existing consolidated application-contract integration target.
- **World:** production-mounted overlapping target, paint-only decoration,
  hit-only transparent target, and background, all with issued semantic handles.
- **Action:** adjudicate one named point in each semantic class.
- **Required result:** painted/no-hit, hit/no-painted-contributor,
  paint-and-hit, and neither are four distinct outcomes; exact total hit order
  selects the intended target.
- **Oracle:** authored participation and independently declared geometry, not
  the production indexes.
- **Controls:** paint-index reuse as hit index, first-candidate selection,
  missing participation, and incidental vector ordering.

### VS-04: Clip, Occlusion, Disjoint Region, and Edge Geometry

- **Lane:** consolidated real runtime/host-contract integration plus a focused
  geometry property family for generated boundary cases.
- **World:** opaque overlap, nested clip, partially offscreen node, one node
  with disjoint realized regions, and exact half-open boundary points.
- **Action:** query interior, clipped, occluded, right/bottom edge, and
  intersecting regions.
- **Required result:** opaque paint terminates contribution; clipped and
  offscreen portions do not resolve; right/bottom edges are excluded; region
  results are many-to-many and complete-postured.
- **Oracle:** independently authored canonical boxes and layer/clip contract.
- **Controls:** allocation-box substitution, clip deletion, inclusive
  right/bottom edges, one-box-per-node, and one-owner-per-region.

### VS-05: Coordinate Transform and Multi-Surface Selection

- **Lane:** consolidated application-contract integration; real egui adapter
  contract cases are required for fractional scale and nonzero origin.
- **World:** two presented surfaces plus a fractional-scale single-surface
  world with nonzero native client origin.
- **Action:** attempt unresolved frame capture, select each surface, translate
  screen/client/logical points, and submit foreign-surface coordinates.
- **Required result:** unresolved multi-surface capture returns
  `SurfaceSelectionRequired`; selected captures cannot cross; pixel-center,
  scale, origin, orientation, and rounding match the independent transform
  oracle.
- **Controls:** first-surface fallback, screen/client alias, integer cast,
  uniform-scale assumption, and foreign-surface acceptance.

### VS-06: Retention, Historical Capture, and Managed Disposal

- **Lane:** consolidated application-contract integration plus the existing
  retention state-machine proof owner.
- **World:** current snapshot, retained predecessor with pixels, retained
  predecessor without pixels, expired predecessor, pending capture, and
  published overlay.
- **Action:** request geometry and required pixels, drop parent snapshot after
  deriving an overlay target, cancel before and after host request, exhaust
  leases/bytes, clear overlay, and shut down with each managed posture.
- **Required result:** retained pixels crop honestly; missing historical pixels
  are typed; expiry and capacity are distinct; subordinate leases remain valid;
  cancellation reports whether readback occurred; clear requires its successor;
  shutdown enumerates and disposes every resource.
- **Controls:** historical repaint, unbounded retention, drop-as-clear,
  leaked late observation, parent-drop invalidation, and cleanup-as-success.

### VS-07: Authority, Disclosure, and Protocol Isolation

- **Lane:** existing two-session public compile contracts plus pulse protocol
  contract tests and consolidated runtime integration for dynamic scope.
- **World:** geometry-only, pixel, and overlay grants with different scopes and
  expiry; foreign runtime identities; protocol v1, v2, and unsupported future
  envelopes.
- **Action:** attempt every cross-capability and cross-scope use and decode each
  protocol posture.
- **Required result:** static grant substitutions do not compile; dynamic
  scope/expiry violations deny before effects; wire projections never re-enter
  runtime; v1/future visual events reject before adjudication; v2 succeeds.
- **Controls:** generic authority marker, serializable grant, raw-ID
  constructor, protocol reinterpretation, and diagnostic disclosure widening.

### VS-08: Spatial and Ordinary-Lane Cost

- **Lane:** focused deterministic index property/cost tests inside the owning
  crate and unchanged-frame counters in consolidated application contracts.
- **World:** deterministic generated region sets at 1, 1,024, and 65,536
  records, including sparse, nested, and maximum-overlap distributions.
- **Action:** build indexes; run empty, small-result, maximum-overlap, truncated,
  and candidate-budget-exhausted point/region queries; run unchanged frames
  without capture.
- **Required result:** exact structural counters satisfy the declared build,
  memory, query, and examination bounds; exhaustion is incomplete rather than
  falsely exact; unchanged frames report zero snapshot work.
- **Oracle:** generator-owned geometry and structural counter equations, not
  elapsed time or production query results.
- **Controls:** full scan, per-pixel identity buffer, unbounded candidate walk,
  omitted retained memory, and diagnostic work on unchanged frames.

### VS-09: Public Compile-Time Contract

- **Lane:** existing two-session compile-contract owner; no new Rust test crate.
- **Valid twins:** resolved surface target, matching grant, required-pixel
  receipt access, same-scope coordinates, pending poll/cancel, publish-then-
  clear overlay, and diagnostic wire projection used only for display.
- **Invalid programs:** unresolved surface capture, wrong grant, geometry-only
  pixel access, cross-snapshot coordinate, raw identity construction,
  poll-after-completion, cancel-after-completion, clear-before-publish,
  publish-after-clear, and wire-to-runtime conversion.
- **Requirement:** consolidate cases into the fewest existing compiler
  sessions; assert the intended type/authority failure without snapshotting
  incidental compiler prose.

## Product Decision Lock

### Visual Attribution and Hit Testing Are Different Contracts

`UiVisualPointAdjudication` contains two independent outcomes:

```rust
pub struct UiVisualPointAdjudication {
    pub visible: UiVisiblePointOutcome,
    pub hit_test: UiHitTestPointOutcome,
}
```

These names and their semantic separation are public contract:

- visible attribution returns a front-to-back bounded stack of mounted receipt
  traces whose admitted, clipped paint contributes to the final pixel;
- hit testing returns the single admitted target selected by the exact
  hit-test ordering contract, `NoTarget`, or a typed inability to decide;
- a visible result may be nonempty while hit testing returns `NoTarget`;
- a hit target may exist where no visible contributor can be proven;
- unsupported or indeterminate attribution is never rendered as an empty
  result; and
- truncation is an explicit incomplete posture carrying the applicable budget.

For current static filled rectangles, visible attribution must be exact.
Future paint mechanisms may return a typed unsupported or indeterminate
posture until they can prove per-location contribution. Region containment may
not be silently promoted into exact alpha contribution.

Known opaque paint terminates the contributor stack and excludes fully occluded
paint behind it. Known composited paint remains ordered until its contribution
is exhausted or an opaque layer terminates composition. Unknown alpha, blend,
mask, shader, or raster semantics produce typed indeterminate attribution
rather than a plausible region stack. This milestone proves exact opaque static
fills; it does not invent Milestone 3.16 appearance semantics.

### One Snapshot, One Exact Presentation Basis

`UiVisualSnapshotIdentity` is owner-minted and unique per admitted capture. It
does not derive from image bytes, frame digest, time, or caller input.
The opaque type and its crate-private issuer live in the runtime visual
snapshot subsystem. The public facade exports the type for correlation but no
raw-value constructor. The lower inspection contract receives only the
non-authoritative correlation projection needed to render evidence; it cannot
mint or recover snapshot authority.

Every successful `UiVisualSnapshotReceipt` carries:

- snapshot identity;
- protocol/schema identity and version;
- active application generation;
- host session;
- mounted frame identity and current/retained relation;
- mounted presentation attempt;
- semantic surface;
- host surface;
- surface binding generation;
- client-area and viewport observation identity;
- capture target and capture time posture;
- coordinate-transform receipt;
- visible-region index identity;
- hit-test-region index identity;
- pixel-artifact posture;
- retention/disclosure posture;
- structural cost receipt; and
- any typed omission or incompleteness needed to interpret the result.

The runtime-owned `UiVisualSnapshotReceipt` owns the mounted-frame, pixel, and
index leases that keep every referenced mounted receipt and trace basis valid.
It contains one immutable `UiVisualSnapshotEvidence` projection defined by
`worth-ui-inspection`. Dropping or disposing the runtime receipt releases its
managed leases and eligible pixel storage. The lower inspection contract never
owns or reconstructs runtime lifecycle authority.

The host returns a mechanical capture observation. Only runtime inspection
authority may join that observation to mounted, graph, declaration, and
evidence truth and mint `UiVisualSnapshotReceipt`.

### Host Presentation Affinity Requires a Readback Fence

Echoing a capture request identity does not prove which pixels were copied.
Every completed surface presentation therefore advances an opaque,
host-issued `UiHostSurfacePresentationEpoch`. The completed mounted
presentation receipt binds that epoch to the exact presentation attempt,
mounted frame, host surface, and binding generation.

A host capture request targets that bound epoch. The adapter must atomically
establish one of these facts:

- pixels were copied while the requested epoch was still presented;
- the epoch advanced before any pixels were copied, producing
  `SupersededBeforeReadback`; or
- it cannot determine which epoch supplied the bytes, producing
  `CaptureAffinityIndeterminate` and no usable pixel artifact.

An asynchronous backend may complete later, but its observation must carry the
epoch at which copying occurred. Request echo, completion time, current-frame
lookup, matching dimensions, or visually plausible bytes cannot substitute for
the fence. A backend unable to establish the fence reports exact capture as
unsupported.

The host observation also reports realized client-space paint and hit-region
mechanics keyed by the opaque mounted projection row and requested epoch.
Runtime meaning still owns participation, paint order, hit-test order, and
mounted identity; the host observation owns only the clipping, quantization,
scale, and raster/readback mechanics it actually applied. The runtime validates
every observed row against the pinned projection before indexing it. Neither
runtime allocation boxes alone nor host geometry alone may claim final visible
or hit-test identity.

### Historical Capture Is Retention, Not Reconstruction

An exact retained-frame target does not authorize the host to repaint or
recapture a presentation that is no longer native-current.

- If a matching visual snapshot and pixel artifact were retained when that
  presentation was current, frame, node, and region requests may derive bounded
  child artifacts from those retained bytes while preserving the parent
  snapshot and coordinate affinity.
- If only mounted frame and geometry evidence remain, an identity-and-geometry
  request may succeed with a retained-predecessor posture.
- If pixels are required but no causally captured pixel artifact remains, the
  result is `HistoricalPixelsUnavailable`.
- The runtime may not temporarily re-present the predecessor, ask the current
  surface to recreate its bytes, use replay, or relabel current pixels as the
  retained frame.

Derived node and region crops receive their own artifact identity and cite the
parent snapshot. Cropping never changes the mounted, presentation, or snapshot
truth of the parent.

### Capture Progression Is Compiler-Visible

Capture progresses through these sealed internal phase types:

```text
UiVisualCaptureIntent
  -> UiAdmittedVisualCapture
  -> UiPinnedVisualCapture
  -> UiRequestedHostVisualCapture
  -> UiObservedHostVisualCapture
  -> UiIndexedVisualCapture
  -> UiVisualSnapshotReceipt
```

Each phase consumes its predecessor. The host accepts only the requested host
capture basis and returns an observation bearing that request identity. Index
construction accepts only the pinned frame plus matching host observation.
Receipt projection accepts only an indexed capture.

Skipped admission, observation without request, receipt construction by the
host, reuse after consumption, mismatched frame or binding, and seal without
retention must be unavailable through visibility and types.

The executable world extends its existing generic published stage without
replacing the world:

```text
PulseExecutableWorld<Published<InitialBlue>>
  -> PulseExecutableWorld<Published<SnapshotCaptured<InitialBlue>>>
  -> PulseExecutableWorld<Published<IdentityTraced<InitialBlue>>>
  -> PulseExecutableWorld<Published<OverlayPublished<InitialBlue>>>
  -> PulseExecutableWorld<Published<OverlayCleared<InitialBlue>>>
  -> existing green-replacement progression
```

Snapshot assertion before snapshot observation, trace assertion before trace
observation, overlay pixel assertion before overlay publication, replacement
while the overlay remains owned, and reuse of a consumed world do not compile.

### Compile-Time Enforcement Contract

Compile-time enforcement is required where validity depends only on API
construction, capability, phase, ownership, or coordinate provenance. Runtime
facts such as the number of presented surfaces, host capability, epoch change,
budget availability, native completion, and disclosure policy evaluation remain
typed runtime outcomes. The implementation must not create ceremonial
typestates for facts the compiler cannot know.

The public and host-facing type topology is fixed:

```rust
UiVisualSnapshotRequest<Target, ArtifactPolicy>
UiVisualSnapshotReceipt<ArtifactPosture>

UiVisualGeometryGrant
UiVisualPixelCaptureGrant
UiVisualOverlayGrant

UiPendingVisualCapture<Target, ArtifactPolicy>
UiVisualCapturePoll<Target, ArtifactPolicy: UiVisualArtifactPolicy> {
    Pending(UiPendingVisualCapture<Target, ArtifactPolicy>),
    Completed(
        UiVisualSnapshotOutcome<
            <ArtifactPolicy as UiVisualArtifactPolicy>::CapturedPosture
        >
    ),
}

UiPendingVisualOverlay
UiPublishedVisualOverlay
UiClearedVisualOverlayReceipt
```

`Target`, `ArtifactPolicy`, and `ArtifactPosture` are sealed marker families.
They are not caller-implemented traits and are not generic authority bounds.
The concrete target types are:

- `UiCurrentPresentedSurfaceTarget`;
- `UiRetainedPresentedSurfaceTarget`;
- `UiMountedNodeVisualTarget`; and
- `UiClientRegionVisualTarget`.

The concrete artifact policies are `UiGeometryOnly`,
`UiPixelsOptional`, and `UiPixelsRequired`. A captured
`UiVisualSnapshotReceipt<UiPixelsRequired>` exposes
`pixel_artifact() -> &UiVisualPixelArtifact`; the required artifact is not an
`Option`. Geometry-only receipts have no pixel accessor. Optional-pixel
receipts expose the typed artifact posture.

The sealed artifact-policy family has a fixed associated captured posture:

```text
UiGeometryOnly   -> UiGeometryOnly
UiPixelsOptional -> UiPixelsOptional
UiPixelsRequired -> UiPixelsRequired
```

There is no generic grant trait. The facade exposes separate concrete entry
points:

```rust
begin_visual_geometry_snapshot(
    &UiVisualGeometryGrant,
    UiVisualSnapshotRequest<Target, UiGeometryOnly>,
) -> UiPendingVisualCapture<Target, UiGeometryOnly>

begin_visual_pixel_snapshot<P: SealedPixelArtifactPolicy>(
    &UiVisualPixelCaptureGrant,
    UiVisualSnapshotRequest<Target, P>,
) -> UiPendingVisualCapture<Target, P>
```

`SealedPixelArtifactPolicy` is implemented only for `UiPixelsOptional` and
`UiPixelsRequired`; it carries no authority. Overlay publication accepts only
`&UiVisualOverlayGrant`.

The compiler and ownership system must enforce:

- capture cannot begin from an unresolved current frame or ambiguous
  multi-surface selector;
- a node target can be created only from a live retained mounted receipt;
- a region target can be created only inside one snapshot/surface scope;
- geometry, pixel, and overlay grants cannot substitute for one another;
- a pending capture is linear: polling, cancellation, timeout finalization, or
  shutdown disposition consumes and returns the only next valid handle;
- a completed, cancelled, denied, or indeterminate capture cannot be polled;
- a host capture observation can be constructed only from a consumed
  `UiRequestedHostVisualCapture` and matching host-issued presentation epoch;
- an indexed capture can be constructed only from the pinned frame, validated
  host observation, and retained leases;
- an overlay target owns a subordinate base-snapshot lease;
- only a pending overlay can publish, only a published overlay can clear, and a
  cleared overlay cannot publish or clear again;
- wire identity projections, graph identities, declaration identities,
  coordinate integers, and pixel artifacts cannot construct governed runtime
  targets; and
- immutable inspection evidence can be cloned or serialized, but live runtime
  receipts, grants, pending handles, and overlay resources cannot.

Snapshot-bound coordinates use a generative scoped brand rather than merely
storing and rechecking a snapshot ID:

```rust
snapshot.with_coordinate_scope(|scope| {
    let point = scope.client_pixel(UiClientPhysicalPixel::new(80, 48)?)?;
    let result = scope.adjudicate_point(point)?;
    // A point created by another snapshot scope does not type-check here.
    Ok(result)
})
```

The scope brand is invariant and cannot escape the closure. This makes
cross-snapshot point and region use unrepresentable instead of relying on a
developer to remember a runtime comparison.

Internal capture phase types remain crate-private and consuming. Public compile
contracts prove only the product-valued impossibilities above. Internal phase
shape is proved by compiling production code and focused state-machine tests;
it must not multiply `trybuild` sessions.

### Enforcement Boundary Matrix

- **Unrepresentable:** grant substitution, unresolved surface capture,
  cross-snapshot coordinates, missing required pixels on a successful required-
  pixel receipt, poll-after-completion, cancel-after-completion, overlay
  clear-before-publish, overlay reuse after clear, and runtime identity
  construction from wire projections.
- **Compile-fail contract:** one consolidated invalid example for each distinct
  public impossibility above, paired with a compiling valid twin.
- **Typed runtime admission:** surface cardinality, policy scope and expiry,
  host capability/version, budget, deadline-before-effect, foreign-world
  identity, and invalid numeric geometry.
- **Typed runtime lifecycle:** pending, cancelled, timed out, superseded,
  captured, omitted, denied, indeterminate, cleared, and shutdown disposition.
- **Structural counters and tests:** spatial breadth, pixel bytes, retained
  memory, candidate examination, trace probes, overlay regions, and cleanup.
- **Dependency/topology enforcement:** host/runtime/inspection direction,
  Query exclusion, sole pulse binary/target, and prohibition of test-only
  product paths.

An implementation plan must name the enforcement level for every public
guarantee. Choosing a weaker level when an earlier honest boundary is available
is a plan defect.

### Targets Are Proof-Bearing

The public query supports distinct target types:

- current-frame plus exact presented-surface target;
- exact retained-frame plus exact presented-surface target;
- exact mounted-node target; and
- exact client-region target.

An exact frame or node target is derived from a live mounted inspection receipt
or visual snapshot receipt. Callers cannot pair a raw frame identity with an
unrelated node receipt. A point or region is constructed in the coordinate
context of one snapshot, preventing accidental cross-snapshot coordinate use.

Raw `u64`, tuple, string, egui ID, source ID, graph node, or diagnostic wire
projection is never accepted as a capture or adjudication target.

Current-frame capture is resolved and pinned before host effects. Once pinned,
the operation never changes its target because a successor becomes current.

One `UiVisualSnapshotReceipt` describes exactly one semantic-surface /
host-surface / binding-generation presentation. A multi-surface frame therefore
requires explicit surface selection. The single-surface convenience constructor
returns `SurfaceSelectionRequired` when the retained frame has zero or multiple
eligible surfaces. Node targets carry their surface from the mounted receipt;
region targets are snapshot-and-surface bound.

Frame capture covers the selected surface. Node and region capture retain the
same full-surface identity/region indexes but may retain only bounded pixel
segments. Every segment carries its parent snapshot, client-space origin, and
extent. A point outside the captured pixel extent returns
`OutsideCapturedPixelExtent`; absence of pixels outside that extent does not
mean absence of mounted identity.

### Coordinate and Edge Semantics

The public contract has distinct newtypes for:

- native screen physical pixels;
- native client-area physical pixels;
- viewport logical points;
- host-surface logical points; and
- snapshot-bound point and region values.

The snapshot coordinate-transform receipt records:

- native client origin relative to screen;
- client physical dimensions;
- logical viewport dimensions;
- horizontal and vertical scale;
- orientation;
- translation;
- rounding policy;
- pixel-center policy; and
- source and destination coordinate spaces.

Rectangles use half-open edges: left and top inclusive, right and bottom
exclusive. A physical pixel is adjudicated at its center. Fractional
logical-to-physical conversion uses the transform's declared rounding posture,
never adapter-local casts. Non-finite, negative, overflowing, inverted, or
unrepresentable geometry returns a typed denial before index insertion.

The Platform Pulse uses client-area physical pixels as its external oracle.
Any roadmap or user-facing reference to a “screen point” enters through the
typed screen-to-client transform; screen and client coordinates are never
aliases.

### Visible Regions Are Not Allocation Boxes

A visible-region record includes:

- frame, surface, and binding affinity;
- mounted node receipt;
- coordinate space;
- one canonical clipped region;
- layer and paint order;
- visible-participation posture;
- clip lineage or explicit omission;
- paint-mechanism attribution posture; and
- source projection identity sufficient to detect stale reuse.

One mounted receipt may own zero, one, or many visible-region records. One
queried region may intersect zero, one, or many mounted receipts.

The hit-test index is independently derived from admitted hit-test
participation, host-realized hit geometry, clipping, an explicit
`UiMountedHitTestOrder`, and surface affinity. It is not a filtered alias of the
paint index. The mounted planner must emit a total hit-test order per surface
before presentation; duplicate or missing order is denied rather than resolved
by vector order, paint order, graph order, receipt integer, or adapter
incidental order. Later portal and control work may add order inputs but may not
replace this total-order contract. Milestone 3.11 uses hit-test participation;
it does not invent later control enabledness.

Milestone 3.11 does not invent semantic focus, portal policy, intent dispatch,
or appearance. It establishes additive fields and index families into which
those later decisions can project.

### Region Adjudication Is Bounded and Complete-Postured

Region queries return a bounded ordered set of intersections, not one guessed
owner. Each entry relates a mounted receipt trace to an exact intersected
snapshot region.

The result declares one of:

- complete;
- empty and complete;
- truncated by caller or session budget.

Unsupported paint mechanics, stale or expired capture authority, unavailable
coordinate transforms, and indeterminate native presentation affinity prevent
the runtime from sealing an exact live receipt. They remain typed capture
outcomes and are not duplicated as unreachable query branches. Disposal
consumes the receipt, so an expired receipt cannot be queried.

Coverage percentages or alpha-weighted ownership are not claimed in this
milestone. Later visual evaluation may add them as derived evidence without
changing region identity or receipt authority.

### Pixel Artifacts Are Derived, Governed, and Optional

The request chooses a typed artifact policy:

- identity and geometry only;
- pixels requested but omission permitted; or
- pixels required for success.

A successful pixels-required request contains a bounded pixel artifact or
returns a typed non-success outcome. A geometry-only receipt remains a visual
identity receipt but may not be described as a screenshot.

Pixel bytes carry format, dimensions, stride, color-space posture, capture
source, byte count, disclosure audience, redaction posture, and retention
disposition. Redacted or transformed bytes remain explicitly derived and may
not be represented as the original native capture.

Pixel data is never authoritative application, graph, mount, hit-test, or
source truth. Destroying all snapshot indexes and pixel artifacts leaves the
current runtime truth intact; a new explicit request can rebuild eligible
derived evidence.

### Disclosure Authority Is Concrete

Application construction declares one `UiVisualInspectionPolicy`. Launch seals
that policy into session-owned `WorthUiVisualInspectionAuthority`; no caller or
host can construct it. That authority issues three distinct non-serializable
proof types:

- `UiVisualGeometryGrant`;
- `UiVisualPixelCaptureGrant`; and
- `UiVisualOverlayGrant`.

Each grant is scoped to:

- application session;
- allowed semantic or host surfaces;
- artifact policy;
- audience;
- maximum byte and retention budgets; and
- expiry or session lifetime.

Self-description, an inspection query, a host capability, a frame identity, or
an executable runner is not a grant. A geometry grant cannot call pixel or
overlay entry points; a pixel grant does not imply overlay authority. Scope,
audience, expiry, or budgets are never widened at runtime.

The Platform Pulse uses a local-development grant over synthetic checked-in
content. That posture is part of ordinary pulse launch configuration and is
visible in the snapshot observation; it is not activated by `cfg(test)`.

### Outcomes Are Typed

The top-level contract is:

```rust
pub enum UiVisualSnapshotOutcome {
    Captured(UiVisualSnapshotReceipt),
    Superseded(UiVisualSnapshotSuperseded),
    Omitted(UiVisualSnapshotOmission),
    Denied(UiVisualSnapshotDenial),
    Indeterminate(UiVisualSnapshotIndeterminate),
}
```

`Omission` owns no current frame, transition in flight, unknown or expired
frame, node not presented, node not visible, historical pixels unavailable,
host capability unsupported, and pixels omitted by admitted policy. `Denial`
owns foreign world/session/surface/binding/node, surface selection required,
outside captured extent, invalid geometry/transform, disclosure, deadline
already elapsed, protocol incompatibility, and every pre-effect capacity or
accounting rejection. `Superseded` owns a proven epoch change and states whether
an exact predecessor artifact was already copied. `Indeterminate` owns timeout
after host request, capture-affinity uncertainty, native-presentation
uncertainty, host completion uncertainty, and cleanup uncertainty. Internal
invariant defects remain defects and are not rendered as ordinary denial.

Typed current, retained-predecessor, stale, superseded, omitted, denied, and
indeterminate postures must not collapse into `Option`, boolean, generic error,
empty bytes, or unchanged pixels.

### Overlay Is a Managed Successor Presentation

The 3.11 overlay is one fixed minimal identity-target diagnostic appearance. It
does not introduce theme roles, component states, animation, or general
appearance authoring.

An overlay request consumes:

- a live visual-inspection grant;
- a retained base snapshot;
- an exact selected mounted receipt trace; and
- a bounded overlay artifact policy.

The runtime derives a diagnostic projection referencing the base snapshot,
base frame, and target receipt. That projection enters ordinary mounted frame
assembly and host presentation. A successfully visible overlay therefore has:

- a distinct successor mounted frame and presentation attempt;
- its own overlay receipt;
- an explicit relation back to the base snapshot and target;
- a managed clear/dispose operation; and
- a typed superseded or expired outcome if the target cannot still be
  represented honestly.

The adapter may mechanically translate the diagnostic projection. It may not
draw an unreceipted egui rectangle, mutate the captured image, or claim the
overlay was present in the base frame.

The overlay is paint-only: it has no hit-test, focus, accessibility, intent, or
service participation and therefore cannot replace the selected node as the
hit target. Its fixed 3.11 appearance is an opaque two-physical-pixel magenta
border whose client-space rounding is recorded by the host observation; it
does not fill or obscure the target interior. The executable oracle proves the
border appears only around the target, a background control region remains
unchanged, and the cleared successor restores the pre-overlay pixels.

`overlay_target` creates a subordinate base-snapshot lease, so dropping the
caller’s snapshot cannot invalidate pending overlay work. Publishing produces a
managed `UiPublishedVisualOverlay`. Clearing stages removal and is complete only
after a distinct cleared successor frame is presented; the clear receipt cites
the overlay and cleared frame. Dropping a pending pre-effect overlay cancels it.
A published overlay survives until explicit clear or application shutdown, and
shutdown reports its typed disposition. Disposal never silently claims native
removal.

### The Pulse Observation Is Derived and Versioned

The pulse observation protocol advances from schema version 1 to schema version
2 and gains typed snapshot, point-trace, overlay-published, and overlay-cleared
observations. Incompatible versions are rejected before adjudication.

The stdout observation stream carries bounded receipt metadata and identity
trace projections only. It never serializes screenshot bytes. Native captures
remain external evidence artifacts under the existing bounded failure-artifact
policy, and product-owned pixel payloads remain governed by the visual snapshot
retention policy. The existing 1 MiB maximum encoded observation size remains
unchanged.

Wire projections may contain diagnostic representations of identities for
cross-process correlation. Those representations:

- are typed inside the pulse observation contract;
- cannot construct public runtime identity types;
- carry the enclosing run, sequence, protocol, frame, and snapshot relation;
- are never accepted back by the product; and
- do not replace the public receipt from which they were derived.

The executable runner imports only the pulse observation facade, not runtime,
inspection internals, DSL, host internals, or certification support.

## Public Contract and DX

Capture is an event-loop-spanning managed operation. Even an adapter that can
complete immediately uses the same handle contract:

```rust
use worth_ui::facade::inspection::{
    UiClientPhysicalPixel, UiVisualArtifactPolicy, UiVisualSnapshotRequest,
};

let frame = shell
    .inspect_mounted_frame(Default::default())
    .into_available()?;
let target = frame.current_visual_target()?;

let mut capture = shell
    .begin_visual_pixel_snapshot(
        &visual_pixel_capture_grant,
        UiVisualSnapshotRequest::for_frame(target)
            .artifacts(UiVisualArtifactPolicy::pixels_required())
            .deadline(capture_deadline)
            .cancellation(cancellation),
    )?;

let snapshot = loop {
    shell.present_frame(presentation_deadline, now_tick)?;
    match shell.poll_visual_snapshot(capture, now_tick)? {
        UiVisualCapturePoll::Pending(next) => capture = next,
        UiVisualCapturePoll::Completed(outcome) => break outcome.into_captured()?,
    }
};

let point = snapshot.client_pixel(UiClientPhysicalPixel::new(80, 48)?)?;
let adjudication = snapshot.adjudicate_point(point)?;

let visible = adjudication.visible().frontmost_required()?;
let hit_target = adjudication.hit_test().target_required()?;
let trace = hit_target.identity_trace();

assert_eq!(visible.node_receipt(), trace.node_receipt());
trace.graph_node();
trace.declaration();
trace.authored_provenance();
trace.evidence();

let overlay = shell.show_identity_overlay(
    &visual_overlay_grant,
    snapshot.overlay_target(hit_target)?,
)?;
let published = shell.present_visual_overlay(overlay, presentation_deadline)?;
shell.clear_visual_overlay(published)?;
```

`WorthUiActiveApplicationSession` owns mounted-frame inspection, capture
registration, polling, cancellation, and disposal.
`WorthUiNativeApplicationShell` delegates `inspect_mounted_frame`,
`begin_visual_geometry_snapshot`, `begin_visual_pixel_snapshot`,
`poll_visual_snapshot`, `cancel_visual_snapshot`,
`show_identity_overlay`,
`present_visual_overlay`, and `clear_visual_overlay` as the stable downstream
native-product facade. The caller-visible guarantees are:

- the target is derived from retained production authority;
- pixel policy and deadline are explicit;
- coordinates are typed and snapshot-bound;
- paint and hit-test results are distinct;
- identity trace is discoverable from the adjudication result;
- overlay publication is a managed lifecycle;
- disposal is explicit; and
- no caller knows runtime module paths, registry keys, host adapter types, wire
  identifiers, or internal index representation.

Every capture handle is registered with the session, enumerable at shutdown,
and bounded by the one-in-flight-per-surface policy. Cancellation before host
request is a no-effect cancellation. Cancellation after request does not
rollback host work: late observations are drained and discarded, leases are
released, and the final cancellation receipt reports whether readback occurred.
Timeout follows the same before-effect versus after-effect distinction.

Public examples compile in the existing documentation/example verification
lane. Compile-fail evidence is limited to valuable public impossibilities:
cross-snapshot points, raw identity construction, and overlay publication
without the required grant or retained target. It is consolidated into the
existing compile-contract sessions.

## Cost and Resource Contracts

### Named Lanes

- **Ordinary presentation lane:** no capture request means zero pixel readback,
  zero snapshot-index construction, zero overlay derivation, zero pulse visual
  observation encoding, and zero visual-snapshot retention allocation.
- **Explicit snapshot lane:** frame pinning, host capture, pixel transfer,
  region projection, index construction, identity trace projection, and
  receipt sealing.
- **Explicit query lane:** point or region lookup against an already sealed
  immutable snapshot index.
- **Overlay lane:** bounded diagnostic projection, successor presentation, and
  managed disposal.
- **Executable-world lane:** external native capture, cross-process
  correlation, artifacts, and teardown.

No diagnostic work may be laundered into ordinary frame execution,
replacement, or host repaint.

### Default Bounds

The default production policy supports:

- at most 8 simultaneously retained visual snapshot receipts per application
  session;
- at most 64 MiB of pixel payload per capture;
- at most 256 MiB of retained snapshot pixel payload per application session;
- at most 64 MiB of retained snapshot index/trace structure per receipt and
  256 MiB per application session;
- at most 65,536 visible-region records and 65,536 hit-test-region records per
  captured frame;
- at most 32 returned contributors or region intersections per ordinary query
  unless the caller explicitly admits a narrower or larger bounded policy; and
- at most 4,096 candidate regions examined per ordinary query before a typed
  incomplete result; and
- one in-flight host capture and one active identity overlay per host surface
  in the default policy.

Application configuration may lower these bounds. Raising them requires an
explicit typed policy and remains subject to host capability and allocation
admission. Exhaustion rejects before additional capture or overlay effects
where possible; partial host effects return an indeterminate posture and
recovery/disposal handle.

### Complexity Contracts

Let:

- `R` be the number of admitted visible and hit-test region records for the
  targeted surface;
- `P` be the captured pixel byte count;
- `K` be the number of matching candidates returned by a query; and
- `T` be the number of trace edges materialized for returned mounted receipts.

Required bounds:

- frame pin and exact-frame lookup: bounded indexed lookup, never a retained-
  frame scan;
- region projection: `O(R)`;
- immutable two-dimensional orthogonal range-index construction:
  `O(R log R)` time and `O(R log R)` retained structural entries;
- pixel capture and transfer: `O(P)` time and at most one retained owned pixel
  payload plus adapter-declared transfer amplification;
- point query: worst-case `O(log^2 R + K + T)`;
- region query: worst-case `O(log^2 R + K + T)` excluding the caller-admitted
  result width;
- identity tracing: indexed by mounted receipt and graph/declaration identity,
  never a full graph, declaration, evidence, or mounted-frame scan; and
- overlay derivation: proportional to the selected target's admitted region
  count, not total mounted nodes or captured pixels.

Structural cost receipts expose at least region records examined, spatial-index
probes, candidates considered, trace-index probes, pixel bytes requested,
pixel bytes retained, coordinate transforms, overlay regions emitted, lease
count, and retained structural bytes.

Elapsed-time assertions alone do not certify these bounds.

If the candidate-examination budget is exhausted before visible composition or
hit-test order can be decided, the query returns typed incomplete evidence; it
does not truncate candidates and then claim an exact target.

## Destination Topology

Files marked `committed successor` describe additive insertion homes and are
not created empty.

```text
workspaces/worth-ui/
  crates/worth-ui-host-contract/src/
    visual_snapshot/
      mod.rs
        [create: curated host capture contract facade]
      capability.rs
        [create: capture and coordinate capability posture]
      capture_request.rs
        [create: presentation-bound mechanical request]
      capture_observation.rs
        [create: typed host result and affinity]
      presentation_epoch.rs
        [create: host-issued surface readback fence]
      realized_region.rs
        [create: epoch-bound mechanical paint/hit geometry observation]
      coordinate_transform.rs
        [create: client, viewport, scale, and rounding receipt]
      pixel_artifact.rs
        [create: bounded mechanical pixel payload contract]
    operational_adapter.rs
      [modify: admit the visual capture operation without inspection meaning]
    lib.rs
      [modify: curated visual-capture exports]

  crates/worth-ui-inspection/src/
    query/
      visual_snapshot/
        mod.rs
          [create: curated request exports]
        request.rs
          [create: target, deadline, and artifact policy]
        disclosure.rs
          [create: policy, grant scope, capability, and audience meaning]
      mod.rs
        [modify: export visual snapshot request family]
    receipt/
      snapshot/
        mod.rs
          [replace existing committed home with immutable evidence facade]
        capture_basis.rs
          [create: presentation, surface, viewport, and retention affinity]
        geometry.rs
          [create: snapshot-bound point and region values]
        pixel_artifact.rs
          [create: governed pixel artifact projection]
        visible_region.rs
          [create: visible contributor records and stacks]
        hit_test.rs
          [create: hit-test region and point outcomes]
        point_adjudication.rs
          [create: distinct visible and hit-test result]
        region_adjudication.rs
          [create: bounded many-to-many region result]
        identity_trace.rs
          [create: mounted-to-graph/declaration/evidence projection]
        overlay.rs
          [create: overlay target, receipt, and lifecycle outcomes]
        outcome.rs
          [create: capture, omission, denial, and indeterminate topology]
        cost.rs
          [create: visual inspection structural cost receipt]
        comparison/
          [committed successor: identity-aware snapshot comparison for 3.12
           and later visual inspection]
      mod.rs
        [modify: export snapshot receipt family]
    lib.rs
      [modify: curated public inspection exports]

  crates/worth-ui-runtime/src/
    inspection/
      visual_snapshot/
        mod.rs
          [create: visual snapshot orchestration facade]
        identity.rs
          [create: owner-minted snapshot and spatial-index identities]
        admission.rs
          [create: grant, target, capability, and budget admission]
        grant.rs
          [create: session authority and concrete geometry/pixel/overlay grants]
        capture_progression.rs
          [create: compiler-visible host capture phases]
        capture_handle.rs
          [create: registered poll/cancel/dispose lifecycle]
        frame_basis.rs
          [create: retained mounted and presentation affinity]
        region_projection.rs
          [create: clipped visible and hit-test region derivation]
        spatial_index.rs
          [create: immutable 2D orthogonal frame-scoped lookup indexes]
        hit_test_order.rs
          [create: explicit total per-surface target order]
        point_adjudication.rs
          [create: indexed point resolution]
        region_adjudication.rs
          [create: indexed region intersection]
        identity_trace.rs
          [create: existing-index trace assembly]
        overlay_plan.rs
          [create: base-snapshot-derived diagnostic projection]
        retention.rs
          [create: snapshot, pixel, and overlay managed lifecycle]
        receipt.rs
          [create: public runtime-owned receipt and live leases]
        receipt_projection.rs
          [create: immutable inspection evidence projection]
        cost.rs
          [create: structural accounting]
      mounted_frame/
        [modify: derive proof-bearing visual targets from retained inspection]
      mod.rs
        [modify: expose runtime-internal visual snapshot facade]
    mounting/
      retention/
        [modify: add bounded snapshot and overlay lease classes]
    facade/
      entry/
        visual_snapshot.rs
          [create: active-session and native-shell capture workflow]
        visual_overlay.rs
          [create: managed overlay publication workflow]
        mod.rs
          [modify: facade membership]
      inspection_bridge/
        visual_snapshot.rs
          [create: public inspection projection boundary]
        mod.rs
          [modify: curated exports]

  crates/worth-ui-host-egui/src/
    adapter/
      visual_snapshot/
        mod.rs
          [create: egui capture adapter facade]
        capture.rs
          [create: presentation-bound request lifecycle]
        coordinate_observation.rs
          [create: client/viewport transform observation]
        presentation_epoch.rs
          [create: egui surface epoch and readback fence]
        pixel_readback.rs
          [create: bounded screenshot event translation]
      egui_host.rs
        [modify: implement capture contract]
      mod.rs
        [modify: private adapter membership]
    lib.rs
      [modify: no inspection meaning exported]

  crates/worth-ui/src/
    facade/
      inspection.rs
        [modify: curate public visual snapshot contract]
    tests/
      ui/visual_snapshot/
        pass/
          governed_visual_snapshot_lifecycle.rs
            [create: consolidated valid public twin]
        fail/
          [create: fewest files needed for VS-09 distinct impossibilities]
      suites/
        compile_contract_cases.csv
          [modify: register VS-09 cases]
        compile_contract_execution.csv
          [modify only if existing session membership requires it]

  apps/platform-pulse/
    app/main.wui
      [modify: nondegenerate background and inset target world]
    src/
      observation_contract/
        visual_snapshot.rs
          [create: derived versioned snapshot/trace/overlay observations]
        mod.rs
          [modify: curated wire exports]
      visual_identity_pulse.rs
        [create: one-shot ordinary product snapshot and overlay lifecycle]
      native_frame.rs
        [modify: orchestrate pulse lifecycle without absorbing snapshot logic]
      lifecycle_observation_publication.rs
        [modify: publish derived visual events]
    tests/executable_world/
      courtroom/
        platform_pulse_journey.rs
          [modify: extend the same cumulative child-process journey]
        visual_identity_pulse.rs
          [create: named courtroom step orchestration, not a new test target]
      product_process/
        visual_snapshot_progression.rs
          [create: published -> snapshot -> overlay typestate transitions]
        progression.rs
          [modify: curated cumulative progression]
      external_observation/
        lifecycle_stream.rs
          [modify: decode the advanced protocol]
        native_client_area.rs
          [modify: expose point/region pixel observations]
      adjudication/
        identity_trace.rs
          [populate committed home: join product trace and external pixels]
        mod.rs
          [modify: curated adjudication exports]
      failure_teardown/
        retained_artifact.rs
          [modify: bounded snapshot and overlay failure evidence]

  crates/worth-ui-certification/tests/
    suites/application_contracts/
      visual_snapshot.rs
        [create: consolidated production-boundary integration scenarios]
    suites/application_contracts.rs
      [modify: register module without new integration target]

  docs/application-lifecycle.md
    [modify during implementation: human Platform Pulse snapshot workflow]
  docs/visual-inspection.md
    [create during implementation: continuing developer contract]
```

### Structural Axes and Owners

- `worth-ui-host-contract/visual_snapshot/` owns host-neutral capture mechanics,
  coordinate observations, and pixel payload posture. It excludes graph,
  declaration, evidence, inspection grant, and semantic adjudication.
- `worth-ui-inspection/query/visual_snapshot/` owns caller intent, disclosure,
  artifact policy, and budget. It excludes runtime execution and adapter types.
- `worth-ui-inspection/receipt/snapshot/` owns immutable derived visual evidence
  types and outcome meaning. It cannot own runtime leases, construct authority,
  or call the host.
- `worth-ui-runtime/inspection/visual_snapshot/` owns the only join between
  retained mounted truth, host observations, spatial indexes, identity traces,
  and inspection evidence. Its public `UiVisualSnapshotReceipt` owns the live
  retention and disposal lifecycle.
- `worth-ui-host-egui/adapter/visual_snapshot/` owns egui-specific request and
  readback mechanics. It cannot own semantic region or overlay meaning.
- `facade/entry/visual_snapshot.rs` and `visual_overlay.rs` own public workflow
  orchestration and nothing else.
- `platform-pulse/src/visual_identity_pulse.rs` owns permanent one-shot product
  exercise. It is ordinary product composition, not a general inspector or
  test protocol.
- executable-world `external_observation/` owns facts outside the product;
  `adjudication/identity_trace.rs` owns the derived certification verdict. It
  never creates production receipts.

The tree forbids `visual_helpers`, a generic geometry bag, a universal
inspection manager, snapshot behavior inside `native_frame.rs`, graph tracing
inside the host adapter, executable runner mechanics inside production crates,
or adapter drawing hidden behind the overlay facade.

The committed `comparison/` child is justified by Milestone 3.12 predecessor/
successor identity and later human/agent visual evaluation. It is not populated
until a milestone owns comparison semantics.

## Dependency and Visibility Enforcement

Mechanical enforcement must prove:

- only runtime inspection authority can construct visual snapshot, point,
  region, identity-trace, and overlay receipts;
- the host contract and egui adapter cannot depend on `worth-ui-runtime`,
  `worth-ui-inspection` implementation modules, graph internals, DSL, Query, or
  pulse code;
- `worth-ui-inspection` remains a meaning/evidence contract and cannot depend
  on the runtime or egui adapter;
- the executable runner cannot depend on runtime, DSL, certification support,
  test support, host internals, or egui capture APIs;
- the pulse product source contains no `cfg(test)` capture path, runner-only
  command, copied runtime authority, or direct overlay paint;
- no public constructor accepts raw identity representations for governed
  targets;
- Query crates remain absent from the snapshot, host, and pulse dependency
  closure;
- no second pulse binary, canonical source, executable target, or screenshot
  harness appears; and
- all code and test files remain at or below the 400-line cap without a new
  exemption.

Boundary-check, agent-context, dependency audits, public compile contracts, and
source topology checks carry these rules.

## Phase 1: Contract, Courtroom, and Topology Freeze

### What Becomes True

The milestone has one agreed truth model, one nondegenerate canonical page, one
coordinate doctrine, one outcome topology, one resource policy, one decisive
courtroom, and one additive destination tree.

### Required Work

- extend the canonical pulse source to the background-plus-target world;
- define authored semantic handles and independent target/background points;
- add public query, target, identity, coordinate, artifact-policy, disclosure,
  outcome, cost, and receipt contracts;
- add internal capture typestate contracts;
- establish host, runtime, inspection, adapter, facade, pulse, and executable-
  world module homes;
- register boundary and dependency denials before implementation fills the
  governed paths; and
- freeze ordinary, snapshot, query, overlay, and executable-world cost lanes.

### Mechanical Prohibitions

- sole-node pulse worlds cannot satisfy closure;
- raw IDs cannot construct capture or overlay targets;
- paint and hit-test outcomes cannot share one public result type;
- screen, client, viewport, and host-surface coordinates cannot alias;
- later implementation cannot add another target or product command channel.

### Exit Gate

Compile-pass and compile-fail contracts prove the public target and grant
boundary. Topology and dependency checks recognize the new homes. The canonical
world provides distinct production-minted target and background receipts
through the existing application lifecycle before snapshot code is trusted.
VS-09's valid/invalid public contract skeleton and VS-01's causally valid pulse
world must be established; later runtime outcomes are not required to be green
before their owning phases.

## Phase 2: Presentation-Bound Host Capture and Retention

### What Becomes True

An explicitly admitted visual request can pin one exact mounted presentation,
cross the host capture boundary, return typed pixels and coordinate observation,
and release every resource without yet claiming semantic point resolution.

### Required Work

- implement application-declared policy, session-owned grant authority, and
  concrete disclosure admission;
- extend mounted retention with snapshot and overlay lease classes;
- implement compiler-visible capture progression and the registered
  poll/cancel/dispose handle lifecycle;
- bind completed presentation to a host surface epoch and enforce the readback
  fence;
- implement host-neutral capture request, observation, transform, capability,
  and pixel-artifact contracts;
- implement the egui adapter using real capture/readback behavior;
- reject mismatched request, presentation, frame, surface, binding, and
  viewport affinity;
- implement required/optional/omitted pixel postures;
- implement deadline, cancellation-safe point, capacity, indeterminate, and
  disposal outcomes; and
- expose structural byte, transfer, lease, and transform counters.

### Mechanical Prohibitions

- the host cannot mint a visual snapshot receipt;
- image bytes cannot select or reconstruct a frame;
- completion-time current-frame lookup cannot retarget a pinned capture;
- no capture request means no adapter readback or snapshot allocation.

### Exit Gate

Real-adapter integration proves exact frame affinity, fractional coordinate
translation, required-pixel success, unsupported/indeterminate outcomes,
replacement-during-capture posture, bounded retention, and disposal. Destroying
all capture artifacts leaves runtime truth and mounted publication intact.
VS-02, the capture/transform portions of VS-05, the capture-lifecycle portions
of VS-06, and the runtime authority/protocol portions of VS-07 are required.

## Phase 3: Spatial Identity and Explanation Closure

### What Becomes True

A sealed host capture plus retained mounted frame produces immutable visible and
hit-test indexes, point and region adjudication, and exact mounted-to-authored
identity traces without a parallel tree.

### Required Work

- derive clipped, layered visible-region records from canonical mounted paint
  plus validated host-realized geometry;
- derive hit-test-region records independently from hit-test participation,
  host-realized geometry, and explicit total order;
- build immutable frame-scoped spatial indexes;
- implement point adjudication with separate visible and hit-test outcomes;
- implement bounded many-to-many region adjudication;
- join node receipt to mounted instance, incarnation, graph node, declaration,
  authored provenance, and evidence using existing authoritative indexes;
- seal `UiVisualSnapshotReceipt` with its live retention lease and cost receipt;
- support frame, node, and region capture targets; and
- add exact, empty, and truncated query postures while preserving unsupported,
  stale, expired, transform, and affinity failures as capture outcomes before a
  live receipt can be sealed.

### Mechanical Prohibitions

- allocation boxes cannot masquerade as visible regions;
- a paint index cannot masquerade as a hit-test index;
- point queries cannot scan all mounted nodes;
- graph/source lookup cannot skip the mounted receipt;
- host or runner diagnostic representations cannot re-enter the trace path.

### Exit Gate

Focused and consolidated integration scenarios prove overlap, clipping,
paint-only, hit-only, no-target, multi-region, repeated mount, remount,
foreign-world, expiry, truncation, and exact boundary behavior. Structural
counters prove indexed rather than global scans. Mutation controls make
sole-node fallback, paint/hit aliasing, clip omission, stale-frame reuse, and
parallel-tree reconstruction red.
VS-03, VS-04, the multi-surface portions of VS-05, and the index/query portions
of VS-08 are required.

## Phase 4: Canonical Overlay and Platform Pulse Closure

### What Becomes True

The public facade can show what a point resolved to, and the exact real product
process visibly presents that explanation through a successor mounted frame.

### Required Work

- expose snapshot, adjudication, trace, and overlay workflows through the
  curated Worth UI facade;
- implement the fixed identity-target overlay as a runtime diagnostic
  projection;
- implement overlay publication, supersession, clear, disposal, and shutdown;
- add the permanent one-shot visual identity exercise to the ordinary pulse
  product;
- advance the pulse observation protocol with snapshot, point-trace, overlay-
  published, and overlay-cleared events;
- extend `PulseExecutableWorld<Published<InitialBlue>>` through the declared
  snapshot and overlay stage types;
- populate executable-world `adjudication/identity_trace.rs`;
- externally prove target/background differentiation and overlay pixels; and
- continue the inherited valid replacement, malformed preservation, recovery,
  normal close, and cleanup sequence in the same child.

### Mechanical Prohibitions

- the overlay cannot be direct egui drawing or pixel mutation;
- the overlay cannot claim membership in its base frame;
- pulse behavior cannot be test-feature-gated;
- wire identities cannot construct runtime identities;
- product event or pixels alone cannot close the courtroom.

### Exit Gate

The cumulative Windows journey relates exact source, process, window, first
frame, snapshot, target point, distinct background point, mounted trace,
successor overlay frame, external overlay pixels, replacement, malformed
predecessor preservation, recovery, close, exit, and zero residue. Removing
either product identity evidence or external consequence evidence turns it red.
VS-01 and the overlay lifecycle portions of VS-06 are required in the inherited
targets and cumulative child process.

## Phase 5: Cost, Documentation, and Successor Handoff

### What Becomes True

The capability is bounded, documented, constitutionally enforced, and ready for
3.12 to add identity-aware rebind relationships without redesign.

### Required Work

- prove zero unchanged-frame snapshot and overlay work;
- prove the declared index, query, trace, byte, lease, capture, artifact, and
  executable-world budgets;
- prove incompatible pulse protocol versions reject before adjudication;
- audit disclosure, redaction posture, disposal, expiry, failure artifacts, and
  shutdown cleanup;
- compile and run public DX examples;
- revise `docs/application-lifecycle.md` with the human pulse workflow;
- create `docs/visual-inspection.md` for continuing application developers;
- run code-quality and test-quality review across every dirty file;
- run format, clippy, workspace tests, line-cap, boundary-check, and
  agent-context checks; and
- reconcile the roadmap with the exact closed contract and 3.12 handoff.

### Mechanical Prohibitions

- no unbounded or undocumented artifact posture remains;
- no required correctness claim depends only on a scheduled or manual lane;
- no new warning, line-cap exemption, compile target, retry, ignored test, or
  residue is accepted;
- 3.12 cannot require moving the snapshot facade or reversing authority.

### Exit Gate

All acceptance evidence passes on exact final source. The public docs compile
against the real facade. The executable journey remains within its inherited
launch, duration, capture, failure-artifact, and teardown budgets plus the
explicit VS-01 ceiling of 30 seconds and 8 native captures. A clean boundary
and agent-context result is required for closure.
Every VS-01 through VS-09 ledger row must be closed with its exact command,
fixture provenance, typed result, mutation control, structural cost, and
teardown evidence. A green phase without that ledger is not milestone closure.

## Documentation Deliverables

### Continuing Application Developer Contract

Create `workspaces/worth-ui/docs/visual-inspection.md` for application
developers and future inspector/agent authors. It must explain:

- snapshot versus screenshot;
- authoritative mounted truth versus host and pixel observations;
- frame, node, region, and point targets;
- visible contributor versus hit-test target;
- coordinate spaces and edge rules;
- current, retained, stale, expired, denied, and indeterminate outcomes;
- grants, disclosure, artifact policy, budgets, retention, and disposal;
- overlay successor-frame semantics;
- exact identity trace;
- failure recovery; and
- executable public examples.

### Continuing Human Pulse Contract

Revise `workspaces/worth-ui/docs/application-lifecycle.md` so a human can:

- run the same checked-in pulse binary;
- identify the inset target;
- see the mounted-identity overlay;
- inspect the human-readable trace;
- perform the existing valid and malformed edits;
- understand which frame the trace describes; and
- run the same executable-world command that certifies the product entry.

### Governing Diagnostic Architecture

Revise `ai-diagnostics.md` only where the implemented names, outcome topology,
or snapshot lifecycle make its conceptual example incomplete. It must continue
to describe one shared human/agent evidence substrate and must not claim
identity-aware frame comparison until its committed successor is implemented.

## Must Ship

- `UiVisualSnapshotIdentity`;
- `UiVisualSnapshotReceipt`;
- `UiVisualSnapshotEvidence`;
- concrete visual-inspection grant and disclosure posture;
- managed capture handle with polling, cancellation, timeout, and disposal;
- host surface presentation epoch and readback fence;
- typed frame, node, and region capture targets;
- presentation-bound host capture request and observation;
- governed optional or required pixel artifact;
- screen/client/viewport/host coordinate transform receipt;
- immutable visible-region and hit-test-region indexes;
- explicit total per-surface hit-test order;
- frame capture by identity;
- node capture by identity;
- region capture by identity;
- separate visible-contributor and hit-test point outcomes;
- bounded many-to-many region adjudication;
- `point -> mounted receipt trace`;
- `region -> mounted receipt traces`;
- `mounted receipt -> mounted instance/incarnation -> graph -> declaration ->
  authored provenance -> evidence`;
- fixed identity-target overlay with successor-frame receipt and disposal;
- structural snapshot cost receipt;
- bounded snapshot and overlay retention;
- versioned pulse snapshot/trace/overlay observations;
- nondegenerate Platform Pulse target and background world;
- cumulative real-process executable-world identity trace; and
- continuing developer and human documentation.

## Must Preserve

- every 3.10 mounted identity, projection, participation, presentation,
  publication, predecessor-preservation, retention, and host-authority
  guarantee;
- every 3.10.1 DSL ownership and runtime separation guarantee;
- every 3.10.2 checked-in source, product binary, static-paint, replacement, and
  human pulse guarantee;
- every 3.10.3 real-entry, process/window correlation, observation, artifact,
  platform, and teardown guarantee;
- runtime authority over visual meaning and host authority over mechanical
  capture and translation;
- pixels, indexes, overlays, and executable verdicts as derived disposable
  evidence;
- one runtime-to-host presentation path;
- one pulse binary, source, in-process integration target, executable-world
  target, and cumulative journey;
- Query-free applications and no Query dependency in the snapshot path;
- 3.12 ownership of semantic observation admission and bounded hot rebind;
- 3.13 ownership of broad Query projection;
- 3.14 ownership of admitted intents;
- 3.15 ownership of portals, focus, and runtime services;
- 3.16 ownership of appearance roles, state axes, themes, and motion styling;
- 3.17 and 3.18 ownership of authored expressions and pleasant composition;
- 3.19 through 3.22 ownership of full human and agent inspection products; and
- the existing compile-test topology, zero flake retries, warning-free
  workspace, and 400-line code/test file cap.

## Acceptance Evidence

Closure requires all of the following:

- the real pulse page contains at least two causally distinct visible mounted
  regions and independently known target/background points;
- a pixels-required snapshot binds exact product-issued frame, presentation,
  surface, binding, viewport, and retention identity to a real host capture;
- the target point resolves separately to an exact visible contributor stack
  and exact hit-test target;
- the background point does not return the target receipt;
- the target receipt traces through mounted instance, incarnation, graph node,
  target declaration, authored provenance, and evidence without a reconstructed
  tree;
- overlap, clipping, paint-only, hit-only, empty, disjoint-region, remount,
  foreign-world, stale, expiry, truncation, and coordinate-edge scenarios pass
  across the narrowest honest real subsystem boundaries;
- deterministic before-copy and after-copy replacement schedules distinguish
  superseded readback from exact retained-predecessor completion without an
  executable timing race;
- the overlay is published as a distinct canonical mounted frame, cites the
  base snapshot, remains paint-only, becomes visible in an external native
  capture, and is cleared by a separately observed successor before disposal;
- the cumulative executable world joins product identity observations and
  external pixels in the same child process;
- product-event-only, pixel-only, sole-node, paint/hit alias, clip omission,
  current-at-completion, direct-egui-overlay, forged-identity, parallel-tree,
  and residue controls turn red for the intended reason;
- ordinary unchanged frames perform zero visual snapshot and overlay work;
- snapshot construction and queries satisfy their structural complexity and
  retention bounds;
- incompatible protocol versions, disclosure denial, unsupported capture,
  capacity exhaustion, deadline, and indeterminate presentation are typed and
  mutation-sensitive;
- public examples compile and the human pulse workflow runs;
- no second truth path, product branch, binary, target, harness, or Query
  dependency exists; and
- format, clippy, workspace tests, line-cap, boundary-check, agent-context,
  dependency, compile-contract, consolidated integration, and Windows
  executable-world gates pass on exact final source.

## Successor Handoff

Milestone 3.12 inherits:

- exact snapshot, frame, presentation, surface, binding, and viewport affinity;
- immutable visible and hit-test spatial indexes;
- distinct paint and hit-test adjudication;
- retained current and predecessor snapshot postures;
- mounted-to-authored identity traces;
- explicit capture and query cost receipts;
- canonical overlay lifecycle;
- the nondegenerate pulse world;
- the advanced pulse observation protocol;
- `PulseExecutableWorld` progressed through snapshot and overlay evidence; and
- the committed snapshot-comparison insertion home.

Milestone 3.12 may add observation admission, affected-aspect detection,
preserve/remount planning, and identity-aware predecessor/successor comparison.
It may not redefine snapshot identity, reinterpret old pixels as current,
replace mounted receipt authority, or build a second visual index.

Milestones 3.13 through 3.18 add their semantic evidence to the same mounted
trace and pulse page. Milestones 3.19 through 3.22 expose the same receipts,
omissions, costs, and overlays through human and agent query products. By
Milestone 3.24, product entry, visual identity, action, observation, tracing,
artifact retention, and teardown are inherited infrastructure; remaining work
may concentrate on polish rather than discovering the first honest visual
boundary.
