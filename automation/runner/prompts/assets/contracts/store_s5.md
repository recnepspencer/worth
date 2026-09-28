---

# Operating contract for this automated turn

There is no human in this loop. Approval policy is `never` and the sandbox is
full-access. The JSON state file is lightweight phase progress state. The chat
transcript is where plans, reviews, QA lists, command output summaries, and
implementation explanations belong.

## Load before you act

Reason from the sources, never from the phase title alone. Read the spec file,
this phase's scope paths, the relevant public APIs, and the project laws/context:

{project.context_files}

Read `docs\coding-guidelines\MENTALITY.md`,
`docs\coding-guidelines\arch_laws.md`,
`docs\coding-guidelines\composition_laws.md`,
`docs\coding-guidelines\domain_structure_laws.md`, and
`docs\coding-guidelines\perf_laws.md` with special attention on every turn.
Read `docs\coding-guidelines\dx_laws.md` when planning or changing public caller
experience.

For S.5, treat the work as physical isolation for bytes, not semantic MVCC and
not a new test harness. Store owns physical read stability: latches, epochs,
stable read plans, byte guards, copy-on-write publication, reachability
barriers, reclaim eligibility, and S.6 handoff evidence. `worth-relational`
owns semantic visibility. `worth-store-recovery-physics` owns S.4 recovery
correctness. `worth-store-physical-certification` and the S.4.5 harness own the
shared simulation mechanics and certification-owned oracle/evidence pipeline.

Every hostile S.5 interleaving lane must consume the S.4.5 harness readiness and
the public scenario/lowering/schedule/observer/oracle/transcript/evidence
pipeline. Do not build a local S.5 runner, do not put verdict meaning in test
support, and do not satisfy proof with logs, fixture labels, same-run
self-comparison, private mutation, JSON authority, copied readiness rows, or
terminal projections.

## JSON state

Keep the state file small and factual. The runner uses only the current phase,
turn, phase status, QA status, completed_at, short phase note markers, and small
history entries to resume work.

## State-mutation protocol

The state file may be written by more than one process. Obey this exactly:

1. Read the state file fresh from disk in the same command or script that writes
   it.
2. Mutate only the current phase row, the `current` cursor, `completed_at`, and
   small history entries describing this turn.
3. Preserve everything else exactly: all other phase rows, `session`, `project`,
   `turn_templates`, prompt text, and existing history.

## Status values

Phase `status` is one of: {status_values}
QA `qa_status` is one of: {qa_status_values}

Use them this way:

- implementation still in progress -> `status: in_progress`
- implementation done, phase-done QA not yet run -> `status: complete`,
  `qa_status: needed`
- phase-done QA passed -> `status: complete`, `qa_status: passed`
- phase-done QA found a real gap -> `status: regressed`, `qa_status: failed`
- genuinely blocked -> `status: blocked`

The phase-done loop and structural code-quality loop are mandatory gates.
`review` may send the phase to `repair`, and `repair` returns to `review`.
`code_quality_review` sends the phase to `code_quality_repair` when it finds
concrete composition-law, domain-structure-law, file-size, directory-topology,
public-facade, `mod.rs` business-logic, helper-placement, missed-abstraction, or
ownership-boundary violations. Vague perfection concerns do not loop, but
concrete structural-law violations are phase defects and must not be recorded as
optional residue. Do not route structural findings through generic `repair`;
semantic repair, test repair, and structural repair are separate turns.

## Cursor rules

The runner sends exactly the turn named by `current`. Available turns: {turns}

Use focused verification by default: `cargo check`, `cargo test --no-run`,
touched module tests, touched integration targets, targeted compile-fail tests,
and focused line-cap/diff checks. Run broad workspace suites only when the phase
acceptance explicitly requires them.

Advance like this:

- after `plan`: same phase, turn `implement`
- after `implement`: same phase, turn `review` if implementation is ready for
  the phase-done check; otherwise stay on `implement`
- after `review`: same phase, turn `repair` if the phase is not actually done;
  turn `test_review` if the phase is actually done
- after `repair`: same phase, turn `review`
- after `test_review`: same phase, turn `test_repair_plan` if test findings
  need fixes; turn `code_quality_review` if test hardening is not needed
- after `test_repair_plan`: same phase, turn `test_repair_implement`
- after `test_repair_implement`: same phase, turn `code_quality_review`
- after `code_quality_review`: same phase, turn `code_quality_repair` if
  structural QA found concrete law violations; otherwise next phase at turn
  `plan`, or `current: null` and `completed_at` if this was the last phase
- after `code_quality_repair`: same phase, turn `code_quality_review`

Only passing `code_quality_review` advances to the next phase in this prompt
set.

Runner sync note: The phase runner JSON is the milestone state file in
`automation/phase_runner/`. If it gets out of sync, use the completed work,
current phase status, prompt kind, open findings, and spec phase to correct it
so the next prompt resumes from the real state.
