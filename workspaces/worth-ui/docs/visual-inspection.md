# Visual Inspection

## What This Feature Is

Use visual inspection when you need to answer, “Which UI element produced this
pixel or region?” It captures one presented frame and lets you trace its
geometry and optional pixels back to the application element and source that
produced them. You hold a `UiVisualSnapshotReceipt<P>`, an immutable and
resource-bounded result for that one presentation.

A screenshot is only an image. A visual snapshot may include a screenshot, but
also carries snapshot, frame, presentation, surface, binding, coordinate,
visible-region, hit-test, identity-trace, cost, and retention evidence. Pixels
remain derived observations; they never become runtime authority.

## Why You Use It

- Explain which mounted element painted or accepts input at a point.
- Capture pixels for a known frame without losing their surface, coordinate,
  or presentation identity.
- Trace a visible region back to authored source and typed runtime evidence.
- Show a temporary identity overlay without creating an app-owned debug shape.
- Diagnose stale, superseded, omitted, denied, or uncertain capture work
  without guessing from a screenshot.

## Stable Entry Points

Import application-owned visual inspection types from
`worth_ui::facade::inspection` and the running shell from
`worth_ui::facade::app`.

- `WorthUiNativeApplicationShell::visual_inspection_authority()`
- `WorthUiVisualInspectionAuthority::issue_geometry_grant()`
- `WorthUiVisualInspectionAuthority::issue_pixel_grant()`
- `WorthUiVisualInspectionAuthority::issue_overlay_grant()`
- `WorthUiVisualInspectionAuthority::issue_comparison_grant()`
- `WorthUiNativeApplicationShell::begin_visual_geometry_snapshot(...)`
- `WorthUiNativeApplicationShell::begin_visual_pixel_snapshot(...)`
- `WorthUiNativeApplicationShell::poll_visual_snapshot(...)`
- `WorthUiNativeApplicationShell::cancel_visual_snapshot(...)`
- `WorthUiNativeApplicationShell::dispose_visual_snapshot(...)`
- `WorthUiNativeApplicationShell::show_identity_overlay(...)`
- `WorthUiNativeApplicationShell::present_visual_overlay(...)`
- `WorthUiNativeApplicationShell::clear_visual_overlay(...)`
- `WorthUiNativeApplicationShell::compare_visual_snapshots(...)`
- `UiUnbudgetedVisualSnapshotComparisonRequest::between(...)`

## Core Mental Model

The runtime owns visible meaning and mounted identity. The host owns native
capture and presentation mechanics. A pixel artifact records what the host
observed for an exact presentation epoch. These roles do not collapse:

```text
mounted runtime truth
-> governed host capture
-> immutable snapshot receipt
-> scoped point or region adjudication
-> mounted-to-authored identity trace
```

`WorthUiNativeApplicationShell` owns the active visual inspection authority.
The shell mints concrete geometry, pixel, and overlay grants from the policy
sealed at application construction. Grants are session-bound, audience-bound,
surface-scoped, artifact-scoped, and non-serializable. A receipt proves what
happened; it cannot execute a frame, publish an application, or mint another
grant.

## How It Executes

```text
select one current, retained, node, or snapshot-scoped region target
-> choose geometry-only, optional-pixel, or required-pixel artifact posture
-> issue the matching concrete grant from the running shell
-> begin a bounded capture with exact disclosure and an optional deadline
-> poll the linear pending handle
-> Captured | Superseded | Omitted | Denied | Indeterminate
-> inspect points, regions, identity traces, pixels, and cost
-> optionally publish and clear one identity overlay
-> optionally compare exact predecessor and successor receipts through a
   published rebind receipt
-> explicitly dispose the retained snapshot or shut down the application
```

The pending handle is linear: polling or cancellation consumes it and returns
the only valid next state. The runtime reserves bounded resources before it
asks the host to capture.

On the qualified Windows native host, a required-pixel request uses two host
progress steps. The first step admits one logical readback reservation and has
no GPU effect. The second step revalidates the exact presented source and may
submit its texture-to-buffer copy. Later polls observe completion. Cancellation
therefore distinguishes `CancelledBeforeReadback` from
`ReadbackMayHaveBegun`; an unprovable submitted completion becomes
`HostCompletion` indeterminate instead of being relabeled as captured.

## Small Example

Once you hold a required-pixels receipt, pixel access is total because the
artifact posture is encoded in its type. This exact fragment is compiled inside
the complete public program below:

