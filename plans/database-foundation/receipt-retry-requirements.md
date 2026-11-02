# Receipt and retry requirement map

Owner: P-D. Specification: `milestone-3.md`, D9/D17 and Phase 4. This is an
implementation/evidence map, not a change to the specification. Pending and
blocked entries are not qualification claims.

| Requirement | Compiler fence / behavior court | Current evidence |
|---|---|---|
| D9 Native receipt, version, position, digest and all original semantic axes | `live_original_publication_reads_exact_native_source_after_a_later_publication`; `receipt_fields::assert_live_agrees` and complete serialized semantic projection | Earlier World source court passes; integrated Host court in progress |
| D9 ordered changed records/count and runtime-neutral Native before/after descriptors | Same World court; full `receipt_fields::projection`; `at_recorded_commit` selects original state after later mutations | Reopen proof blocked on P-A until rerun qualifies it |
| D9 co-committed key, stable intent, product affinity and enclosing Native association | `changed_intent_refuses_after_later_directory_publications`; `inherited_directory_key_refuses_a_sibling_product_affinity` | Focused integrated Host court in progress |
| D9 original World occurrence, old/new heads, attempt and publishing Home incarnation | Exact Native/World owner join; complete original projection before kill and after fresh open | Existing RootLinkage / CompleteNativeRequired blockers, current courts still require the real API |
| D9 outcome identity, emitted count and exact output settlement | Full binding-row projection; interleaved output-role retry compares Native creation correspondences and original settlement slot | New original-owner memory court unrun |
| D9 outbox seal and aftermath causal rows | Original creation-row readers use exact enclosing artifact; recovered wrappers expose persisted fields only | Supported Phase 4 directory mutation records absence; nonempty dispatch/correction admission remains refused under D15 (Phases 3-4) |
| D9 lost response, process kill, reopen, retry before checkpoint | `lost_response_kill_reopen_retry_preserves_receipt_before_checkpoint` | Blocked by Native recovered source synchronization RootLinkage; P-A owns repair |
| D9/D12 same journey after actual checkpoint | `lost_response_kill_reopen_retry_preserves_receipt_after_checkpoint` | Blocked by CompleteNativeRequired; P-A owns Complete reopen |
| D7 stale open selector refuses before effect | `reopened_home_refuses_the_prior_open_branch_selector_before_effect` | Written against real reopen; unrun |
| D9 cancellation/no effect may execute once on retry | `cancelled_directory_key_without_effect_executes_once_on_retry` | Focused integrated Host court in progress |
| D9 original lookup preserves exact request cancellation/work stops | `original_directory_receipt_preserves_cancellation_at_every_consult`; `durable_idempotency_door_preserves_stops_at_every_consult` | Public Host cancellation court in progress; memory cancellation/work court unrun |
| D9 unresolved/retained Native effects never become a fresh effect | Existing pending/preparation/resolution branches retained; recovered receipt data grants no completion/release authority | Real process no-effect and retained-effect cancellation courts written; unrun |
| D9 same-key concurrency | `concurrent_identical_directory_keys_publish_one_original_receipt`; `concurrent_server_identical_keys_publish_one_original_occurrence` | Host court in progress; Server court unrun |
| D9 Server changed intent and request decoration | `server_changed_intent_refuses_without_reentering_the_handler`; `restarted_server_returns_the_original_receipt_after_later_history` | Written against actual Home and public Server facade; unrun |
| D9 Server process restart | `killed_server_reopens_home_and_returns_its_original_query_receipt` | Written against real child death and Home reopen; unrun, P-A may block |
| D9 evidence index is disposable | `interleaved_retry_reads_original_after_discarding_derived_evidence` | Written; owner review additionally found the missing lifetime across history retirement; correction committed; Query court pending |
| D17 receipt constructor consumes concrete owner inputs | Recovered receipt public-positive/private-field/from-live/Performed compile fences; private concrete `QueryReadmitted<OriginalReceiptInputs>` and separate memory specialization | Rustdoc lane unrun |
| D9 ordinary World admission is not transient unavailability | `concurrent_installed_calls_all_retain_admission_until_their_own_return`; unchanged borrowed-preclose test | 13 affected owner-gate tests pass after fixing the preclose deadlock |

