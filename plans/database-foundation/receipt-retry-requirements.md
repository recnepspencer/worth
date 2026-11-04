# Receipt and retry requirement map

Owner: P-D. Specification: `milestone-3.md`, D9/D17 and Phase 4. This is an
implementation/evidence map, not a change to the specification. Pending and
blocked entries are not qualification claims.

| Requirement | Compiler fence / behavior court | Current evidence |
|---|---|---|
| D9 Native receipt, version, position, digest and all original semantic axes | `live_original_publication_reads_exact_native_source_after_a_later_publication`; `receipt_fields::assert_live_agrees` and complete serialized semantic projection | Two exact World source courts pass (`world-original-source-courts.log`); complete reopen comparison remains blocked on P-A |
| D9 ordered changed records/count and runtime-neutral Native before/after descriptors | Same World court; full `receipt_fields::projection`; `at_recorded_commit` selects original state after later mutations | Reopen proof blocked on P-A until rerun qualifies it |
| D9 co-committed key, stable intent, product affinity and enclosing Native association | `changed_intent_refuses_after_later_directory_publications`; `inherited_directory_key_refuses_a_sibling_product_affinity` | Changed intent after later publications passes; sibling creation refuses DurableBranchPublicationRequired before resolution |
| D9 original World occurrence, old/new heads, attempt and publishing Home incarnation | Exact Native/World owner join; complete original projection before kill and after fresh open | Existing RootLinkage / CompleteNativeRequired blockers, current courts still require the real API |
| D9 outcome identity, emitted count and exact output settlement | Full binding-row projection; interleaved output-role retry compares Native creation correspondences and original settlement slot | Memory original/settlement owner evidence in `direct-original-owner-qualified/run.log` and `bounded-cleanup-receipt-qualification/run.log`; funding and actual-claim owner courts pass in `query-funded-original-claim-owners/run.log` (12 pass, 0 fail, 1 ignored inventory entry, none executed) |
| D9 outbox seal and aftermath causal rows | Original creation-row readers use exact enclosing artifact; recovered wrappers expose persisted fields only | Supported Phase 4 directory mutation records absence; nonempty dispatch/correction admission remains refused under D15 (Phases 3-4) |
| D9 lost response, process kill, reopen, retry before checkpoint | `lost_response_kill_reopen_retry_preserves_receipt_before_checkpoint` | Blocked by Native recovered source synchronization RootLinkage; P-A owns repair |
| D9/D12 same journey after actual checkpoint | `lost_response_kill_reopen_retry_preserves_receipt_after_checkpoint` | Blocked by CompleteNativeRequired; P-A owns Complete reopen |
| D7 stale open selector refuses before effect | `reopened_home_refuses_the_prior_open_branch_selector_before_effect` | Real reopen currently refuses RootLinkage before stale-selector admission |
| D9 cancellation/no effect may execute once on retry | `cancelled_directory_key_without_effect_executes_once_on_retry` | Public filesystem court passes in `public-host-original-receipt-qualified/run.log` |
| D9 original lookup preserves exact request cancellation/work stops | `original_directory_receipt_preserves_cancellation_at_every_consult`; `durable_idempotency_door_preserves_stops_at_every_consult` | Memory full cancellation/work sweep passes in 278.631 seconds (`bounded-cleanup-receipt-qualification/run.log`); corrected public Host callback court passes in `host-normal-and-original-cancellation/run.log` (312 passing ordinary cases, 20 separate failures) |
| D9 unresolved/retained Native effects never become a fresh effect | Existing pending/preparation/resolution branches retained; recovered receipt data grants no completion/release authority | Public directory pending and post-Native unwind courts pass; kill/reopen variants remain blocked by Native RootLinkage |
| D9 same-key concurrency | `concurrent_identical_directory_keys_publish_one_original_receipt`; `concurrent_server_identical_keys_publish_one_original_occurrence` | Host concurrent court passes (`host-directory-receipt-owner-fixes/run.log`); Server concurrent court passes (`separated-intent-server-receipt-matrix/run.log`) |
| D9 Server changed intent and request decoration | `server_changed_intent_refuses_without_reentering_the_handler`; `restarted_server_returns_the_original_receipt_after_later_history` | Both positive public Server behaviors pass in `separated-intent-server-receipt-matrix/run.log` |
| D9 Server process restart | `killed_server_reopens_home_and_returns_its_original_query_receipt` | Real child-death/reopen court refuses RootLinkage (`separated-intent-server-receipt-matrix/run.log`) |
| D9 evidence index is disposable | `interleaved_retry_reads_original_after_discarding_derived_evidence` | Derived-index disposal is qualified by the Query correction lane; actual memory history retirement and later original reads pass in the public World ordinary lane |
| D17 receipt constructor consumes concrete owner inputs | Recovered receipt public-positive/private-field/from-live/Performed compile fences; private concrete `QueryReadmitted<OriginalReceiptInputs>` and separate memory specialization | `query-original-constructor-fences.log`: 4 pass, 0 fail, 0 ignored (1 positive and 3 compile-fail fences) |
| D9 ordinary World admission is not transient unavailability | `concurrent_installed_calls_all_retain_admission_until_their_own_return`; unchanged borrowed-preclose test | Owner normal target 307 pass; public ordinary courts 63 pass/2 corrected failures, then those exact 2 pass; no test-side serialization |

