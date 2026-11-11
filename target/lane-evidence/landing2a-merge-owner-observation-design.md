# Landing2a merge-history boundary and implementation plan

## Boundary brief

The shipping merge owner resolves owner-bound target/source bases and demands
canonical envelopes in planning::plan_history_scope. Its LoweredMergePlan still
holds the exact per-record originals, but artifact materialization discards the
two BranchCommitDelta values. The serialized artifact is not the truth source
for a newly inferred commit list. No public transaction merge-parent door will
be added (coordinator 2026-11-09 merge-parent ruling).

An immutable Native owner observation will retain those actual deltas alongside
the existing artifact, moving rather than cloning originals. It exposes only
per-record CommitId iterators and an immutable artifact. Existing planning entry
points keep the same artifact format and owner pipeline. A new observation entry
accepts only OwnerBoundMergePlanningRequest. No mutable roots, constructors,
extra merge execution route, serialization change or allocation is introduced.

The public court uses the actual external medium, real branch commits, original
checkpoint capture and a fresh cold receiver. It proves the source overlap row
includes the intermediate commit and actual canonical bytes are demanded.

The exact Conflict/MergeConflictOverlap result belongs to mvcc/validation. Its
unit constructs the private owner input the same way prepared merge execution
does. It uses the owner's memory journal, original Native checkpoint and actual
InlineCheckpointSource restoration, not the three adapters. Ordinary inline
restore eagerly reads envelopes: this half proves exact validation over restored
originals; the public half separately proves cold intermediate demand and denied
overlap. Together they replace the original claim; neither claims filesystem
durability. The reviewer caught and removed the false cold-unit precondition.

The cold-read failure matrix is C4 under the explicit ruling. Its exact source
is preserved by restore reference before removal. The original HAPPY court is
removed only after both replacement halves pass. No fault claim is retired.

## Plan

1. Add merge/planning_observation.rs, owner-only constructor, public read-only
   artifact and contribution accessors. Materialization moves existing deltas
   into it; old callers consume into_artifact. Export via merge and facade;
   document capability. Same request funding and work occur before observation.
2. Add external cold merge court: actual a/b history, source intermediate update,
   later source update, conflicting target update, checkpoint/reopen, owner-bound
   planning observation plus actual denied execution overlap and no frontier
   advancement. Assert canonical demand includes intermediate position.
3. Add mvcc/validation/merge_history_tests.rs: original owner checkpoint/recovery,
   exact restored intermediate, owner-issued target/parent input, exact Conflict and
   diagnostic, unchanged frontier. No public transaction creation change.
4. Move fault matrix to C4 with restore 9addca7179 and exact path; after two green
   halves remove obsolete cold fixture/source modules and baseline rows. Keep
   runtime attachment consumers mounted for their separate classification.
5. One focused build/run per affected binary (cold court and Native lib), then
   Root/court Clippy, fmt, dirty caps, boundary and context at phase completion.
   Reuse unchanged evidence. Fresh independent source/evidence review, commit.

Remaining attachment consumers, dead production warnings and Complete/ACK stay
next in the gate. No broad suites, ignored tests or new adapter.

The explicit owner-unit ruling requires relocating its existing runtime-root
allowance: transfer only the original matrix ExecutionRequest baseline row to
mvcc/validation/merge_history_tests.rs; remove all obsolete fixture rows. This
is the same authorized owner claim, not new baseline debt or hidden root.
