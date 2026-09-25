# Milestone 3.16.2 — Enforcement review: raw coordinate edges

The enforcement section of [milestone 3.16.2](milestone-3.16.2.md) has
boundary-check forbid raw coordinate representations outside declared edges
in the covered modules. This record covers that rule: the conversion it needed
first, how the rule reads source, the edges it admits, and the tool debt found
along the way. The constructor and `Default` rules, the compile-fail proofs,
the interleaving model, and the Scroll regression proofs are separate phases.

## Platform points are typed

The [requirement 1 review](milestone-3.16.2-r1-review.md) kept the platform
event as an edge that took a bare `f32` point. Enforcement makes that edge a
type. `UiPlatformPoint` wraps the `UiHostSurfacePosition` a platform event
reports, and `from_host_position` is its only constructor. A position on any
basis other than viewport logical points is refused with its basis, so no
caller converts a physical or surface-relative position by accident.

Every reader of a pointer position takes the same point:

- `admits_platform_point` and `admits_platform_point_after` on a presented hit
  rect;
- the presented hit index, whose `at_point` and change check `affects` both
  take a platform point. Only the index reaches the raw `at_index_point`
  behind them, so its tests can still probe precision and nonfinite points;
- the presentation refresh check, which decides whether a pointer must be
  retested after a new frame;
- Scroll chrome targeting, track clicks, and thumb drags;
- pointer presence and pointer affordance;
- Portal dismissal. `UiPortalDismissalTrigger::OutsidePress` carries a
  `point` instead of `viewport_point_bits`.

Each reader handles a refused basis the way it handled any unusable report
before:

- targeting and the refresh check deny with `UnsupportedPositionBasis`;
- pointer presence reports no change;
- hover skips the report;
- chrome declines a press or drag, which leaves the report to ordinary
  routing;
- Portal dismissal names no trigger.

A release is the exception. A release on a refused basis places nothing, but
it still ends the drag it names and is reported as `PositionBasisRefused`. A
drag left latched would keep silencing the wheel on its axis and keep
refusing new chrome presses. A pending capture is released at the last point
it admitted, so the physical completion places no drag.

Tests mint points with `platform_point_for_test`, which goes through the same
host position.

## The rule

`BC7004_RAW_GEOMETRY_REPRESENTATION` reads every covered crate's production
source and rejects a written `[f32; 2]`, `[f64; 2]`, `[f32; 4]`, `[f64; 4]`,
or two-element tuple of `f32` or `f64` outside a declared edge.

The rule reads written types. It sees a raw form written in:

- a signature;
- a field or variant;
- an alias definition;
- an annotated binding;
- a closure parameter.

It does not see:

- a use of an alias;
- an inferred or destructured binding;
- separate scalar parameters, such as `UiPresentedPortalBounds::new(x, y,
  width, height)`;
- a macro body;
- an array whose length is a named constant.

Raw values therefore still flow outside the edges wherever no type is
written. Examples are destructured rect components in Portal planning and
requests, focus reveal, and Scroll replacement.

What bounds those flows is the accessors they start from. The sealed types
release raw components only through reviewed accessors:

- `components()` on the truth-status rects;
- `x()` and `y()` on a platform point;
- `index_point()` on a platform point.

An edge also exposes no raw-typed API more visible than it needs. That is why
the hit index's raw query is private to the index, and its public query takes
a platform point.

How the rule reads source:

- **Production only.** A `cfg` predicate is read three-valued, because a
  production build is not a test build and its features are unknown. Source is
  skipped only when a predicate can never hold, such as `test`,
  `all(test, ..)`, or `any()`. Anything feature-selected or unparseable is
  read, so certification-support code is covered and the rule fails closed.
  A module is skipped when every declaration of it, or of a module above it,
  is compiled out.
- **Every production target.** The library and each binary target (`main.rs`,
  `src/bin`, and manifest `[[bin]]` paths) are read. Platform Pulse is a
  binary, so this is how its layout is covered. A source that two targets both
  compile is reported once.
- **Items are named.** A finding names its item path, such as
  `UiMountedHitMechanicSource::update_tree`. An inline module is read once,
  with the file that declares it, so its items are named the way an edge
  written against that file names them.
- **Edges match exactly.** An edge path holds that file, or a directory and
  everything under it; `src/li` does not hold `src/lib.rs`. An edge item holds
  that item and items nested in it; `Bounds::adm` does not hold
  `Bounds::admit`.
- **Stale edges fail.** An edge that no longer holds any raw form is reported,
  so the edge list shrinks as conversion continues.