The memory-only owner now records a compact, root-free original before World
movement. It retains Native owner-sealed component descriptors, never live
receipt authority. Four focused World original/cutoff courts passed before
commit b84a8953d0. The compact owner and public history-retirement courts subsequently
pass in the World owner/ordinary evidence logs.

## Complete receipt field projection

| Specification field | Owner source / public recovered projection | Evidence |
|---|---|---|
| Commit receipt/id, version, journal position, artifact digest | Exact Native original artifact; `commit_reference()` (commit/version), `native_journal_position()`, `native_artifact_digest()` | `receipt_fields::projection`, exact World source court |
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
comparison. Memory-only projections explicitly have no Home identity, World journal
position or publishing Home incarnation. Their Native positions describe actual
resident frames and do not claim persistent backing.

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

The active performing attempt receives its actual original completion through a
funded, session-bound one-shot slot, including an internally recovered lost
provider response. Its real preimage, dispatch and native cleanup owners remain
available. A later keyed call receives only durable original receipt data and
cannot manufacture those live owners. The response-loss courts retain the actual
first completion and compare later recorded meaning against it. The slot and
cleanup envelope are reserved before effects; cleanup failure moves the genuine
pending original into that same funded storage. Rejoining refused cleanup reuses
the envelope retained by its unique native owner.

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

The authority-evidence work oracle now ends after its one-way DecisionReadSet
phase has been traversed. It keeps every authority revalidation charge and its
exact by-value resource cause; unrelated later Native/capture charge prefixes
belong to the existing Native/source-stop and new retained-effect capture courts.
Traversing every later full commit did not strengthen the authority claim and
timed out at the committed 600-second test limit during the first normal run.
The original positive whole-commit control remains, and an observed post-Native
stop still asserts its exact retained-effect fate.

Concurrent unseen mutation requests may prepare pure candidates before the branch
commit lane (Milestone 3 D9 and Phase 4 concurrent retry requirement; declaration
`WorthQueryPreparedProgramMutation`). The earlier court's handler-count-equals-one
assertion incorrectly treated speculative `decide` calls as committed effects.
The corrected courts demand exactly one fresh completion, equal original receipts,
one actual additional World history entry, and no handler entry on subsequent
sequential replay. Lost-response/reopen/restart courts keep unchanged-handler checks.

## Closed consumers and lifecycle observations

D9/D17 and commit `98b39a07bd` retire retry assertions that synthesize new live
phase/session owners. Bank lifecycle retries now compare complete original data
through fresh authorized lifecycle observation; each actual fresh predecessor
moves once into its next phase. Approval, close and review outcomes retain the
actual supplied predecessor on recorded completion. Their previous success
getters and fictitious replay work counters are replaced by original data and
unchanged World publication assertions; fresh canonical work ceilings remain.
The former CommittedReceiptNotRetained HTTP recovery instruction is deliberately
retired by D9; unverifiable persisted intent still requires operator action.

Bank closed source and capability values retain the entire native close tuple.
HTTP collection release retains its actual receipt; descriptive wire replies do
not discharge its original dispatch/recovery obligation. Closed native evidence
whose dispatch posture is Pending, AttemptedUnresolved or Observed stays owned.
Its authenticated local handoff transfers the complete carrier and original
runtime, restores exact custody and capacity on lost delivery, and releases the
worker slot after confirmed transfer. No pending owner is reconstructed from
recorded data. Native NotDeclared/TerminalObserved and absent-owner closes may
release ordinary transport records.

The unit test `a_key_committed_before_a_restore_asks_for_a_refresh_not_a_retry`
is deliberately removed: it asserted the retired `CommittedReceiptNotRetained`
request denial. D9 and commit `98b39a07bd` delete that historical-only outcome;
there is no equivalent replacement request-denial variant. It must not be
renamed to foreign affinity or weakened to another cause. Real foreign-affinity
and restore/reopen courts remain separate; their blocked entries above are not
qualification claims.

