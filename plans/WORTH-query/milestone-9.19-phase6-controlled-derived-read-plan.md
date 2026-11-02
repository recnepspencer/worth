# Phase6 controlled pinned derived read plan

The [boundary review](milestone-9.19-phase6-controlled-derived-read-boundary-review.md)
precedes code. Deliver one real acquisition contract needed by root-path Source.

DX: `pin.read_derived_source(request, PinnedDerivedRelationScope::new(&handle,
root, shape).with_work_limit(limit).with_control(&checkpoint))`. The actual pin and
original request remain mandatory. Success returns the existing funded body;
failure returns `{ kind: DerivedRelationReadDenial, reads, work }`.

1. Add descriptive scope/failure types under derived_relation/pinned/scope.rs and
   expose them through the same Native runtime facade. Extract acquisition into
   pinned/acquisition.rs so physical read implementation and checkpoint/funding
   remain within the 400-line cap. All old doors delegate without extra meaning.
2. Carry optional borrowed control through SourceRead and every physical adjacency
   checkpoint. Retain read/work counters outside the native callback result,
   preserve exact nested/request/definition refusals and invoke final control
   after successful callback handoff. No body escapes a refusal and no new budget.
3. In the existing Native event certification family, use the registered runtime
   fixture and actual published pin/schema handle. Qualify forward/reverse/empty
   rosters and support, old-door compatibility; deterministically stop before work,
   within physical adjacency and at final handoff, preserve partial reads/work;
   work-limit/malformed/foreign-definition/stale/cancellation refusal and fresh twin.
   Use normal transactions and authored expected rosters, never evaluator truth.
4. Run cheap formatting/caps/topology first, then one existing Native court build
   and selected new plus old event/control cases. Reuse the binary; no broad sweep.
   Run actual boundary/context, function scrutiny, strict text/links/defaultdiff,
   independent qa-loop/qa-tests/code-quality-qa gates and commit the slice.

This does not claim Query root paths, maintained Source/currentness, private CAD
adoption/sharing/contact/scale/Acceptance8/full gate. Update evidence and current
milestone/capability handoff accurately; SDK pin remains unchanged.
