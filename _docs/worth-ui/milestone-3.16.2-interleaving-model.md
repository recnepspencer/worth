# Milestone 3.16.2 — Enforcement: the interleaving model

The enforcement section of [milestone 3.16.2](milestone-3.16.2.md) asks for a
deterministic interleaving model. It mixes prepare, owner edit, rebind,
retarget, commit, pre-effect rejection, and resize. After every step it
asserts that displayed Motion, Scroll, hit-test, and cursor geometry equal the
latest witness's geometry. Debug builds assert the same invariant at every
commit.

This record covers the model, what it found, and how each finding was fixed.
The Scroll regression proofs are a separate phase.

## The model

`interleaving_model.rs` sits beside the other integrated appearance World
tests. It runs eight fixed seeds of 64 steps each. A 64-bit linear
congruential generator makes every choice, so a failure names the seed and
step that reproduce it.

Motion ticks and wheel notches go through the entries the native shell uses.
Publications go through the session's own publication path. The steps are:

| Step | What it does |
| --- | --- |
| Frame | Prepare and present a Motion tick at once. If no tick is prepared, pay the settles still owed. |
| Prepare | Prepare a tick and hold it. |
| Commit | Present the held tick. |
| RejectTick | The host refuses the held tick before any effect. |
| Notch | One wheel notch. A notch that lands while a settle is in flight is recorded as Retarget. |
| OwnerEdit | A track page places an offset directly, published at once. |
| Resize | Stage the region's content at its other length. A later publication lands it. |
| Publish | Publish the surface. |
| RejectPublication | The host refuses a publication before any effect. |
| BeginInFlight / CompleteInFlight | The host holds a publication open, then completes it. |
| Rebind | Rebuild the surface's layouts and rebind it to a new host generation. The pointer is reported again. |

The host is scripted only for calls the runtime owes it. A tick is owed a
host call when it moves something on the basis the host still displays. The
new `push_native_display_as_issued` acknowledgement reports paint exactly when
the issued work carries paint, and otherwise settles without effects. The model
never guesses what a frame paints.

A step that cannot run in the state it meets does nothing and says so. Every
kind of step must take effect in at least one run, so the model cannot pass by
refusing everything it tries.

### The witness

After every step, `witness.rs` checks:

- The scripted host has no pending call.
- Scroll and mounted geometry hold one pose, unless a resize is staged. No
  witness has shown a staged layout.
- Motion shows a sample the latest witness displays.
- Motion shows the settled pose unless a settle is owed. An owed settle must
  be the recorded lag of a deferral: `DeferredPendingGeometry` or
  `DeferredPresentationInFlight`.
- The host draws the nested text where Motion shows it, to within the
  device-grid snap. It draws it whenever it is in view.
- Both hit-test lanes, the interaction basis and the presented hit index, hold
  the same nested row. That row is exactly where Scroll settled it, or exactly
  where the host draws it.
- The cursor is over what a click at the resting point would reach.

The host's drawn text is the oracle for what is on screen.
`accepted_text.rs` replays the scripted host's retained draw list. A complete
projection replaces commands and samples. A delta retires the samples of every
command it touches. A re-issued sample replaces its predecessor. Each text
command is drawn at its committed bounds, moved by the transform of the sample
it is shown with.

The runs must also reach every state the witness tells apart: a sample on
screen, an owed settle, a staged resize, and the pointer over the nested
component. The model asserts that each one is reached.

## What the model found

Each finding was fixed at the boundary that owns it.

### Hit rows did not follow committed Scroll poses

When a settle landed, the presented hit index moved its rows by the committed
pose. The interaction basis kept reading the retained frame's rows, so the two
lanes disagreed until the next publication.

`UiPresentedHitTestBasis::follow_committed_scroll_poses` now moves each row by
the translation the hit index committed since that row's frame published it.
The move applies after any Motion samples. Focus placement reads the same
basis without samples, so it follows the settled pose too.

### A pose that changes clip coverage owed paint

