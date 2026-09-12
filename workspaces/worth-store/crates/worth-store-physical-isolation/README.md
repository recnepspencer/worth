# worth-store-physical-isolation

Owns Roadmap 2 S.5: physical byte stability while compaction, checkpointing,
reclaim, tier movement, and blob migration interleave with foreground reads.

This crate owns lower physical read plans, generation references, interlocks,
and local plan completion. It does not implement semantic MVCC or own live
Store byte access.

`StablePhysicalReadHandle::complete_plan()` returns
`PhysicalReadPlanCompletionReceipt`: the plan's counters and release facts,
not evidence that bytes were read. B-tree lookup and lower checkpoint and
compaction interlocks consume this local evidence.

The live chunk guard, security-scope adapter, and byte execution facade now
belong to `worth_store::physical_runtime::stability`. Byte callers import
`PhysicalByteGuard`, `StablePhysicalReadExecution`, and
`StablePhysicalReadReceipt` there. Isolation must not depend on Store; the
repository boundary checker enforces that direction. There is no conversion
from local completion to a byte-read receipt.

Root correlation is derived only from an already-issued `CurrentPhysicalRoot`.
The former recovery-readiness entry family has been removed: recovered-root
strings, source digests, replay counts, and certification recipes cannot mint
root epochs or live reader authority. Recovery keeps its real sealed handoff;
ordinary readers enter through `ServingPhysicalRuntime::records()`.

`CompactionReadPlanCompletion::from_publication` correlates a sealed local
publication with completed pre/post read plans, including both exact roots and
footprint bases. `plan_post_cutover_read()` derives the post plan from that
publication. Neither call reads Store bytes or retains live roots. Blob rewrite
binding additionally checks the exact physical plan and expected manifest epoch.

The former recovery-gated cutover proof is removed. Recovery's selected
checkpoint compaction product describes an operation-binding index, not physical
rewrite visibility. Isolation has no Store or Recovery dependency; all three
upward edges are denied by the repository boundary checker.

Publication planning ends at `CopyOnWritePublicationPlan::complete_plan()` and
returns `PhysicalPublicationPlanCompletion`. It carries validated roots, epochs,
ordering and local release correlation, not a root swap or durable execution.
The old execution receipt, inert atomic-swap wrapper, executed Foundational
projection and synthetic crash/recovery wrappers are removed. Store's real
publication owner alone returns `CompletedPhysicalRootPublication` after its
namespace-durable transition; production recovery remains in Recovery.