The former combined test `a_committed_key_asks_for_a_refresh_and_an_unverifiable_one_for_the_operator`
is now `an_unverifiable_recorded_intent_asks_for_the_operator`. Only its
`CommittedReceiptNotRetained` assertion is deliberately retired by D9 and
`98b39a07bd`; the unverifiable-intent operator requirement remains exact.

Recorded recovery delivery retains a distinct wire shape from live completion.
`admitted_original_status_tracks_only_its_actual_provider_and_world_claim`
checks actual Unclaimed/Live/Disposed custody and exact provider/World isolation.
The existing foreign-admission and revoked-capability courts additionally prove
that this new observation refuses before disclosure. The HTTP finite-capability
court obtains a real token and Recorded owner before expiry, then demands
PermissionDenied at physical close after expiry; it may expose neither token nor
receipt. The exact claim and admission courts pass in `query-funded-original-claim-owners/run.log`; finite-capability disclosure and both token-replay cases pass in `bank-current-disclosure-and-workflow/run.log` (its two workflow-owner failures are separately tracked).

The inbound-completion example deliberately retires its former assertion that a
held original becomes `ExactCommitUnavailable` when the mandatory receipt-window
pin retires. D9 and `97abf4065e` preserve the actual closed owner independently
of that derived window. The example now requires the original owner-backed
outbox read and compares correlation family/token, effect, protocol/version,
payload bound/bytes and Native receipt/version against the actual dispatched
request and held original. Its independent callback and exactly-one rail-effect
checks remain unchanged.

The explicit workflow owner acceptance/recovery APIs now require the genuine live
receipt; Bank wrappers require its privately constructed performed operation.
D9 and `98b39a07bd` retire key-only reconstruction of DispatchPending/Performed
custody. The former external-rail retry `into_performed()` and drop-before-mint
assertions are replaced by original Native/World correspondence, recorded data,
no second transport contact, and the retained first owner. All prior exact rail
count/payload, request-opening, first-reader, zero-work and actor-role assertions
remain. The ordinary owning workflow fixes an actual lost handoff by lending its
existing completion.live_receipt(), as its inspection and acceptance paths do.
Host workflow_owner_contract includes positive Live and negative Data type proofs.

The conventional workflow input correction derives retained governed identity
from `admission.governed_input_identity()`, as fresh recovery admission does.
Original encoded handler input remains separately retained for correction and
disclosure; its digest does not assert capability-governed authorization.
The existing `governed_input_mismatch_drift_denies_distinctly` refusal remains.
The new public `recorded_workflow_retry_closes_without_disposing_original_dispatch_custody`
court closes a duplicate descriptive wrapper, preserves the genuine original
recovery door, compares both World owner and ordinal, and requires one actual
rail effect/contact. Both workflow request-phase courts pass in `bank-workflow-governed-input-correction/run.log` (2/0/0). The duplicate-close court passes in `bank-recorded-workflow-qualified-courts/run.log`; all ten subsequent consumer failures pass in `bank-recorded-consumers-correction/run.log` (10/0, one ignored inventory entry, not executed).

Definition-publication and condition/approval retry courts retain the genuine
first publication/transition, then demand Recorded original data. The common
comparison responsibility is `recorded_commit_axes.rs`;
it preserves the former approval court's exact commit, outcome, key, changed
count, emitted count, output settlement and every memory World-head/attempt axis.
ReadyPaymentWorld declares the held definition before its runtime fixture so it
drops first. D9/98b39a07bd deliberately retire the old second-keyed Published
definition and Completed-transition assertions; genuine local settlement
confirmation with a supplied Live owner remains supported and still checks its
replayed posture. No ignored tests execute; changed Docker callers are compile
coverage only. The Bank selected normal/affected court run passes 172 cases; its ten failures subsequently pass in `bank-recorded-consumers-correction/run.log`. Query Publication and affected input/recovery owner courts pass 54/0 in `query-workflow-binding-owner-qualification/run.log`.

Bank's keyed correction and retained-read courts deliberately replace their
second-call Live Recovered/AlreadyCommitted assertions with Recorded original
data (D9/98b39a07bd). Their original retained-read observations, exact commit
selection, causal source/current capability checks, account balances, journal
contents, rail counts and normal closes remain. The personal-account court
compares the durable assigned create correspondence and exact output settlement
against its independently held first output; Data exposes no outputs_of authority.
The foreign-runtime outbox court now requires the actual ForeignRuntime owner
refusal before reading another owner's Native commit, while its positive full
outbox payload/protocol/correlation court remains unchanged.
Bank `historical_original` forwards Data to the existing Query
`at_original_receipt`, retaining fresh admission, work/result bounds and typed
denials. The original live historical method remains for genuine live receipts.
The payment correction source is borrowed from the actual retained operation's
fresh inspect_source view; its duplicate answer is only compared as Data.

