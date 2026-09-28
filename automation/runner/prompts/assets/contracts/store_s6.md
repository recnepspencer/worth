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

For S.6, treat hardware-aware I/O, QoS, and background-work pacing as Store-owned
physical authority. Backend/media capability admission, foreground reservation,
background pacing, durability ordering, access-mode policy, secure-I/O
preservation, and later-milestone handoffs must be defined in lower Store crates.
Certification is the courtroom: it materializes and proves executed Store law,
but does not define the law or mint runtime authority.

When review failures repeat in the same phase, treat that as a root-cause signal,
not a request for another wrapper. Identify the bad ownership boundary or
forgeable authority path, then repair the law surface directly. It is acceptable
to move/create a lower Store vocabulary crate, seal constructors, replace public
count bags with private-field witnesses, move authority out of certification,
and add compile-fail/API-misuse proof when that is the principled fix.

Use `worth-foundational` only where the S.6 spec names it: profiles, canonical
bases, boundary evidence, performance policy receipts, support/compatibility
posture, and counter-backed evidence publication. Use `worth-proof` for checked
progression, freshness, rebind-required states, denial/failure topology, and
fixed-shape evidence binding. Neither Foundational nor Proof evidence alone may
stand in for Store capability, reservation, pacing, durability, security, or
handoff witnesses.

## S.6 hard rules

- backend/media capability is never a raw label, config string, OS name,
  environment variable, copied probe row, or terminal projection
- evidence class strength must be preserved: config, probe, external guarantee,
  unverifiable assumption, and certified backend profile are not interchangeable
- resource units are explicit: `QueueSlot`, `BandwidthToken`, `FlushPermit`,
  `SyncDebt`, `ReadAheadWindow`, `WriteBackWindow`, `DirtyPageBudget`,
  `WorkerPermit`, `CacheResidencyHint`, and `ReclaimPermit`
- foreground envelopes must distinguish admitted, held, denied, stale/rebind,
  and violated-with-cause states
- foreground-vs-foreground arbitration is as real as foreground-vs-background
  arbitration
- background idle-capacity use requires revocable leases and typed yield,
  revoke, defer, deny, throttle, admit-with-debt, or violation outcomes
- execution may adapt mechanically within admitted policy, but may not
  reclassify work, strengthen claims, hide debt, or change durability/security
  meaning
- write grouping must declare the positive basis for grouping; "not known
  unsafe" is not a grouping basis
- durability claims must be backend-neutral and must not depend on a backend
  choosing semantics at execution time
- direct I/O, mmap, page-cache, and mixed-mode access require admitted
  coherence and fault posture; unsupported or unknown posture is a typed denial
- trim, punch-hole, and cold-tier policies must preserve protected reachability
  and distinguish physical zeros, logical holes, unavailable bytes, and
  non-observable reclaimed storage
- secure-I/O scope from S.5.1 must survive reservation, grouping, batching,
  read-ahead, write-back, background leases, and repair/backup/verification
  pressure
- latency and interference counters must declare strength: exact, bounded,
  sampled, derived, certification-only, or unavailable
- deterministic replay is scoped to policy decisions, counter topology, and
  proof progression, not wall-clock OS timing
- post-admission violations are typed outcomes, not successful execution with a
  warning log
- S.7/S.10/S.11 handoffs are non-claim readiness seeds only; they must not imply
  compaction, placement, or operator authorization readiness

## Authority and cursor recovery

The durable runner's event log is authoritative. The projection is derived.
Prompts include a `Runner turn instance id`; your `RUNNER_EVENT` payload must
echo it exactly when the prompt asks for it.

The runner sends exactly the turn named by `current`. Available turns: {turns}

If the runner gets out of sync, compare the current prompt, projection, event
history, completed work, open findings, and S.6 phase text. Emit the
`RUNNER_EVENT` for the phase/turn that actually just completed so the next prompt
resumes from the real state. Do not invent missing phases, rewrite the static
config, or continue from a stale cursor.

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
