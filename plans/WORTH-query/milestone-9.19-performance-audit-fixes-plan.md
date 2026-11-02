# Required performance audit repairs

The user explicitly requires fixes for all six findings in the static audit of
58032cbcb4, plus the churn axis. The audit ran no tests; its algorithm observations
are inputs to verification, not measured results. This work supplements the full
9.19 milestone; it does not replace or narrow that objective.

Use ordinary data structures in their existing owners. Preserve actual original
payer/control authority, immutable snapshots, branch isolation, every incoming
occurrence/admission and typed refusal. Keep predeclared qualification guards.

Required repairs and current evidence:

1. **Verified and independently cleared:** [Support growth](milestone-9.19-support-growth-plan.md): geometric capacity growth throughout actual affected-expression,
   merge and scope-closure producers; original persistent capacity-delta and
   temporary old/new-overlap admission. Small real composed-fanout evidence.
2. **Verified and independently cleared:** [Candidate collection](milestone-9.19-sparse-candidates-plan.md): output-sized accumulation for pruned non-maintained
   Sources too; sort/deduplicate only emitted candidates, preserve canonical order,
   bounds/duplicates and genuine slot ownership. Empty pruning does no N-bit sweep.
3. **Verified and independently cleared:** [Cold Source graph validation](milestone-9.19-cold-validation-plan.md): single DFS/BFS with visitation state, preserving
   Source bijection, connectivity/acyclicity and hostile/disconnected/skewed cases.
   Incremental checks continue for actual new edges.
4. **Verified and independently cleared:** [Borrow-confined decoded text](milestone-9.19-prefix-text-reuse-plan.md)
   removes repeated String decoding/copying and owned selector-key construction.
   The initial jump interpreter failed the real small-width guard and is removed.
   [Bounded per-region membership reuse](milestone-9.19-summary-membership-cost-repair-plan.md)
   preserves per-use admission with four positive facts and paid fallback. Owner3/3,
   actual small32/64 and [1k/2k/+4096 width qualification](milestone-9.19-prefix-width-qualification-plan.md)
   pass unchanged25%/50% whole original SDK guards; no universal depth bound is
   claimed. Source bound8192 and full cold validation remove fixture ceilings;
   sparse Parent cap16 remains. Affected managed49/49 and Candidates9/9 pass.
5. **Verified and independently cleared:** [Directory batching](milestone-9.19-directory-batch-plan.md)
   seals the ordered union of touched paths, copies every affected ancestor once,
   preserves untouched cells and prunes empty far growth without discarded
   wrappers. Owner6/6 proves actual dense65/128 and sparse11 branch allocations,
   typed original partial-seal refusal, grant release and old/sibling isolation.
   Affected managed49/49 and Candidates9/9 pass. Historical coordinate depth and
   spatial grouping remain separate obligations.
6. **Verified and independently cleared:** [Exact centroid Morton grouping](milestone-9.19-spatial-order-plan.md)
   replaces the fixture's lower-x-first lex order while keeping exact closed
   unions, stable occurrence ties and the same cold/warm AVL edits. Pure owner2/2
   agrees with an independent130-bit oracle at all coordinate extremes. Actual
   alternating separated rows Source33/65 use13/15 comparisons for empty band
   gaps; cold/warm membership, boundary, Parent order and retained old rows pass.
   Existing collinear/keyed edit cases remain green within managed50/50;
   Candidates9/9 passes. Morton grouping is an ordinary heuristic with no universal
   cheap-empty-query, overlap selectivity or production CAD adoption claim.
7. **Verified and independently cleared:** [Compressed live directory](milestone-9.19-live-directory-plan.md)
   keeps NodeLineage monotonic, uses aligned-prefix checks and unary collapse,
   and retains at most P leaves/P-1 branches for P live pages. Owner8/8 proves
   label-gap-invariant work/memory, genuine4096-ID churn, old/sibling identities
   and original refusal cleanup. The measured large geometry regression exposed
   adjacent bounds-rekey churn; extraction/reinsertion now reuses its actual
   still-live leaf/parent instead of retiring/appending them. Keyed edit2/2 and
   managed51/51 pass. Fresh prefix1k/2k/+4096 and geometry1k/2k/+50k all pass their
   unchanged whole-SDK guards. Candidates9/9 and repository gates pass. Retained
   snapshots keep their lawful shares; no universal log(P)/all-history bound.

Each repair needs an owner/boundary review, saved concrete plan, focused honest
runtime evidence, independent qa-loop/qa-tests/code-quality-qa clearance and a
commit. No item is closed merely by recording this list or by prior collinear
geometry success. Predicate routing, refinement/disclosure and later phases also
remain unfinished.