<!-- compile-pass-fragment:required_pixels_are_total -->
```rust
fn required_pixels_are_total(receipt: &UiVisualSnapshotReceipt<UiPixelsRequired>) {
    let _ = receipt.pixel_artifact().bytes();
}
```

With `UiGeometryOnly`, `pixel_artifact()` is unavailable at compile time.
Receipt disposal still belongs to the running shell; reading the bytes does not
release their retained resource.

## Targets

Choose the narrowest target that answers the question:

- `UiCurrentPresentedSurfaceTarget` captures the current presentation of one
  selected surface.
- `UiRetainedPresentedSurfaceTarget` captures an explicitly retained
  predecessor or historical presentation.
- `UiMountedNodeVisualTarget` starts from one node selected from the current
  mounted frame.
- `UiClientRegionVisualTarget` is derived by consuming a retained snapshot and
  sealing a region in that snapshot's coordinate scope.
- a point is created only inside `UiVisualCoordinateScope`, so it cannot escape
  its snapshot or be reused against another receipt.

Multi-surface applications must select a surface. Foreign sessions, surfaces,
bindings, nodes, points, and regions deny at admission instead of silently
falling back to a first surface.

## Coordinates And Edges

Snapshot geometry uses client physical pixels. The receipt also records the
native client origin, client physical dimensions, logical viewport dimensions,
scale, translation, orientation, and rounding rule that relate authored
logical geometry to captured pixels.

Native pixel artifacts are canonical, tightly packed top-left RGBA8 rows. The
host keeps GPU row padding private and removes it only after completion. The
recorded transform uses the current native client origin, physical extent,
scale, and pixel-center-nearest rounding; consumers do not reconstruct that
mapping from a desktop screenshot.

Rectangles are half-open: left and top are included; right and bottom are
excluded. A point exactly on a shared right or bottom edge belongs to the
neighboring region, not both. Clipping and occlusion are applied before visible
contributors are indexed. Fractional scale and nonzero screen origin do not
change the client-pixel coordinate brand.

Appearance attribution names the mounted occurrence, surface, role, selected
cell, theme source, and contributing state evidence. Rounded surface and outline
bounds include their curved edge and qualified anti-alias fringe. Border seam
omissions are reported in surface-local coordinates, so an internal regional
edge cannot masquerade as a missing exterior border.

State and theme comparisons must retain exact predecessor/successor affinity.
The cumulative Pulse design adjudication captures zero-, one-, and two-modal
states at both qualified native sizes and checks relational Backdrop/Portal
composition, text, radii, and border corners. Those pixels remain secondary
evidence: mounted receipts and host acceptance establish what was presented.

## Visible Meaning Versus Hit Testing

`UiVisualPointAdjudication` deliberately contains two results:

- `UiVisualVisibleOutcome` reports the front-to-back stack of paint
  contributors at the point.
- `UiVisualHitTestOutcome` reports the target selected by an explicit total
  hit-test order.

A node may paint without accepting hits, accept hits without painting, do
both, or do neither. The visible and hit-test index identities are distinct
types and cannot substitute for one another. Bounded queries return
`Incomplete(UiVisualQueryBudget)` when candidate or result ceilings prevent a
complete answer; they never disguise a partial scan as complete.

## Outcomes And Affinity

Polling a pending capture returns either another
`UiVisualCapturePoll::Pending` handle or one
`UiVisualCapturePoll::Completed(UiVisualSnapshotOutcome<P>)`.

Completed outcomes are:

- `Captured(receipt)` for an exact retained snapshot;
- `Superseded(receipt)` when replacement crossed the capture schedule, with an
  explicit indication of whether predecessor pixels were copied;
- `Omitted(posture)` for absence such as no current frame, an expired frame,
  invisible node, unavailable historical pixels, or unsupported host capture;
- `Denied(posture)` for wrong-world, disclosure, coordinate, deadline, protocol,
  record, byte, structural, snapshot, or surface admission failure; and
- `Indeterminate(posture)` when a host request may have begun but exact
  affinity, native presentation, completion, or cleanup cannot be proved.

`UiVisualSnapshotAffinity` distinguishes `Current`, `RetainedPredecessor`, and
`Historical`. Never reinterpret an old receipt as current because its pixels
look unchanged.

## Policy, Artifacts, And Bounds

Application construction seals `UiVisualInspectionPolicy`. A request must match
the grant's exact disclosure posture. Local unredacted pixels are an explicit
policy choice, not an ambient debug bypass.

The policy bounds:

