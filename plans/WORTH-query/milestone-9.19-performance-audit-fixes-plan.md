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
4. **Partially implemented:** [Borrow-confined decoded text](milestone-9.19-prefix-text-reuse-plan.md) removes repeated String decoding/copying and owned selector-key construction from current prefix paths. [Invocation-local ancestry reuse](milestone-9.19-summary-ancestry-reuse-plan.md) also replaces repeated parent walks with lazy paid jump metadata and preserves per-region lookup admission. Prefix scale evidence remains open. Prefix proof/key reuse: memoize within actual immutable invocation/root/version/
   receiving audience; preserve region restriction and per-use admission. Qualify
   prefix scale separately. Current fixture Source bound16 and observer bound8
   must be repaired before large complete-census tests; sparse Parent cap16 stays.
5. Directory updates: batch touched pages so each shared directory ancestor copies
   once, with original funding/controls and old-root/sibling isolation.
6. Spatial grouping: improve demonstrated poor grouping using ordinary spatial
   heuristics and qualify representative/adversarial distributions. No universal
   cheap-empty-query promise: not every spatial query/distribution can be selective.
7. Churn: separate stable identity from physical placement, reuse/compact safely or
   use an equivalent bounded-live structural representation. Small live indexes
   must not silently inherit ever-issued-coordinate traversal cost. Preserve old
   snapshots, sibling edits, failed edits and non-ABA identity guarantees.

Each repair needs an owner/boundary review, saved concrete plan, focused honest
runtime evidence, independent qa-loop/qa-tests/code-quality-qa clearance and a
commit. No item is closed merely by recording this list or by prior collinear
geometry success. Predicate routing, refinement/disclosure and later phases also
remain unfinished.
