# Typed-phase structural audit: worth-ui-runtime (3.17)

This audit was read-only. It covers worktree `C:/forge_workspace/worth-ui-3.17` at HEAD 2056f4bd4b, in a clean state.

**Paths.**
- Runtime paths are relative to `workspaces/worth-ui/crates/worth-ui-runtime/src/`. The same crate holds `facade/`, `graph/`, `mounting/` and `runtime/`.
- DSL paths are relative to `workspaces/worth-ui/crates/worth-ui-dsl/src/source/compile/`.

**Ruling.** The user ruled that "phases must ALWAYS be typed". This audit therefore treats every runtime check of phase, lifecycle, or authority as a finding, even when a test currently covers it.

**Method.**
- Scripted scans ran over production files only: the module tree walked from `lib.rs`, with test and `cfg(test)` modules excluded. They covered:
  - guards;
  - `..Default` / `_ =>`;
  - bool and Option fields;
  - functions over 60 lines or with 5+ args.
- Every non-numeric guard was then read by hand (1094 lines of context), along with every wildcard and the cited call sites.
- Every `file:line` below was opened and read.
- Nothing was compiled.

## 0. Inventory (production code only)

| Pattern | runtime crate | DSL `source/compile` | `worth-ui/src/facade` |
|---|---|---|---|
| `assert!` / `assert_eq!` / `debug_assert*` | 131 / 111 / 87 | 11 guards total | 0 |
| `unreachable!` / `panic!` | 118 / 24 | (incl. above) | 0 |
| `.expect(` / `.unwrap()` | 973 / 63 | (incl. above) | 0 |
| **Guards total** | **1419** (325 numeric/capacity, 1094 other) | 11 | 0 |
| `_ =>` wildcards | 153 | 14 | 0 |
| `..Default::default()` | 17 | 0 | 0 |
| `bool` fields | 460 | 1 | 0 |
| `Option<` fields | 2236 | 31 | 0 |
| Fns >60 lines or >=5 args | 1353 candidates (~831 after rough test filter) | 8 | 0 |

The worth-ui `facade/` is clean. The facade-implements debt lives in the runtime crate's `facade/entry/`, which has 131 production scrutiny candidates.

## 1. Ranking summary

The rank weights the next phases:
- 3b payload;
- 3c scalar text;
- 3d conditional presence;
- 3e participation;
- 3f appearance tokens;
- 4 rebind expression lane;
- 5 inspection.

Within that, it weights failures that are silently wrong (they compile and produce a wrong value) above failures that panic.

| Rank | Cluster | Phase(s) | Failure mode |
|---|---|---|---|
| 1 | P1 Participation lowering: bools override graph axes | 3e (3d) | silently wrong |
| 2 | P2 Mount eligibility: caller-supplied prior, no Unmount lane | 3d | silently wrong (latent) |
| 3 | P3 Mounted frame mode and lane set | 3d/3e/3f | runtime denial / misused denial |
| 4 | P4 Appearance aspect/value untyped pair | 3f | silently dropped |
| 5 | P5 Appearance demand owners (bool toggles, Option owners) | 3f | panic / facade implements |
| 6 | P6 Presentation work write-once and phase order | 3f | panic |
| 7 | P8 Intent payload projection | 3b | silently incomplete record / panic |
| 8 | P9 Intent owner-bundle duplication | 3b (all intent) | drift between bundles |
| 9 | P7 Query projection fact triple-Option | 3c/3g/4 | unreachable! |
| 10 | P14 DSL region/expression wildcards | 3d/4 | silently skipped / explanation lost |
| 11 | P10 Operability proof by debug_assert | 3b | wrong proof in release |
| 12 | P11 Inspection `non_exhaustive` + Optional generation | 5 | silently Unsupported / panic |
| 13 | P12 Service installation Options | all session flows | panic / reused stop reason |
| 14 | C Wide mounted-frame outcome | all publication flows | unreachable! arms |
| 15 | P13 Service proposal settlement completeness | session | runtime IncompleteOwnerSettlement |
| 16 | D Cutover transition stored as Option | replacement (overlaps succession) | expect/unreachable |
| 17 | I Miscellaneous phase guards | various | panic |
| - | E, currentness, freeze | succession agent | cross-reference only (section 4) |
| - | B Drop-guard completion handles | - | reclassified: local invariant plus debt |

---

## 2. Clusters

### P1. Participation lowering: bools override graph axes (rank 1; 3e, touches 3d)

**Owner.** Graph participation posture → mounted participation lowering.

**Evidence.**
- `mounting/projection/participation.rs:10-35`. The signature is `lower_participation(posture, admits_native_paint: bool, admits_hit_test: bool)`.
  - The Paint, Visible (clip) and HitTest axes go through `projected_fact`.
  - `motion` is hard-coded `Deferred` at `:31`.
- `mounting/projection/participation.rs:37-46`. `projected_fact` returns `Admitted` whenever the bool is true, whatever the graph axis says. A `Withheld` or `Deferred` graph axis is overwritten.
- `mounting/projection/lowering/node_lowering.rs:129`. The caller derives the bools:
  - paint from `semantic_text.is_some() || graph_node.has_appearance_attachment()`;
  - hit-test from `hit_test.is_some()`.
- At the same site, a 5-tuple `map_or((None, None, None, Default::default(), true), ...)` defaults portal paint to `true` (law 9: a positional tuple with a default).
- `graph/admission/graph_participation_seed.rs:100-140`. `deferred_axis` maps five different axes (Exists, Mounted, QueryBound, ServiceBound, Diagnostic) to one reason, `VisibleAxisAwaitsRuntimeMutation` (law 10: one reason reused for distinct facts).
- `graph/participation/participation_posture.rs`. `UiGraphAxisParticipation::new(status, source, reason, evidence_handle)` is a `pub const fn`, so any status/source/reason combination is constructible. `with_axis` accepts any participation for any axis.
- `graph/participation/participation_axis.rs` keeps a manual `COUNT` / `ALL` / `as_index` table (law 9: adding an axis relies on editing three places).
- `graph/participation/participation_mutation.rs:120-135`. `page_participation_mutation` matches a bool pair with `_ => None`.

**Laws.**
- Arch 3 (types over runtime discovery): the lowering discovers admission from `is_some()`.
- Arch 10: reasons are reused, and bools stand in for axis facts.
- Arch 16: the graph authority does not mint the participation.
- Arch 9: the positional tuple and the axis table.
- 3.17 "a denial is never false": a Withheld axis can be reported as Admitted.

**Failure (compiles, wrong).**
- 3e will add layout, focus and hit-test participation. A node whose graph HitTest axis is `Withheld` (for example, a conditionally absent region in 3d) but that still has a hit-test mechanic is lowered as HitTest `Admitted`. It then receives hits.
- The same holds for paint and clip, for any node with appearance.
- Motion is always Deferred, whatever the graph says.

**Target.**
```rust
// Graph authority mints the per-node participation; lowering can only narrow it.
pub(crate) struct UiAdmittedParticipation<'g> { axes: [UiGraphAxisParticipation; UiGraphParticipationAxis::COUNT], _g: PhantomData<&'g UiGraphSnapshot> }
pub(crate) enum UiMechanicPresence { Complete(UiMechanicReceipt), Absent }
fn lower_participation(p: UiAdmittedParticipation<'_>, paint: UiMechanicPresence, hit: UiMechanicPresence) -> UiMountedParticipation
// rule, in one table: mounted = min(graph_axis, mechanic); never max.
// UiGraphAxisParticipation constructors: one per (status, source) pair, e.g. ::runtime_admitted(evidence), ::deferred(reason: UiDeferredAxisReason) with a reason per axis.
// Axis table: derive COUNT/ALL from a single const array (or strum-like macro) so a new axis is one edit.
```

