# Milestone 3.16.2 — Enforcement: regression proofs and closure

The enforcement section of [milestone 3.16.2](milestone-3.16.2.md) closes
requirements 1-4 with two last conditions:

- The two post-3.16.1 Scroll regressions keep their focused tests, and both
  fail when their conversion is reverted.
- Closure requires total conversion across the covered owners. A remaining raw
  lane is a material review finding.

This record covers both. The
[enforcement review](milestone-3.16.2-enforcement-review.md), the
[construction review](milestone-3.16.2-enforcement-construction.md), and the
[interleaving model](milestone-3.16.2-interleaving-model.md) cover the rest of
enforcement.

## The regressions fail when their conversion is reverted

Both regressions were found in native Pulse after 3.16.1 and fixed before
3.16.2 began. Requirements 1 and 2 then turned each fix into a type. Reverting
a conversion here means putting the old behavior back through the typed code,
since the types themselves no longer admit the old shape.

### A retarget in flight was recorded as unpresented

A notch that retargets a settle while a Motion tick is in flight installs a
successor. The successor departs from the frame that tick put on screen. It
was recorded as unpresented, so the Scroll settle could not see the frame the
host showed. The next sample then damaged the published geometry instead of
the place the content left.

Requirement 2 made a track's screen state a sum type:
`Unpresented`, `OnScreen`, or `Departed`. A successor that departs from the
sample on screen is `Departed` there (`depart_on_screen`).

The focused test is
`a_scroll_retarget_landing_with_a_tick_in_flight_is_on_screen_where_that_tick_left_it`.

| Reverted | Failure |
| --- | --- |
| `depart_on_screen` records nothing, so every successor stays `Unpresented` | "the Scroll settle reads the frame the host shows": no accepted sample, where the tick showed `[0, -15.56]` |
| Only the landing tick's departure (`departing_from_presented`) is skipped | the same assertion |

### A rebuilt group counted the gap twice

A Scroll group rebuilt before its accepted sample settled measured from the
published offset. Its commands already showed the displayed translation, so
the published-to-accepted gap was added a second time. Recent activity came
loose from its scrollbar and went blank.

Requirement 1 made group offsets carry their truth status. A group stands at
a `UiPublishedGroupOffset` until a witness displays it, and at a
`UiDisplayedGroupOffset` after. A command with a displayed base moves only by
a `UiDisplayedToAcceptedTranslation`, from where its bound standing was shown.
A rebind carries the group's standing forward (`group_standing`).

The focused test is
`a_group_rebuilt_before_its_accepted_sample_settles_moves_from_where_the_host_shows_it`.

| Reverted | Failure |
| --- | --- |
| A rebound group stands at its published offset again | the next settle's first frame translates by -80 where the host shows -40 |

The test used to check the bound standing before it presented the next
settle. The revert then failed on that structural check first. The test now
asserts what the host is given first, and checks the standing afterward, so
the revert fails on the doubled translation itself.

## Closure audit

### What the tools already hold

Most of closure is enforced mechanically, so a regression fails the build:

- `BC7004_RAW_GEOMETRY_REPRESENTATION` rejects a written raw coordinate array
  or scalar pair in every covered crate's production source outside a declared
  edge.
- The construction rules reject building a requirement 1 or 3 type outside
  its owner, and `Default` on requirement 2 state.
- The compile-fail probes prove one forbidden move per requirement.
- The owner-edit handle is the only mutation path for reconciled state. No
  hand-called tracking path remains: `note_owner_change` and
  `rebound_since_prepare` appear nowhere.

### What review had to read

The tools cannot see a lifecycle stage held as an `Option` and filled in
later, or a status chosen by a flag. The [requirement 2 review](milestone-3.16.2-r2-review.md)
converted or justified every such field that existed then. This audit read
each `bool` and `Option` field added to the covered crates since that review,
through requirements 1 and 4 and enforcement.

One raw lane remained. It is converted here:

- **A prepared Scroll group update held its displayed rect as an `Option`.**
  `UiScrollGroupMotionUpdate` started with `displayed: None`. Validation
  against the admitted witness filled it in, and commit wrote whatever it held
  into the group's displayed slot. A commit that skipped validation would have
  cleared the group's displayed sample with no error. Validation now returns a
  `UiDisplayedScrollGroupMotion`, which owns the displayed rect, and only that
  type commits. The compile probes `group-update-commit-displayed` and
  `group-update-commit-prepared` prove it: committing a prepared update is
  refused with E0599.

The rest are not lanes:

- **Optional geometry** (`UiAcceptedRect`, `UiPublishedRect`, and
  `UiTrackSamplePlace` on tracks and samples). These are requirement 1
  retypes of fields requirement 2 already reviewed. `None` means the Motion
  has no geometry, such as an opacity-only track. It does not qualify another
  field.
- **A Scroll command's displayed base**
  (`UiMountedScrollMotionCommand::base_translation`). This is a requirement 1
  retype of an `Option<[f32; 2]>`. `Some` carries a sealed
  `UiDisplayedCommandTranslation`, and the command moves from where the
  witness displayed it. `None` means only a publication has placed the
  command, and it moves through the typed published path. It qualifies no
  other field.
- **Owed work** (`UiNativeViewportExtent::owed`,
  `UiNativeSurfaceReconciliation::owed`). Each `Option` is the whole pending
  state and owns the data it owes. The observed extent beside the owed one is
  its own fact: a newer extent can be observed while an older one is still
  owed.
- **`UiPreparedMountedScrollPose::changes_coverage`**. This is a fact the pose
  computes about itself: whether it moves content across a clip's coverage
  line. It qualifies no other field.
- **The hit index record's `unscrolled` and `effective` rows**. `effective` is
  always derived from `unscrolled` and the accumulated translation, and is
  never set on its own.
- **The Motion install receipt's hit transition**. This is an optional
  payload, the same as a committed tick's. An install that moves no hit rows
  has none.
- **Scroll chrome's `presented` parameter**. This is a private parameter
  behind two named entry points, `scroll_chrome_facts` and
  `presented_scroll_chrome_facts`. Both read committed geometry, as the
  [requirement 1 review](milestone-3.16.2-r1-review.md) records. No flag
  stored on state picks between them.

Parameters, test models, and wire data are outside the rule, because none of
them is covered-owner state. For example, the Portal dismissal's
`sampled_bounds` was retyped to `Option<UiDisplayedRect>` and is only ever a
parameter.