A displayed sample only moves paint the host already holds. A settle that
moved content across the line between sharing clip coverage and sharing none
changed no paint. Content that came out from under disjoint clips stayed
undrawn, and content that went under them stayed drawn.

Clip derivation and Scroll poses now read one rule, `ancestor_clips_suppress`.
A prepared pose reports `changes_coverage`. A pose that changes coverage owes
its whole paint, the same way a direct edit does, so committed geometry never
mixes two poses.

### A retarget hid the sample on screen

A notch that retargets a settle in flight installs a successor track. The
successor started unpresented. The sample the host was showing then dropped
out of the Scroll settle and out of hit testing.

A successor that departs from the sample the host shows now starts on screen
there, the same way a landing tick's departure does (`depart_on_screen` in
the sampler's install).

That successor is on screen but has not been sampled yet, so it is in a
distinct `Departed` track state. A tick already in flight may still land a
later sample of the predecessor. When it does, the successor departs again
from that sample, exactly as an unpresented successor would. A departed
track is shown and counts as on screen, so Scroll settles and hit testing
keep reading its sample until a tick replaces it.

### A new settle departed from a stale offset

A settle deferred behind a publication in flight leaves the owner's accepted
offset short of the sample the host shows. A notch in that window started its
settle from the accepted offset. The content was pulled back to a pose the
reader had already seen it leave.

`settle_basis` now starts a new settle from the displayed offset while a
displayed sample stands. `displayed_scroll_offset` is the one reading of that
offset. The chrome readiness check uses it as well.

### A rebind dropped an owed settle

A rebind ends the surface's generation, which retires its Scroll content
samples. A settle still owed to the old generation was released unpaid.
Retained paint showed the content where the sample left it, but Scroll and
mounted geometry held the older pose.

The Scroll lifecycle rule applies here: content stops where its last accepted
sample left it. `rebind_host_surface_with_interaction_receipt` now pays the
surface's owed settle while its generation is still displayed, through
`settle_owed_scroll_sample_before_ending`, and only then deregisters the
binding.

A settle the ending generation cannot pay is still never paid on the new
generation. For example, a direct placement or a layout may be staged over
the geometry. That settle stays deferred through the rebind. The next frame
finds its generation gone and releases it as `Superseded`, and it moves
nothing.

### A staged layout is not a displayed pose

A resize stages a layout that no witness has shown. Scroll clamps its offset
to the staged travel at once, but mounted geometry keeps the displayed pose
until a publication lands the layout. The witness reads that gap as the
staging it is: Scroll and mounted geometry must agree again once the layout is
published. An owed settle defers with `DeferredPendingGeometry` while the
layout is staged.

### Each fix is load-bearing

Reverting any one fix makes the model fail. A failure the debug invariant
catches inside a step stops the run before the witness checks that step:

| Fix reverted | First failure |
| --- | --- |
| Hit rows follow committed poses | the debug invariant: the hit-test lanes part |
| Coverage change owes paint | seed 1, step 13: nested text in view is not drawn |
| Settle basis is the displayed offset | seed 5, step 55: the host draws at 32, Motion shows 46 |
| Rebind pays the owed settle | seed 6, step 44: the host draws at 45, Motion shows 54 |
| Retarget departs on screen | seed 5, step 55: the host draws at 32, Motion shows 46 |

## The debug invariant

`debug_assert_scroll_geometry_witnessed` runs wherever a witness commits:

- the committed arm of a Motion tick, after its settles;
- every publication, from `settle_presented_scroll_extent`;
- every Motion commit, once it installs its track.

Every publication path settles its Scroll extent through
`settle_presented_scroll_extent`: the session's own path, the framework-turn
execution paths, and mounted previews. The invariant runs there, so no path
can publish without it. Each path lends the invariant what it reads:
mounted geometry, Scroll, interaction, and the owed-settle record.

In debug builds it asserts:

- Each Scroll content sample Motion shows is displayed by its surface's
  latest witness.