**Cost.**
- About 6 files: `participation.rs`, `node_lowering.rs`, `participation_posture.rs`, `participation_axis.rs`, `graph_participation_seed.rs` and `participation_mutation.rs`.
- Risk: medium. Some surfaces that paint today may stop painting where the graph axis was never flipped to Admitted. That is the intended exposure, but expect test churn in the mounted participation certifications.

### P2. Mount eligibility: caller-supplied prior, BecomeEligible-only, Layout forced to Admitted (rank 2; 3d)

**Owner.** Graph mount-eligibility authority.

**Evidence.**
- `graph/mount_eligibility/mount_eligibility_seed.rs`. `graph_eligibility_reserved: bool` is always true; its only constructor is `reserved()`. It is a vestigial flag (law 10).
- `graph/mount_eligibility/mount_eligibility_transition.rs:20-35`.
  - `from_slot_axis_transition` takes the prior from the caller.
  - The `(false, true)` / `(true, false)` match at `:26-27` returns `Option`, which conflates "no slot" with "no change".
- `graph/mount_eligibility/mount_eligibility_mutation.rs:7-9`. `UiGraphMountEligibilityMutationKind { BecomeEligible, BecomeIneligible }` exists as a kind.
- `graph/snapshot/graph_snapshot.rs:134-149` and `graph/closeout/graph_authority.rs:119` also accept the prior from the caller.
- Three writers each re-read the prior Mounted axis and call `runtime_mutation(Admitted)`:
  - `facade/entry/mounted_allocation_establishment.rs:245-262`;
  - `facade/entry/native_replacement_allocation.rs:215-245` (`expect("prepared graph node has one mount-eligibility slot")`);
  - `facade/prepared_application_authority/authority/graph_successor.rs:92-108`.
- `graph/mutation/mount_eligibility_admission.rs:30-80` re-validates at commit with runtime denials: ForeignMountEligibility (four distinct causes collapse into it), DuplicateNode, PriorMountedPostureMismatch, NextMountedPostureNotAdmitted and EmptyTransitionSet.
  - Each transition does a linear `nodes().iter().find` (`:38-42`).
  - `:67` rejects any next status other than Admitted, so **there is no Unmount lane at all**.
- `graph/mutation/graph_mutation_stage.rs:120-155`. `mount_eligibility_admitted_successor` sets `Layout = runtime_mutation(Admitted)` for every transition (`:145-150`) without reading `kind()`. If a BecomeIneligible transition were ever admitted, it would mark the node Mounted=Withheld and Layout=Admitted. This is a latent bug that 3d will hit.

**Laws.**
- Arch 16: the authority should mint the transition; callers do.
- Arch 1: the prior is not scoped to a snapshot handle.
- Arch 10: `ForeignMountEligibility` covers four causes; `graph_eligibility_reserved` is always true.
- Arch 5: the Option return.
- Arch 19: the framework should own the mount lifecycle.
- 3.17 "bounded invalidation": the dependent axes are not derived from one table.

**Failure.**
- 3d conditional presence needs unmount. Adding it by allowing a `Withheld` next status in admission compiles, and it produces Layout=Admitted on an unmounted node.
- Three writers must each remember to read the prior from the same snapshot.

**Target.**
```rust
impl UiGraphSnapshot {
    // Prior is read here, never passed in; foreign/duplicate/missing-slot are unrepresentable because the
    // token borrows the snapshot and is keyed by slot.
    pub(crate) fn mount_transition(&self, slot: UiGraphMountEligibilitySlotRef<'_>, to: UiMountIntent) -> UiGraphMountTransition<'_>;
}
pub(crate) enum UiMountIntent { Mount, Unmount }
pub(crate) enum UiGraphMountTransition<'g> { Mount(UiSlotRef<'g>), Unmount(UiSlotRef<'g>), Unchanged }
// Dependent axes derived from one graph-owned table:
const fn dependent_axes(t: UiMountIntent) -> &'static [(UiGraphParticipationAxis, UiGraphParticipationStatus)];
```
The three writers then collapse to one graph method. Delete `graph_eligibility_reserved`, and index the nodes by identity.

**Cost.**
- About 8 files: the graph files above plus the three facade writers.
- Risk: medium. The prior read moves into the graph, which is exactly where the succession agent's establishment and cutover writers call. **Coordinate:** the succession agent owns those writers. This cluster owns only the graph-side mint and the Unmount lane.

### P3. Mounted frame mode and lane set (rank 3; 3d/3e/3f)

**Owner.** Mounted frame assembly (`mounting/frame_assembler.rs`) and the mounted projection entry.

**Evidence.**
- `mounting/frame_assembler.rs:13-19`. `UiMountedLaneAssembly { ordinary, virtualized, canvas, realtime, preview: bool }` is used as two independent sets, `required` and `recorded`.
  - `..Default` at `:225-227`.
  - `record_*` at `:234-270` flip the bools.
  - `finish_with_reconciliation` (`:287`) compares `recorded != required` at runtime and returns `IncompleteManifest`.
- `facade/entry/mounted_preview/pending.rs:183-185` builds another `..Default` lane set.
- The lane derivation is duplicated:
  - `facade/entry/active_framework_turn/mounted_projection/lane_participation.rs` (`mounted_lanes`);
  - `facade/entry/application_replacement/mounted_frame.rs:104` (`candidate_lanes`), with `execute_candidate_lanes` at `:127-165` and `range.expect("admitted lane has a range")` at `:146`.
- `mounting/projection/prepared_projection.rs:160-180` records the per-lane Option receipts.
- `facade/entry/active_framework_turn/mounted_projection.rs:270`. `begin_mounted_projection` takes 8 args (91 lines), four of them Options: `predecessor`, `pointer`, `occurrence_geometry` and `theme`.
  - `:343-354` maps the `(Some, Some)` combination to `IncompleteManifest`. That denial means "a lane was not recorded", so this is a misused denial (law 10).
  - Four callers pass None patterns: `:140`, `:186`, `:215` and `:250`.
  - The finish matrix at `:153-168` picks `finish` vs `finish_for_reconciliation`, and retained vs theme vs plain appearance finish, from the same Options again.

**Laws.**
- Arch 1 and 16: the lane receipts are bools, not tokens.
- Arch 9: `..Default` on a semantic set.
- Arch 10: the misused `IncompleteManifest`.
- Arch 3: lane completeness is discovered at finish.
- Composition 5 (plan before effects): the mode is chosen twice, at begin and at finish.

**Failure.**
- 3d adds conditional regions that can appear or disappear per frame, and 3e adds participation lanes. A new lane or mode compiles in `begin` without its `finish` counterpart, and the error surfaces only as a runtime `IncompleteManifest`.
- An invalid Option combination compiles and yields a misleading denial.

**Target.**
```rust
pub(crate) enum UiMountedFrameMode<'f> {
    Fresh,
    Superseding(&'f UiMountedFrame),
    OccurrenceGeometry(&'f UiOccurrenceGeometryPlan),
    ThemeSwitch { predecessor: &'f UiMountedFrame, theme: &'f UiThemeAdmission },
    PointerSuccession { predecessor: &'f UiMountedFrame, pointer: &'f UiPointerAffordance },
    Reconciliation(&'f [UiReconciledOwner]),
    Reconstruction(&'f [UiReconstructedRegion]),
}
// begin returns a builder whose finish is selected by the same mode value.
pub(crate) struct UiRequiredLanes { /* from plan */ }
impl UiRequiredLanes { fn ordinary(&self) -> Option<UiLaneObligation<Ordinary>> /* ... */ }
fn record_ordinary(ob: UiLaneObligation<Ordinary>, ..) -> UiLaneReceipt<Ordinary>;
fn finish(assembly, receipts: UiLaneReceipts /* one field per lane, Option<UiLaneReceipt<L>> matched to obligations */) -> ...
```

