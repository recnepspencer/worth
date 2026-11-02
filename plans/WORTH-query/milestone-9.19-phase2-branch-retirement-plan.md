# 9.19 Phase 2: Query cleanup retires Native Source lookups

## Boundary brief

The previous prerequisite gives genuine retained roots/observations immutable
Native affinity and weak live Core links. Query still retains the live Source
cell in source_branches until registry disposal. Its lawful release trigger is
actual successful Native DeletedRelationalBranch custody, not a World branch name,
World occurrence retirement, WaitingForActiveOperations or unresolved durability.
Native retired-name discipline prevents replacing that deleted branch identity.
ReuseExact creates no Native owner custody and must not delete a shared component.

Product owner cleanup is the canonical Query integration: it releases World
history, then retires exact Native/Signal components. Native owner access ends
before the deletion result returns, but RetryRecordGuard still holds the cleanup
entry mutex. Upper close, inbound/transport recovery, generated materialization
and program-adoption paths can also hold command, application, custody or
publication guards and may discard successful cleanup receipts inside them.
Dropping an opaque Source payload at component completion or in an ordinary local
retry wrapper would violate those boundaries. Partial progress is also real:
Native deletion can succeed before later Signal/checkpoint refusal.

An explicit private runtime retirement accumulator must therefore be borrowed
through guarded callers. It owns prepaid entries containing the actual Native
Deleted receipt and a strong reference to the existing retention follower.
Preparation reserves container/receipt metadata under the attempt's execution
before deletion. Entry Drop performs an infallible notification after all guards;
the exact graph owner checks runtime affinity, removes the point lookup/empty map
backing under the registry mutex, then destroys them after releasing it. Until
that boundary the registry remains the one canonical owner. No ambient queue,
background worker, new provider callback or competing cache is introduced.

Bare public retry/recovery-release entrances own an accumulator outside their
private locks. Every internal guarded caller uses a borrowing variant. Native
and Query continue to own their existing lifecycle, history, READ, purpose and
currentness decisions. The actual Native deletion effect remains complete even
if the original request's final poll stops: completed cleanup returns its captured
receipt, while partial failure keeps only remaining work and still drains the
completed Native retirement outside guards.

## Implementation plan

1. Add owner_cleanup/retirement.rs for funded explicit retirement custody. Extend
   and rename the existing history follower to ProductRetentionFollower for its
   two retention notifications. Only the actual provider consumes Native Deleted;
   compare against its installed Native runtime before point removal. Keep each
   retained grant after the metadata it funds and all final follower destruction
   outside the guarded attempt.
2. Thread the accumulator through component retirement, record retry and registry
   retry. Prepare capacity before Native delete; move actual Deleted into that
   entry before recording progress. Waiting/refusal creates no retirement event.
   Add private borrowed retry variants; public retry owns the outer accumulator.
3. Thread explicit borrowed custody through product publication release and all
   guarded terminal callers: application close; inbound occurrence recovery;
   installed transport custody/recovery (including mutex-held prior release);
   suspension/restoration continuation and settlement; program-adoption recovery.
   Declare outer accumulators before their coordination/custody guards. Public
   unguarded release wrappers own local accumulators. Split touched files if the
   added protocol would exceed 400 lines; transport resume has a named module.
4. Capture completed admitted cleanup outside the real final poll. Preserve only
   actual complete success; retain partial work/refusal semantics. Add one narrow
   real final-poll owner control and partial-progress retirement control.
5. Add an actual Query registry retirement owner court using real Native fork and
   deletion custody; prove no removal on foreign identity, exact lookup removal,
   final live Core release and grant cleanup. Add a public geometry close/alias
   court: real Native fork, real ReuseExact alias, close the owning fork, fresh
   alias read/evict/refill remains correct, close final readers and aliases, exact
   all-owner allocation balance. Close nonessential managed registrations before
   attributing retention; make no promise that legitimate historical roots vanish.
6. Typecheck the consumer first, then run the new public journey, affected public
   managed-index controls and narrow owner groups. Use shared Cargo target and
   one Cargo process at a time; no feature/profile/env variants or broad suites.
   Run boundary/context, dirty format/caps/functions and independent QA. Update
   the qualification and guide precisely, preserve unrelated spec edits, commit.

## Explicit remaining work

