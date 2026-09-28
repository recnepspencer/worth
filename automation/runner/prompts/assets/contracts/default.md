---

# Operating contract for this automated turn

There is no human in this loop. Approval policy is `never` and the sandbox is
full-access. The JSON state file is only lightweight progress state. The chat
transcript is where plans, reviews, QA lists, command output summaries, and
implementation explanations belong.

## Load before you act

Reason from the sources, never from the phase title alone. Read the spec file,
this phase's scope paths, the relevant public APIs, and the project laws/context:

{project.context_files}

Read `docs\coding-guidelines\MENTALITY.md` and
`docs\coding-guidelines\arch_laws.md` with special attention on every turn.
Read `docs\coding-guidelines\dx_laws.md` when planning or changing public caller
experience.

## JSON is progress state, not an evidence bundle

Do not put logs, artifacts, command output tails, long plans, long findings,
full QA reports, or proof transcripts into the JSON state. Keep those in chat.

The JSON may contain only short tracking markers:

- `status`
- `qa_status`
- `current`
- `completed_at`
- short entries in `notes.plan`, `notes.done`, `notes.remaining`,
  `notes.findings`, and `notes.verification`
- small runner history entries

Every note entry should be a compact pointer, not a report.

## Runner-owned authority files

Never write, rewrite, append to, or repair the event log, projection,
checkpoint, or any other runner runtime artifact. The runner alone owns those
files and assigns event identity, sequence, phase, and turn authority.

Report your completed turn only by emitting the requested `RUNNER_EVENT` line
in chat. The runner validates and records it. Do not use shell commands, editor
tools, or scripts to update runner state.

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
  public-facade, `mod.rs` business-logic, helper-placement, missed-abstraction,
  or ownership-boundary violations. Vague perfection concerns do not loop, but
  concrete structural-law violations are phase defects and must not be recorded
  as optional residue. Do not route structural findings through generic
  `repair`; semantic repair, test repair, and structural repair are separate
  turns.

## Cursor rules

The runner sends exactly the turn named by `current`. Available turns: {turns}

Iteration evidence is not the same thing as closeout evidence. During `plan`,
`implement`, `review`, and `repair`, prefer the narrowest command that proves
the phase claim: `cargo check`, `cargo test --no-run`, the touched module tests,
the touched integration target, and the named compile-fail target that protects
the public boundary. Do not run whole-crate or whole-workspace suites such as
`cargo test -p worth-query --tests`, `cargo test -p worth-spatial --tests`, or
the `worth-spatial` `public_api_contract` umbrella unless the phase acceptance
explicitly names that exact command as required closeout evidence. Heavy
metaboss, public API umbrella, architectural closeout, and broad compile-fail
suites are closeout lanes, not ordinary iteration lanes. If a listed acceptance
item is too broad for iteration, replace it with focused proof plus test
compilation (`--no-run`) and record the deferred broad command as explicit
closeout-only evidence rather than burning the loop.

For deletion-ledger acceptance, also point to the former public/exported surface
and the new outcome: deleted file/symbol, collapsed canonical proof surface,
certification-only boundary, capped residue row, or named Query gap. If you
cannot name the old surface and its new enforced outcome, the deletion ledger is
not resolved.

Default phase turns advance like this:

- after `plan`: same phase, turn `implement`
- after `implement`: same phase, turn `review`
- after `review`: same phase, turn `repair` if the phase is not actually done;
  turn `test_review` if the phase is actually done
- after `repair`: same phase, turn `review`
- after `test_review`: same phase, turn `test_repair_plan` if test findings
  need fixes; turn `code_quality_review` if test hardening is not needed
- after `test_repair_plan`: same phase, turn `test_repair_implement`
- after `test_repair_implement`: same phase, turn `test_review` if fixes need
  re-review; turn `code_quality_review` if tests are now honest enough
- after `code_quality_review`: same phase, turn `code_quality_repair` if
  structural QA found concrete law violations; otherwise next phase at turn
  `plan`, or `current: null` and `completed_at` if this was the last phase
- after `code_quality_repair`: same phase, turn `code_quality_review`

Only passing `code_quality_review` advances to the next phase in this prompt
set.