**Cost.**
- About 7 files: `frame_assembler.rs`, `mounted_projection.rs` and its `lane_participation.rs`, `mounted_frame.rs` (replacement), `pending.rs` (preview) and `prepared_projection.rs`.
- Risk: medium-high. This is a hub with about 10 callers. It should land before 3d adds region lanes.

### P4. Appearance aspect/value untyped pair (rank 4; 3f)

**Owner.** Appearance projection (`runtime/appearance/projection`).

**Evidence.**
- `runtime/appearance/projection/resolved_aspect.rs`. `UiResolvedAppearanceAspect { aspect: UiAppearanceAspect, value: UiThemeValue, support: UiAppearanceSupportPosture, ... }` is an independent aspect/value pair plus a runtime support tag. `new` takes 9 args under `allow(too_many_arguments)`.
- Every consumer matches `(aspect.aspect(), aspect.value())` and silently drops unknown pairs:
  - `mounting/projection/appearance/style.rs:75` (`_ => {}`) and `:98` (`_ => None`);
  - `mounting/projection/appearance/overlay_input.rs:119`;
  - `mounting/projection/appearance/scroll_chrome.rs:110`;
  - `mounting/projection/frame_storage/appearance_state_lowering.rs:96`.

**Laws.**
- Arch 9 and 10: the pair admits invalid combinations, such as Opacity with a color value.
- Arch 5: support is a runtime tag.
- 3.17 "one canonical form".

**Failure.** 3f adds appearance tokens. A new token aspect, or a mismatched value, compiles and is silently not painted at five sites. No diagnostic is retained.

**Target.**
```rust
pub(crate) enum UiAppearanceValue { Background(UiColor), Foreground(UiColor), Opacity(UiUnitInterval), Radius(UiCornerRadii), Outline(UiOutline) /* one per aspect */ }
pub(crate) enum UiResolvedAppearance { Supported(UiAppearanceValue, UiAppearanceProvenance), Unsupported(UiAppearanceAspect, UiUnsupportedReason) }
```
Consumers then match exhaustively on `UiAppearanceValue`, with no wildcard. Split `new` into a builder or a provenance struct.

**Cost.**
- About 7 files.
- Risk: low-medium. The change is mechanical, and the wildcards become compiler-driven.

### P5. Appearance demand owners (rank 5; 3f; extends cluster F)

**Owner.** Active application session activation (the demanded appearance owners).

**Evidence.**
- `runtime/intent/admission/state.rs:60-75`. `reconcile_operability_appearance(enabled: bool)` with `standing_owner: Option`.
- `runtime/intent/payload/application_fact_state.rs:112-126`. `reconcile_validation_appearance(bool)`.
- `runtime/interaction/state.rs:230-241`. `reconcile_pointer_observation_demand(presence: bool, pressed: bool)`.
- `facade/entry/active_application_session/activation.rs:70-95` (inside `new`, which is 243 lines) checks axis demand against `service_policy_plan.focus()/selection()` and returns `AppearanceOwnerUnavailable`.
- Later sites re-assert the owner:
  - `facade/entry/observation.rs:82-114`, `:52`, `:131` and `:139`;
  - `facade/entry/appearance_close.rs:54-83`;
  - `facade/entry/pointer_affordance_close.rs:57`;
  - `facade/entry/active_application_session.rs:338` (`.expect("demanded X owner")`).
- `facade/entry/intent_payload.rs` (`evaluate_intent_operability`) implements the demand check and the standing-fact recording inside the facade (composition 13).

**Laws.**
- Arch 11 (typed injected context).
- Arch 16: the activation check is not carried as a witness.
- Arch 5: the bool toggles.
- Composition 13.

**Failure.** 3f adds token-driven appearance axes. A new demanded axis compiles without an activation check, and it panics on first use when the owner is absent.

**Target.**
```rust
pub(crate) struct UiDemandedAppearanceOwners { focus: Option<UiFocusAppearanceOwner>, selection: Option<..>, operability: Option<..>, validation: Option<..>, pointer: Option<..> }
// built once at activation from the demand set; each consumer takes `&UiFocusAppearanceOwner`, not the session.
fn reconcile_operability_appearance(&mut self, demand: UiAppearanceDemand<Operability>) // Demanded(owner) | NotDemanded
```

**Cost.**
- About 8 files.
- Risk: low-medium.

### P6. Presentation work: write-once binds and phase order (rank 6; 3f)

**Owner.** Mounted presentation authority and work producer.

**Evidence.**
- `mounting/presentation/work_producer.rs:71-76`. `issue_initial` does `assert!(self.predecessor.is_none())`.
- `mounting/presentation/work_producer.rs:108-113`. `issue_reconstruction` does `assert_eq!(self.predecessor, Some(predecessor))`.
- `mounting/presentation/authority/work.rs:60-80`. `bind_layout_owner` and `bind_appearance` assert "binds exactly once".
- `mounting/presentation/coordinator/work_preparation.rs:60-95` binds them in sequence, and `:81` consumes the appearance.
- `mounting/assembly/prepared_frame.rs:290-302`. `appearance_output_available() -> bool` is paired with an `appearance_projection()` that expects "presentation begins only after appearance output admission". The bool is checked at `mounting/presentation/state/appearance_admission.rs:98`.
- `mounting/presentation/.../projection_owner.rs:10,43`. `unpublished_appearance: Result<Option<..>, Denial>` is a three-state value encoded in nested wrappers.
- The H guards cover the same authority: `mounting/presentation/authority/validation.rs:13-150`, `appearance_retry.rs:5`, `consumption_view.rs:56`, `coordinator.rs:169`, `work_producer/state.rs:283` and `work_preparation.rs:47`.

**Laws.** Arch 1, 16 and 19. Arch 5 (the bool, and the `Result<Option>`).

**Failure.** Calling `appearance_projection()` without the admission check, or binding twice, compiles and panics. A 3f token-refresh path that re-binds appearance is exactly that shape.

**Target.**
```rust
fn admit_appearance(frame: UiPreparedFrame) -> Result<UiAppearanceAdmittedFrame<'_>, UiAppearanceAdmissionDenial>;
struct UiPresentationWork<S> { .. } // Unbound -> LayoutBound -> Bound
impl UiPresentationWork<Unbound> { fn bind_layout(self, o: UiLayoutOwner) -> UiPresentationWork<LayoutBound> }
impl UiPresentationWork<LayoutBound> { fn bind_appearance(self, a: UiAppearanceAdmittedFrame<'_>) -> UiPresentationWork<Bound> }
enum UiWorkIssue { Initial, Reconstruction(UiPredecessorWork) } // replaces the two asserts
enum UiUnpublishedAppearance { None, Pending(..), Denied(..) }
```

**Cost.**
- About 6 files.
- Risk: medium, because the presentation state machine has many certifications.

### P8. Intent payload projection (rank 7; 3b)

**Owner.** Intent payload (`runtime/intent/payload`).

