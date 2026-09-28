# Milestone 3.14.1: Aspect-Native Host Platform and egui Retirement

## Status and Placement

Status: implementation specification for the ten-phase slice immediately
after Milestone 3.14 and before Milestone 3.15. Phases 1-5 are implemented and
historically reviewed; Phases 6-7 were implemented and independently reviewed
on 2026-08-23. `worth-ui-body-default-v1` remains immutable Phase 1-2
predecessor evidence; the qualified `worth-ui-global-text-v2` profile is the
Phase 4 authority consumed by Phase 5. A different asset, dependency, capacity,
signature, or platform policy is a specification change rather than
implementation discretion. Phase 5 includes atlas/physical-Signal behavior,
the Query async-presentation lifecycle, the 4×8 locality suite, native
reconstruction/pixels, and terminal-zero resource behavior. Phase 8 was
implemented and independently reviewed on 2026-08-24. The corrected Phase 9
implementation record and its frozen source snapshot were independently
certified and closed on 2026-08-24. The corrected Phase 10 native-cutover
implementation and its frozen source snapshot were independently certified and
closed on 2026-08-24, completing Milestone 3.14.1.

The detailed Phase 5 raster, atlas, native text-presentation, courtroom,
cost, topology, and documentation contract is governed by
[`milestone-3.14.1-phase-5.md`](milestone-3.14.1-phase-5.md). It is a
normative subordinate specification, not a separate roadmap milestone. This
parent retains the overall milestone contract, phase order, and successor
handoff.