Workflow start, cancellation, proposal and approval replay courts retire their
former second-call Performed projections and replay booleans under D9 and
98b39a07bd. A first-only typed `started`/`cancelled` match proves genuine fresh
provenance; later answers require RecoveredCommitted and compare Native receipt,
outcome, key, changed/emitted count, settlement, World head and attempt against
that first owner. Original cancellation performed paths and approval actor,
evidence, purpose and expiry remain asserted on the actual first owner. Duplicate
Data cannot claim those live getters. Original assigned create correspondences
preserve transition/instance identity. Input encoding remains exactly once on
both requests; the original admission retains its exact canonical-work assertion,
while a Data receipt deliberately exposes no new-attempt canonical work.
Actor courts retain their genuine first proposal for the existing original-field
decoder; generation courts explicitly drop it before actual retirement and slot
reclamation. The ordinary reference-returning fixture delegates to the same real
post-proposal assessment/join/approval progression and gains no new authority.
The normal workflow court run passes 70 cases with one remaining lineage helper failure. Its targeted successor will qualify that helper while retaining all positive-charge, finite-ceiling, warm/cold no-new-evidence and fork-inheritance assertions.


## Exact late consumer and resource corrections

D9/98b39a07bd retire duplicate correction `AlreadyCommitted` and live-Source
claims. The original authorship, causal source, current authority, unchanged
head, explicit active-reversal consumption and required-output delivery remain
asserted. Tree reapplication closes the actual Source and retains the whole
closed carrier; only a genuine first Fresh/Recovered completion yields a live
receipt. The same-key path requires Recorded data. Required-output tests retain
the actual first performed result across cancellation and foreign-host denial.

The lineage evidence spender now returns the genuine first transition beside
its settlement/key instead of discarding it into a replay boolean. Warm/cold
retries require original data and compare it against that retained owner.

The temporal exact-budget court retains paid/exact/one-short calibration.
Its last unit stops at original capture after Native settlement, so D4 and
Phase4 custody require exact ProductUnpublished OriginalObservationStopped
WorkExhausted, one owner effect, live obligations and Inspect. Its Case keeps
the actual application alive after the outcome; inspect() must reach that owner.
The prior Denied/no-effect claim was deliberately retired by the original-capture
cutover in 13a09db9c1, not hidden by increasing the budget. The narrow facade
entry is existing descriptive ProductUnpublishedCause.

The shared-component sibling court keeps equal Native descriptors, real sibling
branch and security checks, requiring the exact earlier IdempotencyForeignAffinity
family instead of IntentDrift. Fresh Home's retained runtime clone is refused
by Busy(ProgramRuntime), before sealing; the court verifies ordinary reads still
work and retains its final successful owning-close expectation. That final step
remains an enabled Source-lifecycle blocker if Handler Unsupported is returned.

D9/98b39a07bd also retire duplicate composite-definition live Signal-publication
and conditional-generation getters. The actual original still requires Published
Native and Signal postures, a Signal publication and conditional generation,
changed predicate, and a real due operation. The duplicate now compares full
recorded Native/outcome/settlement and every World-head/attempt axis against that
original plus the exact causal source; it cannot supply new Signal authority.
The inline byte court keeps its historical certified observations and exact
byte assertions, closing the actual duplicate Source and comparing Recorded data.
The retained-clone legacy conditional-close gate is RuntimeKeepaliveRetained
before the inner Home-lifecycle gate, per program_runtime.rs::close_conditional_runtime.

The workflow normal lane's 70/1 is completed by both targeted lineage cases in
`host-workflow-recorded-retry-corrections/run.log` (12pass/3fail overall; the
other failures are separately corrected inline callers and final Home close).
`host-inline-causal-original-retry-qualified/run.log` passes both inline cases
and leaves the final Home close enabled/red at Handler Unsupported. The
original-attempt comparison also checks its owner, not only its ordinal.
Canonical Native descriptor agreement excludes the process runtime/root identity
that component_capture deliberately removes; logical descriptor version, branch,
reference generation, truth version, schema, visibility and posture all agree.

Qualification at5cf0287c8b: original-attempt-owner-axes-qualified/run.log passes
all four selected owner/basis/directory/lineage cases, zero failures, two ignored
inventory entries unexecuted. Phase-completion fmt for all27 touched Rust owners,
dirty caps and full base-to-HEAD cap audit (565 Rust files), boundary and generated
context checks all pass. Logs: receipt-completion-{format-root,format-query,
format-bank,format-store,caps-dirty,caps-range,boundary,context}.log.
Independent final review remains pending; this is not a Phase4 exit claim.