**Evidence.**
- `runtime/intent/payload/projection.rs:16-60`. `prepare_intent_payload(route, definitions, execution_bindings, generation, owners, occupancy)` takes **6 params** (coordinator lead).
  - The facade wrapper `facade/entry/intent_payload.rs:4-30` only assembles them.
  - `projection.finish()` returns a positional 5-tuple (composition 3: named facts).
- `runtime/intent/payload/projection.rs:93-225`. Four parallel Vecs (`values`, `query_inputs`, `application_inputs`, `owner_revisions`) are pushed per branch, and the branches do not all push the same set. A branch that forgets `owner_revisions.push` compiles (law 9).
- `application_input(field, fact, expected: UiIntentPayloadFieldKind)` checks the kind at runtime. Its callers then re-extract with expects:
  - `application_text` at `:262` (`.text_value().expect("validated application text fact has text shape")`);
  - `application_boolean` at `:275`;
  - `application_unsigned64` at `:288`.
- `projection_text` and `projection_selection` return a runtime `ProjectionShapeMismatch`.
- Related (cluster G): `axis_observation.rs:183,186`, `input_basis.rs:93` and `application_fact_state.rs:111,264` expect the fact value shapes.

**Laws.** Arch 9 (parallel Vecs), arch 3 and 10 (the runtime kind check, then an expect), arch 5 (the tuple), composition 3.

**Failure.** 3b extends the payload fields. A new field kind compiles:
- with a record missing from one of the four Vecs, which makes the input basis incomplete and currentness checks silently weaker;
- or with an expect that panics on a shape mismatch the type system could have ruled out.

**Target.**
```rust
fn application_input<K: UiIntentFieldKind>(field: &UiIntentField<K>, fact: &UiApplicationFact) -> Result<UiTypedApplicationInput<K>, UiPayloadDenial>;
struct UiProjectedField { value: UiIntentPayloadValue, input: UiPayloadInputRecord /* enum: Query{..} | Application{..}, owns its revision */ }
struct UiPreparedIntentPayload { values: Box<[UiIntentPayloadValue]>, inputs: UiIntentInputBasis /* built from the records */ , .. } // replaces the 5-tuple
fn prepare_intent_payload(reads: &UiIntentGenerationReads<'_>, route: &UiIntentRoute) -> Result<UiPreparedIntentPayload, UiPayloadDenial>; // 2 params, via P9
```

**Cost.**
- About 4 files.
- Risk: low. The 3b WIP branch (`worth-ui-3.17-3b-wip`) is touching this, so land it together with that work.

### P9. Intent owner-bundle duplication (rank 8; 3b and all intent phases; coordinator lead)

**Owner.** Intent read contexts.

**Evidence.** Seven overlapping structs bundle the same references:

| Struct | Location | Fields |
|---|---|---|
| `UiIntentAdmissionCurrentnessContext` | `runtime/intent/admission/currentness.rs:11-19` | catalog, definitions, generation, mounted, application_facts, expressions, command_contexts |
| `UiIntentConfirmationReadContext` | `runtime/intent/confirmation/validation.rs:8-16` | catalog, definitions, generation, mounted, application_facts, occupancy, expressions |
| `UiIntentInputOwners` | `runtime/intent/payload/input_basis/view.rs:11-15` | mounted, application_facts, expressions |
| `UiIntentOperabilityDependencyReads` | `runtime/intent/operability/basis.rs:194-199` | mounted, application_facts, expressions, generation |
| `UiIntentOperabilityAuthority` | `runtime/intent/operability/standing_observation.rs:44-59` | catalog, definitions, execution_bindings, occupancy |
| `UiIntentOperabilityReadOwners` | `runtime/intent/operability/standing_observation.rs:44-59` | authority, generation, inputs |
| `UiIntentConsequenceCurrentnessContext` | `runtime/intent_execution/state/consequence.rs:14-18` | catalog, generation, mounted |

The struct literals are rebuilt by hand at:
- `facade/entry/intent_admission.rs:103`;
- `facade/entry/intent_execution.rs:36`;
- `facade/entry/intent_confirmation.rs:20` and `:46`;
- `runtime/intent/operability/standing_observation.rs:82` (re-bundles `ReadOwners` into `ConfirmationReadContext`);
- `runtime/intent/admission/currentness.rs:226` and `runtime/intent/confirmation/validation.rs:112` (re-bundle into `DependencyReads`).

The `UiIntentInputOwners` literals are repeated in:
- `expression_access.rs:93`;
- `active_framework_turn.rs:304`;
- `intent_operability_observation.rs:17`;
- `intent_payload.rs:22`;
- `mounted_preview.rs:94`;
- `pointer_affordance_close.rs:75`.

The outcome shapes are inconsistent:
- `payload_inputs_are_current` returns a bare `bool` (`input_basis.rs:113`, `prepared.rs:91`);
- `operability_dependencies_are_current` returns `Result<(), Drift>`;
- `validate_challenge` returns `Option<StopReason>`, where `None` means success (an inverted outcome).

**Laws.**
- Arch 11 (typed injected context): the bundles are hand-assembled.
- Arch 1: no bundle is scoped to a generation.
- Arch 5: bool vs Result vs inverted Option.
- Composition 13: the facade assembles the owner sets.

**Failure.** Adding an owner in 3b (for example, a new payload source) compiles when it is added to one bundle and not the others. Admission and confirmation then read different owner sets for the same intent.

**Target.**
```rust
#[derive(Clone, Copy)]
pub(crate) struct UiIntentGenerationReads<'g> { authority: UiIntentOperabilityAuthority<'g>, generation: &'g UiApplicationGeneration, inputs: UiIntentInputOwners<'g> }
impl ActiveApplicationSession { fn intent_reads(&self) -> UiIntentGenerationReads<'_> } // the only mint site
impl UiIntentGenerationReads<'_> { fn dependencies(&self) -> UiIntentOperabilityDependencyReads<'_> }
struct UiIntentAdmissionContext<'g> { reads: UiIntentGenerationReads<'g>, command_contexts: &'g .. }
struct UiIntentConfirmationContext<'g> { reads: UiIntentGenerationReads<'g> }
fn payload_inputs_are_current(..) -> Result<(), UiIntentInputDrift>;
fn validate_challenge(..) -> Result<UiValidatedChallenge, UiIntentStopReason>;
```
The **"observed at generation G" type** belongs to the succession agent. `UiIntentGenerationReads<'g>` is the natural carrier for it, so hand the struct to that agent rather than designing the currentness proof here.

**Cost.**
- About 14 files. Each is a small change (struct-literal replacement).
- Risk: low.

### P7. Query projection fact triple-Option (rank 9; 3c scalar text, 3g, 4 rebind)

**Owner.** Produced Query facts (`fact_contract/produced/query.rs`).

**Evidence.**
- `fact_contract/produced/query.rs:98-150`. `scalar_projection()`, `application_scalar_projection()` and `collection_projection()` are three Option accessors over one internal enum, `UiProjectionObservation`.
- `UiQueryChangedFactKind::ScalarProjection` covers both Scalar and ApplicationScalar (law 10).
- `runtime/rebind/planning/content_plan.rs:95` and `:213` reconstruct the enum from the three Options, with `unreachable!("a Query projection fact has one sealed shape")`.

**Laws.** Arch 5 and 10, and 3.17 "one canonical form".

**Failure.** 3c scalar text adds another projection shape. It compiles as a fourth Option accessor, and the `content_plan.rs` reconstruction reaches `unreachable!`.

