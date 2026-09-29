# Generation succession: survey and target design

Basis: `worth-ui-3.17` at HEAD `2056f4bd4b` ("Phase 3a: operability from conditions").
The worktree is clean except `plans/worth-ui/milestone-3.17.md`. Phase 3b now lives on
`worth-ui-3.17-3b-wip` and is not surveyed.

- Paths are relative to `workspaces/worth-ui/crates/worth-ui-runtime/src/` unless they
  are rooted at the repo.
- Every `file:line` below was read at HEAD. I re-read the files that were dirty during
  the survey after the revert, and their lines matched.
- **UNVERIFIED** marks anything I did not read.
- Nothing was compiled and cargo was not run.

Terms:

- **Owner**: a session field whose value is scoped to the active application
  generation.
- **Writer**: a code path that moves the active generation from G to G+1.
- **Intra-generation writer**: a code path that changes an owner without changing the
  generation. Examples are an application fact update, a frame publication and a
  consequence observation.

---

## 1. Inventory

### 1.1 Generation-scoped owners

These are fields on `WorthUiActiveApplicationSession` (`facade/entry/active_application_session.rs:164-220`).

| Owner | Field (line) | Prepared type |
|---|---|---|
| Application authority | `application` (166) | `WorthUiPreparedApplicationAuthority` |
| Expression owner | `expressions` (193) | `UiPreparedExpressionSuccession` (`runtime/expression/owner/prepared_succession.rs:15`) |
| Pointer affordance snapshot | `pointer_affordance_snapshot` (203) | `UiPreparedPointerAffordanceGenerationSuccession` (`runtime/pointer_affordance/generation_succession.rs:15-28`) |
| Appearance themes and inspection | `appearance_theme_admission` (199), `appearance_inspection` (201) | `UiPreparedAppearanceGenerationSuccession{theme, inspection}` (`facade/entry/appearance_generation_succession.rs:9-12`) |
| Retained appearance owners | `appearance_owner_snapshot` (205) | `UiPreparedRetainedAppearanceOwnerSuccession` (`runtime/appearance/state/owner_snapshot/retention.rs:4`) |
| Authored overlay bindings | `authored_overlay_bindings` (175) | `UiPortalOverlayBindingLifecycle`, the whole value |
| Occurrence geometry | inside `mounted` (168) | `UiMountedOccurrenceGeometryState`, the whole value |
| Standing operability facts | inside `intent_admission` (195) | `UiPreparedIntentAdmissionRebind` (`runtime/intent/admission/state/application_rebind.rs`), cutover only |
| Application facts | `intent_application_facts` (192) | `UiIntentApplicationFactState`, replaced whole at cutover |
| Frame-scoped receipt successions | `mounted_owner_receipt_successions` (207) | `UiPreparedMountedOwnerReceiptSuccession` (`facade/entry/mounted_owner_receipt_succession.rs:5-8`) |
| Cutover-only clears | `interaction`, `focus`, `selection`, `scroll`, `command_routing`, `intent_confirmation`, `motion`, `portal` | various |

Standing operability facts carry no generation field (`runtime/intent/operability/standing_fact.rs:12`
onward). That is why an evidence-only succession can keep them.

### 1.2 Generation writers, with entry and commit points

**W1. Evidence-only rebind.**

- Entry is `prepare_rebind`, `prepare_native_rebind` or `prepare_rebind_with_reconciliation`
  (`facade/entry/rebind_execution.rs:12,20,31`), which lead to `prepare_rebind_with_inputs` (40-157).
- When a retained successor authority exists, expressions are prepared first and pointer
  second (`prepare_retained_successions`, 180-198; expressions at 194-195, pointer at 196).
- `WorthUiPreparedEvidenceOnlyApplicationRebind::new` (`rebind_execution/evidence_only.rs:19-99`):
  - denies an in-flight presentation (36-40);
  - prepares appearance (51-64), overlay bindings (65-74), occurrence geometry (75-84) and
    retained owners (85-86).
- Commit: `evidence_only.rs:101-156`.
  - Runtime re-checks run first (110-134): `validate_predecessor`, `matches_projection`, and
    a re-run of `prepare_retained_appearance_owners`.
  - Then it calls `session.commit_evidence_only_successor(..)` (147-155).
- Commit body: `active_application_session/evidence_only_successor_commit.rs:26-32`. It
  carries `#[allow(clippy::too_many_arguments)]` at lines 8-11.

**W2. Authored mounted-content rebind.** It publishes a frame between prepare and commit.

- Entry is at `rebind_execution.rs:92` (`AuthoredContent`) and `104-128` (evidence-only
  upgraded because a pointer publication is required).
- Both call `prepare_authored_content_rebind` (200-265). That function repeats W1's
  overlay, geometry, appearance and owners preparation (209-248), then calls
  `WorthUiPreparedMountedContentRebind::prepare_authored` (249-259).
- The same seven owners travel as `WorthUiMountedContentPublication::AuthoredSuccessor`
  (`facade/entry/mounted_content_rebind.rs:35-52`).
- Attached path:
  - `present` checks `prepared_owners_are_current` (162-168).
  - `complete` and `cancel` call `finish` (219-240).
  - `finish` commits through `commit_evidence_only_successor` (283-299).
- Detached path:
  - `rebase` re-prepares geometry, expressions, pointer and owners
    (`mounted_content_rebind/detached.rs:50-59`). It does **not** re-prepare appearance
    or overlay bindings.
  - `WorthUiDetachedMountedContentRebindInFlight::complete` (`detached.rs:114-121`)
    calls `finish` with **no currentness check**, even though the session borrow was
    released while detached.
- `refresh_before_effects` (`mounted_content_rebind/preparation.rs:162-203`) re-prepares
  and rebuilds only when `prepared_owners_are_current` (125-159) fails.

**W3. Application cutover.**