- visible-region and hit-test-region record counts;
- query candidates and returned results;
- retained bytes per receipt and per session;
- retained structural bytes per receipt and per session;
- active snapshot count;
- overlay count and retained overlay leases; and
- capture deadline and artifact posture.

`UiGeometryOnly`, `UiPixelsOptional`, and `UiPixelsRequired` encode artifact
posture in the receipt type. Geometry-only receipts have no pixel accessor.
Required-pixel receipts expose a total `pixel_artifact()`. Redaction, color
space, capture source, stride, retention, and byte count remain explicit
evidence.

## Inspection And Debugging

Visible contributors and hit-test targets carry `UiVisualIdentityTrace`. The
trace follows:

```text
mounted node receipt
-> mounted instance and incarnation
-> graph node
-> declaration
-> authored semantic name and source provenance
-> typed evidence references
```

The trace explains why a pixel belongs to mounted meaning. It does not
reconstruct meaning from the pixel.

Start debugging from `UiVisualSnapshotAffinity`: confirm the expected frame,
surface, binding generation, presentation epoch, and current/retained relation.
Then inspect point or region outcomes, their query budgets, identity traces,
and `UiVisualInspectionCostReceipt`. Inspect the pixel artifact last. Matching
colors cannot repair wrong affinity or incomplete identity evidence.

## Overlay Lifecycle

An identity overlay begins from a target consumed from a retained snapshot.
`show_identity_overlay` admits a pending overlay; `present_visual_overlay`
publishes a successor mounted frame containing four border regions and returns
`UiPublishedVisualOverlay`. The base snapshot remains the explanation basis.

Clearing consumes the published overlay and returns
`UiClearedVisualOverlayReceipt` for another successor frame. A published
overlay cannot be cleared twice. A superseded, expired, foreign, or unretained
target denies instead of being silently redrawn against current pixels.

## Recovery And Disposal

Cancel pending capture work through `cancel_visual_snapshot`. Its typed
`UiVisualCancellationPosture` says whether host readback did not begin, may
have begun, or cleanup is indeterminate.

Dispose retained receipts explicitly through `dispose_visual_snapshot`.
Clearing an overlay and disposing its base snapshot are separate lifecycle
operations. Application shutdown cancels pending captures, disposes retained
snapshots and overlays, and reports every released count, pixel byte, and
structural byte. A drop is not evidence of a successful clear or exact cleanup.

Retry only from the returned posture:

- a pre-effect denial opens no host work and may be corrected and resubmitted;
- `Superseded` requires selecting the intended current or retained target;
- `Indeterminate` requires reconciliation or shutdown, not optimistic relabeling;
- capacity denial requires disposal or a narrower admitted policy;
- disclosure denial requires a matching grant and request, never widened
  authority.

## Real Example

The following block is an exact mirror of the existing
`visual_snapshot_facade_compile.rs` pass source. The Phase 5 topology audit
rejects documentation drift, and the existing two-session compile matrix
compiles it against the real public facade.

