# QA: public #368, stage-granular correction evidence

Lane pub-368, branch `codex/public-368-stage-correction-20261124` on master `455d12d8a4`.
Design and trace: `reports/plan-public-368.md`.

## Behaviour
A later authored stage of a coupled edit envelope is now its own correction
source, on memory and on Directory Homes:

- receipts, coordinates and recorded sources name the coupled-group member;
- selection picks that writer;
- Home material is keyed by `(Original, Member)`;
- aftermath lineage and reversal selectors carry the member.

## Defects found and fixed in this lane
| # | Defect | Trigger | Fix | Court |
|---|---|---|---|---|
| 1 | Memory silently reversed the root for a stage source | reverse stage 1 of `[10,20,30]` | `observe_member` selects the named writer | `stage_correction` memory courts |
| 2 | A Home could not capture a later stage | multi-stage envelope on a Directory Home | stage retains its original program before `prepare_coupled_application` | `stage_correction` directory courts |
| 3 | One material entry per Original | two stages of one commit | catalog key `(Original, Member)` | stage courts, catalog unit tests |
| 4 | Shared reversal selector per commit | reverse stage 1, then stage 2 | member in the lineage coordinates and selectors | second-stage reversal in the stage courts |
| 5 | Reopened Home refused `OriginalMismatch` for stages | close/reopen between capture and correction | the stage join uses its own recorded intent against the root's envelope row | `reopened` directory court |
| 6 | Directory refused a foreign member as `Material(Unavailable)`, memory as `SourceMemberUnavailable` | forged `AuthoredStage{ordinal:7}` | `MemberUnavailable` → `SourceMemberUnavailable` | `foreign_member` courts, both paths |
| 7 | A v3 material frame refused `MalformedFrame`, not by version | any pre-#368 material record | `decode` reads the version before interpreting the body | `a_version_3_material_frame_is_refused_by_version_never_reinterpreted` (red before the fix) |

## Durable format change and compatibility
- **Bumped:**
  - `MATERIAL_RECORD_VERSION` 3 → 4 (capture and ready carry the member);
  - the aftermath causality aspect revision 4 → 5 (a `*-member` field on each
    coordinate set and on the source).
- **No legacy decoder.**
  - Policy:
    - `docs/coding-guidelines/arch_laws.md:69` (law 21: a current-format-only
      window, with a typed rejection before effects);
    - `docs/relational-maintenance.md:104-107`;
    - `docs/build-an-application.md:5247-5248, 9758-9759`;
    - the qa-loop/qa-tests "Preproduction" rule.
  - Precedents with no decoder:
    - `776e7e8d42` (material v3 dropped v1/v2);
    - `98b39a07bd` (aftermath 3 → 4);
    - `60549db58a` (aftermath 2 → 3).
- **Old Home, measured.** Master `455d12d8a4` wrote a Directory Home (an edit
  plus its reversal, closed gracefully). This branch then opened it in place.
  - Result: a typed refusal, before any effect, with no panic and no Damage:
    `WorthQueryApplicationHomeOpenRefused("primary graph installation denied: InvalidSchemaMember (original entity role differs from the installed meaning)")`.
  - Source: the predecessor schema check
    (`schema_layout/physical_identity/predecessor.rs`). The stored aftermath
    causality registration (revision 4) is not the installed one (revision 5).
    It is the only schema-layout change on this branch.
  - The refusal names the changed schema member, not the numeric revision.
  - The material frames are never read, because installation refuses first.
  - Control: master reading its own Home in place opens and closes cleanly.
- **Old material record, frozen.** A v3 frame with no member refuses
  `UnsupportedVersion(3)` from both `decode` and `fold`. The catalog is left
  untouched. Before this lane's decode fix (defect 7) the frame refused
  `MalformedFrame`.
- **Side finding, filed as #378.** A *copied* Home directory is refused on
  master too, as an opaque `ServingOpen "did not complete"`. The discarded
  cause was `Failed(SignalConstruction(DurabilityStateReopenRejected(Wal(MemberPayloadRejected))))`.
  This is pre-existing and unrelated to #368. An earlier evidence attempt that
  copied the Home hit it; the in-place controls above replace that attempt.

## Mutants
Each mutant was env-gated in production code, run against `stage_correction`
(6 courts), and then removed.

| Mutant | Change | Killed by |
|---|---|---|
| Member selection | `observe_member` accepts any writer | 3 memory courts (each-stage, last-stage, foreign member) |
| Catalog key | capture/ready keyed `(Original, Primary)` | directory each-stage, reopened directory |
| Stage join | stage joins only on the root's exact intent | reopened directory |
| Member refusal | `MemberUnavailable` stays `Material(..)` | directory foreign member |

The version-order guard in `MaterialRecord::decode` is covered by the frozen v3
court, which failed before the fix.

