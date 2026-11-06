# Independent funding merge review 02

Verdict: **REJECT** for merge closure. The funding implementation and conflict resolutions have no supported material semantic defect in this review, but the mandatory boundary gate fails on the merged dirty set. Passing functional evidence does not authorize constitutional closure.

## Reading boundary and provenance

Read-only independent review; no builds/tests or source edits were performed by this reviewer. I read AGENTS.md, every coding-guidelines document, milestone-3 D12, and the approved checkpoint-funding and cold-region-funding design records. I did not read previous reviewer reports, the merge plan, progress note, or report draft.

Compared the material funding/custody/capture conflict clusters against both parents:

- Ours: `25619ac5d1ae6092ab5a91bc4b7123be6042475c`.
- Incoming: `14b16fa292e96ff0f5d96b55bd88c338a290cd87`.
- Reviewed staged candidate: `8a2a4092233c2579f092ed1a3a6823a0cfe1cdb6`.
- Initial qualification compilation tree: `c7e6413c053ef82c98dbccf8bfb3bdad930c6bde`.

Direct Git comparisons confirmed that the only source changes after initial qualification were checkpoint_complete/source.rs and its new private source/tests.rs. Their candidate blobs are respectively `8b7d31271579831de3b066035789e940769b6bfa` and `855db95e12e9c270fd8b821e6cb45a62c1e7b98b`. This delta exhaustively preserves Content request/semantic refusal causes and adds a pure value regression. Post-correction Root Clippy and that rebuilt regression qualify the correction. Source-identity logging agrees with my comparisons. The older logs' premerge HEAD and dirty headers are not represented as clean-commit evidence. Query-extra explicitly reuses the initial Host binary; its scalar controls do not exercise the corrected Content arm.

The boundary metadata operation subsequently updated UI Cargo.lock. The normalized raw audit records zero existing-package removals, four local dependency-row updates, and 41 added resolutions matching already-qualified workspace locks. This is generated lock propagation, not another implementation resolution or a claim that the initial tree contained that lock update.

## Findings

1. **P1 — Five sealed control/construction references prevent merge closure.** `funding-merge-boundary.log` reports BC7012 for Query's affinity_tests/retained_program.rs reference to `test_host_request_for_test`, and four BC7013 references in Store's no_pending_owners courts: `world_recovery_authority` in transition_catalog_readmission.rs; `prepare_fresh_home_schema_installation` and `prepare_native_schema_for_home` in transition_compilation/fresh_installation.rs; and `bind_query_native_proposal` in transition_native_effect/native_original.rs. These callers are outside the closed permitted reach. Direct comparisons show all affected source is changed from ours and exactly inherited from incoming. The merger therefore contains the violations even though funding conflict resolution did not introduce their bodies.

   Contract: AGENTS.md Law 1 reach and exact certification-construction caller rule; MENTALITY section 3 requires mechanical enforcement. Root cause: the incoming caller paths and the merged sealed caller configuration do not form a lawful closed caller set. Consequence: authority construction/control reach fails the required gate, so the complete scoped dirty set cannot be declared complete. Correction requires the responsible Lifecycle/integration owner to remove the unauthorized reach or obtain an explicit boundary ruling and implement its exact permitted path. A wildcard, suppression, automatic allowlist expansion, or bypass is not a correction. Required verification: rerun boundary-check and generated-context checks after the owner correction, plus the affected caller proofs.

2. **P1 — Nine source-court diagnostics in the staged merge violate production test topology.** BC9002 flags these changed-source families: Native home_program_transition/body_digest/tests.rs and catalog_digest/tests.rs; record_identity/partition_frontier/tests.rs; native_checkpoint/allocation/encoded_payload.rs; semantic_result/execution_posture.rs; World's durability/artifact/retention_tests.rs and occurrence_custody/retention_tests.rs; World lifecycle/owner/execution_service/tests/public_ports/released_source.rs; and Bank authorization_denial/instruction_tests.rs. Each is changed from ours and identical to incoming in my direct comparisons. Their runtime/request roots remain in production src courts.

   Contract: Iteration Law 2, Testing Laws 20 and 23, and MENTALITY section 3; AGENTS.md makes constitutional failures present in the complete scoped dirty set completion-blocking. Root cause: incoming source courts were not placed behind the required registered court boundaries. Consequence: the merged artifact fails the mandated topology guard despite green selected behavior. Correction: the named Native queue owner handles its granted migrations; Lifecycle/World/Bank owners handle the remaining court relocation or lawful decomposition, preserving each useful assertion and real boundary. Do not grow the source-court baseline or weaken/delete tests. Required verification: boundary-check, context, dirty line caps, and focused relocated court proofs.

