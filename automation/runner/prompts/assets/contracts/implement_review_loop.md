---

# Operating contract for this automated turn

There is no human in this loop. Approval policy is `never` and the sandbox is
full-access. The JSON state is lightweight phase progress only. The chat
transcript is where plans, reviews, command output summaries, and implementation
explanations belong.

## Load before you act

Reason from the sources, never from the phase title alone. Read the spec file,
this phase's scope paths, the relevant public APIs, and the project laws/context:

{project.context_files}

Read `docs\coding-guidelines\MENTALITY.md`,
`docs\coding-guidelines\arch_laws.md`,
`docs\coding-guidelines\composition_laws.md`,
`docs\coding-guidelines\domain_structure_laws.md`, and
`docs\coding-guidelines\perf_laws.md` with special attention on every turn.

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

Never emit unsupported blocked-style events. If you cannot finish the current
turn, emit an allowed event for the actual turn with explicit findings/notes:
`review_failed` from review, or `repair_completed` from repair after a real
repair attempt.

## Status values

Phase `status` is one of: {status_values}
QA `qa_status` is one of: {qa_status_values}

Use them this way:

- implementation still in progress -> `status: in_progress`
- implementation done, phase review not yet run -> `status: complete`,
  `qa_status: needed`
- review passed -> `status: complete`, `qa_status: passed`
- review found a real gap -> `status: regressed`, `qa_status: failed`
- genuinely blocked -> `status: blocked`

## Turn advancement

- after `boundary_review`: same phase, turn `plan`
- after `plan`: same phase, turn `implement`
- after `implement`: same phase, turn `review`
- after `review`: same phase, turn `repair` if the phase is not actually done;
  otherwise next phase at turn `boundary_review` when this run has that
  configured turn, else next phase at turn `plan`; or `current: null` and
  `completed_at` if this was the last phase
- after `repair`: same phase, turn `review`

Only the implementation review loop is authoritative in this prompt set. Do not
invent test-review or code-quality-review obligations for this run unless the
phase instructions explicitly tell you to treat them as ordinary implementation
work.

## Verification shape

Use focused verification by default: targeted checks, narrow tests, and direct
artifact inspection tied to the phase acceptance. Broad workspace suites are
closeout lanes unless a phase acceptance item explicitly names them.