## Checks run
All cargo runs used `source build_jobs.sh` with `CARGO_BUILD_JOBS=8`, plain rustc and filtered courts.

| Check | Scope | Result |
|---|---|---|
| Stage courts | `stage_correction` (6 courts, memory and Directory) | 6/6 |
| Correction-material courts | `test(correction_material)`, host integration | 19/19 |
| Catalog unit | execution-attempt lib `test(retained_correction)` | 11/11, including frozen v3 |
| Home and owned-output courts | host integration `test(application_home) \| test(owned_output)`, 8 threads | 116/117; one wall-clock failure, see below |
| Mutants | member selection, catalog key, stage join, member refusal | 4/4 killed |
| Old-Home read | master-written Directory Home opened in place by this branch | typed `InvalidSchemaMember`, no effect (above) |
| Dependents: bank-world | clippy `--all-targets -D warnings`: `bank-http-adapter`, `bank-server`, `bank-courtroom`, `worth-query-bank-court-external-rail-server`, `worth-query-bank-court-user-node`, `worth-query-bank-testkit-server` | clean |
| Dependents: store | check `--all-targets`: `worth-store-application`, `worth-store-court-application` | clean |
| Dependents: fixtures | `consumer_entry` check `--all-targets`; `consumer_values` lock restored after `cargo tree` churn | clean |
| Dependents: exhaustive matches | repo-wide grep for `WorthQueryCorrectionSelectionDenial` and `QueryCorrectionMaterialDenial` matches | only bank `refusal.rs`, which is updated |
| Constitution | `boundary-check` (road1), `agent-context check` | valid / clean |
| Line caps | changed and new `.rs` files at 400 lines or fewer | 62 files, none over |

**The wall-clock failure (#379).**
`reopening::checkpoint_restart::automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom`
asserts its own 120 s deadline (`checkpoint_restart.rs:79`).

- In the 8-thread run it failed at 120.2 s, at mutation 15 of 37.
- Rerun alone, it passed at 110.7 s.
- This lane leaves its path untouched. A plain mutation's group has only a
  `Primary` writer, so the per-stage capture loop in `opening.rs` visits nothing.
- Filed as #379 (a court red on wall clock alone). The court is not changed here.

## Found and filed
- **#379.** A checkpoint-restart court fails its own 120 s wall-clock deadline under parallel load. It passes alone; see "Checks run".
- **#378.** A copied Home is refused with an opaque `ServingOpen`, and the WAL cause is discarded. This is pre-existing; see above.
- **#375.** A Directory Home close refuses `Busy(PublicationReceipts)` after a
  committed edit envelope. Reproduced by the stage reopen court, which drops the
  Home instead of closing it and cites #375.

## Review lenses
Lenses applied to the final diff in the order AGENTS.md sets: qa-loop, qa-tests, then code-quality-qa.

- **qa-loop.**
  - **A suspected finding, ruled out by trace.** The suspicion was that the
    stage ORIGINAL-row join should also compare the edit envelope.
    - The platform's envelope rule was the reason
      (`idempotency/envelope_candidates.rs:13,52`).
    - The trace says otherwise. `for_authored_stage_identities` never calls
      `bind_edit_envelope`, so a stage's recorded intent has no `edit-envelope`
      slot. The envelope rule compares root bindings across retries.
    - The available join is therefore the right one: the key, an envelope row,
      and the stage's own intent matching the captured root, mutation and input.
    - The edit was reverted before any court ran.
    - Rejection-side gap, kept from the original analysis: the court world
      cannot forge a material frame, so no court covers the refusing side of
      the stage join. The accepting side is covered by the reopened Directory
      court and the stage-join mutant.
  - **No material finding.** Checked:
    - lifecycle records stay per `Original` across all members;
    - `original_available` requires every member;
    - expiry by coordinates filters by member;
    - the release sweep is deduplicated through a `BTreeSet`;
    - a correction commits as its own group's `Primary`;
    - memory and Directory refuse the same typed member denial.
- **qa-tests.**
  - Every court observes independently: it reads the Numbers and the typed
    refusal name.
  - There is no oracle copied from the implementation.
  - The fixture's extra Numbers are opt-in, so the default Home's work budget
    is unchanged.
  - The stale catalog expectation (version 3 → 4) was a deliberate contract
    change, so the expectation was updated.
  - The frozen v3 court failed before the decode fix.
  - No unaffected test was changed.
- **code-quality-qa.**
  - `scrutinize_rust_functions.py --dirty`: 49 candidates, mostly pre-existing.
  - `authored_edit/execution.rs` `prepare` grew to 199 lines with the
    stage-retention block. No finding: the root path runs the same steps
    inline, and there is no shared owner to extract into without restructuring
    the root.
  - The selection.rs and cold_selection.rs hunks stay minimal: one helper,
    two call sites and one denial mapping.