- The mounted path is prepared through `prepare_application_cutover`
  (`application_replacement/cutover.rs:119-191`, in-flight denial at 147-149). The
  unmounted path is `activate_prepared_replacement` (47-117).
- Mounted owners are prepared in `application_replacement/mounted/appearance_projection.rs:41`
  and committed via `mounted/published.rs:48-53`.
- Overlay bindings are prepared in `application_replacement/portal_lifecycle.rs:52-58` and
  committed at `:87`, with `.expect` calls at 83 and 85.
- Geometry is prepared in `application_replacement/mounted/geometry.rs:8`. Its commit
  point is UNVERIFIED; it is probably inside `mounted.commit_graph_replacement_successor`
  (`cutover/application_commit.rs:100-101`).
- `commit_application_activation` (`cutover/application_commit.rs:9-127`) runs in this
  order:
  1. `.expect`/`unreachable!` on the transition (17-26).
  2. Application (27).
  3. Appearance succession (28-30).
  4. Command routing (32-47).
  5. Lifecycle, including overlay bindings at 57.
  6. Focus, portal and motion (48-63).
  7. `intent_confirmation.cancel_all` (64-66).
  8. Scroll (67-74).
  9. The owner cutover `match` (75-99):
     - `Mounted` calls `owners.commit(self)` (`owner_succession.rs:163-177`).
     - `Unmounted` replaces facts (86-90), cancels admission (91-94), then runs
       `reconcile_successor_owners` (96) and cancels interactions (97).
  10. Graph successor (100-101).
  11. **Expressions are prepared and committed in the same step, after all effects**
      (102-106).
  12. Evidence is retired (107-113).

**W4. Native mounted establishment.** This is the initial generation on a native
allocation.

- `establish_native_viewport_allocation` (`facade/entry/mounted_allocation_establishment.rs:84-109`)
  wraps `establish_mounted_allocation_catalog` (111-213).
- Preparation:
  - denies an in-flight presentation (119-121);
  - graph (127-135), overlay graph succession (136-141), appearance (142-153);
  - `capture_retained_appearance_owners` (154-159);
  - expressions (178-185);
  - pointer through `prepare_pointer_graph_succession` (186-190), then
    `validate_predecessor` (191-202).
- Commit (203-211): application, appearance, pointer, overlay, then expressions last (211).
- There is **no occurrence-geometry step**.

### 1.3 Writer × owner matrix

Legend:

- **C**: compiler-enforced. The value must be produced for the call to type-check.
- **V**: conventional. Nothing fails to compile if it is omitted or reordered.

