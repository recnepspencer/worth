# Phase5 typed topology bounds: boundary review

> Scope update2026-11-01: this is a historical receipt/plan under the superseded
> public CAD scope. The Scene/Face application and its authoring/contact expansion
> are removed. Distinct generic numerical/provider fixes remain. This record does
> not reopen a public CAD prerequisite; see the [cleanup evidence](milestone-9.19-phase5-public-cad-cleanup-evidence.md).

The next numerical boundary is actual SceneFaceRow u64 Metre/model-local points
entering the same Query-free box/tree mechanics currently used by signed Catalog
geometry. Existing Box2 uses full i64 coordinates, fixture-integer-length-v1 and
fixture.cartesian-xy.v1. Casting or relabeling topology values into that domain is
not valid. Native still owns contour eligibility and publication; bounds supply
only conservative numerical candidates.

The sole implementation becomes IntegerBox2<Space>. Box2 names the concrete
existing SignedFixture specialization. Independent review ruled this a genuine
domain specialization, with no adapter, second constructor or authority conversion.
It also preserves existing Box2::new and constant inference, which a generic type
default alone cannot promise in expression position. ModelLocalMetre is the other
sealed Space, with u64 coordinates and the existing topology unit/frame.
That marker names a coordinate/unit/frame class, not a particular model's frame
instance. Actual Scene/context custody still supplies common-origin affinity;
the numerical primitive gains no runtime model ID or cross-model transformation.

Box2Coordinates<C=i64> remains the public raw {lower,upper} serde carrier; do not
add phantom fields or serialized domain tags. Validated IntegerBox2 remains
non-deserializable except through checked construction. Its Space marker is only
in the validated value. Union/overlap require identical Space at compile time.
BoxNode appends Space after existing Id/Source parameters. BoxEditStorage has a
defaulted Space trait parameter so existing signed storage implementations retain
their meaning. The same cold build, AVL edits, ordering and query algorithms serve
both domains, with original storage/checkpoint receivers unchanged.

Exact Morton ordering compares 65-bit endpoint sums. Normalize each coordinate by
its sealed class minimum before summing: signed min -2^63 reproduces the existing
+2^64 bias exactly; unsigned min0 requires no bias. i128 sum/difference and u128
normalized representation are exact for both classes. Raw-coordinate/source ties
retain full identity and no averaging, epsilon or truncation enters. Segment-box
contact consumes the same sealed integer orientation owner in either typed Space.

Actual callers are the existing value owner tests, consumer value-edit tree,
immutable box provider and maintained Catalog geometry provider. Existing callers
use Box2/default storage meaning; none needs a new serialization contract. The
real new consumer is SceneFaceRow::bounds(), an exact u64 enclosure derived from
its actual projected points. No bounds are written into Native graph truth or
retained in a private cache.

Independent review accepted this API shape. Existing signed byte/unit/frame
meaning, controlled tree storage and constructor inference must remain qualified.
Maintained Scene Source/provider, exact face contact, scene-context creation and
spatial locality remain subsequent work; a numerical bound is not topology proof.
