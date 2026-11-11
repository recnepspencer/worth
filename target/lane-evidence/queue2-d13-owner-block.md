# D13 interim restore: required Lifecycle owner

Status: BLOCKED on this item under the current conditional ownership grant.
No D13 source was changed; restore tests are unrun for this item.

Required file:
`workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/construction.rs`.

`git diff --quiet 7d0e5189bc codex/m3-p3-lifecycle -- <that file>` exits 1.
`git diff --numstat` reports 1 insertion. Exact owner comparison/ref identity
is recorded in queue-owner-eligibility.json.

The actual restore chain is program/checkpoint.rs (including roster restore)
to program/construction.rs::in_memory_program_with_optional_authorization_time_source,
then in_memory_program_in_advancement, then this shared owner's
in_memory_with_contributions_in_advancement. The latter owns checkpoint decode,
native preparation, program transition, publication and recovered-output
attachment (checkpoint argument at line30 and decode at line82).

The requested single WorthQueryInterimCheckpointRestore door and elimination
of in_memory/InMemory names along the interim path require separating or
renaming that real shared construction path. A public wrapper that still
calls it would leave the existing competing path/names. Duplicating its owner
or retaining aliases would violate clean cutover and composition, rather than
complete the task. Therefore no partial door, alias or caller list is invented.

The latest user instruction says that if a needed file belongs to Lifecycle,
stop on that item, report the file and proceed to the next queue item. This
handoff follows that instruction; it does not request another observer door.
