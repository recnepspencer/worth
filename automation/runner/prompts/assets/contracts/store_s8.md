---

# Operating contract for this automated S.8 turn

There is no human in this loop. Approval policy is `never` and the sandbox is
full-access. The JSON runner state is lightweight phase progress only. The chat
transcript is where plans, reviews, QA lists, command output summaries, and
implementation explanations belong.

## Load before you act

Reason from sources, never from the phase title alone. Read the spec file, this
phase's scope paths, the relevant public APIs, and the project laws/context:

{project.context_files}

Read `docs/coding-guidelines/MENTALITY.md`,
`docs/coding-guidelines/arch_laws.md`,
`docs/coding-guidelines/composition_laws.md`,
`docs/coding-guidelines/domain_structure_laws.md`, and
`docs/coding-guidelines/perf_laws.md` with special attention on every turn.
Read `docs/coding-guidelines/dx_laws.md` when planning or changing public caller
experience.

For S.8, treat physical layout, index strategy, and access-path discipline as
Store-owned database law. `worth-store-layout-indexes` owns shared S.8
layout/access grammar; family crates own their local execution authority.
Certification is the courtroom: it proves executed Store law but does not
define production law or mint runtime authority.

## S.8 hard rules

- The Domain Skeleton Contract in the S.8 spec is binding architecture. Use it
  every phase.
- Phase IDs are native to the milestone config. S.8 starts at phase `0`.
- Keep `worth-store-layout-indexes` as layout/access grammar, not execution
  authority for pages, WAL, recovery, blobs, security, operations, or tests.
- Existing family crates keep execution authority:
  `worth-store-physical-format`, `worth-store-wal`,
  `worth-store-recovery-physics`, `worth-store-buffer-pool`,
  `worth-store-physical-integrity`, `worth-store-physical-isolation`,
  `worth-store-io-scheduler`, `worth-store-blob-chunks`,
  `worth-store-security`, and `worth-store-operations`.
- Build inside the selected target topology. Files may become directories when
  justified, but ownership, lifecycle order, facade position, and authority
  direction must not drift.
- No broad scan may masquerade as point, prefix, range, streaming,
  locality-bounded, or cheap foreground access.
- Explicit degraded exact scan is allowed only as a caller-visible, budgeted,
  counter-backed, non-indexed outcome class.
- Physical indexes and layout projections are derived unless a Store-owned
  type explicitly classifies an artifact as authoritative.
- B-tree and LSM strategy claims must have concrete, mechanically testable
  invariants. Do not leave them as generic "index" vocabulary.
- Counters are evidence only when bound to the executed Store path. Copied
  rows, logs, reports, JSON, terminal projections, and certification fixtures
  are not authority.
- Use `worth-foundational` for shared boundary/performance/aspect vocabulary
  after Store-owned admission or execution, not as a substitute for Store law.
- Use `worth-proof` for checked progression, non-success topology, freshness,
  rebind/readmission, and fixed-shape evidence binding.

## Root-cause repair rule

When review failures repeat in the same phase, treat that as an ownership or
transition-grammar signal. Identify the bad crate boundary, proof-flow collapse,
forgeable constructor, copied-counter path, hidden broad scan, generic helper
bag, or certification-owned-law path. Repair the law surface directly. It is
acceptable to move/create a lower Store vocabulary surface, seal constructors,
split modules, replace public data bags with private-field witnesses, and add
compile-fail/API-misuse proof when that is the principled fix.

## Authority and cursor recovery

The durable runner's event log is authoritative. The projection is derived.
Prompts include a `Runner turn instance id`; your `RUNNER_EVENT` payload must
echo it exactly when the prompt asks for it.

The runner sends exactly the turn named by `current`. Available turns: {turns}

If the runner gets out of sync, compare the current prompt, projection, event
history, completed work, open findings, and S.8 phase text. Emit the
`RUNNER_EVENT` for the phase/turn that actually just completed so the next
prompt resumes from the real state. Do not invent missing phases, rewrite the
static config, or continue from a stale cursor.

Never emit unsupported blocked-style events. If you cannot finish the repair,
emit an allowed event for the actual turn with explicit findings/notes:
`review_failed` from review, or `repair_completed` from repair after a real
repair attempt.

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
ownership-boundary violations. Structural findings go through
`code_quality_repair`, not generic repair.

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
- after `code_quality_review`: same phase, turn `code_quality_repair` if
  structural QA found concrete law violations; otherwise next phase at turn
  `boundary_review` when this run has that configured turn, next phase at turn
  `plan`, or `current: null` and `completed_at` if this was the last phase
- after `code_quality_repair`: same phase, turn `code_quality_review`

Only passing `code_quality_review` advances to the next phase in this prompt
set.

## Verification shape

Use focused verification by default: `cargo check`, `cargo test --no-run`,
touched module tests, touched integration targets, targeted compile-fail tests,
and focused line-cap/diff checks. Broad workspace suites are closeout lanes
unless a phase acceptance item explicitly names them.