**Target.**
```rust
pub(crate) enum UiQueryProjectionFactRef<'a> { Scalar(&'a ..), ApplicationScalar(&'a ..), Collection(&'a ..) }
fn projection(&self) -> Option<UiQueryProjectionFactRef<'_>>;
enum UiQueryChangedFactKind { Scalar, ApplicationScalar, Collection, .. }
```

**Cost.**
- About 3 files.
- Risk: low. This is the cheapest high-value item.

### P14. DSL region and expression wildcards (rank 10; 3d, 4)

**Owner.** DSL sealed semantic package and expression admission.

**Evidence.**
- `sealed_semantic_package/region_bindings.rs:13`. `_ => continue` over the declaration kinds. A new region-bearing declaration (3d conditional presence) would be skipped silently.
- `expression_admission/diagnostics.rs:38` and `:89` collapse owned exhaustive enums into generic codes:
  - `WorthUiExpressionDeclarationErrorKind` (worth-ui-dsl);
  - `ExpressionDenialFamily` (worth-foundational).
  This loses retained explanation in the Phase 4 expression lane.
- `appearance_validation.rs:46` (`_ => {}`), `overlay_identity_resolution.rs:87` and `rust_overlay_identity_resolution.rs:35` are wildcards over owned kinds (3f adjacent).

**Laws.** Arch 9 (wildcards on owned enums) and 3.17 "retained explanation".

**Failure.**
- A 3d region-bearing declaration compiles and gets no region binding.
- A 4 expression denial compiles and surfaces with a generic code.

**Target.** Exhaustive matches that list every non-region kind explicitly, and a one-to-one diagnostic code per denial variant.

**Cost.**
- About 5 files.
- Risk: low.

### P10. Operability proof by `debug_assert` (rank 11; 3b)

**Evidence.**
- `runtime/intent/operability/proof.rs:23-33` and `:62-70`. The proof constructors carry `debug_assert!(decision.is_operable())` and `!is_operable()`. In release builds, any decision can mint either proof.
- `runtime/intent/operability/decision.rs:239` has `_ => None` over ordinal match arms.

**Laws.** Arch 16 (sealed proofs) and 3.

**Failure.** A caller mints an operable proof from an inoperable decision. The mistake compiles and passes in release.

**Target.** `impl UiOperabilityDecision { fn classify(self) -> Result<UiOperableDecision, UiInoperableDecision> }`, with the proofs constructed only from the matching arm. Make the ordinal match exhaustive.

**Cost.**
- 2 files.
- Risk: very low.

### P11. Inspection: `non_exhaustive` workspace enums, Optional generation (rank 12; 5)

**Evidence.**
- Three enums in workspace crate worth-ui-inspection are `#[non_exhaustive]`:
  - `UiInspectionScope` (`scope/inspection_scope.rs:2`);
  - `UiInspectionTarget` (`target/inspection_target.rs:52`);
  - `UiInspectionTargetClass` (`query/relevance_outcome.rs:15`).
- That forces wildcards in the runtime crate:
  - `facade/inspection_bridge/dispatch.rs:40` (`_ => UnsupportedTarget`);
  - `declaration/support/inspection_projection.rs:37` (`_ => return None`).
- `facade/inspection_bridge/admission.rs:9-17`. `InspectionAuthority { generation: Option<..> }` is always Some. As a result, `facade/inspection_bridge/routes.rs` repeats `expect("graph-backed inspection has one active generation")` at `:56`, `:115`, `:140`, `:164` and `:195`.
- `facade/inspection/evidence_ref_expansion.rs:146` and `:148` are bare `unreachable!()`.

**Laws.** Arch 9 (the wildcard forced by `non_exhaustive`), arch 1, and arch 5 ("a denial is never false": an unhandled target reports `UnsupportedTarget`).

**Failure.** Phase 5 adds inspection targets. A new target compiles and is reported as `UnsupportedTarget`.

**Target.** Remove `#[non_exhaustive]` from workspace-internal enums; it buys nothing inside a workspace. Make `generation` non-Optional, or split out a `GraphBackedInspectionAuthority`. Give the two `unreachable!()`s messages, or better, types.

**Cost.**
- About 5 files across 2 crates.
- Risk: low. It may surface additional match sites, which is the goal.

### P12. Service installation Options (rank 13; cluster A)

**Owner.** Active application session services.

**Evidence.**
- `runtime/service_installation.rs`. `UiRuntimeServiceInstallation<Owner> { owner: Option<Owner> }` exposes `is_installed` / `as_ref` / `as_mut` / `take`.
- `facade/entry/active_application_session.rs:164-221` holds:
  - six installations (focus, portal, motion, scroll, selection, command routing);
  - `dormant_portal_stack_ordinal_issuer: Option`, which is an either-or with portal (`activation.rs:162-170`, `.take().expect`);
  - `ime_composing: bool`;
  - five further Options: `appearance_theme_admission`, `observation_clock`, `pointer_affordance_snapshot`, `appearance_owner_snapshot` and `last_scroll_settle_stop`.
- `declaration/service/normalized_plan.rs:11-75` encodes no portal ⇒ focus + motion dependency. Only smooth-wheel ⇒ motion is checked.
- `facade/entry/active_application_session/portal_dismissal.rs:184-250` checks `!focus.is_installed() || !motion.is_installed()`, returns `Stopped(Proposal)` (a reused stop reason; law 10), and then repeats `.expect("... installation was checked above")`.
- `facade/entry/active_application_session/service_state_reconciliation.rs:38-95` repeats `.expect("mounted Scroll ownership was checked above")`.
- `facade/entry/active_application_session/portal_exit_publication.rs:40-66` does `assert!(motion.release_exit_retention(..))` (a bool outcome asserted; law 5). The same pattern appears at `facade/entry/active_application_session/motion_sampling.rs:70-91`.
- Other `is_installed` callers:
  - `command_observation.rs:9`;
  - `scroll_chrome_appearance.rs:50`;
  - `application_commit.rs:68`;
  - `interaction.rs:34, 91, 98, 188`;
  - `mounted_identity.rs:204`;
  - `mounted_interaction_lifecycle.rs:58, 64`.
- The remaining A expect sites were read in the guard pass:
  - `mounted_identity.rs:212`;
  - `mounted_interaction_lifecycle.rs:61,68`;
  - `mounted_occurrence_geometry.rs:67`;
  - `intent_consequence_rebind.rs:90-194`;
  - `mounted_publication.rs:188`;
  - `command_observation.rs:41`;
  - `scroll_direct_control.rs:59-83`;
  - `scroll_observation.rs:127,205`;
  - `portal_settlement.rs:22-112`;
  - `recovery.rs:50-93`;
  - `stop.rs:57,61`;
  - `native_managed_rebind/intent_consequence.rs:197,201`;
  - `portal_dismissal_recovery.rs:125,129`;
  - `native_managed_rebind/shutdown.rs:61-167`;
  - `portal_dismissal/completion.rs:62-215`;
  - `portal_dismissal/handles.rs:191,196`;
  - `focus_settlement.rs:47`;
  - `selection_replacement.rs:42`;
  - `appearance_theme.rs:62`;
  - `application_replacement/portal_lifecycle.rs:83-130`.

**Laws.**
- Arch 3: installation is discovered at runtime.
- Arch 16: the check is not carried forward.
- Arch 5 and 10: the reused stop reason and the asserted bools.
- Arch 19.

**Failure.** Any new flow that needs focus + portal + motion compiles without the check, and it panics under a service plan that omits one of them.

