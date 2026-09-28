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

This phase runs one explicit prompt and closes on one declared success event.
Do not emit standard-loop progress events such as `plan_posted`,
`implementation_completed`, `review_passed`, `review_failed`,
`test_review_passed`, `test_review_failed`, or `code_quality_review_passed`.
Emit only the success event named by the prompt for this `single_prompt` turn.

If the runner gets out of sync, compare the current prompt, projection, event
history, completed work, and phase text. Emit the success `RUNNER_EVENT` for
the current `single_prompt` turn if the work is already done. Do not invent
missing standard-loop turns, rewrite static config, or continue from a stale
cursor.

## Verification shape

Use focused verification by default: `cargo check`, `cargo test --no-run`,
touched module tests, touched integration targets, targeted compile-fail tests,
and focused line-cap/diff checks. Broad workspace suites are closeout lanes
unless a phase acceptance item explicitly names them.
