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

For S.5.1, treat the work as Store-owned cryptographic-boundary, tenant-scope,
authenticity, custody, and security-readiness foundation. This milestone is not
full encryption, not an identity provider, not operator authorization, and not
a certification-owned law layer. Lower Store crates own authority vocabulary,
sealed witnesses, admission, physical metadata, propagation, and readiness
constructors. Certification is the courtroom: it proves the law, but does not
define or mint Store authority.

Use `worth-foundational` for aspect-native boundary facts, canonical basis,
boundary artifacts, boundary evidence, profiles, and counter-backed performance
receipts where the spec names those surfaces. Use `worth-proof` for legal
progression, freshness/readmission topology, non-success outcomes, and
fixed-shape evidence binding. Neither Foundational evidence nor Proof
progression alone may mint Store security authority.

Preserve the hard S.5.1 bans:

- no JSON, serde projection, terminal projection, copied ids, copied proof ids,
  copied counters, or `StoreCurrentAuthorityWitness` alone as security
  authority
- no JWT subject as tenant scope
- no application org id as tenant scope
- no KMS key id as key scope
- no IAM role as custody posture
- no operator identity as repair authority
- no physical metadata declaring `AuthenticityResult`
- no repair readiness standing in for operator authorization
- no backup/import trust-boundary crossing without explicit readmission

## JSON state

Keep the runner event payload small and factual. The runner uses only the
current phase, turn, phase status, QA status, completed_at, short phase note
markers, and small history entries to resume work. Do not put logs, artifacts,
command output tails, full plans, full findings, proof transcripts, or summaries
into `RUNNER_EVENT` payloads.

## Authority and cursor recovery

The durable runner's event log is authoritative. The projection is derived.
Prompts include a `Runner turn instance id`; your `RUNNER_EVENT` payload must
echo it exactly when the prompt asks for it.

The runner sends exactly the turn named by `current`. Available turns: {turns}

If the runner gets out of sync, use the event projection, completed work,
current phase status, prompt kind, open findings, and S.5.1 spec phase to
correct it so the next prompt resumes from the real state. Do not keep working
from a stale cursor.

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

## Turn advancement

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
  `plan`, or `current: null` and `completed_at` if this was the last phase
- after `code_quality_repair`: same phase, turn `code_quality_review`

Only passing `code_quality_review` advances to the next phase in this prompt
set.

## Verification shape

Use focused verification by default: `cargo check`, `cargo test --no-run`,
touched module tests, touched integration targets, targeted compile-fail tests,
and focused line-cap/diff checks. Broad workspace suites are closeout lanes
unless a phase acceptance item explicitly names them.
