No remaining in-scope findings.

1. **Prior P1 resolved — schema refusal bindings.** [Lines 58–72](</C:/programming/worth worktees/worktrees/m3-p3-reopen/crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/checkpoint_schema_plan_refusals.rs:58>) use existing definition tuples and the correctly typed `materialize`. Exact expected/found revisions, typed mismatch families, and contract IDs remain asserted.

2. **Prior P1 resolved — replacement reads.** [Lines 57–89](</C:/programming/worth worktees/worktrees/m3-p3-reopen/crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/checkpoint_record_replacement.rs:57>) use public `try_read_version`, explicitly requiring both request success and historical admission. Current replacement and original historical payload assertions remain; both runtimes retain `AiWorkflow`.

3. **Prior P1 resolved — transition plan configuration.** [Lines 61–87](</C:/programming/worth worktees/worktrees/m3-p3-reopen/crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/checkpoint_transition_descriptors.rs:61>) create the plan through the declared v2 receiver, matching the configuration copied into the plan. Descriptor, continuity, diagnostics, explicit verification mode, and original expected-v2 mismatch assertions remain unchanged.

4. **Prior P2 resolved — latest-commit observation.** [Lines 25–29](</C:/programming/worth worktees/worktrees/m3-p3-reopen/crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/checkpoint_inspection_observation.rs:25>) call public `historical_latest_commit`, the same operation delegated to by the original `latest_commit`. Its actual ID feeds `inspect_commit`; all seven B2 inspection fields remain independently represented.

Compared with `df2420390e`, all assertions assigned to B2 remain represented; physical allocation, retention census, and work-counter assertions retain their governing C2 disposition. Prior acceptance of the other schema/history tests stands. CountedSource and deferred naming/helper placement remain excluded.

Source-only review; no builds, edits, or subagents.

**APPROVE**