# Phase5 typed topology bounds: implementation plan

Boundary review and independent API ruling are complete before Rust edits.

1. Add sealed SignedFixture/ModelLocalMetre spaces to consumer_values. Generalize
   the single numerical box implementation as IntegerBox2<Space>, with Box2 the
   existing concrete signed specialization. Preserve Box2Coordinates<C=i64> raw
   serde shape, bounds denial, constants and validated construction. Add the sealed
   integer class minimum used by exact centroid normalization.
2. Thread trailing Space through BoxNode and defaulted Space through BoxEditStorage
   and build/edit/query/order functions. Share all mechanics. Split AVL rotations
   into their semantic child module if generic signatures exceed the400-line cap.
   Preserve original checkpoints, source resolution, occurrence identity and cell
   reuse; no new tree, storage ledger, flags, dependencies or build roots.
3. Generalize the existing segment-box functions over typed Space and sealed integer
   coordinates. Preserve current signed semantics and full-range orientation owner.
4. Add SceneFaceRow::bounds() as the real unsigned Metre/model-local consumer. It
   encloses exact projected points with checked construction and no stored geometry
   duplicate. Existing Source definition, Native eligibility, v1 binding identities,
   context and original budgets remain unchanged.
5. Add focused pure unsigned box/contact/order/tree tests with independent linear
   overlap and explicit bit-interleaving oracles, full extrema, boundaries and warm
   insert/replace/remove. Keep existing signed oracles. Add a compile-fail doctest
   for cross-Space overlap/union and run it through the existing value manifest.
   Add one actual consumer Scene read of a full-u64 triangle proving exact bounds
   from ordinary projection, without managed Source or spatial cost claims.
6. Run focused value owner/doctest, build the existing consumer root once and run
   new receiver plus affected ordinary family. Use original finite controls and
   existing build roles. The prior large Catalog receipt remains applicable only
   if review verifies identical signed algorithm/layout and cost receivers; otherwise
   rerun affected cost evidence. Preserve the two known cleanup exclusions.
7. Actual Cargo boundary/context, scoped format/caps/function/text checks, evidence
   and all three independent review gates precede the slice commit.

API: Box2::new(existing_signed_raw) retains its concrete meaning;
IntegerBox2::<ModelLocalMetre>::new(unsigned_raw) names a different exact domain.
SceneFaceRow::bounds() returns that typed unsigned enclosure. A box cannot supply
Native identity, contour closure, exact face contact or publication authority.
