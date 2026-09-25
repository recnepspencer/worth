# Milestone 3.16.2 — Requirement 1 geometry review

Requirement 1 of [milestone 3.16.2](milestone-3.16.2.md) says geometry carries
its truth status. Published, accepted, and displayed geometry are distinct
sealed types. Arithmetic is defined only within one status, and crossing
statuses takes a named conversion that consumes the evidence proving it. Raw
coordinate arrays remain only at declared serialization, platform-event, and
GPU-upload edges. This record lists each surface review converted, each raw
form it kept and the edge that owns it, and the crossings that now carry
evidence.

## Vocabulary

The runtime owns the typed geometry in `mounting::presentation::truth_geometry`.
None of the types can be built from a bare array outside it. Each stores its
components as a `UiLogicalRect`, which normalizes `-0.0` to `0.0` so that a
value reached from below equals the same value reached from above.

| Status | Type | Minted by |
| --- | --- | --- |
| Published | `UiPublishedRect` | `from_committed_box` or `from_committed_components`: a committed mounted box, or committed components |
| Accepted | `UiAcceptedRect` | `UiPresentationMotionSampleReceipt::from_track_sample`, against the presentation basis the tick was sampled on, admitting a `UiTrackSamplePlace` (see below) |
| Displayed | `UiDisplayedRect` | `displayed(accepted, displayed_basis)`, which consumes the displayed basis of an admitted presentation witness for the same surface binding |
| Displayed | `UiDisplayedScrollOffset` | `from_rest(published_rest, displayed_rect)` |
| Published | `UiScrollPoseShift` | `between(from, to)`: the translation between two committed Scroll offsets, composed with `then` and differenced with `beyond` |

Maps name both ends: `UiPublishedMap` maps published to published, and
`UiPublishedToAcceptedMap` maps published to accepted. Both refuse a source
without area, which has no extent to scale its layout by.

The named crossings are:

- **Accepted to displayed.** `UiDisplayedRect::displayed` consumes the witness
  basis and refuses a sample drawn on another binding.
- **Displayed to published.** `UiPublishedRect::adopting` is the one place a
  published owner takes on displayed geometry. `UiDisplayedScrollOffset` is the
  one place Scroll does, through `settle_accepted_sample`.
- **Track sample places.** A track names where each sample puts its target
  as a `UiTrackSamplePlace`, and `from_track_sample` admits each kind through
  its own crossing onto the tick's basis. `Published` geometry, a start at the
  predecessor's layout or rest at the target's, is held there by
  `UiAcceptedRect::holding`. A `Carried` accepted sample, an interrupted
  predecessor's, crosses with `rebased`, which refuses another surface or
  binding. A `Curve` point is admitted with `sampled`, in the coordinate space
  of the published target it heads for, so motion sampling never handles a
  bare coordinate kind.
- **Curve evaluation.** `UiTrackCurveSpan` names the span a running track
  travels, from the accepted sample it departed from to the published target
  it heads for. Its private `ends()` is the one place those two statuses meet,
  as the components interpolation consumes. Nothing leaves the span except a
  `UiTrackCurvePoint`, which only the span mints and only a sample admits, a
  velocity bounded to the span, or the outgoing curve a retarget
  differentiates.
- **Accepted against published.** `UiAcceptedRect::stands_at(published)` and
  `UiDisplayedScrollOffset::stands_at(offset)` compare across statuses without
  converting either side. They answer "has published truth caught up?".
