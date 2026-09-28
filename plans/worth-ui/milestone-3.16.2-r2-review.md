# Milestone 3.16.2 — Requirement 2 field review

Requirement 2 of [milestone 3.16.2](milestone-3.16.2.md) says review must convert
or explicitly justify every `bool` and `Option` field in the covered owners that
qualifies another field. The covered owners are the runtime mounting and
interaction owners, the host contract, the native and headless hosts,
certification support, and Platform Pulse. This record lists each field that
review converted and each field it kept, with the reason it was kept.

A field is kept only when it falls into one of these categories:

- **Owned datum.** The `Option` *is* the data: a take-once resource, a mailbox,
  a handoff slot, or a worker joined at shutdown. `None` means "not held", and no
  other field depends on it.
- **Latch.** A one-way `bool` that records that something happened. It gates
  no other field's meaning.
- **Independent fact.** A field in a serialized evidence, report, or
  inspection record, where each field is a separate observation and all
  combinations are meaningful.
- **Lattice bottom.** An accumulator where `None` is the identity of a join.

## Converted

### Runtime

| Owner | Before | After |
| --- | --- | --- |
| Motion track sampling | running flag, optional curve, optional on-screen geometry | `UiTrackMotion::{Running, Settled}` and `UiTrackScreen::{Unpresented, OnScreen}` |
| Pointer presence owner | optional surface and binding beside the target | non-optional surface and binding; the transition fact carries the target shape |
| Mounted frame identity | published flag with optional receipts | `UiMountedFrameState::{Unpublished, Published}` |
| Pointer affordance staging | optional output with a staged flag | `UiStagedPointerOutput` |
| Retention inspection | optional diagnostics selection | `UiSelectedDiagnostics` |
| Appearance reconstruction | loose optional class and binding sets | `UiPendingAppearanceReconstruction` |
| Draft input stop | optional recipient with a recipient-kind flag | `UiLocalInputStopRecipient` |
| Scroll chrome latch | pending and held flags | `UiScrollChromeLatchState::{Idle, Pending, Held}` |
| Portal overlay binding | open/close flag with optional binding | `UiPortalOverlayBindingEffect::{Opens, Closes}`; preparation refuses a close that carries a stage, and `from_transition` returns `TransitionMismatch` rather than dropping it |
| Motion certification snapshot | three per-sample `Option`s, a presentation `Option`, two exclusive last-denial `bool`s | one `Option<UiMotionSampleCertification>` whose placement carries its presentation, and one `Option` last denial |

### Platform Pulse

| Owner | Before | After |
| --- | --- | --- |
| Native input ingress | armed flag and per-kind published flags, `Default` | `AwaitingFirstFrame` / `Armed(published kinds)`, no `Default` |
| Native application terminal state | `terminal_error`, `observation_error`, `terminal_reported` | `PlatformPulseTerminalPosture::{Running, Stopped { error, observation, reported }}` |
| Visual identity execution | `enabled` flag beside an optional journey state | `PlatformPulseVisualIdentityExecution::{Disabled(AwaitingFirstFrame \| Retired), Enabled(journey)}`; the journey state is no longer optional, and reentry is refused by `begin_transition` |
| Query lifecycle application | `BeforeInitial` beside an optional initial observation | `BeforeInitial(observation)` |

The terminal posture keeps the first error that stops Pulse. A later failure
no longer replaces it or reopens its report.

### Native host

| Owner | Before | After |
| --- | --- | --- |
| Readiness slot | optional work, optional pending generation, `pending`, `level_only` | `SlotReadiness::{Committed(Empty \| Committed \| Signaled), Level(Idle \| Signaled { generation })}` |
| Qualification plan | planned ordinal plus observed flag, three times; reconstruction class plus two sets | `UiNativePlannedInjection::{Unplanned, Armed, Spent}` and `Option<UiNativePendingReconstruction>` |
| Qualified external obligation | armed and observed flags, twice | `QualifiedObservation::{Unarmed, Armed, Reported}` |
| Completed presentation | `painted` flag with optional pixels | `UiNativePaintOutcome::{Unpainted, Painted { pixels }}` |
| Retained identity overlay | optional target and optional mechanic, `Default` | `Absent` / `Retained { target, mechanic }`, no `Default` |
| Retained paint order snapshot (native and headless) | `existed` flag, optional rank, optional predecessor | `OriginalOrderPlacement::{Unordered, Ranked { rank, predecessor }}` |
| Host state device and surface | two `Option`s set and taken together, with a shutdown arm for the mixed state | one `Option<UiNativePresentationOwners { device, surface }>`; the mixed-state arm is gone |
| Input observation profile | optional profile, two phase flags, optional transition tick | `UiNativeProfilePosture::{Unknown, Current, AwaitingCompletion, ObservationPending}` |
| Physical signal owner | optional worker beside terminal telemetry and counters | `UiNativePhysicalSignalRuntime::{Owned(worker), Retired(totals)}` |
| Physical progress grant | optional presentation, optional originating presentation, duplicate flag | `UiNativePhysicalProgressCorrelation::{Unattributed, Presentation, DuplicatePresentation, Originating}`; the certification constructor no longer takes a duplicate flag, since no caller set it |

