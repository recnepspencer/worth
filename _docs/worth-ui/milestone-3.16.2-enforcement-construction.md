# Milestone 3.16.2 — Enforcement review: sealed construction and lifecycle defaults

The enforcement section of [milestone 3.16.2](milestone-3.16.2.md) forbids
three things in the covered modules. The [raw coordinate
review](milestone-3.16.2-enforcement-review.md) covers the first. This record
covers the other two, the compile-fail proofs, and the Clippy configuration
that covers what boundary-check cannot:

- construction of requirement 1 and 3 types outside their owners;
- `Default` on requirement 2 state.

The interleaving model and the Scroll regression proofs are separate phases.

## Construction outside owners

`BC7005_SEALED_TRUTH_CONSTRUCTION` reads each covered crate's production
source, the same way BC7004 does: `cfg` predicates are three-valued, every
production target is read, and findings name their item. Each crate lists its
sealed types in `tools/boundary-check/config/road1.toml` under
`[[truth_type_denials]]`. Each type names its requirement and its owners.

An owner is a file, or a directory and everything under it. Inside its owners
a type is built freely. Outside them the rule refuses:

- a struct literal or tuple constructor that names the type;
- an `impl` of the type;
- a `use` that renames the type or reaches through it, and a `type` alias of
  it. Either would let a later construction hide behind another name.

The rule reads the owners' source to find the constructors themselves: enum
variants, associated functions without a receiver that return `Self` or the
type, and associated consts of the type. Outside the owners, a call that names
a constructor by path, such as `UiPublishedRect::from_committed_box(..)`, is a
construction. It is admitted only at a declared mint: the constructor, the
items that may call it, and the reason. Tokens inside a macro body are scanned
for `Type::constructor`, so `vec![UiPublishedRect::from_committed_box(..)]`
is seen too. A match arm naming a variant reads it and is not a construction.

Declarations fail when they go stale:

- a sealed type that is not defined in its owners;
- a mint that names no constructor of a sealed type;
- a declared caller that no longer makes its call.

The mint list therefore shrinks as callers are removed.

What BC7005 does not see:

- a constructor called as a method, such as `rect.translated(..)`. Mints of
  that shape are enforced by Clippy, below;
- a conversion through a trait, such as `x.into()` or `From::from(x)`. No
  sealed type implements a conversion from raw values, so a new one would be
  an `impl` of a sealed type outside its owner, which the rule refuses, or
  inside its owner, which review reads;
- a function outside the owners that returns a sealed type it received. That
  moves a value; it does not construct one.

### Owners

Most owners are the module that defines the types:

- the truth-status rects, offsets, maps, and platform point in `truth_geometry`;
- the Motion track geometry, and the sampled clip and damage regions, in
  `motion_sampling`;
- the presented-frame witness and displayed surface basis in
  `presented_surface`;
- `UiPresentedPortalBounds` in Portal placement.

Three owners needed a decision:

- **`UiPresentedHitRect`** is owned by the whole `interaction_basis` module,
  not its `hit_rect.rs` file. A hit row lifts published geometry into a hit
  rect where it reads committed and sampled rows, and that is the owner's own
  work.
- **The Scroll group offsets, standings, and translations** are owned by the
  `scroll_motion_groups` module. Its files split one owner by concern, and each
  builds the others' values.
- **`UiTrackScreen`** is not listed as sealed. It is private to
  `track_sampling`, so the compiler already refuses it outside that module. It
  is listed as a lifecycle state, below.

`UiMountedSurfacePresentationCompletion` and `UiMountedIssuedSurfaceWork` are
sealed in the host contract's `completion.rs`, and
`UiHostPresentationCompletionToken` in `presentation.rs`. The completion has no
constructor at all. The issued work is built only by
`UiMountedFrameConsumptionView::issued_work`.

### Declared mints

Each runtime mint is called only where its input is read: committed layout, a
Motion sample, a host position, or a witness.

