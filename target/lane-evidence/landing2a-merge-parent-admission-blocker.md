# HAPPY boundary decision: merge-parent transaction admission

Status: BLOCKED on a product creation-contract ruling. Source HEAD `2db457e903b384aa13e8b488dab72b96d8996b7b`.
No owner file permission is missing. No test was weakened, deleted or moved C2
for this functional gap. Both originals remain mounted:

- `cold_merge_history_matrix.rs::a_cold_intermediate_source_commit_cannot_disappear_from_merge_parent_conflicts`.
- `cold_merge_history_matrix/exhaustive.rs::residency_failure_matrix_nonhead_merge_history_covers_every_door_and_class`.

## Exact evidence and all holders

Search: `rg -n merge_parent_bases crates/worth-relational/src --glob '*.rs'`.
`tests/support/transaction_authority.rs:48` observes target/source bases, begins
an ordinary transaction, then directly assigns `transaction.merge_parent_bases`.
`mvcc/transaction/admission.rs:187` always initializes it empty. The only shipping
assignment from another value is in owner-private validation conversion;
`mvcc/validation.rs:163` with_merge_parent_bases and the validation input itself
are crate-private. Shipping independent merge and replay feed that private
validation input, rather than publicly admitting such a transaction.

Search of public capability documentation:
`rg -n 'merge.parent|merge-parent|transaction.*merge|merge.*transaction' docs/capability-index.md crates/worth-relational/README.md`.
The only README hits are ordinary merged-plan preview. Source lookup, not docs
alone, establishes the missing transaction admission. Existing public merge
planning/execution use owner-bound requests (`merge/access.rs`, `merge/facade.rs`,
`runtime/state/runtime_state/merge_authority.rs`). They report NotExecutionReady
for the overlap and cannot produce the exact ordinary-transaction
TransactionCommitError::Conflict with DiagnosticCode::MergeConflictOverlap that
the retained court asserts.

All current helper references from
`rg -l test_owner_begin_merge_transaction crates/worth-relational/src -g '*.rs'`:
- `crates/worth-relational/src/tests/transactions/core/authored_write_history/branches.rs`.
- `crates/worth-relational/src/tests/support/transaction_authority.rs`.
- `crates/worth-relational/src/tests/support/records.rs`.
- `crates/worth-relational/src/mvcc/publication/application_publication/continuity_tests/retirement/authored/refusal.rs`.
- `crates/worth-relational/src/tests/harness/scenario/runner/operation_application.rs`.
- `crates/worth-relational/src/tests/history/queries/merge_commit_authority.rs`.
- `crates/worth-relational/src/tests/durability/medium/cold_merge_history_matrix.rs`.
- `crates/worth-relational/src/tests/domains/fintech/actions/merges.rs`.

## Required ruling

Classification 3 authorizes a minimal typed public observation for a HAPPY claim
without a surface. Creating a transaction with additional merge parents is a
creation/admission capability. It needs a decision about the supported Native
contract, rather than a getter, testkit setter, copied predicate or wrapper.
An async request for that decision is pending; elapsed time is not approval.

Recommended direction if supported: genuine owner-validated transaction admission
using existing exact admitted target/source bases, caller-funded request and
existing concrete phase/authority types, with typed foreign/stale/lifecycle/
request denials. Do not expose the private field or mutable root. The owner must
choose the concrete admission contract before implementation; no speculative
public constructor has been added.

The other functional HistoryPlan assertion needs an immutable owner observation
of actual per-record contributing commit IDs (target overlap and source overlap
including the original intermediate commit). Aggregate ancestry counts and an
independent NotExecutionReady report cannot replace it. This is HAPPY, not C2.
Once admission is decided, implement both together and qualify the unchanged
three-door/six-cause matrix and the actual intermediate-overlap control.

## State at stop

All seventeen independent remaining cold ports qualified and committed with
fresh independent APPROVE. The two history courts, three local adapters, broader
attachment consumer migration, 32-warning deletion/wiring gate and final
Complete/ACK gate qualification remain unfinished. Their order is unchanged.
Read-only Complete/ACK review found no concrete implementation defect, but it
is not a passing named qualification lane. No broader gate is declared closed.