| Owner | W1 evidence-only | W2 authored | W3 cutover | W4 establishment |
|---|---|---|---|---|
| Application authority | prepare+commit, **C** (argument at `evidence_only_successor_commit.rs:14`) | same as W1, **C** | commit, **C** (`application_commit.rs:27`) | commit, **C** (203-206) |
| Expressions | prepare+commit, **C** (argument at :18). Commits last, **V** (:32) | same as W1, **C**/**V** | prepare+commit after effects, **V** (102-106). Omission compiles | prepare+commit, **V** (211 is a free call) |
| Pointer snapshot | prepare+commit, **C** (argument at :19) | same as W1, **C** | clear, **V** (`owner_succession.rs:176`; `cutover/appearance.rs:121`) | prepare+commit, **V** (208 is a free assignment) |
| Appearance themes | prepare+commit, **C** (argument at :15) | same as W1, **C** | commit, **V** (`application_commit.rs:28-30`) | prepare+commit, **V** (207) |
| Retained appearance owners | prepare+commit, **C** (argument at :16) | same as W1, **C** | snapshot replaced without queueing (Mounted, `owner_succession.rs:175`) or reconciled (Unmounted, `cutover/appearance.rs:118-144`), **V** | capture+commit, **V** (154-159, 207) |
| Overlay bindings | prepare+commit, **C** (argument at :17) | same as W1, **C**. Not rebased when detached | prepare+commit, **C** inside the lifecycle bundle (`portal_lifecycle.rs:62-73,87`) | graph succession, **V** (209-210) |
| Occurrence geometry | prepare+commit, **C** (argument at :21) | same as W1, **C** | prepare (`mounted/geometry.rs:8`), commit UNVERIFIED | **none**. Omission compiles, **V** |
| Standing facts | kept, **V** (nothing names the decision) | kept, **V** | cleared, **C** (`owner_succession.rs:170-172`) or `cancel_all`, **V** (`application_commit.rs:91-94`) | nothing, **V** |
| Application facts | kept, **V** | kept, **V** | replaced, **C** (Mounted, :173) or **V** (Unmounted, `application_commit.rs:86-90`) | nothing, **V** |
| Receipt successions | refreshed through `commit_retained_appearance_succession` → `refresh_appearance_owner_receipt_sources` (`retained_appearance_succession.rs:13`), **V** | same as W1, **V** | UNVERIFIED | same as W1, **V** (207) |
| Condition re-observation | through `follow_application_generation` (`expression_access.rs:72`), **V** | same as W1, **V** | same, **V** | same, **V** |

`commit_evidence_only_successor` is the one partly compiler-total site. It takes every
W1/W2 owner as an argument. It still does not stop a caller from committing the owners
of a different preparation, because the arguments are unrelated values. W3 and W4 are
fully conventional.

### 1.4 Places that order owners, and why

1. **Expressions are prepared before pointer.** Pointer reads the successor expression
   owner through `successor_read_owners`
   (`facade/entry/intent_operability_observation.rs:28-37`, substitution at :35).
   Sites: `rebind_execution.rs:194-196` and `mounted_allocation_establishment.rs:178-190`.
2. **Expressions commit last.** The committed owner reads the successor application and
   mounted state. See the doc at `evidence_only_successor_commit.rs:4-7` and at
   `expression_access.rs:52-57`. Sites: `evidence_only_successor_commit.rs:32`,
   `mounted_allocation_establishment.rs:211` and `application_commit.rs:106`.
3. **The appearance theme commit nulls the owner snapshot before the retained owners set
   it again.** Sites: `appearance_generation_succession.rs:62` then
   `retained_appearance_succession.rs:12`.
4. **`queue_closed_owner_invalidation` runs before an appearance owner snapshot is
   replaced.** It is a pull diff against the old snapshot (`facade/entry/observation.rs:35`
   onward). The ordered sites are `observation.rs:29-31`,
   `mounted_owner_receipt_succession.rs:105-106` and
   `mounted_content_rebind/preparation.rs:44`. The unordered sites, which replace the
   snapshot without queueing, are `owner_succession.rs:175`,
   `intent_consequence_observation.rs:85` and `mounted_owner_receipt_succession.rs:44`.
   Whether each unqueued site is covered by an earlier queue is UNVERIFIED except for
   `intent_consequence_observation.rs:85`, which is covered by `preparation.rs:44`.
5. **In frame finishing, the frame settles before receipt successions, and receipt
   successions settle before condition re-observation.** See
   `mounted_publication.rs:93-126`, with the reason at 88-92. The frame settlement calls
   `expressions.invalidate_published_frame` (249-267). The same sequence is repeated at
   `mounted_frame_execution.rs:242-275`, `mounted_application_presentation.rs:122-154`
   and `185-215`, and `mounted_preview/presentation.rs:62,136`.
6. **The expression commit catches up after the swap.** It applies fact updates and a
   frame change that happened since the prepare basis
   (`prepared_succession.rs:96-101`; the doc at 63-67 names the detached-publication case).
7. **Presentation in flight is denied before anything else.** Sites:
   `evidence_only.rs:36-40`, `cutover.rs:147-149`, `mounted_allocation_establishment.rs:119-121`
   and `active_application_session.rs:306-308`.

### 1.5 Conventional hazards, which are the targets of this design

- `.expect("... requires prepared successions")` at `rebind_execution.rs:92,103`.
  `unreachable!` at `rebind_execution.rs:111` and `mounted_content_rebind.rs:332,358`.
- Runtime asserts that turn a stale or mismatched commit into a panic:
  - `prepared_succession.rs:81-84` and `88-91`;
  - `mounted_owner_receipt_succession.rs:40-41` and its `.expect` at 53;
  - `application_rebind.rs:45-48`;
  - `portal_lifecycle.rs:83,85`.
- **Detached completion has no currentness proof.** See `detached.rs:114-121`. The
  expression owner catches itself up (`prepared_succession.rs:96-101`). The pointer
  snapshot and retained owners committed next to it were derived from the
  *pre-catch-up* successor expressions. The claim in the `prepared_owners_are_current`
  doc (`preparation.rs:125-159`) that the expression succession "changes only through a
  commit on the session this preparation holds" does not hold once the borrow is
  released. Whether a later observation repairs the pointer snapshot is UNVERIFIED.
- Detached `rebase` skips the appearance and overlay-binding successions
  (`detached.rs:32-86`). Whether either can change while detached is UNVERIFIED.
- `pointer_affordance_snapshot` has six direct writers, all conventional:
  - `evidence_only_successor_commit.rs:29`
  - `cutover/appearance.rs:121`
  - `owner_succession.rs:176`
  - `intent_consequence_observation.rs:86`
  - `mounted_allocation_establishment.rs:208`
  - `observation.rs:31`

---

## 2. Invalidation wiring inventory

### 2.1 Edges

| Change | Consumer | Mechanism | Declared or call-site |
|---|---|---|---|
| Application fact update (`intent_routing.rs:35-73`) | expressions | `invalidate_application` (`runtime/expression/owner/invalidation.rs:12-30`), via the `readers_of_application` index | Index is declared (built from the catalog). Call is at call-site `intent_routing.rs:89-96` |
| Published frame, projection inputs | expressions | `invalidate_published_frame` (`invalidation.rs:35-57`) probes only the projection slots expressions read | Index is declared. Call is at call-site `mounted_publication.rs:249-267` |
| Expression settlement, condition slots | standing facts | `UiIntentAdmissionState::reobserve_condition_consumers` (`runtime/intent/admission/state.rs:91-115`) through `UiIntentConditionConsumers` (`declaration/intent/catalog/condition_consumers.rs:11-44`) | Index is declared. Calls are at 6 call-sites: `expression_access.rs:72`, `intent_routing.rs:97`, `mounted_publication.rs:124`, `mounted_frame_execution.rs:266`, `mounted_application_presentation.rs:145,208`, `mounted_preview/presentation.rs:62,136`. Three wrappers exist: `expression_access.rs:77`, `active_framework_turn.rs:291`, `mounted_preview.rs:81` |
| Generation commit | expressions, then standing facts | `follow_application_generation` (`expression_access.rs:58-73`) | Call-site in each writer |
| Appearance owner snapshot replaced | appearance invalidation batches | `queue_closed_owner_invalidation`, a pull diff (`observation.rs:35` onward) | Call-site, and inconsistent (§1.4 item 4) |
| Frame published | receipt successions | `settle_pending_*` / `settle_new_*` (`mounted_owner_receipt_succession.rs:125-197`) | Call-site in each frame-finish copy |
| Application fact update | appearance | none direct. Whether a later snapshot diff picks it up is UNVERIFIED | none |

### 2.2 Push vs pull, by operability source kind

The source kinds are in `declaration/intent/operability_contract.rs:31-56` (authored) and
`:67-92` (resolved). `conditions()` (269-289) extracts only the `Condition` sources. The
module doc says so outright (`condition_consumers.rs:1-2`).

| Source kind | Axes | What observation reads (`runtime/intent/operability/basis/axis_observation.rs`) | Push today | Plan |
|---|---|---|---|---|
| `Condition` | mutability, readiness, policy | expression result (155-171) | **push**, through expression settlement | keep |
| `ProjectionReadonly` | mutability | retains the projection input, and the posture is always `Readonly` (46-48) | **pull only** | the plan asks for push (`milestone-3.17.md:218-220`) |
| `Projection` | readiness | projection input current (77-79, 190-204) | **pull only** | the plan asks for push |
| `CommittedDraft` | mutability, readiness | **nothing**: a constant `Writable` / `Ready` (50, 80) | nothing to push | the plan asks for push (218-220). Decision D4 |
| `ApplicationBoolean` | mutability, readiness, policy | application fact (37-44, 72-76, 103-108) | pull only | **retired** (`milestone-3.17.md:216-217`; law 8, "Booleans are conditions"). No hook is designed for it |

Payload sources are pull-only by design (`milestone-3.17.md` 3b: "Payload stays
pull-only"). Standing facts are created only by the host-driven
`evaluate_intent_operability` (`facade/entry/intent_payload.rs:31-64`, creation at 50-59).
Re-observation refreshes existing facts and "never creates a fact"
(`runtime/intent/admission/standing_owner/reobservation.rs:6-9`). **The design below
keeps that invariant: no framework path creates a standing fact.**

### 2.3 Currentness is re-checked piecemeal (coordinator finding 1, verified)

The same four owners travel as four different bundles:

- `UiIntentAdmissionCurrentnessContext` (`runtime/intent/admission/currentness.rs:11-19`):
  `generation`, `mounted`, `application_facts`, `expressions`, plus catalog, definitions
  and command contexts.
- `UiIntentConfirmationReadContext` (`runtime/intent/confirmation/validation.rs:8-16`):
  the same four, plus `occupancy`.
- `UiIntentOperabilityDependencyReads` (`runtime/intent/operability/basis.rs:194-199`):
  the same four.
- `UiIntentInputOwners` (`runtime/intent/payload/input_basis/view.rs:9-15`): "all observed
  at one generation", but it holds **three** owners. The generation travels as a separate
  `&` next to it (`view.rs:2,20,32`, and `UiIntentOperabilityReadOwners` at
  `expression_access.rs:85-97`).

The checks are repeated:

- Admission check 2/3 (world, generation) is at `currentness.rs:176-187`.
- Payload inputs are check 8 (`currentness.rs:214-223` → `payload/prepared.rs:91-99` →
  `payload/input_basis.rs:113-128`). Each application input calls
  `is_current_reference(expected, generation)`.
- Operability dependencies are check 9 (`currentness.rs:224-238` → `prepared.rs:101-106`),
  which reads expressions through `is_current_result(reference, active)`
  (`runtime/expression/owner/state.rs:97-108`, itself re-checking `follows(active)`).
- Confirmation repeats the generation checks for both the candidate and the source
  (`validation.rs:32-44`), payload inputs (104-110) and operability dependencies (111-120).

The generation is checked up to four times per admission, and it is never carried by the
owner bundle.

### 2.4 Successor preparation split (coordinator finding 3, verified)

- Classification prepares the full successor authority
  (`runtime/session/application_state/change_classification.rs:109` →
  `prepare_authored_source`, 122-128 →
  `facade/lifecycle/freeze.rs:125 prepare_successor_application_authority`).
- `prepare_application_authority` (`freeze.rs:33-123`, 91 lines) and the successor
  variant (`freeze.rs:125-242`, 118 lines) share:
  - graph lowering, expansion and admission (62-70 / 174-184);
  - expression catalog (71-81 / 185-195);
  - intent catalog, with an identical `UiIntentOperabilitySourcePlans` literal
    (86-90 / 200-204);
  - lifecycle bootstrap (94-98 / 211-215);
  - `seal` (99-122 / 216-240).
- They differ in:
  - initial vs successor graph commit (68 / 180-182);
  - where the plans come from (the input vs `current`, 231-234);
  - service-policy normalization (157-172);
  - lineage (104 / 220-222);
  - the capability snapshot fallback (59-61 / 154-156).
- Plan law 6 (all-or-nothing source) is structurally held by two things. `current` is
  only `&` (`freeze.rs:126`, `change_classification.rs:123`), and nothing is installed
  before `seal`. Neither is a phase type.
- Cost ordering (arch law 16) is weaker. Candidate admission (`change_classification.rs:137-150`)
  reads only `runtime.replacement_admission_basis()` and the candidate (142-145). The
  candidate is available at `freeze.rs:145-153`, before graph and catalog work. So
  admission could precede the expensive construction. UNVERIFIED that admission needs
  nothing else.

### 2.5 Semantic no-op (coordinator finding 4, verified)

`permits_execution_plan_semantic_no_op` (`runtime/activation/application_publication.rs:70-86`)
requires all four of these together:

- intent contract `Equivalent`;
- appearance consumer contract unchanged;
- projection contract unchanged;
- expression contract unchanged.

The expression contract is separate because the intent contract compares conditions by
identity (9-11). They are four loose fields on an enum variant (4-12). Adding a
comparison compiles without joining the predicate.

---

## 3. Target design

### 3.0 Placement (AGENTS.md:44-51)

- The per-owner prepared types stay in their owning runtime modules, where they already
  live: `runtime/expression`, `runtime/pointer_affordance`, `runtime/appearance`,
  `runtime/intent/admission`. They hold counters and live tables.
- The succession pipeline composes `&mut` borrows of session fields and holds no clock,
  counter or table of its own. It lives beside the session in
  `facade/entry/generation_succession/`, which is the only module that can see the
  fields.
- The one new counter, a succession epoch (§3.3), needs a counter to exist. It is a
  session field and is not a substrate type.
- Nothing here decides platform legality, so nothing goes to `worth-proof`.

### 3.1 One typed read of owners at a generation (findings 1 and 2)

```rust
// runtime/intent/payload/input_basis/view.rs (replaces UiIntentInputOwners)
/// The owners an intent reads, observed at one generation. Mintable only by
/// the session (active) or a prepared succession (successor).
#[derive(Clone, Copy)]
pub(crate) struct UiGenerationReadOwners<'s> {
    generation: &'s WorthUiActiveApplicationGenerationIdentity,
    mounted: &'s WorthUiMountedSessionState,
    application_facts: &'s UiIntentApplicationFactState,
    expressions: &'s UiExpressionRuntimeState,
}
```

- `UiIntentOperabilityDependencyReads`, `UiIntentInputOwners` and the four duplicated
  fields in both contexts all collapse into this type.
- `UiIntentOperabilityReadOwners` becomes `{ authority, reads: UiGenerationReadOwners }`.
- The constructors are `pub(in crate::facade::entry)`:
  - `session.active_reads()` replaces `intent_read_owners`,
    `intent_operability_observation.rs:4-23`;
  - `prepared.successor_reads(&session)` replaces `successor_read_owners`, 28-37.

Currentness is proven once and consumed:

```rust
// runtime/intent/payload/input_basis/currentness.rs (new)
pub(crate) struct UiCurrentInputBasis<'b> { basis: &'b UiIntentInputBasis, checks: u8 }
pub(crate) enum UiInputBasisDrift { World, Generation, PayloadInput, Operability(UiIntentOperabilityDependencyDrift) }

impl UiIntentInputBasis {
    /// World, generation, payload inputs, then operability dependencies,
    /// in the order admission numbers them (2, 3, 8, 9).
    pub(crate) fn prove_current<'b>(&'b self, reads: UiGenerationReadOwners<'_>)
        -> Result<UiCurrentInputBasis<'b>, UiInputBasisDrift>;
}
```

- Admission (`currentness.rs:170-239`) keeps checks 4-7 between the generation and payload
  checks. It keeps its check numbering because `prove_current` is split in two:
  `prove_generation(reads) -> UiGenerationCurrent` first, then
  `UiGenerationCurrent::prove_inputs(self, reads) -> UiCurrentInputBasis`. The second step
  cannot be called without the first (law 16).
- `is_current_result` stops re-checking `follows(active)` once per reference when it is
  called with a `UiGenerationCurrent` proof. This removes work and does not add any.
- Confirmation (`validation.rs:32-44,104-120`) calls the same two steps for the
  candidate. For the source basis it calls `prove_generation` only.

### 3.2 The succession pipeline (laws 9, 12, 16)

```rust
// facade/entry/generation_succession/plan.rs
/// What a writer does with each owner. Adding an owner adds an associated
/// type here, which breaks every plan until it states a disposition.
pub(in crate::facade::entry) trait UiSuccessionPlan: sealed::Sealed {
    type Authority;          // what `application` commits
    type RetainedAppearance; // Prepared | Captured | ReplacedSnapshot
    type OverlayBindings;    // PreparedLifecycle | GraphSuccession
    type OccurrenceGeometry; // Prepared | NotAllocated
    type Pointer;            // Prepared(UiPreparedPointer..) | Cleared
    type Standing;           // Retained | Cleared(UiPreparedIntentAdmissionRebind)
    type ApplicationFacts;   // Retained | Replaced(UiIntentApplicationFactState)
    type Expressions;        // Prepared(UiPreparedExpressionSuccession) | AtCommit
    type Clears;             // () | UiCutoverClears (interaction, focus, selection, scroll, routing, confirmation)
}
pub(in crate::facade::entry) enum Retain {}     // W1, W2
pub(in crate::facade::entry) enum Establish {}  // W4
pub(in crate::facade::entry) enum Replace {}    // W3 (Mounted | Unmounted inside Replace::Clears)
```

- Each disposition is a named zero-sized or owning type, so "keep" is stated and never
  implied by absence.
  - `Retain::Standing = UiStandingRetained` is minted only by a function whose doc cites
    `standing_fact.rs` having no generation field.
  - `Establish::OccurrenceGeometry = UiGeometryNotAllocated` states W4's gap.
- `Replace::Expressions = UiExpressionsAtCommit` states that cutover prepares expressions
  against the facts it commits (`application_commit.rs:102-106`). That is legal because
  cutover clears the pointer, so nothing reads the successor expressions before commit.

```rust
// facade/entry/generation_succession/prepared.rs
#[must_use = "a prepared succession changes nothing until committed"]
pub(in crate::facade::entry) struct UiPreparedGenerationSuccession<P: UiSuccessionPlan> {
    basis: UiSuccessionBasis,        // predecessor identity + session epoch (§3.3)
    authority: P::Authority,
    appearance: UiPreparedAppearanceGenerationSuccession,
    retained_appearance: P::RetainedAppearance,
    overlay_bindings: P::OverlayBindings,
    occurrence_geometry: P::OccurrenceGeometry,
    pointer: P::Pointer,
    standing: P::Standing,
    application_facts: P::ApplicationFacts,
    expressions: P::Expressions,
    clears: P::Clears,
}
```

Phases are consuming methods. Each takes the sealed proof of the phase before it.

```rust
// eligibility: the only mint of the idle witness
impl WorthUiMountedSessionState { fn idle_witness(&self) -> Result<UiNoPresentationInFlight, Denial>; }

// prepare (per plan; one fn per plan, sharing owner-level helpers)
fn prepare_retain(session: &mut Session, idle: UiNoPresentationInFlight, authority: ..)
    -> Result<UiPreparedGenerationSuccession<Retain>, Denial>;

// commit: the single place owner order is written, as an exhaustive destructure
impl<P: UiSuccessionPlan> UiPreparedGenerationSuccession<P> where P: UiCommitOrder {
    fn commit(self, owners: UiSuccessionOwners<'_>) -> UiGenerationSettlement;
}
```

- `commit` destructures with no `..`. The fixed order is: application, appearance, then
  retained appearance (with its queue, §3.4), overlay, geometry, pointer, standing,
  facts, clears, and expressions last. It returns a `UiGenerationSettlement` that the
  change sink (§3.4) consumes.
- `UiSuccessionOwners<'_>` is a struct of `&mut` to exactly the session fields
  succession touches. It is built by one `session.succession_owners()` so that other
  fields stay borrowable (law 1).
- `commit_evidence_only_successor`, its `too_many_arguments` allow, and the free
  `follow_application_generation` calls at the three writer sites are deleted.
- The `.expect` calls at `rebind_execution.rs:92,103` go away because
  `prepare_rebind_with_inputs` branches on the semantic proof *before* it prepares, and
  only the `Retain` branches call `prepare_retain`.

Writer differences become plan branches or parameters (law 12):

| Difference | Encoding |
|---|---|
| Cutover clears and evidence-only keeps | `Replace` vs `Retain` associated types |
| Establishment has no geometry | `Establish::OccurrenceGeometry = UiGeometryNotAllocated` |
| Authored publishes a frame between prepare and commit | `UiPreparedGenerationSuccession<Retain>::into_publication(self, frame) -> UiPublishingSuccession<'s>`. Only `UiPublishingSuccession::finish(self, outcome)` yields the commit-capable value |
| Detached completion | `UiPublishingSuccession::detach(self) -> UiDetachedSuccession` (owned, no borrow). The only path back is `reattach(self, &mut Session) -> UiPreparedGenerationSuccession<Retain>` (§3.3) |

`WorthUiMountedContentPublication::AuthoredSuccessor` (`mounted_content_rebind.rs:35-52`)
then carries one `UiPreparedGenerationSuccession<Retain>` instead of seven loose fields.

### 3.3 Stale commit is unrepresentable

- **Attached** (W1, W2 attached, W3, W4): the prepared value holds, or is used within,
  the same `&mut WorthUiActiveApplicationSession` borrow. Nothing else can move an owner,
  so no runtime check is needed. The re-checks at `evidence_only.rs:110-134` and
  `mounted_content_rebind.rs:162-168` become dead. The equivalence tests in §4 prove
  that before they are deleted.
  - UNVERIFIED: whether the authored attached path really holds the session borrow from
    `present` to `finish`. If it does not, it takes the detached route.
- **Detached** (W2 only): `UiDetachedSuccession` has no `finish` or `commit`.
  - `reattach(self, &mut session)` compares `basis.epoch` with `session.succession_epoch`.
  - Every intra-generation writer that moves an owner a succession read bumps the epoch.
    These are fact updates, frame finishing and consequence observation. The bump is one
    `u64` increment on paths that already do owner work.
  - If the epochs are equal, the prepared owners are returned unchanged.
  - Otherwise `reattach` re-prepares **every** owner the `Retain` plan names, which closes
    the appearance and overlay gap in `detached.rs:32-86`.
  - Both outcomes yield the same type. There is no assert and no stale branch.
- The expression owner's basis asserts (`prepared_succession.rs:81-84,88-91`) become
  `debug_assert!`, since the pipeline cannot reach them. Its catch-up (96-101) remains:
  it is the settlement of facts and frames the reattach did not re-read, and it is how
  W3 absorbs its own effects.
- The same treatment applies to `mounted_owner_receipt_succession.rs:40-41`,
  `application_rebind.rs:45-48` and `portal_lifecycle.rs:83,85`. Each is reachable only
  through an attached commit.

### 3.4 Declared invalidation and consumption (laws 2, 6, 13)

Owners emit typed changes and never name consumers:

```rust
// runtime/invalidation/owner_change.rs (new; runtime-owned, no state)
pub(crate) struct UiOwnerChanges {
    expressions: UiExpressionSettlement,                 // changed condition slots
    projections: Box<[UiProjectionInputSlot]>,          // changed on a published frame
}
```

The framework computes consumption. The intent catalog builds one index from each
contract's declared sources, which generalizes `UiIntentConditionConsumers` (`condition_consumers.rs:16-37`):

```rust
// declaration/intent/catalog/operability_consumers.rs (replaces condition_consumers.rs)
pub(crate) enum UiOperabilitySourceKey { Condition(UiExpressionSlot), Projection(UiProjectionInputSlot) }
pub(crate) struct UiOperabilityConsumers { by_key: BTreeMap<UiOperabilitySourceKey, Box<[Box<str>]>> }
```

- `UiResolvedIntentOperabilityContract::sources()` replaces `conditions()`
  (`operability_contract.rs:269-289`). It matches exhaustively and returns
  `Option<UiOperabilitySourceKey>` per axis:
  - `Condition` → `Some(Condition)`;
  - `Projection` and `ProjectionReadonly` → `Some(Projection)`;
  - `CommittedDraft` → `None`, with a comment that it reads no owner (`axis_observation.rs:50,80`).
- A new source kind cannot compile until it declares a key or declares `None`.
- `ApplicationBoolean` stays `None` until it is retired. That is not a hook: it is the
  current pull-only fact, and the arm is deleted with the kind.

There is one sink per session, replacing the six call-sites and three wrappers in §2.1:

```rust
// facade/entry/generation_succession/change_sink.rs
fn settle_owner_changes(admission: &mut UiIntentAdmissionState,
                        authority: UiIntentOperabilityAuthority<'_>,
                        reads: UiGenerationReadOwners<'_>,
                        changes: UiOwnerChanges);
```

- The sink's signature borrows `intent_admission` mutably and the read owners shared.
  These are disjoint fields.
- `WorthUiActiveFrameworkTurnExecution` and the preview ports build the same arguments
  from the fields they already lend (`active_application_session.rs:369-370`).

The five frame-finish copies (§1.4 item 5) become one function in the pipeline module:

```rust
fn finish_frame(owners: UiFrameFinishOwners<'_>, outcome) -> UiOwnerChanges
```

- It runs the transition, overlay settle, scroll extent, pending and new receipt
  successions, and `invalidate_published_frame`. It returns the changes, and the caller
  hands them to the sink.
- Ordering lives in one body and is not repeated.

Projection push:

- `finish_frame` probes the projection slots in `UiOperabilityConsumers`' `Projection`
  keys. It uses the retained-reference diff that `invalidate_published_frame` already
  uses (`invalidation.rs:43-55`), and it unions with the expression owner's slots so
  that each slot is probed once.
- Where `set(changed projection slots) ∩ keys` is non-empty, the sink re-observes those
  consumers.
- An application that declares no projection operability source probes nothing extra.
- A new `operability_projection_probes` counter keeps the existing counters
  (`operand_probes`, `index_hits`, `operability_reobservations`) exact for current
  scenarios.

Appearance snapshot replacement becomes a single method:

- `replace_appearance_owner_snapshot(&mut self, next)`. It queues and then assigns, and
  it is the field's only writer. The field becomes private to a small
  `UiAppearanceOwnerSnapshotSlot` wrapper in `facade/entry`.
- This makes §1.4 item 4 compiler-enforced.
- The theme commit's `= None` (`appearance_generation_succession.rs:62`) becomes an
  internal step of the pipeline commit.
- `pointer_affordance_snapshot` gets the same slot treatment. Its six writers go through
  `commit`, `clear` or `observe` on the slot.

### 3.5 Authority preparation (finding 3) and semantic comparison (finding 4)

- `prepare_application_authority` and `prepare_successor_application_authority` share
  one core, parameterized by basis (law 12):
  - `UiAuthorityPreparationBasis<'c>::{Initial(UiInitialAuthorityInputs), Successor { current: &'c WorthUiPreparedApplicationAuthority }}`.
  - The basis supplies plans, graph commit mode, lineage and the capability fallback.
  - The core is `prepare_authority_core(material, basis) -> Result<WorthUiPreparedApplicationAuthority, Denial>`.
  - The source-plans literal is written once.
- Optional cost-ordering fix: split `prepare_successor_application_authority` into
  `split_candidate(submission) -> (UiSuccessorMaterial, candidate)` and
  `prepare(material, admitted: &AdmittedCandidate)`. Classification then admits before
  it builds (`change_classification.rs:109-111`). This changes denial precedence, so
  see decision D6.
- Semantic no-op: replace the four fields with
  `UiReplacementSemanticComparison { intent, appearance_consumers, projection, expressions }`.
  `is_semantic_no_op(&self)` destructures without `..`, so a fifth contract cannot be
  added without joining the predicate. The doc keeps the reason why expressions are
  compared separately (`application_publication.rs:9-11`).

---

## 4. Migration plan

Each step compiles and passes the scoped tests on its own (`cargo test -p worth-ui-runtime`,
run later by the implementer). Steps 1-4 change no behavior. Step 9 is the only step
that changes behavior, and it runs only after decision D4/D5.

1. **Characterization first.** Add counter-exact scenario tests for W1, W2 attached,
   W2 detached, W3 Mounted, W3 Unmounted and W4. Each asserts:
   - the committed owner values: generation, pointer snapshot, appearance snapshot and
     overlay revision;
   - `operability_reobservations`, `operand_probes`, `index_hits`, evaluation counts,
     and appearance invalidation batch counts.
   Extend the existing homes: `expression_generation_following_tests.rs`,
   `condition_operability_succession_tests.rs`, `pointer_affordance_succession_tests.rs`,
   `expression_mounted_succession_tests.rs` (all in `facade/entry/active_application_session/`),
   and `runtime/tests/lifecycle/lifecycle_path_parity.rs`.
   Add one **detached-drift** test: update a condition's fact while detached, complete,
   and assert the committed pointer snapshot matches a fresh observation. This test is
   expected to fail today if the §1.5 hazard is real. If it does, it is marked
   `#[ignore]` with a reason until step 6.
2. **`UiGenerationReadOwners`.** Replace `UiIntentInputOwners`,
   `UiIntentOperabilityDependencyReads` and the duplicated context fields. This is
   mechanical and has no counter change.
3. **Two-step currentness** (`prove_generation` → `prove_inputs`). Admission and
   confirmation consume it. Check numbering and violation order are unchanged. The
   existing `expression_currentness_tests.rs` must pass unchanged.
4. **Declared consumers and one sink.** Add `UiOperabilityConsumers` with Condition keys
   and Projection keys, where Projection keys are indexed but not yet probed. Add
   `UiOwnerChanges`, `settle_owner_changes` and a single `finish_frame`. Delete the three
   `reobserve_condition_consumers` wrappers and the four duplicated frame-finish bodies.
   Counters stay exact.
5. **The pipeline with `Retain`.** Port W1 and W2 attached. Delete
   `commit_evidence_only_successor`, the `too_many_arguments` allow, the `.expect` at
   `rebind_execution.rs:92,103` and the duplicated preparation in
   `prepare_authored_content_rebind` (209-248). Add the snapshot slots (§3.4). The
   attached re-checks become unreachable; delete them only once step 1's tests show no
   difference.
6. **Detached reattach and the epoch.** Delete `WorthUiDetachedMountedContentRebindInFlight::complete`'s
   direct `finish` and `rebase`'s partial re-preparation. Un-ignore the detached-drift
   test.
7. **`Establish`.** Port W4.
8. **`Replace`.** Port W3 Mounted and Unmounted. Clears move into `UiCutoverClears`.
9. **Behavior: projection push** (after D4/D5). Enable the probe of the Projection keys.
   Add a test that a projection change re-observes only its readers, with zero probes
   when none are declared.
10. **Asserts to `debug_assert!`** at the sites in §3.3.
11. **Authority preparation core, and the semantic comparison struct** (independent of
    steps 2-10; can go first).

Deletions:

- `commit_evidence_only_successor`
- the three `reobserve_condition_consumers` wrappers
- four frame-finish copies
- `condition_consumers.rs`, which is renamed and generalized
- `UiIntentInputOwners` and `UiIntentOperabilityDependencyReads`
- `WorthUiPreparedEvidenceOnlyApplicationRebind`'s runtime re-checks
- detached `rebase`
- the loose `AuthoredSuccessor` fields
- the duplicated half of `freeze.rs`

**File estimate:**

- about 10 new files: `generation_succession/{mod, plan, prepared, commit, publication, detached, change_sink, frame_finish}.rs`, `operability_consumers.rs`, and `input_basis/currentness.rs`;
- about 30-35 modified files;
- about 6 files removed or merged.

Several touched files are near the 400-line cap:

| File | Lines |
|---|---|
| `active_application_session.rs` | 379 |
| `rebind_execution.rs` | 375 |
| `mounted_content_rebind.rs` | 375 |
| `mounted_allocation_establishment.rs` | 374 |
| `mounted_publication.rs` | 344 |

The pipeline must pull code out of these files, not add to them.

**Compile-fail checks:**

- `worth-ui` is its own cargo workspace (`workspaces/worth-ui/Cargo.toml:1`). `trybuild`
  is not among `worth-ui-runtime`'s dev-dependencies. It is used by the root workspace
  (`Cargo.toml:51`, `crates/worth-proof/tests/*_compile_fail.rs`).
- `worth-ui-runtime` does use ```` ```compile_fail ```` doctests
  (`runtime/allocation_frame_dispatch/framework_turn/owner.rs:38,54`,
  `gateway/durable_resize.rs:10`, `gateway/host_measurement.rs:10,18`). Doctests see only
  the **public** API, though, and every succession type here is crate-private. A doctest
  against them would fail on visibility and prove nothing. See decision D7.

---

## 5. Risks

- **Disjoint borrows.**
  - `UiSuccessionOwners` and `UiFrameFinishOwners` must be built from distinct fields in
    one expression. This is the pattern `execute_framework_turn` already uses
    (`active_application_session.rs:299-378`).
  - Pointer preparation needs `&` of the successor expressions while the other owners
    are prepared under `&mut`. `successor_reads` must borrow the prepared expression
    value and not the session field, as `successor_read_owners` does today at
    `intent_operability_observation.rs:35`.
  - The framework-turn execution struct lends fields individually, so the sink must take
    fields and not `&mut Session`.
- **Performance.**
  - Attached paths lose runtime checks. The detached path gains one epoch compare.
  - Intra-generation writers gain one `u64` increment.
  - Currentness loses repeated `follows` checks.
  - Projection push adds probes only for declared projection operability sources, and
    only after D4/D5.
  - Step 1 pins every existing counter exactly. Steps 2-8 must not move them. Step 5 may
    remove the attached re-check's second `prepare_retained_appearance_owners`
    (`evidence_only.rs:110-134`). If that call is counted anywhere, the counter drops;
    if so, the test is updated with a stated reason. Whether it is counted is UNVERIFIED.
- **Public API.**
  - Everything named here is `pub(crate)` or narrower, except that step 11 touches
    `facade/lifecycle` internals only.
  - `WorthUiDetached*` types appear in `facade/entry`. Whether they are re-exported
    publicly is UNVERIFIED. If they are, step 6 changes a public signature.
- **Detached drift may be a live bug today** (§1.5). Step 1's test decides that. It is
  not a blocker for the design.
- **Committed draft has nothing to push** (§2.2). The plan text asks for push. See D4.
- **Blockers:** none known that stop steps 1-8. The 3b branch touches
  `payload/input_basis`, `admission/currentness.rs` and `condition_consumers.rs`, so
  steps 2-4 will conflict with it. Sequence them after 3b merges, or rebase 3b.

---

## Decisions needed

- **D1.** Should succession be generic (`UiPreparedGenerationSuccession<P: UiSuccessionPlan>`
  with associated types) or one concrete struct per writer that shares owner-level
  helpers? The generic version is compiler-total over owners. The concrete version is
  simpler to read, but a new owner must be added in three places.
- **D2.** Should the detached path always re-prepare on reattach, or re-prepare only
  when the epoch moved? The proposal re-prepares only on a moved epoch, and that
  re-prepare covers the full owner set.
- **D3.** Should the attached re-checks (`evidence_only.rs:110-134`,
  `mounted_content_rebind.rs:162-168`) be deleted outright, or kept as `debug_assert!`?
- **D4.** Should `CommittedDraft` operability become a real dependency, on a draft owner
  revision, or be documented as constant with no push? Today it reads nothing.
- **D5.** Should projection operability stay as source kinds pushed by a frame probe
  (§3.4), or be retired into conditions over projections, consistent with law 8?
- **D6.** Should candidate admission move ahead of successor authority construction
  (cost ordering, law 16)? This changes which denial wins when both apply.
- **D7.** How should phase illegality be proven? Options:
  - (a) add `trybuild` to the `worth-ui` workspace and a `#[doc(hidden)]`
    certification surface;
  - (b) add a certification topology audit in `worth-ui-certification`, in the style of
    `milestone_37_structural_inventory_audit.rs`, that forbids direct writes to the
    succession-owned fields outside `generation_succession/`;
  - (c) rely on privacy plus consuming `self`.
  I recommend (b) plus (c).
- **D8.** Should steps 2-4 wait until 3b merges?
