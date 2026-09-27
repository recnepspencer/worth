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

For S.7, treat native blob/object chunk storage as Store-owned physical
authority. Blob identity, generation publication, chunk integrity, chunk-tree
roots, resumable ingest state, dedupe, reachability, retention, placement,
import/export readmission, capsule readiness, compaction, and closeout witnesses
must be defined in lower Store crates. Certification is the courtroom: it
materializes and proves executed Store law, but does not define the law or mint
runtime authority.

When review failures repeat in the same phase, treat that as a root-cause
signal, not a request for another wrapper. Identify the bad ownership boundary,
whole-object loophole, forgeable authority path, copied-counter path, or
certification-owned law path, then repair the law surface directly. It is
acceptable to move/create a lower Store vocabulary/contract surface, seal
constructors, replace public data bags with private-field witnesses, move
authority out of certification, and add compile-fail/API-misuse proof when that
is the principled fix.

Use `worth-foundational` only where the S.7 spec names it: aspects,
canonicalization, boundary evidence, profiles, performance policy receipts,
support/compatibility posture, and counter-backed evidence publication. Use
`worth-proof` for checked progression, freshness, rebind-required states,
denial/failure topology, trust-boundary readmission, and fixed-shape evidence
binding. Neither Foundational nor Proof evidence alone may stand in for Store
blob identity, publication, chunk, reachability, placement, retention, import,
capsule, compaction, or closeout witnesses.

## S.7 hard rules

- `BlobObjectId`, `BlobGeneration`, `ChunkTreeRoot`, `LogicalContentDigest`,
  `StoredChunkDigest`, `AuthenticatedFrameDigest`, and `LifecycleReceipt` are
  distinct typed concepts
- published blob generations are immutable; byte changes, chunking-rule
  changes, transform changes, or authority-classification changes require a
  new generation or new object identity
- the only ordinary publication event is `BlobGenerationPublished`
- chunks, root candidates, staged reachability, and resume sessions are not
  visible blob generations before publication commits
- blob lifecycle facts must integrate with Store WAL, checkpoints, manifests,
  and recovery replay; backend residue is never authority
- S.7 stores canonical raw byte-stream chunks only; compression is a later
  explicit transform layer
- no S.7 certification path may materialize the full logical blob in heap, one
  scalar buffer, one temp sidecar file, or one expected-byte artifact
- counter strength must be explicit; lifecycle, publication, reachability,
  reclaim, corruption, memory-bound, and heavy multi-GB claims require exact
  counters unless the spec names a weaker non-authoritative strength
- resumable ingest uses named typed states, not one mutable flag bag
- dedupe policy modes are explicit; digest equality alone never authorizes
  cross-scope sharing
- dedupe receipts must bind into reachability and reclaim; shared chunks cannot
  be reclaimed until every admitted sharing edge is absent or denied
- corruption and quarantine are first-class lifecycle states and participate in
  reachability, export/import denial, capsule denial, repair, rebuild, and
  reclaim
- external placement is Store-owned physical storage governed by Store
  witnesses, manifests, security scope, reachability, reclaim, and recovery
- filesystem paths, object-store keys, URLs, external metadata databases, JSON,
  terminal projections, copied receipts, copied proof ids, and raw counters are
  never blob lifecycle authority
- import readmission produces a placement admission plan before an imported
  witness
- export canonical basis is a boundary representation, not the runtime
  chunk-tree format or authority model
- capsule readiness has a positive physical model and is not backup,
  replication closeout, or product API correctness
- blob compaction belongs in S.7, but it must not change blob object identity,
  generation visibility, security scope, logical content, or authority class
- S.8 owns global layout discipline; S.7 owns only blob-local chunk-tree,
  metadata, placement vocabulary, streaming counters, and blob-local compaction

## Authority and cursor recovery

The durable runner's event log is authoritative. The projection is derived.
Prompts include a `Runner turn instance id`; your `RUNNER_EVENT` payload must
echo it exactly when the prompt asks for it.

The runner sends exactly the turn named by `current`. Available turns: {turns}

If the runner gets out of sync, compare the current prompt, projection, event
history, completed work, open findings, and S.7 phase text. Emit the
`RUNNER_EVENT` for the phase/turn that actually just completed so the next
prompt resumes from the real state. Do not invent missing phases, rewrite the
static config, or continue from a stale cursor.

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
