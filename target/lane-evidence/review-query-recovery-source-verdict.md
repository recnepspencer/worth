1. **P2 — Preserve the original relation batch label.** [checkpoint_query_parity.rs:17](<C:/programming/worth worktees/worktrees/m3-p3-reopen/target/lane-evidence/pa-query-recovery-sources/checkpoint_query_parity.rs:17>) calls a helper that constructs `WorkerIntentBatch::new(key)`, producing `"edge"`. The original uses `"relation"`. This violates the plan’s exact batch-input preservation claim even if parity passes. Construct the original batch through the existing public transaction API without changing the shared helper.

2. **P3 — Remove unused query imports.** [checkpoint_generation_identity.rs:4](<C:/programming/worth worktees/worktrees/m3-p3-reopen/target/lane-evidence/pa-query-recovery-sources/checkpoint_generation_identity.rs:4>) imports `IndexParityMode` and `QueryAccessContract` without using them. Remove both before qualification.

The remaining functional assertions, packet fields, request/profile/schema/policy inputs, source and snapshot provenance, immutable C2 closure, and planned deletion/cap scope are consistent. No edits, builds, tests, or delegation performed.

**REJECT — source/plan only.**