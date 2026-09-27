# Milestone 3.16.2 — Requirement 4 owner-change review

Requirement 4 of [milestone 3.16.2](milestone-3.16.2.md) says owner changes
record themselves. Work that is in flight lands against state that may have
changed after it was prepared, and the landing reconciles each change. This
record lists each owner that review found holding such state, how each owner
now records its changes, and what its landing does with them.

In each owner, the state is private to a module of its own. That module's
methods are the edit handle, and every mutation writes its record in the same
call. The owner's shell or sampler can reach the state only through that
handle.

## Motion track table

`UiMotionTrackTable` holds the tracks a Motion sampler retains, together with
the record of edits made to them. `UiMotionOwnerEdit` lists every kind of edit
the owner makes between a tick's preparation and its landing:

- `Installed`: a track installed over the target.
- `Retired`: the target's track retired, by settlement, rebind, direct
  control, or shutdown.
- `ExtentRebased`: a Scroll track restarted from a newer presented extent.
- `EntranceAccepted`: the host found already showing a published entrance.
- `Rebound`: a publication rebound a surface's tracks to a new presentation.

A prepared tick samples a copy of the tracks, and its landing matches every
recorded edit exhaustively. A target the owner changed keeps its live state,
and its installed track departs from the frame the tick presented. A rebind
applies to every other target on the rebound surface.

- **Edits are sequenced by preparation.** Each edit carries the number of
  ticks prepared before it, and a tick lands only the edits made after it was
  prepared. A newer tick that is prepared and then dropped therefore cannot
  erase an edit that the tick in flight still needs.
- **The record stays bounded.** A later edit replaces an equal earlier one,
  and a later rebind of a surface replaces the earlier one. The record
  therefore holds at most one entry for each kind of edit to each target, and
  one rebind for each surface. A rebind that reaches no track leaves no
  record.
- **A stale tick lands nothing.** If a tick was prepared before the latest tick
  that landed, its landing changes nothing and it claims no terminal.

The landing now writes only the tracks. Before this change, a landing
replaced the whole sampler with the copy the tick had sampled. That undid
reduced-motion postures and denial counts that the owner set while the tick
was in flight.

Retired paths:

- `note_owner_change`
- `changed_since_prepare`
- `rebound_since_prepare`
- the sampler-wide `Clone` that a tick was prepared from

## Native pending viewport extent

`UiNativeViewportExtent` holds the latest host viewport observation and the
extent that is still owed a successor presentation.

- **Recording.** `observe` records each observation. A changed basis becomes
  owed when viewport measurement authority admits a successor.
- **Landing.** Settlement is prepared from the owed extent, and `land` ends
  only that extent. An extent observed after the settlement was prepared stays
  owed, so the older settlement cannot clear it.

This replaces direct assignment of the shell's `observed_viewport_basis` and
`pending_viewport_basis` fields, and the equality check hand-written at the
settle site.

## Native surface reconciliation

`UiNativeSurfaceReconciliation` holds the surface-binding replacement that
still needs a publication.

- **Recording.** `replace` records a rebind. A replacement that is still owed
  is reconciled against the binding that is still published, not against the
  unpublished candidate.
- **Landing.** A replacement ends only when a publication proves it, meaning
  the replacement is published and the binding it replaced is not.
  `land_outcome` matches every `UiMountedFrameOutcome` exhaustively, and only a
  publication can prove a replacement.

Before, any frame that published, was unchanged, or was reconciled cleared the
replacement, whichever binding the frame carried. That included a frame
prepared or retried from before a newer replacement, and a superseding frame
prepared without one. Each of these left a newer replacement unpublished and
no longer owed.

Retired paths:

- `surface_reconciliation_settled`
- `native_surface_reconciliation_is_current`
- the four direct clears of `pending_surface_reconciliation`

The Motion table's record is a list of edits because more than one edit can
happen while one tick is in flight. In each native owner, the record is the
owed value, so each landing either proves that value or leaves it owed.

## Mounted text pins

`UiMountedTextPinState` holds each binding's committed glyph pins and how
many bindings own each pin. A candidate is prepared against the committed
pins, and it carries the pin changes the host is told to apply.

- **Landing at the host's commit.** The native host commits a frame's text
  pins before it returns a physical surface still in flight. The candidate
  therefore lands when that in-flight outcome arrives, not when the surface
  completes. A superseding successor is admitted only while its predecessor
  waits on a physical surface. Before, it was prepared from pins that lacked
  the predecessor's, so the host was told to add those pins again, and to
  keep pins the successor had dropped. Only a text atlas transaction that is
  still pending holds its candidate back. The host admits one such
  transaction at a time and refuses any other text work until it settles.
  `UiHostPresentationProgressClass` now states both guarantees.
- **Landing over live pins.** A candidate lands over the binding's pins as
  they stand when it lands. The owner counts follow from that transition,
  not from changes computed when the candidate was prepared, so each binding
  counts its pins once.

## Owners that land by identity

The coordinator's other landings each settle against the identity of the
work they land: in-flight attempts, pending completions, supersession,
cancellation, and Motion samples. Each settles its own attempt, frame, or
binding, and none folds state prepared earlier back over the owner. A newer
attempt supersedes an older one through the coordinator's supersession
admission, not through a landing.