<!-- compile-pass-source:tests/ui/visual_snapshot/pass/governed_visual_snapshot_lifecycle.rs -->
```rust
use worth_ui::facade::app::WorthUiNativeApplicationShell;
use worth_ui::facade::app::{UiMountedInspectedFrame, UiMountedVisualTargetDenial};
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor,
    ComponentHitTestContract, ComponentHitTestOrder, ComponentId, ComponentPropSchema,
    ComponentStateOwnership,
};
use worth_ui::facade::inspection::{
    UiClientPhysicalPixel, UiClientPhysicalRect, UiClientRegionVisualTarget,
    UiCurrentPresentedSurfaceTarget, UiGeometryOnly, UiHitTestRegionIndexIdentity,
    UiMountedNodeVisualTarget, UiPixelsRequired, UiVisibleRegionIndexIdentity,
    UiUnbudgetedVisualSnapshotComparisonRequest, UiVisualComparisonPixelPolicy,
    UiVisualOverlayDenial, UiVisualOverlayGrant, UiVisualOverlayTarget, UiVisualQueryBudget,
    UiVisualSnapshotComparisonBudget, UiVisualSnapshotComparisonOutcome, UiVisualSnapshotDenial,
    UiVisualSnapshotReceipt, UiVisualSnapshotRequest,
};
use worth_ui::facade::rebind::UiRebindReceipt;

struct GovernedRequestInputs {
    geometry_target: UiCurrentPresentedSurfaceTarget,
    pixel_target: UiCurrentPresentedSurfaceTarget,
}

fn governed_requests_typecheck(
    shell: &mut WorthUiNativeApplicationShell,
    inputs: GovernedRequestInputs,
) -> Result<(), UiVisualSnapshotDenial> {
    let authority = shell.visual_inspection_authority();
    let _ = authority.policy();
    let geometry_grant = authority.issue_geometry_grant();
    let pixel_grant = authority.issue_pixel_grant();
    let _overlay_grant = authority.issue_overlay_grant();

    let geometry = shell.begin_visual_geometry_snapshot(
        &geometry_grant,
        UiVisualSnapshotRequest::for_local_development_unredacted_frame(inputs.geometry_target),
    )?;
    let _ = shell.poll_visual_snapshot(geometry, 0);

    let pixels = shell.begin_visual_pixel_snapshot(
        &pixel_grant,
        UiVisualSnapshotRequest::for_local_development_unredacted_frame(inputs.pixel_target)
            .artifacts(UiPixelsRequired::policy()),
    )?;
    let _ = shell.cancel_visual_snapshot(pixels);
    Ok(())
}

fn required_pixels_are_total(receipt: &UiVisualSnapshotReceipt<UiPixelsRequired>) {
    let _ = receipt.pixel_artifact().bytes();
}

fn compare_rebound_snapshots(
    shell: &mut WorthUiNativeApplicationShell,
    predecessor: &UiVisualSnapshotReceipt<UiPixelsRequired>,
    successor: &UiVisualSnapshotReceipt<UiPixelsRequired>,
    rebind: &UiRebindReceipt,
) -> UiVisualSnapshotComparisonOutcome {
    let grant = shell
        .visual_inspection_authority()
        .issue_comparison_grant();
    let request =
        UiUnbudgetedVisualSnapshotComparisonRequest::between(predecessor, successor, rebind)
            .with_pixel_observation(UiVisualComparisonPixelPolicy::IfAlreadyRetained)
            .with_budget(
                UiVisualSnapshotComparisonBudget::bounded(128)
                    .expect("comparison budget is nonzero"),
            );
    shell.compare_visual_snapshots(&grant, request)
}

fn coordinate_brand_is_usable(receipt: &UiVisualSnapshotReceipt<UiGeometryOnly>) {
    receipt.with_coordinate_scope(|scope| {
        let point = scope.client_pixel(
            UiClientPhysicalPixel::new(80, 48).expect("canonical point is nonnegative"),
        ).expect("canonical point lies inside the captured extent");
        let _ = scope.adjudicate_point(point);
        let region = scope.client_region(
            UiClientPhysicalRect::new(0, 0, 160, 96).expect("canonical region is nonempty"),
        );
        let _ = scope.adjudicate_region(region);
    });
}

fn shell_disposes_owned_snapshot(
    shell: &mut WorthUiNativeApplicationShell,
    receipt: UiVisualSnapshotReceipt<UiGeometryOnly>,
) {
    let _ = shell.dispose_visual_snapshot(receipt);
}

fn live_selected_node_seals_target(
    frame: UiMountedInspectedFrame,
) -> Result<UiMountedNodeVisualTarget, UiMountedVisualTargetDenial> {
    frame.node_visual_target()
}

fn snapshot_scope_seals_region_target(
    receipt: UiVisualSnapshotReceipt<UiGeometryOnly>,
) -> Result<UiClientRegionVisualTarget, UiVisualSnapshotDenial> {
    receipt.into_client_region_target(|scope| {
        scope.client_region(
            UiClientPhysicalRect::new(0, 0, 8, 8).expect("compile-contract crop is nonempty"),
        )
    })
}

fn authored_hit_contract_carries_distinct_total_order() {
    let allocation = ComponentAllocationMeasurementContract::fill_viewport();
    let contract = ComponentHitTestContract::allocation_bounds(
        ComponentHitTestOrder::front_to_back(0),
        allocation,
    );
    let component = ComponentDescriptor::new(
        ComponentId::new("compile.visual.hit").unwrap(),
        ComponentPropSchema::named("compile.visual.hit.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_hit_test(contract);
    let _ = component;
}

fn index_identity_types_remain_distinct(
    visible: UiVisibleRegionIndexIdentity,
    hit_test: UiHitTestRegionIndexIdentity,
) {
    fn accepts_visible(_: UiVisibleRegionIndexIdentity) {}
    fn accepts_hit_test(_: UiHitTestRegionIndexIdentity) {}
    accepts_visible(visible);
    accepts_hit_test(hit_test);
}

fn query_budget_receipt_is_interpretable(budget: UiVisualQueryBudget) {
    let _ = (budget.maximum_results(), budget.maximum_candidates());
}

fn governed_overlay_lifecycle_typechecks(
    shell: &mut WorthUiNativeApplicationShell,
    grant: &UiVisualOverlayGrant,
    target: UiVisualOverlayTarget,
) -> Result<(), UiVisualOverlayDenial> {
    let pending = shell.show_identity_overlay(grant, target)?;
    let published = shell
        .present_visual_overlay(pending, 2, 1)
        .map_err(|failure| failure.denial())?;
    let _cleared = shell
        .clear_visual_overlay(published, 3, 2)
        .map_err(|failure| failure.denial())?;
    Ok(())
}

fn main() {
    let _ = (
        governed_requests_typecheck,
        required_pixels_are_total,
        compare_rebound_snapshots,
        coordinate_brand_is_usable,
        shell_disposes_owned_snapshot,
        live_selected_node_seals_target,
        snapshot_scope_seals_region_target,
        authored_hit_contract_carries_distinct_total_order,
        index_identity_types_remain_distinct,
        query_budget_receipt_is_interpretable,
        governed_overlay_lifecycle_typechecks,
    );
}
```

