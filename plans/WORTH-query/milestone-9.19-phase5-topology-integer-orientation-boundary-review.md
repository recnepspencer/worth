# Phase5 topology integer orientation: boundary review

> Scope update2026-11-01: this is a historical receipt/plan under the superseded
> public CAD scope. The Scene/Face application and its authoring/contact expansion
> are removed. Distinct generic numerical/provider fixes remain. This record does
> not reopen a public CAD prerequisite; see the [cleanup evidence](milestone-9.19-phase5-public-cad-cleanup-evidence.md).

Selected slice: repair the exact numerical prerequisite at the existing
Query-free value owner and its real Native positive-turn consumer. Actual
Scene/Face discovery remains the next slice; no schema/composition or spatial
claim is made here.

The topology schema admits PositiveLength over the full nonzero u64 range,
Metre, worth.frames.model-local.v1. PositiveTurnRule evaluation reads those exact
admitted values but currently converts them to i128 and performs checked signed
products/subtraction. It can refuse valid positive determinants whose terms do
not fit i128, including a determinant of exactly +1 after cancellation.

consumer_values::segment_box already implements an exact sign predicate for the
full i64 coordinate range: widen differences to i128, form signed u128 product
magnitudes and compare them without constructing an overflowing signed difference.
The same proof holds for u64: each coordinate difference has magnitude at most
u64::MAX; each product fits u128. This is bounded fixed work and requires no
conversion, approximation, epsilon or additional dependency.

Move that sole arithmetic into integer_orientation.rs. Seal its supported
coordinate class to i64/u64 so arbitrary i128 values cannot violate its bounds.
Segment-box contact continues consuming it with i64; the Native positive-turn
consumer uses exact u64 values. Unit/frame interpretation remains with each
existing declaration and caller; the primitive requires one common unit/frame
and supplies no graph, topology, permission or publication authority.

The positive-turn law remains strictly positive orientation. Negative/zero turns
retain the existing typed invariant violation; large valid turns now produce the
correct verdict rather than an arithmetic-support refusal. Existing Native
component/orbit scope, budgets, source declarations, v1 query receivers and schema
composition remain unchanged in this prerequisite.

Tests must combine a bounded independent determinant oracle and literal full-width
cancellation/extreme cases with real ordinary topology CreateCycle publications
and exact invariant refusals. Do not infer face-discovery qualification, unsigned
box support or a changed scope algorithm from this work.
