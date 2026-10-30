# Phase 2: cancellation after actual World publication

## Boundary review

The next unfinished Phase 2 edge is cancellation after successful World movement.
The existing generated endpoint Source journey already proves cancellation before
owner effects and Native-ahead/World-predecessor custody. Neither proves the
performed-result handoff or cleanup after an authoritative World successor exists.

World's existing comparison pause precedes CAS; its read-currentness control is
not a post-publication seam. `StagedPublicationRecord::mark_committed` samples
cancellation inside reader-excluded cutover. No callback or blocking control may
be inserted there. After `ready.publish_controlled` returns Performed in World's
`finish_publication`, the immutable movement and canonical facts already exist,
and cell locks are released. This is the lawful observation boundary.

World continues to own CAS, canonical facts and actual performed delivery. A
closed one-shot observer may borrow that delivery and cancel the caller's actual
SDK token. It cannot choose a head, replace a result or consume the delivery.
Its scoped registration belongs to this World control receiver, executes outside
the slot lock, and drops only its own pending registration. Cancellation at this
point must not retroactively rewrite the already sampled late-cancellation fact.

Query's direct suspension captures its complete result before the outer final
request poll. Recovery adoption similarly captures its result. After an actual
restoration adoption succeeds, `finish_restoration_published` releases the old
recovery record using the original execution request. A cancellation there must
return RecoveryHandoff with PublishedWorld settlement custody, not pretend the
new World/Source did not publish. Fresh continuation finishes that exact settlement
and cleanup, without another Native/provider/rule execution or product CAS.

## Implementation plan

1. Add `operation_control/performed_publication.rs` to World's existing receiver.
   Expose a sealed `observe_next_performed_publication_for_test(observer)` door
   returning a scoped registration. Use one owner-local slot, an unarmed atomic
   fast path, exact registration identity on Drop, and a borrowed actual
   PerformedCompositePublication. Run the taken observer after releasing the
   slot lock. Invoke it only for actual Performed after `publish_controlled`.
   Export the registration type through existing lifecycle/facade paths.
   Add owner-local `performed_publication/tests.rs` controls for unlocked opaque
   capture destruction on disarm/take and an old guard preserving a replacement.
   These controls fabricate no performed receipt; the public journey supplies
   the actual publication evidence.
2. Add a focused generated geometry court, `court/late_cancellation.rs`, with a
   child for restoration recovery scheduling if needed. Reuse the real program,
   producer, endpoint Source, A/B scopes, provider, rules and reconstruction.
   Open both handles and retain original endpoint-10 Source/Parent observations.
3. Arm the observer to validate a real CAS win and cancel the original suspension
   SDK scope. Require the actual Suspended output, moved selected head, suspended
   read refusal, no unpublished catalog entry and zero active attempts. Preserve
   its custody for lawful endpoint-20 reconstruction.
4. Produce real unpublished restoration using the existing pre-CAS World pause
   and cancellation, keeping Native custody on the issuing thread. Validate its
   exact CancellationAfterEffect, one Native effect and actual catalog identity.
   The selected predecessor remains suspended. Record the provider/rule work.
5. Arm the performed observer for a fresh adoption scope. Actual World adoption
   must move the head before cancellation. Require exact RecoveryHandoff with
   returned custody, while fresh current Source/Parent answers already contain
   endpoint 20 and retained ordinary Source answers contain original 10. The
   original recovery record remains, with zero active publication attempts.
6. Fresh continuation completes that already-published settlement. Require the
   same selected World commit, no provider build/update or rule replay, empty
   recovery catalog and matching current/historical answers. Warm the whole
   journey once, then require exact all-owner physical balance on final release.
7. Add exact closed caller seals and qualify only this evidence in the milestone,
   preserving the unrelated draft. Repair production handoff only if the court
   exposes a concrete defect, revising this plan before any expanded change.

## Verification and limits

Run the narrow new consumer-root test first, then the full managed-index group
in the same compiled binary. Run the narrow World observer slot-lifecycle unit
test. Run dirty formatting, line caps, function scrutiny,
boundary-check and agent-context check. Independent required checks may execute
concurrently using the default shared Cargo cache/profile. The independent
reviewer gates intent, evidence and code quality before the slice commit.

This slice does not qualify cancellation at the atomic sampling instant, alter
immutable late-cancellation facts, authorize adoption across a competing Signal
winner, or close Phase 2 growth/hostile controls and Phases 3–6. It adds no second
scheduler, provider authority or publication implementation.