- Unless a deferral owes the settle, the owner's offset stands where that
  witness displayed the sample. So does the mounted pose, unless geometry no
  witness has shown is staged over it. That geometry is a layout, or a direct
  placement awaiting its publication.
- Every Scroll region on a presented surface holds one pose in Scroll and
  mounted geometry, under the same staging exception.
- On every presented surface, both hit-test lanes hold the same rows at the
  same rects. One lane is the retained frame's rows, moved by the on-screen
  Motion samples and then by the committed poses. The other is every row the
  presented hit index holds, whether or not a modal shields it.
- Each hovering pointer's target is what its position resolves to on the
  current presentation.

A surface whose last settle was refused reports that in its disposition. Its
Scroll poses are not asserted until a later settle is paid there. Other
surfaces are still asserted.

The lanes are compared by where they place each row, not by what proved it. A
row a displayed sample moved and the same row committed at that rect are one
row.

A publication that displays a Portal entrance puts the entrance in the hit
index at once. The Motion commit that installs the entrance's sample follows
the publication, so interaction reads the entrance only from then on. Until
that commit installs, the surface's hit lanes are not compared. The install is
itself a commit, and the invariant runs again there.

### What the debug invariant found

Running the invariant across the runtime's world tests found three more
faults.

**The hit lanes drifted apart by rounding.** The hit index moved a row by
each settle in turn. The interaction basis moved the row once, by the sum of
the settles. After a few settles the two rects differed in the last bit of a
coordinate. Each index record now keeps its row as Motion projects it
(`unscrolled`) and derives the indexed row by moving that once by the
accumulated translation. Both lanes now do the same arithmetic.

Every committed pose adds to that translation. That includes a pose too small
to move the row's box and a pose that moves a row Motion hides. The next pose
or Motion projection then moves the row from the full sum.

**Opening a modal did not retest hover.** A pointer is retested only when the
hit rows near it change. A modal shields the rows beneath it without moving
them, so a pointer resting on the background kept hovering a target no click
could reach. The committed hit transition now compares what each binding's
modal admits: its input floor and the Visible Portals at or above it. A menu
above a modal that starts closing changes the admission but not the floor. A
binding whose admission changed counts as changed at every point, so every
pointer on it is retested.

**A Motion install did not move the hit index.** A successor track that
departs from the sample on screen is on screen from its install. When a
dismissal installs a Portal's exit, interaction reads the exit at once, and
the exit admits no hits. The hit index moved rows only at committed ticks, so
it kept the closing content where the entrance had left it until the next
tick. `install_motion_commit` now projects the installed target into the hit
index. It hands the resulting hit transition to interaction, the same way a
committed tick does.

Reverting any of these fixes fails the tests that reach it. Moving index
rows settle by settle fails the Scroll settle frame tests: the hit-test lanes
part. Dropping a pose that leaves the row's box in place fails
`scroll_poses_too_small_to_move_a_row_still_accumulate`. Dropping a pose that
moves a hidden row fails
`a_pose_that_moves_a_hidden_row_still_moves_it_once_revealed`. Dropping the
shielding comparison fails the modal pointer shielding tests: a pointer hovers
what the latest witness no longer shows under it. Comparing only the floor
fails
`a_portal_above_a_modal_that_starts_closing_changes_what_the_modal_admits`.
Skipping the projection at install fails the Portal dismissal tests: the
hit-test lanes part.

## Follow-ups

- **Content under disjoint clips appears at settle, not before.** A sample can
  move only paint the host holds. Content that a pose brings out from under
  disjoint clips therefore appears when the settle pays its paint. It does not
  appear while the sample is in flight. The fix is to issue that paint with
  the sample.
- **Hit rows and drawn text can differ by a subpixel.** A sample moves commands
  between device-grid points. A settle moves hit rows by the exact offset. At
  a fractional pose the hit rows can therefore sit up to half a device pixel
  from the drawn text until the next publication measures them from the drawn
  commands. The witness accepts either position exactly. Making settles land
  hit rows on the drawn grid would close the gap.
