# Text platform

WORTH UI qualifies text once and carries the same typed layout through measurement,
accessibility geometry, headless presentation, native rendering, and reconstruction. Text
consumers do not choose fonts, reshape source strings, or ask the operating system for a
substitute font.

## The model

A text layout is produced from three owned inputs:

1. a qualified global font collection;
2. an admitted UTF-8 paragraph with explicit constraints and style spans; and
3. explicit profile, font-collection, and text-scale generations.

`qualify_text_layout` performs admission, Unicode 17 analysis, whole-cluster fallback,
OpenType shaping, bidi visual ordering, line fitting, and interaction recording. Its result,
`UiQualifiedTextLayout`, owns the selected font-byte identity and immutable layout artifact.
Hosts receive borrowed views of that artifact. This is why measurement, caret geometry,
accessibility, headless output, native rendering, and reconstruction cannot silently disagree.

## Bring your own fonts

Start from `UiGlobalFontCollection::admit_qualified_profile()`, then register one or more
application packs with `register_application_pack`. A pack may contain multiple families and
multiple faces per family. The application owns every supplied byte.

Accepted containers are TTF, OTF, TTC, and OTC. Set `face_index` for a collection face.
WOFF, WOFF2, malformed fonts, unsupported color tables, fonts beyond the admitted byte and
face capacities, and fonts without qualified license metadata are rejected before publication.
WORTH UI never searches system fonts as a fallback.

Each face declares its family, static weight, width, slant, and license record. Admission
derives and freezes its name records, variable axes, OpenType feature inventory, Unicode
coverage, color-glyph posture, glyph-expansion bound, and exact font-byte digest. A text span
then supplies:

- an ordered `UiFontFamilyStack`;
- a `UiTextFaceRequest` for weight, width, and upright/italic/oblique slant;
- optional `UiFontVariationCoordinate` values such as `wght` or `wdth`; and
- optional `UiOpenTypeFeature` values such as `liga`.

The resolver considers one best face per authored family in stack order. Registration order is
not a selection rule. Unsupported explicit features, out-of-range axes, foreign family
receipts, and stale collection authority are typed denials.

## Fallback and emoji

Fallback is deterministic and atomic at the shaped cluster boundary. WORTH UI never splits a
cluster merely to mix fonts. The route is:

- the span's ordered application family stack;
- qualified profile defaults for ordinary text;
- the qualified color-emoji face for Unicode 17 RGI emoji sequences; and
- Last Resort when no qualified face covers the whole cluster.

The frozen Unicode 17 corpus includes grapheme, word, line-break, bidi, variation-selector,
and all 3,953 RGI emoji sequences. ZWJ families, flags, keycaps, skin-tone modifiers, tag
sequences, and VS15/VS16 stay atomic. Emoji selection carries the same exact face and font-byte
identity into shaping, measurement, rendering, and reconstruction.

## Alpha and intrinsic color

Alpha glyphs produce coverage only. Presentation combines that coverage with the authored
logical foreground color as straight RGBA; changing the foreground is paint-only work. An
intrinsic-color glyph instead produces qualified RGBA from its admitted palette or bitmap
source and ignores an adjacent foreground token. It is never tinted to make it resemble an
alpha glyph.

The qualified color-source set is deliberately finite: COLRv0/CPAL, the admitted COLRv1/CPAL
paint graph, CBDT/CBLC, and sbix PNG data (including one admitted duplicate-image hop). SVG,
sbix JPEG or TIFF data, malformed paint graphs, unknown enum values, malformed images, cycles,
and every unlisted source are rejected during font admission. Runtime does not silently
downgrade a rejected color source to monochrome or ask an operating-system renderer to choose
another interpretation.

## Staged appearance foreground (Milestone 3.16)

An existing `ComponentSemanticTextSpanContract` can explicitly adopt its node's
appearance role color with `span.with_appearance_foreground()`. Other spans keep
their foreground token. Adoption preserves the original UTF-8 range, style, and
paint identity; the value range does not adopt a separate posture row.

This currently feeds unpublished mounted appearance and headless translation.
Live presentation still uses the existing text/token publisher until Gate 5.
An empty `UiNativeComponentSemanticTextChange::successor` clears the value while
retaining declared formatting; it still requires the current semantic revision.
Content and span-adoption changes refresh mounted foreground without requiring
an unrelated owner-state change. Prepared text revisions remain pending until
publication succeeds; dropping a prepared frame or rejecting its publication
before effects does not consume them.

Publishing a subset of surfaces does not acknowledge text for an omitted mounted
copy. Each binding retains its accepted presentation predecessor when the scope
expands again; that surface's complete reconstruction is separate from ordinary
paint refresh cost.

During staged lowering, foreground membership is restricted to current qualified text candidates
after mounted ancestor and Portal clipping. A completely clipped candidate has
no foreground override; restored coverage uses its current original-span identity.
A partial clip retains the candidate's original spans. This is not per-glyph
visibility or exact span-damage support.
Missing retained correspondence or unresolved clip evidence denies preparation
instead of treating the text as successfully suppressed.

