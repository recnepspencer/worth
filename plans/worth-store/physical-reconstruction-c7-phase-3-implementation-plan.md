# WORTH Store C.7 Phase 3 Implementation Plan

## Slice

Phase 3 binds one prepared physical mutation to one immutable WAL member and
append range, then performs that append only through Store's existing Signal,
scheduler, executor, and C.4 media topology.

Phase 3 ends at `WalAppendedPhysicalMutation`. It creates no WAL-durable proof,
data-dispatch authority, root-publication fact, or caller acknowledgment.

## Boundary Review

The authoritative input is `PreparedPhysicalMutation`: one Store-issued
mutation identity, idempotency lease, canonical request fingerprint, admitted
placement, deadline, policy basis, and nonempty prepared record batch.

`worth-store-wal` owns frame grammar, LSN topology, path-free append planning,
bounded inspection, and stable WAL identities. It does not choose Store work,
open an ordinary filesystem writer, settle Store truth, or acknowledge a
mutation.

Store owns:

- member and LSN allocation;
- the persisted attempt binding;
- canonical redo ordering;
- exact work declaration and Signal dependency;
- scheduler admission and executor dispatch;
- the transition from reserved to appended after matching C.4 evidence; and
- sealing the WAL owner when an effect may have occurred.

C.4 remains the only ordinary media-effect owner. Scheduler completion cannot
substitute for C.4 completion, and C.4 completion cannot substitute for a
future WAL barrier.

Recovery Physics retains crash posture, bounded recovery observation, and redo
meaning. Its direct `execute_wal_durability*` writer is displaced. The older
backend durability runtime still supplies non-WAL barrier-mechanism evidence
consumed by Phase 4 and older certification fixtures, but its WAL-specific
append input and execution methods are displaced in this phase.

The path-bound `WalAppendPlanner` performs reconstructive prefix inspection.
It must not remain the ordinary crate-root append-planning API. Ordinary Store
construction uses only `WalAppendFrontier` plus `plan_wal_frame_append`.

## Destination Shape

```text
worth-store/
  physical_runtime/
    durability/
      mutation/progression/
        wal_reserved.rs
        wal_appended.rs
      wal/
        canonical_redo.rs
        member_basis.rs
        append_declaration.rs
        append_settlement.rs
        observation.rs
        runtime_owner.rs
        port.rs
    instance/executor/
      wal_append.rs
    record_serving/work_semantics/durability/
      policy_binding_basis.rs
      wal_append_basis.rs

worth-store-wal/
  append/
    frame_plan.rs
  artifact_store/
    bounded inspection only
```

A directory may contain one file when the responsibility is expected to grow;
the directory expresses the durable semantic boundary, not a file-count
optimization.

## Ordered Implementation

1. Complete the path-free WAL planner and Store-owned reserved/appended phase
   types. Prove exact mutation, lease, fingerprint, member, redo, LSN, artifact
   range, and payload bindings.
2. Add the WAL work family, Signal dependency basis, scheduler lane, executor
   command, C.4 append dispatch, and exact settlement comparison.
3. Integrate the WAL owner and port at Store construction. Expose the ordinary
   prepared-mutation append and read-only observation while keeping later
   durability, data, root, completion, and acknowledgment transitions
   unavailable.
4. Delete Recovery Physics ordinary WAL execution and executor-only tests and
   exports.
5. Remove the backend durability runtime's WAL-specific append input and
   methods. Preserve generic barrier proof types required by Phase 4.
6. Remove the path-bound `WalAppendPlanner` from the ordinary
   `worth-store-wal` root facade. Keep bounded reconstructive inspection under
   its explicit inspection owner where still required.
7. Update owner documentation and public API guidance. Let Git history retain
   the removed-source record; do not maintain a parallel inventory or ledger.

## Evidence

- one real two-record WAL append through Store and independent bounded scan;
- persisted idempotency lease, request fingerprint, member range, and redo
  inspection;
- consecutive mutations receive distinct contiguous bindings;
- a strict partial frame is indeterminate and seals later allocation;
- retry and completed-receipt substitution cannot cross exact
  work/range/digest checks;
- one production owner each for the C.4 append call, Store settlement, and
  appended-type construction;
- direct Recovery Physics execution and WAL-specific backend runtime methods
  are absent from ordinary facades; any retained lower-mechanism probe is
  explicitly certification-only and cannot construct Store progression; and
- formatting, focused crate tests, boundary enforcement, agent context, and
  line-cap checks pass.

## Out of Scope

Phase 4 owns WAL barriers, WAL-before-data, pageLSN binding, data dispatch, and
data settlement. Later phases own grouping, checkpoints, root publication,
final ordinary mutation facade cutover, caller acknowledgment, recovery, and
reclamation.
