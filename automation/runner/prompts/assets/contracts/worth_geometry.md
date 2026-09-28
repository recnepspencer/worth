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

For Worth geometry work, treat geometry-kernel law as lower-surface authority.
Touched-graph closure, query-backed reads, aspect membership, index products,
geometry operation witnesses, replay/checkpoint parity, and downstream handoff
receipts must be defined by the crate that owns the law. Kernel/certification is
the courtroom: it materializes, composes, and proves executed law, but it does
not define the law or mint runtime authority.

When review failures repeat in the same phase, treat that as a root-cause
signal, not a request for another wrapper. Identify the bad ownership boundary,
forgeable authority path, copied-counter path, projection-owned law path,
fixture-owned proof path, or mixed ordinary lane, then repair the law surface
directly. It is acceptable to move/create a lower Worth vocabulary/contract
surface, seal constructors, replace public data bags with private-field
witnesses, move authority out of certification, and add compile-fail/API-misuse
proof when that is the principled fix.

Use Query, Index, Aspect, and Touched Graph surfaces only where their authority
actually applies. Query proves query-backed read/admission facts. Index products
prove indexed lookup/reuse facts. Aspect products prove aspect membership and
routing facts. Touched graph products prove invalidation/closure/reachability
facts. None of those may be copied into Worth as a second ontology, and none may
be replaced by projection summaries, counters, strings, terminal output, JSON,
or test fixtures.

## Worth geometry hard rules

- lower owning crates define sealed law, runtime witnesses, and authority-bearing receipts
- kernel/certification composes and proves executed law; it does not mint lower truth
- touched graph, query, index, aspect, replay, checkpoint, and geometry-operation authority are distinct typed concepts
- public constructors must not make impossible geometry/proof states constructible
- copied ids, copied receipts, copied counters, projections, strings, JSON, logs, terminal output, and fixtures are never authority
- hostile proofs must pass through the real ordinary owner seam, not a certification-only helper seam
- replay and checkpoint parity must bind to the same admitted authority basis consumed by ordinary callers
- downstream handoffs must consume typed receipts, not reconstructed local summaries
- query/index/aspect/touched-graph facts must synthesize into one ordinary production lane instead of parallel local ontologies
- parallel cutover is the migration shape: create/use the new owner lane, cut ordinary callers over, then cap/delete/disconnect the displaced lane
- in-place refactors that leave mixed ownership as ordinary behavior do not close a phase
- deletion/residue claims must name the old public/exported surface and its new enforced outcome
- performance claims require the phase-appropriate counter strength and must not hide repeated reconstruction behind certification
- if a lower capability is missing, name the exact missing capability; if it cannot be named, treat the issue as Worth-local residue, a bad assumption, or a wrong test seam

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
`review_failed` from review, or `repair_completed` from repair after a real
repair attempt. Do not use unsupported JSON to communicate blockage.

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

The only mandatory loop is the phase-done loop: `review` may send the phase to
`repair`, and `repair` returns to `review`. Test quality, test repair, and
code-quality review are follow-up passes after phase done-ness; they do not
loop on aerospace-grade or vague perfection.

## Turn advancement

- after `boundary_review`: same phase, turn `plan`
- after `plan`: same phase, turn `implement`
- after `implement`: same phase, turn `review` if implementation is ready for
  the phase-done check; otherwise stay on `implement`
- after `review`: same phase, turn `repair` if the phase is not actually done;
  turn `test_review` if the phase is actually done
- after `repair`: same phase, turn `review`
- after `test_review`: same phase, turn `test_repair_plan` if test findings
  need fixes; turn `code_quality_review` if test hardening is not needed
- after `test_repair_plan`: same phase, turn `test_repair_implement`
- after `test_repair_implement`: same phase, turn `code_quality_review` or
  `test_review` only when the prompt explicitly requires re-review
- after `code_quality_review`: next phase at turn `boundary_review` when this
  run has that configured turn, otherwise next phase at turn `plan`; or
  `current: null` and `completed_at` if this was the last phase

Only `code_quality_review` advances to the next phase in this prompt set.

## Verification shape

Use focused verification by default: `cargo check`, `cargo test --no-run`,
touched module tests, touched integration targets, targeted compile-fail tests,
and focused line-cap/diff checks. Broad workspace suites are closeout lanes
unless a phase acceptance item explicitly names them.