The memory-only owner now records a compact, root-free original before World
movement. It retains Native owner-sealed component descriptors, never live
receipt authority. Four focused World original/cutoff courts passed before
commit b84a8953d0; the postcommit trim still needs qualification.

## Complete receipt field projection

| Specification field | Owner source / public recovered projection | Evidence |
|---|---|---|
| Commit receipt/id, version, journal position, artifact digest | Exact Native original artifact; `relational_receipt`, `native_version`, `native_journal_position`, `native_artifact_digest` | `receipt_fields::projection`, exact World source court |
| Create correspondences, changed records, structural/schema/history/change/aspect/validation semantics | `relational_semantic_result` from original Native artifact | Full serialized semantic projection and live/reference comparison |
| Changed record count | Original ordered changed records, validated change summary | Projection and `assert_live_agrees` |
| Live CommitResult, transaction/proposal identity, snapshots, timings/complexity, late interruption | Unavailable on recovered data | Recovered receipt rustdoc type fences; no live constructor |
| Idempotency binding | Original creation row, including all key bytes, intent, affinity and enclosing-artifact association | Changed-intent, transport-key and affinity courts |
| Outbox seal | Original outbox creation row in same artifact | Complete `outbox` projection; memory emitted-effects courts |
| Provider session binding | Unavailable; original publishing Home incarnation exposed instead | No recovered session field; publishing-incarnation comparison |
| Basis descriptor | Original Native before/after recorded components, no snapshot authority | Complete component projection |
| Occurrence, old/new product heads, attempt | Actual exact World original owner record | Complete head/attempt projection; exact source court |
| Outcome identity, emitted effect count | Original Query binding row | Projection and independent live overlap |
| Exact output settlement | Original Query outcome row slot/Absent | Independent live settlement comparison; retained owner output court |
| Aftermath causality | Original Relational causal creation rows | Complete causal projection and independent live overlap |
| Publishing runtime | Actual original Home identity/incarnation in World occurrence | Complete projection; no reopening runtime substitution |

`receipt_fields::projection` is captured before real process death and compared
byte for byte after actual reopen; its live overlap assertions independently
check the supported original fields. A blocked reopen does not qualify that
comparison. Memory-only projections explicitly have no Home identity, journal
position or publishing Home incarnation.

D9 explicitly retires synthetic process-cache retry authority and historical-only
receipt fallback. The old Server mock-workspace cache assertions are replaced by
an explicit missing-Query-owner refusal (zero raw effects), while the actual Home
Server courts require the positive original-receipt behavior. Live-only retry
session/snapshot/timing/output-authority assertions are retired by D9's receipt
table and replaced with original semantic/settlement data plus compile fences;
no test is ignored or deleted to conceal a failing supported behavior.

The public Host native-input-guard and mutation-attempt-report retry courts now
expect `RecoveredCommitted`, per D9's clean cutover, and compare the same
original outcome/commit against the independent held live receipt. Their graph,
audit, invocation, revision and no-resurrection assertions remain intact.


## Response-loss migration and exact resource courts

D9's live-only row retires the response-loss first response's reconstructed
terminal, timing, precondition-observation and preimage receipt claims. Those
courts now require `RecoveredCommitted` on the first internally resolved lost
response as well as later retries, and compare the complete recorded value.
Actual preimage/dispatch custody is still checked independently through its
own already-held owner observations; receipt data never mints a live receipt.
The terminal/session/snapshot cleanup counts remain real owner observations.

The interleaved snapshot court measures both started attempts and checks exactly
one peer remains AFTER `finish_application_commit`, where progressed.rs releases
lease custody and mutation cleanup. Its former pre-finish fixed count described
a temporary implementation phase, not release. Final baseline, zero provider
resources and released-attempt evidence remain mandatory.

The checked-rule exact-work court keeps its independent zero-body control,
authored delta and exact total ceiling. One total unit short now stops at the
funded original capture AFTER Native settlement: exact
`ProductUnpublished(OriginalObservationStopped(WorkExhausted))`, one owner effect,
inspection/obligation custody and zero World/Query publications are required.
It cannot honestly claim Native no-effect or resume a weak owner after the
fixture has dropped. The consult oracle's explicit healthy budget covers the
original canonical artifact hash; every cancellation/work boundary stays checked.