### Host contract and headless host

| Owner | Before | After |
| --- | --- | --- |
| `UiMountedAppearanceWork` | optional predecessor frame and optional manifest, checked for equal presence | one optional predecessor holding both; the public constructor rejects a mismatched pair instead of storing it |
| Headless appearance transcript | the same pair | one optional `(frame, manifest)` |
| `WorthUiHeadlessMountedResourceCache` | optional binding beside handles keyed under it | one `Option<BoundResources { binding, by_content }>` |

Motion track sampling also lost its `queued` track and `begin_queued`. Nothing
ever set `queued`, so the drain that read it was dead code.

### Public surface changes

- The `UiMountedAppearanceWork` accessors `predecessor()` and
  `predecessor_manifest()` are no longer `const fn`.
- The pointer-presence transition gains the public accessors `previous_target()`
  and `current_target()`. `current_surface()` no longer returns an `Option`.
- `UiRuntimeServiceProposalStopReason` gains `CloseCarriesOverlayBindingStage`.

A native readiness test was removed:
`level_grant_take_does_not_clear_pending_state_before_generation_validation`.
It corrupted a slot into "pending without a generation", which the slot type
can no longer represent.

## Kept

### Runtime

- Motion geometry and visibility on published samples: independent facts of
  one sample.
- Coordinator latches, `release_on_drop`, scroll chrome hover and drag,
  and the wheel window: latches or owned data.
- Native shell `pending_viewport_basis`: an owned datum that is consumed when
  the next viewport is applied.
- Hit-test portal fields, and damage predecessor and successor: independent facts.
- Appearance retirements, unpublished frame receipts and predecessors, and
  the semantic predecessor: owned data.

### Platform Pulse

- `PlatformPulsePendingProjection.first`: an independent fact.
- `ActionPortRequest.completion` is a one-shot sender, which is an owned datum.
  `query_denial_requested` is authored input.
- Worker and watcher `Option`s in the theme, intent, external value, and visual
  readiness watches: taken once at shutdown or on drop.
- `PendingPreference`: a mailbox. `source_watch.pending` is a lookahead buffer.
- `PlatformPulseRetainedSnapshot.overlay_clear`: an optional receipt.
- `PlatformPulseApplicationRuntime`:
  - `startup_ready` and `external_close_requested` are latches.
  - `initial_source` is consumed at the first frame.
  - The shell, watchers, and `pending_*` fields are owned obligations.
- Visual identity journey readiness: installed later, and owns its signal.
- `observation_contract` records such as `PlatformPulseVisualComparison`, the
  shutdown evidence, and the intent traces: serialized independent facts.

### Native host

- `UiNativePendingWgpuObligation.presented` is a handoff slot.
  `terminal_indeterminate` is a latch.
- `UiNativePresentationRetryPolicy` wake, round, requirement, and effect
  posture: lattice accumulators.
- `AtlasCore.reservation` owns an id. `quarantined` is a fault latch.
- `text_atlas_completion`: a mailbox that outlives the in-flight upload.
  `UiNativeTextForegroundAtlasModel.presentation` is owned data.
- The text atlas plan's `committed` flag: a latch checked on drop.
- `UiNativePresentationEffects`: independent effect flags.
- `UiNativeEventLoopApplication`:
  - `client` is taken once at close.
  - `pending_close` and `failure` are latches.
  - `first_frame_presented` is a per-turn latch that owes observation readiness.
- `UiNativeInputObservationState`:
  - `ime_enabled` and `ime_composition_active` mirror separate platform
    reports, which winit does not order.
  - `stop_history_complete` is a latch.
  - The session, recipient, sequence, and terminal stop `Option`s are owned
    data.
- `UiNativePresentationSurface.suspended` and `occluded`: separate platform
  facts.
- `UiProtocolExecution.pending_readback_observed`: an evidence latch.
- Stop, run, and close certification reports (`client_cleanup_complete`,
  `cleanup`, `client_shutdown`, and the trace-complete flags): independent
  facts.

### Host contract and headless host

- `WorthUiHeadlessMountedResourceCache` and the appearance state keep
  `Default`. The converted inner state defaults to "nothing bound", which is
  the only valid empty value, so no variant is picked silently.
- `UiMountedPresentationAffinity.predecessor` and `receipt_affinity` are
  independent. An initial frame has no predecessor. A removal-only frame has
  no receipt affinity.
- `UiHeadlessRetainedPresentation.epoch` is issued after the retained frame, so
  it is owned data. `receipt_affinity` is independent, for the reason above.
- Capture `pixels_requested` is authored input. `UiHostCaptureObservation.pixels`
  is the artifact itself.
- The measurement environment generations, the portal group, collection row, and
  resource `Option`s in projection rows, and the report affinities: independent
  facts.
- `UiHeadlessMeasurementEnvironment` viewport and DPI: independent
  observations.
- The scripted presentation host's `indeterminate_next_registration` and
  `wrong_next_deregistration_receipt`: independent one-shot fault injections.
