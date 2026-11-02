# 9.19: spatial ordering for alternating separated rows

## Boundary review

Audit item6 is demonstrated statically in the pure consumer-values box tree:
box_tree/edit.rs orders leaves by [lower_x,lower_y,upper_x,upper_y] and Source.
Its AVL height is bounded, but neighbors with alternating distant y coordinates
share broad vertical unions at almost every node. An empty query inside the x
range and between the rows can therefore visit the whole tree. Exact closed-box
unions, overlaps and Query's checked summary qualification are sound; grouping is
the weak owner. Source/GEN identities, paid storage, controls and publication stay
in the existing Query/Native adapters. No spatial-specific framework branch is
needed. Both cold build and maintained insert/remove/replace already share this
pure edit ordering, including last_bounds/last_source routing separators.

The numerical owner here is the existing query-agnostic certification consumer
value fixture, not production CAD adoption. Box2 has validated i64 coordinates in
its explicit fixture length unit and XY frame, exact integer union and inclusive
overlap, no float/tolerance/approximation. No CAD law document exists in this
platform checkout; the CAD skill's numerical/ownership review applies to this
exact existing fixture boundary without substituting another checkout's spec.
No new generic geometry capability or public authority is introduced.

## Plan (saved before Rust edits)

1. Replace the lexicographic leading coordinate with ordinary Morton (Z) ordering
   of box centroids. Compare the exact twice-centroid sums in i128, biased by2^64
   into65 unsigned bits. No averaging division, rounding, lossy cast, origin/grid
   perturbation or floating area heuristic. Compare the highest differing x/y
   bit (x first for equal significance), equivalent to interleaving all65 bits.
   Full raw coordinates then stable Source break equal-centroid ties. Ordering
   affects grouping only; exact bounds and overlap semantics do not change.
2. A private box_tree/order.rs owns this one pure comparison. Route every insert,
   removal, replacement separator and branch validation through it; remove the
   old key function. Keep deterministic AVL balancing and existing storage ports.
   No duplicate ordering registry or alternative cold tree path.
3. Fast consumer-values owner tests compare the comparator with an independent
   explicit130-bit interleaving oracle for negative/zero/positive coordinates,
   both i64 extremes, degenerate and wide boxes, equal-centroid/raw/source ties
   and deterministic mixed samples. Check total order, antisymmetry/transitivity
   via a single independently sorted expected sequence. No SDK/private root build.
4. Existing real keyed-box scan parity must stop requiring the retired lex leaf
   order; retain unique occurrence roster, rightmost separator metadata, AVL
   balance, exact whole union, repeated edit/remove and closed interval scans.
   Do not use the new comparator as its semantic result oracle.
5. Add a real installed public managed geometry court using existing Vertex/Edge
   authoring (no new batch/operation lane). Author32/64 disjoint x intervals on
   alternating y=0 and y=2^40 rows with fixed real endpoint incidence. Check the
   full ordinary Source census, cold explicit rebuild, and sparse0/1/boundary
   probes using independent authored coordinate comparisons and Parent's actual
   delivered order. Within-range gap probes must stay within4*ceil(log2(N))+3
   actual checked-summary callbacks; the old lex grouping fails this finite
   contract. No universal cheap-empty-query guarantee.
6. Move one actual endpoint across the gap, verify the warm update and exact
   resulting enclosure/query set, unchanged old retained rows and no hidden cold
   build. Include unrelated outside queries and negative-coordinate/extreme
   numerical owner cases. Report original query SDK invoices separately from
   actual callback counts; do not infer large-world invoice bounds from them.
7. Run fast pure owner tests first; then targeted real adversarial court, affected
   managed family (skip separately accepted large cases), Candidates9/9, boundary/
   context, fmt/whitespace, dirty caps/functions and independent all-three review.
   Default Rust1.99.1/sccache/shared target, no killed builds or extra test matrix.
   Commit this cleared slice separately.

This qualifies the demonstrated alternating-row defect plus existing collinear
and numerical distributions. Morton grouping can still be poor on overlapping
or adversarial boxes; no universal selectivity or CAD adoption is claimed. Churn,
retained predicate routing, exact refinement/disclosure and remaining phases stay
open. This is an internal fixture-provider algorithm change, no consumer API.

The real court lives under existing geometry/court/growth/spatial_rows.rs and
reuses that parent's original SDK invoice and authentication functions. No new
SerialRequest factory, SDK hook, private Source factory or batch action is needed.
The two isolated pure order tests pass2/2 in0.04s after a3.24s default build.

## Focused implementation evidence

Pure owner2/2 passes0.04s with74 full-width samples, all5476 pair comparisons and
independently sorted order. The installed adversarial court passes9.55s after a
34.18s default consumer build. Real Source33/65 empty within-range band gaps use
13/15 checked-summary comparisons, with whole original SDK query92324/130541.
After moving the real endpoint across the gap, exact enclosures produce one match
with13/15 comparisons and whole original query135592/200533. Inclusive boundary
queries, outside-root zero matches, full ordinary Source census/order, explicit
cold rebuild, actual Parent order, unchanged retained old rows and no warm cold
reconstruction all pass. Other bounded sparse probes use at most21 comparisons.

These are query/callback observations for this finite row distribution, not
large-world edit scaling, universal selectivity or production CAD adoption.
Affected managed50/50 passes18.76s (excluding the two separately accepted large
cases), including keyed edit/refit/retire scan parity and existing collinear,
summary, disclosure, refusal and lifecycle cases. Candidates9/9 passes0.56s.
Required cargo boundary/context, rustfmt/whitespace, dirty caps7 files and function
scrutiny7 advisories/0 errors pass. No full large geometry/prefix rerun was needed
for the explicit finite spatial grouping contract. Independent qa-loop, qa-tests and code-quality-qa
review clears this dedicated slice without a remaining scoped blocker.