**Target.**
```rust
// Normalization owns dependencies; the plan yields grouped units.
pub(crate) struct UiPortalServices { portal: UiPortalOwner, focus: UiFocusOwner, motion: UiMotionOwner }
struct UiSessionServices { portal: Option<UiPortalServices>, scroll: Option<UiScrollServices>, selection: Option<..>, command_routing: Option<..> }
// flows take the unit: fn dismiss_portal(services: &mut UiPortalServices, ..)
enum UiPortalOrdinalSource { Installed(UiPortalServices), Dormant(UiPortalStackOrdinalIssuer) }
fn release_exit_retention(..) -> Result<UiReleasedRetention, UiRetentionMismatch>;
```

**Cost.**
- About 30 files. This is the widest cluster.
- Risk: medium. Do it incrementally, one service group per change, starting with the Portal group because 3e/3f touch focus.

### C. Wide mounted-frame outcome (rank 14)

**Evidence.**
- `mounting/publication.rs:41-52`. `UiMountedFrameOutcome` has 10 variants, returned from every mode.
- `mounting/presentation/outcome.rs:76-82` has `UiMountedPresentationOutcome`.
- Callers carry `unreachable!` / `panic!` arms for variants their mode cannot produce:
  - `facade/entry/intent_consequence_publication.rs:331-341`;
  - `mounted_content_rebind.rs:332,358`;
  - `native_intent_posture/execution.rs:316-323`;
  - `portal_exit_publication.rs:227`;
  - `portal_dismissal/completion.rs:167,171`;
  - `mounted_preview/presentation.rs:222`;
  - `progression/presentation.rs:149`;
  - `session_state/replacement/settlement.rs:15,41`;
  - `coordinator/settlement.rs:204`;
  - `content_mapping.rs:67,131`;
  - `mapping.rs:57,101`;
  - `intent_consequence.rs:187,244,256`;
  - `native_theme_switch.rs:150`;
  - `native_managed_rebind/intent_consequence.rs:71`;
  - `native_managed_rebind/portal_dismissal.rs:177`;
  - `portal_dismissal_recovery.rs:167`.

**Laws.** Arch 5 and 3.

**Failure.** A mode that gains a new outcome compiles, and it hits another mode's `unreachable!`.

**Target.** A per-mode outcome type, derived from `UiMountedFrameMode` in P3 (for example, `UiReconciliationFrameOutcome` without the `Superseded` arm). **Do this together with P3.**

**Cost.**
- About 20 caller files, each a small change.
- Risk: medium.

### P13. Service proposal settlement completeness (rank 15)

**Evidence.**
- `runtime/session/service_proposal/compiler/settlement_compiler.rs:1-100` runs this sequence:
  1. `begin_settlement`;
  2. N × `acknowledge_owner(&mut settlement, ack)`;
  3. `finish_settlement`, which checks `settlement.is_complete()` at runtime, returns `IncompleteOwnerSettlement`, and does `terminal_parts().expect`.
- The callers all expect success:
  - `runtime/session/application_state/service_proposal/settlement.rs:19-115, 147-239, 251, 280, 325`;
  - `:185` is `unreachable!("Portal Closing and Motion ExitRetention are one decision per transition")` over two Options;
  - `:164` and `:167` are `panic!("validated Motion proposal must commit")`;
  - `:200` is `selection_state.expect`.
- Same pattern elsewhere:
  - `terminal.rs:49-92`;
  - `scroll_settle_settlement.rs:50-127`;
  - `portal_frame_binding.rs:39-196` (`discard_portal_proposal`);
  - `scroll_settle.rs:78`;
  - `compiler/terminal.rs:140-141` (`debug_assert_eq!` on the release counts);
  - `occupancy.rs:286`.

**Laws.** Arch 19 (the framework owns lifecycles), arch 16, and composition 5.

**Failure.** Forgetting one `acknowledge_owner` compiles, and the settlement is denied at runtime.

**Target.**
```rust
struct UiOwnerAcknowledgements { portal: UiPortalAck, focus: UiFocusAck, scroll: UiScrollAck, selection: Option<UiSelectionAck>, motion: Option<UiMotionAck> }
fn settle(proposal: UiCompiledProposal<Owners>, acks: UiOwnerAcknowledgements) -> UiSettledProposal;
enum UiPortalMotionDecision { Closing(UiPortalClosing), ExitRetention(UiMotionExitRetention), Neither } // replaces :185
```
The owner set comes from the compiled proposal, so the required fields are known at compile time. If the owner set is dynamic, fall back to a per-owner Option whose presence is decided by `UiCompiledProposal`.

**Cost.**
- About 8 files.
- Risk: medium.

### D. Cutover transition stored as Option (rank 16; overlaps succession)

**Evidence.**
- `facade/entry/application_replacement.rs:123-131`. `WorthUiApplicationCutoverTransition { Prepared, Committed }` is held in an `Option`.
- `prepared_activation_access.rs:51-62` uses expect and `unreachable!`.
- Further sites:
  - `receipt.rs:74-120`;
  - `cutover/application_commit.rs:20,24`;
  - `cutover/outcome.rs:27`;
  - `mounted_frame.rs:75`;
  - `owner_succession.rs:71`;
  - `mounted/admission.rs:59`;
  - `mounted/appearance_projection.rs:51`;
  - `cutover.rs:266-267`;
  - `cutover_generation.rs:18-19`.

**Target.** `WorthUiPreparedApplicationActivation<Phase>`, with Prepared → Committed by value.

**Coordination.** The succession agent owns the cutover writers, so hand this cluster to them. This audit only records that the transition enum stored in an Option is the typestate gap.

### I. Miscellaneous phase and lifecycle guards (rank 17)

These guards encode "X happened before Y" and are not in the clusters above. Each is small; convert each one when its owner is touched, not as a campaign.

**Stream policy, sources and handoff**
- `runtime/stream_policy/resolution/accessors.rs:31,79`. The accepted/rejected plan has resolved/denied Option expects; this is a typestate candidate.
- `stream_policy/composition/pair_policy.rs:228` is `unreachable!`.
- `source_ingress/provider.rs:59` has an `assert_ne!` on the provider kind (law 10).
- `semantic_handoff_preparation/evidence.rs:170-391` expects sealed shapes. `:352` panics on a stringly field name (law 5 and 10).

**Frames, capture and observation**
- `runtime/viewport_resize/outcome.rs:96`.
- `observation/state.rs:88,114`.
- `dispatcher.rs:140,189`; `submission.rs:13`.
- `framework_turn/completion.rs:191,203`.
- `capture_handle.rs:151,157`; `capture_progression.rs:277,284`.
- `facade/entry/visual_overlay.rs:47-186`.

**Allocation and planning**
- `allocation_transaction.rs:149`; `candidate.rs:66`; `committed_allocation.rs:21`.
- `allocation_planning/basis.rs:61-103`; `plan_allocation.rs:73,100`.
- `execution_plan_lowering_authority.rs:211-227`; `execution_plan.rs:154-168`.
- `query_aware_plan_outcome.rs:50-243`; `mounted_catalog_gate.rs:180`.
- `receipt_ledger.rs:358`; `receipt_ledger_entry.rs:213`; `mounted.rs:97`.

**Intent lifecycle**
- `prepared.rs:159,184`; `intent_execution/state.rs:271,278`.
- `state/consequence.rs:39-397`; `lifecycle.rs:48`; `state/recovery.rs:47`.
- `progression/settlement.rs:184`.
- `confirmation/continuation.rs:123,166`; `confirmation/lifecycle.rs:61`; `confirmation/observation.rs:69,111`.
- `admission/settlement.rs:76`; `occupancy.rs:193`; `standing_observation/inspection.rs:34`.

