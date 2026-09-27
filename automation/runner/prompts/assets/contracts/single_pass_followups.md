---

# Operating contract for this automated turn

There is no human in this loop. Approval policy is `never` and the sandbox is
full-access. The JSON state is lightweight phase progress only. The chat
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

## Authority and cursor recovery

The durable runner's event log is authoritative transport. The projection is
derived status. The chat transcript is the artifact of record for plans,
findings, explanations, and command summaries.

Prompts include a `Runner turn instance id`; your `RUNNER_EVENT` payload must
echo it exactly when the prompt asks for it.

The runner sends exactly the turn named by `current`. Available turns: {turns}

If the runner gets out of sync, compare the current prompt, projection, event
history, completed work, open findings, and phase text. Emit the `RUNNER_EVENT`
for the phase/turn that actually just completed so the next prompt resumes from
the real state. Do not invent missing phases, rewrite static config, or continue
from a stale cursor.

Never emit `repair_blocked`, `review_blocked`, or any blocked-style event. The
runner event vocabulary does not support those events. If you cannot finish the
repair, emit an allowed event for the actual turn with explicit findings/notes:
`review_failed` from review, `test_repair_completed` from test repair, or
`code_quality_repair_completed` from code-quality repair after a real repair
attempt. Do not use unsupported JSON to communicate blockage.

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

## Turn advancement

- after `boundary_review`: same phase, turn `plan`
- after `plan`: same phase, turn `implement`
- after `implement`: same phase, turn `review` if implementation is ready for
  the phase-done check; otherwise stay on `implement`
- after `review`: same phase, turn `repair` if the phase is not actually done;
  turn `test_review` if the phase is actually done
- after `repair`: same phase, turn `review`
- after `test_review`: same phase, turn `test_repair_implement` if test
  findings need fixes; turn `code_quality_review` if test hardening is not
  needed
- after `test_repair_implement`: same phase, turn `code_quality_review`
- after `code_quality_review`: same phase, turn `code_quality_repair` if
  structural findings need fixes; otherwise advance to the next phase at turn
  `boundary_review` when this run has that configured turn, otherwise next phase
  at turn `plan`
- after `code_quality_repair`: advance to the next phase at turn
  `boundary_review` when this run has that configured turn, otherwise next phase
  at turn `plan`; or `current: null` and `completed_at` if this was the last
  phase

The only recursive loop is the implementation done-ness loop:
`review -> repair -> review`. Boundary review is mandatory when configured.
`test_review` and `code_quality_review` are mandatory closeout passes, and their
repair turns are mandatory when findings exist, but each of those families gets
one repair implementation turn rather than a recursive loop.

## Blocking threshold

These closeout turns are mandatory, but not every concern is phase-blocking.

- blocking findings are defects that leave the admitted public lane dishonest,
  leave an immediate trusted-owner bypass alive, violate the declared phase
  boundary, or leave concrete structural-law defects in touched code
- non-blocking follow-up hardening is deeper same-repo trusted-internal shaping
  residue that does not reopen ordinary consumer bypass, does not falsify the
  acceptance claim, and does not leave the touched structure dishonest

Review turns should say which category a concern belongs to. The runner should
advance once blocking findings are closed, even if follow-up hardening residue
is noted in chat.

## Verification shape

Use focused verification by default: `cargo check`, `cargo test --no-run`,
touched module tests, touched integration targets, targeted compile-fail tests,
and focused line-cap/diff checks. Broad workspace suites are closeout lanes
unless a phase acceptance item explicitly names them.