## How It Relates To Other Features

The permanent product workflow and its independently certified native pixels
are documented in [Application lifecycle](./application-lifecycle.md#platform-pulse).
The Pulse uses this same snapshot, point-trace, overlay, and disposal surface;
it does not own a diagnostic shortcut.

- Use [Application inspection](./inspection.md) for non-visual declaration,
  graph, plan, obligation, and evidence questions.
- Use the application lifecycle to obtain and shut down the running shell.
- Use an identity overlay when a human needs to see the exact selected region;
  use the receipt directly when only machine-readable evidence is needed.
- Keep host-specific screenshot tooling outside semantic explanation. A native
  capture may corroborate consequence, but the mounted trace explains meaning.

## Anti-Patterns

- Treating equal pixels as proof that two frames or mounted identities are
  equal.
- Taking an unscoped desktop screenshot and guessing which surface produced it.
- Reusing a point, region, pending handle, overlay, or receipt after its
  consuming transition.
- Using a pixel grant for geometry-only work or an overlay grant for capture.
- Treating `Incomplete`, `Omitted`, or `Indeterminate` as an empty successful
  result.
- Dropping a receipt and assuming that explicit disposal or overlay clearing
  succeeded.
- Building a renderer-local debug overlay that bypasses the mounted successor
  frame.

## Predecessor And Successor Comparison

Comparison is admitted only when the caller supplies two retained snapshot
receipts and the exact published `UiRebindReceipt` that relates their mounted
frames. The comparison grant is minted by the same running visual authority.
This makes session, frame, presentation, and identity affinity explicit rather
than inferring continuity from equal pixels.

`UiVisualComparisonPixelPolicy::IfAlreadyRetained` may examine pixels already
owned by both receipts. Comparison never recaptures either frame.
`UiVisualComparisonPixelPolicy::Omit` still returns structural and identity
continuity. A bounded `UiVisualSnapshotComparisonBudget` limits structural
records examined.

The result is `Compared`, `Omitted`, `Expired`, `Incompatible`, or `Denied`.
Foreign sessions, unrelated frames, expired receipts, wrong grants, and
insufficient budgets remain named outcomes. Dispose predecessor and successor
receipts independently after the comparison is no longer needed.

## Current Limits

Pixel capture depends on the selected host capability and policy. Historical
pixels may be unavailable even when historical mounted identity remains
inspectable. The qualified Windows host retains at most four native readback
slots and 16 MiB of aggregate padded readback capacity. A larger or fifth
concurrent request denies before GPU work. A submitted request keeps consuming
that bounded capacity after caller cancellation or unknown physical completion
until the native host proves settlement; shutdown retries do not report a false
zero while such work remains.

Current Windows Platform Pulse certification reads the exact retained native
presentation source through the runtime and separately captures the
compositor-visible client for correlation. The external client image is not a
substitute for source affinity, retained regions, or the runtime receipt. Other
platform adapters do not inherit that certification from a successful compile.

## Related Docs

- [Application lifecycle](./application-lifecycle.md)
- [Hot rebind](./hot-rebind.md)
- [Application inspection](./inspection.md)
- [Worth UI architecture](./architecture.md)
- [AI diagnostics architecture](../../../_docs/worth-ui/ai-diagnostics.md)