Direct Native deletion outside Query's owner cleanup is not this integration.
Native retention/reclamation remains its existing authority. Physical suspension/
rematerialization and recovery composition, remaining hostile controls, standalone
late-updater cutover, growth qualification and later 9.19 phases remain open.

## Verified implementation refinements

Registry construction now receives the actual Native owner identity and rejects
foreign preparation, fork, preflight and deletion callbacks itself. Retirement
preparation uses immutable installed Source capability, not current lookup
presence: a real publication can install its first lookup between a presence
probe and deletion. Deferred point-removal is a lawful no-op if no slot exists.
Internal phased cleanup/recovery forwards the actual original execution request;
only bare public Host wrappers create their established own unbounded request.
The owning recovery result is captured across final polling so consumed World
record custody cannot be replaced by an empty retry state or panic.

## Boundary revision from the real public court

The new public close/alias journey is red at fresh Product selection with
BridgeSourceUnavailable. The actual chain is Query product admission -> Bridge
retain_branch_basis_for_bridge -> Native retain_observation_snapshot ->
retain_component_basis -> require_local_branch_identity, which demands a live
branch cell. Native snapshot cache custody already survives deletion; this is
an earlier additional-retention admission failure. An existing authored-history
assertion explicitly refused a new external hold, so the governing 9.19 contract
now separates extension of actual admitted immutable custody from fresh live
branch admission. This decision is saved before the Native repair.

Extend the original observation obligation's private owner-relationship projection
and require SameOwner against current Native history retention before acquiring
external retention from its actual binding/root. Preserve all fresh observation,
descriptor, currentness, write and fork doors. Update the old branch court's
negative to fresh observation/descriptor admission, while actual retained holds
succeed. Add one actual owner-reset court beside history_retention.rs, retaining
the old owner strongly so a missing pointer-owner check cannot pass accidentally.
No Bridge fallback, cache selector, serialized proof or policy toggle is added.
This adjacent Native repair is required for the unchanged real public alias court.

## Qualification and closeout

The unchanged real public close/alias court passed after that Native repair,
including fresh selected read, eviction, refill, sibling isolation and final
all-owner allocation balance. The full public managed-index group passes 26/26.
Native's actual owner-reset court and retained-history/deletion court pass; the
latter also directly refuses a retained-basis write with UnknownBranch, alongside
fresh observation, descriptor-only admission and fork refusals.

Two new owner cleanup controls pass: original cancellation after Native deletion
preserves Signal-only remaining work, and actual final polling cannot replace a
completed receipt. Both verify deferred notification outside actual cleanup-entry
and explicit outer guards and immediate original-request metadata grant release.
The real Query lookup court passes same-name foreign deletion refusal, exact
local lookup reclamation and final original grant balance. Its foreign source is
seeded through a genuine Native transaction; an initially empty fixture correctly
refused EmptySource and was repaired without weakening the ownership oracle.

The existing creation-recovery group passes 4/4. Forwarding the original admitted
request now classifies the actual injected unwind as Execution(Panicked); the
court requires that exact denial, Signal-only work and canonical retry to total2.
The existing inbound transport-recovery group passes 3/3 through Cargo's unchanged
repository stack configuration. A direct binary attempt omitted that configured
stack and overflowed; it supplies no behavior evidence. No environment, flags,
profile, feature, manifest or target variant was introduced for these checks.
Generated suspension/restoration recovery has no dedicated runtime court here;
only signature typechecking and the independent guard/forwarding review qualify
this mechanical carrier extension.

The implementation also repairs a causal test-only initialization call left by
the prior slice: schema-resolver lowering now supplies its real test host request.
The first owner test build exposed that missing argument before execution.
Independent QA reviewed receipt preparation, immutable capability, actual Native
runtime binding, insertion/retirement races, guard order, failure ownership and
the focused test repairs. Dirty formatting, 400-line caps and function scanning
are checked; existing long integration journeys and explicit boundary parameters
remain intentional evidence/topology, with no scan errors.
The required boundary-check and agent-context check both pass. All 41 touched
Rust files pass staged formatting and the 400-line cap; the staged diff is clean.

This closes Query cleanup-driven lookup retirement, not direct Native deletion
outside Query cleanup, all historical reclamation, generated materialization or
the remaining Phase 2 growth/affinity work. Later milestone phases remain open.
