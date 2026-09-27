# Milestone 3.14.1 Phase 5: Qualified Glyph Raster and Native Text Presentation

## Status, Placement, and Authority

This is the normative subordinate specification for Phase 5 of
`Milestone 3.14.1: Aspect-Native Host Platform and egui Retirement`.

It does not create a new roadmap milestone or an alternate text platform. The
parent milestone continues to own the overall goal, the ten-phase order,
inherited contracts, and the Phase 6 handoff. This document owns the detailed Phase 5
product decisions, authority graph, destination topology, courtroom, cost and
resource contract, implementation gates, and documentation obligations.

The sources of authority are ordered as follows:

1. the repository engineering constitution and coding laws;
2. the parent milestone specification;
3. this subordinate Phase 5 specification;
4. the qualified `worth-ui-global-text-v2` profile and its content-addressed
   assets;
5. the Phase 5 implementation plan, which is derived from this specification.

Historical ledger rows and closure artifacts are frozen records, not active QA
authority. They are not updated or reopened, and ledger-only failures do not
block current work. Current completion follows
[the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
[testing laws](../../docs/coding-guidelines/testing_laws.md): this specification states
QA considerations in prose, relevant tests and repository checks run against
the current commit, and code review decides whether evidence is adequate.

The parent and subordinate specifications must agree. A conflict between them
is a specification defect and blocks implementation; an implementer may not
silently choose one. The profile is exact data and policy within the authority
the specifications assign to it. Changing a Phase 4 profile contract requires
a specification change plus affected qualification checks, not a Phase 5
workaround.

## Goal and Central Claim

Turn the closed Phase 4 `UiQualifiedTextLayout` into attributable alpha and
intrinsic-color glyph pixels through a bounded native atlas and the existing
presentation lifecycle, without granting any consumer authority to reshape,
refallback, rebreak, substitute fonts, reinterpret paint spans, or retain
framework-owned text state.

The complete Phase 5 causal chain is:

```text
mounted semantic-text authority
  -> Query-owned pending presentation resource
  -> Runtime Bridge correspondence and Query-semantic Signal invalidation
  -> WORTH UI Query binding and runtime presentation-attempt lineage
  -> runtime-owned UiQualifiedTextLayout
  -> exact glyph-raster demand
  -> bounded host-native physical Signal request
  -> effect-free native atlas plan and capacity admission
  -> text-owned alpha/color raster batches for admitted misses
  -> native external submit/poll observations
  -> physical Signal completion reconciliation
  -> native staged upload and glyph-run command settlement
  -> retained target and compositor-visible pixels
  -> native settlement observation, recovery, or reconstruction
  -> Runtime Bridge validation against the exact presentation basis
  -> Query-owned current/stale/failed/cancelled/superseded/unresolved result
  -> Query-semantic Signal invalidation of exact downstream consumers
```

Every arrow carries typed identity and proof. No stage may rediscover upstream
meaning from strings, raw source text, a system font, an atlas entry, or
previous pixels.

Phase 5 is complete only when the chain works for the full qualified text
profile, the exact Unicode 17 RGI emoji set, mixed fonts and paint spans,
ordinary and reconstructive work, saturation, failure, cleanup, headless
inspection, and real Windows/WGPU pixels.

## Inherited Guarantees

Phase 5 consumes these closed Phase 4 guarantees and must not reproduce them:

- exact application and profile font bytes, face selection, variations,
  OpenType features, fallback, color-source admission, and generation-safe
  retention;
- canonical Unicode 17 analysis, grapheme/word/line boundaries, bidi levels,
  logical and visual runs, shaping, original byte ranges, and Last Resort;
- canonical line fitting, baselines, logical and ink bounds, carets, hit
  testing, selection geometry, overflow, and measurement identity;
- runtime ownership of the exact `Arc<UiQualifiedTextLayout>` used by every
  downstream consumer;
- borrowed host-contract layout views rather than host-owned layout authority;
- paragraph-local ordinary work and named reconstructive work;
- explicit text-scale, DPI, font-collection, profile, and width generations;
- exact paint-span identity and original-range mapping;
- typed aspect and field authority at WORTH UI admission.

Phase 5 may use those records. It may not re-run their decisions.

## Explicit Non-Goals

Phase 5 does not:

- redesign Unicode analysis, fallback, shaping, line fitting, measurement,
  caret, selection, or accessibility meaning;
- permit native, headless, accessibility, or inspection consumers to reshape
  or derive substitute geometry;
- consult ambient or machine-installed fonts;
- complete the broad Query audience-facade migration. Phase 5 nevertheless
  must add the minimal presentation-specific Query async declaration and
  binding through `worth-ui-query-binding`; no Phase 5 boundary may flatten
  its request basis into formatted strings, JSON values, debug output, or a
  digest used as operational authority;
- introduce public atlas, page, upload, packing, raster-source, or GPU knobs;
- expose either Signal runtime, its local aspect slots, ready frontier, or
  native effect port as an application API;
- use one ambient Signal graph for both Query semantics and native physical
  work, or construct a Signal runtime per glyph, upload, surface, atlas entry,
  or presentation;
- promise pixel-identical output across unqualified GPUs or environments;
- implement text editing, IME semantics, focus, accessibility semantics, or
  Phase 6 input affinity;
- change the qualified profile merely to make an implementation easier;
- create a second QA authority, closure portfolio, or roadmap item;
- accept a native-only, screenshot-only, Latin-only, one-font, one-color, or
  representative-emoji shortcut as closure.

## Current Boundary

The current source intentionally freezes only inert destination vocabulary:

| Boundary | Current authority | Phase 5 destination |
|---|---|---|
| Layout | runtime owns `Arc<UiQualifiedTextLayout>` | unchanged; runtime remains the only durable layout owner |
| Consumer view | host contract lends `UiQualifiedTextLayoutView<'layout>` | extended with borrowed raster-demand and raster-batch views, never owned text state |
| Raster meaning | `worth-ui-text` has typed alpha/color batch and cost vocabulary | concrete qualified raster-demand and raster production |
| Native atlas | host-native has typed entry, capacity, pin, and lifecycle vocabulary | concrete effect-free plan, move-only reservation, GPU page/upload ownership, pin and eviction lifecycle |
| Orchestration | runtime owns mounted presentation coordination | private native text-presentation transaction above text and host-native |
| Native physical progression | current Gate D source has manual pending upload, polling, recovery, and callback-settlement structures | one bounded physical `SignalRuntime` per native host/device lifecycle; WGPU resources remain native-owned |
| Async presentation meaning | upstream scalar projection state already uses Query through `worth-ui-query-binding`; native presentation completion is not yet Query-owned | one presentation-specific Query declaration/binding and retained application-visible async result |
| Semantic invalidation | current Query binding translates retained async states without a Phase 5 presentation graph | Runtime Bridge correspondence plus a distinct Query-semantic Signal graph using Milestone 13 locality-first execution |
| Evidence | readiness tests and partial Phase 5 evidence | direct behavioral, lifecycle, resource, locality, and native-boundary tests reviewed against this specification |

Readiness types and topology tests cannot substitute for the behavioral,
lifecycle, resource, locality, or native-boundary tests required below.

## Async Presentation Authority

The Query resource is one semantic mounted presentation attempt/completion
basis. A glyph, atlas entry, atlas page, placement, staging buffer, upload
submission, WGPU resource index, or redraw wake is never a Query resource.

Authority is ordered and non-interchangeable:

1. Query installs the semantic presentation-resource/completion meaning and
   owns the retained application-visible async posture.
2. Runtime Bridge binds the installed Query meaning to the exact WORTH UI
   presentation operation. `worth-ui-query-binding` is the sole production WUI
   Query importer and audience edge through which that correspondence is
   installed and consumed; neither copies Query authority into runtime or
   host-native.
3. The Bridge-owned Query-semantic Signal graph evaluates installed semantic
   conditions and propagates application-visible invalidation. Its decisions
   are semantic eligibility evidence, never permission to rasterize, upload,
   submit, present, settle, or recover.
4. Runtime owns mounted presentation orchestration and exact attempt/currentness
   lineage.
5. Host-native owns one separate bounded physical-work Signal runtime per
   native host/device lifecycle. That graph owns upload request progression,
   eligibility, retry, cancellation, supersession, timeout, completion
   reconciliation, recovery scheduling, backpressure, wakes, and shutdown.
6. Host-native separately owns WGPU devices, queues, surfaces, textures,
   buffers, submission indexes, atlas placement, polling, and the physical
   truth of whether an external consequence completed. The native external
   adapter returns typed observations to the physical Signal owner; Signal
   does not store or serialize raw WGPU handles.
7. Runtime Bridge translates current, owner-issued physical completion evidence
   into the installed Query operation correspondence. It does not restamp a
   Signal eligibility decision as physical truth.
8. Foundational supplies portable boundary meaning, canonicalization, and
   reporting vocabulary only after the stronger owners have decided.

Host-native decides physical completion; Query records the admitted typed
observation. Query owns application-visible pending, current, stale, failed,
cancelled, superseded, and unresolved posture. Query never infers completion
from callback order, silence, timeout, disconnect, redraw, wake coalescing, or
transport behavior. The Winit readiness registry remains transport-only.

Terminal projections are reporting-only and are mechanically forbidden from
operational control flow. JSON is an outer evidence/materialization format,
never live state, semantic identity, currentness, or authority. Live recovery
handles remain native/runtime-owned, nonserializable, and unreconstructible
from a string, digest, numeric generation, terminal projection, or JSON.

### Semantic presentation request identity

The admitted Query request/binding must bind, as typed semantic parts:

- mounted presentation identity;
- target surface and native binding lineage;
- exact qualified-layout basis;
- paint-span boundaries and paint-value basis;
- DPI and text-scale basis;
- presentation-attempt and predecessor/currentness lineage.

The exact owner-issued types are architecture-reviewed at Gate E. They use
Foundational canonicalization only to derive evidence identity after typed
admission. The Query-issued installed binding, not its canonical digest, is the
operational authority. The current Query declaration surface exposes only
`WorthQueryAsyncRequestIdentityPart::text`; Gate E therefore must extend the
Query audience facade with a narrow, closed typed request-identity-part contract
before WORTH UI binds a presentation. Formatting these fields into strings is
forbidden.

Gate E also extends the existing Query/Runtime Bridge async vocabulary rather
than pretending the required posture already exists:

- `worth-query` owns a typed `Unresolved` retained-result state alongside its
  existing pending/current/failed/stale/cancelled/retried/revalidating/
  superseded/denied states;
- `worth-runtime-bridge` owns an effects-indeterminate completion class that
  can produce `Unresolved` only from an exact owner-issued native observation;
- `worth-ui-query-binding` owns the presentation-specific translation between
  those general substrate contracts and the exact WORTH UI presentation basis.

Neither extension may encode WORTH UI types inside Query or Runtime Bridge.
Query and Runtime Bridge receive typed opaque identity parts and portable
completion meaning; the WORTH UI binding retains the correspondence to mounted,
layout, paint, DPI, scale, target, attempt, and predecessor types.

Private atlas layout, page numbers, allocator state, entry identities, glyph
upload identities, staging buffers, and submission indexes are excluded from
the Query identity and result shape.

### Managed presentation-resource policy

The Query binding and runtime bridge must define and bound:

- pending-presentation capacity and admission denial before effects;
- backpressure and coalescing without changing semantic attempt identity;
- supersession and cancellation before effects;
- cancellation or unresolved posture after partial external effects;
- duplicate, out-of-order, stale-basis, and foreign-lineage completion denial;
- retry, revalidation, recovery, and reconstruction progression;
- terminal close, cancellation, and exact resource release;
- downstream live-view observation of the retained Query posture.

Only host-native can report physical completion. Query may retain stale/current
application-visible value according to the installed failure policy, but it
cannot upgrade an unresolved or indeterminate native observation to completed.

### Two non-interchangeable Signal domains

Phase 5 contains exactly two Signal domains with different construction
authority, runtime identity, declarations, local aspect slots, capacities,
effects, and shutdown:

| Domain | Construction owner | Owns | Must not own |
|---|---|---|---|
| Query semantic presentation graph | Query/Runtime Bridge substrate, installed for WORTH UI only through `worth-ui-query-binding` | installed presentation-resource and completion meaning, semantic request currentness, continuations, retained Query posture, and precise downstream invalidation | raster, atlas, WGPU, native retry/effect authority, raw device resources |
| Native physical-work graph | `worth-ui-host-native`, exactly once in each native host/device lifecycle | bounded atlas-upload and presentation-work progression, ready work, retry/timeout/cancel/supersede, physical-completion reconciliation, recovery scheduling, backpressure, wakes, and quiescent shutdown | Query state, application-visible meaning, font/layout decisions, raw WGPU handles or atlas storage |

The physical owner placement is mandatory, not provisional. The manifest audit
shows that `worth-ui-host-native` is the strongest owner because it already
owns the WGPU/resource/poll/recovery/shutdown truth and its workspace already
qualifies `worth-signal` as a substrate dependency. Gate D adds a direct
`worth-signal.workspace = true` dependency to host-native. Host-native's only
*WORTH UI* dependency remains `worth-ui-host-contract`; it imports neither
Query nor Runtime Bridge. Runtime does not own or construct the physical
Signal runtime.

The native Signal graph may run on a dedicated bounded worker, following WORTH
Store's `PhysicalWorkSignalOwner` precedent, but the worker owns only Signal
state and bounded typed mailboxes. The native event/device thread remains the
only owner of WGPU effects. The Winit readiness registry transports a
level-triggered wake from ready physical work to that event thread; it is not a
queue, retry policy, timer, completion graph, or scheduler.

Signal aspects are producer-local runtime slots. Query-semantic aspect slots
and native-physical aspect slots are disjoint and cannot be persisted, shared,
serialized, declared as Foundational aspect identities, or copied into Query
request identity. A topology or compile proof must reject a shared ambient
graph, a second physical runtime, and any operation evaluated by both graphs.

## Decisive Product Courtroom: `HP-03`

### Real world

Use `MountedPresentationWorld::qualified_text` through the ordinary public
application, Query admission, mounting, runtime, headless, and native product
boundaries. The world uses the exact qualified profile and explicitly owned
application font packs. It must not expose a certification-only alternate
driver, direct state mutation, hidden font injection, or test-only atlas path.

The checked-in corpus includes:

- Latin with combining marks and canonical-equivalence twins;
- Greek and Cyrillic;
- Arabic joining, marks, digits, paired brackets, and embedded English;
- Hebrew with numbers;
- Devanagari and Bengali conjuncts;
- Thai and Khmer dictionary-sensitive wrapping;
- Hangul and CJK punctuation/wrapping;
- mixed-script fallback and one valid unassigned scalar using Last Resort;
- every exact Unicode 17 RGI emoji sequence, including text/emoji variation
  selectors, keycaps, flags, tags, modifiers, gender/family/person sequences,
  and multi-person ZWJ sequences;
- hard breaks, empty and trailing lines, tabs, whitespace, narrow/wide wraps,
  maximum lines plus ellipsis, clipping, maximum lawful paragraph bytes, and
  multiple style/fallback/paint runs;
- one line containing a small red application-font run, a large green run
  from the same family, a differently sized blue italic Arabic run from
  another family, and intrinsic-color emoji.

The application owns every non-profile font byte. No oracle or product path
depends on a proprietary or ambient face.

### Hostile sequence

Against the same mounted world:

1. present the mixed line and multilingual corpus;
2. change one paint value without changing text, spans, layout inputs, or
   generations;
3. change a paint-span boundary through a shaping-sensitive cluster;
4. introduce repeated and new alpha and color glyphs;
5. fill alpha and color atlases to their qualified limits;
6. retain live layouts that pin entries, remove unpinned candidates, and
   admit a deterministic replacement;
7. request a candidate that cannot fit without evicting live entries;
8. change pure DPI while preserving logical layout inputs;
9. change framework text scale and then paragraph width;
10. inject before-effect denial, upload failure, and effects-indeterminate
    settlement at the real external port;
11. destroy layout, raster, atlas, retained draw-list, and target-derived
    state independently, then reconstruct from mounted authority;
12. perform an unchanged turn;
13. close while no text work is pending and while one staged upload is held.

### Independent observations

The courtroom binds four observations to the same mounted identities:

1. Phase 4 Unicode, shaping, and layout fixtures adjudicate the immutable
   input layout and original-range geometry. Phase 5 does not regenerate them.
2. A pinned independent raster oracle adjudicates alpha coverage, color-layer
   order/compositing, palettes, bitmap strike selection, resampling,
   fractional origin, extents, and content digests for admitted source types.
3. An independent atlas model adjudicates key identity, separate alpha/color
   capacity, placement, pins, deterministic unpinned eviction, staging bytes,
   generations, and reconstruction.
4. Headless transcripts, retained-target readback, and compositor-visible
   capture adjudicate glyph-run attribution, paint-span RGBA, ordering,
   clipping, transparent baseline, and representative real pixels.

Exhaustive internal evidence and representative external evidence are both
required. A screenshot cannot prove the full RGI set; a model cannot prove the
real Windows/WGPU boundary.

### Required verdict

- every raster record names the exact layout attribution, font collection and
  profile generations, face, glyph, original cluster range, variation and
  palette selection, raster source, scale, origin, extent, format, and content
  digest;
- every native glyph-run command names the mounted mechanic, layout, paint
  span, original range, exact atlas entry, and logical straight foreground;
- alpha glyphs receive foreground color only in the qualified presentation
  pipeline; intrinsic-color glyphs preserve their qualified colors and ignore
  adjacent foreground;
- pure DPI preserves layout identity, line/caret/selection geometry, and
  original ranges while replacing raster and atlas generations;
- text-scale or width change replaces layout before presentation;
- color-value-only change reuses the exact layout and atlas entries and
  changes only affected paint payloads and damage;
- a boundary change re-itemizes only the intersecting run and may not split a
  grapheme or shaping cluster;
- ordinary atlas admission is complete before rasterization, allocation,
  upload, acquisition, or presentation effects;
- live entries are never evicted, alpha and color resources never alias, and
  failure never silently degrades quality or color;
- reconstruction starts from mounted/runtime authority, not a stale atlas,
  raster cache, draw list, transcript, or pixels;
- unchanged work performs zero analysis, fallback, shaping, layout,
  rasterization, atlas mutation, upload, acquisition, or presentation;
- close reaches an exact zero census for pages, entries, pins, plans,
  reservations, staging buffers, GPU buffers/textures, readbacks, pending
  presentations, and recovery authorities.

### Mandatory mutants

The proof must turn red for each named fault:

- reshape, refallback, rebreak, or system-font substitution in a consumer;
- scalar fallback, cluster split, dropped RGI sequence, ignored FE0E/FE0F,
  dropped COLR layer, unsupported color-source admission, or emoji tinting;
- alpha/color atlas merger, stale key field, live-entry eviction,
  registration-order eviction, growth past capacity, or unreported staging;
- stale-DPI raster reuse or layout replacement on a pure-DPI change;
- single-color substitution, visual-order color assignment, intrinsic-color
  foreground substitution, or layout regeneration on a color-only edit;
- headless/native transcript or attribution mismatch;
- stale raster/cache/draw-list reconstruction;
- complete retained-document scan or complete glyph reraster on local work;
- commit before the external handoff, rollback omission, cleanup receipt loss,
  or missing predecessor/feature behavior.

Each control must inject or causally emulate its named fault at the production
boundary. A passing nearby test, fixed marker, or self-authored output is not a
mutation control.

### Test economy

Execute each unique focused command, exhaustive corpus, model world, and real
Windows/WGPU world once per applicable test run. Multiple assertions may inspect
distinct observations from one shared execution. Retain concise diagnostics for
failures that need reproduction, but do not require receipt, nonce, portfolio,
digest, or publication machinery. Report progress and duration at the test or
shard level.

## Product Decision Lock

### Authority and truth graph

`worth-ui-text` owns all decisions that interpret fonts or create glyph
pixels. `worth-ui-host-native` owns all native resource allocation, atlas
placement, upload, queue, texture, physical completion truth, and one bounded
physical Signal lifecycle. Runtime owns the mounted layout, selects ordinary
versus reconstructive semantic work, and privately orchestrates text meaning
with the native facade. `worth-ui-host-contract` carries only borrowed, inert
records and typed observations across that boundary.

Therefore:

- host-native's only WORTH UI dependency remains host-contract; it additionally
  depends directly on the generic `worth-signal` substrate for its private
  physical-work runtime;
- host-native imports neither Query nor Runtime Bridge, and runtime does not
  construct or retain a native physical Signal graph;
- host-native does not import `worth-ui-text` or receive source text/font APIs;
- text does not import host-native, wgpu, a platform facade, or a resource
  registry;
- host-contract does not own layout/font/raster/atlas resources or policy;
- runtime does not duplicate font interpretation, raster policy, atlas
  placement, or GPU effects;
- headless receives exact borrowed records and may retain only its bounded
  semantic transcript, not framework-owned layout or font bytes;
- accessibility, measurement, hit testing, and inspection consume the exact
  qualified layout identity and cannot open a second shaping/raster lane.

### Staged raster and atlas transaction

An ordinary presentation uses this exact order:

1. runtime derives a `UiGlyphRasterDemandBatch` from its owned qualified
   layout, mounted paint spans, damage, and raster scale;
2. host-native admits one bounded physical Signal resource request for the
   exact host lineage, presentation attempt, demand, pin transition, target,
   and capacity basis;
3. under that request, host-native performs an effect-free lookup, capacity,
   pin, eviction, and placement plan against current native atlas state;
4. host-native returns a move-only `UiNativeTextAtlasPlan` and the exact miss
   subset as borrowed host-contract demand records;
5. text validates that subset against the original demand and rasterizes only
   the admitted misses into typed alpha/color batches;
6. runtime passes borrowed raster-batch views and the still-unconsumed native
   plan back to host-native;
7. the physical Signal runtime admits ready upload work; the native external
   port validates exact plan/batch identity, stages uploads, submits or polls
   WGPU work, and returns typed attempt/currentness/completion envelopes;
8. the physical Signal runtime rejects stale, duplicate, foreign,
   out-of-order, cancelled, superseded, or timed-out observations and
   reconciles the lawful current completion;
9. only that reconciled physical completion lets host-native settle atlas
   entries, use epochs, and pins; Gate F later joins it to glyph-run commands,
   target pixels, presented affinity, and the Query operation correspondence.

Validation, Signal admission, capacity reservation, and eviction choice occur
before rasterization or external effects. The replaceable external port
returns inert external observations and receipts; it may not mint Signal,
atlas, Query, or framework lifecycle verdicts. Host-native physical Signal
reconciliation maps those observations into typed physical completion,
before-effect denial, retry/cancellation, or effects-indeterminate recovery.
Runtime Bridge later maps only exact reconciled evidence into Query meaning.

The native plan is move-only, private-field, non-serializable, and has no
public constructor or `Clone`. Dropping an uncommitted plan releases only its
effect-free reservation. An effects-indeterminate result carries recovery
authority; it is never flattened into a boolean or generic error.

### Raster demand, records, and sources

The concrete text-owned record names:

- layout and mounted attribution;
- font collection/profile generation and exact face identity;
- glyph identity and original cluster byte range;
- variation coordinates, palette, size, DPI scale, fractional origin, and
  qualified raster source;
- raster origin/bearing, dimensions, stride, format, bounded bytes, content
  digest, and actual cost;
- alpha versus intrinsic-color posture.

Raster behavior is exactly the qualified profile:

- source order is color outline, color bitmap, alpha outline, Last Resort;
- alpha uses `R8Unorm`; intrinsic color uses `Rgba8UnormSrgb`;
- outlines use qualified font instructions and grayscale antialiasing;
- origins are quantized to 1/64 pixel;
- color is composited and premultiplied in linear RGB before storage;
- COLRv0/CPAL, COLRv1/CPAL, CBDT/CBLC, and admitted sbix PNG/one-hop-dupe are
  supported with the qualified semantics;
- SVG, sbix jpg/tiff, malformed graphs/enums/images, or unsupported sources
  are rejected at application-font admission, never silently flattened;
- bitmap strike selection is nearest qualified strike followed by
  deterministic premultiplied-linear bilinear resampling;
- ink bounds are derived from the same qualified outline/color/bitmap
  semantics used by rasterization, including variations and nonzero alpha.

The exhaustive RGI proof must raster every exact sequence-to-glyph mapping.
Representative real pixels cover every admitted raster source and sequence
class, but do not replace the exhaustive proof.

### Cache identity and equivalence

An atlas cache key contains exactly the profile fields:

- font collection generation;
- profile generation;
- face;
- glyph;
- variation coordinates;
- palette;
- size;
- raster source;
- DPI scale;
- fractional origin.

Layout identity is attribution, not part of raster equivalence. Two layouts
may safely reuse an entry only when the complete raster key and content digest
agree. Every draw command still records its own layout, mechanic, paint span,
and original range. This preserves exact attribution without forcing duplicate
pixels.

No numeric generation alone establishes collection, layout, raster, or atlas
identity. Cross-lineage equality is impossible without the exact opaque
identity/lease.

### Alpha and intrinsic-color presentation

Alpha and color atlases are physically and logically separate. An alpha entry
contains coverage only; the qualified glyph-run shader applies the logical
straight foreground and premultiplies at the required boundary. An
intrinsic-color entry contains qualified RGBA and ignores text foreground.

Foreground is selected by original-range paint-span identity before visual
reordering. A glyph may not straddle two paint spans. Visual order changes draw
order, not paint ownership. Bidi, ligatures, marks, emoji, clipping, damage,
headless transcript, and native commands must retain the same attribution.

### Atlas lifecycle, pinning, admission, and eviction

The profile caps are exact:

- four 1024 by 1024 alpha pages;
- two 2048 by 2048 color pages;
- 8,192 total entries;
- 512 by 512 maximum glyph extent;
- 36 MiB total atlas texels;
- 8 MiB staged upload bytes.

Each retained layout pins its exact alpha and color raster keys. Runtime
explicitly advances and releases pin sets at the presentation settlement
boundary. Native code may not infer release from frame timing, absence in one
delta, object address, or cache pressure.

Candidate admission accounts for existing entries, live pins, exact misses,
page placement, staging, and peak replacement overlap before rasterization or
effects. Only unpinned entries may be candidates. Eviction order is
deterministic: oldest completed-use epoch, then canonical raster-key bytes.
Registration, hash-map, allocator, or GPU enumeration order is never a tie
break.

Packing representation is a private native mechanism. It may change only if
it remains bounded and deterministic, does not alter public identity, admits
or denies the same governed courtrooms, preserves pins, and reports exact
work/resources. The specification does not require maximal packing.

Saturation returns a named typed before-effect denial. It may not evict live
entries, grow without qualification, reduce quality, convert color to alpha,
partially upload, or partially publish. Reconstructive saturation is a
separate named posture and carries recovery authority.

### DPI, text scale, and paint-only change

- Pure DPI replaces raster and atlas generations only. The exact layout,
  line/caret/selection geometry, original ranges, and logical damage remain.
- Framework text-scale or paragraph-width change replaces layout before
  presentation and then derives new raster demand.
- A color-value-only change with unchanged text, font collection, style-span
  boundaries, width, locale, direction, and text scale reuses the exact layout,
  raster entries, measurements, and interaction geometry. It updates only the
  affected foreground payloads and logical damage.
- A paint-span boundary change may re-itemize and reshape only the intersecting
  run. It remains cluster-safe and paragraph-local.

### Reconstruction, failure, and cleanup

Reconstruction is owner-issued work distinct from Initial, Delta, and
Unchanged. It carries a complete current mounted layout/raster demand and may
rebuild any destroyed derived structure: raster cache, atlas pages/index,
pins, retained glyph-run commands, target pixels, and presentation affinity.

Reconstruction never trusts a stale derived structure as source. Independent
destruction mutants remove each structure and require identical layout
identity, raster content, atlas model, transcript, and representative pixels.
The next ordinary delta must return to local cost.

Before-effect denial preserves predecessor derived state and performs zero
raster/allocation/upload/acquisition/presentation work. Effects-indeterminate
settlement retains typed recovery authority and cannot publish candidate
semantic state. Cleanup drains/retries every staged/pending/recovery owner and
reports an exact class census; bookkeeping tokens cannot substitute for actual
resource lifetime.

The physical Signal runtime supersedes manual lifecycle progression. A native
collection may retain staging buffers, submission indexes, textures, atlas
entries, or owner-issued recovery capabilities because those are physical
truth. It may not decide readiness, retry, timeout, cancellation,
supersession, completion currentness, recovery scheduling, or shutdown order
through a bespoke pending-work queue, polling loop, callback-settlement route,
timer wheel, or recovery coordinator. Those decisions belong to the one
physical Signal runtime. Close revokes admission, drains or retains bounded
physical obligations, reconciles terminal completions, joins the worker, and
only then may report zero.

### Typed outcomes

The implementation has compiler-visible distinctions for at least:

- malformed or stale raster demand;
- unsupported or malformed raster source;
- glyph extent, entry, page, texel, staging, or pinned-capacity denial;
- stale layout, collection, profile, scale, plan, pin, or presentation
  affinity;
- effect-free rejection;
- external-effects indeterminate with recovery authority;
- reconstruction required;
- presented and committed settlement;
- incomplete cleanup with retained recovery authority.

String labels may appear in diagnostics and serialized evidence only after a
typed owner has made the decision. They are never control flow or identity.

## Exact Cost and Resource Contract

Every relevant layer reports actual work, not inferred outcome labels or
hard-coded expected values.

Ordinary counters include:

- damaged text mechanics and paint spans;
- glyph-demand records considered;
- atlas key lookups, hits, misses, page/placement probes, and eviction
  candidates;
- rasterized alpha/color glyphs and texels;
- staged/uploaded bytes and copy/write operations;
- glyph-run commands, clipped commands, damage regions, draw calls, render
  passes, acquisitions, submissions, presentations, and presented pixels;
- pin additions/releases and resource current/peak counts;
- retained-layout, paragraph, command, or cache scans/clones.

Every UI-specific locality world also records the *realized* Milestone 13
frontier observation from the graph that performed the invalidation. The
required Signal fields are:

- `source_output_deltas_consumed`;
- `direct_subscriber_edges_examined`;
- `reverse_index_bucket_probes` and `reverse_index_candidates_returned`;
- `candidates_rejected_by_aspect_contract`;
- `candidates_rejected_by_scope`, with WORTH UI's independently checked
  partition/detail/range rejection breakdown;
- `candidates_rejected_by_comparator`;
- `direct_settlements_produced`;
- `work_items_admitted` and `work_items_merged`;
- `ready_items_enqueued` and `ready_items_popped`;
- `stale_work_rejected`;
- `nodes_evaluated` and `produced_deltas_emitted`;
- `propagation_stops` from unchanged output;
- `non_semantic_node_visits`;
- `maximum_ready_frontier_width` and `retained_ready_frontier_width`;
- `topology_revision_revalidations` and `rejected_topology_mutations`;
- `batch_local_allocations` and `peak_batch_memory_items`;
- `recovery_reconstruction_work`.

Predicted work, a declared zero, or an outcome-derived estimate cannot
substitute for the performed observation.

Reconstructive counters include the same dimensions under a distinct lane and
name every complete rebuild. Ordinary counters may not hide reconstructive
work, and a successful result may not imply a zero counter.

Required slope worlds cover retained sizes 1, 32, 2,048, and 4,096; first,
middle, and last local glyph changes; repeated hits; bounded new misses; alpha
and color saturation; mixed maximum overlap; and one unchanged turn. Ordinary
local work scales with changed/damaged demand plus atlas probes and emitted
commands, never all retained paragraphs, layouts, glyphs, entries, or pages.

The thirty-two authoritative worlds are executed as sixteen source-bound
shards. CI assigns one shard to each fresh worker; local closure admits exactly
eight shard processes in a large-first wave (`8..15`), followed by the eight
ordinary/middle-retained shards (`0..7`), under one shared eight-and-a-half-
minute runner deadline inside the nine-minute CI envelope. The deterministic
join rejects partial, duplicate, missing, or timed-out evidence, and no test or
shard may reach ten minutes.

At each retained size, the independent matrix exercises these axes separately:

1. one content-only paragraph edit;
2. one width-only paragraph change;
3. one paint-value-only span change;
4. one paint-span-boundary change;
5. one pure-DPI change;
6. one atlas miss among many hits;
7. one native upload completion among many mounted presentations;
8. one layout removal with both shared and exclusive raster-key pins.

Each row combines the realized Signal fields above with analyzed bytes, bidi
contexts, fallback probes, shaped runs/glyphs, raster keys considered, raster
misses produced, atlas hits/misses, placement/eviction work, staged bytes,
submissions, damage, presentations, and pixels. Disjoint aspect, partition,
detail, and byte-range consumers must be rejected before dirty mutation or
enqueue. Unchanged output stops immediately.

The terminal census enumerates every resource class compiler-totally. Adding a
new resource class must require updating the census, schema, serializer,
hostile controls, and closure validator.

## Destination Topology

The intended ownership tree is:

```text
workspaces/worth-ui/crates/
  worth-ui-host-contract/src/qualified_text/
    layout_view.rs
    raster_demand_view.rs
    raster_batch_view.rs
    glyph_run_view.rs

  worth-ui-text/src/raster/
    demand.rs
    key.rs
    source.rs
    alpha.rs
    color/
      colr.rs
      bitmap.rs
      compositing.rs
    batch.rs
    cost.rs
    reconstruction.rs

  worth-ui-runtime/src/native_platform/text_presentation/
    preparation.rs
    rasterization.rs
    settlement.rs
    recovery.rs

  worth-ui-query-binding/src/presentation_async/
    declaration.rs
    request_basis.rs
    runtime_bridge.rs
    semantic_invalidation.rs
    observation.rs
    retained_posture.rs
    terminal_projection.rs

  workspaces/worth-query/crates/worth-query/src/
    application/declaration/async_resource/request_identity.rs
    runtime/async_result_state.rs

  crates/worth-runtime-bridge/src/source/async_declaration/completion/
    completion.rs
    indeterminate.rs

  worth-ui-host-native/src/native/physical_work_signal/
    mod.rs
    construction.rs
    identity.rs
    declarations/
      mod.rs
      aspects.rs
      resources.rs
    routing/
      mod.rs
      request.rs
      external_observation.rs
    completion_reconciliation.rs
    wake_delivery.rs
    shutdown.rs
    observation.rs
    counters.rs
    worker.rs

  worth-ui-host-native/src/native/text_atlas/
    plan.rs
    capacity.rs
    alpha.rs
    color.rs
    placement.rs
    pinning.rs
    eviction.rs
    upload.rs
    ownership.rs
    reconstruction.rs
    census.rs

  worth-ui-host-native/src/native/presentation/text/
    validation.rs
    commands.rs
    pipeline.rs
    transaction.rs
```

Names may be refined during implementation, but semantic ownership and
dependency direction may not. Every file has one named responsibility; no
`helpers`, `common`, `util`, or compatibility module may absorb the protocol.
All code and test files remain within the repository line cap unless an
explicit specification exemption is added before implementation.

Compile/topology enforcement proves:

- native and headless cannot import text shaping/raster implementation;
- text cannot import native/GPU/resource owners;
- consumers cannot construct or retain framework layout/raster authority;
- only the private runtime native-platform orchestrator sees both text and
  native effect facades;
- only `worth-ui-query-binding` imports Query for WORTH UI; runtime and
  host-native import no Query facade in any target;
- host-native is the sole WORTH UI crate that imports `worth-signal` for native
  physical work, constructs exactly one physical runtime per host/device
  lifecycle, and exports no Signal or atlas-control authority publicly;
- the Query/Runtime Bridge semantic graph and host-native physical graph have
  distinct runtime identities, declarations, local aspect slots, capacities,
  observations, and shutdown; neither graph accepts the other's handles or
  envelopes;
- WGPU devices, queues, surfaces, textures, buffers, atlas stores, staging
  owners, and submission indexes remain under native resource homes and never
  enter Signal-owned generic maps;
- no parallel local scheduler, pending-work progression queue, retry queue,
  timer wheel, callback settlement authority, recovery coordinator, unbounded
  command mailbox, or second Signal worker exists beside the physical owner;
- the Winit readiness registry is wake transport only and cannot decide
  readiness, completion, retry, cancellation, or recovery;
- terminal presentation projections cannot flow into preparation, planning,
  rasterization, upload, settlement, retry, or recovery decisions;
- plan/reservation/pin/recovery types are concrete and non-forgeable;
- no raw aspect/field/font/raster/atlas string decides identity or legality;
- no system-font API, duplicate shaper, duplicate layout, or consumer raster
  lane is reachable;
- atlas and raster implementation stays outside public application APIs.

The topology paths above are responsibility contracts. Physical atlas and
WGPU mechanics stay in their existing resource modules; the
`physical_work_signal` owner stores only Signal topology/progression state,
bounded routing, identities, performed observations, and completion
correspondence. It does not become a generic native-resource bag.

## Internal Implementation Gates

These are implementation gates inside Phase 5, not roadmap phases or separate
QA state machines.

### Gate A: predecessor and protocol exactness

- run the focused predecessor contract and integration tests affected by the
  current change;
- freeze borrowed demand/batch views, move-only plan, typed settlements,
  resource census, source mappings, and negative compile twins;
- do not treat readiness checks as completed behavior.

### Gate B: demand and alpha outline raster

- derive exact local demand from runtime-owned layout and paint spans;
- implement qualified outline coverage, variation-aware ink, origins, extents,
  batches, digests, and ordinary/reconstructive costs;
- prove no consumer reshape/system-font lane.

### Gate C: intrinsic color and exhaustive RGI

- implement every admitted color source and reject every unsupported/malformed
  form at the lawful boundary;
- prove exact sequence-to-glyph mapping for all 3,953 Unicode 17 RGI records;
- prove ordered compositing, palettes, bitmap semantics, nonzero-alpha ink,
  selector posture, and no tint/split/layer loss.

### Gate D: atlas transaction and lifecycle

- implement effect-free lookup/capacity/placement/eviction planning;
- implement separate alpha/color resources, explicit pins, staged uploads,
  settlement, rollback, recovery, and exact census;
- integrate the completed Signal Milestone 13 substrate before adding native
  progression; do not clone its mechanism or implement against the pre-M13
  local baseline;
- construct exactly one bounded host-native physical Signal runtime per native
  host/device lifecycle, with owner-issued identity, exact native operation and
  aspect declarations, bounded routing, performed observations, wake delivery,
  completion reconciliation, and quiescent shutdown;
- route atlas upload eligibility, WGPU submission attempts, completion polling,
  retry, timeout, cancellation, supersession, recovery scheduling,
  backpressure, and terminal drain through that physical graph while retaining
  WGPU handles and resource truth in native owners;
- expose owner-issued inert observations bound to exact native transaction,
  host lineage, mounted presentation basis, and currentness, including pending,
  completed, rejected-before-effects, lawful rejected-after-rasterization,
  effects-indeterminate, recovery-required, and recovery-resolved postures;
- keep live recovery authority native/runtime-owned and nonserializable, keep
  readiness transport-only, and add no Query dependency;
- retire or demote every manual scheduler/retry/timer/callback/recovery
  progression structure superseded by Signal; retain only bounded physical
  resource registries and typed external effect adapters;
- prove saturation, cancellation, retry, timeout, stale/duplicate/out-of-order
  completion rejection, recovery reconciliation, pending census, and bounded
  shutdown before real presentation integration;
- rerun the atlas ownership, pinning, physical Signal lifecycle, recovery, and
  resource tests because the physical Signal owner changes their causal basis.

### Gate E: paint, scale, and Query async binding

- bind original-range paint spans through headless and native commands;
- prove color-only layout/atlas reuse, cluster-safe boundary changes, pure-DPI
  raster replacement, and text-scale/width layout replacement.
- add the presentation-specific Query async declaration and installed binding
  under `worth-ui-query-binding`;
- extend Query's audience declaration facade with closed typed async request
  identity parts and add the Query-owned `Unresolved` retained-result state;
- extend Runtime Bridge completion meaning with an effects-indeterminate class
  whose only lawful Query projection is `Unresolved`;
- install the Runtime Bridge correspondence for exact mounted presentation,
  layout, paint, DPI, text-scale, target, attempt, and predecessor bases;
- construct the distinct Bridge-owned Query-semantic Signal graph with exact
  producer-local aspects and mounted-instance/layout/range/raster-key/
  presentation-attempt/target/host-lineage partitions;
- translate Query pending/current/stale/failed/cancelled/superseded/unresolved
  posture into lawful WORTH UI runtime behavior without importing Query into
  runtime or host-native;
- preserve content, width, paint-value, paint-boundary, pure-DPI, upload-
  completion, and pin-release locality through the Query lifecycle using
  Milestone 13's reverse index, immediate-dependency provenance,
  deduplication, comparator stops, and performed frontier observation;
- prove the two Signal domains have no shared runtime, aspect slot, operation
  family, completion authority, or ambient graph.

### Gate F: pixels, reconstruction, and cost

- integrate with the real retained target and Windows/WGPU world;
- feed owner-issued native completion through physical Signal reconciliation,
  Runtime Bridge, and Query; then propagate only the exact semantic downstream
  invalidation and prove the exact ten-event transition trace defined below;
- prove cancellation separately as a causal before-effects control and prove
  cancellation after partial external effects becomes unresolved rather than
  completed;
- prove representative pixels for each source/class, transparent baseline,
  clipping, damage, external capture, destruction/reconstruction, next-local
  delta, unchanged zero, slope, and cleanup;
- execute the independent 1/32/2,048/4,096 UI locality matrix for content,
  width, paint value, paint boundary, DPI, one miss/many hits, one completion/
  many presentations, and shared/exclusive pin release; bind realized
  Milestone 13 counters to the Phase 5 domain counters;
- observe Query posture, native pixels, retained presentation affinity, and
  terminal physical-Signal/native/Query resource census in the same `HP-03`
  world.

### Gate G: final review

- review each material guarantee against its real production path, independent
  oracle, named adversarial case, and structural counter where applicable;
- require causal controls for complete subscriber-closure walking, late aspect
  filtering, late partition/range filtering, global mounted invalidation,
  paint-to-layout widening, DPI-to-layout widening, dropped deduplicated cause,
  global-union scope, hidden document scan, completion outside physical Signal,
  Query current publication outside Query admission, Signal-as-effect
  authority, stale/duplicate/out-of-order completion, and leaked terminal
  Signal/native resources;
- run the focused test suite, required constitutional checks, and affected CI
  lanes on the final commit;
- complete only when Query async presentation and all inherited guarantees pass
  code review.

## QA Considerations and Required Tests

Phase 5 review covers these categories without assigning row IDs or maintaining
progress state in this document:

- architecture and authority: Phase 4 remains the sole layout authority;
  consumers cannot reshape or consult system fonts; Query and physical Signal
  graphs remain distinct; host-native alone owns atlas and WGPU resources;
- correctness: qualified layouts produce attributable alpha/color raster,
  exhaustive Unicode 17 RGI mappings preserve cluster and intrinsic-color
  semantics, and mixed bidi paint follows original source ranges;
- lifecycle and recovery: atlas planning is effect-free, live entries remain
  pinned, eviction is deterministic, stale/duplicate/out-of-order completion is
  rejected, indeterminate effects require recovery, and terminal census is zero;
- performance and resources: pure DPI preserves layout, color-only edits reuse
  layout and atlas state, unchanged work is zero, and the independent
  1/32/2,048/4,096 locality matrix exposes rescan or global-fanout regressions;
- integration: headless attribution, retained-target pixels, compositor-visible
  pixels, Query async state, mounted lineage, and reconstruction agree through
  the real public path;
- test quality: fixtures use qualified assets and real boundaries, oracles do
  not import disputed production logic, adversarial cases fail for their causal
  reason, and expensive worlds are shared rather than duplicated.

The physical Signal integration changes atlas ownership, lifecycle, completion,
recovery, and shutdown behavior, so those focused tests must run on the current
commit. Historical Phase 5 ledger state is irrelevant to that decision.

### Async-presentation causal evidence

The main is the ordinary public `HP-03` product path through application,
Query admission, `worth-ui-query-binding`, mounting, runtime, host-native
transaction, native polling/completion, Runtime Bridge, and retained Query
result. It records one exact, independently adjudicated ten-event transition
trace. The structural counter `presentation-transitions=10` means exactly:

1. attempt A becomes `pending`;
2. attempt B supersedes A;
3. A's stale completion is rejected without changing B;
4. B becomes `completed` from its owner-issued native observation;
5. a duplicate or out-of-order B observation is rejected without a new result;
6. attempt C becomes `pending`;
7. C becomes `unresolved` from effects-indeterminate;
8. C records `recovery-required` without live authority entering Query;
9. reconstruction resolves C into a fresh current successor;
10. terminal close releases the managed Query resource.

Cancellation is a separate hostile/control transition and is not counted in
that main trace. The same execution binds mounted and native lineage, retained
presentation affinity, representative native pixels, pending-resource census,
and zero terminal census.

The hostile family `bypass-query-or-stale-presentation-completion` must
causally reject each of these faults at the production boundary:

- runtime or host-native publishes completion without Query admission;
- native code publishes physical completion, retry, recovery, or cancellation
  outside the one host-native physical Signal runtime;
- a completion from an older or foreign presentation basis becomes current;
- a duplicate or out-of-order completion creates a new current result;
- effects-indeterminate is flattened into completed;
- a Signal condition authorizes raster, upload, submit, present, or settle;
- either Signal graph accepts the other graph's runtime identity, aspect slot,
  resource handle, completion envelope, or shutdown receipt;
- serialized material reconstructs live native recovery authority;
- a terminal projection, formatted string, digest, or numeric generation is
  accepted as operational authority;
- a Query-managed presentation resource survives terminal close.

The control uses typed owner-issued corruptions or an independently modeled
transition adjudicator. Merely printing the mutation label, editing JSON after
execution, or rerunning the lawful main does not satisfy the row.

### Cost and causal-frontier evidence

The cost main is one shared, deterministic UI frontier world, not eight cargo
targets and not a table of asserted constants. At each retained size 1, 32,
2,048, and 4,096 it applies every independent axis listed in the Exact Cost and
Resource Contract and records:

- the performed `SignalInvalidationPerformedObservation` (or its exact M13
  successor) from the Query-semantic graph for semantic invalidation;
- the performed physical-Signal request/completion observation for native
  upload, poll, retry, cancellation, recovery, wake, and shutdown work;
- the WORTH UI domain counters from analysis through pixels and terminal
  census;
- exact source, immediate dependency, partition/detail/range, mounted
  instance, layout, raster-key set, presentation attempt, target, and host
  lineage identities.

The independent adjudicator predicts the exact immediate subscribers from a
separately authored dependency model, compares optimized and deterministic
execution, and proves that irrelevant fanout changes no relevant counter. It
must distinguish zero work from unobserved work and ordinary work from
reconstruction.

The `complete-document-rescan` hostile family is a grouped name for causal
mutants at the exact owners. It includes restoring complete subscriber-closure
walking; moving aspect or scope filtering after enqueue; replacing exact
partition/detail/range with a global union; invalidating every mounted
presentation for one physical completion; widening paint-only or DPI changes
into layout work; dropping one immediate-dependency cause during deduplication;
adding a hidden retained-document scan; and emitting predicted instead of
performed counters. Each mutant must fail for its intended counter or identity
cause while the lawful optimized/deterministic twin remains green.

The hostile execution may use separately authored deterministic
owner-mutant twins in certification when the wrong behavior is an internal
owner algorithm rather than an admissible public input. A twin consumes the
same typed case basis as the lawful owner and is adjudicated against the
immutable owner-issued performed observation before serialization. It may not
construct, edit, default, or replace that observation, and it does not create a
second large-world portfolio. The 32-world result must retain the per-case
outcomes, and the exact shard join must reject a missing,
duplicated, or unknown hostile member. A bounded small-world control may
re-execute the minimum causal cases needed to authenticate the grouped family.

The 32-world matrix runs as ignored, exactly filtered tests in the existing
`application_contracts` integration target. Its shard workers use the explicit
certification-only `CertificationWorker` event-loop posture because Rust's
test harness does not invoke filtered tests on the operating-system process
main thread. Every row must report `certification-worker` and must still prove
that the Winit callback thread matches the worker launch thread. This posture
changes only certification scheduling; it does not weaken the real native
Winit/WGPU/Query world or create a certification binary or test target.

Certification-worker evidence cannot prove the Phase 2 operating-system
main-thread guarantee. That guarantee remains bound to the separately executed
Platform Pulse product process, whose result must report
`main-thread-required`. A change to either posture owner requires rerunning the
affected Phase 2 lifecycle plus Phase 5 locality, async-presentation, and cleanup
tests before Phase 6 relies on the changed behavior. Historical phases do not
reopen.

## Public Developer Experience and Documentation

The existing public application font-pack, text request, measurement, and
interaction APIs remain the authoring surface. Phase 5 adds no required atlas
or raster configuration to application code.

`workspaces/worth-ui/docs/text-platform.md` must explain and demonstrate:

- alpha versus intrinsic-color rendering and foreground semantics;
- supported and rejected application color-font sources;
- original-range paint spans across bidi and clusters;
- DPI versus framework text-scale behavior;
- deterministic atlas capacity, live pins, eviction, saturation denials, and
  why application code does not manage them;
- reconstruction, failure, and retained-layout byte lifetime;
- the exact boundary between text meaning, runtime orchestration, native
  resources, later editing/accessibility semantics, and Phase 6 input;
- the qualified deterministic posture rather than universal machine-font or
  pixel-equivalence claims.

The public text example must compile through the governed shared example
portfolio. It demonstrates multiple application families, explicit family
stacks, variations/features, mixed paint spans, emoji, measurement, and the
absence of atlas knobs. Documentation tests and examples must use real public
imports and product composition roots.

## Must Ship

- concrete qualified glyph demand and alpha/color raster production;
- complete admitted color-source semantics and exhaustive RGI mapping;
- separate bounded native alpha/color atlases with effect-free planning,
  live-layout pins, deterministic unpinned eviction, staged upload, and exact
  cleanup;
- one bounded host-native physical Signal runtime per native host/device
  lifecycle, with typed declarations, current completion reconciliation,
  retry/cancel/timeout/recovery progression, wake delivery, performed
  observations, and quiescent shutdown;
- one distinct Query/Runtime Bridge semantic Signal graph with precise
  application-visible invalidation and no native effect authority;
- original-range paint-span attribution through headless and native pixels;
- pure-DPI, text-scale, color-only, saturation, failure, reconstruction,
  unchanged, and slope behavior;
- real retained-target and compositor-visible pixel evidence;
- exact cost/resource observations and risk-proportionate adversarial tests;
- updated public text documentation and compiled example;
- concise retained diagnostics for failures that need reproduction.

## Must Preserve

- Phase 4 as the sole canonical layout and measurement owner;
- application-owned and profile-owned exact font bytes with no system
  substitution;
- runtime durable layout ownership and host borrowed-view topology;
- one-way dependency direction and host-native's host-contract-only WUI edge;
- disjoint semantic and physical Signal identities/aspects/capacities, with no
  ambient graph, duplicate scheduler, or Signal-owned raw WGPU resource;
- mounted semantic authority over all derived raster/atlas/pixel state;
- existing lifecycle, presentation, failure, recovery, and close semantics;
- typed aspect/field/identity/denial authority;
- frozen historical QA records without treating them as current authority;
- test economy: no duplicate exhaustive or native worlds.

## Acceptance and Phase 6 Handoff

Phase 5 is complete only when:

- exhaustive Unicode/color evidence and the shared real native world pass;
- the exact resource census reaches zero after ordinary, hostile, recovery,
  reconstruction, and close paths;
- both Signal runtimes prove exact construction, distinct identity/declaration,
  bounded capacity, performed counters, stale-completion denial, and terminal
  shutdown, while host-native's native census remains the physical truth;
- the public guide and example are current and governed;
- boundary checks, agent-context checks, format, line-cap, composition,
  dependency, and topology enforcement pass;
- focused tests, affected CI lanes, and required constitutional checks pass on
  the final commit; and
- code review finds the architecture, test realism, oracle independence,
  adversarial coverage, lifecycle handling, and resource evidence adequate.

The Phase 6 handoff is narrow: Phase 6 may trust deterministic,
framework-grade, attributable multilingual and intrinsic-color text pixels,
canonical layout/measurement/interaction geometry, exact last-completed
presentation affinity, bounded native text resources, and reconstruction from
mounted authority. Phase 6 must not redesign font selection, Unicode analysis,
shaping, layout, raster policy, atlas identity, or paint-span ownership while
adding native input and presentation affinity.