Staged appearance work exposes `text_damage_requirements()` for exact mounted
target/span insertions, replacements, and removals. Headless transport preserves
these unresolved requirements separately from completed non-text logical damage.
The node allocation is no longer reported as text damage. Actual old/new image
coverage must be finalized by text/presentation after host raster admission and
before surface submission; that integration remains unfinished.

Gate 4b remains open for clipping lifecycle,
intrinsic-color admission, exact span damage, and complete work-accounting proof;
this staged API does not expand the frozen text profile.

## Generations and replacement

Register, replace, or remove a pack by consuming the current collection and naming the exact
successor generation. A predecessor collection remains usable only by layouts and
reconstruction sources that already pin it. It cannot qualify new text after its successor is
published.

This separation is intentional:

- fresh qualification requires the current admission lease;
- existing layouts keep their original `Arc`-owned font bytes and identities;
- replacing or removing a pack does not mutate an existing layout; and
- reconstruction of an existing layout uses its original collection, not the newest pack.

Do not reuse a numeric generation as collection identity. Layout requests carry the exact
collection lineage, so two independent collections with the same generation number cannot be
substituted.

## Paragraphs, spans, and language

`UiTextParagraphConstraints` defines the BCP-47 language (or `und`), base direction, wrapping,
alignment, overflow policy, font size, width, line height, letter and word spacing, tab
interval, and line limit. Every byte of a nonempty paragraph must be covered exactly once by
ordered, nonoverlapping `UiTextStyleSpan` ranges on UTF-8 boundaries.

Authored text-paint spans use original UTF-8 ranges too. Paint ownership is attached before
bidi reordering and remains attached to the same logical clusters afterward; it is not
reconstructed from visual order. A shaped glyph cannot straddle two authored paint spans. A
span boundary that would split its cluster is denied instead of assigning the glyph an
arbitrary color.

Style-span ranges, caret positions, hit results, and selection ranges refer to original UTF-8
byte boundaries. Selection cannot bisect a grapheme or shaped cluster. Bidi carets retain their
visual edge and upstream/downstream affinity, and a logical selection may produce multiple
visual rectangles. Empty paragraphs and trailing hard-break lines still have canonical line
and caret anchors even though the break itself paints no glyph.

Each qualified layout records paragraph, line, run, cluster, and glyph logical geometry plus
font-derived ink bounds. Italic overhangs and displaced combining marks therefore remain
measurable even when their ink extends beyond advances or line logical boxes.

## Capacity and cost

Admission reserves conservative bounds before analysis or shaping. The qualified profile caps
source bytes, style spans, families, faces, application font bytes, graphemes, shaping runs,
glyph expansion, lines, and retained layouts. A pack's derived shaping expansion participates
in that preflight; an application font cannot defer an unbounded expansion until shaping.

Ordinary content-only and width-only changes are paragraph-local. Unchanged sibling layouts
are reused by identity, and their text work is zero. Changing language, family stack, selected
face, feature values, variation coordinates, scale generation, or other layout-affecting
constraints invalidates only the affected paragraph unless the authored constraint itself is
global. Reconstruction is a named, separately costed lane.

## Measurement and accessibility

Use the layout's lines, glyphs, logical bounds, ink bounds, caret stops, hit testing, and
selection rectangles. Headless measurement and accessibility geometry consume the same
borrowed layout identity as presentation. They must not infer geometry from the source string
or invoke a font library independently.

Color is a paint-only property. A color-only update may reuse layout only when the exact font
collection lineage, layout-affecting style, and layer order are unchanged.

## DPI, text scale, and raster generations

DPI scale and framework text scale are different authorities. A pure monitor-DPI change keeps
the logical layout identity, line breaks, original UTF-8 ranges, and caret geometry. It replaces
only the device-pixel raster and atlas generations derived from that layout. A framework text-scale
change is layout-affecting: qualify a successor layout before presentation because advances, line
breaks, ink bounds, hit geometry, and selection rectangles may all change.

The same rule applies to width. Changing one paragraph's authored width creates a successor layout
for that paragraph; it does not authorize reshaping unchanged siblings. Do not multiply a DPI scale
into the authored font size and then treat the result as a new logical layout—that would make a
window move between monitors change measurement and accessibility identity.

## Raster and atlas lifecycle

Phase 4 freezes the handoff to native rasterization; it does not let layout or a host widget own a
GPU atlas. The native renderer owns separate bounded alpha and color atlases. The qualified profile
admits at most four 1024×1024 alpha pages, two 2048×2048 color pages, 8,192 entries, 36 MiB of atlas
texels, and an 8 MiB staged-upload budget. An atlas key includes the font-collection and profile
generations, exact face and glyph, variation coordinates, palette, size, raster source, DPI scale,
and fractional origin.

`UiGlyphRasterBearing::positioned_bounds` places the actual raster image from its
retained bearing and extent. Rasterization already includes the signed fractional
origin, so native drawing adds only the integral physical origin, truncated toward
zero. Native clipping crops the corresponding atlas texels. This shared geometry
describes the image rectangle; it does not establish nonzero-alpha coverage or the
complete appearance span-damage handoff.