**Portal, pointer and motion**
- `portal/state/commit.rs:27-231`; `portal/dismissal.rs:121,170`; `portal/rebind.rs:123,168`.
- `portal/state/mounted_projection.rs:86`; `portal_overlay.rs:120`.
- `interaction/draft/*`; `pointer/transition.rs:260-297`.
- `motion/state.rs:177-346`; `motion/track.rs:147`.

**Publication, session state and retention**
- `session_state/publication.rs:249,280`; `publication/reconciliation.rs:109-242`; `publication/settlement.rs:78`.
- `session_state.rs:134-138`; `session_state/shutdown.rs:26-56`; `service_shutdown.rs:104,108`.
- `retention/coordinator.rs:117,264-292`; `retention/authority.rs:195-319`; `pin_accounting.rs:60,75`.
- `overlay_registry.rs:96-190`.

**Rebind, scope and receipts**
- `receipt.rs:87,318`; `graph_lookup_boundary.rs:114`.
- `rebind/planning/compiler.rs:43,259`; `plan.rs:251`; `rebind/execution/receipt.rs:327,346`.
- `scope/recoverable.rs:41`; `scope/resolver.rs:129`.

**Declaration and admission**
- `admitted_mosaic_state_family.rs:21-87`; `frozen_mosaic_state_capabilities.rs:79-91` (`MissingForDiagnostics`, law 10).
- `declaration/family/admission.rs:164`; `structural_semantics/admission.rs:294,326`.
- `produced/fact.rs:150`; `observation/admission/host.rs:167`; `observation/admission/intent.rs:230`.
- `classifier.rs:59-163`; `classification/owner/host.rs:32`; `host_request_shape_digest.rs:11-74`.
- `receipt_inspection.rs:189`; `inspection_receipt/receipt.rs:69`.

**Mounted lowering, appearance and scheduling**
- `topology_mutation.rs:19`; `plan_builder.rs:39`; `lowering.rs:186,213`.
- The four "resolved handle evidence is not a denial" guards in `frame_executor.rs`.
- `appearance/clip.rs:49-50`; `appearance_state_membership.rs:215`; `text_paint.rs:32-33`.
- `epoch.rs:64`; `delta.rs:100`; `intent_posture/table.rs:54`.
- `async_correspondence.rs:143`; `work_observation.rs:56`; `pending_completion.rs:41`.
- `motion_sampling.rs:185,188`; `presentation_recovery.rs:77`.

The strongest of these are:
- `stream_policy/resolution/accessors.rs` (a clean Accepted/Rejected split);
- `evidence.rs:352` (a stringly field name);
- `frozen_mosaic_state_capabilities.rs:79-91` (a denial variant reused for diagnostics).

---

## 3. Composition debt (coordinator leads plus scrutiny)

**Coordinator leads.**
- **The context duplication is P9 above.** There are seven overlapping bundles and about 14 hand-built literal sites. Target: one `UiIntentGenerationReads<'g>` minted by the session.
- **`prepare_intent_payload`'s 6 params is P8 above.** It falls to 2 params once P9 exists, and the 5-tuple becomes `UiPreparedIntentPayload`.
- **The freeze.rs duplication (`prepare_application_authority` at 91 lines vs `prepare_successor_application_authority` at 118 lines)** belongs to the succession agent. Cross-reference only.

**Facades that implement (composition 13 and 2).** These are the worst `facade/entry` functions (orchestrators that are not tables of contents):

| Function | Size |
|---|---|
| `intent_consequence.rs:76` (`publish_intent_consequence_handoff`) | 310 lines |
| `appearance_projection.rs:22` (`finish`) | 294 lines |
| `active_application_session/activation.rs:8` (`new`) | 243 lines; it also hosts the P5 demand check |
| `native_managed_rebind.rs:84` | 215 lines |
| `interaction.rs:53` | 182 lines |
| `scroll_extent_retarget.rs:39` | 181 lines, 5 args |
| `scroll_observation.rs:59` | 174 lines, 7 args |
| `native_managed_rebind/shutdown.rs:4` | 171 lines |
| `service_state_reconciliation.rs:38` | 166 lines |
| `intent_consequence_rebind.rs:49` | 166 lines |
| `overlay_appearance.rs:129` (`lower_overlays`) | 147 lines, 11 args |
| `overlay_appearance.rs:76` | 11 args |
| `overlay_appearance.rs:44` | 10 args |
| `rebind_execution.rs:40` | 118 lines, 6 args |
| `application_replacement/mounted.rs:56` | 108 lines, 10 args |
| `rebind_execution.rs:267` | 104 lines, 6 args |
| `active_framework_turn/mounted_projection.rs:270` (`begin_mounted_projection`) | 91 lines, 8 args; resolved by P3 |
| `mounted_content_rebind.rs:94` | 9 args |
| `evidence_only_successor_commit.rs:12` | 7 args; succession agent's writer |

The production candidate counts by directory are:
- mounting/projection 52;
- runtime/planning 34;
- mounting/presentation 33;
- appearance 25;
- replacement 24;
- interaction 24;
- session 23;
- allocation_neighborhood 23;
- source/lower 22;
- overlay_composition 21.

The DSL has two candidates, both in `semantic_package_exact_basis/declaration_basis.rs`:
- `:60` (69 lines);
- `:130` (`fold_into`, 93 lines).

Most of the large facade functions shrink mechanically once P3, P5 and P12 land. The mode enum, the demanded owners and the service units remove most of their branching. Recommendation: do not split them first. Split what remains after the typed clusters land.

## 4. Cross-references to the succession agent (not redesigned here)

- **E, prepared succession guards:**
  - `mounted_owner_receipt_succession.rs:40-206`;
  - `intent/admission/state.rs:54`;
  - `application_rebind.rs:45`;
  - `application_fact_state/validation.rs:136`;
  - `interaction/state/application_rebind.rs:88`;
  - `pointer/cancellation.rs:78`;
  - `expression/owner/prepared_succession.rs:81,88`;
  - `scroll/state/route_candidate.rs:55`;
  - `selection/staged_transition.rs:103-130`;
  - `overlay_binding_lifecycle/generation.rs:40`;
  - `focus/rebind.rs:141-145`, where the `structural_revision` `checked_add` done in prepare is not carried in the prepared value, so commit recomputes it;
  - `rebind_execution.rs:92-111`;
  - `focus_reveal.rs:163-168`;
  - `scheduler.rs:37-39`;
  - `receipt_ledger_entry.rs:25`.
