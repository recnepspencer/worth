# Plan: public #368, stage-granular correction evidence

Lane pub-368, branch `codex/public-368-stage-correction-20261124`, based on master `455d12d8a4`.

## The problem (trace)
A multi-stage edit envelope commits as one coupled writer group. The root writer
(stage 0, member `Primary`) and the `AuthoredStage` members (stages 1..n) share
one Relational commit and one publication attempt (`Original`). Before this
change, correction evidence named only that commit:

- **Memory: silent wrong-stage reversal.**
  - `select_correction_source` resolved `observe_completed_application(commit)`,
    which returns the group's root writer.
  - A reversal of a stage-1 or stage-2 receipt therefore derived and committed
    the inverse of stage 0. This is a correctness defect, not a refusal.
- **Home: no correctable multi-stage step.**
  - A later stage never retained its original program meaning
    (`retain_original_program`), so it could not be captured as correction
    material.
  - The catalog also kept one entry per `Original`.
- **Aftermath lineage, on both paths.**
  - The causal-fact row and its `reversed-source` selector named a source by
    `(owner, world commit, branch, commit)`.
  - Two stages of one commit therefore shared a selector.

## Discriminator
`ApplicationEditStageIdentity { ordinal, identity }` already exists on every
member's idempotency binding (`authored_stage_identity()`). The root carries
`None`. No new identity is invented.

## Design (as built)
1. **Member coordinate.**
   - `WorthQueryCorrectionSourceMember = Primary | AuthoredStage { ordinal, identity }`
     lives in the lineage crate, with `of_writer`, `names_writer`, and
     `text`/`parse` (`stage:{ordinal}:{hex}`).
   - `WorthQueryRecordedCorrectionSource` and `WorthQueryCorrectionSourceCoordinates`
     carry it, with an explicit constructor argument and no default.
   - `receipt.correction_source()` fills it from the receipt's binding.
   - A correction always commits as its own group's `Primary`.
2. **Stage program retention.** `authored_edit/execution.rs` retains each stage's
   original program before `prepare_coupled_application`, the same as a root
   entry. This is the root cause of the Home gap.
3. **Memory selection.**
   - One helper, `observe_member(commit, member)`, in `selection.rs` picks the
     group writer that `member.names_writer(...)`.
   - It is used for receipt and locator selection.
   - A member that names no writer refuses `SourceMemberUnavailable`.
   - Cost: O(writers) of one completed group.
4. **Home material.**
   - `MaterialCapture`/`MaterialReady` carry `member`.
   - The catalog key is `(Original, Member)`.
   - Lifecycle records (`Abandon`, `Expire`, pins) stay per `Original` and apply
     to every member of it, since all writers share one World original and fate.
   - Capture runs per writer at the World open (`capture.rs`, `of_writer`).
   - `observe_original(commit, member)` filters by both.
   - A commit whose material exists only for other members refuses
     `QueryCorrectionMaterialDenial::MemberUnavailable`. `cold_selection` maps
     it to `SourceMemberUnavailable`, giving the same typed refusal as memory.
5. **ORIGINAL-row join** (`original_join.rs`).
   - A commit has exactly one ORIGINAL idempotency row: the root's binding.
   - A `Primary` member joins by exact intent equality, as before.
   - An `AuthoredStage` member requires an envelope row (`row.envelope.is_some()`)
     and the same key. Its own recorded intent must match its root, mutation and
     input identity (`matches_mutation_input`).
   - Without this rule, a Home reopen between capture and correction refused
     `OriginalMismatch` for every stage.
6. **Aftermath causal-fact row.**
   - Each coordinate set (source, root, selected, active, prior) gains a
     `*-member` field.
   - The `reversed-root` and `reversed-source` selectors include the member, so
     each stage has its own reversal selector.
7. **Format.**
   - `MATERIAL_RECORD_VERSION` goes 3 → 4, and the aftermath aspect revision 4 → 5.
   - There is no legacy decoder (see below).

## Durable format policy
- **Policy.**
  - `docs/coding-guidelines/arch_laws.md:69` (law 21): a current-format-only
    window, with a typed rejection before effects.
  - `docs/relational-maintenance.md:104-107`.
  - `docs/build-an-application.md:5247-5248, 9758-9759`.
  - `skills/qa-loop/SKILL.md` "Preproduction": do not preserve obsolete stored
    formats, and recreate development artifacts instead of migrating them.
- **Precedent bumps with no decoder.**
  - `776e7e8d42`: material v3 dropped v1/v2.
  - `98b39a07bd`: aftermath revision 3 → 4.
  - `60549db58a`: aftermath revision 2 → 3.
- **Old records.**
  - A v3 material frame refuses `UnsupportedVersion(3)` before any fold. This is
    pinned by `a_version_3_material_frame_is_refused_by_version_never_reinterpreted`.
  - The behaviour of a Home written by the pre-fix code is recorded in
    `reports/public-368-qa.md`.

## Courts
Court world: `worth-query-court-decl-host-integration`, numeric Home, on both
MemoryOnly and Directory. One envelope sets three Numbers to `[10, 20, 30]`
(`stage_correction.rs`).
- For each stage: reversing it restores exactly that Number.
- A second, different stage of the same commit then reverses.
- Reversing the same stage again refuses `SourceAlreadyReversed`.
- A forged member (`AuthoredStage{ordinal: 7}`) refuses `SourceMemberUnavailable`
  on both paths.
- Directory: the Home is reopened between capture and correction.
- Mutants: member selection, the catalog key and the stage join (see the QA report).

The fixture's extra Numbers are opt-in (`STAGE_NUMBERS` and the `_with` openers).
Seeding them for every Home pushes `kill_divergent_branch`'s step-5 read past the
default per-request work ceiling (`WorkExhausted`); an A/B run confirmed this.

## Found and filed
- **#375.** A Directory Home close refuses `Busy(PublicationReceipts)` after any
  committed edit envelope. The reopen court drops the Home and cites #375.

## Overlap
- **tbu-p2.**
  - The CS files this lane touches are `selection.rs` (one helper and two call
    sites) and `cold_selection.rs` (the member argument and one denial mapping).
  - It also touches `lineage.rs`, `occurrence.rs`, `exact_original.rs` and
    `alternatives.rs`, with one constructor argument each.
  - None of these files was restructured.
- **pub-358.** The readmission material decode version and `original_join` change.
  Readmission marks are untouched.