Live layout and presentation work pin their entries. Eviction may choose only unpinned entries;
saturation is a typed denial and never overwrites a live glyph. Phase 5 supplies deterministic
alpha-outline, COLR/CPAL, qualified bitmap, and color-emoji raster output without changing Phase 4
font selection or layout identity. Runtime derives exact demand from mounted layout and paint
authority; text owns raster meaning; the native host alone plans capacity, stages uploads, owns GPU
pages, reconciles physical completion, and commits atlas entries and pins.

When capacity requires eviction, the native owner chooses the unpinned entry with the oldest
completed-use epoch and then the canonical raster-key bytes. Hash-table order, registration
order, allocator addresses, and GPU enumeration are never tie-breakers. Admission and eviction
planning complete before rasterization or native effects begin. If the staged upload, page,
entry, or pin budget cannot be satisfied without touching a live entry, presentation is denied
as saturated and the previous atlas remains current.

Presentation is asynchronous at both of its distinct boundaries. One bounded host-native Signal
runtime progresses physical atlas-upload and surface-presentation work. Separately, the WORTH UI
Query binding installs each mounted presentation attempt as a Query-owned async result and publishes
producer-local semantic invalidation. Signal eligibility never grants permission to rasterize,
upload, present, or settle, and Query never owns native resources.

Effects-indeterminate presentation retains an exact recovery capability. Reconstruction consumes
current mounted/runtime authority and retained font bytes, independently destroys the qualified
layout plus derived raster, atlas, draw-list, and target state, and rebuilds every layer under
successor generations from the mounted source and font authorities.
It never treats stale raster bytes, atlas placement, a transcript, or captured pixels as source
truth. Ordinary locality remains bounded to changed demand and exact subscribers; the qualified
closure portfolio separately exercises 1, 32, 2,048, and 4,096 fresh worlds.

A denial before native effects leaves the previous presentation and resource owners unchanged.
If effects may have begun, the attempt becomes unresolved and carries the exact recovery
capability until reconstruction or terminal close consumes it. WORTH UI does not flatten that
posture into success, retry from a formatted diagnostic, or use a lower-quality or monochrome
fallback. Terminal close must release pending Query results, physical Signal work, uploads,
pins, atlas pages, readbacks, draw lists, and recovery capabilities.

## Authority boundary and qualified posture

Applications own source text, original-range style and paint intent, font bytes, and declared
layout constraints. Text qualification owns Unicode analysis, fallback, shaping, layout, and
interaction geometry. Runtime mounts that qualified meaning and derives exact presentation
demand. The native host owns raster scheduling, atlas and GPU resources, physical progress,
and external completion. Query retains application-visible async posture and semantic
invalidation, but neither Query nor Signal grants permission to perform native effects.

Editing, selection policy, accessibility actions, keyboard input, and IME composition remain
later semantic consumers. Phase 5 preserves the original-range and caret geometry they will
consume; it does not make raster or atlas state an editing authority. Phase 6 input must enter
through its own host-issued observations and may request a new qualified paragraph—it may not
mutate a retained layout or reconstruct text from pixels.

These guarantees describe WORTH's qualified profiles, admitted fonts, deterministic native
owners, and governed Windows/WGPU evidence worlds. They are not a claim that arbitrary machine
fonts, unsupported drivers, or all compositors produce universally byte-identical pixels.
Unsupported input is denied rather than substituted, and external pixel evidence is compared
only inside the qualified source/class and host posture recorded by owner-issued receipts.

## Compiling example

The repository example registers two application families, selects them through explicit ordered
family stacks, enables OpenType features, requests `wght` coordinates, and qualifies mixed Latin,
Arabic, and emoji text in three original-range style and paint spans. Emoji falls back as a whole cluster to the
qualified color-emoji face. It reads logical and ink bounds from the qualified layout for
measurement; application code does not configure atlas pages, raster sources, or upload policy.

```powershell
cargo run --manifest-path workspaces/worth-ui/Cargo.toml -p worth-ui --example text_platform
```

See [text_platform.rs](../crates/worth-ui/examples/text_platform.rs) for the complete compiling
program. In an application, use your own licensed `Arc<[u8]>` values rather than embedding the
profile fixture used by the example.

## Failure guidance

- `UiFontCollectionAdmissionDenial` means the profile, pack, face metadata, container, byte
  budget, color tables, feature/axis metadata, generation, or license record was not admitted.
- `UiTextQualificationDenial::Admission` means the paragraph, spans, generations, or derived
  capacity were rejected before text work.
- `Fallback`, `Shaping`, and `Layout` denials identify the exact later effect-free staging
  boundary; no partially qualified layout is published.
- A stale collection denial should be resolved by qualifying new text against the current
  collection. Keep the predecessor collection only for already-qualified layout or
  reconstruction authority.

Do not recover from a denial by asking a host widget or OS API to render the raw text. That
creates a second font, shaping, measurement, and accessibility authority and is outside the
WORTH UI text contract.