- **Missing crates fail.** A covered crate whose manifest or modules cannot be
  read is reported rather than skipped.

The edges live in `tools/boundary-check/config/road1.toml` under
`[[raw_geometry_denials]]`. Each edge names its kind and a reason, and each
covered crate carries guidance naming the sealed types to use instead. An edge
names the file that holds the raw form, and the items in it where the file
holds more than the edge's use. The directory edges that remain name whole
subsystems whose job is the raw form:

- the Motion sampler;
- `truth_geometry`;
- native presentation.

A stale edge fails, but a directory edge never goes stale while any file under
it holds a raw form. A directory edge is therefore kept only where the whole
directory is the edge.

## Edge kinds

The milestone names three kinds of edge. The requirement 1 review recorded
three more places where a raw form is the owner's inside, not a leak. The
configuration names them as kinds of their own rather than listing them as
serialization.

| Kind | Holds |
| --- | --- |
| `serialization` | host-contract projections, visual snapshots, inspection reports, Motion certification snapshots, and bit-pattern comparisons |
| `platform-event` | the scripted certification host, which reports a viewport extent the way a platform does |
| `gpu-upload` | the Scroll group clip translate and bounds, fill shading and glyph bounds in the host contract, native presentation, and the headless transcript raster |
| `spatial-index` | index boxes, which come only from `index_box` and `index_box_after`, and the index's private raw query. Every candidate the index returns is still tested against the platform point |
| `sealed-owner` | `truth_geometry`, the Motion sampler, and the Scroll group offset types, which own their representation and its named conversions |
| `committed-layout` | arithmetic on committed allocation before any truth status is assigned: paint omission, the subpixel grid correction, Portal content extent, and viewport placement |

`UiMountedCanonicalBox` and the mounted projection types are host-contract
vocabulary with named fields, not raw forms. They cross the serialization
edge as themselves, so the rule does not name them.

## Covered crates

The spec lists these covered owners:

- runtime presentation;
- Motion;
- Scroll;
- occurrence and viewport geometry;
- interaction;
- hit testing;
- Portal anchoring;
- pointer affordance;
- host contract;
- native and headless hosts;
- certification support;
- Pulse.

The rule covers every crate that holds one of these owners or reaches them:

- `worth-ui-runtime` holds every runtime owner and certification support.
- `worth-ui-host-contract` holds raw forms only as projection wire data and
  raster input.
- `worth-ui-host-native` holds raw forms only in its presentation raster and
  glyph atlas upload.
- `worth-ui-host-headless` holds raw forms only in its transcript raster.
- `platform-pulse` holds raw forms only in committed layout and in bit-pattern
  comparisons of observed values.
- `worth-ui-inspection` holds raw forms only in its receipt snapshot
  coordinate transform, which is wire data.
- `worth-ui-certification`, `worth-ui-test-support`,
  `worth-ui-native-platform`, and the public `worth-ui` facade hold none. Each
  is covered with no edges, so a raw form added there fails.

One crate holds raw forms and is not covered. `worth-ui-text` computes glyph
outlines and ink bounds in font units, before layout assigns any geometry a
place. The remaining uncovered crates, such as `worth-ui-dsl`,
`worth-ui-query-binding`, and `worth-ui-retained-order`, hold no presented
geometry.

A mutation run adds `fn probe(_point: [f32; 2]) {}` to Scroll chrome
`pointer_target.rs`. Boundary-check then reports exactly one BC7004 and
exits 1.

## Tool debt fixed

Running the tool's own gates surfaced failures in files this phase did not
otherwise touch. They are fixed here, not deferred:

- Eight clippy findings in `authority_value_gate_projection_canonical.rs`,
  `authority_value_gate_projection_matching.rs`, and
  `store_integrity_routes/policy.rs`. Seven allocated a new `Box` to replace a
  boxed type that can be written in place, and one was a match that is
  `matches!`.
- `constitution_entrypoint_contract` asserted that CI runs the constitution
  script. Commit 4fd335ad17 deliberately reduced CI to the line-cap check. The
  test now asserts that the hook and the terminal share one entrypoint, and
  that CI holds no second lane running boundary-check or agent-context.

## Debt reported

`public_bank_api_never_exposes_raw_query_elevation_authority` fails because
`bank-server/src/application_query/account_activity.rs::historical` takes a
`WorthQueryApplicationCommitReceipt`, since c5e53e9ef3. That is Bank
authority, outside this milestone's covered owners, and it is tracked as its
own task.