| Mint | Callers |
| --- | --- |
| `UiPublishedRect::from_committed_box` | the Scroll settlement owner, the Portal content extent, committed and mounted hit rows, and the presented interaction geometry |
| `UiPublishedRect::from_committed_components` | Portal bounds, Scroll content geometry, and certification scale geometry |
| `UiPublishedRect::adopting` | a hit rect adopting its prepared entrance |
| `UiPublishedMap::between` | a hit row's prepared entrance |
| `UiPublishedToAcceptedMap::of_sample` | a hit row's Motion sample |
| `UiAcceptedRect::sampled`, `holding` | the Motion sampler's track place |
| `UiPresentationSampledClipGeometry::from_presented_components` | Motion sample work, from a paint command's committed clip |
| `UiDisplayedRect::displayed` | the Scroll settlement, Scroll group acceptance, a hit row's Motion sample, and committed Motion geometry |
| `UiDisplayedScrollOffset::from_rest` | the Scroll settlement owner |
| `UiPlatformPoint::from_host_position` | Scroll chrome ingress, Portal dismissal, the presentation refresh check, and presented targeting |
| `UiScrollPoseShift::between`, `none` | occurrence geometry's Scroll pose, the hit index base, and a displayed hit rect |
| `UiPresentedPortalBounds::new` | Portal planning |
| `UiPresentedHitRect::Published` | certification interaction geometry |
| `UiPresentedSurfaceWitness::admit` | the three presentation coordinator settlements |

`UiDisplayedRect::displayed` is the only way to reach displayed geometry, and it
takes a displayed basis. Only an admitted witness yields a displayed basis.

## Clippy covers the method-shaped mints

Three mints are methods, which BC7005 cannot resolve from a path.
The workspace root `clippy.toml` lists them under `disallowed-methods`:

- `UiPublishedRect::translated`, the Portal entrance start;
- `UiMountedFrameConsumptionView::acknowledge_presented` and
  `UiHostPresentationCompletionToken::acknowledge_presented`, a host saying its
  work is on screen.

Each permitted call carries `#[expect(clippy::disallowed_methods, reason =
"..")]`. An `expect` that stops matching a call fails the build, so a removed
call cannot leave a stale permission behind. BC7005 also reads these
suppressions:

- In production source, an attribute that can silence the lint must sit at a
  caller declared for a Clippy mint. That means `clippy::disallowed_methods`,
  the `clippy::style` and `clippy::all` groups, or rustc's `warnings`, in
  `allow`, `expect`, or `cfg_attr`. The audit reads attributes on items, on
  `mod` declarations, and a file's own inner attributes. A file-level
  suppression names no item, so it holds only for a caller declared by path
  alone. None is declared.
- A suppression must be `expect`, not `allow`.
- A covered crate's manifest may not set any of those lints to `allow` in its
  `[lints]` table, or in the `[workspace.lints]` it inherits.
- Every Clippy mint in `road1.toml` must be disallowed in `clippy.toml`. A
  boundary-check test reads both files to hold the two lists together.

An `expect` covers every disallowed method called in its scope, so the audit
admits a suppression at any caller of any Clippy mint. It cannot tell which
method the `expect` was written for. Clippy has the same granularity.

The audit does not read lint levels set outside source and manifests: a
`-A` flag in `RUSTFLAGS` or a `.cargo/config.toml`, or a `clippy.toml` nearer
a crate than the root one, which Clippy would read instead. Today neither
exists: the repository's `.cargo/config.toml` sets no lint flag, and the root
`clippy.toml` is the only one.

The production hosts that acknowledge work are:

- the native completion and pending settlement;
- the headless recorder;
- the scripted certification host;
- the certification witness.

Test hosts also acknowledge, and they carry the same `expect`:

- the native retry tests;
- the runtime admission tests;
- the identity trace host;
- the text presentation coordinator harness.

BC reads production only, so these hosts are not declared as mints. Clippy
still requires each one to state its reason.

The configuration uses no `disallowed-types`. The sealed runtime types are
crate-private. The public host-contract types have private fields and no public
constructor. So nothing outside an owner can build one except through a mint.
A disallowed type would forbid naming the type in a signature. That would
forbid the legal reads that every consumer of a witness or a displayed rect
makes.

## `Default` on lifecycle state

`BC7006_LIFECYCLE_STATE_DEFAULT` rejects `Default` on each requirement 2
lifecycle state, whether it comes from `derive(Default)`, from
`cfg_attr(.., derive(Default))`, or from an `impl Default` written with any
path. A default would pick a variant silently, so a state's variant is always
named where the state is built. A declared state that is not defined in
production source fails as stale.

The lifecycle states are the enums that the [requirement 2
review](milestone-3.16.2-r2-review.md) converted:

- **Runtime:** `UiTrackMotion`, `UiTrackScreen`, `UiMountedFrameState`,
  `UiStagedPointerOutput`, `UiSelectedDiagnostics`,
  `UiPendingAppearanceReconstruction`, `UiLocalInputStopRecipient`,
  `UiScrollChromeLatchState`, `UiPortalOverlayBindingEffect`, and
  `UiMotionSampleCertification`.
