# Independent merge-history design review

Reviewer: /root/merge_observation_design_review. HEAD 9addca7179 and the named
candidate merge-observation source files. No builds, tests or edits by reviewer.

Initial verdict REJECT: inline restore eagerly reads originals; the proposed
cold owner-unit precondition was false. Corrected before verification: public
shipping merge proves cold demand; owner unit proves exact conflict over restored
originals. The literal coordinator ruling composes these two halves.

Final verdict APPROVE revised design/source (qualification pending):
1. Previous HIGH resolved by honest claim placement; restore behavior unchanged.
2. Public half retains actual intermediate/source and target contributions,
   intermediate canonical demand, overlap execution denial and no effects.
3. Owner half restores actual Native artifact, obtains owner-issued target and
   parent inputs, exact Conflict/MergeConflictOverlap, diagnostic and frontier.
4. Observation moves actual lowered deltas; existing serialized Core and execution
   materialization caller unchanged. No transaction creation or authority door.
5. Exact original ExecutionRequest baseline placement transfer is warranted by
   explicit owner-unit ruling; remove original rows after replacements pass.

This is design/source approval, not passing-test or final completion evidence.