Three additional BC9002 diagnostics are unchanged from both parents: Native cold_source.rs, cold_source/protected_reads.rs, and support/durable_medium/runtime_attachment.rs. They are explicitly identified as existing queued repository debt, not newly attributed to this merge's source changes. The raw gate reports 17 diagnostics total: the 14 changed-source diagnostics in findings 1-2 plus these three unchanged sites. The six explicitly granted Native migrations remain queued; this review neither starts them nor grants ownership over Lifecycle/Bank paths.

## Semantic review and distinguishing evidence

| Risk | Source conclusion and evidence |
| --- | --- |
| Request backing and custody escape after reconstruction/edit | Native backing owners carry their actual request reservations. Head/Full reconstruction, history images/rows, prefix custody and kind installation keep tickets with escaping storage. Relevant denial/retry, InlineBytes, peer-history, shared-custody, and last-owner tests passed. Coarse materialization admission does not replace the incoming storage owners' grants. |
| Selected capture replays covered history or substitutes bytes for authority | The selected base pairs the bound source/image/normalized graph with its real directory. ACK installs the prepared base with the directory; recovery restores it through existing admission. Capture uses current/external held roots, tail-only original demand, and flat covered-region reuse corroborated against the actual cell. Live/cold zero-covered-read, index-tail, retry, Store observer-drop/ACK recovery, and discarded-body growth proofs passed. |
| Maintenance consumes caller budget or replaces business outcomes | The one joined finite Home worker uses its own retained ledger and deadline. Query reaches the closed Host hook after retaining the business result. Explicit checkpoint still takes its caller request. Exhaustion, caller-cancellation trigger, head-41 ordinary requests, close funding/exhaustion, and unchanged bounded-tail filesystem controls passed. |
| Merge weakens outcomes, format cutover, or tests | Content causes remain typed through the corrected mapping. InlineBytes allocation accounting and ContentRef size meaning survive the Foundation merge; retired Bytes has no compatibility alias. Canonical v5, CHK18 and REG6 keep explicit version admission. Relevant tests are retained; the <=8 tail / <=40 body-read assertions and 120-second bound remain. No relevant test deletion/ignore or enforcement weakening was found. |

## Raw qualification inspected

- Root Clippy 03 and post-correction 04: exit 0; the latter retains the reported 33 Native warnings, not a warning-free claim.
- Query workspace check and Store workspace Clippy: exit 0.
- Root courts: 14/0/0; Root units: 4/0/0.
- Query: 9/0/0, including direct default-stack execution with `RUST_MIN_STACK present: False`; Query-extra: 5/0/0.
- Store observer-drop/history/mutation-before-ACK recovery: 1/0/0.
- Rebuilt post-correction refusal regression: 1/0/0. Selected total: 34 passed, zero failed, zero selected ignored. Unselected/ignored suite entries are not counted as selected coverage.
- Formatting and source-whitespace logs: exit 0. Final dirty caps: exit 0, 7,677 Rust files checked, existing foundational facade exemption reported.
- Generated context: exit 0.
- Mandatory boundary-check: **exit 1, 17 unresolved diagnostics**. The owner-audit script's own exit 0 means the audit ran; it does not turn the boundary result green.

No broader qualification is pending for this report. There is adequate scoped functional evidence for the reviewed funding behavior and the narrow correction; there is no approval to label this real merge complete. An explicitly IN PROGRESS local preservation commit must retain that distinction.

**REJECT**