- **Host contract:** `BoundResources`.
- **Native host:** the readiness states, `UiNativePlannedInjection`,
  `UiNativePendingReconstruction`, `QualifiedObservation`,
  `UiNativePaintOutcome`, `UiNativeRetainedIdentityOverlay`,
  `OriginalOrderPlacement`, `UiNativePresentationOwners`,
  `UiNativeProfilePosture`, `UiNativePhysicalSignalRuntime`, and
  `UiNativePhysicalProgressCorrelation`.
- **Headless host:** `OriginalOrderPlacement`.
- **Platform Pulse:** native input ingress, terminal posture, visual identity
  execution, its disabled state, and the Query owner state.

These types keep `Default`, and none of them is a lifecycle state:

- `WorthUiHeadlessMountedResourceCache`, which the requirement 2 review kept.
  Its default binds nothing.
- `UiScrollGroupBind`, a bind counter. Its default is the bind before any
  groups are bound.
- `UiMountedMotionSampler`, an owner. Its default holds an empty track table.

## Compile-fail proofs

Sealed truth is crate-private, so the public compile-contract fixture cannot
reach it. The runtime carries its probes beside the owners they probe instead:

- `compile_probes.rs` at the crate root;
- `scroll_motion_groups/compile_probe.rs`;
- `motion_sampling/sampling/compile_probe.rs`.

Each case is announced by `// expect: compiles` or `// expect: E0000` and gated
by `cfg(worth_ui_compile_probe = "name")`. The modules exist only under
`cfg(worth_ui_compile_probe)`, which the workspace declares in `check-cfg`.

| Requirement | Forbidden move | Refused case | Error | Valid counterpart |
| --- | --- | --- | --- | --- |
| 1 | apply a published map to a displayed rect | `published-map-displayed` | E0308 | `published-map-published` |
| 1 | chain a published Scroll group translation with a displayed one | `group-move-displayed` | E0308 | `group-move-published` |
| 3 | claim `OnScreen` from a displayed basis instead of a witness | `presented-by-basis` | E0308 | `presented-by-witness` |
| 3 | mint a witness by its literal | `witness-literal` | E0451 | `witness-read` |
| 4 | edit the Motion track table without recording the edit | `track-table-direct` | E0616 | `track-table-install` |

A Motion sampling becomes presented, which sets its tracks `OnScreen`, only
through `into_presented`, and that takes a witness. The track table's fields
are private, and `install`, `retire`, and the other edits each record
themselves.

`scripts/ci/run_worth_ui_compile_probes.py` runs the probes, and the
compile-contracts lane of the Worth UI test runner calls it:

- Every `compiles` case is built in one `cargo rustc --profile check` session.
  That session must emit no error and no warning.
- Each refused case is built alone. It must fail, and every error it raises
  must carry the expected code, with its primary span inside that case's lines.
- Each probe source must hold both a compiling case and a refused one, and case
  names are unique.

Refused cases are built one at a time because rustc stops before privacy
checking when type checking fails. When the refused cases were first built in
one session, the E0308 cases hid `witness-literal`'s E0451 completely.

Boundary-check reads `worth_ui_compile_probe`, bare or with a value, as never
holding in production, the way it reads `test`. Before that, BC7005 reported
`witness-literal` as a construction outside the owner. The probe exists to
make exactly that move, and the compiler refuses it.

## Mutation runs

Each mutation below was added, run, and reverted:

- A `fn probe` calling `UiPublishedRect::from_committed_components` in Portal
  `planning.rs`: boundary-check reports exactly one BC7005, naming `probe` and
  the constructor, and exits 1.
- `#![allow(clippy::disallowed_methods)]` at the top of `planning.rs`: exactly
  one BC7005, for the module, exit 1.
- `disallowed_methods = "allow"` in the Worth UI `[workspace.lints.clippy]`:
  one BC7005 for each of the five covered crates that inherit it, exit 1.
- `derive(Default)` on Pulse `PlatformPulseTerminalPosture`: exactly one BC7006,
  exit 1.
- The track table's `tracks` field made `pub(super)`: the probe runner reports
  `track-table-direct: compiled, but must fail with E0616` and exits 1.
- Before the `expect` attributes were written, Clippy rejected every host
  acknowledgement and the Portal entrance translation, each at its call.