- **Generation currentness:**
  - admission `currentness.rs:170-235`;
  - `payload_inputs_are_current` (`input_basis.rs:113`, `prepared.rs:91`);
  - `is_current_result`;
  - `confirmation/validation.rs:30-45, 90-130`.
  P9's `UiIntentGenerationReads<'g>` is offered as the carrier for their "observed at generation G" type. The law 5 fix (bool → Result) is compatible with either design.
- **freeze.rs duplication:** owned by the succession agent.
- **D (cutover typestate):** overlaps their cutover writers. Hand it over.
- **P2:** the three mount-eligibility writers are their establishment and cutover writers. This audit proposes only the graph-side `mount_transition` mint that those writers would call.

## 5. Law 17 (caches) check

- `mounting/projection/semantic_text/qualification_cache.rs` has an explicit equivalence contract: a reflow key digest plus `admits_width`. Only `qualify` returning `(Arc<..>, bool)` is a minor law 5 issue; name the bool as `UiQualificationReuse::{Reused, Fresh}`.
- `runtime/overlay_composition/relation_cache.rs:7-34` is a `BTreeMap<surface, compiled graph>` built from the declaration set. It is held as `relations: Option<UiOverlayRelationCache>` in `UiOverlayCompositionState` (`planner.rs:247-254`), next to `declarations` and `declaration_revision`.
  - The equivalence contract is implicit: the same owner holds the declarations. The key does not include `declaration_revision`.
  - This is acceptable only while that state rebuilds `relations` on every declaration change. That rebuild was not traced.
  - Low risk. An improvement would be to key or tag the cache with `declaration_revision`, so a stale cache is unrepresentable.
- No other memo or cache structure with cross-input reuse was found in the scan. The Option-field scan was not exhaustively reviewed for caches (see Limitations).

## 6. Fine local invariants (not phase findings)

These guards check a fact established in the same function or module, or a numeric or capacity bound set by an admission that already ran. They are fine as written. There are **15 categories**, covering roughly 325 numeric guards plus about 540 of the 1094 non-numeric guards. The split is approximate: the guard pass classified by reading, not by counting every line.

1. Numeric and capacity fits (`try_from`, `checked_add/sub/mul` after an admitted reservation). About 325. For example, `inspection/visual_snapshot/registry/resource_lifecycle.rs:34-42` and `host_exchange/observation_report_validation/retention_admission.rs:142`.
2. The persistent_index AVL structural expects (the tree's own balance and length invariants).
3. The const-fn identity validators (nonzero identity construction). For example, `facade/entry/active_application_session.rs:231`.
4. Bounded queue pops after a length check in the same function.
5. Byte accounting in retained-structure footprints. For example, `mounting/projection/frame_storage/appearance_order.rs:61` and `appearance_order/partition.rs:36-40`.
6. Mailbox slot occupancy.
7. `topological_order.rs:122`, `resolution.rs:54` and `interval_index.rs:225`: graph and index structure invariants.
8. The shutdown persistence `debug_assert`s.
9. Same-function index lookups (insert, then get). For example, `mounting/projection/frame_storage/semantic_projection.rs:227-253` and `mounting/occurrence_geometry/state/scroll.rs:332`.
10. source/lower support-catalog expects. These look up modules the same pass just registered:
    - `binding_semantics_context.rs:90-295`;
    - `snapshot_resolution_context.rs:59-179`;
    - `structural_legality_context.rs:103-311`;
    - `artifact_*` module lookups.
11. `filesystem_source_reader.rs:139`.
12. `session/service_proposal/coordination.rs:63,127` and the occupancy/cancellation count fits.
13. Completion Drop-guard handles (cluster B, reclassified): `intent_consequence_publication.rs:198-210` and `portal_dismissal/handles.rs:8-18`. A Box-in-Option is taken exactly once, in Drop or in complete. This is a correct local pattern; the duplication is debt (section 7).
14. Test-only functions compiled in production files, which the scanner could not split. For example, `motion/state.rs:315,325`, `appearance/theme/capability.rs:284-317`, and the `mint_unbound().unwrap()` in `mounting/presentation/truth_geometry/{displayed,scroll_offset}.rs:79-87`.
15. DSL guards (all 11 are local):
    - `sealed_semantic_package.rs:154,173`;
    - `semantic_package_lowering_receipts.rs:106-151`;
    - `component_attachment.rs:18`;
    - `overlay_graph.rs:20,25`;
    - `admission_order.rs:77`.
    The DSL variant-extraction `filter_map(_ => None)` wildcards are also fine:
    - `sealed_semantic_accessors.rs:33,56`;
    - `sealed_semantic_appearance.rs:74,87`;
    - `sealed_semantic_layout.rs:39`;
    - `expression_admission/mod.rs:154`;
    - `admission_order.rs:34`.

**Wildcard and `..Default` split.**
- Of the 153 runtime `_ =>` wildcards, the owned-enum wildcards that matter are the ones cited in P1, P4, P10, P11 and the DSL items. Most of the rest are variant extraction (`_ => None` in a projection accessor), which is acceptable.
- The `..Default` sites are fine: counters and work reports. They include:
  - `hit_test_work.rs:95`;
  - `presented_hit_index.rs:85`;
  - `hit_mechanics.rs:41,160`;
  - `interaction_basis.rs:68`;
  - `transaction_outcome.rs:88`;
  - `plan_equivalence/counters.rs:23`;
  - `overlay_registry.rs:224-233`;
  - `coordinator.rs:130`;
  - `command_motion_layers.rs:237`.
  The exception is `UiMountedLaneAssembly` (P3).

## 7. Plain debt (not phase typing)

1. A shared completion Drop-guard. `intent_consequence_publication.rs:198-210` and `portal_dismissal/handles.rs:8-18` (including `admitted.proposal.expect`) re-implement the same Box-in-Option completion guard. Extract a `UiLiveCompletion<T>`.
2. The "must preserve a root member" guard is repeated about 7 times:
   - `constraint_bound_reconciliation.rs:112`;
   - `constraint_child_intrinsic_contribution.rs:79`;
   - `constraint_durable_resize_input.rs:36`;
   - `constraint_equal_share_distribution.rs:118`;
   - `constraint_parent_available_space.rs:85`;
   - `collect_authority.rs:19`;
   - `measurement_basis.rs:44`.
   Carry a `UiNonEmptyMembers` (or `(root, rest)`) type through the constraint pipeline.
3. The `qualification_cache` returns `(Arc, bool)`; name the bool.
4. Orphan files, not in the module tree from `lib.rs`. These are probably dead, so verify and delete:
   - `graph/allocation_neighborhood/replan_selection/transaction/invalidation_sources.rs`;
   - `host/tests/measurement_fixture.rs`;
   - `host/tests/measurement_result_test_support.rs`;
   - `runtime/overlay_composition/tests/support.rs` and `runtime/overlay_composition/tests/surface_scrim.rs`;
   - `runtime/tests/replacement/candidate_admission_artifact_nodes.rs`.
   The modtree walker follows `#[path]`, but some of these may be included through a macro. Confirm with a grep before deleting.
5. `semantic_package_lowering_receipts.rs:120` `structural_spec(add_component_reference: bool)`: a bool parameter selects the variant.
6. `resolved_aspect.rs` `new` has 9 args under `allow(too_many_arguments)` (removed by P4).
7. The facade-implements list in section 3.
8. `evidence_ref_expansion.rs:146,148` bare `unreachable!()` with no message (P11).

## 8. Suggested landing order

1. **Small and independent, can start now:** P7, P10, P11, P14. Each is 2-5 files, low risk, and compiler-driven.
2. **3b:** P9, then P8. Coordinate with the `worth-ui-3.17-3b-wip` branch, and hand `UiIntentGenerationReads` to the succession agent.
3. **Before 3d:** P2 (graph-side mint plus the Unmount lane; this fixes the Layout=Admitted latent bug), then P3 together with C.
4. **Before 3e:** P1.
5. **Before 3f:** P4, then P5, then P6.
6. **Incremental, as owners are touched:** P12 (Portal group first), P13, I.

## 9. Limitations

- Nothing was compiled, so the target sketches are unverified signatures.
- The bool-field (460) and Option-field (2236) scans were reviewed only where they intersected guards or the clusters above, plus the full session struct (`active_application_session.rs:164-221`). A field-by-field pass over the Option fields was not done.
- The `relation_cache` rebuild-on-declaration-change was not traced.
- The orphan files were detected by a module-tree walk, not confirmed against macros or build scripts.
- The guard split between local invariants and phase guards is by reading. The totals in section 6 are approximate.
