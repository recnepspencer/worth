# Rulings on succession-survey.md (D1-D8)

Governing: arch laws 2, 9, 12, 13, 16; MENTALITY.md; the user's 2026-09-29
directive "phases must ALWAYS be typed ... the goal is clean code".

Verified at HEAD 2056f4bd4b:
- detached.rs:114-121 `complete` calls `finish` with no currentness proof. Confirmed.
- detached.rs:32-59 `rebase` re-prepares geometry, expressions, pointer, and
  owners, but not appearance or overlay bindings. Confirmed.
- axis_observation.rs:46-50,77-80: `ProjectionReadonly` is always Readonly and
  retains the projection. `Projection` readiness is Ready iff the input is
  Current. `CommittedDraft` is a constant. Confirmed.
- resolution.rs:86-88,121-123: `CommittedDraft` requires a draft interaction
  (`require_draft_source`). Confirmed.

## D1. Generic plan

Use `UiPreparedGenerationSuccession<P: UiSuccessionPlan>` with associated
disposition types.
- A new owner must break every writer at compile time (law 9).
- A writer must not be able to state a disposition its kind cannot have. For
  example, cutover cannot "retain" standing facts.
- One concrete struct per writer satisfies neither.
- Readability requirement: every associated type gets a doc line naming its
  legal dispositions. The trait lives in one file, with the three plans beside it.

## D2. Reattach always re-prepares the full Retain set

- No epoch counter. A hand-bumped epoch would recreate the convention this
  cleanup removes: an intra-generation writer that forgets to bump.
- Preparation is deterministic, so when nothing drifted the result is identical.
  When something drifted, the result is the truth.
- The cost falls on detached authored rebinds only, which are not a per-frame
  path. The characterization tests record the op-count delta, and the commit
  message states it.
- The prepared value carried across detach exists only to build the presented
  frame. `UiDetachedSuccession` holds what is needed to present, and nothing
  that can be committed.

## D3. Delete the attached re-checks, with no `debug_assert!` stand-ins

- Delete them after step 1's characterization tests show equivalence.
- The same applies to §3.3's list: `prepared_succession.rs:81-84,88-91`,
  `mounted_owner_receipt_succession.rs:40-41,53`, `application_rebind.rs:45-48`
  and `portal_lifecycle.rs:83,85`.
- Where a type cannot yet carry the invariant, restructure until it can: the
  prepared value borrows its session, or it is minted only by `reattach`.
- Step 10 becomes "delete", not "demote".
- The expression commit's catch-up (`prepared_succession.rs:96-101`) stays only
  where a typed phase names its input. In the authored attached path, `finish`
  hands the frame the succession itself published into the commit as a typed
  argument. It is not an unnamed revision diff. On the detached path, re-preparation
  makes the catch-up dead.

## D4. `CommittedDraft` is an honest constant

- It is legal only on a draft interaction (`require_draft_source`). It reads no
  owner. The draft owner gates the edit, and draft currentness is checked at
  payload admission.
- A push for it would push nothing. The consumer index declares
  `CommittedDraft => None`, with that reason in the doc.
- Correct the 3a plan sentence that says a committed draft re-observes.

## D5. Keep the projection sources and push them

- `Projection` readiness is Query currentness posture (Ready iff Current). It is
  not an authored Boolean, so law 8 ("Booleans are conditions") does not retire it.
- `ProjectionReadonly` is a structural fact: projection-backed data is read-only.
  It is not a Boolean either.
- Both retain their projection input, so a standing fact goes stale when the
  projection moves. Push through the frame probe keeps those facts current,
  which is what the plan requires.
- `Projection` and `ProjectionReadonly` both map to `Some(Projection(slot))`.

## D6. Admission before construction

- Candidate admission moves ahead of successor authority construction (law 16
  cost ordering: eligibility before expensive work).
- The changed denial precedence is intended. Admission denials win, and a test
  pins that precedence.
- The implementer must verify that admission reads only the admission basis and
  the candidate (`change_classification.rs:137-150`). If it needs construction
  output, stop and report.

## D7. Privacy plus consuming `self`, plus a certification topology audit

- The audit lives in `worth-ui-certification`, in the style of the milestone 37
  structural inventory. It forbids writes to succession-owned session fields
  outside `facade/entry/generation_succession/`.
- It also forbids writes to the two snapshot slots except through their
  methods.
- No `trybuild`: the types are crate-private, so a compile-fail test would prove
  only privacy.

## D8. Do not wait for 3b

- 3b is redone on the new structure from `worth-ui-3.17-3b-wip`. The branch is a
  parts bin, not a rebase target.

## Additions the survey did not cover (laws decide these)

- **Retire `ApplicationBoolean` for every operability axis AND confirmation**
  (`observe_confirmation`, `axis_observation.rs:129-143`). Law 8.
  - This deletes the two `.expect`s in `application_boolean`
    (`axis_observation.rs:178-187`).
  - The payload `ApplicationBoolean` retires in the 3b redo, where `payload
    condition` replaces it.
- A confirmation source becomes `NotRequired | Condition`, and the consumer
  index covers it.
- The typed-phase audit (`typed-phase-audit.md`) findings merge into the same
  phase.