- **Scroll group offsets.** A group's accepted or displayed offset is
  measured from its published rest content box only by `of_sample`, which
  takes the accepted sample or the displayed rect as evidence. An accepted
  group offset carries the presentation basis it was accepted on: its
  sample's, or, for a group the tick does not
  move, the tick receipt's through `UiAcceptedGroupOffset::held_by(tick,
  standing)`. Every translation carries that basis, and a command's moves
  compose only on the basis the sample work is prepared for; anything else is
  refused as `PresentationBasisMismatch`. A standing is shown against a
  displayed command only through `UiBoundGroupStanding::shown_by(base)`. Each
  bind stamps its displayed bases and bound standings with one
  `UiScrollGroupBind`, and `shown_by` refuses a base another bind read, as
  `DisplayedBaseFromAnotherBind`. Its `move_to` is the one source of a
  displayed-to-accepted translation.

## Converted

### Runtime

| Owner | Before | After |
| --- | --- | --- |
| Presented hit rows | `UiMountedCanonicalBox` bounds and clip | `UiPresentedHitRect::{Published, Displayed}`; owners commit from `adopted()`, the spatial index reads `index_box()`, and events test `admits_platform_point` |
| Displayed hit rows under Scroll | a displayed box translated by each pose shift | `Displayed { rect, layout_shift }`. The displayed rect never moves. The published shift is applied to it only at the platform-event edge (`admits_platform_point_after`) and the index edge (`index_box_after`). `adopted()` adopts the displayed rect, then follows the pose as published |
| Hit-test basis | a bare observation basis | `UiDisplayedSurfaceBasis`, the witness's displayed basis; `classify` returns it |
| Hit-index motion refresh | an observation basis | the displayed basis retained with the binding |
| Hit-index scroll translation | `[f32; 2]` summed per record | `UiScrollPoseShift`; succession differences with `beyond` |
| Hit-index change detection | `==` on raw boxes | `UiPresentedHitTestRow::occupies_same_place`. It compares each rect by `index_box()`, the box the spatial index would hold, because the question is whether the index must change; it is read at the index edge, not across statuses |
| Retained interaction rows | a committed pair rebuilt through an unreachable branch | `committed()` reads the mechanic's bounds and clip boxes directly through `from_committed_box` |
| Occurrence Scroll translations | `[f32; 2]` per mounted instance | `UiScrollPoseShift` per mounted instance. Rows, clips, and regions follow a pose through `UiPublishedRect::box_following_pose` |
| Motion sample geometry for a target | a canonical box rebuilt from sample components | `UiDisplayedRect`, displayed against the hit-test basis's witness |
| Track curve ends | predecessor components read from both statuses | `UiTrackCurveSpan` (see Vocabulary) |
| Track sample geometry | components flattened into `from_track_sample`, whatever their status | `UiTrackSamplePlace`, admitted through `holding`, `rebased`, or `sampled` (see Vocabulary) |
| Portal entrance acceptance | component arrays compared across statuses | the witness's displayed basis is passed in and must equal the sample's; the published base is compared as a value; the accepted sample `stands_at` the published initial geometry |
| Portal entrance transform | a hand-built canonical box | `UiPublishedRect::canonical_box` at the host-contract edge |
| Portal entrance offset | a raw `y` addition | `UiPublishedRect::translated`, whose only caller is this declared offset |
| Scroll group sampling | offsets reissued through `UiGroupMove` | `held_by` and `shown_by` (see Vocabulary) |
| Displayed Scroll offset | an offset with a basis nothing read | `apply_presented_scroll_geometries` refuses, as `ForeignSurface`, an offset displayed on another surface's binding |

### Interaction and Portal

| Owner | Before | After |
| --- | --- | --- |
| Presented interaction geometry | canonical boxes | `UiPresentedHitRect` bounds and clip; the viewport is a `UiPublishedRect` |
| Presented-frame targeting | a local `contains` on raw boxes | `admits_platform_point` on both rects |
| Pointer presence geometry | canonical boxes, with an `f64` `contains` | `UiPresentedHitRect`; a retest converts the subpixel position to the same `f32` platform point targeting uses, so a retest and its hit test agree at every edge |
| Portal placement | anchor, clip, and bounds as canonical boxes | `UiPublishedRect` anchor and clip; `UiPresentedPortalBounds` wraps a `UiPublishedRect`. Planning works on published rects and checks `has_area` |
| Portal content bounds | canonical layout and paint boxes | `UiPublishedRect` layout and paint |
| Portal outside-press dismissal | an optional canonical box, a local `contains` | an optional `UiDisplayedRect`; otherwise the committed placement and anchor, both `UiPublishedRect`, tested with `admits_platform_point` |
| Portal overlay projection | placement boxes passed through | `canonical_box()` at the host-contract wire edge in `portal_overlay` |
| Portal planning anchor | the row's raw bounds | `adopted()`, a `UiPublishedRect` |
| Focus reveal, service reconciliation, Scroll replacement | the row's raw bounds | `adopted().components()` |

### Host contract

`UiHostSurfaceLogicalGeometry` is removed, with its constructor
`from_sampled_logical_projection` and the `NonFinite` and `NegativeExtent`
denials that only it produced. It built host logical geometry from a raw
array and a bare observation basis with no evidence. Nothing used it, and the
runtime module the boundary rule allowed to call it no longer exists. The two
boundary rules that named the constructor were removed with it.
`UiHostPhysicalPixelGeometry` is host-observed input and stays.

## Kept edges

A raw array or canonical box is kept only at one of these edges.

- **Serialization.** The host-contract projection (`UiMountedHitTestMechanic`,
  `UiMountedCanonicalBox`, Portal overlay rows, and appearance rows), the Motion
  certification snapshot, and inspection reports. Hosts and certification read
  these as wire data.
- **Platform event.** `admits_platform_point` and
  `admits_platform_point_after` take the `f32` point a platform event reports.
  `UiPresentedHitRect::platform_box` exists only under `test`, where scenarios
  aim synthetic events at a row. Scroll chrome points are derived from the
  committed mounted pose, which is published, and are compared at this edge.
- **GPU upload.** `UiMountedPresentationTransform` in `sample_transform` and the
  Portal entrance transform, and the clip translate in Scroll group sampling.

Two uses sit next to these edges but are not new edges:

- The spatial index receives boxes only from `index_box` and `index_box_after`
  on a presented hit rect. It narrows candidates, and every candidate is still
  tested with `admits_platform_point`.
- `occurrence_geometry::state::scroll::translate` moves a raw mounted box by a
  subpixel grid correction. Its only caller is the allocation projection,
  which applies the correction to committed allocation before any status is
  assigned.

## Crossings reviewed

- **Pose shifts are published.** A Scroll pose shift is the translation between
  two committed offsets. The hit index composes shifts with `then` and
  differences them with `beyond`. A displayed rect never follows a shift; the
  shift is applied only where a platform point or the index reads the row, so
  a shift never becomes displayed truth.
- **Scroll chrome.** Chrome geometry is derived from the committed mounted
  pose. It never reads a Motion sample, so no crossing is needed.
- **Adoption.** An owner that commits from a displayed hit rect goes through
  `adopted()`, which crosses with `UiPublishedRect::adopting` and then follows
  the published pose. That is the only displayed-to-published path for rects.
- **Dismissal.** An outside press is tested against the displayed Motion sample
  when the host shows one, otherwise against the committed placement. It never
  mixes the two.
- **Pointer presence.** Presence and targeting read the same rects and the same
  `f32` point, so they cannot disagree about an edge.
- **Rebinding.** When a publication moves a surface to a new binding, a
  track's current sample follows through `UiAcceptedRect::rebound`, which
  takes any binding. Its one caller is `rebind_published_presentation`: the
  publication that rebound the surface is the evidence, and the sample stays
  accepted, not displayed, until a witness of the new binding shows it.
- **Entrance with no witness.** `install_motion_commit` installs every committed
  track, not only entrances. It accepts a Portal entrance as displayed only
  against the witness `current_presentation_for_surface` returns for the
  target's surface. When the surface has no current witness, no frame could
  have displayed the entrance, so it stays published. Its first sample then
  damages from the published successor, which is what the host still shows.
  This is the same posture as an entrance that was never presented. It is not
  tested separately: every fixture that installs a Portal entrance publishes a
  witness first, and reaching the branch would take a surface unbound between
  publication and installation, which no current path produces.

## Other covered owners

- **Native and headless hosts** receive only host-contract projections and
  report platform events. They hold no runtime truth geometry. Native cursor
  mapping carries no geometry. In `event_loop::contract`,
  `UiNativePhysicalProgressCorrelation::Unattributed` is now compiled only
  under `test` or `certification-support`, the only builds that construct it.
  The host attributes every progress it issues.
- **Platform Pulse** layout does committed arithmetic only, on published values
  it owns.
- **Certification support** reads the host-contract projection, which is the
  serialization edge. It cannot reach runtime truth geometry.

## Enforcement

This record covers conversion only. Boundary rules and `compile_fail` doctests
that stop raw geometry from returning outside the kept edges belong to the
enforcement phase of 3.16.2, which follows R4.
