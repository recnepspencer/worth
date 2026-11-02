# Phase5 topology integer orientation: implementation plan

> Scope update2026-11-01: this is a historical receipt/plan under the superseded
> public CAD scope. The Scene/Face application and its authoring/contact expansion
> are removed. Distinct generic numerical/provider fixes remain. This record does
> not reopen a public CAD prerequisite; see the [cleanup evidence](milestone-9.19-phase5-public-cad-cleanup-evidence.md).

The boundary review is complete for this numerical prerequisite. Broader face
universe/composition work is recorded separately and does not enter this slice.

1. Extract the current signed-product comparison from segment_box.rs into one
   Query-free integer_orientation.rs. Export a sealed i64/u64 coordinate contract
   and exact orientation-sign function. Keep fixed-width proofs adjacent to code.
   Cut segment-box contact to that one owner and delete its duplicate arithmetic.
2. Cut PositiveTurnRule evaluation to the exact u64 sign function. Retain its
   original Native field borrows, declared unit/frame, prepared scope, controls,
   work allowance and strict-positive verdict. No scalar cast or new denial lane.
3. Add pure tests for bounded signed/unsigned triples against an independent
   subtraction oracle, complete-range positive/negative/collinear cases and
   determinants of exactly +/-1 with huge product cancellation. Retain all existing
   segment-box clipping/full-width/contact courts.
4. Add an existing consumer-root topology court using actual CreateCycle commands:
   publish full-u64 positive rings, read exact stored values, and refuse opposite
   winding/collinearity as PositivePlanarTurn while preserving the selected head.
   Mount it in the existing proof family; no new court root/SDK family or hooks.
5. Run the focused value owner, new actual receiver and affected consumer family.
   The established unrelated cleanup exclusions stay explicit. The previous large
   geometry maintenance receipt remains applicable only if the signed arithmetic
   extraction is identical and the tree/scope/meter behavior is unchanged; otherwise
   rerun affected cost evidence. No face size claim is made by this slice.
6. Run actual Cargo boundary/context, scoped formatting/caps/function/text checks,
   update evidence, obtain all three independent QA gates and commit this slice.

Developer-facing result: integer_orientation([u64;2], [u64;2], [u64;2]) returns an
exact Ordering through the same primitive used by existing i64 segment contact.
The Native caller reads Metre/model-local values unchanged and compares that
Ordering with Greater. Scene/Face model, bounded scope cutover, unsigned box-tree
adapter, actual spatial discovery and large face cost court remain subsequent work.