Historical QA artifacts, including `milestone-3.14.1-proof-ledger.csv`, are
frozen records. They are not current implementation or release authority, are
not updated or reopened, and ledger-only failures do not block work. Current
phase completion follows
[the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
[testing laws](../../docs/coding-guidelines/testing_laws.md): the specification states
QA considerations in prose, relevant tests and repository checks run against
the current commit, and code review decides whether the evidence is adequate.

Milestones 3.10-3.14 established one real application lifecycle, exact mounted
identity, observation-driven rebind, projected product data, and
presentation-bound interaction and intent. This milestone replaces the
interim `egui`/`eframe` mechanics beneath those contracts before portal,
focus, motion, appearance, and later native services would otherwise be built
against a host scheduled for deletion.

The milestone is deliberately broad in native mechanics and narrow in product
meaning. It does not redesign authored meaning, broad Query migration, targeting,
intent admission, consequence publication, or appearance semantics. Phase 5
does add one narrow Query-owned async presentation result through the existing
WORTH UI Query binding boundary.

Treat this as the largest single 3.x platform-migration slice. Its ten phases
are implementation and authority gates, not equal units of effort; implementation
planning must budget roughly one quarter to one third of the milestone for
Phase 1. Earlier line-count estimates are not scope limits or acceptance
criteria, and they do not license compressed trust handoffs, deferred
qualification, or reduced proof.

## Goal and Central Claim

The same mounted frame and observation contracts drive a Worth-owned native
platform. A native mechanics crate whose only WUI dependency is the host
contract owns the event loop, window, device, surface, retained draw list,
input translation, capture, and shutdown. A host-neutral Worth text-mechanics
crate owns qualified fonts, Unicode analysis, shaping, line layout,
measurement, cluster geometry, and glyph rasterization for both headless and
native consumers; the native host alone owns GPU atlases and presentation.
A higher application-platform crate composes those mechanics with the public
Worth UI lifecycle. Runtime issues exact presentation work and total paint
order, and the host lowers it into attributable pixels without rediscovering
UI meaning. After parity is proved, every `egui` and `eframe` dependency and
ordinary path is deleted.

The closure claim is:

```text
admitted runtime change
-> owner-issued Initial | Delta | Reconstruction | Unchanged presentation work
-> receipt-keyed retained native commands
-> canonical Unicode analysis, bidi/line layout, complex shaping,
   deterministic font fallback, and original-range cluster geometry
-> bounded alpha/color glyph raster, emoji, atlas, and GPU effects
-> presented-source pixels plus compositor-visible client pixels,
   or typed non-success at their owning boundary
-> existing publication, observation, capture, and recovery contracts
```

Every native draw command remains causally bound to mounted node receipt,
frame, surface, and binding generation. Derived draw-list, atlas, backing
texture, and GPU state can be destroyed and reconstructed from the current
mounted projection without changing application truth.

### Explicit non-goals

- new authored layout, appearance, component, portal, focus, motion, command,
  selection, or accessibility semantics;
- a Worth-authored shaping algorithm, IME composition engine or service,
  compositor, or GPU API; WORTH owns text orchestration, qualification, font
  collection and layout authority while consuming pinned Unicode, shaping,
  and raster dependencies, and host translation of OS IME observations into
  the inherited 3.14 contract remains required;
- ambient system-font discovery or renderer-selected fallback in the ordinary
  lane; the framework instead admits repository-pinned and explicitly
  application-bundled, content-addressed font collections;
- rich-text authoring, text-editor selection/focus semantics, spell checking,
  hyphenation dictionaries, or vertical writing; Phase 4 must nevertheless
  produce the grapheme, cluster, visual-order, caret-stop, selection-geometry,
  and writing-mode-ready artifacts those successor semantics consume;
- general vector tessellation, arbitrary paths, images, icons, canvas, or
  realtime rendering;
- platforms beyond the currently executable-certified Windows lane;
- a second application composition root, binary, executable-world harness,
  or permanent host-selection mode.

## Current Boundary and Required Closure

| Current fact | Defect exposed by native ownership | Required closure |
| --- | --- | --- |
| `UiMountedFrameConsumptionView` carries a complete projection. | A host can rescan or diff the whole projection and become a second damage planner. | Runtime/mounting issues typed initial, exact delta, or unchanged presentation work. |
| Paint mechanics carry `layer_semantic_order`. | Equal-layer commands have no host-independent total order, while dense replacement ordinals would amplify local insertion. | Mounting issues stable order identities plus an initial total sequence or exact relative order edits. |
| `BodyDefault` records size, weight, clipping, and baseline posture. | The Phase 1 profile proves only printable Basic Latin, one left-to-right run, no wrapping, no fallback, and grayscale glyphs; that is a migration seed, not framework-grade text. | A host-neutral, versioned Unicode text platform freezes deterministic font collection and fallback, complex shaping, bidirectional and line layout, grapheme/cluster mapping, color emoji, shared measurement/rendering artifacts, bounded caches/atlases, and typed coverage disposition before native text ships. |
| Platform Pulse implements `eframe::App` and requests continuous repaint. | Product code owns platform scheduling and unchanged turns still reach native presentation. | The Worth host owns the event loop and wakes only for typed OS, application-readiness, deadline, capture, or recovery causes. |
| egui dependencies exist outside `worth-ui-host-egui`. | Deleting one adapter leaves a false retirement claim. | Certification and Pulse migrate; the isolated egui-era theme/component crates are removed rather than prematurely ported. |

The required host-contract additions refine transport and proof carriage; they
do not change mounted semantic meaning. If implementation discovers that
native input cannot preserve a 3.14 interaction distinction, mounted rows do
not determine deterministic pixels, or measurement cannot preserve the
declared text profile, that is a defect in the current change. A lossy shim or
adapter-local policy is not an allowed migration answer.

## Pre-Implementation Specification-Closure Gate

This gate is specification work, not Phase 1 implementation. No production
edit, dependency addition, font ingestion, protocol revision, or implementation
plan may begin until both records below are committed into
this specification and its continuing authoritative documents. A planning
document may investigate candidates, but it cannot turn an unresolved choice
into implementation discretion.

### Qualified native and text profile record

The governing design must name one exact redistributable `BodyDefault` font
asset and license, asset digest, normative supported code-point set, shaping
and rasterization dependencies and versions, normalization, hinting/subpixel
posture, size/weight/baseline metrics, DPI rules, atlas capacities, and the
unsupported-text compatibility outcome. The support set is a product contract,
not whatever glyphs a dependency or operating system happens to find. It must
state explicitly whether every currently admitted Platform Pulse value is
supported and how arbitrary otherwise-valid 3.13 Unicode projection text
stops before presentation effects while preserving predecessor publication.

The same record must freeze the exact Windows event/window backend, graphics
backend, adapter-selection rule, surface and retained-target formats,
present/alpha modes, blend equations, antialiasing posture, shader inputs,
coordinate and DPI rounding, clear/baseline behavior, dependency versions,
bounded queue/resource capacities, and executable qualification environment.
Repository manifests, assets, profile types, documentation, pixel manifests,
and relevant test fixtures must all derive from this record. Phase 1 consumes
it and proves it; Phase 1 does not choose or amend it.

The closed text record is `worth-ui-body-default-v1`:

| Field | Exact qualified value |
| --- | --- |
| Asset | Noto Sans v2.015, `NotoSans/hinted/ttf/NotoSans-Regular.ttf`, 621,572 bytes |
| Asset SHA-256 | `478c558ea716033cd60c03438f628dfa75694dcf6b5f6d505a2f05fd2b4f3823` |
| Upstream archive | official `NotoSans-v2.015.zip`, 117,491,253 bytes, SHA-256 `0c34df072a3fa7efbb7cbf34950e1f971a4447cffe365d3a359e2d4089b958f5`, release commit `c4a321e123e4d4ff315f57f4e0adf294fe3a95be` |
| License | SIL Open Font License 1.1, repository copy byte-identical to upstream `OFL.txt`, 4,396 bytes, SHA-256 `cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a` |
| Normative support | exactly printable Basic Latin `U+0020..U+007E`; glyphs present in the asset outside this set are not admitted |
| Pulse coverage | every currently checked-in Platform Pulse projected text value is inside the normative set |
| Shaping | `rustybuzz = "=0.20.1"`; horizontal left-to-right, one run, no language or script fallback, no normalization |
| Rasterization | `swash = "=0.2.10"`; hinted outlines, grayscale coverage, no LCD/subpixel or color glyph rendering |
| Face/profile | regular face, weight 400, `14_000` millipoints, alphabetic baseline, no wrapping, clip overflow |
| DPI | physical size is logical size multiplied by event-time scale factor; the scale basis is part of profile generation |
| Rounding | glyph origins use round-to-nearest ties-to-even; rectangle and clip minima floor, maxima ceil; clips are half-open |
| Atlas | four `1024 x 1024` `R8Unorm` pages, at most 4,096 entries and 4 MiB texels; one glyph bitmap is at most `256 x 256`; one staged upload is at most 1 MiB |
| Saturation | live retained glyphs are pinned; candidate admission denies before upload if the exact bounds cannot be met; no fallback, growth, or live eviction |
| Canonical qualification observation | asset, license, manifest, dependency, support-set, and capacity digests under `asset-license-profile-dependency-digest-v1` |

Otherwise-valid Unicode outside `U+0020..U+007E` yields the typed
`UiMountedTextPresentationDenial::UnsupportedCodePoint` before shaping,
rasterization, atlas mutation, upload, surface acquisition, or presentation.
The semantic projection and predecessor publication remain current. The
denial records the first unsupported scalar, its UTF-8 byte range, the mounted
receipt and generation set, and `worth-ui-body-default-v1`; it never replaces
the value with tofu, omission, transliteration, or a system font.

#### Framework-grade text correction and Phase 4 qualification gate

`worth-ui-body-default-v1` remains the exact Phase 1-2 migration record and its
historical qualification evidence is not rewritten. It is not the Phase 4 text
destination. The product requirement for serious framework-grade text
supersedes the earlier Basic-Latin-only Phase 4 plan with
`worth-ui-global-text-v2`.

No Phase 4 production edit or implementation plan may begin until one
canonical `worth-ui-global-text-v2` manifest commits the exact asset list,
face indexes, licenses, byte lengths and SHA-256 digests; exact Unicode data,
shaping, segmentation, line-breaking, and raster dependency versions and
features; generated coverage and fallback indexes; conformance-data digests;
and every capacity below. The choice of a smaller incidental font set, an OS
fallback, or whatever behavior a dependency happens to expose is not
implementation discretion.

The normative contract for that manifest is:

| Field | Required `worth-ui-global-text-v2` contract |
| --- | --- |
| Unicode version | exactly Unicode 17.0 data and conformance tests; upgrading Unicode is a profile/protocol migration rather than an invisible dependency update |
| Text preservation | source UTF-8 bytes and scalar order remain authoritative; analysis or shaping may derive normalized views only when required by the pinned engine, and every grapheme, shaping cluster, glyph, caret stop, and diagnostic maps back to exact original UTF-8 ranges |
| Segmentation | extended grapheme clusters and default word boundaries conform to Unicode Standard Annex #29; line-break opportunities conform to UAX #14, with pinned locale-aware dictionary segmentation for scripts whose ordinary wrapping cannot be honest from space boundaries alone |
| Directionality | full horizontal Unicode Bidirectional Algorithm conformance from UAX #9, including isolates, paired brackets, numbers, neutral resolution, explicit controls, and line-by-line visual reordering; the source remains logical order |
| Run formation | paragraphs split into exhaustive runs by bidi level, script, explicit BCP-47 language or `und`, style, selected face, variation axes, and feature set before shaping; no code-point, font-family, or arrival-order tie break may replace that sequence |
| Shaping | HarfBuzz-compatible complex shaping through one pinned dependency and configuration for Arabic, Hebrew, Indic, Southeast Asian, Tibetan, Hangul, combining-mark, ligature, and joining behavior; cluster level preserves monotone grapheme mapping and never permits a line, caret, selection, fallback, or ellipsis boundary inside an indivisible shaping cluster |
| Default font collection | a repository-pinned Noto release catalog, including the required script faces, CJK coverage, symbols, math, and a Unicode-17-compatible Noto Color Emoji face; the final manifest enumerates every consumed face rather than naming an archive or family as a wildcard |
| Application fonts | an application may register one or more immutable content-addressed packs from application-owned OpenType `TTF`, `OTF`, `TTC`, or `OTC` bytes before layout effects; each admitted face receives an application-pack-scoped content identity, so identical pack bytes and metadata reconstruct identically across application instances while same-name families in different packs never alias; admission validates the table directory, face index, family/style metadata, weight, width, slant, variable axes, features, coverage, license record, and limits before returning a font-collection generation; fallback position is authored only by each span's explicit ordered family stack, never by pack registration order; `WOFF`/`WOFF2`, ambient installation state, and path/name lookup outside the admitted bytes are excluded until separately qualified |
| Authored family selection | every text style span carries an ordered, nonempty `UiFontFamilyStack` of qualified application or profile family identities plus an explicit weight, width, slant, variation-coordinate, and OpenType-feature request; deterministic matching selects a face from that stack before complete-cluster fallback, and neither a renderer nor a platform adapter may replace it with a local default |
| Application font lifecycle | admitted packs are immutable within one `UiFontCollectionGeneration`; adding, replacing, or removing a pack creates a successor generation, while already-published layouts pin the exact predecessor face bytes and remain valid until their owners release them; family-name collisions never merge authority; a paragraph is reanalyzed exactly when its resolved layout-affecting input changes, including family stack, selected face, weight, width, slant, variation coordinates, OpenType features, language, text, or constraints, while unchanged siblings remain zero-work and a color-value-only change with unchanged paint-span boundaries remains the explicit no-reshape exception |
| Fallback | deterministic cluster-level fallback over the exact admitted collection and generated coverage index; a face must support the complete grapheme/shaping cluster before selection, and fallback cannot split combining sequences, Indic syllables, or emoji sequences |
| Missing coverage | valid UTF-8 never aborts an otherwise admissible presentation merely because a face lacks a glyph; a pinned Last Resort face renders one attributable missing-cluster glyph and records `UiTextCoverageDisposition::MissingCluster` with the original range and attempted collection generation |
| Emoji | all Unicode 17.0 RGI emoji sequences are qualified, including text/emoji variation selectors, keycaps, flags, tag sequences, skin-tone modifiers, gendered and family ZWJ sequences; fallback and line layout treat each qualified sequence as one grapheme/cluster, and rasterization preserves every required color layer or color bitmap |
| Layout | hard line breaks, preserved whitespace, tabs with explicit tab stops, no-wrap, Unicode word-wrap and grapheme-wrap, start/center/end alignment, explicit line height, letter/word spacing, maximum lines, clip, and cluster-safe ellipsis are typed constraints decided before mechanics execution |
| Span foreground | authored foreground is an exact original-range `UiTextForegroundSpan` carrying logical straight RGBA and a paint-span identity; its boundaries participate in run itemization so no glyph crosses two foreground spans, but the RGBA value is excluded from the layout cache key, so a color-only successor with unchanged boundaries reuses analysis, fallback, shaping, line fitting, metrics, and interaction geometry and changes only text paint commands and damage |
| Layout result | one immutable reconstructible `UiQualifiedTextLayout` carries original-range mapping, paragraph and line records, logical and visual runs, selected faces, glyph positions/advances, ink and logical bounds, baselines, break and overflow decisions, coverage dispositions, grapheme-safe caret stops, selection rectangles, profile/font/locale/direction/width/text-scale generations, and exact work counters |
| Measurement/rendering identity | intrinsic measurement, baseline metrics, hit testing, selection geometry, glyph rasterization, and native rendering consume the same qualified layout artifact or its exact identity; no adapter may reshape, refallback, rebreak, or recompute metrics independently |
| Rasterization | grayscale outline glyphs and color outline/bitmap glyphs are distinct typed raster outputs; source order, palette, hinting, antialiasing, fractional-origin quantization, and premultiplication are manifest fields; emoji color is not replaced by the surrounding text color |
| DPI and text scaling | logical layout is keyed by logical constraints and a distinct framework text-scale generation; a pure DPI change replaces raster/atlas generations without changing logical breaks, while a text-scale or width change creates a new layout generation before presentation |
| Bounds | at most 4,096 retained paragraphs, 8 MiB retained UTF-8, 65,536 UTF-8 bytes per paragraph, 262,144 glyphs, 262,144 grapheme/cluster records, 65,536 line records, and 32 style/fallback runs per paragraph; admission denies before shaping when exact declared bounds cannot be satisfied |
| Raster/atlas bounds | four `1024 x 1024` `R8Unorm` alpha pages plus two `2048 x 2048` `Rgba8UnormSrgb` color pages, at most 8,192 total entries, a `512 x 512` maximum glyph image, 36 MiB total atlas texels, and 8 MiB maximum staged upload bytes |
| Saturation | retained layouts pin exact alpha/color glyph entries; complete candidate admission and deterministic unpinned eviction precede raster/upload effects; no live eviction, unbounded growth, silent quality reduction, or color-to-monochrome fallback is allowed |

The default profile must render ordinary multilingual application content and
emoji without caller configuration. `BodyDefault` becomes a semantic style
selection inside the v2 font collection, not the name of one physical face.
The qualified font resolver may select different pinned faces across runs
while preserving one semantic style, layout identity, and mounted receipt.
Custom rich-text authoring remains later product meaning, but the mechanics
input and layout artifact are span-capable from their first v2 revision so
adding rich text does not require replacing the shaping, fallback, line,
measurement, hit-testing, or atlas architecture.
Basic per-span font, size, slant, feature, and foreground appearance is part of
that v2 mechanics contract rather than deferred rich-text product semantics.
Foreground authority remains outside `worth-ui-text`: text mechanics owns the
stable paint-span boundaries and cluster mapping, while mounted appearance
owns the RGBA value consumed by headless and native glyph-run presentation.

The v1 `UnsupportedCodePoint` denial remains valid only for the already-closed
Phase 1-2 seed profile. Under v2, unsupported formatting, malformed admitted
font bytes, impossible original-range mapping, stale layout/profile affinity,
and capacity exhaustion retain distinct typed denials before effects; ordinary
missing font coverage is an attributable Last Resort disposition, not a
discarded value or a whole-frame denial.

The closed native record is `worth-ui-windows-dx12-v1`:

| Field | Exact qualified value |
| --- | --- |
| Executable lane | Windows 11 x86-64 with a D3D12-capable display adapter and composition-enabled desktop; other platforms return typed profile denial before effects |
| Window/event dependency | `winit = "=0.30.13"` |
| Graphics dependency | `wgpu = "=29.0.4"` with only the DirectX 12 backend admitted; `pollster = "=0.4.0"` for bounded preparation joins |
| Compiled dependency features | `winit`: defaults off, exactly `rwh_06`; `wgpu`: defaults off, exactly `std`, `parking_lot`, `dx12`, and `wgsl` |
| Device admission | runtime backend mask exactly `Backends::DX12`, empty optional device features, and `wgpu 29.0.4 Limits::downlevel_defaults().using_resolution(adapter.limits())`; the downlevel baseline is fixed while only the selected qualified adapter's physical texture-resolution ceilings are retained so the 16,384 window bound is realizable |
| Adapter selection | enumerate surface-compatible DX12 adapters satisfying the exact required limits; reject CPU/Other; sort Discrete, Integrated, Virtual, then vendor ID, device ID, name, and driver-info bytes; consume the first |
| Surface/target | wgpu DX12 `DxgiFromVisual` DirectComposition presentation, `Bgra8UnormSrgb` surface, and `Rgba8UnormSrgb` retained presentation target |
| Presentation | FIFO present mode and premultiplied composite alpha; unsupported exact modes deny rather than fall back |
| Shader/blend | input is logical straight RGBA; shader premultiplies RGB by alpha; color and alpha use source `One`, destination `OneMinusSrcAlpha`, operation `Add` |
| Antialiasing | sample count one; filled rectangles have no renderer-selected edge antialiasing; text uses the qualified grayscale glyph coverage |
| Coordinates | event-time logical coordinates and scale generation; physical minima floor and maxima ceil; half-open scissor and clip bounds |
| Baseline | canonical transparent `[0, 0, 0, 0]`, available only through a same-surface/binding/profile `UiMountedSurfaceBaselineReceipt` |
| Window/surface | one window and one surface in 3.14.1; transparent client background; initial logical size is the application profile size |
| Work bounds | 4,096 retained commands, 2,048 rectangle commands, 2,048 text commands, 4,096 damage regions, 4,096 order edits, and 1 MiB admitted text bytes |
| Scheduling bounds | 32 registered resource owners, eight readiness owners, 64 coalesced causes per owner, eight total ready owner slots, two presentation slots, four capture/readback slots |
| Readback bound | 16 MiB total framework-owned mapped or pending readback bytes; a native GPU submission/readback wait is bounded to 5,000 ms |
| Qualification observations | OS build; adapter name/vendor/device/driver; observed scale factors; required formats, modes, alpha, limits, and DX12 surface compatibility |
| Independent Windows observation | retained-target readback remains distinct from the existing Pulse executable-world `xcap = 0.9.7` WGC client capture, `winsafe = 0.0.28` window binding, and `uiautomation = 0.25.0` input/close observation |

Profile preparation records the operating-system build, selected adapter
identity, vendor/device IDs, driver information, observed scale factors, and
support for every exact required mode as qualification observations. Those
observations cannot change selection policy or silently widen the profile.
The profile identity is the SHA-256 of the canonical UTF-8 manifest containing
the table values above; Phase 1 checks in that manifest and focused tests verify
its digest wherever the profile is consumed.

The committed canonical manifest digests are text
`6f140249866e6815e9284fe1c8c959a8bb1b8cab252cfbe8c7c397f9a7eb9b01`
and native
`1c937a22f42660267480a055e48256b25decf0c4cd5d4d7b493e5df034c6c65b`.
These successor digests deliberately replace the earlier incomplete
manifest identities: every normative field above, including compiled feature
posture, device-limits basis, saturation, and observation boundary, is now
inside the canonical bytes rather than inferred from hard-coded production or
test constants.

That first text digest identifies `worth-ui-body-default-v1` only. The
canonical `worth-ui-global-text-v2` candidate now has manifest digest
`cec6005c5baef6d69ada9c30c02ced25b0f253f80c012784fe925e307935c3f2`.
Its exact 30-face catalog, Unicode 17 source and conformance data, dependency
pins, generated coverage/fallback indexes, capacities, licenses, and artifact
inventory are repository-owned and adversarially tested. Phase 4 production
may consume the candidate only after the manifest and its referenced assets,
licenses, data, indexes, dependency posture, and digest pass focused
qualification checks and code review. Phase 4 may never reuse the v1 digest or
mint a provisional replacement identity.

### Public native-application authority record

The governing design must freeze these named responsibilities before
implementation planning:

- `UiNativePlatformBindingGrant`: a platform-issued, move-only grant whose
  private authority admits exactly one application to exactly one prepared
  native platform; it is neither a host adapter nor a serializable identity;
- `UiNativeApplicationPreparation`: an effect-free, move-only preparation
  scope carrying only a host-neutral public `WorthUiApplicationBuilder` and
  the private platform-binding grant; subsystem workers do not exist yet;
- `UiNativeApplicationDefinition`: the only public product contract accepted
  by `UiPreparedNativePlatform::run`; it consumes the preparation scope and
  can return only a complete prepared application or a typed preparation
  denial;
- `UiPreparedNativeApplication`: the sealed result containing one frozen
  `WorthUiHostNeutralApp` and its private affine platform binding; Phase 2
  consumes this result to create the real application driver and its bounded
  subsystem owners; and
- `UiNativeApplicationPreparationDenial`: the before-native-effects outcome
  produced only by consuming the sealed effect-free preparation scope. Phase 1
  exposes no resource, worker, readiness, cleanup-census, or event-loop-client
  type that this scope could have opened or published.

The final public signatures make construction, consumption, and denial
compiler-visible. Product preparation cannot register or open source, Query,
intent, inspection, or readiness workers: those owners require the frozen
application/runtime authority that does not exist until preparation succeeds.
Phase 2 creates them inside the private application-driver progression. Product
code cannot implement the mechanics event-loop client, extract or replace the
host grant, submit raw wakes, or retain a second application/publication lane.

The closed public progression is:

```rust
pub trait UiNativeApplicationDefinition: Sized {
    fn prepare(
        self,
        preparation: UiNativeApplicationPreparation,
    ) -> UiNativeApplicationPreparationOutcome;
}

#[must_use]
pub enum UiNativeApplicationPreparationOutcome {
    Prepared(UiPreparedNativeApplication),
    Denied(UiNativeApplicationPreparationDenial),
}

impl UiPreparedNativePlatform {
    pub fn run<A: UiNativeApplicationDefinition>(
        self,
        application: A,
    ) -> UiNativePlatformOutcome;
}
```

`UiNativePlatformBindingGrant`, `UiNativeApplicationPreparation`,
`UiPreparedNativeApplication`, and both enum payloads have private fields and
no `Clone`, `Copy`, serialization, public constructor, or parts conversion.
The prepared platform privately issues one grant bound to its native-profile
identity, preparation identity, and single application slot. Any unsuccessful
preparation consumes that slot; a retry requires a newly prepared platform.

### Enforceability correction for the Phase 1-2 implementation

Rust crate privacy cannot distinguish one blessed sibling crate from another
downstream caller. This milestone therefore does not treat sibling-crate
identity, `#[doc(hidden)]`, a Cargo feature, or an unexported constructor in a
different crate as an authority boundary. The enforceable native activation
boundary is one fixed concrete transition: it consumes the move-only
host-neutral application, the private application-slot grant, and the exact
qualified native mechanics bundle. The transition and grant issuer live in
the same `worth-ui-runtime` privacy boundary as host-neutral application
construction; `worth-ui-native-platform` is the public facade over that gate.
It exposes no generic adapter parameter and cannot bind a caller-defined host.
Supported application crates reach it only through `WorthUiNativePlatform`;
repository topology rejects direct product dependencies on runtime and native
mechanics. The temporary egui and certification lanes use equivalently fixed
transitions and no feature-gated issuer.

The application-slot grant remains private to runtime-owned native-platform composition and
prevents reuse or replacement inside the public progression. It is not passed
across crates as a supposed friend token. A low-level integration function
that must be Rust-public for host mechanics composition is not itself the
application authority: it cannot receive or bind a host-neutral Worth
application. Product dependency topology prevents importing that mechanics
surface, while the actual application-binding transition remains crate-private.

Likewise, `worth-ui-host-contract` may expose inert mechanics inputs and
borrowed views needed by host consumers and their focused tests. Those values
are not authoritative presentation work. Only runtime's private lease can seal
them into the owning work envelope, admit them to an active host session, or
mint a completion token. Compiler evidence attacks the real sealing and
admission operations; it does not claim that inert data constructors are
private merely because they cannot open an operational door.

Retained test artifacts are reproducibility and failure-diagnostic records, not
unforgeable authorities. A repository file can always be hand-authored by
someone able to edit the repository. Acceptance therefore comes from relevant
tests and repository checks on the current commit plus code review of the
actual implementation and test oracles. Tests that share an expensive native
world should reuse that world within one run; they do not need receipt,
portfolio, nonce, digest, or publication machinery to prove that reuse.
Cached artifacts may speed a diagnostic rerun, but CI and reviewers judge the
current commit and must not infer a pass from retained bytes alone.

`UiNativeApplicationPreparation` owns the host-neutral
`WorthUiApplicationBuilder` and no live subsystem resource. Its public
`builder(&mut self)` returns only a
borrowing `UiNativeApplicationBuilder<'_>`. That borrow mirrors the existing
concrete Worth application registration operations while replacing the owned
builder internally; it cannot freeze, extract, replace, or bind a host. The
scope exposes no worker, readiness, callback, event-loop client, wake proxy,
or raw host-adapter registration. `complete(self)` freezes the internal builder
into a host-neutral application, consumes the binding grant, and is the only
constructor of `UiPreparedNativeApplication`. `deny(self, cause)` proves that
no subsystem or event-loop client could exist and is the only constructor of
`UiNativeApplicationPreparationDenial`. A constant zero cleanup report is not
evidence and is absent; Phase 2 introduces a real census only alongside real
resource owners and consuming stop/join transitions.

Phase 2 consumes `UiPreparedNativeApplication` into a private
`UiNativeApplicationDriverPreparation`. Only that phase or a later native
product-integration phase may ask the real filesystem, Query, intent, and
inspection subsystem boundaries for sealed stopped owners. The first vertical
presentation registers only the application-driver, event-wake, window,
graphics, presentation, and readback owners that it actually opens; it does
not invent placeholder filesystem, Query, intent, or inspection workers.
Whenever a later phase opens one of those real subsystem owners, it must first
register it in the same 32-entry resource registry before activating its
eight-slot readiness capability. The driver retains reverse stop/join and
incomplete-cleanup authority for every owner it has actually opened. A generic
counter thread, owner-kind enum, or test-only refusal posture cannot stand in
for a real subsystem owner.

Phase 1 creates `worth-ui-host-native` as the owner of these checked-in
profiles, capacities, sealed inert mechanics contracts, and trust manifests,
and creates `worth-ui-native-platform` as the effect-free public preparation
and binding owner. Phase 2 activates event-loop, window, surface, and device
effects in those already-existing owners; it does not create placeholder
crates or choose their authority topology. During coexistence the application
platform privately binds either the revision-4 egui mechanics or future native
mechanics through an affine `UiHostMigrationGrant`. The grant is not public
product configuration; Phase 9 admits it only to the existing Pulse runner,
and Phase 10 deletes it with the egui path. Public
`WorthUiApplicationBuilder::with_host(...)` is removed in Phase 1; headless
certification and migration bindings use sealed framework-owned admission.

## Decisive Product Courtroom: `HP-01`

### Real world

Run `PulseNativeParityWorld` through the existing `worth-ui-platform-pulse`
binary, checked-in `main.wui`, public application composition root, OS
filesystem and Query watchers, native window, public intent provider, visual
snapshot contract, lifecycle stream, and existing executable-world target on
executable-certified Windows.

During coexistence, a migration-only launch grant selects either the egui host
or Worth native host inside the same binary. It is not a product setting and
is deleted at cutover. The external runner applies the same causal actions and
adjudicates both processes through OS-delivered input, process lifecycle,
actual client-area pixels, and the versioned Pulse observation stream.

### Hostile sequence

1. Launch with no Query value and observe the pending frame, attributed blue
   background, yellow identity target, semantic text posture, and first-frame
   publication.
2. Publish the first current value, replace it with a longer mixed-direction
   multilingual value and then one value containing a qualified color-emoji
   ZWJ sequence. Prove layout identity, text/emoji pixels, original source
   ranges, and mounted correlation follow the existing Query path without
   changing Query truth or using host-selected fallback.
3. Exercise the 3.14 native activation, denied operability, confirmation,
   cancellation, completed product effect, and visible consequence actions by
   OS input against the exact presented frame.
4. Run the identity snapshot, point trace, overlay publication, surface
   readback, comparison, overlay clear, and explicit snapshot retirement.
5. Apply the green source edit, malformed source, exact predecessor
   preservation, restored blue source, incompatible Query schema, schema stop,
   and compatible recovery without replacing the window or application
   composition root.
6. Leave the application quiescent across an observed idle interval and issue
   an explicit unchanged mounted-frame attempt in the in-process companion
   world. Neither path may mutate the draw list, shape or rasterize glyphs,
   acquire a surface, encode commands, submit GPU work, or present.
7. Close the real native window while no product mutation is pending. Join all
   workers and require zero live window, surface, device, queue, backing
   texture, draw-list, GPU-buffer, atlas, readback, capture, watcher, Query,
   intent, and application resources.

### Required verdict

- Both hosts produce identical semantic, mounted receipt, frame, binding,
  interaction, intent, consequence, and lifecycle evidence for the same
  versioned causal-action manifest. The recorded migration grant is the only
  differing world input. Host-mechanical identities and timestamps are
  excluded only by a named parity contract, never by a generic normalization
  pass.
- Native filled-rectangle control points retain the established pixel values.
  Glyph-region expectations change exactly once. The rebaseline record names
  predecessor and successor pixels, font asset digest, text profile,
  dependency versions, DPI, surface/color posture, and external adjudication.
- Every nontransparent presented-source pixel contribution resolves through a
  retained command to its exact mounted receipt and generation set. Cleared
  uncovered source pixels resolve instead to the exact surface-issued
  transparent baseline receipt; no host-selected clear color or appearance
  value exists.
- A denied or indeterminate native effect preserves the prior semantic
  publication and returns the existing typed recovery posture; no renderer
  state is promoted into application truth.
- Unchanged work reports exact zero through every named native counter.
- After native parity closes, rerunning the courtroom is possible only through
  the native host because the migration selector, egui shell, adapter, and
  dependency edges no longer exist.

### Defects this courtroom must convict

The courtroom turns red if an implementation repaints continuously, diffs the
full projection in the host, loses command receipts during preparation,
retargets input to a newer frame, invents geometry or order, captures an
internal data structure instead of rendered pixels, silently changes font
fallback, publishes after an uncertain effect, leaks native resources, or
keeps egui as a fallback.

## Required Test Worlds

### `HP-02`: Delta and amplification courtroom (Phase 3 closure)

Phase 1 seeds `MountedPresentationWorld::maximum_overlap` through the public
application, production lowering/graph/mounting, runtime work issuer, and real
headless consumer with the 2,048-rectangle ceiling. Its independent manifest
and oracle prove the complete initial order, sparse/partial/full removal,
exact insertion restoration, logical damage, and an unchanged-zero turn.
Phase 3 uses two causally distinct extensions rather than pretending Phase 4
layout or Phase 5 native glyph rendering already exists. The native ordered-
pixel/damage/replay courtroom extends the governed world to the complete
2,048-filled-rectangle ceiling. A separate 4,096-command mixed-carrier world
crosses public lowering, mounting, runtime issuance, and the real headless
consumer with 2,048 filled rectangles and 2,048 inert semantic-text commands at
the revision-4 1 MiB text-byte ceiling. It replaces one text carrier, removes
one rectangle, inserts one rectangle while every other row remains identical,
and then submits an unchanged successor. The mixed world proves delta carriage,
retained indexing, total order, and slope only; it cannot claim Unicode layout,
glyph mechanics, native text pixels, or Phase 4's expanded text capacities.

Command mutation and order planning must consume work proportional to the
three changed commands plus explicitly affected order/damage entries, not all
retained commands. Physical damage replay additionally consumes exactly the
retained commands whose visible bounds intersect admitted damage, in total
paint order. Removing or moving a command clears its vacated region and
replays every intersecting predecessor command required to restore the
correct pixels; any uncovered remainder returns to the surface-issued
transparent baseline rather than an adapter default. A damage index narrows
that set without a retained-list scan;
lawful complete overlap may still require complete replay and must report it.

Named counters distinguish rows carried in the delta, draw-list and order
mutations, damage-index probes, intersecting and replayed commands, cleared,
rendered, and presented pixels, GPU writes, render passes, surface copies,
acquisitions, submissions, and presents. If the native surface requires a
full-area copy or present after local rendering, that physical amplification
is reported separately and is never relabeled local.

The Phase 1 seed's independent model adjudicates command retention, removal,
insertion restoration, logical damage, and total order without reconstructing
expected rows from the production transcript. Phase 3 adds native ordered-
pixel adjudication for the rectangle world and an independent retained-command/
order model for the mixed carrier world. Together they cover clipping,
clearing, overlap, equal-layer order, exact carrier mutation, and retained-state
slope without assigning text pixels to Phase 3. Their mutation controls remove
delta carriage, force a full scan, widen damage, omit vacated replay,
substitute the transparent baseline, change the total-order tie break, or
rescan an unchanged text carrier; each must fail for its named reason.

### `HP-03`: Text and atlas courtroom

Use `MountedPresentationWorld::qualified_text` through the ordinary public
application, Query admission, mounting, runtime, headless, native, and
Windows/WGPU boundaries. It combines the exact qualified text profile,
application-owned fonts, multilingual and complex layout, the full Unicode 17
RGI set, mixed original-range paint spans, alpha and intrinsic-color raster,
separate bounded atlases, saturation, DPI/text-scale changes, failure,
destruction/reconstruction, unchanged-zero, and exact cleanup.

Unicode/layout fixtures, a pinned raster oracle, an independent atlas model,
headless transcripts, retained-target readback, and compositor-visible capture
must adjudicate the same mounted identities. Exhaustive internal raster
evidence and representative real pixels are both required. Each unique corpus,
model, and native execution runs once per source-state and claim digest; rows
share authenticated receipts and the final gate validates rather than reruns
the retained portfolio.

The exact real world, hostile sequence, decisions, typed outcomes, cost/resource
contract, mutants, and proof-economy requirements are owned by the
[Phase 5 subordinate specification](milestone-3.14.1-phase-5.md#decisive-product-courtroom-hp-03).

### `HP-04`: Native lifecycle courtroom

Drive the exhaustive schedule through `NativeLifecycleProtocolWorld`, then run
the environment-real subset through `WindowsNativeBoundaryWorld`: input before
the first completed presentation and while a successor is in flight; zero-
sized/minimized surfaces; resize and DPI changes around input; surface lost/
outdated/timeout posture; device loss; capture readback in flight; close with
application readiness queued; close during prepared upload; close after
submission but before completion; and recovery followed by ordinary
presentation. Cross close with one held 3.14 application attempt so platform
shutdown proves the inherited cancellation or settlement posture rather than
only native-resource disposal.

Against one exact runtime-owned local draft recipient, deliver Windows IME
preedit, composition commit, cancel, and native-range conversion, then repeat
with no recipient, stale presentation/input/text-profile affinity, over-
capacity text, and an unprovable range. The oracle distinguishes visible preedit, committed draft
text, semantic `edit-commit`, payload formation, and intent effects so no host
event can collapse those phases or manufacture focus authority.

Input coordinates use the DPI and presentation basis observed at event time.
Every before-effect stop, in-flight state, indeterminate effect, recovery
handle, reconstruction, and shutdown disposition is typed. Fault injection
proves protocol logic but does not claim a real GPU failure; required native
integration separately crosses the actual window, surface, device, and
readback boundary.

### `HP-05`: Retirement and topology courtroom

`HostRetirementTopologyWorld` uses workspace metadata, dependency inversion,
source inventory, public examples, boundary checks, and positive/negative
compile twins to prove:

- zero `egui`, `eframe`, `egui-wgpu`, and `egui_extras` dependency
  declarations or resolved edges across the repository root workspace, the
  Worth UI workspace, supported member manifests, and final lockfiles;
- no `WorthUiHostKind::Egui`, `WorthUiHostContract::egui`, egui adapter,
  eframe shell, migration selector, or compatibility facade;
- no vendor event, window, surface, font, or GPU types in the `worth-ui`
  product facade or host contract;
- host adapters cannot import runtime internals or mint mounted presentation
  work; and
- external callers cannot construct receiptless draw commands or promote
  capture, diagnostics, lifecycle, or host identities into authority.

## World Architecture and Fixture Provenance

A courtroom world is a typed, causally complete baseline plus one named
scenario delta. It is not a bag of constructors. World construction,
action, observation, and teardown return distinct typed outcomes, and every
valid identity, receipt, generation, resource, and authority handle comes from
the production owner or a narrow world compiler proven to establish the same
postconditions. Raw literals are reserved for invalid-input, protocol, pixel-
contract, and corruption claims where the literal is itself disputed.

The milestone uses four stateful worlds and one static proof world inside the
existing test targets. They do not create another product composition root,
binary, executable-world harness, Cargo target, or mutable universal fixture.

| Courtroom | Governing world | Honest proof boundary |
| --- | --- | --- |
| `HP-01` | `PulseNativeParityWorld` | real Cargo-built product process, public composition root, OS window/input/capture, product evidence, and explicit teardown |
| `HP-02` | `MountedPresentationWorld::maximum_overlap` | public Worth application through production lowering, graph, mounting, presentation-work issuance, native-host execution, and independent ordered-pixel model |
| `HP-03` | `MountedPresentationWorld::qualified_text` | the same mounted authority with the committed text profile, independent glyph/atlas models, external pixel contract, and derived-state destruction |
| `HP-04` | `NativeLifecycleProtocolWorld` plus `WindowsNativeBoundaryWorld` | deterministic production lifecycle protocol under injected external outcomes, separately joined to the real Windows event-loop/window/surface/device/readback boundary |
| `HP-05` | `HostRetirementTopologyWorld` | workspace metadata, supported build configurations, source/dependency inventories, compile twins, examples, and post-deletion executable reachability |

### `PulseNativeParityWorld`

This world extends the existing executable Platform Pulse machinery. It
installs byte-identical checked-in `.wui`, Query, and intent inputs into fresh
isolated roots for each host run and records one versioned causal-action
manifest containing every external edit, input action, observation checkpoint,
ordering edge, and deadline. The egui/native migration grant is
the only differing input, and the world records both grant identity and action-
manifest digest. Runs are sequential under the existing exclusive native-
desktop lease; they share no mutable installation or process state.

The external pixel oracle consumes a versioned, test-owned control-point
manifest containing literal logical coordinates, expected RGBA values and
tolerances, DPI/profile identity, and the qualified font digest. It may share
stable wire and identity types, but it may not import production geometry,
order, color-selection, shaping, rasterization, damage, or expected-image
functions. Phase 9 changes glyph-region expectations exactly once through an
explicitly reviewed rebaseline; native source-target readback and OS client capture remain
separate observations. Failure retains the action manifest, profile, source,
bounded lifecycle trace, both pixel observations where available, and complete
teardown disposition.

### `MountedPresentationWorld`

The mounted world compiler enters through the public Worth application facade,
production file- or Rust-authored lowering, graph construction, surface
registration, allocation, mounting, and runtime-owned presentation-work issuer.
It returns semantic handles such as `world.surface`, `world.initial`,
`world.text_row`, and `world.current_presentation`; it cannot expose raw
identity constructors, host command constructors, atlas entries, GPU offsets,
or a method that asks the host to invent a delta.

`maximum_overlap` compiles one immutable maximum-size baseline once and applies
isolated owner-issued deltas for sparse, partial-overlap, and complete-overlap
regimes. `qualified_text` uses the same authority path with the multilingual
and emoji corpus, repeated/new/missing clusters, mixed-direction paragraphs,
exact line/paragraph/alpha-atlas/color-atlas capacity boundaries, pure-DPI,
width, text-scale and font-collection generation replacement, and complete
derived-state reconstruction. The ordered-pixel oracle implements clipping,
clearing, overlap, and total order without calling production ordering, damage,
draw-list, or rendering code. Separate Unicode, shaping-fixture, layout, and
atlas oracles model their exact responsibilities without calling production
analysis, fallback, line fitting, hit testing, or eviction classifiers. Reuse
recompiles from immutable semantic baselines or reconstructs isolated derived
state; no proof-bearing authority or resource handle is cloned, and no scenario
shares mutable layout, draw-list, atlas, device, or counter fate.

### `NativeLifecycleProtocolWorld`

This world runs the production native lifecycle orchestrator against
contractual window, surface/graphics, and readback ports whose real
implementations own vendor calls. Scripted implementations inject only named
external observations from acquisition, encoding/submission, present-handoff,
and readback calls. Pre-effect preparation denials arise from public malformed,
stale, unsupported, or over-capacity inputs, never an injected lifecycle
verdict. Scripted ports cannot return the expected platform verdict, mint
presentation epochs, change application truth, or replace the orchestrator.
They exist because the external effects are real boundaries; there is no
`cfg(test)` lifecycle, alternate composition root, or validation weakening.

An independent exhaustive transition model consumes the same fault schedule
and predicts effect posture, predecessor retention, recovery authority,
resource census, and valid next actions without calling the production state
machine. This world proves protocol behavior only. It cannot claim a real GPU
failure, OS event, compositor result, or vendor cleanup.

### `WindowsNativeBoundaryWorld`

This world enters through `WorthUiNativePlatform` and the existing Platform
Pulse application in the one serialized native courtroom. Its Phase 2 seed
crosses the real Windows event loop, process-bound window, observed startup
DPI, surface/device acquisition, presentation-source readback,
compositor-visible capture, normal close, and residue checks. Resize,
minimize/restore, cross-monitor DPI, input, device loss, and held-attempt
schedules remain deterministic protocol evidence until their later real-world
phases; the Phase 2 startup seed must not be treated as evidence for those paths.

Only environment-qualified facts receive real-boundary credit. A single-
monitor runner proves its observed DPI basis but not a cross-monitor DPI
transition; injected DPI/device-loss schedules remain protocol evidence.
Conversely, a real successful surface/device/readback path cannot substitute
for the deterministic partial-effect matrix. The specification and tests state
which world supports each `HP-04` behavior.

### `HostRetirementTopologyWorld`

The retirement world reads real workspace metadata and source on the exact
post-cutover tree. It audits the repository-root and Worth UI workspace
manifests, default and all-feature workspace graphs, every supported Windows
target/profile, member and fixture manifests, binaries, examples, final
lockfiles, boundary rules, and source/doc inventories. It parses dependency
declarations as well as resolved metadata so an unused root-workspace entry
cannot survive merely because no current member selects it. Negative fixtures
whose purpose is to prove the prohibition are classified by exact fixture
identity and cannot enter a supported graph. Positive twins prove the native
and headless homes remain lawful; negative twins prove forbidden dependency
and authority directions. It cannot use a curated crate list,
default-feature-only metadata, ignored source roots, or absence from one binary
as proof of retirement.

Each courtroom carries at least one cheap mutation control in the ordinary
gate, and every listed mutant must fail for its own causal reason:

| Courtroom | Required mutants |
| --- | --- |
| `HP-01` | omit or reorder one causal action; substitute production-expected pixels for OS capture; reuse mutable installation state; retain the egui launch path after cutover |
| `HP-02` | remove owner delta carriage; force retained-list discovery; widen damage; break equal-layer order; omit vacated-pixel replay; substitute a host clear color for the surface baseline |
| `HP-03` | retain the Basic-Latin one-run path; shape before bidi/run segmentation; split grapheme, Indic, or emoji ZWJ clusters during fallback/wrapping/ellipsis; ignore emoji variation selectors or color layers; permit system fallback; diverge measurement/render layout identity; evict a pinned alpha/color glyph; retain stale width/text-scale/DPI state; rescan every retained paragraph; skip derived-state reconstruction |
| `HP-04` | admit an illegal transition; drop or duplicate a wake; promote indeterminate effect to success; collapse IME preedit or composition commit into semantic `edit-commit`; skip resource disposal; replace a claimed real boundary with a scripted result |
| `HP-05` | hide a retired edge behind a feature, example, fixture, lockfile entry, ignored source root, or curated metadata scope |

Every courtroom records the world identity and version, baseline, scenario,
authority provenance, production entry, independent oracle, fault boundary,
teardown result, and construction/execution cost needed to interpret the test.
A broken world stops as fixture failure and cannot satisfy a product denial
assertion.

## Product Decision Lock

### Presentation work is owner-issued and compiler-total

The host consumption contract exposes exactly one move-only presentation-work
variant:

```rust
pub enum UiMountedPresentationWork<'frame> {
    Initial(UiMountedPresentationSeed<'frame>),
    Delta(UiMountedPresentationDelta<'frame>),
    Reconstruction(UiMountedPresentationReconstruction<'frame>),
    Unchanged(UiMountedPresentationUnchanged),
}
```

Runtime/mounting is the only producer. `Initial` carries a complete admitted
surface projection and establishes the retained generation. `Delta` carries
exact predecessor/successor frame and binding affinity, command insertions,
replacements, removals, order changes, and logical damage issued from the
admitted mounting/rebind scope. `Unchanged` proves equivalence to the retained
generation and carries no projection rows.

`Reconstruction` is a complete, cold, owner-issued projection for a current
mounted successor whose per-surface derived host state is absent or was
explicitly discarded. It carries exact predecessor/successor affinity but is
neither an initial surface generation nor a delta. The host rebuilds command,
order, damage, and presentation state from this envelope; it cannot reuse an
intact cache and call the repaint a reconstruction.

Removal and replacement work retains enough predecessor affinity to validate
the retained command. Logical damage covers the union of predecessor and
successor visible bounds after clipping, so vacated pixels cannot survive a
removal or move. The host may coalesce those regions mechanically but cannot
omit them or derive a broader semantic scope from the full projection.

This contract retains Worth host protocol revision 4 and advances only the
mounted-presentation schema to revision 5 for the compiler-total
`Reconstruction` carrier. Mounted-frame and measurement remain revision 4 and
the 3.14 observation schema remains revision 6. Both hosts implement that exact
contract during coexistence. Older or mixed revisions reject before effects;
there is no downgrade or reinterpretation path, and the superseded protocol
window is retired with egui at cutover.

Protocol revision 4 with mounted-presentation schema 5 is the complete Phase 3
carrier contract and retains its
2,048 semantic-text-row, 4,096-byte-per-row, and 1 MiB aggregate text bounds.
After `worth-ui-global-text-v2` qualification closes, Phase 4 advances the
protocol, mounted-frame, and measurement schemas to revision 5; the already
reconstructive mounted-presentation schema remains revision 5.
Revision 5 replaces the v1 semantic-text mechanic with span-capable admitted
paragraph constraints and raises only the qualified-text capacities to the v2
limits above: 4,096 paragraphs, 65,536 UTF-8 bytes per paragraph, and 8 MiB
aggregate retained UTF-8. Revision 4 and revision 5 text inputs cannot coexist
inside one agreement, frame, retained generation, or reconstruction. The
revision-5 transition is a cold protocol replacement whose admission completes
before any v2 layout effects; it is not an adapter-side reinterpretation of a
v1 row or a reason to rewrite the proved v1 profile.

The consumer input is the proof-carrying `UiHostProtocolAgreement`, not a raw
protocol contract. Negotiation owns old/mixed-revision denial; headless and
egui prove consumption of the validated revision-4 agreement, while a raw
revision-3/mixed contract cannot be substituted into either consumer. Tests
must not open a second constructor merely to counterfeit an invalid agreement.

The host may reject a malformed or stale delta before effects. It may not
recover by scanning the successor projection, comparing semantic digests,
inventing removals, widening logical scope without accounting, or rebuilding
an unchanged generation. Full reconstruction is a separately named cold or
recovery operation consuming current mounted authority.

Mounting derives one inert `UiMountedPaintOrderIdentity` for every drawable
command identity. Neither identity is orderable; the authoritative meaning is
the runtime-issued sequence that contains it. `Initial` carries the complete
total sequence. `Delta` carries exact `Remove` and `PlaceAfter` edits plus a
successor-order integrity value; `PlaceAfter(identity, None)` establishes the
front and the same operation represents insertion or movement. A local edit
does not renumber unrelated commands. A genuinely global semantic reorder may
carry global work and must report it.

Layer meaning remains inspectable. Equal-layer order across nodes comes from
the mounted semantic-node sequence. Within one node, drawable references are
ordered by their distinct explicit mechanic layers; two drawable mechanics on
the same node and layer deny projection before host effects rather than
manufacturing a family tie break. Command identities are never consulted as a
tie break. Presentation production requires every drawable to be named by
exactly one exhaustive typed drawable-reference source; a missing or duplicate
source is an internal mounting invariant violation before host effects. The
host never supplies a family-, receipt-, hash-, or arrival-based tie break. A
new drawable family cannot compile into presentation until the exhaustive
reference match carries its order and receipt contract.

### Native commands are derived, attributed, and non-authoritative

The native host lowers mounted rows into private command variants. A command
contains a generational command key, mounted node receipt, mounted instance,
frame, semantic surface, binding generation, paint-order identity, geometry,
clip, and the mechanic-specific payload. There is no receiptless constructor
and no generic raw-paint escape hatch.

Draw-list keys, GPU offsets, glyph references, atlas coordinates, backing
textures, and surface epochs are mechanical identities. They can correlate
evidence but cannot target an interaction, construct presentation work,
publish a frame, or reconstruct mounted authority.

The runtime-side native presentation attribution also carries opaque digests
for the authored-source provenance and authored semantic identity joined from
the exact mounted node receipt through the retained prepared-generation
graph/declaration indexes. The native host does not derive or interpret either
digest. `WindowsNativeBoundaryWorld` compares them with the independently
compiled seed declaration, so changing the authored declaration identity or
consistently reporting the same wrong mounted row cannot satisfy the courtroom
merely through correlated mechanical identities.

### The surface baseline is explicit and carries no appearance authority

Every registered presentation surface carries one move-only
`UiMountedSurfaceBaselineReceipt` issued by the mounting/surface-binding owner.
Milestone 3.14.1 admits exactly one baseline mechanic: canonical transparent
RGBA. The receipt binds semantic surface, binding generation, profile
generation, color/alpha contract, and clear semantics. It is not a mounted node
receipt, does not participate in hit testing or interaction, and cannot mint a
paint command or select an opaque color.

Initial preparation establishes that baseline before replaying the complete
runtime-issued order. Delta damage first restores affected pixels to the same
baseline and then replays every intersecting retained command in total order.
An opaque surface background, window color, theme value, or fallback appearance
must therefore arrive as an attributed runtime-issued filled rectangle; the
native host may not invent it. Presented-source capture preserves transparent
baseline pixels. Compositor-visible client pixels remain a separate OS-level
observation whose relationship to the transparent source is frozen by the
qualified native profile.

The baseline receipt authorizes only deterministic clearing for its exact
surface generation. It cannot publish semantic truth, satisfy a drawable-row
receipt requirement, cross a surface replacement, or be recovered from a
clear color, capture, native surface, or diagnostic record.

The host reports only the inert observation that a registration completed in
the canonical known-empty posture. Runtime/mounting consumes that one current
observation and privately mints the move-only mounted baseline receipt. A
copyable registration request, diagnostic baseline identity, repeated host
confirmation, or reconstructed surface/binding tuple cannot mint a second
receipt. The authoritative surface requirement and baseline lease live with
runtime mounting; the host contract carries only facts a mechanics consumer
must inspect.

### The platform owns scheduling and native resources

`worth-ui-host-native` owns the native event loop on its required thread, all
windows and surfaces registered beneath that loop, graphics adapter/device/
queue acquisition, input translation, redraw admission, capture completion,
device recovery, and normal close. It depends on `worth-ui-host-contract`, not
runtime, the product facade, Query binding, DSL, inspection, or application
code.

`worth-ui-runtime` is the higher application composition owner. The thin
`worth-ui-native-platform` crate re-exports that fixed progression and does not
own a parallel binding transition. Runtime privately binds one host-neutral
application to one qualified mechanics bundle and keeps the Worth session on
the event-loop thread.

The low-level mechanics boundary has one typed event-loop client contract, not
arbitrary closures. Its methods consume host-issued wake grants and return only
an exhaustive `Continue | WaitUntil | Close` directive. Because Rust has no
cross-crate friend visibility, this mechanical integration surface is callable
by a crate that deliberately depends on `worth-ui-host-native`; it is not
described as a platform grant boundary. It cannot receive, construct, or bind a
host-neutral Worth application. The enforceable product authority is the
runtime-private transition that alone owns `UiPreparedNativeApplication` and
its affine binding grant. Supported product topology forbids direct mechanics
dependencies, while focused host mechanics tests may exercise the low-level
contract without claiming application authority.

Worker threads may carry only typed readiness observations through bounded
channels and a native event-loop wake proxy. They do not receive the runtime
session, host adapter, window, device, mounted projection, or publication
authority.

A wake is a level-triggered readiness hint, never the semantic payload.
Each registered owner commits work to its bounded owner slot before signalling
its generation-bearing readiness state. Coalescing preserves the newest ready
generation and cannot erase pending work. The Phase 2 rectangle seed registers
one application presentation owner; later phases add the remaining concrete
filesystem, Query, intent, inspection, input, and capture owners without
replacing this protocol. One event-loop turn drains the committed work; work
committed during or after a drain re-arms exactly one wake. Queue
exhaustion, stale wake generation, coalescing, per-owner work consumed, turns
yielded, and starvation bounds are typed and counted. Shutdown closes wake
registration before worker join, so a late proxy signal cannot resurrect the
platform or strand owner work.

The application-side driver is owned above the adapter by
`worth-ui-native-platform`. It may submit owner-issued source, Query, intent,
deadline, and shutdown readiness into existing public lifecycle operations.
It cannot receive vendor graphics types, draw callbacks, raw paint access,
direct target selection, or a second publication method. The mechanics crate
sees application readiness only as an opaque bounded wake slot; it learns no
source, Query, or intent meaning. Every wake is classified, coalesced under an
explicit policy, bounded, and attributed in the lifecycle cost report.

Ordinary scheduling is event driven. A redraw is admitted only by new
presentation work, an exact capture request, required native recovery, or a
future framework-issued deadline. Cursor movement, idle polling, diagnostic
inspection, and an unchanged semantic turn do not inherently authorize GPU
work.

### Native input preserves the inherited IME contract

Windows remains the composition owner. The native host translates OS IME
`preedit`, `commit`, and `cancel` observations plus native text ranges into the
existing host-neutral observation contract. Translation binds the exact local
draft recipient, input revision, and event-time presentation/DPI basis and
carries the existing proof that canonical UTF-8 ranges fall on Unicode scalar
boundaries. The host cannot infer a recipient from OS focus, tree position,
current coordinates, or a newer presentation.

Preedit may change only visible runtime-owned draft posture and never enters a
payload. IME commit updates committed draft text but does not itself mint the
semantic `edit-commit`; only the declared commit gesture may do so. Cancel
discards the active composition posture. Missing recipient, stale affinity,
stale text-profile generation, over-capacity text, or unprovable native-range
conversion returns its distinct
typed stop before payload or intent effects and preserves the prior draft as
required by the inherited 3.14 lifecycle. General focus routing remains 3.15
work; this milestone adds no Worth-owned IME service or composition policy.

### Text mechanics have a frozen trust boundary

The closed v1 profile establishes migration provenance only. Before Phase 4,
the separate `worth-ui-global-text-v2` qualification gate commits the exact
default font collection, optional application-font-pack admission contract,
Unicode data and conformance suites, analysis/shaping/raster dependencies,
fallback order, feature/variation posture, layout constraints, raster formats,
and bounds. Changing any of those facts advances the text profile generation
and requires explicit requalification. Ambient OS fonts, locale, renderer
defaults, and dependency upgrades cannot silently change measurement or
pixels.

The host-neutral `worth-ui-text` mechanics crate owns three directionally
ordered derived phases:

```text
UiAdmittedTextParagraph
-> UiAnalyzedTextParagraph
-> UiQualifiedTextLayout
-> UiRasterizedGlyphBatch
```

Runtime owns semantic text, style-profile selection, explicit language/base-
direction, width, wrapping, alignment, line-height, spacing, overflow, clip,
and mounted identity. `worth-ui-text` receives only those completed constraints
plus one admitted font-collection generation. It may analyze, choose a face by
the frozen fallback contract, shape, fit lines, derive cluster geometry, and
rasterize qualified glyphs. It cannot change text, author style, choose a
system font, infer product locale, target interaction, publish a frame, or mint
mounted authority.

`UiAnalyzedTextParagraph` preserves logical source order and carries original-
range grapheme/word/line opportunities, bidi levels, scripts, languages, and
exhaustive run boundaries. `UiQualifiedTextLayout` is the canonical derived
artifact for measurement and rendering. It carries line and visual-run order,
selected face/feature/variation identities, glyph clusters and positions,
logical/ink metrics, baselines, break/overflow decisions, original-range caret
stops and selection rectangles, coverage disposition, complete affinity, and
cost. A measurement result derived from any other shaping/layout pass is
invalid. The artifact is non-authoritative and can be destroyed and rebuilt
from mounted text plus the exact profile/font-collection generations.

The dependency boundary carries that artifact without introducing a crate
cycle or a second owner. `worth-ui-host-contract` defines only inert borrowed
`UiQualifiedTextLayoutView<'layout>` and record-view types sufficient for a
consumer to inspect the qualified result. `worth-ui-text` depends on the host
contract, owns and constructs the concrete immutable layout, and exposes its
borrowed view. Runtime privately retains `Arc<UiQualifiedTextLayout>` beside
the exact mounted text affinity and seals a borrowed view into presentation
work for the active host session. Headless and native hosts may inspect that
view; they cannot construct, clone as authority, reshape, refallback, rebreak,
or extend its lifetime beyond the owning runtime work. The host contract never
imports `worth-ui-text`, and the layout identity alone cannot stand in for the
borrowed records required by measurement or rendering.

Fallback is a font-collection decision over complete clusters, not a loop over
characters. A selected face must shape the complete cluster without `.notdef`
before it is admitted. Emoji presentation selectors, modifiers, regional-
indicator pairs, tag sequences, and ZWJ sequences remain indivisible. If no
qualified face supports the cluster, the pinned Last Resort face produces one
missing-cluster glyph and an explicit coverage disposition while preserving
the exact source range and semantic value. This is distinct from malformed
font, stale generation, impossible mapping, or budget denial.

Raster output distinguishes alpha masks from premultiplied color glyph images.
The native host owns separate bounded alpha and color atlases keyed by font-
collection/profile generation, face, glyph, variation coordinates, palette,
size, raster source, DPI scale, and fractional-origin quantization. Candidate
allocation and deterministic eviction finish before uploads. Retained layouts
pin their exact glyph entries. Saturation denies before effects or enters a
named reconstructive posture; it cannot grow without bound, evict a live glyph,
drop emoji color, or substitute lower-quality pixels.

Text editing, focus, and selection authority remain above mechanics. However,
their later implementations consume v2 original-range mapping, grapheme-safe
caret stops, visual-order hit testing, and selection rectangles directly; they
must not reverse-engineer those facts from glyph positions or introduce a
second text layout engine.

### Damage and presentation are physically honest

Each native surface owns a retained presentation target suitable for both the
qualified render path and exact readback. It is derived state. Logical damage
comes from runtime-issued command changes; the host may coalesce or lower that
damage into physical scissors only under a named deterministic policy.
The host maintains a derived spatial index over retained command bounds and
uses it to issue one total-order replay plan for commands intersecting those
scissors. Clearing damage without replaying underlying commands, replaying
only changed commands, or scanning all retained commands for an ordinary local
delta is incorrect. Destroying the index and rebuilding it from the retained
draw list changes no pixels or authority.

Swapchain acquisition, full-target copy, compositor damage, and presentation
are distinct costs. If the selected platform/API cannot preserve prior
swapchain contents, the host retains its own presentation target and reports
any whole-surface transfer required to present it. Local CPU or render damage
does not license a false local end-to-end claim.

Mounted RGBA values, the native surface format, linear/sRGB conversion,
premultiplication, DPI rounding, clip half-open semantics, and readback
canonicalization are frozen in the host profile and covered by external pixel
control points. The pre-implementation specification-closure gate commits the
exact Windows backend, adapter-selection posture, surface/target formats,
present/alpha modes, blend equations, anti-aliasing posture, shader inputs,
coordinate rounding, and dependency versions as one versioned native profile
manifest. Phase 1 implements and proves that record without changing it.
Vendor defaults and runtime environment discovery are observations admitted by
that profile, never policy.

### Capture observes the exact presented-source target

Visual capture copies from the same retained target used to produce the
presentation identified by the existing snapshot contract. It proves the
exact source pixels handed to native presentation, not post-compositor scanout.
The executable-world client-area observation separately proves the compositor-
visible window. The qualified Windows profile freezes the transform and color
relationship between those two observation levels, and `HP-01` requires them
to agree at named control points without treating one as a substitute for the
other. Capture admission binds frame, surface, binding generation,
presentation epoch, coordinate transform, byte budget, and deadline before the
copy begins. Completion waits for the relevant GPU fence and canonicalizes
pixels into the existing artifact without changing their identity basis.

A draw list, atlas, command buffer, expected-color table, or reconstructed
image is not a snapshot. Capture cancellation before effects and uncertain
completion after copy/submit remain distinct. Readback buffers and mapped
bytes are framework-managed, bounded, disposable, and included in shutdown
census.

### Failure and recovery preserve authority

Presentation prepares and validates protocol, generation affinity, command
changes, total order, text support, capacities, atlas allocation, and GPU
resource requirements before effects. Candidate retained and atlas state stays
separate until the host reaches the declared effect boundary.

Prepared, surface-acquired, encoded, submitted, present-handoff, completed,
cancelled-before-effects, and indeterminate states are distinct consuming
types. A presentation epoch is issued only after the qualified native present
handoff succeeds; it means host completion at that declared boundary, not
proof of physical scanout. Input binds only the last such completed epoch,
never a prepared or in-flight successor. Input before the first completed
epoch, or whose event-time DPI/surface basis cannot resolve to that epoch,
produces a typed no-presentation-basis stop rather than current-coordinate
retargeting.

After effects may have begun, cancellation is not rollback. The host reports
in-flight or indeterminate posture through the existing mounted presentation
contract. Runtime keeps the predecessor semantic publication and exact
recovery authority. Device or surface reconstruction consumes current mounted
authority through a named reconstructive operation; it never uses a cache,
pixel snapshot, lifecycle event, or diagnostic record as source truth.

Shutdown owns and accounts for windows, surfaces, adapter/device/queue,
presentation targets, draw lists, GPU buffers, atlas pages/entries, pending
submissions, readback buffers/maps, event proxies, and application wake
registrations. Normal close is not complete until all census values are zero.

### Coexistence is bounded and deletion is final

Dual-host coexistence exists only inside this milestone and only for parity.
It shares the same host contract, Pulse composition root, binary, external
runner, causal-action manifest, and evidence schema; only the recorded host
grant differs. It may not introduce a common lowest-denominator renderer,
host-neutral pixel normalization, a permanent feature flag, second product
composition root, or parallel parity harness.

Revision-4 work in the egui host is deliberate, deletion-bound migration
scaffolding. Phase 10 removes that implementation, but `HP-01` is honest only
when both hosts consume the exact same protocol and mounted work. Its cost
belongs to the migration lane and cannot be optimized away through parity over
different protocol revisions, evidence-level normalization, or an egui-only
reinterpretation of revision 4.

At cutover:

- `worth-ui-host-egui` and the eframe Pulse shell are removed;
- `WorthUiHostKind::Native` becomes the only native-display kind;
  `WorthUiHostKind::Egui` and its constructor are removed and exhaustive
  matches are repaired;
- egui-specific certification is replaced by host-contract or native-platform
  evidence;
- the isolated `worth-ui-theme` and `worth-ui-components` egui-era Rust crates
  are removed rather than ported into pre-3.16 architecture;
- assets from those crates survive only if an existing non-egui semantic owner
  and current consumer are proven; they are not moved into a speculative asset
  bucket; and
- workspace enforcement permanently rejects all egui-family dependency edges.

## Public Developer Experience

The public type topology and launch calls below are normative:

```rust
use worth_ui_native_platform::{
    UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec,
    WorthUiNativePlatform,
};

let profile = UiNativePlatformProfile::single_window(
    UiNativeWindowSpec::new("WORTH UI Platform Pulse", [160, 96]),
);

let prepared_platform = WorthUiNativePlatform::prepare(profile)?;
let outcome = prepared_platform.run(
    PlatformPulseApplication::new(launch, publisher),
);

match outcome {
    UiNativePlatformOutcome::Closed(receipt) => observe_close(receipt),
    UiNativePlatformOutcome::ApplicationPreparationDenied(denial) => report(denial),
    UiNativePlatformOutcome::Stopped(stop) => recover_or_report(stop),
}
```

`WorthUiNativePlatform` is exported by `worth-ui-native-platform`. Application
authors do not import `worth-ui-host-native`, `winit`, `wgpu`, shaping,
rasterization, or native-window handle types. The application object supplies
product composition; the platform supplies the only native host grant and
consumes the application at run. Phase 1 exposes only preparation denial and
the explicit `NativeEffectsNotActivatedInPhaseOne` stop. Phase 2 adds typed
close, cleanup, in-flight native-effect, device-recovery, and terminal-resource
outcomes only alongside their real owners and observations.

`prepare(profile)` performs effect-free static profile validation and returns
a move-only `UiPreparedNativePlatform` or a typed profile denial; it creates no
event loop, window, surface, or device. `run(application)` accepts only an
`UiNativeApplicationDefinition`, consumes both it and the prepared platform,
owns all native effects, and returns `UiNativePlatformOutcome` directly.
`ApplicationPreparationDenied` means the application preparation scope closed
before native resource effects and never published an event-loop client. Its
compiler-visible absence of resource-opening operations is the proof; Phase 1
does not manufacture or expose a zero subsystem census. Programmer defects do
not masquerade as that outcome.

The `UiNativeApplicationDefinition` preparation contract consumes one
`UiNativeApplicationPreparation` carrying only a host-neutral public Worth UI
builder and the private affine platform binding. Product code may configure
the host-neutral application but cannot create subsystem or readiness owners,
attach, replace, clone, extract, or downcast a host. Completing the scope seals
the host-neutral application and platform binding into one
`UiPreparedNativeApplication`; product code cannot construct that result from
parts. Phase 2 consumes it, privately creates the real subsystem owners and
readiness progression, composes the fixed native host session, and launches on
the same event-loop thread.

Generic application freeze must therefore succeed without a
`WorthUiHostSessionPlan`. Host binding is removed from generic builder posture
and from frozen application authority. Certification/headless and native
platform composition each own a separate higher transition from the same
host-neutral application into an active session; neither transition exposes a
generic adapter parameter or host-replacement capability.

The exact Rust method signatures and denial payload fields are filled by the
pre-implementation public native-application authority record. They must
preserve the named types and proof relationships above; aliases, generic
callback registration, arbitrary mechanics-client implementations, and a
second host-bearing `WorthUiApplicationBuilder::with_host(...)` path are not
compatible implementations.

## Compile-Time and Mechanical Enforcement

- Runtime's owning `UiMountedPresentationWork` envelope, its
  initial/delta/reconstruction/unchanged choice, lease seal, and completion authority have
  private fields. The host contract may expose inert command, order, damage,
  and input mechanics, but they enter an active host session only through the
  runtime-owned envelope and lease.
- The mounted presentation lease, authoritative work variants, consumption
  view, and completion-token minting live under runtime's private host
  presentation authority. `worth-ui-host-contract` exposes only inert command,
  order, damage, outcome, and cost mechanics. No Cargo feature exposes runtime
  issuance to another crate; Rust features are dependency unification, not
  friend visibility.
- Initial, delta, unchanged, reconstruction, and completion are distinct
  proof-bearing types. A delta cannot be applied to the wrong retained frame,
  surface, binding, capability, device, or profile generation.
- Native draw-command constructors are private to the projection translator
  and require a complete mounted row; a receiptless or generationless command
  is unrepresentable.
- Surface clearing consumes an exact `UiMountedSurfaceBaselineReceipt` for the
  same surface/binding/profile generation. The only 3.14.1 baseline payload is
  canonical transparent RGBA; neither the native nor headless host can supply
  an opaque clear value or use the baseline as a drawable-row receipt.
- The mounted world compiler exposes semantic handles and typed actions with
  private fields. It cannot construct presentation work, native commands,
  atlas entries, platform authority, or expected production outcomes directly.
- Damage-index entries derive only from retained attributed commands. Delta
  execution consumes owner-issued damage plus a mechanical intersection/replay
  plan; neither the index nor replay plan can construct semantic damage or
  presentation work.
- Admitted paragraph, analyzed paragraph, qualified layout, rasterized alpha
  glyph, rasterized color glyph, atlas reservation, uploaded glyph, and
  retained glyph reference are distinct phases. Measurement and rendering
  require the same qualified layout identity. External dependencies cannot
  mint font-collection admission, mounted, interaction, or publication
  authority.
- The v2 font-collection manifest, Unicode conformance-data inventory, default
  fallback order, RGI emoji corpus, and app-font-pack limits are exhaustive
  generated/parsed contracts. Removing a face, script pack, color source,
  conformance file, or fallback entry fails qualification rather than shrinking
  coverage silently.
- Grapheme, shaping-cluster, line-break, bidi, logical/visual-order, caret, and
  original-byte-range identities remain distinct types. Raw byte, scalar,
  glyph, or atlas indexes cannot substitute for one another at public or host
  boundaries.
- Native IME observation phase, canonical range-conversion receipt, runtime
  draft transition, and semantic `edit-commit` are distinct types. The native
  translator can construct only host-neutral observations and conversion
  evidence; it cannot construct a draft transition, semantic interaction,
  payload, intent, or focus authority.
- Exact font and native platform manifests carry protocol/profile identity;
  mixed or environment-selected profiles reject before effects.
- Window, surface, device, presentation target, atlas, readback, and event-loop
  resources are registered under framework-owned generational handles and
  require exhaustive shutdown propagation.
- Event-loop client implementation, wake observation, and platform-binding
  grant are distinct types. The low-level mechanics client is not application
  authority and cannot receive a Worth application; only runtime's move-only
  binding grant can bind the host-neutral application into the fixed native
  product progression.
- Prepared, acquired, encoded, submitted, present-handoff, capture-readback,
  recovery, and terminal resource states are consuming types with only their
  lawful next transitions visible.
- Scripted window, surface/graphics, and readback ports can inject only typed
  external observations. They cannot mint lifecycle or platform outcomes,
  bypass the production orchestrator, or exist only under `cfg(test)`.
- Independent pixel, ordering, damage, atlas, and lifecycle oracles live
  outside the production mechanics they adjudicate. Boundary/source checks
  reject imports from disputed production algorithms, and the executable pixel
  oracle consumes a versioned test-owned control-point manifest.
- Vendor types remain private to `worth-ui-host-native`; boundary and
  dependency checks reject imports from `worth-ui`, runtime, DSL, Query
  binding, and inspection.
- `worth-ui-host-headless` and `worth-ui-host-native` may depend on
  `worth-ui-host-contract` and the identity-agnostic
  `worth-ui-retained-order` mechanism, but not `worth-ui-runtime`, `worth-ui`,
  DSL, Query, inspection, certification, or one another. The retained-order
  crate has no WUI dependency and owns no protocol or host policy. Runtime-facing
  session authority wraps the hosts' inert mechanics contracts from above.
- The host contract and native mechanics cannot import runtime or product
  internals. Runtime's ordinary semantic and mounting modules cannot import
  native mechanics. One isolated host-activation composition module may depend
  on the native mechanics facade solely to name the fixed qualified bundle;
  vendor types and native implementation modules remain unreachable there.
- Runtime's native-platform module privately owns the application-slot grant
  and combines a host-neutral Worth application definition with the fixed
  native mechanics implementation. `worth-ui-native-platform` re-exports only
  that public progression. The generic Worth builder exposes no host-bearing
  bind or replacement transition. No Cargo feature, generic adapter binder,
  caller-defined marker, or host-contract surrogate grants native application
  activation.
- Public outcome matches are exhaustive and `#[must_use]`; wildcard
  translations over lifecycle, presentation, capture, or recovery topology
  are forbidden.
- The workspace gate rejects `egui`, `eframe`, `egui-wgpu`, `egui_extras`, the retired
  adapter, and the migration selector. It parses every repository workspace
  dependency declaration and final lockfile in addition to resolved metadata,
  including the root `Cargo.toml`; `egui-wgpu` is prohibited with the rest of
  the family. Positive/negative twins prove the intended native and headless
  homes remain lawful.
- Compiler evidence stays in the existing consolidated compiler sessions.
  Production compilation and local model tests enforce private topology that
  has no public misuse value.

## Cost Contract

This is the destination contract closed by Phase 3 for retained delta work and
by later text phases for shaping/atlas work. Phase 1 proves only that the
revision-4 carrier is sparse, attributable, and reports exact carried
command/order/damage lengths, plus an unchanged carrier with no rows. It does
not claim that the current producer or migrated headless compatibility
consumer computes that carrier in `O(k)`: their retained-size scans and clones
remain named Phase 3 work. Phase 2 presents one Initial and does not claim a
retained-delta slope.

Ordinary host work is:

```text
delta validation       O(k + o)
draw-list mutation     O(k log n) worst case, with bounded indexed lookup
damage selection       O(d log n + r) for indexed intersection and order
damage replay          O(r + p_render)
Unicode analysis       O(t_changed + c_context) only for changed paragraphs
fallback/run shaping   O(c_affected + f_probes + g_affected)
line layout            O(b_affected + g_affected + l_affected)
atlas lookup/update    O(g_changed) with bounded candidate allocation
GPU upload             O(bytes changed or reconstructed)
presentation           explicit damaged work + physical surface amplification
```

`n` is retained command count, `k` changed commands, `o` owner-issued order
changes, `d` damage regions, `r` retained commands intersecting admitted
damage, `p_render` pixels actually rerendered, `t_changed` changed text bytes,
`c_context` the exact Unicode/shaping context invalidated around those bytes,
`c_affected` affected grapheme/shaping clusters, `f_probes` deterministic font
coverage probes, `g_affected` affected glyphs, `b_affected` affected break
opportunities, and `l_affected` affected lines. Complete overlap may lawfully
make `r = n`, and a context-sensitive script or width change may lawfully
invalidate a complete paragraph; each index and counter must prove that breadth
rather than scanning every retained command or paragraph to discover it. The
final implementation may prove stronger bounds but may not weaken an ordinary
local delta to a retained-list scan or an unchanged paragraph to reanalysis.

Counters separately expose:

- initial/delta/reconstruction/unchanged work and rows consumed;
- commands inserted, replaced, removed, reordered, reused, and retained;
- logical damage regions/pixels, damage-index probes, commands selected and
  replayed, cleared/rerendered pixels, and physical copy/present pixels;
- analyzed paragraphs/bytes/scalars/graphemes, bidi runs, script/language/style
  runs, line-break opportunities, fallback probes, shaped runs/clusters/glyphs,
  lines, caret stops, selection rectangles, alpha/color rasterizations,
  alpha/color atlas hits/misses/evictions, upload bytes, and retained layout/
  atlas bytes;
- buffer writes, encoder/pass counts, surface acquisitions, submissions,
  presents, fences, capture copies, and readback bytes;
- window/surface/device reconstruction and every retained resource class; and
- wake causes, coalescing, queue saturation, and dropped diagnostic richness.

Elapsed time supplements these counters and never substitutes for slope proof.
Cold start, ordinary delta, unchanged, capture, resize/DPI reconstruction,
device recovery, migration parity, and diagnostic materialization remain
separate cost lanes.

Courtroom construction has the same cost discipline. Maximum-size worlds
compile immutable semantic baselines once per existing test process, then
derive isolated scenario state and owner-issued deltas. No scenario shares
mutable draw-list, atlas, device, counter, installation, or process fate. Real
process/window/device startup is paid only by the serialized worlds whose claim
requires that boundary; exhaustive schedules remain in independent models.

## Architectural Destination

### Ownership

| Owner | Owns | Excludes |
| --- | --- | --- |
| runtime mounting | mounted projection, total paint order, exact presentation work, logical damage, publication affinity | native scheduling, GPU resources, font fallback |
| `worth-ui-host-contract` | inert presentation-work protocol, mounted mechanics, host observations, capture and typed outcomes | runtime truth, vendor types, native implementation |
| `worth-ui-retained-order` | bounded generic order-statistic sequence, exact mutation/high-water cost, and rollback-safe indexed ordering | UI identities, presentation protocol, runtime truth, host policy |
| `worth-ui-host-headless` | production headless mechanics, bounded transcripts, measurement evidence, and record-only presentation over the inert host contract | runtime internals, native resources, application truth, certification-only authority |
| `worth-ui-text` | qualified font collections, Unicode analysis, deterministic fallback, complex shaping, line layout, canonical measurement/cluster geometry, and alpha/color glyph rasterization | authored meaning, mounted authority, GPU resources, system fonts, input/focus/editing authority |
| `worth-ui-host-native` | native scheduling and readiness, windows, surfaces, graphics device, derived draw list, input translation, alpha/color atlas, readback, resource recovery | runtime, authored meaning, text-layout policy, targeting, intents, Query, publication |
| runtime native platform | effect-free application preparation, private native binding, prepared-application driver handoff, and terminal outcome projection | vendor mechanics, drawing, target or intent authority |
| `worth-ui-native-platform` | thin public facade over the runtime-owned native application lifecycle | binding issuance, runtime internals, vendor mechanics, drawing |
| application composition root | product source/Query/intent wiring and typed readiness | event-loop ownership, raw input translation, drawing |
| inspection | bounded immutable host and presentation evidence | operational construction or recovery |
| certification | independent models, governed worlds, parity and deletion proof | production authority |

### Destination tree

`create` means populated in this milestone. Committed successors show where
known work enters; no empty placeholders are created merely to realize the
diagram.

```text
plans/worth-ui/
  milestone-3.14.1.md                            [create: governing design]
  milestone-3.14.1-phase-5.md                    [create: Phase 5 subordinate design]
  milestone-3.14.1-proof-ledger.csv              [frozen: historical QA record]

workspaces/worth-ui/
  crates/
    worth-ui-host-contract/src/
      mounted_frame/
        presentation.rs                         [modify]
        surface_baseline.rs                     [create: transparent baseline authority]
        presentation_work/                      [create: owner-issued work]
          {mod,initial,delta,unchanged,command_change,
           damage,paint_order}.rs
        presentation_cost.rs                    [modify]
      mounted_projection/
        {static_paint,semantic_text}.rs          [modify: total order/profile]
        text_profile.rs                         [create]
      runtime/
        runtime_host_contract.rs                [modify; delete egui kind]
        native_platform_binding.rs              [create: affine run admission]
      visual_snapshot/                          [modify: native readback posture]

    worth-ui-runtime/src/mounting/projection/
      lowering/delta.rs                         [modify]
      presentation_work/                        [create: mounting authority]
        {mod,initial,delta,logical_damage,order}.rs

    worth-ui-runtime/src/host/adapter/           [modify]
      headless_{host,measurement,recorder,
                transcript,translation}.rs       [move to host-headless]

    worth-ui-host-headless/                     [create by moving production headless mechanics]
      Cargo.toml                                [host-contract-only WUI dependency]
      src/
        lib.rs                                  [facade only]
        adapter/{mod,host,measurement,recorder,presentation}.rs
        transcript/{mod,frame,static_paint,semantic_text}.rs

    worth-ui-retained-order/                    [create in Phase 3: shared bounded mechanism]
      Cargo.toml                                [no WUI dependencies]
      src/{lib,index,cost}.rs                   [generic identities only]

    worth-ui-text/                              [create in Phase 4: shared text mechanics]
      Cargo.toml                                [host-contract-only WUI dependency]
      assets/fonts/                             [exact v2 catalog and licenses]
      profiles/worth-ui-global-text-v2.toml     [canonical qualification manifest]
      src/
        lib.rs                                  [facade only]
        profile/{mod,manifest,identity,capacities,qualification}.rs
        font_collection/
          {mod,admission,face,coverage_index,fallback,application_pack}.rs
        analysis/
          {mod,paragraph,grapheme,word,line_break,bidi,script_language,
           original_range}.rs
        layout/
          {mod,request,run_segmentation,shaping,line_fitting,overflow,
           artifact,caret,selection,cost}.rs
        raster/
          {mod,request,alpha,color,source_order,cost}.rs

    worth-ui-host-native/                       [create: host-contract-only WUI dependency]
      Cargo.toml
      assets/fonts/body-default/
        {font-asset,LICENSE,profile.toml}        [create: exact qualified asset]
      src/
        lib.rs                                  [facade only]
        native/
          event_loop/{mod,client,client_binding,run,wake_grant,wake_proxy}.rs
          window/{mod,port,registry,lifecycle,dpi,close}.rs
          platform/windows.rs                   [current certified platform]
        presentation/
          {mod,session,transaction,cost,reconstruction}.rs
          command/{mod,filled_rect,glyph_run}.rs
          draw_list/{mod,state,delta,order}.rs
          damage/{mod,logical_lowering,intersection_index,
                  replay_plan,physical_report}.rs
          surface/{mod,port,binding,target,present}.rs
        graphics/
          {mod,device,buffer,submission,failure}.rs
          backend/{mod,port,wgpu}.rs             [external contract/private vendor]
        text/
          {mod,layout_admission,glyph_batch}.rs  [consume worth-ui-text]
          atlas/
            {mod,state,reservation,eviction,pinning,cost}.rs
            page/{mod,alpha,color}.rs
        input/
          {mod,observation,pointer,keyboard,text_ime}.rs [OS translation only]
        capture/
          {mod,port,admission,readback,completion,cancellation}.rs
        lifecycle/
          {mod,resource_registry,census,recovery,shutdown}.rs

    worth-ui-runtime/src/native_platform/       [create: private binding gate]
      {mod,application,application_driver,
       native_platform_binding,outcome,platform,profile}.rs

    worth-ui-native-platform/                   [create: public facade]
      Cargo.toml
      src/
        lib.rs                                  [facade only]

    worth-ui-host-egui/                         [remove after parity]
    worth-ui-theme/                             [remove; isolated egui era]
    worth-ui-components/                        [remove; isolated egui era]

    worth-ui-certification/
      src/topology/host_platform/               [create inside existing suite]
        {mod,dependency_boundary,destination,retirement,
         retirement_world}.rs
      tests/application_contracts/host_platform/ [create inside existing target]
        {mod,presentation_delta,text_atlas,input,capture,recovery,
         lifecycle,cost}.rs
        world/{mod,mounted_presentation,native_lifecycle_protocol}.rs
        oracle/{mod,ordered_pixels,atlas_lifecycle,native_lifecycle}.rs

  apps/platform-pulse/
    src/
      main.rs                                   [modify: Worth native launch]
      application.rs                            [modify: no egui context]
      native_frame.rs                           [replace]
      native_application/                       [create: product driver]
        {mod,composition,readiness,lifecycle}.rs
    tests/executable_world/                     [modify; same target/courtroom]
      adjudication/native_profile_control_points.toml [create]
      courtroom/host_parity.rs                  [create or modify]
      source_delta/causal_action_manifest.rs    [create]
      product_process/host_migration_grant.rs   [create in Phase 9; remove in 10]
```

The stable structural axes are mounted authority, inert host protocol,
headless mechanics, shared text mechanics, native mechanics, application-
platform composition, retained presentation, graphics, input observation,
capture, and derived evidence. Headless is a real contract consumer and
production proof surface, not runtime-owned semantics or certification
support; it consumes `worth-ui-text` for the same qualified layout and depends
on no runtime internal. Native consumes that layout/raster facade and alone
owns GPU atlases. Do not flatten these axes
into `renderer.rs`, `text.rs`, `font_manager.rs`, `host.rs`, `state.rs`,
`resources.rs`, `helpers`, `common`, a product callback bag, or the
application session.

Committed successors enter additively:

- 3.15 adds framework deadlines and service-ready wake causes without moving
  event-loop ownership; portal layers become new mounted command/order input,
  never host-local popup state;
- 3.16 adds appearance-derived mounted values upstream; the host continues to
  render completed mechanics without theme authority;
- later focus/editing/accessibility work consumes the existing text layout's
  original-range, grapheme, caret, visual-order, and selection geometry and
  adds semantic owners above it; rich text adds authored span meaning to the
  existing span-capable request rather than a second shaper or renderer;
- Milestone 4 adds window registrations beneath the existing native window
  registry rather than replacing a single-window loop;
- later vector, icon, canvas, and realtime mechanics add command-family
  siblings beneath presentation with the same receipt/order contract; and
- accessibility adds a distinct mounted effect/observation boundary rather
  than being inferred from draw commands.

## Ordered Phases

### Current implementation status

Phases 1-5 are implemented and historically reviewed; Phases 6-7 were
implemented and independently reviewed on 2026-08-23; Phase 8 was implemented
and independently reviewed on 2026-08-24. The corrected Phase 9 implementation
record and its frozen source snapshot were independently certified and closed
on 2026-08-24. The corrected Phase 10 native-cutover implementation and its
frozen source snapshot were independently certified and closed on 2026-08-24,
completing Milestone 3.14.1.
Their old ledger rows,
nonces, receipts, handoffs, and artifacts are frozen and are not current QA
state. Historical phases do not reopen. If a current change regresses an
earlier guarantee, the regression is a finding against the current change and
is demonstrated through the relevant product test or repository check.
Readiness evidence alone still does not imply pixels, async completion truth,
recovery, parity, or completion.

### Phase 1: Protocol, qualification, and topology closure

Consume the closed qualified native/text profile and public native-application
authority records without altering them. Implement the owner-issued
initial/delta/reconstruction/unchanged protocol, exact
predecessor/successor affinity, stable total paint order, logical damage,
expanded cost vocabulary, explicit transparent surface baseline, and the
runtime/mounting producer. Move presentation issuance, the authoritative
surface requirement, and baseline receipt into runtime-owned private modules;
remove feature-gated cross-crate issuers. Keep native binding authority private
to native composition and make generic application preparation host-neutral.
Move production headless mechanics into
`worth-ui-host-headless`, then update egui, headless, and certification
consumers through the same revision-4 contract; none may retain the
complete-projection ordinary lane or import runtime internals. Materialize the
already-governed `BodyDefault` asset/profile manifest, native Windows profile,
pinned dependency versions and trust record, public native-application types,
capacities, destination tree, and egui retirement inventory exactly as the
pre-implementation records require. Add compile/topology/dependency enforcement
plus the governed mounted world compiler, independent delta/order/damage
oracles, and the test-owned control-point manifest with filled-rectangle
expectations; glyph expectations remain pending. No later phase begins while a
hidden default, unbounded capacity, mixed protocol, host-side rediscovery path,
headless runtime coupling, or oracle import from disputed production mechanics
remains open. Focused qualification tests verify that the committed manifest
identities match the specification; implementation code may not select or amend
them.

Phase 1 remains one atomic trust gate, but its implementation plan closes five
ordered internal batches: protocol and runtime producer; egui plus the moved
headless and certification consumer migration; materialization and proof of
the prequalified font/native/application records; governed world compiler plus
independent oracles; and topology/compiler enforcement plus final review.
These batches are not independently trusted phases. Phase 2 waits for their
combined exit gate rather than advancing on partial revision-4 adoption or
provisional manifests.

Phase 2 may trust complete, ordered, attributable presentation work, the frozen
native profile, the v1 migration text profile, and mechanically enforced
dependency direction. It may not treat v1 as Phase 4 text qualification.

Phase 1 is complete when its protocol, authority, profile, producer, consumer,
headless, topology, damage, ordering, affinity, world, and cleanup behavior is
implemented; the focused tests and constitutional checks pass on the current
commit; and code review finds the evidence adequate. Carrier-shape cost tests
cover sparse payload lengths and unchanged-zero carriage only; they do not
substitute for the Phase 3 computational-slope, retained-index, or native replay
tests.

### Phase 2: First native vertical presentation

Activate native effects in the Phase 1-created mechanics and runtime-owned
application-platform gate: Worth-owned event loop, private affine platform
binding, application driver, host-native level-triggered readiness
scheduling, Windows window/surface/DPI lifecycle, graphics device/queue,
retained target, and resource registry. Present one attributable initial filled
rectangle through `WindowsNativeBoundaryWorld` in a real native window, observe
its actual client pixels, remain quiescent without readiness, and close with
exact zero framework resources. Establish the production external-effect ports
and their real implementations at the boundaries this vertical slice crosses.
External quiescence compares only the courtroom-owned opaque rectangle control
points. Transparent retained baseline is proved at presented-source readback;
the OS compositor backdrop behind transparent client pixels is observed but is
not assigned to WORTH or frozen to an expected color.
This phase stays vertical: it does not end at a window skeleton, clear color,
fake surface, or in-memory renderer.

Phase 3 may trust one owned, visible native lifecycle and one attributable
initial presentation path without continuous repaint or vendor leakage.

Phase 2 is complete when the prepared-application handoff, event-loop ownership,
level-triggered scheduling, real Windows window/surface/DPI lifecycle, graphics
ownership, attributable rectangle presentation, independent client pixels,
external-effect ports, and terminal cleanup pass their focused tests and review.
Rectangle-only evidence cannot be reused to claim Phase 3 retained-delta or
Phase 4-5 text behavior; an in-memory renderer or window-only smoke test cannot
prove either successor.

### Phase 3: Retained delta, total order, and damage replay

Build the receipt-keyed draw list, exact command mutation and order indexes,
derived damage intersection index, total-order replay plan, staged presentation
transaction, unchanged zero path, reconstruction from mounted authority, and
complete structural/amplification counters. Close `HP-02` and mutation controls
in `MountedPresentationWorld::maximum_overlap` against retained-list scans,
changed-command-only redraw, widened damage, stale delta reuse, vacated pixels,
and equal-layer nondeterminism.

Phase 3 is complete when stale-delta rejection, owner-issued delta source,
draw-list ownership, total ordering, damage indexing and replay, baseline replay,
transactionality, unchanged-zero behavior, clipping, reconstruction, headless
cost, physical amplification, and the native pixel world pass focused tests and
review. The slope tests compare retained sizes 1, 32, 2,048, and 4,096 with
constant changed/damage/replay widths and require exact retained-row scan and
clone counters rather than elapsed-time thresholds. The native world owns the
2,048-rectangle pixel courtroom; the mixed 4,096-command
headless world supports carrier/index/slope rows and cannot supply native text
pixel evidence.
`P3-CLIPPED-DELTA-01` proves the distinct lawful case where a mounted logical
successor changes retained truth but clips wholly outside the current physical
target: it advances successor affinity with zero acquire, submission, present,
or newly minted physical presentation epoch and the following unchanged frame
remains ordinary rather than entering reconstruction.

Phase 4 may trust scalable filled-rectangle initial, delta, unchanged,
overlap-correct replay, and reconstruction behavior.

### Phase 4: Qualified Unicode text platform and canonical layout

First close the blocking `worth-ui-global-text-v2` qualification record:
exact font faces/licenses/digests, Unicode 17 data and conformance corpus,
analysis/shaping/raster dependencies, generated coverage/fallback indexes,
emoji corpus, capacities, and profile identity. Then create the host-neutral
`worth-ui-text` mechanics crate and advance the host text protocol so runtime
issues complete paragraph/style/language/direction/width/overflow constraints.
Implement original-range mapping, UAX #29 grapheme/word segmentation, UAX #14
plus qualified dictionary line opportunities, full UAX #9 bidi analysis,
script/language/style/face run segmentation, deterministic cluster fallback,
HarfBuzz-compatible complex shaping, line fitting, cluster-safe clipping and
ellipsis, logical/ink metrics, baselines, caret stops, hit testing, selection
rectangles, and the reconstructible `UiQualifiedTextLayout` artifact.
Unicode 17 RGI emoji is a first-class Phase 4 layout obligation: variation
selectors, keycaps, regional-indicator flags, tag sequences, skin-tone
modifiers, and gendered/family ZWJ sequences remain atomic through
segmentation, fallback, shaping, wrapping, ellipsis, caret movement, hit
testing, selection, and original-range mapping. Phase 5 adds their actual
color-glyph raster and native pixel proof; it does not get to repair a broken
Phase 4 cluster or fallback decision.

Application fonts are an ordinary framework path, not a certification-only
escape hatch. Phase 4 must expose a public, host-neutral
`UiApplicationFontPackDefinition -> UiQualifiedFontPackReceipt` admission
transition and an authored `UiFontFamilyStack` on every style span. A pack may
contain several families and several faces per family, including static
regular/bold/italic/oblique faces, variable weight/width/slant axes, and
multiple face indices from one collection. The resolver matches the authored
family order and requested face attributes deterministically, then performs
complete-cluster fallback through later authored families. An RGI emoji
cluster then resolves through qualified color emoji and Last Resort; every
other cluster resolves through the qualified profile defaults and Last
Resort. Profile defaults may not decompose or monochrome-substitute an RGI
emoji, and the color-emoji face is not a generic fallback for non-RGI text.
File extensions, localized name records, registration order, ambient OS
installation, and adapter availability are never identity or tie-break
authority.

Font-pack mutation is generational and reconstructible. Adding, replacing,
or removing a pack publishes a successor
`UiFontCollectionGeneration`; live predecessor layouts keep their exact face
bytes pinned, newly admitted work cannot use a stale generation, and
reconstruction consumes the mounted paragraph plus its exact collection
generation rather than reopening a path or querying the operating system.
One application may use different qualified families in adjacent spans and
different paragraphs without forking shaping, measurement, hit testing,
accessibility geometry, headless recording, or native rendering into separate
font-selection authorities.

Interaction geometry is part of the canonical layout, not a later renderer
guess. `UiTextCaretPosition` carries an exact original UTF-8 boundary, visual
edge, and upstream/downstream affinity. `UiTextHitTestResult` records the
point-to-line-to-visual-run-to-cluster decision and returns one of those typed
caret positions. A logical selection owns an ordered, potentially
discontiguous set of per-visual-run rectangles. At a shared LTR/RTL visual
edge, both lawful affine caret positions remain distinguishable even when
their pixel coordinate is equal. Accessibility geometry is an inert consumer
of this same layout identity; it may not reshape, refallback, rebreak, or
derive logical-order bounds independently.

Capacity admission is two-stage and effect-free. Exact input bytes and
declared constraints reserve before analysis; a conservative bound derived
from the qualified font collection reserves worst-case glyph/run/line output.
Actual segmentation, shaping, and line layout occur in bounded unpublished
staging. Any derived overflow denies atomically before publication,
rasterization, upload, or retained insertion. Partial shaped output is never
observable.

Headless measurement and transcripts must consume this exact artifact; the
native host may not yet render it. Close the Unicode, shaping-fixture, layout,
measurement-identity, missing-cluster, stale-affinity, capacity, unchanged-
zero, and reconstruction portions of `HP-03`. A one-run Latin implementation,
scalar fallback, separate measurement shaper, or output lacking original-range
cluster/caret geometry cannot pass this gate.

Phase 4 is complete when the qualified text profile, font collection and color
font admission, Unicode segmentation and emoji sequences, bidi, fallback,
shaping, line layout, capacity, measurement identity, original-range and
interaction geometry, accessibility geometry, locality, reconstruction,
unchanged-zero behavior, and text cost pass focused tests and review. The tests
must name their Unicode/reference data, corpus slice, production transition,
independent oracle, adversarial case, and counters where those details matter;
one generic multilingual screenshot cannot satisfy them.

`P4-FONT-COLLECTION-01` is not satisfied by counting the profile's default
faces. Its governed cases must include two application families with
overlapping coverage that select different face identities, regular/bold/
italic and variable-axis matching, a multi-face collection index, an authored
fallback stack whose first family lacks a complete cluster, emoji fallback,
family-name collision, over-capacity and malformed-pack denial, successor
generation replacement/removal while a predecessor layout remains live, and
reconstruction from the exact pinned generation. Independent font-table and
layout oracles must reject ambient-system substitution, a hard-coded single
family, registration-order selection, stale-generation reuse, or face/style
matching that differs between measurement and presentation.

Profile qualification precedes production consumption. The canonical v2
manifest, every referenced font/license/data/corpus artifact, generated index,
dependency feature posture, and manifest digest must exist and independently
validate before Phase 4 production uses them. A provisional manifest identity
or dependency-default behavior cannot be consumed as production configuration.

Locality evidence keeps independent axes separate: a content-only edit holds
width, locale, direction, font collection, profile, and text-scale generations
fixed; a one-paragraph width edit changes only that paragraph; a document-wide
width replacement is named separately. Each axis runs retained sizes 1, 32,
2,048, and 4,096 and reports analyzed bytes, bidi contexts, fallback probes,
shaped runs/glyphs, lines, and unchanged-sibling work. Combining content and
width mutation into one favorable fixture cannot prove paragraph locality.

Application font admission explicitly accepts COLRv0/CPAL, COLRv1/CPAL,
CBDT/CBLC, and sbix under the qualified compositing, palette, strike-selection,
and resampling rules. The sbix lane admits `png` and one-hop `dupe`-to-`png` records;
`jpg`, `tiff`, and OpenType SVG are explicitly unsupported in this profile and
are rejected atomically before layout effects. Silent color-layer loss,
monochrome fallback, or adapter-selected raster semantics are forbidden.

Phase 5 may trust bounded deterministic multilingual/RTL/complex-script/emoji
layout and measurement from repository or explicitly application-bundled fonts,
with no system-font or adapter-layout authority.

### Phase 5: Color glyphs, emoji, atlas, and native text presentation

Consume only Phase 4 `UiQualifiedTextLayout` authority. Close qualified alpha
and intrinsic-color raster, separate bounded native atlases, live-layout
pinning, deterministic unpinned eviction, staged upload, original-range paint
spans, DPI/text-scale behavior, native pixels, exact cost, cleanup, and complete
derived-state reconstruction. Native completion is fed through Runtime Bridge
into one Query-owned semantic mounted-presentation async result; glyphs, atlas
entries, pages, and uploads remain native implementation resources. Every
exact Unicode 17 RGI mapping is proved
internally; representative real pixels cover every admitted raster source and
sequence class. Consumers may not reshape, refallback, rebreak, substitute
fonts, or own framework text state.

Phase 5 uses two disjoint Signal domains. Query/Runtime Bridge owns the
semantic presentation graph and its application-visible invalidation. One
bounded physical Signal runtime in each host-native host/device lifecycle owns
atlas/upload/poll/retry/cancel/recovery/wake/shutdown progression around native
resources. Host-native retains WGPU and atlas physical truth and imports no
Query; Signal stores no raw WGPU handles and grants no native effect authority.

The normative authority graph, staged raster/atlas transaction, profile
semantics, `HP-03` courtroom, resource/cost contract, implementation gates,
destination tree, documentation, and acceptance evidence are specified in
[`milestone-3.14.1-phase-5.md`](milestone-3.14.1-phase-5.md).

Phase 5 tests cover qualified glyph raster, exhaustive color emoji, bounded
alpha/color atlas ownership and pinning, DPI behavior, original-range span
paint, retained and compositor pixels, reconstruction, locality and resource
cost, Query-owned async presentation, and terminal cleanup. They include an
external mixed-font/mixed-size/bidi/mixed-foreground observation, an external
color-emoji observation, and separate alpha/color atlas/resource census. Span
paint tests reject single-color substitution, visual-order color assignment,
emoji tinting, and layout regeneration on a color-only edit. Color-emoji tests
raster every exact RGI sequence mapping internally; native pixel evidence
supplies representative observations for every admitted color source and
sequence class and cannot replace the exhaustive internal corpus test. The
public `HP-03` path must retain
pending/current/stale/failed/cancelled/superseded/unresolved posture in Query,
while host-native alone decides external completion and runtime retains exact
attempt lineage. Query remains absent from host-native and runtime; the sole
production WORTH UI Query edge is `worth-ui-query-binding`. Host-native alone
may import Signal for the dedicated physical runtime. Runtime owns neither
Signal graph.

Destination-topology tests may freeze ownership, type separation, and forbidden
consumer dependencies before implementation. They are readiness evidence only
and cannot substitute for behavioral, lifecycle, resource, or native-boundary
tests. Phase 5 remains part of this milestone.

Phase 6 may trust deterministic framework-grade text layout and attributable
multilingual/color-emoji pixels without system fallback, environment-selected
coverage, duplicate shaping, or unbounded retained state.

### Phase 6: Native input and presentation affinity

Replace egui input translation with host-neutral native observations for every
shipped 3.14 family. Prove exact last-completed-presentation affinity, input
before first presentation, input during successor presentation, event-time
DPI/resize basis, lossless ordering, bounded observation delivery, and the
inherited gesture/IME distinctions. Close the input portion of `HP-04` and the
3.14 interaction mutation controls through `WindowsNativeBoundaryWorld` for
real OS delivery and `NativeLifecycleProtocolWorld` for unavailable or
adversarial schedules. Exercise preedit, composition commit, cancel, canonical
range conversion, and their no-recipient, stale-affinity, stale-text-profile,
over-capacity, and unprovable-conversion stops. Prove preedit never enters payload, IME commit does
not mint semantic `edit-commit`, and a native event, current coordinate, or in-
flight frame cannot retarget itself.

Phase 7 may trust the complete native input-to-presented-target path without
targeting, intent, or publication authority in the host.

### Phase 7: Presented-source capture and snapshot integration

Implement capture admission, retained-target copy, fence-bound completion,
canonical readback, bounded buffers/maps, cancellation before effects, and
indeterminate completion after effects may begin. Join presented-source pixels
to the existing snapshot identity and independently compare them with external
client-area control points. Close the capture portion of `HP-04` through both
native worlds and the test-owned control-point manifest, keeping real readback
and injected completion claims distinct. Draw lists, expected-color tables,
reconstructed images, and compositor screenshots cannot impersonate one
another.

Phase 8 may trust exact bounded capture of the native presentation source with
typed affinity, cancellation, disposal, and external pixel correlation.

### Phase 8: Failure, recovery, and hostile shutdown

Complete typed surface/device loss, timeout/outdated posture, resize/minimize
reconstruction, prepared/acquired/encoded/submitted/present-handoff effect
states, close during upload/presentation/readback, and reconstruction only from
current mounted authority. Cross shutdown with queued readiness and a held 3.14
application attempt, then prove exact zero for every platform, native, worker,
application, Query, intent, capture, draw-list, atlas, and GPU resource class.
Close all remaining `HP-04` rows with fault-injected protocol evidence and real
native integration kept explicitly distinct. The protocol world runs the
independent transition model; the Windows world supplies only the real-boundary
facts its environment actually exercised.

During Phase 2, before qualified text exists, the Platform Pulse library
supplies `PlatformPulseNativeSeedApplication`: one public-composition-root
application containing exactly one attributed filled rectangle and no text.
That seed is the only honest application the first native vertical can present
atomically. It does not impersonate the complete Pulse product journey. The
full source/Query/intent/text application replaces the seed in its ordered
phases and is the application used by the cumulative `HP-01` courtroom.

Phase 9 may trust the complete native product path under ordinary, denied,
in-flight, indeterminate, recovery, and shutdown postures.

### Phase 9: Dual-host cumulative parity and native candidate freeze

Introduce the migration-only launch grant and run `HP-01` against egui and
Worth native hosts through the same Pulse binary, composition root, runner,
versioned causal-action manifest, and evidence schema in
`PulseNativeParityWorld`. Record the migration-grant identity as the sole
differing world input. Close cumulative semantic, receipt, interaction, intent,
consequence, capture, lifecycle, and control-pixel parity; adjudicate and record
exactly one glyph-region rebaseline. Freeze the proven native candidate and
cutover inventory. This phase does not delete or silently bypass the
predecessor until parity evidence is final-source green.

Phase 10 may trust a migration-complete native candidate whose only remaining
work is authority cutover and predecessor removal.

#### Phase 9 closed implementation record (independently certified)

The frozen candidate is migration identity `pulse-worth-native-v1` through the
existing `worth-ui-platform-pulse` binary and public application composition.
`PulseNativeParityWorld` runs that candidate and predecessor identity
`pulse-egui-r4-v1` sequentially against the same checked-in causal-action
manifest and the same installation path. The predecessor must complete
external close and prove installation absence before a fresh native candidate
is installed at that exact path. Both runs retain every decoded lifecycle
envelope and compare the complete ordered variant stream through exhaustive
typed correspondence. Stable semantic, source, receipt, interaction, intent,
consequence, visual, and shutdown fields remain exact. Host-issued frames,
bindings, snapshots, attempts, mounted identities, interaction sequences, and
semantic-target digests retain their raw values and require typed bijective
correspondence; presentation epochs retain their raw values and require an
order-preserving correspondence that admits host-side coalescing. The complete
named host-mechanical exclusion set is envelope run identity, product-process
identity, confirmation-expiry timestamp, host-surface identity, native window
origin, filesystem event-burst identity, watcher-notification multiplicity,
native-host effect multiplicity, host input-translation multiplicity, and exit
poll multiplicity. Every excluded raw pair remains in typed parity evidence:
process identities must be distinct and nonzero, exit polling must be nonzero,
translated input must retain the same family and posture, and native effects
must remain nonzero. The grant identity is the sole differing world input and
is adjudicated outside lifecycle correspondence. Native pointer and keyboard
probes use counted Win32 input events, deny before effects when the target world
is not qualified, and report every failure after `SendInput` as indeterminate
with the delivered count. The causal manifest is consumed at each real edit,
activation, await, successful observation, close, and shutdown boundary rather
than replayed or batch-recorded after a completed journey.
Shutdown carries and directly requires Query close completion, empty intent
resources, joined watchers, and zero pending input/observation queues. Bounded
external capture observes a stable canonical predecessor before requiring the
schema recovery to restore those exact pixels. The product driver lives under
`native_application`; `native_frame.rs` is only the deletion-bound
eframe/input-translation adapter. The parity courtroom lives at
`courtroom/host_parity.rs`, and the causal manifest owner lives at
`source_delta/causal_action_manifest.rs`.

The predecessor installs the same source, Query, intent, and visual readiness
owners through a host-neutral application signal. A product wake causes egui
to replay retained paint for that frame, but only a changed presentation tick
schedules another frame; there is no continuous repaint loop. After final
schema recovery, each real product process must remain alive for one 500 ms
quiescent interval while producing zero lifecycle envelopes and byte-identical
client pixels. The existing certification-owned
`public_unchanged_is_allocation_free_and_one_instance_change_is_bounded` lane
supplies the separate in-process companion: it issues an exact unchanged
mounted request and requires zero allocation, zero mount mutations, zero
adapter work, zero additional host transcripts, and one retained-instance
reuse. The executable runner does not import certification, runtime,
test-support, or headless-host authority to reenact that proof.

External pixel and input adjudication owns the versioned literal manifest
`platform_pulse_control_points.json`. Its logical extent, control points,
colors, tolerances, pixel bounds, DPI, text profile, and font digest are
test-owned oracles; executable adjudication does not import production visual
geometry or color constants.

Exactly one glyph-region rebaseline is frozen in
`apps/platform-pulse/tests/executable_world/adjudication/glyph_region_rebaseline.json`.
It records predecessor and successor external pixels, both renderer/font
profiles and asset digests, dependency versions, 144 DPI, the 240×144 client
extent, surface/color posture, and the independent Windows Graphics Capture
boundary. Both host journeys admit that same record digest while checking
their host-specific pixels; filled-rectangle control expectations remain
unchanged.

The Phase 10 cutover inventory is frozen to these deletion and repair owners:

- remove `crates/worth-ui-host-egui`, the `eframe` Pulse shell, and all
  `eframe`/`egui` manifest and lockfile edges;
- remove the deletion-bound `native_frame.rs` adapter after the continuing
  product driver is solely native, and remove any obsolete egui-era application
  owner left by that cutover;
- remove the isolated `crates/worth-ui-theme` and
  `crates/worth-ui-components` egui-era crates; an asset from either crate may
  survive only when an existing non-egui semantic owner and current consumer
  are both proven, never by creating a speculative asset owner;
- remove both production and courtroom `UiHostMigrationGrant` selectors,
  make the native path unconditional, and rerun the same cumulative journey
  without a predecessor selector;
- remove `WorthUiHostKind::Egui`, the `legacy-egui-migration` feature and entry
  surface, then repair exhaustive matches and certification fixtures;
- retire the executable predecessor glyph expectation while preserving this
  rebaseline as version history, and replace egui-specific certification with
  native-platform or host-contract evidence; and
- update continuing launch, lifecycle, architecture, visual-inspection, and
  interaction documentation only after the source cutover is complete.

No Phase 10 deletion was pulled into this closure. The egui path remains live
only for the frozen parity predecessor and is not an application fallback.

The final executable source passed two consecutive clean `HP-01` runs without
an intervening source edit. Each run retained and adjudicated 75 lifecycle
envelopes for each host, performed 25 externally counted captures per journey,
completed one process launch and native window per host, completed external
close and successful exit, and removed the shared installation before the
successor install and after terminal shutdown. The
focused library, hostile-input, correspondence, compile-contract, feature,
test-topology, line-cap, formatting, Clippy, boundary, and generated-context
checks are green. Independent certification of this frozen source snapshot
completed on 2026-08-24. Phase 10 has not begun; certification does not reopen
this implementation record or create a separate proof ledger.

### Phase 10: Native cutover, egui deletion, and final closure

Make the Worth native host the sole native-display path, then delete the egui
host, eframe shell, migration selector, egui-era theme/component crates, old
glyph expectations, and every dependency edge. Close `HP-05`, continuing
documentation, exact cost budgets, exhaustive host-kind repairs, code review,
and constitutional/recurrence gates through `HostRetirementTopologyWorld` on
the exact post-deletion source. Rerun the complete Pulse journey natively so
deletion cannot manufacture a green result by removing parity coverage.

#### Phase 10 closure (independently certified 2026-08-24)

The production application and Pulse composition now enter the Worth native
platform unconditionally. The migration selector, legacy transition, eframe
shell, egui host, and isolated theme/component crates are absent from the
workspace and lockfiles. Continuing headless evidence remains contract-only;
native lifecycle, input, capture, and cumulative Pulse evidence remain in
their established targets.

`HostRetirementTopologyWorld` discovers repository manifests, lockfiles,
sources, documentation, deleted crate destinations, and the existing compile
matrix dynamically. It rejects hidden or aliased retired dependencies and
unclassified current references while explicitly distinguishing historical
milestone records, detector source, and the negative compile twin. The
negative twin proves that the legacy public transition is absent; the positive
twin proves the fixed headless certification path remains lawful.

Historical proof-ledger runners, obsolete phase-runner configurations, and
test suites whose only purpose was to validate those ledgers or compare the
deleted host were removed. They are not release authority and were not replaced
with a new bookkeeping layer. Focused verification and all three independent
review disciplines certified the exact frozen source snapshot, closing Phase 10
and Milestone 3.14.1 on 2026-08-24.

No phase creates a new integration-test target, executable-world target,
binary, nested Cargo invocation, compiler session, product composition root, or
universal mutable fixture. Hostile lifecycle and environment qualification are
consolidated into the deliberately serialized `WindowsNativeBoundaryWorld`;
`PulseNativeParityWorld` pays only for its cumulative product journey and
cannot reuse hidden process state. Combinatorial state, maximum-table, overlap,
and long fault schedules use cheap independent models inside existing targets.
Within one Phase 1 closure or operational-verification run, `P1-WORLDS-01`
owns the single 2,048-row maximum-overlap courtroom. `P1-HEADLESS-COST-01`
binds that exact artifact by digest and validates its cost-specific observation
without reconstructing or replaying the world; its cost record names one
shared mounted-world reference and zero marginal main-test or presentation
executions.
Within one Phase 2 closure or operational-verification run, `P2-WORLD-01` owns
the single `WindowsNativeBoundaryWorld` process/window/GPU execution and its
immutable result artifact. The other Phase 2 rows bind that exact artifact by
digest, validate their requirement-specific observation slice, and execute
only their own cheap hostile control. Their cost records name one shared-world
reference and zero marginal process, courtroom, presentation, or main-test
executions; they may not relaunch the same native world per row.
On the recorded executable-certified Windows reference environment, warm
ordinary execution
remains at or below 60 seconds, the real native lifecycle courtroom at or
below 30 seconds, each `HP-01` host journey at or below 45 seconds, and the
temporary sequential dual-host run at or below 90 seconds. There are no blind
retries; the ordinary gate retains the cheapest mutation-sensitive example of
every guarantee.

## Documentation Deliverables

| Document | Continuing audience and required truth | Verification |
| --- | --- | --- |
| `workspaces/worth-ui/README.md` and `AI_README.md` | Developers discover the Worth native launch path, the sealed application-preparation contract, and that mounted/runtime meaning remains above native or headless mechanics. | Public launch example compiles and dependency audit stays green. |
| `workspaces/worth-ui/docs/native-host-platform.md` | Application authors and maintainers learn application preparation/resource cleanup, lifecycle, wake scheduling, exact supported platform/text posture, unsupported-Unicode predecessor preservation, transparent surface baseline, OS IME translation versus runtime draft authority, typed failures, recovery, capture, cost, and anti-patterns. | Examples compile; qualification identities and `HP-02`-`HP-04` prove described behavior. |
| `workspaces/worth-ui/docs/text-platform.md` | Application authors and text/input/component maintainers learn v2 font-pack admission, Unicode 17 support, deterministic fallback and Last Resort posture, bidi/complex-script/emoji behavior, wrapping/overflow, measurement-layout identity, caret/selection geometry, DPI versus text scaling, capacity denials, atlas lifecycle, and the boundary between text mechanics and later editing/accessibility meaning. | Public font-pack and measurement examples compile; Unicode/HarfBuzz fixtures, the complete `HP-03` corpus, and text topology enforcement remain green. |
| `workspaces/worth-ui/docs/application-lifecycle.md` | The cumulative Pulse command and visible journey use the Worth host, including native actions and exact-zero close. | `HP-01`. |
| `workspaces/worth-ui/docs/architecture.md` and `runtime-subsystems.md` | Ownership of presentation work, contract-only headless/native mechanics, public application-platform preparation, derived retained state, and 3.15 insertion is accurate. | Topology and dependency enforcement. |
| `workspaces/worth-ui/docs/visual-inspection.md` | Snapshot capture is exact presented-source-target readback with typed affinity, cancellation, budget, disposal, and a distinct compositor-visible observation posture. | Native capture and anti-substitution evidence. |
| `workspaces/worth-ui/docs/interaction-and-intents.md` | Native observations replace egui translation without changing targeting, IME/draft phase distinctions, or intent authority. | 3.14 cumulative input evidence. |
| `plans/worth-ui/worth_ui_roadmap.md` | 3.14.1 contract closure and 3.15 dependency remain current. | Documentary consistency audit. |
| `plans/worth-ui/milestone-3.14.1-phase-5.md` | Phase 5 raster, atlas, pixels, cost, reconstruction, and QA considerations remain explicit without duplicating a roadmap milestone. | Documentary consistency audit and focused Phase 5 test review. |

Delete or rewrite every continuing example that names eframe, egui context,
adapter repaint, or the retired host. Do not create a milestone closeout guide
or migration guide with no post-cutover audience; version history records the
temporary coexistence and glyph rebaseline.

## Must Ship and Preserve

Ship the owner-issued presentation-work protocol, total paint order, Worth
headless mechanics, native mechanics, and application-platform crates,
event-driven lifecycle, the sealed public native-application preparation
progression, retained attributable draw list, surface-issued transparent
baseline, indexed overlap-correct damage replay, filled rectangles, the
qualified Unicode 17 text platform, deterministic font collection/fallback,
complex shaping, bidirectional and line layout, canonical measurement/cluster/
caret geometry, multilingual and color-emoji rasterization, bounded alpha/
color atlases, level-triggered readiness, native 3.14 input observations,
exact presented-source-target
capture, failure/recovery lifecycle, exact resource census, the governed
five-world test suite, independent oracles, versioned causal-action and
control-point manifests, the historical Phase 9 dual-host parity conclusion
without its retired executable predecessor machinery, one glyph rebaseline,
final egui deletion, and recurrence gates described above.

Preserve all closed 3.10-3.14 guarantees, especially:

- the public application facade and one cumulative Platform Pulse composition
  root;
- exact mounted node, frame, surface, binding, presentation, snapshot, and
  interaction identity;
- the single 3.12 observation/rebind/publication path;
- Query-owned projection and mutation authority;
- presentation-bound targeting, typed intent admission, provider effects, and
  consequence handoff;
- predecessor semantic truth under denial or uncertain native effects;
- Query-free and unchanged semantic zero-cost posture;
- bounded inspection and non-authoritative diagnostics; and
- the consolidated integration, executable-world, and compiler topology.

## Acceptance and Successor Handoff

Planning for each unfinished phase begins only after its qualification records
are complete and internally consistent. In particular, text work is blocked
until `worth-ui-global-text-v2` contains the exact asset/dependency/data/corpus
record required above. The milestone is complete when the `HP-01`-`HP-05`
behaviors have honest, risk-proportionate evidence; all governed native
resources reach exact zero; the glyph rebaseline is reviewed; the Unicode 17
conformance and RGI emoji corpus is complete; public examples compile;
continuing docs agree; the production headless adapter is contract-only and
outside runtime; no egui-family dependency declaration, resolved edge,
lockfile package, or path remains outside explicitly classified negative
fixtures; and formatting, strict lint, line-cap, test-topology, boundary,
agent-context, ordinary certification, compile-contract, native integration,
and executable-world checks are green on the final commit.

Each phase specification states the relevant architecture, authority, security,
lifecycle, recovery, performance, resource, integration, developer-experience,
and test considerations in prose. During implementation, run the focused owner
and integration tests affected by the change. At final review, run the required
CI and constitutional checks once on the final commit, plus expensive native or
scheduled lanes only when their boundary is affected. Shared maximum-table,
GPU, window, and executable worlds should execute once per applicable test run
and expose observations directly to the assertions that need them.

Code review is the evidence authority. Reviewers inspect the final diff, the
specification, test-oracle independence, fixture realism, boundary coverage,
failure handling, resource cleanup, and the actual check results. Results are
reported as passed, failed, or environment-blocked. A product or test failure
blocks according to its causal impact. A failure that exists only in historical
ledger digests, receipts, nonces, artifact publication, predecessor handoffs,
row status, or phase-reopen machinery is legacy QA infrastructure and is
non-gating. Historical phases never reopen; a regression is handled as a
finding against the current change.

`milestone-3.14.1-proof-ledger.csv` and retained closure artifacts remain frozen
for historical traceability. Do not update, repair, regenerate, validate, or
consult them as current authority. Maximum-table model tests reuse immutable
worlds inside existing targets, real GPU/window startup is paid only by the
serialized platform/executable lanes that require it, and no claim is supported
solely by a rarely run soak lane.

Milestone 3.15 may trust a Worth-owned event loop and native presentation
platform where exact mounted deltas become attributable pixels, 3.14 input
families arrive through host-neutral observations, framework deadlines can
wake the loop, exact presented-source pixels can be captured and correlated
with compositor-visible client pixels, and all native resources have typed
failure, recovery, and disposal. It adds portal, focus, motion, command,
scroll, and selection meaning above these mechanics; it may not reintroduce
adapter-local state, generic callbacks, continuous repaint, renderer-selected
appearance, or egui compatibility.
