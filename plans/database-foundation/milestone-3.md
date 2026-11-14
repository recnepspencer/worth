# Milestone 3: Durable Application Runtime

Governing roadmap: [Database Foundation Roadmap](roadmap.md#milestone-3-durable-application-runtime).
Builds on: [Milestone 1](milestone-1.md) and [Milestone 2](milestone-2.md).

An application opens one file-backed home through one Query host call. Every
supported runtime owner publishes its own durable artifacts through Store,
recovers through its own readmission, and continues working after process
death. That covers graph state, application occurrences and their original
results, Signal and Bridge definitions, programs, workflows and their
obligations, external custody and corrections. A retry after a lost response
returns the original occurrence. Nothing an application was told is committed
depends on process memory, orderly close or application-owned byte transport.

This milestone is delivered in two tracks:
- a **happy track** (H3–H10), with every application-visible capability working across a graceful close and reopen;
- a **cleanup track** (C1–C5), with crash and fault recovery, cost bounds and certification.

Both tracks finish inside this milestone. The ten numbered phases below still state the requirements, design, tests, oracle, documentation and exit criteria. **Delivery tracks** decides which track each requirement and test belongs to, and when it lands. The cross-cutting decisions below bind every phase. D17 decides how each invariant is enforced and where each new type lives.

## Delivery tracks

**Authority.** Where a decision or phase section conflicts with this section on *timing* or *track*, this section governs. It never weakens a requirement. It only says whether the requirement is proven before the happy landing or before the milestone closes.

### Landings

| Landing | Contents | State |
|---|---|---|
| 1 | Phases 1–2 | Landed |
| 2a: the first happy landing | H3–H5 and the coherent H6–H9 slices already merged; every other partial surface refuses with a typed reason | Integration branch `codex/m3-host-home`; final gate waits for H5 LANDED |
| 2b: the remaining happy landing | Complete H6–H10 and remove each corresponding temporary refusal | After landing 2a |
| 3: the cleanup landing | C1–C5; closes the milestone | After landing 2b |

- **Landing 2a reaches master** after H5 LANDED, a reviewed batch merge, green affected Tier-2 lanes, one green final Tier-3 run and the coordinator's final certification. Its coherence ledger names a graceful court for every Supported surface and a typed, before-effect refusal court for every Refused surface. No PARTIAL row reaches master. Landing 2b completes the remaining happy requirements under the same coherence rule.
  - A CLEANUP court that is red at that moment leaves the tree, with its restore reference listed under its C item ("Moved courts").
  - It is never `#[ignore]`d, gated or weakened.
- **Phases 5–9 lift their own D15 refusals** only for the families their merged graceful courts prove. The ledger ties each temporary H6–H9 family refusal to the H-phase that removes it in landing 2b; C-track refusals retain their C-phase removal obligation. H10 deletes `UnsupportedDurableFamily`.
- **Consumers may build on landing 2a's Supported surfaces.** They support graceful close and reopen, and a kill between operations. Its Refused surfaces stop with their named typed reason. After an unclean kill, open refuses with a typed reason (D18).

### The happy constraint (track H)

A home is closed gracefully, or killed while no operation is executing (a *quiescent kill*). It is then reopened any number of times, in a fresh process, under the same or a changed application build identity.

Each reopen:
- recovers every owner's full acknowledged state;
- answers every retry of an admitted key with its original occurrence or its typed no-effect posture, never with a second effect;
- never reissues a committed identity;
- serves only after its D8 typestate steps complete.

Open custody retained at close is readmitted as recovered custody (D5 and D11 below), and work continues on it.

**D18 (new, track H only): an unclean open refuses.** When the journal tail holds an associated owner effect with neither an occurrence nor a disposition, open refuses before service with `UncleanRecoveryUnsupported`. The refusal names the owner and the effect, and the tail scan finds it. It never guesses a fate. C5 deletes this refusal once C1 recovers that case.

### Decision splits

| Decision | Track H | Track C |
|---|---|---|
| D4 | Owner publication order; product movement as its own step | `Indeterminate` fates |
| D5 | Association fields; disposition before live; recovered custody across graceful reopen (integration's recovered-custody service) | `OpenAttempt`; reconciliation after an unclean kill; pending settlement |
| D8 | All typestate steps; refused openings expose no service | — |
| D11 | Close as written; `CloseRefused { posture }`; retained custody readmitted at reopen | — |
| D12 | The checkpoint is taken **only at a quiesced close** (no executing attempt, no external obligation); the explicit application checkpoint is allowed under the same condition | Automatic threshold triggers; exact Native component obligations under concurrency; the cross-owner capture cut while attempts execute; proportional reopen and backing budgets |
| D13 | One interim door (`WorthQueryInterimCheckpointRestore`), frozen callers, no `in_memory` names, the authenticated World-original section | Deleting the door (C5) |
| D14 | — | All of it |
| D16 | Plan, intent, step-body fence, carrier, stable identity allocation, Host transition request, final promotion and binding | Resume from the first nondurable step; `TransitionPlanDrift`/`TransitionPending`; discard tombstones; kill courts |
| D1–D3, D6, D7, D9, D10, D15, D17 | In full | — |

- **Format stability is relaxed for track C.** A cleanup item may change a format landed in track H. The old format then refuses with a typed reason and is never migrated.
- **Track H keeps clean cutover.** No compatibility shims.

### Track H items

Each H item's proof is its phase's graceful journeys, plus the graceful oracle.

- **H3 (Phase 3): Home and host lifecycle.**
  - One application home through the host open call. `RelationalStoreHome` and every `in_memory*` path are deleted.
  - Newness, `initial_state` and `discard_home`.
  - Descriptor binding and rebuild; D16's track-H scope.
  - The Empty Signal/Bridge seed basics.
  - Contention, incarnation fencing and close settlement; the frozen callers.
  - The Memory suites; D18.
  - **Graceful oracle:** port the M2 `relational_home` oracle, plus a minimal memory-only reference host with `DurableIdentityCorrespondence`. It compares owner state after graceful close/reopen and quiescent kills, with no fault injection.
- **H4 (Phase 4): Durable occurrences and original-result retry.**
  - World artifacts and home-scoped identity; durable before live; rebuild from tails.
  - Artifact v3 association and the semantic result; the D9 receipts table.
  - Deleting the evidence authority, `CommittedReceiptNotRetained` and the Server caches.
  - Intent drift and affinity refusals.
  - The decisive journey, retry refusals, receipt fields and concurrent same-key retries.
  - History with no identity collision; Server restart.
  - Released keys keep their meaning across reopen.
- **H5 (Phase 5): Signal and Bridge authored state; mixed publication.**
  - The authored/derived split and codec; the wasm port; artifacts and association.
  - Identities and foreign-owner refusal; multi-branch readmission.
  - The evidence tag; Bridge and stale-under-build; R2–R5.
- **H6 (Phase 6): Programs, adoption and build identity.**
  - Support lifecycle; activation on every branch.
  - Adoption custody across graceful reopen; build identity.
  - Retirement; unrostered and changed builds. This restores the producer directory court.
- **H7 (Phase 7): Workflows, conditional operations and long-running work.**
  - Workflow records and resume across reopen; transition dispositions.
  - The managed cleanup obligation; R9 durable and process-local.
  - The temporal key; managed and owned-async custody; the R7 route table.
- **H8 (Phase 8): External dispatch and custody.**
  - Outbox rebind; the attempt recorded before the call; `PayloadRejected`.
  - Inbound custody and dedup; recovery-handle claims; rebuilt indexes.
  - Deleting `StoreCapabilityRequired`; R13.
  - An outstanding dispatch across a graceful close.
- **H9 (Phase 9): Correction and original-owner advancement.** Phase 9's graceful paths, with every kill read as a close or a quiescent kill; durable history and branch retirement belong to C2.
  - Durable original-interpretation membership is published at the acknowledged
    World-open seam and jointly readmitted in
    `application_installation/home_preparation/world/custody.rs`. Its funded
    terminal disposal uses the protected World checkpoint source route in
    `durability/checkpoint/original_observation.rs`, for released unpublished
    originals only. Disposal of retired and inherited occurrences, together
    with the durable history and branch retirement protocol, belongs to C2.
    `AuthoritativeDurableHistory` and `DurableBranchPublicationRequired` are
    Refused rows on track H. H6 keeps original-meaning
    interpretations process-local and counts each retained meaning once; it
    transfers none to durable membership.
- **H10 (Phase 10): Public cutover.**
  - Callers and examples on the open call. They run on their genuine main thread under the default stack in the dev profile. Entry-point stack growth through `stacker::maybe_grow` satisfies this main-thread requirement.
  - The guides; deleting `UnsupportedDurableFamily`.
  - A graceful full journey across every family.
  - Documentation that states plainly that crash recovery is not yet certified.

### Track C items

- **C1: Crash core.**
  - `OpenAttempt`; reconciliation; recovered publication after an unclean kill.
  - Exact Native components; lazy discovery; unresolved fates and `ProvenNoEffect`.
  - Transition resume and drift; discard completion and tombstones.
  - The Phase 3 refusal matrix; the damaged descriptor; M2 unresolved forks.
  - Crashes between commit and product movement; the release-kill case.
- **C2: Cost and bounds.**
  - Automatic checkpoint triggers; the cross-owner capture cut.
  - Proportional reopen and request-funded cold materialization. The funding already done on P-A's branch lands here.
  - Backing and work budgets; growth courts.
  - Durable World history and branch retirement, including disposal of retired
    and inherited occurrences. Track H retains funded disposal of released
    unpublished originals.
  - Close maintenance's 60-second wall-clock deadline is a timing trigger.
    Graceful-close courts must not depend on it; never raise the deadline or
    retry until green to certify them.
  - **A frame and future size budget lane.** No `worth_*` function frame over a declared debug-profile ceiling, measured from the compiled binaries' unwind metadata. Reduce the large-frame census found on 2026-11-06 to that ceiling.
- **C3: Owner crash cases for Phases 5–9.**
  - Crashes between owner effects; damaged frames and basis mismatch; mixed kills.
  - Adoption killed at each stage; mid-step workflow kills; R9 `Indeterminate`.
  - Crashes after dispatch; R14; disappearance mid-dispatch.
  - Split completion under a true kill; discarding derived caches (D14).
- **C4: The court oracle.** The adversarial constraint in full:
  - the media sweep, kills at every durable I/O boundary, and recovery interruption;
  - the build and derived dimensions;
  - the injected-fault and interrupted-publication courts held in the cleanup ledgers.
- **C5: Retirement and certification.**
  - Delete the D13 door and the D18 refusal.
  - Boundary gates at zero; re-audit; no deleted callers.
  - Tier 3 and the cumulative oracle; milestone certification.

### Moved courts

- **C3:** cold discharge of an H7 process-local application-resource obligation whose provider/queue/Native release was outstanding at quiescent kill; keep its original input/request and typed pending record; never release it against fresh empty resources.
- **C3:** cold discharge of an H7 process-local application-resource obligation whose provider/queue/Native release was outstanding at quiescent kill; keep its original input/request and typed pending record; never release it against fresh empty resources.
- **C3:** cold discharge of an H7 cleanup obligation whose process-local provider-memory, queue or Bridge release was outstanding at a quiescent kill; reopen retains the typed pending obligation and close refuses until an owner-defined cold release contract lands (coordinator ruling 2026-11-13).

Cleanup courts moved out of the tree are listed under their C item, with `restore from <sha>:<path>`. Existing entries:
- **C1:** the three transition-source recovery courts. They are listed in D16 under "Cleanup track: moved courts".
- **C2 (cost and bounds): exact deduplicated physical accounting of nested persistent aspect backing.** No court has left the tree yet; the claim moved. Pending: 24 exact-accounting and cost-equality courts are still red after the master merge and are not yet classified court by court (exact-equality courts move here with a restore reference; bound-at-least-actual courts stay and must pass); and the full-inventory totals (`RecordArena::allocation_inventory`, `PartitionState::allocation_inventory`, the fork-materialized total in `branch/root.rs`) still saturate instead of refusing, which is open repair work until the coordinator accepts it as C2. `restore from 171d380829:crates/worth-relational/src/inspection/mvcc/sharing_authoritative_byte_metrics.rs` (lines 6–11, 25–28) and `restore from 171d380829:crates/worth-relational/src/inspection/mvcc/sharing_byte_metric_scope.rs` (lines 7–11, inspection version 6). Claim: `unique_physical_*` sums each distinct allocation exactly once, and shared payloads keep node and value granularity, including persistent aspect nodes and admitted values shared between distinct Extra carriers. Inspection version 7 reports that backing as a checked per-carrier upper bound that may count shared structure once per holder, and refuses an unrepresentable bound with a typed overflow. C2 supplies an allocation visitor that sees through shared nodes and values, restores the exact wording under a new version, and adds the court: two Extras sharing one aspect map contribute the shared backing once, and the rows sum to an allocator-observed union. C2 also makes full region inventories and reclamation counters refuse an unrepresentable bound instead of saturating.
- **C2:** `reclose_before_any_image_demand_keeps_the_selections_the_image_carried`
  (restore from `10d117085f:crates/worth-relational/src/tests/durability/medium/complete_index_selections.rs`).
  Restore its carried-only fixture mode from
  `10d117085f:crates/worth-relational/src/tests/durability/medium/cold_index_fixture.rs`.
  C2 owns `a deferred index image is not read on re-close`. The correctness
  claim that selections survive re-close is already covered through public
  owners by Query's `reopening/reclose.rs`; no deferral observer is added here.
- **Phase 6 / H6:** the producer directory court (Phase 6, Required tests).
  - H5 note: a Directory Home with installed producers now refuses at preparation with `ProducerCleanupCustody` (family table: phase 7), because Query refuses Home retirement while producers are installed. Restoring the producer directory court needs that retirement lift (or an earlier family) first.
- **C1:** the recovered Signal-only and mixed original continuation after an unclean reopen (`recovered_owners` court). It never landed: D18 refuses that reopen (proven by `owner_evidence/unclean_signal.rs`). Preserved without activation; restore the source from `a3e8d0fd340aedddaa6398d3ec874d198692799f:plans/database-foundation/moved-courts/c1/recovered_owners.rs.txt` into its original court path when C1 supports that continuation.
- **C1:** Adoption cancelled after Native effect, before World occurrence: BranchAdoptionRecovery continuation across reopen (D18). Typed refusal court: `workspaces/worth-query/crates/worth-query-court-decl-host-package-archive-replay-application-graph/tests/worth_query_court_decl_host_package_archive_replay_application_graph/certification/application_graph/adoption/directory_refusal.rs` (H6 current base `0c6e9e0e4dfb645c1235b01774c65f81003fe8fd`). Close refuses `UnresolvedBusinessCustody` and returns a usable Home; this is not a successful graceful close. No graceful close may replace this abandoned prefix.
- **C1:** `process::host::durable_retry::fate_reopening::cancellation_after_native_effect_retains_key_across_kill_and_reopen` and its retained-writer/reader helpers. D18 (`d27064d826`) deliberately refuses this unresolved owner-effect reopen before service; the court expects service and original continuation. Restore the named court and helpers from `f15ff306ecec9ca4a67be1091269d8d10898ab81:workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/process/host/durable_retry/fate_reopening.rs`. The independent proven-no-effect quiescent-reopen court remains active.

- **C1: Inbound completion after Native effect but before World occurrence (D18; D1808 track classification).** Restore the **complete, unchanged module** from `6c64705f06a59312917403df2fac0218f2f60f77:workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/host/application_home/dispatch_custody/inbound_custody/unpublished.rs` when C1 supports this original continuation. All four entry tests belong to this obligation:

  ```text
  host::application_home::dispatch_custody::inbound_custody::unpublished::an_unpublished_authenticated_completion_continues_its_original_after_required_home_reopen
  host::application_home::dispatch_custody::inbound_custody::unpublished::an_unpublished_authenticated_completion_fences_new_effects_until_original_recovery
  host::application_home::dispatch_custody::inbound_custody::unpublished::an_unpublished_transport_completion_continues_without_fabricating_a_live_dispatch_after_reopen
  host::application_home::dispatch_custody::inbound_custody::unpublished::an_unpublished_transport_completion_fences_new_effects_until_original_recovery
  ```

  Preserve the module's complete helper closure: `authenticated_completion`, `transport_completion`, `complete_original` and `assert_new_effect_fenced`. Restore `mod unpublished;` in the parent `host/application_home/dispatch_custody/inbound_custody.rs` and only the original two exact owner/caller rows from the same immutable commit's `tools/boundary-check/config/control-callers.txt` (Query execution and World owners, both naming this module). The existing `inbound_custody/fixture.rs` supplies the actual Directory owners, route, transport, terminal observations and `end_live_home_for_required_reopen`; that shared fixture and the ordinary graceful/empty-original callers remain active and unchanged.

  Both completion setups cancel at the real product-compare boundary **after the Native effect and before World occurrence submission**. Their shared end helper requires close refusal, resumes the same Home and drops it; this is not a successful graceful close. The actual D1806 run at clean `6c64705f06a59312917403df2fac0218f2f60f77` failed both `fences_new_effects_until_original_recovery` entries at D18 before service (`UncleanRecoveryUnsupported`, owner Relational, distinct exact effect digests). The other two entries were not run in that selection; their classification follows the same scenario helpers, not an invented runtime failure or pass. Raw evidence is preserved in worth-orchestration `5904f98d5fa62c54754717ff641712f9d187eaf6:handoff/m3-h8/host3-terminal-d1806/`.

  C1 retains every original completion/attempt/World identity, new-effect fence, dedupe, no-second-effect/physical-contact, genuine receipt/custody and release assertion, including repeated reopen and typed conflicting-message behavior. Do not convert the retained original to no-effect, suppress D18, fabricate a live dispatch or replace these originals with graceful cases. This classification proves no new recovery capability and no all-runtime-green M3 status. H8's separate `inbound_custody::reopening::authentic_callback_completes_the_original_after_reopen_and_deduplicates_another_reopen` and `inbound_custody::transport::recorded_completed_transport_acquires_its_first_audience_and_publishes_after_reopen` keep their actual successful close/reclaim and outstanding-custody assertions; they are not evidence of this C1 continuation.

- **C1: Native effect before World disposition (D18).** The following complete
  `process::application::world_durable_release` originals require unclean
  recovery before their later graceful assertions. D18 refuses that first open;
  no kill is substituted by close and no successful recovery is claimed here.
  Restore each complete original and its unchanged prefix-capable child at C1:
  - `killed_native_effect_reopens_original_world_custody_releases_and_reopens_again`:
    restore `22e638d638:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/world_durable_release/mod.rs`.
  - `a_native_fork_from_a_released_predecessor_reopens_before_another_checkpoint`:
    restore `22e638d638:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/world_durable_release/native_transitions.rs`.
  - `killed_native_after_two_publications_recovers_exact_unpublished_predecessor`:
    restore `22e638d638:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/world_durable_release/published_predecessor.rs`.
  - `released_native_effect_preserves_history_and_allows_a_fresh_edit_after_graceful_reopen`:
    restore `22e638d638:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/world_durable_release/graceful_source.rs`.
  Their Native receipt/source, unchanged predecessor, subsequent fork/tail and
  checkpoint, incarnation fencing and released-original assertions all remain
  obligations in those preserved originals. Live durable release currently
  refuses `ReopenRequired`, and D18 refuses the unclean Native-effect receiver;
  no no-effect court claims to prove that Native receipt continuation.
  The active `unclean` and `unclean/quiescent` controls retain typed before-service
  refusal, the prefix-capable child, quiescent kill, graceful close and a fresh
  publication. `world_original_state` separately protects acknowledged published
  state and no-effect released state across graceful reopen and further editing.

- **C1: Original Native transition before World occurrence (D18 / D16).**
  All six `process::application::no_pending_owners::transition_native_effect`
  courts first abandon an associated Native effect without its World occurrence
  or disposition. D18 refuses that reopen before service; their later assertions
  remain C1 obligations, not passing Landing 2a evidence. Preserve all of:
  original receipt/attempt/commit/custody identity; zero provider reexecution;
  stale/foreign grants before use; release refusal and claim reclamation; ACK
  retention through allocation refusal; actual head installation; one associated
  effect; cleared recovery inventory; close refusal/resume; spent movement; and
  observation without retaining the old Home lease.
  The complete courts and their distinct retained claims are:
  - `a_durable_native_transition_effect_continues_its_original_world_occurrence_without_execution`: original receipt/identity and zero-reexecution continuation across reopen.
  - `affinity::original_native_continuation_refuses_stale_and_foreign_grants_before_use`: stale/foreign admission preserves custody for the current receiver.
  - `allocation::an_acknowledged_original_retains_its_native_custody_through_allocation_refusal_and_retry`: actual ACK and receipt survive refusal/reclaim/retry.
  - `interruption::consuming_original_world_movement_refusal_preserves_custody_for_actual_home_reopen`: consuming movement refusal retains original custody and refuses close until reopen.
  - `interruption::unwind_at_the_actual_committed_world_head_keeps_the_original_in_reopen_required`: actual head and spent movement survive unwind without reexecution.
  - `interruption::dropping_an_original_interruption_guard_disarms_only_its_receiver_control`: busy arming and disarm/rearm preserve original continuation and receiver isolation.
  Restore the complete assertion-bearing files from these immutable references:
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_native_effect.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_native_effect/native_original.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_native_effect/progression.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_native_effect/affinity.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_native_effect/allocation.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_native_effect/interruption.rs`.
  Shared opener context (retained active for its other callers): `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_open_original.rs`.
  The sole-caller interruption controls and their attached plumbing move with
  these courts. Restore the relevant control portions from these references,
  preserving ordinary movement witnesses, recovery barriers, continuation
  progression and independent owner publication-unwind tests:
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/publication/home_transition_effect/control.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/lifecycle/owner/operation/home_transition_effect/control.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/publication/home_transition_effect.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/lifecycle/owner/operation/home_transition_effect.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/publication/mod.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/facade.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/lifecycle/service_ports/publication/home_transition_effect.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/recovery/recovered_publication.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/lifecycle/close/recovered.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/recovery/recovered_publication/home_transition/work.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/lifecycle/owner/operation/home_transition_effect/preparation.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/recovery/recovered_publication/home_transition/advance.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/branch/reference_cell/durable_movement.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/branch/reference_cell.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/branch/reference_cell/publication.rs`.
  - `restore from 4ea38baa3bd783462c48aee1d753d2e4248d20ac:crates/worth-runtime-world/src/recovery/product_unpublished.rs`.
  Reintroducing the removed `query-world-commit-preparation` certification
  caller for `transition_native_effect/native_original.rs` requires a fresh
  boundary ruling; restoration references grant no caller-list expansion.
  Restore complete courts only through public owner recovery after C1 supports
  this Native-effect prefix and C5 lifts D18. No graceful close may replace
  their abandoned prefix or weaken the original assertions.

Courts held out under D13 are listed the same way, under the H item whose Home capability they need. The second master merge (`171d380829`) brought in Query cold journeys that reach `WorthQueryInterimCheckpointRestore` from outside the frozen caller list, directly or through a shared reopen fixture. D13's list only shrinks, and Directory Home refuses their producers until the phases below land, so each court is deleted from the tree and returns through the Directory Home open, never through the interim door. The assertions move intact.

- **Phase 6 / H6** (also needs H5 producer admission). All `restore from 171d380829:`.
  - `workspaces/worth-query/crates/worth-query-court-decl-host-owned-codec-transition/` (whole crate) with its fixture `workspaces/worth-query/crates/worth-query-testkit-certification-canonical/src/certification/application_graph/source_owned_transition.rs` and its two `mod`/`pub use` lines. Also needs D16's Host transition request. Claim (`same_route_codec_retirement_covers_the_source_owned_origin`): a source-owned V1 output crosses an authenticated same-route codec transition with authored payload, source identity and original bytes preserved; the codec-retired route reopens; ordinary source deletion removes both publication and presence rows and stays removed across reopen; an independent branch produces a fresh V2 output and preserves it on edit.
  - Under `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/owned_outputs/`:
    - `world.rs`: `restore_changed_build`, `restore_same_build`, `restore_program_checkpoint` (the shared cold fixture).
    - `cold_selection.rs` (whole file): `cold_same_build_selection_retains_both_families_without_contacts`, `worker_only_cold_same_build_selection_retains_both_families_without_contacts`, `cold_role_outside_the_selected_publication_refuses_without_effect`, `worker_only_cold_refusal_keeps_data_without_native_custody`. Claim: same-build cold selection reuses both families with zero producer and source-projection contacts; a role outside the selected publication refuses with the original source, producer and family and no effect; the refusal outlives host teardown without retaining Native custody.
    - `build_change.rs` (whole file): `changed_build_contacts_only_the_family_whose_publication_is_still_old`, `worker_only_changed_build_regenerates_only_the_old_family`. Claim: under a changed build, navigation reruns only the family whose publication is still old (value 2001, one contact).
    - `hierarchy/cold_parent/{mod,fixture,program,corruption,meaning}.rs` (whole directory): `corrupt_non_leaf_parent_lineage_refuses_without_effect`, `changed_discovery_build_runs_real_parent_fallback_and_discovers_its_new_child`, `a_cold_two_family_model_edit_succeeds_the_current_owned_publications`, and the `worker_only_` twin of each. Claim: wrong cold parent lineage refuses atomically as `OutputPublicationCorrespondence` naming source, family and producer; a changed discovery build runs the real parent fallback and discovers its new child; a cold two-family model edit succeeds the current owned publications.
    - `predecessor/courts.rs`: `cold_reopen_reports_unavailable_instead_of_substituting_current_values`; `predecessor/foreign.rs` (whole file): `an_old_foreign_value_is_not_a_current_dependency_and_current_output_reopens`. Claim: after cold reopen a predecessor read reports `PredecessorBasisUnavailable` and never substitutes a current value; a historical foreign value never becomes a current output dependency.
  - A restored court reaches its doors through the landed H capability (the Directory Home open); the frozen caller lists never grow, so any caller row a restored court would need requires a fresh boundary ruling. Rows removed with these courts (`tools/boundary-check/config/control-callers.txt`, owner `workspaces/worth-query/crates/worth-query-execution/`): `owned_codec_transition.rs`, `build_change.rs`, `cold_selection.rs`, `hierarchy/cold_parent/corruption.rs`, `hierarchy/cold_parent/meaning.rs`. Restoring the codec-transition court crate also restores its `court-frontier-splits.toml` entry.
- **Phase 9 / H9** (also needs H5 producer admission and H6 required-output custody; every journey executes a correction-bound Create, Retire or Edit). All `restore from 171d380829:workspaces/worth-query/crates/worth-query-court-decl-host-authored-ownership/tests/authored_ownership/<path>`.
  - Shared fixtures: `coupled/history.rs` (`reopen`, `try_reopen`, `retire_sentinel`, `require_unique_edge_ownership`), `native_checkpoint_tamper.rs`, `co_checkpoint_tamper.rs`.
  - `co_history.rs`: `corrupt_recorded_co_anchor_refuses_at_the_real_closure_owner_with_its_source_subject`. Claim: after retire and reverse, a cold-corrupted recorded co-anchor refuses at the authored-closure owner as `CorruptPublication` naming the source, leaves unrelated records readable, and the clean twin admits the same candidate.
  - `cold_owned_revision.rs`: `an_ordinary_body_revision_settles_after_checkpoint_and_cold_reopen` (with private `reopen`, `edit`, `limits`; the `RequiredProgram` fixture stays in the tree). Claim: a performed source revision settles its required body before capture; after reopen the next revision settles, removes output 103 and exposes 104.
  - `coupled/multi.rs`: `each_coupled_writer_owns_only_its_own_business_edges_even_with_a_shared_endpoint` and the `if MODE == 15` block of `two_bodies`. Claim: two writers of one coupled publication; a historical twin claiming the same recorded edge makes retirement refuse as `CorruptPublication` naming the source with a usable sentinel; clean history admits the candidate.
  - `current_roster.rs`: `dropped_added_and_repointed_current_routes_refuse_after_real_frame_admission`, `a_pre_roster_store_cannot_be_read_as_empty_current_ownership`. Claim: roster changes 0/1/2 refuse during reopen with the exact owned-recovery subject; change 9 survives frame admission and fails the first ordinary mutation as `CorruptCurrentOwnership` with no record loss; a pre-roster store refuses as `UnsupportedLegacySemantics`.
  - `coherent_roster.rs`: `coherent_whole_roster_and_route_drops_and_both_digest_corruptions_refuse_typed`. Claim: coherent whole-roster and route drops and both digest corruptions refuse typed, at reopen or at the first cold custody use, never as an opened fence.
  - `current_history.rs`: `published_owned_history_cannot_replace_missing_current_presence_on_either_route_door`. `current_fence.rs`: `ordinary_source_edit_head_preserves_warm_and_cold_fence_decisions`. Claim: published history cannot stand in for missing current presence on either route door; warm and cold fence refusals are identical.
  - `content/courts.rs`: `coupled_writers_read_and_publish_their_actual_content_across_checkpoint_and_correction`, `a_closed_owned_publication_restores_content_and_retains_it_after_checkpoint`. Claim: coupled writers decode and publish their actual Native content across a cold boundary and a correction; a restored owned publication keeps its content after reopen.
  - `coupled/history_seams.rs`: `cold_coupled_history_refuses_an_authored_anchor_adopted_by_an_earlier_publication`, `cold_coupled_restoration_refuses_an_owner_whose_preceding_publication_has_no_live_member`. `coupled/owner_set_history.rs`: `cold_retirement_owner_roster_is_recomputed_from_preceding_facts`, `cold_restoration_owner_roster_is_recomputed_from_preceding_facts`. Claim: cold historical headers cannot adopt an authored anchor or lose a surviving writer; dropped, added or repointed group rosters refuse as `CorruptPublication` naming the recomputed owner.
  - `parameter_inverse/adjacency_gap.rs`: `recorded_support_refuses_uncovered_native_adjacency_with_its_exact_relation_and_kind`. Claim: recorded correction support refuses uncovered Native adjacency as `IncompleteAdjacencyHistory` with the exact commit, relation and kind.
  - `relation_splice/checkpoint.rs`: `coupled_stage_retirement_reuses_native_writer_custody_after_checkpoint_reopen`. Claim: after reopen, the next stage retirement drives the cold Preserve directly with one producer contact.
  - A restored court reaches its doors through the landed H capability; the frozen caller lists never grow, so any caller row a restored court would need requires a fresh boundary ruling. Rows removed with these courts (`control-callers.txt`): owner `crates/worth-relational/` for `coherent_roster.rs`, `current_history.rs`, `current_roster.rs`; owner `workspaces/worth-query/crates/worth-query-execution/` for `coherent_roster.rs`, `current_history.rs`, `parameter_inverse/adjacency_gap.rs`.
  - The Native subclaims of these courts (co-anchor closure, duplicate coupled-edge custody, current-roster corruption, codec-retirement semantics) are track H and are rebuilt as Native-owned courts through existing Native issuance; they never stand in for the Query journeys above.

### C2: fixture exact-work, island growth and shared-process memory campaigns

These are whole cost/bounds campaigns under Landing 2a's C2 disposition, not
passing H results and not ignored tests. Immutable source for every artifact
below is `89039fa7350156c5a62deb38eccaddbec66a297c`.

- **Parameter exact-work campaign.** Restore both
  `parameter_lease_exact_budget_keeps_the_serial_verdict` and
  `parameter_lease_one_unit_short_is_the_work_stop`, their complete `run`
  input matrix, original `Case`/`committed` helpers, serial-versus-leased
  comparison, measured `charged_work`, exact paid grant and paid-minus-one
  refusal. The actual one-short run returned `ProductUnpublished`, rather than
  its asserted pre-effect `Denied(WorkExhausted)`. This remains failed evidence;
  no arbitrary grant, accepted-unpublished substitution or weaker assertion
  replaces the campaign.
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/parameter_entry/src/parameter_invariant/lease_tests.rs`.
  Restore its original exact setup closure at that SHA:
  `.../parameter_entry/src/parameter_invariant/lease_tests/authentication.rs`,
  `.../parameter_entry/src/parameter_invariant/lease_tests/program.rs`,
  `.../parameter_entry/src/parameter_invariant/lease_tests/resources.rs`,
  `.../parameter_entry/src/parameter_invariant/lease_tests/schema.rs` and
  `.../parameter_entry/src/parameter_invariant/lease_tests/world.rs` under the
  same `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/`
  prefix. This is the COMPLETE RECURSIVE `lease_tests/` tree, including
  `lease_tests/schema/records.rs` explicitly mounted by `schema.rs`. Preserve both path mounts and the original native commit door. The
  independent `parameter_lease_cancelled_commit_keeps_the_cancellation` remains
  live, on the same actual leased commit path, with its cancellation and one
  advancement assertions intact. Its still-used placement, budget-reset and
  request-observer controls and exact caller row remain mounted. Restore C2's
  original observer/arm statements from the same-SHA
  `tools/boundary-check/config/control-callers.txt` and Query Execution's
  `application_contribution/request_execution` owner tree; do not add new rows.

- **Component-island growth campaign.** Restore the whole
  `real_split_component_reads_are_unchanged_with_ten_and_ten_thousand_unrelated_islands`
  comparison, including both real populations, complete `split` setup,
  independent before/after membership and heir identity assertions, actual two
  selected three-member components, preparation/evaluation captures, nonzero
  adjacency assertion and exact business-read comparison. Its large population
  actually refused `CanonicalArtifactEncoding(SizeLimit { observed: 268435457,
  maximum: 268435456 })` during installation, before the growth comparison.
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/component_islands/locality.rs`.
  Restore the exact `mod locality` mount from same-SHA
  `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/component_islands/mod.rs`,
  its complete same-SHA `component_islands/{world,identity_model,union_find}.rs`
  and `component_islands/world/` tree, plus the complete same-SHA
  `checkpoint_recovery/support.rs` and `checkpoint_recovery/support/` tree
  under that prefix. The still-used World, observers and differential/oracle
  functional courts stay live. No smaller population, raised artifact limit or
  replacement semantic proxy discharges the moved growth claim.

- **Shared-process exact memory campaign.** Restore the entire isolated-child
  `execution_memory_is_shared_by_installed_worlds_and_reselected_on_checkpoint_reopen`
  journey: omitted-authority refusal, two genuine Worlds sharing one host
  authority, all bootstrap-return observations, exact reservation capacities,
  allocator contact counts, live payload bytes, retained ticket lifetimes
  across World/lease drops, checkpoint reconstruction and authority reselection,
  exact Process-level refusal payloads, and final full-capacity release.
  Its actual post-reopen assertion at line116 failed; it is not evidence of a
  passing memory bound. This campaign deliberately fills process capacity and
  asserts exact remaining bytes; the complete campaign belongs in C2.
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/process/certification/program_example_authority/execution_memory/mod.rs`.
  Restore the exact original process mounts at that SHA:
  `workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/process/certification/program_example_authority/mod.rs`
  and
  `workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/process/certification/mod.rs`.
  Restore its complete support source and path export:
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-certification/src/certification/execution_memory.rs`;
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-certification/src/certification/program_example_authority.rs`.
  The full same-SHA `program_example_authority/product_workflow_support/`
  tree supplies the original actual schema, program, binding, seed, reads,
  clock and configuration. It stays live for independent ordinary Temporal
  examples/courts. The moved campaign has no unique control-caller row or
  production hook. Its sole interim checkpoint authorization in
  `tools/boundary-check/config/interim-checkpoint-callers.txt` is removed with
  that caller. Its historical row is recoverable from the same immutable SHA,
  but MUST NOT be reinstated: D13 freezes that list against growth. Restore
  this entire semantic/resource journey through the ordinary public Home open
  path and genuine configured execution authority, or an already-authorized
  retained fixture owner where lawful; do not add an interim capture/restore
  caller, byte transport authority, or adapter shortcut. The general runtime
  Execution-memory implementation stays.

All paths abbreviated with `.../` above use the explicit common immutable
prefix in their paragraph. Restoration must take the original complete source
and parent mounts, not a reduced wrapper or only the final red assertion. H
quiescent-kill idempotency and historical-read cases are independent and remain
active; neither is moved by this cost classification.

### C2: complete logical-rule acquisition growth campaign

`logical_local_rule_business_acquisitions_ignore_ten_and_ten_thousand_unrelated_cohorts`
moves as one whole C2 growth campaign. Restore its ten-versus-ten-thousand
comparison and complete `run` sequence: real graph/rule installation, original
projected membership, actual edge replacement, both independently recorded
preparation/evaluation read measures, nonzero read assertions, literal edge
oracle, unchanged record identities, post-edit logical members, retained
Native logical identity, and close. Its real failure was installation's
`CandidateLifetimeExpired { maximum_lifetime_millis: 30000 }`, before the
large-scale comparison completed. It is not a passing growth result; no timeout
is raised, no smaller population substituted and no comparison weakened.

Restore each complete artifact and mount/hook below from immutable
`89039fa7350156c5a62deb38eccaddbec66a297c`:

- `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-host-package-archive-replay-application-graph/tests/worth_query_court_decl_host_package_archive_replay_application_graph/certification/application_graph/relation_laws/logical/locality/mod.rs`
- Court mount: `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-host-package-archive-replay-application-graph/tests/worth_query_court_decl_host_package_archive_replay_application_graph/certification/application_graph/relation_laws/logical/mod.rs`
- Whole growth setup and `LocalLogical` policy: `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-certification-application-graph/src/certification/application_graph/relation_laws/logical/locality.rs`
- Whole scoped observer/guard: `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-certification-application-graph/src/certification/application_graph/relation_laws/logical/observation.rs`
- Fixture-specific schema model: `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-certification-application-graph/src/certification/application_graph/relation_laws/schema/policy_models/local_logical.rs`
- Both support mounts: `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-certification-application-graph/src/certification/application_graph/relation_laws/logical.rs`
- Model mount/export: `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-certification-application-graph/src/certification/application_graph/relation_laws/schema/policy_models.rs`
- The scoped-versus-census policy and observer hooks: `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-certification-application-graph/src/certification/application_graph/relation_laws/schema.rs`
- Rule capture and scoped preparation/evaluation branches: `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-certification-application-graph/src/certification/application_graph/relation_laws/logical/rule.rs`

The ordinary `Logical` rule courts stay live with their existing census
semantics, projected-member meaning, old/proposed comparison and stable
logical identity claims. Only the removed locality policy selected the scoped
branch and its measurement callback. No court is ignored or conditionally gated.

### Landing 2a authored current-ownership cold campaigns

- **C4 — damaged current fact-family pairing.** Move the whole
  `external_cold_pairing_rejects_whole_fact_family_loss_before_an_unrelated_access`
  campaign, not only its red assertion. It captures a real Query publication,
  corrupts each complete fact family, reconstructs cold current members, then
  asserts the typed corrupt-current-ownership refusal on unrelated access.
  Restore the complete Query court and Native probe:
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-host-authored-ownership/tests/authored_ownership/cold_pairing.rs`;
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:crates/worth-relational/src/publication/ownership/cold_pairing_court.rs`.
- **C2 — complete physical cold-locality campaign.** Move the whole
  `current_custody_contacts_only_the_touched_publication_and_leaves_other_regions_cold`
  campaign, including its seed and value encoders, 64-partition setup, exact
  63-cold-member invoice, before/after residency observations, roster counts,
  real unrelated preparation and typed owned-delete refusal. Restore:
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-host-authored-ownership/tests/authored_ownership/locality.rs`;
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:crates/worth-relational/src/publication/ownership/locality_court.rs`.

For either campaign restore its shared complete capture/corruption/recovery
source, including original frame/region handling and source ownership:
`restore 89039fa7350156c5a62deb38eccaddbec66a297c:crates/worth-relational/src/publication/ownership/cold_checkpoint_source.rs`.
Restore the external-placement corruption method used only by that source from
`restore 89039fa7350156c5a62deb38eccaddbec66a297c:crates/worth-relational/src/durability/data/native_checkpoint/roster_corruption.rs`.
The active embedded-placement corruption method stays compiled.
Restore each campaign's exact Query bridge method from
`restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_runtime/certification_controls/owned_publication.rs`,
the corresponding court and Native mounts from
`restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-host-authored-ownership/tests/authored_ownership/mod.rs`
and `restore 89039fa7350156c5a62deb38eccaddbec66a297c:crates/worth-relational/src/publication/ownership.rs`,
and its exact Native/Query caller rows from
`restore 89039fa7350156c5a62deb38eccaddbec66a297c:tools/boundary-check/config/control-callers.txt`.
C2 also restores `CurrentOwnershipLocalityForTest` and its export from
`restore 89039fa7350156c5a62deb38eccaddbec66a297c:crates/worth-relational/src/facade.rs`.
These are function/mount/export restorations, not wholesale replacement of the
other active controls. Production current-ownership selection, membership,
metadata, retargeting, merge and ordinary authored-correction courts stay live.
Both actual failing executions remain red historical evidence; no passing
completion or successful cold reconstruction is claimed.

### Landing 2a Query cancellation sweeps and temporal identity cost

These complete campaigns move to their named cleanup lanes. No test is ignored
or conditionally gated. Immutable source is
`89039fa7350156c5a62deb38eccaddbec66a297c`, before the Python-only runner fix.
That source contains the full tests and their original arming closure.

- **C4 — original Directory receipt cancellation at every consult.**
  `original_directory_receipt_preserves_cancellation_at_every_consult` counts
  the admitted business callback, injects cancellation at each observed consult,
  verifies the exact typed stop, then verifies late cancellation preserves the
  already-produced original receipt, one handler contact and unchanged balance.
  Restore the WHOLE test, including its local call helper, baseline, full sweep
  and post-sweep/late-cancellation observations:
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/process/host/durable_retry/request_stops.rs`.
  Restore its exact `crates/worth-execution/` caller row from
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:tools/boundary-check/config/control-callers.txt`.
  The containing module and the ordinary cancelled-key/no-effect/retry court
  remain live. The sweep's real run was externally stopped after exceeding the
  committed CI deadline under a runner that had omitted that deadline; no pass
  or native-timeout event is asserted.

- **C2 — typed terminal at every charged work prefix.**
  `conditional_operation_work_stop_terminal_matches_its_typed_cause` first
  executes a healthy temporal operation, records every real charged prefix,
  reruns each unique positive prefix one unit short, and compares the terminal
  with its actual typed owning cause. Restore the WHOLE test and local run
  function, charge collection, all terminal/cause variants and assertions:
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/oracle/host/temporal_conditional_operation/counted_read_stops/mod.rs`.
  Restore its exact `crates/worth-execution/` caller row from
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:tools/boundary-check/config/control-callers.txt`.
  The containing mount, shared counted-read support, and the neighboring
  cancellation-at-selected-consults court remain live. The work-prefix sweep
  was externally stopped after exceeding the omitted committed CI deadline;
  that execution remains failed. This move does not weaken its terminal
  classification contract or fit a new work ceiling.

- **C2 — complete exact temporal identity-work campaign (two wrappers).**
  Both `temporal_identity_work_is_cold_or_fresh_admission_only` wrappers failed
  the initial exact basis-preparation invoice (actual3, expected1). Restore both
  wrappers and the ENTIRE shared campaign, including binding/installation
  preparation, digest/text counts, admission phase counts, and every later
  zero-work assertion:
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/cost/host/temporal_conditional_operation/mod.rs`;
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/cost/certification/runtime_world_branching/mod.rs`;
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-testkit-host/src/host/temporal_conditional_operation/courtroom.rs`.
  Functional H coverage remains in
  `courtroom::host_installs_and_executes_due_operation`, which asserts actual
  committed lineage, application-attempt identity and authored effects. The
  unrelated-row reconstruction growth courts and all other temporal lifecycle
  courts remain unchanged.

### Landing 2a Query checkpoint wire campaigns (C4)

- Move the complete
  `checkpoint_recovery_retains_corruption_and_unsupported_semantics_by_type`
  campaign, including all corruption and version twins, outer checksum/length
  repair, legacy comparison, typed cause checks and positive original restore.
  Its obsolete CHK20.1/CHK20 producer assumption and successful legacy import
  claim require C4 reconciliation with the current-format-only cutover; they
  must not be repaired by reintroducing retired formats in H. Restore:
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/aspect_patterns/one_of/checkpoint_recovery.rs`
  and its mount from
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/aspect_patterns/one_of/mod.rs`.
- Move the whole
  `checkpoint_inspection_compares_query_header_count_and_output_metadata_bytes`
  function, including the actual optional-output producer, healthy capture and
  baseline inspection, every authenticated header/count/metadata-byte attack,
  and all inequality observations. Restore that complete function from
  `restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-execution-package-archive/tests/worth_query_court_decl_execution_package_archive/domain_computation/primary_graph/output_lineage/ownership_oracle/optional_aspects/retained_recovery/mod.rs`.
  The containing module, optional-aspect absence/invalidation journey and
  projection/identity/no-authoring assertions remain live. Its arming and
  observation caller rows remain because those same live functions still use
  the existing doors. No function is ignored or conditionally gated.

Both original executions remain failures, not successful format or corruption
evidence. Ordinary current checkpoint restore/continue courts and typed Native
cause propagation courts remain active.

### C2: producer succession's exact physical shrink invoice

The exact physical live-record removal invoice is split from the still-live
functional producer-succession history/restore journey. Restore the COMPLETE
`assert_shrink_delta` helper, including checkpoint inspection, publication-record
filter, entity/relation classification, exact four-entity/four-relation invoice
and diagnostic record sets from
`restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/producer_succession/checkpoint.rs`.
Its WHOLE campaign is preserved in
`restore 89039fa7350156c5a62deb38eccaddbec66a297c:workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/producer_succession/history_courts.rs`:
restore the complete original
`checkpoint_keeps_exact_per_route_retirement_and_recovers_the_surviving_generator`
sequence, including two-child production/settlement, baseline capture, actual
shrink/settlement, subsequent capture, exact invoice invocation, independent
per-route history/origin checks, restore, absent retired sources, regrowth and
all final live-source assertions. Those H assertions remain active now; C2 must
retain them when restoring the mixed physical-cost claim. No helper-only or
assertion-only restoration is sufficient.

Only the invoice call and its otherwise-unused baseline capture leave the H
journey. Every typed/current-format history, separate retired route, exact
predecessor identity, producer/dependency/birth metadata, retired-role and
no-recreated-edge assertion remains, as do real restoration and further writes.
The actual old invoice execution failed and remains failed evidence; no amount
of physical reclamation is certified by the retained H test. No test is ignored
or gated and no count is adjusted to the latest observed value.

### Landing 2a restored-header cleanup campaigns

- **C4:** restore the complete `669ac9fcd4:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/restoration_header/` tree:
  `mod.rs`, `cascade.rs`, `restoration.rs`, `scope.rs`, `journey.rs`, and
  `journey/publication.rs`, with its same-SHA `relational/mod.rs` mount.
  Preserve every individual corruption, actual source/subject, typed refusal,
  exact detail, affected-record identity, unchanged current basis, lawful twin,
  restored history, deliberate unwind and history-movement/commit-survival
  observation. The ordinary source/realization retirement and full restoration
  positives remain active in `relational/authored_retirement`. The complete
  genuine co-anchor journey and every existing setup assertion remain active
  under `authored_retirement/co_anchors/{journey.rs,journey/publication.rs}`;
  its ordinary court preserves fresh restored anchor/co-anchor identities,
  their exact handoff into the next retirement, repeated lawful inverse
  admission and unchanged current basis. It adds no corruption door/caller.
- **C2:** restore `restoration_header_is_validated_once_and_work_grows_linearly`
  and `public_create_grow_retire_restore_relation_scale_report` from
  `669ac9fcd4:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/restoration_header.rs`,
  including its complete same-SHA `restoration_header/{journey.rs,republication.rs}`
  fixtures and `cost/relational/mod.rs` mount. Preserve all three scale axes,
  once/repeated validation equality, strict monotonic work, linear differences,
  and the actual 10-publication/720-relation create/grow/retire/undo workload.
- **C4:** restore `missing_restoration_entity_mapping_is_refused_before_origin_lookup`
  from that same cost file, with the complete genuine journey above and its
  exact missing-entity refusal before origin lookup. It is an injected copied
  roster defect, not a cost assertion.
- Restore the complete same-SHA control closure for these campaigns:
  `crates/worth-relational/src/history/authority/corruption_court.rs` and
  `corruption_court/header.rs`, their authority/history/facade exports,
  `history/data/canonical_commit_envelope/required_output_origin/{header_corruption.rs,header_work.rs}`
  and their parent mounts, and the original begin/finish observation wrapper in
  `required_output_origin/restoration_header.rs`. Restore the exact original
  three `control-callers.txt` rows only through the governing boundary ruling;
  no replacement caller was added at move-out. Ordinary validation retains its
  full indexed algorithm, original charging, outcomes and caller signature.

Landing 2a adapter closure (lane P-A, merged at `e4d3b01c8d`) moved these Native durability roots out, each with its restore reference:

#### C4 - Injected faults

Integration protocol restoration refresh (same per-claim disposition; no retired campaign is remounted):
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore/reuse_tests.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/storage/residency/cross_region_forgery_tests.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/tests/durability/contracts/semantic_result/custom_versions.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/tests/support/durability/canonical_commit_frame.rs`
Restore the complete same-SHA `crates/worth-relational/src` tree, including original parent mounts, fixture/control/support/producer and reconstruction-lane dependencies. Preserve every original assertion/input/contrast and genuinely owned source boundary when migrating the campaign; never revive runtime_attachment discovery. These references retain integration revisions to the already moved private cost/fault families. Ordinary source functionality remains in the public counterparts.


Custom enforcement decision corruption retains its full original campaign:
- C4 full original semantic_result_preserves_custom_enforcement_version_distinct_from_aggregate_request, including callback tuple indices, exact decision rewrite, outer reseal and ValidationMismatch refusal: `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src/tests/durability/contracts/semantic_result/custom_versions.rs`, with OnePlannedEntity in that file, original semantic_result.rs/contracts/mod.rs parents and `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src/tests/support/durability/canonical_commit_frame.rs`.

Each restore includes the complete immutable shared fixture/control trees `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src/tests/support.rs`, `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src/tests/support`, plus direct owner/canonical/control ancestors from `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src`. Preserve complete original claims/arming/observations when restoring through genuinely owned source boundaries; never revive the runtime-discovery adapter. Copy these exact entries into the spec before removing original roots.


The original rewrite/reseal setup in
`durability_contract_replays_merge_from_typed_authority_when_diagnostics_are_absent`
retains its complete callback and all original guards: restore from
`622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs`,
with the same-SHA `tests/support.rs`, complete `tests/support` tree and exact
`history/data/canonical_commit_envelope` owner/control closure recorded in C2.
Its actual ordinary source readmission/typed-authority/whole-DATA counterpart is
`durable_publication/checkpoint_merge_continuity.rs`. Never revive the retired
runtime-discovery adapter to restore the original rewritten source campaign.


Additional orphan control closure for the moved transition/publication campaigns:
restore `86e84dc5cd:crates/worth-relational/src/branch/transition/publication_pause_for_test.rs`
with its fields, initialization, arming door and pause call in same-SHA
`branch/transition/durable_step.rs`. Restore the complete local scripted-answer
fixture from same-SHA `tests/support/durable_medium.rs` and
`tests/support/durable_medium/{scripted_publication,journal,transition_journal,transition_contents}.rs`.
These controls lost every arming caller with the moved C4 roots. Their complete
original behavior remains required on restoration through genuine owned media;
the ordinary next-position/sequence and identical-retry rules stay live unchanged.


Remaining pure cleanup roots (per-claim Landing 2a disposition):

- C4: `branch_root_image_versions_refuse_before_restoration_and_current_recovers`,
  introduced on the authorized integration base: restore the complete unsupported
  version mutation/refusal/current-format control from
  `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/unsupported_root_version.rs`,
  its same-SHA `branch_root_checkpoint.rs` parent and complete
  `crates/worth-relational/src` fixture/codec/authority closure. This is the original
  private malformed-plan campaign; ordinary current-format public recovery
  remains mounted in `checkpoint_branch_targets.rs`.


- `durability_contract_failure_missing_authoritative_parent_closure_is_explicit`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/contracts/rejected_and_parent_closure.rs`.
- `durable_fork_and_source_commit_complete_without_deferral` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs`.
- `durable_fork_and_source_archive_complete_in_lock_order` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs`.
- `durable_fork_waits_for_another_branch_commit` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs`.
- `memory_commit_waits_behind_a_transition_without_consuming_its_candidate` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs`.
- `divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`.
- `equal_image_cannot_bypass_global_contract_readmission`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`.
- `forged_cross_region_kind_commitment_is_refused_before_member_reads`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs`.
- `an_indexed_envelope_storage_failure_is_typed_and_sticky`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs`.
- `replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs`.
- `restore_refuses_unknown_fields_before_publishing_any_model_state`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs`.
- `rehashed_cross_region_kind_forgery_is_refused_by_the_canonical_root`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/storage/residency/cross_region_forgery_tests.rs`.
- `refused_durable_publication_leaves_no_replacement_lineage_behind`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/lineage/resolution.rs`.
- `managed_fork_refusal_and_success_drop_lookup_outside_native_coordination` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/branch/coordination/managed_fork_cleanup.rs`.
- `fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs`.
  Restore each root's complete same-SHA parent/descendant fixture and control
  closure, including its original local medium script and owner units; migrate
  the source boundary through genuine owned media without reviving the retired
  runtime-discovery adapter. All original guards remain required.

  The inert lock-order control is deleted completely. Restore its full arming,
  observations, fields and call sites from `f9f15bf8c8:crates/worth-relational/src/runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs`,
  with the same-SHA `canonical_publication_routes.rs` owner and its
  `canonical_publication_routes/{branch_transition_stream,checkpoint_admission}.rs`
  call sites when restoring the original lock-order campaigns.


- Associated-original missing-route and dropped-unrelated-body campaign parts:
  `associated_effects_refuse_memory_and_absent_settlement_evidence` and
  `associated_effects_discover_cold_subject_without_reading_unrelated_history`:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/associated_effects.rs`.
  Restore the exact route removal / unrelated-artifact deletion and their
  original refusals or successful selected-body guards with `perform` fixture.
- `native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`,
  actual three-body deletion followed by successful metadata enumeration:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/associated_effects_discovery.rs`.
- `caller_association_survives_embedded_checkpoint_and_refuses_old_semantics`,
  both original `CHK13`/`CHK14` mutations and exact `UnsupportedLegacySemantics`:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/caller_association.rs`.
  The positive actual embedded original association/re-encoding assertion stays
  as a genuine Memory owner codec unit.
- `recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor`,
  `recorded_component_checkpoint_readmits_original_empty_predecessor`, and
  `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`,
  injected unrelated-artifact deletion parts:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs`.
  Restore complete same-SHA `original_custody`/original capture/recovery fixtures
  and successful selected readmission guards, coordinated with C2's physical
  custody/release portions. Public ordinary readmission is separately retained.


- `refused_durable_transition_returns_its_authority_and_moves_nothing` and
  `unresolved_durable_transition_installs_nothing_and_names_its_fate`:
  `restore from e4d3b01c8d:crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs`.
  C4 retains the actual ProvenNoEffect/Indeterminate scripts, exact original
  offered fate, authority return/retry and no-effect guards with the same-SHA
  `recovered_world`, `candidate`, `main_basis` fixtures. Restore without the
  deleted local provider/discovery adapters. Ordinary transition ownership
  remains covered through `checkpoint_successor_authority.rs` public courts.


- `checkpoint_recovery_rejects_foreign_branch_target_substitution`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs`.
- `tail_recovery_rejects_foreign_branch_target_substitution`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs`.
- `checkpoint_recovery_rejects_missing_exact_branch_root_artifact`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_recovery_rejects_root_artifact_without_commit_envelope`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_recovery_rejects_duplicate_global_partition_images`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_recovery_rejects_duplicate_branch_root_partition_images`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_rejects_previous_branch_root_format_before_readmission`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_rejects_altered_root_content_with_recomputed_image_digest`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_rejects_omitted_root_partition_with_recomputed_image_digest`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_rejects_same_commit_truth_root_relabeling`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_rejects_same_commit_schema_root_relabeling`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `tail_rejects_same_commit_root_descriptor_substitution`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_owner_root_is_stable_across_fingerprint_symbol_ids`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `recovery_rejects_a_missing_branch_root_schema_carrier`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs`.
- `recovery_rejects_a_tampered_branch_root_schema_carrier`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs`.
- `recovery_rejects_a_schema_carrier_swapped_between_exact_roots`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs`.
- `checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_shared_target_binding.rs`.
- `checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_shared_target_binding.rs`.
- `a_deleting_checkpoint_cell_requires_its_retired_name`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs`.
- `every_transition_fate_preserves_the_durable_before_live_boundary`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs`.
- `an_unresolved_commit_also_blocks_branch_transitions_with_its_original_fate`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs`.
- `a_damaged_transition_frame_is_refused_at_the_medium_read`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs`.
- `a_sequence_gap_and_invalid_follows_positions_are_refused`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs`.
- `a_commit_carrying_an_unknown_branch_cell_cannot_replace_its_missing_fork`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs`.
- `durability_contract_damaged_checkpoint_refuses_rather_than_shortening_history`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery/checkpoint_fallback.rs`.
- `durability_contract_recovery_fails_closed_without_exact_branch_cells`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
- `durability_contract_recovery_rejects_branch_cell_target_without_catalog_artifact`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
- `durability_contract_recovery_rejects_tail_checkpoint_drift_before_mutation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
- `legacy_index_affinity_is_checked_before_replacing_a_live_recovery_target`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs`.
- `envelope_index_affinity_requires_admission_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs`.
- `digest_valid_unissued_scoped_branch_denies_without_replacing_target_state`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/scoped_affinity.rs`.
- `checkpoint_index_artifact_missing_payload_denies_before_recovery`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `checkpoint_recovery_denies_a_digest_valid_foreign_generation_affinity`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `contradictory_descriptor_version_refuses_before_recovery_or_owner_effects`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs`.
- `durability_contract_failure_descriptor_canonical_basis_version_mismatch_is_explicit`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs`.
- `recovery_rejects_one_sided_fork_provenance_in_checkpoint`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs`.
- `recovery_rejects_mismatched_fork_provenance_identity_in_checkpoint`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs`.
- `recovery_rejects_one_sided_fork_provenance_in_tail`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs`.
- `tail_created_lineage_allocator_exhaustion_denies_before_replay`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_allocator_denials.rs`.
- `live_lineage_allocator_exhaustion_denies_before_public_effects`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_allocator_denials.rs`.
- `tail_created_lineage_regression_below_checkpoint_high_water_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery/lineage_regression.rs`.
- `tail_lineage_event_regression_below_checkpoint_high_water_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery/lineage_regression.rs`.
- `checkpoint_duplicate_lineage_event_ids_deny`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `tail_duplicate_lineage_event_ids_deny`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_and_tail_lineage_event_id_collision_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `tail_lineage_commit_and_branch_cross_splices_deny`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_lineage_commit_cross_splice_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `replay_and_durable_lineage_payload_conflict_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_lineage_node_deletion_denies_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_lineage_node_remap_denies_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_lineage_node_swap_denies_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_duplicate_lineage_entity_mapping_denies_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `tail_lineage_event_allocator_exhaustion_denies_before_replay`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs`.
- `unresolved_durable_publication_blocks_descendants_and_recovers_last_checkpoint`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs`.
- `multi_event_reservation_exhaustion_denies_before_public_effects`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs`.
- `native_field_text_missing_tail_dictionary_refuses_without_target_or_source_effect`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/managed_field_text/missing_dictionary.rs`.
- `native_field_text_prior_checkpoint_format_refuses_before_managed_text`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs`.
- `durability_contract_rejects_parent_order_tampering_at_source_admission`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs`.
- `durability_contract_does_not_reconstruct_missing_merge_authority_from_diagnostics`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs`.
- `corrupt_native_checkpoint_is_denied_without_publishing_recovered_state`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs`.
- `tail_replay_rejects_canonical_allocation_targeting_a_live_slot`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `tail_replay_rejects_missing_canonical_allocation_evidence`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `tail_replay_gap_admission_cannot_replace_a_different_canonical_effect`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `metadata_only_merge_rejects_unconsumable_allocation_evidence`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `recorded_component_checkpoint_refuses_chk15_before_source_admission`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs`.
- `semantic_result_bulk_malformed_correspondence_refuses_after_frame_reseal`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/semantic_result/bulk.rs`.
- `durability_contract_recovery_rejects_tail_target_without_immutable_artifact`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/tail_checkpoint_validation.rs`.
- `a_refused_publication_was_offered_to_the_medium_and_moved_nothing_live`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/durable_before_live.rs`.
- `a_refused_position_is_not_consumed`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/durable_before_live.rs`.
- `fork_against_checkpoint_capture` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `fork_against_runtime_fork` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `fork_against_reclamation` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `archive_against_checkpoint_capture` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `archive_against_runtime_fork` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `archive_against_reclamation` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `checkpoint_registry_is_readable_while_a_fork_is_durable_not_yet_live`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `an_artifact_whose_frame_is_damaged_is_refused_at_the_read`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `an_artifact_that_does_not_readmit_refuses_as_corrupt`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_gap_in_the_journal_refuses_as_non_contiguous`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_repeated_entry_refuses_as_non_contiguous`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_tail_that_does_not_continue_from_the_checkpoint_refuses`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_checkpoint_filed_under_the_wrong_position_refuses_as_corrupt`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_damaged_checkpoint_refuses_as_corrupt`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
  These refusal courts and the `branch_transition/tampering.rs` courts were the only readers of the frame
  refusal carried by `TestRecoverySourceRefusal` (`tests/support/durable_medium.rs`); its variants are
  unit-only now, and a restore re-adds the `Commit`/`Transition` frame-refusal payloads.
- `a_durable_publisher_waits_for_the_reservation_instead_of_deferring` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/reservation_wait.rs`.
- `a_cancelled_waiter_stops_before_any_effect` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/reservation_wait.rs`.
- `an_unresolved_publication_installs_nothing_and_refuses_every_later_commit_alike`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs`.
- `reopening_resolves_an_absent_unresolved_commit_as_never_committed`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs`.
- `reopening_resolves_a_present_unresolved_commit_as_committed_exactly_once`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs`.
  C4 owns each of these durability contract/medium roots under Landing 2a's
  per-claim disposition. Restore each root's same-SHA fixture closure (parent
  module helpers, `tests/support/durability.rs` and the original local medium
  controls) as recorded per root in
  `target/lane-evidence/landing2a-durability-contract-claim-disposition.md`.
  This is relocation with the original assertions, scripted fates and no-effect
  oracles, not retirement or weakened evidence. Restore over genuine owned
  media; do not revive the removed runtime-discovery adapter.
  The `cfg(test)` owner control hooks that only these moved roots called were
  removed with them and restore from the same SHA (paths under
  `crates/worth-relational/src/`): `durability/data/branch_root_schema_images.rs::corrupt_schema_root_for_test`, `durability/migration/readmitted_canonical_commit.rs::envelope_mut_for_test`, `history/data/canonical_commit_envelope.rs::rebuild_lineage_basis_for_test`, `history/data/canonical_record_allocation.rs::new`, `history/retention/owner.rs::retired_branch_binding_count`, `indexes/access/pinning/definition_inventory.rs::definitions_snapshot`, `lineage/data/artifacts/checkpoint_artifact.rs::nodes_mut`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::LifecycleSnapshotPause`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::await_admission`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::await_transition_lifecycle_contact_for_test`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::pause_exclusive_snapshot_for_test`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::release`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::transition_lifecycle_contacts_for_test`, `runtime/state/subsystems/history_retention.rs::retired_branch_binding_count`, `runtime/state/subsystems/lineage.rs::event_count`, `runtime/state/subsystems/lineage.rs::node_count`.

- `compile_execution_ready_merge_plan_rejects_missing_source_record`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/compilation_denials.rs`.
- `verify_prepared_merge_execution_rejects_corrupted_compiled_plan`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/compilation_denials.rs`.
- `verify_prepared_merge_execution_rejects_schema_semantic_drift`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs`.
- `verify_prepared_merge_execution_rejects_merge_base_drift`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs`.
- `derive_merge_commit_mutation_plan_does_not_rely_on_raw_lowered_record_arrays`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_d.rs`.
- `authoritative_merge_execution_certification_rejects_metadata_reference_movement`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs`.
- `authoritative_merge_execution_certification_rejects_schema_semantic_drift_after_planning`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs`.
- `compile_rejects_corrupted_non_executable_resolution_class`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_b/executable_admission.rs`.
- `promoted_deleted_on_both_sides_compiles_to_an_explicit_executable_record_variant`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs`.
- `promoted_deleted_on_both_sides_derives_zero_mutation_intent_execution_plan`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs`.
- `promoted_deleted_on_both_sides_executes_through_authoritative_merge_publication`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs`.
- `deleted_on_both_sides_prepared_merge_rejects_schema_semantic_drift`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/prepared_merge_denials.rs`.
  C4 owns the original claim under Landing2a's per-claim disposition.
  Restore each named root's same-SHA owning fixture/module imports and private
  control references recorded in `target/lane-evidence/landing2a-history-compiler-claim-disposition.md`.
  This is relocation with its original guard assertions, not retirement or
  weakened evidence. Restore with genuine owned inputs, without reviving the
  removed local provider/discovery adapters.
  The `cfg(test)` plan-mutation hooks only these merge roots called were removed
  with them. Restore `execution_ready_plan_mut_for_test`,
  `authority_binding_mut_for_test`, `bound_executable_plan_mut_for_test` and
  `replace_bound_plan_and_mutation_plan_for_test` from
  `f9f15bf8c8:crates/worth-relational/src/merge/data/execution/test_support.rs`,
  and `compile_execution_ready_merge_plan_for_test` and
  `replace_bound_merge_plan_for_test` from
  `f9f15bf8c8:crates/worth-relational/src/merge/execution/mod.rs`.


- `retained_merge_strategy_witness_live_authority_denies_internal_forged_strategy_truth`: `restore from 26cc5a58f2:crates/worth-relational/src/tests/history/milestone_7e_phase_i/mod.rs`.
  C4 owns this injected typed-summary/witness or stored-original fault; restore
  the same-SHA `fixtures.rs` beside the root, preserving exact denials.
- `merge_support_inspection_witness_denies_inconsistent_retained_proof_authority`: `restore from 26cc5a58f2:crates/worth-relational/src/tests/history/milestone_7e_phase_l/mod.rs`.
  C4 owns this injected typed-summary/witness or stored-original fault; restore
  the same-SHA `fixtures.rs` beside the root, preserving exact denials.
- `merge_collaboration_truth_denies_cross_family_witness_mismatch`: `restore from 26cc5a58f2:crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs`.
  C4 owns this injected typed-summary/witness or stored-original fault; restore
  the same-SHA `fixtures.rs` and `drift_support.rs` beside the root, preserving exact denials.
- `merge_collaboration_truth_denies_summary_only_and_planner_shortcuts`: `restore from 26cc5a58f2:crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs`.
  C4 owns this injected typed-summary/witness or stored-original fault; restore
  the same-SHA `fixtures.rs` and `drift_support.rs` beside the root, preserving exact denials.

- Disconnected-graph half of `merge_branch_basis_denial_happens_before_planning_for_missing_or_disconnected_branches`:
  `restore from 45b6e87a0a:crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs`.
  Restore `graft_disconnected_branch_head` from that file. C4 owns its forged
  receipt/cell/graph installation and exact missing-base/binding denials. The
  ordinary missing-source half now observes shipping owner binding publicly.
- `replay_rebuild_denies_summary_and_packet_drift`:
  `restore from 45b6e87a0a:crates/worth-relational/src/tests/history/milestone_7e_phase_e/replay_drift.rs`.
  Restore its parent fixture from the same SHA. This injects summary/packet drift
  and checks the private consistency predicate; it never actually ran a rebuild.
  C4 must retain the exact fault assertions without reviving runtime attachment.

- `residency_failure_matrix_nonhead_merge_history_covers_every_door_and_class`:
  `restore from 9addca7179:crates/worth-relational/src/tests/durability/medium/cold_merge_history_matrix/exhaustive.rs`.
  Moved under the coordinator's 2026-11-09 merge-parent ruling. C4 owns the
  cold-read fault matrix for merge history (every door and cause). Its original
  fixture, door dispatch and cold provider are retained at the same restore SHA
  in the parent `cold_merge_history_matrix.rs`, `cold_merge_history_fixture.rs`,
  `cold_recovery_fixture.rs`, `cold_read_matrix.rs`, `cold_source.rs` and
  `cold_source/protected_reads.rs`. This is relocation, not retirement or
  weakened fault assertions.

- C4: the original local-medium checkpoint corruption injection in
  `fintech_failure_injection_helpers_cover_savepoints_replay_and_checkpoint_corruption`:
  restore full original from `bbbce86edd7a7e843ecd1e44a8d3ffbfca2cf52f:crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs`.
  Restore same-SHA `tests/domains/fintech/failure_injection/{mod.rs,durability.rs}`,
  `fixture` (complete tree), `scenarios.rs`, `actions/recovery.rs`, owning module parents,
  `tests/support.rs`, complete `tests/support` and direct production/control closure
  under `crates/worth-relational/src`. Preserve the actual checkpoint damage and
  `damage_filed_checkpoint == true` assertion through genuine owned storage;
  never revive runtime_attachment. The original savepoint/replay checks remain
  mounted as `fintech_failure_injection_keeps_savepoint_and_replay_refusals`.

#### C2 - Cost and bounds: backing and work budgets

- C2: complete private campaigns
  `selected_ack_partial_transfer_preserves_later_cells_and_resumes_same_capture`
  and `selected_ack_exposes_backed_data_before_retirement_and_releases_holds_only_on_completion`:
  restore `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/durability/data/native_checkpoint/tests/selected_ack.rs`.
  Restore both delayed-selection variants
  `native_custody_correction_delayed_canonical_selection_retains_actual_root_and_interrupted_ack`
  and `native_custody_correction_delayed_empty_selection_retains_actual_schema_and_interrupted_ack`
  from the same immutable SHA at
  `crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs`.
  Each restore includes its full original body and helper, same-SHA parent mounts,
  Native checkpoint/canonical publication/region placement/catalog/custody/retirement
  owners and controls, direct runtime/history/schema ancestors, `tests/support.rs`
  and COMPLETE `tests/support` tree under `crates/worth-relational/src`.
  Preserve exact one-unit/C+regions work windows, all per-cell placement and
  source `same_owner`/Arc identities, acknowledged catalog frontiers, Weak/census,
  original root/schema/Full-read and capture-release/reclamation assertions.
  Restore through actual Home-owned capture, physical selection and ACK;
  never revive fixed-generation selection or runtime_attachment discovery.
  Functional Home counterparts stay mounted: selected observer-drop/history/mutation
  recovery, pending selected DATA catalog delta, and both retained Empty/canonical
  original-schema/entity-read journeys through pending ACK/recovery/reopen/advancement.
  The Query original-attempt interrupted-ACK restart court stays mounted too.


- C2: exact `branch_transition_artifacts(...).next().is_none()` in
  `completing_an_older_capture_cannot_replace_the_compacted_transition_basis`.
  Restore the complete original at `bbbce86edd7a7e843ecd1e44a8d3ffbfca2cf52f:crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs`,
  same-SHA parent mounts, checkpoint_capture/checkpointing/append owners,
  checkpoint catalog and branch-transition state/control closure, `tests/support.rs`
  and complete `tests/support` tree under `crates/worth-relational/src`.
  Restore the actual durable compaction and original capture/coverage/budgets
  through genuine owned storage; never revive runtime_attachment. The complete
  private catalog-ordering counterpart stays mounted as
  `older_checkpoint_completion_keeps_the_newest_transition_basis` on Memory.
  Actual Home stale-capture/selected-generation proof remains separately mounted.


Integration protocol restoration refresh (same per-claim disposition; no retired campaign is remounted):
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore/reuse_tests.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/history/commit/artifact_tests.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`
- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs`
Restore the complete same-SHA `crates/worth-relational/src` tree, including original parent mounts, fixture/control/support/producer and reconstruction-lane dependencies. Preserve every original assertion/input/contrast and genuinely owned source boundary when migrating the campaign; never revive runtime_attachment discovery. These references retain integration revisions to the already moved private cost/fault families. Ordinary source functionality remains in the public counterparts.


Generation selection private counter remainder:
Generation case retains original two pre-recovery and two post-recovery definitions, nonreuse identity/generation inequalities, pinned old vs new field values, exact observed generation and candidate entity ids. Only exact generation_payload_reads delta1 and history_inventory_entries delta0 leave the functional case for C2; no public cost proxy. Complete immutable original at `14b6075657193d0f74ce944beab6124b5dec7b3c:crates/worth-relational/src/tests/query/indexes/generation_identity_recovery.rs`, plus same-SHA indexes/mod.rs, tests/support.rs, complete tests/support tree and complete `crates/worth-relational/src` implementation/control closure, preserving original counters/arming/input contrasts when restored through genuine owned source infrastructure; never restore runtime_attachment.


Fresh admission private source/root/schema custody remainder:
Full original fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete including source_plan, all root/schema/source Arc identities, retained-empty/schema registry assertions, unused constraints and wrong receipts at `5d37a209ee227fdef639cbec2c438d82396a7209:crates/worth-relational/src/durability/authority/fresh_source/tests.rs`; restore same-SHA fresh_source.rs/directory, tests/support.rs and complete tests/support tree, direct durability/runtime/schema control/source ancestors. Preserve original arming/observations; never restore runtime_attachment discovery. This is a C2 split under the coordinator per-claim ruling.


Pinned-reader restoration private remainders:

Full original native_restore_work_counts_revised_history_sibling_roots_and_partition_images and divergent_sibling_is_not_reused_as_the_storage_mirror including shares_first_generation/shares_partition_generation: `5b18804fbd34c814d69aefd527653b11e3d2da59:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`. Restore the same-SHA runtime_rebuild.rs and runtime_rebuild directory, `crates/worth-relational/src/tests/support.rs`, complete tests/support tree, and direct durability/history/runtime source/control ancestors. Preserve every section/work/directory/partition and pointer comparison, complete original arming/observations. Restore through genuinely source-owned infrastructure; never resurrect runtime_attachment discovery.


Capture, dictionary and lineage private remainders:

All these original roots and fixtures are unchanged at f9a53a5d17e01562d503b39b4627252bb63d8639:

- C2 full original native_checkpoint_retains_all_admitted_capture_owners_and_restores_editable_truth, including zero/cancel/retained-grant pressure and every owner/charge/pointer/section assertion: `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src/tests/durability/contracts/native_checkpoint/allocation.rs` with request() in that file and its original native_checkpoint.rs/contracts/mod.rs parents.
- C2 full original unpublished_client_keys_leave_checkpoint_dictionary_equal_and_published_keys_restore, including before/after symbol and record frontiers, all image equality/normalization and actual symbol demand/membership: `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src/tests/durability/contracts/native_checkpoint/speculative_symbols.rs`, same original parents.
- C2 full original checkpoint_restore_protects_published_lineage_identities_in_each_lane and inflight_tail_replay_resumes_after_the_largest_durable_lineage_values, including RestoreLane/discard_after_a_durable_anchor, exact live/discarded/recovered frontiers, cold directory/empty nodes, and tail_log.len()==1: `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src/tests/durability/medium/lineage_frontiers.rs` and original medium/mod.rs parent.

Each restore includes the complete immutable shared fixture/control trees `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src/tests/support.rs`, `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src/tests/support`, plus direct owner/canonical/control ancestors from `f9a53a5d17e01562d503b39b4627252bb63d8639:crates/worth-relational/src`. Preserve complete original claims/arming/observations when restoring through genuinely owned source boundaries; never revive the runtime-discovery adapter. Copy these exact entries into the spec before removing original roots.

Index, merge, bulk and empty-head private remainders:

All original nine roots are unchanged at622f101e99a3bda26132bebf19c2f3c318a7b6f3. Use that immutable revision for every referenced tree; no moving HEAD is a restore authority.

- C2, `checkpoint_index_artifact_survives_tail_replay_without_envelope_caches`: exact `derived_index_artifacts.is_empty()` and `derived_index_checkpoint_format == 2`, restore from `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`. Fixture closure is its original name_index/build/build_on_branch definitions in that same file, its original `contracts/mod.rs` parent, and the immutable support tree below.
- C2/C4, `durability_contract_replays_merge_from_typed_authority_when_diagnostics_are_absent`: exact populated-versus-omitted authoritative_nested_bytes comparison and original rewrite_commit_envelope callback/resealed frame, restore from `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs`. Fixture closure is the original `contracts/mod.rs` parent, support tree below (especially durable_medium/tamper.rs), and `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/history/data/canonical_commit_envelope/control_allocation.rs` with its same-SHA parent module. Restore the complete callback, including authority take/restore and empty-diagnostic assertion, rather than a no-op or count proxy.
- C2, `semantic_result_bulk_entities_and_relations_seal_and_readmit_original_bindings`: actual readmitted envelope symbol_texts dictionary contains bulk-input-00 under original PreferInterned producer, restore from `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/durability/contracts/semantic_result/bulk.rs`. Fixture closure includes publication() in that file, `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/durability/contracts/semantic_result.rs`, its original parent and immutable support tree below.
- C2, `durability_contract_empty_checkpoint_restores_a_live_retained_head`: exact head_installs == 1, restore from `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery/empty_head.rs`. Fixture closure is `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`, its original parent and immutable support tree below.

Shared exact fixture/control trees: `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/durability/contracts/mod.rs`, `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/support.rs`, and `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src/tests/support` (complete runtime/schema/records/aspect/history/owner-input/durability/local-medium descendants). All production owner controls and direct parents come from the same immutable `622f101e99a3bda26132bebf19c2f3c318a7b6f3:crates/worth-relational/src` tree. On restoration migrate source boundaries through genuine owned media, preserving every original assertion and arming/observation/control closure; never revive runtime-discovery adapter authority. These original private claims remain required at their restoration courts; their nine public functional counterparts stay mounted.


- Exact old-layout portion of `memory_frame_preserves_original_payload_and_scalar_tuple_observations`:
  `restore from 86e84dc5cd:crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs`.
  Restore the original complete OldBacking (both variants and order), OldState,
  OldCell, original Memory construction/drop diagnostic and both exact size
  assertions with the same-SHA parent type imports. The functional original
  Memory payload length, digest, real backing and full-frame/payload distinction
  remain in the mounted genuine owner unit. Never delete the Journal variant
  to fit a smaller layout oracle; this physical comparison belongs to C2.


Remaining pure cleanup roots (per-claim Landing 2a disposition):

- C4: `branch_root_image_versions_refuse_before_restoration_and_current_recovers`,
  introduced on the authorized integration base: restore the complete unsupported
  version mutation/refusal/current-format control from
  `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/unsupported_root_version.rs`,
  its same-SHA `branch_root_checkpoint.rs` parent and complete
  `crates/worth-relational/src` fixture/codec/authority closure. This is the original
  private malformed-plan campaign; ordinary current-format public recovery
  remains mounted in `checkpoint_branch_targets.rs`.


- `positioned_agreement_uses_original_work_budget_and_preserves_authoritative_digest`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/contracts/canonical_commit_artifact.rs`.
- `native_custody_correction_last_actual_capture_abandonment_releases_shared_root_custody`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs`.
- `equal_exact_root_reuses_the_readmitted_partition_substrate`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`.
- `native_custody_correction_cross_lane_acquire_release_has_one_actual_capture_cut`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/retention/component_cut_tests.rs`.
- `native_custody_correction_current_lane_inventory_is_collected_inside_actual_cut`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/retention/component_cut_tests.rs`.
- `an_indexed_envelope_remains_cold_until_explicit_readmission_and_loads_once`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `relinking_other_envelopes_preserves_an_index_even_while_its_body_is_in_flight`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `settlement_releases_a_journal_flight_but_preserves_memory_history_and_position`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `positioned_demand_owns_its_result_across_journal_settlement`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `journal_publication_keeps_journal_placement_without_memory_frame`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs`.
- `canonical_artifact_accounting_excludes_nested_diagnostic_perturbation`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests.rs`.
- `populated_derived_index_artifacts_only_change_the_optional_cache_lane`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests.rs`.
- `strategy_input_clone_retains_only_its_original_decoded_bytes_grant`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/strategy_readmission.rs`.
- `strategy_descriptor_clone_retains_only_its_original_decoded_locator_grant`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/strategy_readmission.rs`.
- `shared_readmitted_envelope_clones_keep_the_original_allocation_and_grant_until_last_drop`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/shared_readmission.rs`.
- `actual_weak_position_retains_only_the_fixed_original_control_bill_until_last_drop`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/shared_readmission.rs`.
- `embedded_checkpoint_preserves_ignored_unknown_nested_fields_without_retaining_their_heaps`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/shared_readmission.rs`.
- `result_snapshot_cleanup_registry_contention_preserves_exact_cell_and_competing_task`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/runtime/state/subsystems/visibility/snapshot_handles/published_registry/original_cleanup/tests.rs`.
- `checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs`.
- `seeking_past_an_acknowledged_index_prefix_never_demands_its_journal_bodies`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/canonical_commit_artifact/retained_export/skip_tests.rs`.
- `structural_identity_recovery_preserves_current_evidence_and_queries`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/inspection/structural_identity.rs`.
- `complexity_budget_schema_transition_classification_is_changed_atom_bounded`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs`.
- `complexity_budget_subscriber_resume_continuity_is_boundary_local`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs`.
- `complexity_budget_milestone5_closeout_keeps_schema_cdc_and_recovery_boundary_local`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs`.
- `complexity_budget_merge_planning_reports_request_shaped_work`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/merge_budgets.rs`.
- `complexity_budget_merge_execution_reports_admitted_records_and_emitted_mutations`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/merge_budgets.rs`.
  Restore each root's complete same-SHA parent/descendant fixture and control
  closure, including its original local medium script and owner units; migrate
  the source boundary through genuine owned media without reviving the retired
  runtime-discovery adapter. All original guards remain required.

  The merge execution root's four public semantic assertions remain in
  `merge_source_copy_identity.rs::source_only_merge_emits_one_new_entity_with_an_independent_identity`.
  C2 restores its full original cost/contract campaign and fixture closure.


B2 schema/history consumer split (coordinator per-claim Landing 2a ruling):

- `durability_contract_recovery_rebuilds_branch_head_root_obligations` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs`: exact active-snapshot and branch pin census.
- `durability_contract_recovery_preserves_inspection_truth_bundle` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs`: retention_summary and record_retention fields of the complete original bundle; keep all original producer and inspection helpers.
- `tail_replay_consumes_canonical_reused_record_allocation` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`: exact reclaim limit, one reusable slot, same local slot and incremented generation; public current/historical truth is ported.
- `durability_certification_recovery_authority_continuity_is_explained_and_counted` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs`: exact digest-parity and mismatch counters, alongside original diagnostic producer.
- `durable_recovery_and_schema_mismatch_test` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/schema_transition_continuity.rs`: actual recovered digest-parity work counter; all semantic descriptor and refusal assertions stay public.
- `native_field_text_checkpoint_then_tail_recovers_exact_json_after_original_owners_drop` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs`: exact Symbol representation, charged managed payload and System client-key custody; retain read_exact and original request policy closure.

- Fixture closure for `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`:
  `restore from 0615dd733c:crates/worth-relational/src/history/retention/lane/retained_release/tests.rs`
  (`assert_recovered_cut_and_lane_busy`), with its same-SHA exact re-export lines
  in `history/retention/mod.rs`, `history/retention/lane.rs`, and
  `history/retention/lane/retained_release.rs`. Its final caller moved to C2;
  the other external-cut fixture remains live. The original source tests above
  also reference the same-SHA local-medium `head`, `transition_head`,
  `drop_artifact` and journal `remove` fixtures. Restore those physical/fault
  observations through the actual owner/provider, without reviving adapters.

- Associated-original checkpoint metadata/body nonresidency portions:
  `associated_effects_discover_cold_subject_without_reading_unrelated_history`
  (`restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/associated_effects.rs`)
  and `native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`
  (`restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/associated_effects_discovery.rs`).
  C2 preserves the exact per-cell before/after resident census, with original
  same-SHA fixture closure. Actual metadata/semantic-result behavior stays in
  public `association_checkpoint_journeys.rs`.
- `caller_association_bounds_and_rebinding_refuse_without_retaining_publication`
  (pending allocation/route census) and
  `caller_association_survives_cold_checkpoint_history_and_journal_tail`
  (selected directory, journal absence and cold-body posture):
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/caller_association.rs`.
  Restore original `propose`/`publish` fixture closure for those physical claims.
  Wire/bounds and embedded original codec remain Native value units; all public
  binding and cold original-frame equality remains in `caller_association_journeys.rs`.
- `recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor`
  and `recorded_component_checkpoint_readmits_original_empty_predecessor`,
  private resident/cold-root and full physical-root demand portions;
  `recorded_component_checkpoint_omits_released_components_and_plain_observations`
  (whole retained-root census) and
  `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`
  (whole private pin/release/stop accounting campaign):
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs`.
  Restore `original_custody`, `Original`, capture/recovery and retirement fixtures
  at the same SHA. C2 must supply honest public pin/allocation observations;
  never revive the local provider adapters. The Canonical/Empty original
  descriptors, readable snapshots after maintenance, semantic result,
  historical-currentness refusal and no-republication guards stay public in
  `recorded_original_checkpoint_journeys.rs`. The actual unrelated-body deletion
  remains a C4 component of this original campaign, as listed below.
- `a_recovered_runtime_publishes_its_next_commit_at_the_next_position`,
  original/reopened cold-source allocation pointer identity only:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/medium/recovered_publication.rs`.
  Public source bytes/limits and actual recovered originals, receipt/coverage,
  no recovery publication, next ID/parent/position and second reopen remain in
  `medium_recovered_continuation.rs`.
- `checkpoint_size_tracks_branch_state_not_covered_transitions`, manifest
  growth bound and covered resident-log release portions:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs`.
  Restore both exact 32-branch lifecycle scripts, 64/128 counts, captured
  sequences, empty covered logs and `through_archive_and_pending_deletion <= direct + 32`.
  The same lifecycle inputs and exact recovered sequence continuation stay
  public in `checkpoint_transition_continuation.rs`.


- `checkpoint_recovery_readmits_shared_root_once_for_sibling_heads`, physical
  single image and exact readmitted Root pointer sharing:
  `restore from e4d3b01c8d:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
  C2 restores the original fixture closure; the two readable branch contents
  and equal admitted targets remain public in `checkpoint_branch_targets.rs`.
- `durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits`,
  exact retained envelope census and original partition-image backing pointer:
  `restore from e4d3b01c8d:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
  C2 restores these original allocation assertions; exact checkpoint/tail
  coverage, receipts and old envelope readability remain in public
  `checkpoint_tail_and_indexes.rs`.
- `acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot`,
  `foreign_recovery_and_foreign_candidate_are_refused_before_effects`, and
  `moved_source_and_stale_candidate_cannot_extend_recovery_lineage`, private
  published/preparation snapshot-handle census portions only:
  `restore from e4d3b01c8d:crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs`.
  Restore the original same-SHA fixture closure for C2 allocation/custody
  observation. All exact public authority admission/refusal/retry and unchanged
  descriptors/counts remain in `checkpoint_successor_authority.rs`.


- `controlled_resident_cell_keeps_exact_charge_until_last_real_weak` and
  `controlled_resident_cell_refuses_exact_control_before_any_new_arc`:
  `restore from f9f15bf8c8:crates/worth-relational/src/storage/residency/cell/resident_full/tests.rs`.
  These two assert the exact request-funded charge of `resident_for_request`
  (exact accounting, C2 by the sharing-bound ruling). The constructor is live:
  the master merge made the Full `PreparedResidentRegion::initial`
  (`storage/residency/region_initialization.rs`) its production caller, so C2
  restores these roots against that producer's bound.
- The Full region courts are not moved. On the live producer
  `PreparedResidentRegion::initial`,
  `controlled_full_region_matches_current_kind_preimages_and_truth` and
  `controlled_full_commitment_clone_retains_only_its_actual_node_backing` stay in
  `storage/residency/region_publication/full/tests.rs` and must pass.
  `PreparedResidentRegion::initial_for_request` had no production caller and was
  deleted with the 32 Native dead-code diagnostics
  (`target/lane-evidence/landing2a-dead-code-deletion-notes.md`).

- Instrumentation half of `execute_prepared_merge_reports_execution_counters_and_structural_summary`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_e/execution_counters_diagnostics.rs`.
- Instrumentation half of `execute_prepared_merge_records_attempt_without_success_on_failure`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_e/failure_attempt_diagnostics.rs`.
- Instrumentation half of `native_merge_touches_only_changed_revision_and_both_index_membership_keys`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/merge_descriptive_touches.rs`.
- Instrumentation half of `built_in_monotonic_counter_merge_is_auto_resolved_with_inline_value_and_recovery_parity`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/monotonic_counter_resolution.rs`.
- Instrumentation half of `disjoint_rewire_neighborhoods_do_not_escalate_to_topology_region_conflict`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire.rs`.
- Instrumentation half of `topology_region_conflict_detection_reports_bounded_neighborhood_counters`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_region.rs`.
  Each of these roots is split under Landing 2a's per-claim disposition. Its
  public semantic claim is ported to the certification court
  (`relational/durable_publication/merge_*`); C2 owns only the exact
  instrumentation-counter, complexity-delta or private resident-Root assertions
  named for the root in
  `target/lane-evidence/landing2a-history-compiler-claim-disposition.md`.
  Restore the whole original root and its same-SHA fixtures, with genuine owned
  inputs and without reviving the removed runtime-discovery adapter.

- `branch_roots_with_one_schema_deduplicate_the_durable_carrier`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs`.
- `replay_reads_only_the_gap_after_its_checkpoint`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs`.
- `durability_tail_replay_releases_each_commit_snapshot_beyond_live_capacity`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
- `deleted_branch_reader_retains_only_its_exact_scoped_generation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/deleted_branch_reader.rs`.
- `historical_reader_from_live_head_survives_head_retirement`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/historical_reader.rs`.
- `reclamation_and_checkpoint_keep_sibling_head_generation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `repeated_edits_checkpoint_only_the_live_index_payload`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `reclamation_keeps_a_pinned_old_reader_until_release`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `repeated_closed_branches_leave_no_latest_generation_residue`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `repeated_global_same_basis_builds_reclaim_duplicates_but_keep_pinned_root`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/global_generation_reclamation.rs`.
- `native_field_text_checkpoint_readmission_denial_keeps_bytes_and_target_root_intact`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs`.
- `repeated_native_captures_do_not_retain_checkpoint_images`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs`.
- `synthetic_native_restore_and_image_digest_benchmark`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/native_checkpoint_restore_benchmark.rs`.
- `checkpoint_restores_burned_append_frontier_beyond_all_retained_roots`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `recovery_releases_orphaned_append_reservation_without_reusing_its_gap`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `recovery_returns_orphaned_reclaimed_reservation_to_reuse`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `durability_contract_branch_heads_do_not_mutate_legacy_record_pins`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs`.
- `embedded_restore_allocation_grows_linearly_in_relation_bearing_regions`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/embedded_cost.rs`.
- `an_attachment_the_medium_moved_out_of_publishes_nothing`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/single_owner.rs`.
  C2 owns each of these durability contract/medium roots under Landing 2a's
  per-claim disposition. Restore each root's same-SHA fixture closure (parent
  module helpers, `tests/support/durability.rs` and the original local medium
  controls) as recorded per root in
  `target/lane-evidence/landing2a-durability-contract-claim-disposition.md`.
  This is relocation with the original assertions, scripted fates and no-effect
  oracles, not retirement or weakened evidence. Restore over genuine owned
  media; do not revive the removed runtime-discovery adapter.

- `verify_prepared_merge_execution_does_not_increment_planning_counters`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/verification_counters.rs`.
- `verify_prepared_merge_execution_reports_verification_counters_without_planning_work`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/verification_counters.rs`.
- `unrelated_relation_additions_do_not_inflate_topology_region_detection_counters`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire/neighborhood_detection.rs`.
  C2 owns the original claim under Landing2a's per-claim disposition.
  Restore each named root's same-SHA owning fixture/module imports and private
  control references recorded in `target/lane-evidence/landing2a-history-compiler-claim-disposition.md`.
  This is relocation with its original guard assertions, not retirement or
  weakened evidence. Restore with genuine owned inputs, without reviving the
  removed local provider/discovery adapters.


The following adapter-dependent Native performance campaigns move to C2 under
Landing 2a's cost/growth disposition. Their assertions and durable posture are
preserved at the restore references. Restore each root's descendant modules
from the same SHA and its shared imports from `tests/performance_profiles.rs`;
the exact mount closures and metric/scale evidence are recorded in
`target/lane-evidence/landing2a-performance-claim-disposition.md`. The thirteen
campaigns outside the adapter closure remain unchanged in this tree.

Campaign-only fixtures removed with those roots also restore from `ee7aef9014`:
`tests/performance_profiles/measurement_support.rs`, `dense_patch_record_count`
in `tests/performance_profiles/invariant_support.rs`, `PerfMetricValue::is_null`
in `tests/performance_metrics.rs`, the ten performance adapters/exports in
`tests/domains/fintech/mod.rs`, and the two Chip variants/arms of
`PerfDiagnosticsPolicy` in `tests/support/runtime.rs` (paths relative to
`crates/worth-relational/src/`). Remaining shared fixtures and all ordinary
Fintech scenarios remain. C2 must restore these claims with actual owned durable
media through the public testkit, preserving the claims without reviving the
retired runtime-discovery adapters.

- `perf_artifact_recoverability_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/artifact_recoverability_matrix.rs`.
- `perf_chip_simulator_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/chip_simulator_matrix/mod.rs`.
- `perf_commit_delta_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/commit_delta_matrix.rs`.
- `perf_durability_append_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/durability_append_matrix.rs`.
- `perf_geometry_kernel_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/geometry_kernel_matrix/mod.rs`.
- `perf_hot_cold_path_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/hot_cold_path_matrix/mod.rs`.
- `perf_index_parity_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/index_parity_matrix.rs`.
- `perf_merge_lineage_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/merge_lineage_matrix/mod.rs`.
- `perf_recoverability_policy_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/recoverability_policy_matrix.rs`.
- `perf_replay_recovery_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/replay_recovery_matrix.rs`.
- `perf_sustained_load_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/sustained_load_matrix/mod.rs`.
- `perf_workflow_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/workflow_matrix/mod.rs`.

- `request_root_refusal_preserves_original_and_releases_partial_backing`:
  `restore from 2a51552fc5:crates/worth-relational/src/branch/root_tests/request.rs`.
  Moved under the coordinator's 2026-11-09 Landing 2a ruling, not retired or
  weakened. It proves the specific unchanged-root replacement allocation refuses
  at controller + `size_of(root)` - 1, releases partial backing, preserves the
  original, and honours cancellation. C2 must provide the public
  allocation-refusal observation before restoring this claim.

- `publication_cost_distinguishes_new_and_reused_schema_authority`: `restore from b736be6f47:crates/worth-relational/src/branch/root_tests.rs`.
- `replay_pin_of_a_cold_partition_is_typed_and_has_no_effect`: `restore from b736be6f47:crates/worth-relational/src/tests/durability/medium/cold_isolation.rs`.
- `resident_pin_custody_refuses_unread_mirror_frames_after_exact_source_admission`: `restore from b736be6f47:crates/worth-relational/src/tests/durability/medium/cold_pins.rs`.
- `selected_branch_views_preserve_cold_probe_and_demand`: `restore from b736be6f47:crates/worth-relational/src/tests/durability/medium/cold_invariants.rs`.
  These courts retain private physical pin actions/counts, resident-only probe
  state, or exact publication/materialization accounting. Restore the original
  claims in C2 through its honest allocation/residency observation; they are not
  retired. Public semantic read/failure courts remain in the happy tree.

- C2: the Graph and Divergence subcases of
  `residency_failure_matrix_lineage_event_doors_preserve_every_failure_class`:
  restore from `ea2ffee250:crates/worth-relational/src/tests/durability/medium/cold_lineage_event_matrix.rs`
  and `ea2ffee250:crates/worth-relational/src/tests/durability/medium/cold_lineage_event_matrix/exhaustive.rs`.
  These collectors exist only under cfg(test), with no shipping public door.
  Seven distinct shipping owner paths, plus the two original aspect-access
  forwarding aliases mapped to their identical public delegates, retain their full
  typed-refusal, repair/stickiness, no-publication and frontier assertions in
  the external cold-residency court.
- C2: private zero-resident-node/event preconditions from the lineage fixture
  helpers used by the migrated concurrent and integrity courts: restore from
  `ea2ffee250:crates/worth-relational/src/tests/durability/medium/cold_lineage_fixture.rs`
  and `ea2ffee250:crates/worth-relational/src/tests/durability/medium/cold_lineage_event_fixture.rs`.
  The external courts preserve zero physical reads at reopen and all original
  public result, byte-integrity, concurrency and source-custody assertions.

- C2: the removed `resident_replay_pins` resident-only fixture hook:
  `restore from ea2ffee250:crates/worth-relational/src/visibility/pins/pin_authority/replay_pins.rs`.
  Its only callers were the already-moved private pin courts; the shipping
  funded `demand_replay_pins` and its resident implementation remain unchanged.

- C2: the exact cached-Head/absent-per-runtime-dictionary preparation window in
  `metadata_pin_refuses_shared_head_until_its_runtime_has_the_aspect_meanings`:
  `restore from 427700edd0:crates/worth-relational/src/tests/durability/medium/cold_metadata_pins.rs`.
  Preserve private `demand_region_head_cached`, its no-read assertion and the
  original `SymbolsNotResident` refusal. Its public read/pin suffix is retained
  as `fork_metadata_pin_after_public_head_read_preserves_original_aspect_revision`.
- C2: the two direct Cold residency probes in
  `residency_retained_kind_demands_only_its_cold_head_and_preserves_damage`:
  `restore from 427700edd0:crates/worth-relational/src/tests/durability/medium/cold_materialization_doors.rs`.
  Preserve the before-kind Head-Cold and after-kind Full-Cold observations.
  Actual suspension, retained kind and typed storage refusal remain in
  `retained_kind_after_public_suspension_preserves_kind_and_storage_refusal`.
  Original fixture dependency: `restore from 427700edd0:crates/worth-relational/src/tests/durability/medium/cold_current_fixture.rs`.
  These private claims are moved, not retired; a public I/O count is not their
  physical residency/dictionary observation.


- C2: resident node/event cardinalities, exact per-runtime event-index population,
  and query-index nonmutation from the mixed lineage courts:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs`
  and `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs`.
  Keep all zero/unchanged/admitted count, absent-node and shared-page warmup
  assertions; the external lineage semantics, point guards and capture-row
  courts retain original resolution/digest, member noninterference, sticky
  typed failures, public frontier/offer invariance and serialized row coverage.
  Original zero-node/zero-event preparation also belongs to C2:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_event_fixture.rs`.
  The capture's exact no-load/shared-index preparation and no member-read suffix
  are resident-state/cost claims; C2 restores them, not a public semantic proxy.
- C2: Graph/Divergence subcases and the exact retry-resident-node predicate in
  `residency_failure_matrix_lineage_doors_preserve_every_failure_class`:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_matrix/exhaustive.rs`
  with node-door setup in `cold_lineage_nodes.rs` above. Seven shipping doors
  keep all six failure classes and original successful/sticky/retry/publication
  consequences in the external node matrix. The fixture's resident-zero setup:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_fixture.rs`.
- C2: exact zero-node observation after cold mirror refusal in
  `lineage_point_guard_preserves_a_cold_mirror_head_failure`:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_observation_reads.rs`.
  Public historical inspection preserves the live mirror-lineage helper's exact
  typed/sticky refusal; the paired retained-observation court preserves all four
  original exact-root guards and no publication effects.
- C2: physical replay pin counts, direct historical-root Full/cold-mirror probes,
  replay-retention census, private cache diagnostics and exact cache-clear window:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_pins.rs`.
  Public paired courts are `replay_pinning_preserves_sticky_mirror_refusal_without_publication`
  and `replay_retention_releases_original_references_without_cold_reads`.
  Keep C2's exact source-state/pinned-record setup and failed-installation census.
- C2: one-pin/zero-pin census in `concurrent_replay_references_share_one_physical_pin_set`:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/replay_pin_concurrency.rs`.
  Its public two eight-worker retain/release phases, all return values and final
  false release remain in `concurrent_replay_references_release_each_original_reference_once`.
- C2: exact discarded-round materialization and adjacency-lease counters, including
  resets and accepted-round equality, in
  `discarded_visibility_rounds_settle_only_accepted_materialization_and_leases`:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_visibility_round_counters.rs`.
  Public cold/warm nonempty result equality and actual additional carrier reads
  remain in `partially_cold_visibility_reads_preserve_nonempty_warm_results`.
  All these private assertions move to C2; none is retired or weakened.

- C2: now-uncalled resident-node fixture probe used by the moved node matrix:
  `restore from 81d41f5575:crates/worth-relational/src/runtime/state/subsystems/lineage.rs`.
  Restore its cfg(test) `LineageSubsystem::node` only with the actual resident
  assertions above; the shipping counted semantic node read remains unchanged.


- C2: exact Weak canonical-body lifetimes, resident envelope/payload census and
  private reclamation setup in the three mixed envelope-lifetime courts:
  `restore from 1403ddb275:crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs`.
  Originals: `settled_live_nonhead_envelopes_release_only_when_a_durable_copy_exists`,
  `released_live_history_remains_in_external_capture_and_retained_recovery`,
  `recovered_nonhead_tail_envelopes_release_their_actual_journal_backed_bodies`.
  Public counterparts retain original patch readability in memory/durable cases,
  all original commit identities after both recovery routes, the <32 source-read
  bound, zero initial tail-source reads and receipt equality. Public reclamation
  and the original <4 retired-root observation stay in the happy tree.
- C2: provisional packet/slot-scan/materialization counter snapshots and exact
  accepted-round equality in `invariant_storage_refusal_is_sticky_and_does_not_settle_provisional_counts`
  and `cold_and_warm_invariant_reads_settle_one_round_of_evidence`:
  `restore from 1403ddb275:crates/worth-relational/src/tests/durability/medium/cold_invariants.rs`.
  The paired public invariant courts keep every nonempty/result-equality,
  typed-refusal, first/warm/sticky I/O, selected-History noninterference and
  no-publication assertion, including the unchanged third Head/Full court.
- C2: exact private dictionary snapshot and provisional packet/scan counters in
  `candidate_invariant_discovery_refuses_stickily_without_publishing_round_effects`,
  Main-Full/remote-Cold probes and arena-slot floor in the candidate fixture,
  plus exact abandoned-allocation slot comparisons/headroom/settled counters in
  `a_region_retry_spends_the_applied_draft_identity_and_settles_one_round`:
  `restore from 1403ddb275:crates/worth-relational/src/tests/durability/medium/cold_candidates.rs`.
  Its paired failure court preserves Validation/Candidate/Commit refusals,
  sticky I/O and original diagnostic Vec equality through the public owner;
  its paired successful court preserves both changed-entity observations,
  actual created record values and <24 source reads. Public diagnostic contents
  are not moved to C2. Source recipe remains mounted for its remaining callers.
- C2: exact Cold/Resident Head probes and physical occupied-slot/generation
  enumeration in `head_probes_admit_each_forks_shared_region_without_reading_history`
  and `cold_forks_edit_existing_ring_records_in_both_orders_and_checkpoint`:
  `restore from 1403ddb275:crates/worth-relational/src/tests/durability/medium/cold_fork_edits.rs`.
  Public paired courts keep missing-History Head name reads in both fork orders,
  both edit orders, exact updated/deleted entity values, real relation creation,
  standalone checkpoint/recovery, partition4 relation count>=2 and per-ID
  effective relation equality. These private subclaims move, not retire.

- C2: private per-runtime interner absence in current-scan cells of
  `residency_failure_matrix_full_read_subjects_cover_every_door_and_class`:
  `restore from 8283728aff:crates/worth-relational/src/tests/durability/medium/cold_read_matrix/exhaustive.rs`.
  The public counterpart keeps all 47 doors, three subjects, six causes, exact
  refusal/retry behavior, target reads, current-scan history noninterference and
  publication invariance. The existing request-funded version read is now public.
- C2: exact receiver dictionary absence/admission/resolved text in
  `every_full_read_door_admits_its_runtime_dictionary_or_preserves_page_damage`:
  `restore from 8283728aff:crates/worth-relational/src/tests/durability/medium/cold_symbol_read_doors.rs`.
  All original public success/refusal, sticky I/O, Head identity and frontier
  observations remain in `full_read_doors_preserve_history_symbol_refusals_and_current_head`.
- C2: private Full/Head probes, metadata-history length, dictionary contents and
  <400 entry bound, and shared-Full/receiver-cold preparation in
  `head_symbols_never_cover_full_history_or_inherit_its_page_damage`:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs`.
  Public paired court retains actual current field identity, Full typed refusal,
  sticky/no-extra-I/O retry, unchanged Head, historical values and frontier.
  Public history entries do not replace the private metadata-history claim.
- C2: private exact-basis representation assertion `view.is_exact_basis()` in
  `residency_failure_matrix_index_doors_preserve_every_failure_class`:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_index_matrix.rs`.
  Public issued-version equality, all 14 doors/six causes, no-load generation
  residency, exact generation/entry equality, successor bounds, reservation
  counters and publication/I/O assertions remain in the public matrix.
- C2: exact resident interner text and cold-envelope cache probes in
  `historical_index_reconstruction_uses_the_old_value_and_full_cold_history` and
  its `assert_cold` preparation:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix.rs`;
  matrix cold-envelope assertion:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix/exhaustive.rs`.
  Both public counterparts preserve genuine old source/rows, generation bounds,
  all five targets/six causes, exact unconsulted-envelope reads, reservation
  counters and no publication. The actual single carried MVCC root is checked
  against an issued main observation; an old receipt is not inferred to be a root.
- C2: whole private allocation/inventory court
  `residency_cold_index_frontier_prevents_reissue_before_any_image_demand`:
  `restore from 8283728aff:crates/worth-relational/src/tests/durability/medium/cold_index_frontier.rs`.
  Restore all four actual IndexSubsystem reserve/publish/frontier/inventory
  sequences with their zero/positive source reads. Public actual builds retain
  monotonic generations and the original <old+16 bound; they do not stand in for
  these private allocator actions. No claim is retired.

- C2 (2026-11-09 Landing2a move ruling, not retirement): exact mirror-Head
  residency preparation, capture layer/input bounds and >=2 consolidation
  count in `checkpoint_restart_cycles_preserve_full_reads_and_retention`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/checkpoint_restart_bound.rs`.
  Its public counterpart preserves three initial updates, 32 iterations, both
  genuine recoveries per iteration, selected-root and mirror Head exclusion,
  exact selected New/Carried membership (unselected bytes disappear), retention
  and exact record comparison. It does not claim to reproduce private Head-only
  preparation or prove repeated consolidation.
- C2: shared merge fixture's exact resident dictionary absence in
  `recovered_shape`, used by the two basic merge courts and member/index matrices:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_merges.rs`.
  Public counterparts retain all typed refusals/retries, source reuse, effects,
  adopted/changed counts, memory-reference records and exact index key/entity
  parity. Functional contributing-commit provenance remains HAPPY in the two
  separate merge-history courts; it is not moved with this dictionary claim.
- C2: exact receiver dictionary snapshot equality and provisional preparation
  packet/invariant slot-scan work accounting in
  `residency_failure_matrix_remote_invariant_heads_cover_every_preparation_door_and_class`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_preparation_failure_matrix/exhaustive.rs`.
  Restore original Main Full/remote Cold/floor setup with it from
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_candidates.rs`.
  Both public matrices retain Validation/Candidate/Commit, all six causes,
  diagnostics Vec equality, sticky/transient I/O, publication effects
  and exact History noninterference. Public outcomes do not replace private work.
- C2: private accepted-round adjacency lease/materialization counter equality in
  `projection_adjacency_discovery_settles_only_one_accepted_round`, plus exact
  resident dictionary text resolution in the incoming-door fixture:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs`.
  Public counterpart `cold_and_warm_projection_adjacency_reads_preserve_actual_edges`
  retains genuine cold/warm nine-door reads and adds warm no-I/O. The public
  nine-door/six-subject/six-cause matrix retains all current/historical, callbacks,
  exact failures/retries, unconsulted sources and publication assertions. The
  two genuine index DATA readers include audit-retained edges, never Live-only
  replacements. Exact private work accounting stays with this C2 original.
- C2: exact interner absence before/after anchor read in
  `the_remote_ring_label_stays_cold_until_its_incoming_edge_is_read`, and private
  dictionary resolution in its real paged-edge preparation:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_ring_dictionary.rs`.
  Restore the exact original physical shape from
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_isolation.rs`.
  Public counterpart preserves incoming-edge typed refusal and proves the actual
  page is unconsulted before/after anchor read, then physically demanded.
- C2: exact resident dictionary snapshot equality in
  `cold_symbol_buckets_refuse_each_preparation_door_without_journal_effects_and_stay_damaged`,
  and arena spent-slot/floor/+16 headroom plus <population/2 resident-dictionary
  assertions in `a_cold_symbol_probe_spends_the_abandoned_record_identity_and_reads_only_touched_buckets`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_symbols.rs`.
  Matrix dictionary equality:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_symbol_failure_matrix/exhaustive.rs`.
  Preserve original 600 client-key entities plus selected collision key. Public
  counterparts keep all five doors including Preview, all six causes, exact
  diagnostics/effects/sticky/retry behavior and successful actual relation with
  original <16 source reads. Public successful case is named
  `cold_symbol_publication_reads_only_touched_buckets`, not a spent-slot claim.
- C2: exact resident symbol absence and private direct Full-warming preparation in
  `cold_head_warm_and_full_warm_uniqueness_reuse_the_original_paged_field_identity`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_unique_symbols.rs`.
  Public counterpart keeps all three warmth postures (actual public snapshot
  Full read), original producer-admitted symbol identity, exact public duplicate
  conflict payload equality and all frontier/offers/reservation assertions.
- C2: resident dictionary inventory/demand/resolution loop and exact private
  root warming in `a_reserved_candidate_published_after_another_branch_checkpoint_keeps_its_pages`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_reserved_checkpoint.rs`.
  Public counterpart keeps exact reservation/publication order, both actual
  selected captures and ACKs, original member membership, reopen/record survival,
  exact counted lineage-node equality, public Full record demand and emitted
  nonempty lineage directory. No root, mutable constructor or counter is exposed.

Merging lane P-A into the integration branch (2026-11-10) moved the private halves of
three integration courts whose in-source fixture P-A retired. Their public halves keep
the same test names in the cold-residency court
(`cold_residency/index_selection_reads.rs`):

- C2: the `mutable` private index snapshot/install loop and the check that capture does
  not admit the carried image, in
  `cold_checkpoint_preserves_carried_index_selection_and_reads_postings_without_a_full_warmup`:
  `restore from 8283728aff:crates/worth-relational/src/tests/durability/medium/cold_index_selections.rs`
  (fixture: `restore from 8283728aff:crates/worth-relational/src/tests/durability/medium/cold_index_fixture.rs`).
- C2: the private Complete codec checks (artifact format version, empty artifacts, roots
  beside bases, exact bindings and `readmit`) in
  `complete_checkpoint_saves_the_selection_carried_to_its_current_root` and
  `reclose_before_any_image_demand_keeps_the_selections_the_image_carried`:
  `restore from 8283728aff:crates/worth-relational/src/tests/durability/medium/complete_index_selections.rs`.
- The two `resident_full` C2 charge courts above, restored from `f9f15bf8c8`, target
  `resident_for_request`, which the integration branch's production region constructor
  `PreparedResidentRegion::initial(lane, request, partition, symbols)` calls; restore them
  against that signature. The Full region courts were not moved:
  `controlled_full_region_matches_current_kind_preimages_and_truth` and
  `controlled_full_commitment_clone_retains_only_its_actual_node_backing` stay in
  `storage/residency/region_publication/full/tests.rs` with the v3 kind-commitment oracle
  (`full/tests/kind_oracle.rs`) and its src-court baseline row.

The integration branch's cleanup ledgers (`merge-cleanup-reds.md`, `merge-families.md`) become entries here when their courts leave the tree before landing 2.

### Batch 1 Relational cleanup moves

The following original claims move under Delivery tracks; mixed courts retain their independent happy semantics. No claim is retired.

- **C4** `content_cold_envelope_preserves_a_literal_version_refusal_without_poisoning_history`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/cold_envelope.rs`. Injected literal content-version corruption, independent outer reseal, repeated refusal and cold-cell retry behavior.
- **C2** `content_durable_artifact_cancellation_after_original_allocation_never_calls_medium`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/durable_publication.rs`. Cancellation is triggered by observing an 8 MiB allocation increase; all assertions protect the C2 original-backed cancellation/refund boundary, not an independent already-cancelled ordinary journey.
- **C2** `content_durable_artifact_memory_refuses_before_the_medium_and_selected_commit`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/durable_publication.rs`. Finite artifact-memory refusal and no-effect/refund posture.
- **C2** `content_durable_artifact_original_work_exhaustion_is_exact_and_pre_effect`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/durable_publication.rs`. Exact original work ceiling, pre-effect exhaustion and final available-memory equality.
- **C2** `original_kind_custody_survives_successor_basis_checkpoint_tail_and_reopen`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/historical_kind/custody.rs`. Original-request retained funding, equality across reopen and exact final refund; carry the related original-budget/retained-basis lifetime setup into the restored C2 court.
- **C2** `a_reused_relation_slot_keeps_retention_readmission_valid`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/historical_kind/retention_roundtrip.rs`. Both tests in this file require durable retirement, slot reuse and retention across cold reopen; the latest durable-history/branch-retirement ruling places this protocol in C2.
- **C2** `mixed_retirement_replacement_reuse_and_cold_roundtrips_keep_region_integrity`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/historical_kind/retention_roundtrip.rs`. Seeded mixed retirement/replacement/reuse campaign and cold recapture. Shares the file with row 5; move the file once, listing both original tests.
- **C4** `content_controlled_journal_seed_admits_the_canonical_omitted_history_tail`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/journal_recovery.rs`. Controlled removal of authored-index history plus reseal/admission counterpart; the entire generated omission world is the C4 claim.
- **C4** `content_decoded_journal_omission_refuses_before_selecting_a_root`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/journal_recovery.rs`. Injected missing journal content with resealed frame and no selected-root effect.
- **C2** `content_original_field_capture_charges_real_aspect_count_and_key_bytes`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/preimage_work.rs`. Variable aspect/key-width growth matrix, binary-searched work threshold and scope refund measurements.
- **C4** `content_corruption_refuses_even_after_an_independent_outer_digest_repair`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/recovery.rs`. Injected inner content corruption with independently repaired outer digest.
- **C2** `cold_selected_index_refresh_preserves_budget_gate_and_reads_head_without_history`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/cold_index_refresh.rs`. The one test specifically arms missing historical material and probes head-only refresh and reconstruction budgets. Its successful projection assertions are the positive counterpart in this C2 world, not an unperturbed graceful Home proof.
- **C4** `required_catalog_delta_survives_all_projections_and_rejects_missing_or_older_format`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog.rs`. Move the hostile assert_malformed_deltas invocation and its wire.rs helper/mutation matrix alongside row 20.
- **C4** `valid_delta_shape_still_refuses_source_identity_gap_and_conflicting_overlap`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog/wire.rs`. All wire.rs behavior is hostile catalog-delta mutation/resealing, including assert_malformed_deltas called by mixed row 19. Move the helper and this test together.
- **C4** `semantic_result_is_required_even_after_a_corrupt_frame_is_resealed`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/semantic_result.rs`. Deleted semantic image after resealing. The unchanged-frame counterpart belongs with this C4 negative; the separate semantic_result_preserves_original_create_bindings_and_validation_meaning court stays HAPPY.
- **C2** `shared_history_custody_refuses_unfunded_admission_then_survives_until_the_last_basis_drops`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/cold_materialization/shared_custody.rs`. One funding-exhaustion/shared-hold-lifetime court. Successful reads are the positive counterpart to funded custody admission; there is no independent source-value assertion in this file.
- **C2** `checkpoint_image_roots_build_cold_and_incremental_indexes_without_canonical_body_reads`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/indexes.rs`. Cold-slot allowances, exact zero/slot counters and no-canonical-body-read cost claim; move associated constrained refresh scenarios.
- **C4** `checkpoint_image_records_original_required_output_membership_with_logical_basis`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/required_outputs.rs`. Mutate the root publication variant byte and expect InvalidProjectedMeaning.
- **C2** `cold_complete_recapture_preserves_original_index_generations_and_live_tail`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/cold_indexes.rs`. Exact original derived-cache generation byte preservation in subsequent Complete captures, including the stale generations() tuple parser.
- **C2** `entity_boundary_refuses_wrong_model_kind_and_original_request_limits`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/relational_certification/history_ownership/entity_boundary.rs`. Metered boundary backing, work-ceiling traversal and zero-memory refusal. Shared positive setup may be used by both separated courts.
- **C4** `graph_selection_preserves_corrupt_cold_relation_material_as_a_typed_refusal`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/relational_certification/history_ownership/graph_boundary.rs`. Warm endpoints only to force the later damaged relation read; the independent cold ownership/graph positive courts remain. All target assertions concern injected integrity failure.
- **C2** `graph_relation_preparation_retains_its_funding_and_cannot_reset_the_entity_meter`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/relational_certification/history_ownership/graph_resources.rs`. Entity-versus-relation retained funding, final refunds, and cannot-reset-the-entity-meter work bound.
- **C4** `selected_publication_keeps_original_ids_under_fresh_commit_and_cold_current_custody`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/relational_certification/history_ownership/historical_publication.rs`. Only require_unknown_origin_refusal and its historical_artifact.rs tamper helper subclaim move.
- **C4** `missing_cold_ownership_edge_material_refuses_with_its_real_partition`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/relational_certification/history_ownership/mod.rs`. Warm endpoint preparation then deliberately lose unread region members; all target assertions concern the typed missing-material refusal and its partition.
- **C2** `ownership_observation_preserves_work_and_memory_refusals`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/relational_certification/history_ownership/mod.rs`. Warm ordinary owner read is the resource-denial counterpart; the independent cold_model_ownership_demands_cross_partition_endpoints_and_edges HAPPY court already retains this same positive owner result. Move only this budget-refusal function; keep that sibling.
- **C4** `missing_cold_publication_material_refuses_at_its_actual_custody_partition`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/relational_certification/history_ownership/publication_membership.rs`. Warm output read selects which remaining material to delete; all target assertions concern the injected missing-custody refusal and real partition.
- **C4** `rehashed_selection_artifacts_refuse_wrong_source_identity_predecessor_and_mixed_profiles`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/relational_certification/owner_service_completion/retained_entity_selection/provenance.rs`. One rehashed provenance/profile fault matrix with private byte mutation helpers. Unchanged readmission/head assertions establish the matrix counterpart and no-effect posture.
- **C2** `unfunded_validated_proposal_backing_refuses_and_releases_original_custody`: `restore from 9ed22bccaa:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/validated_proposal_memory.rs`. Owner-layout-specific finite-budget refusal and exact refund/no-effect.
- **C2** `one_work_account_bounds_each_entry_with_its_original_canonical_evidence`: `restore from 9ed22bccaa:crates/worth-relational/src/authority/commit/pipeline/request_checkpoints/work_account.rs`. Exact common work account and one-unit-short denial across entries.
- **C2** `one_work_account_bounds_the_same_proposal_on_every_entry`: `restore from 9ed22bccaa:crates/worth-relational/src/authority/commit/pipeline/request_checkpoints/work_account.rs`. Equal SDK work settlement and shared one-unit-short ceilings across entries.
- **C2** `selected_ack_exposes_backed_data_before_retirement_and_releases_holds_only_on_completion`: `restore from 9ed22bccaa:crates/worth-relational/src/durability/data/native_checkpoint/tests/selected_ack.rs`. Interrupted/work-limited partial ACK, publication while ACK is pending, old read/capture hold retention and exact terminal retirement. Keep shared setup/observations in the restored C2 case.
- **C4** `forged_cross_region_kind_commitment_is_refused_before_member_reads`: `restore from 9ed22bccaa:crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs`. Four forged kind/directory/count dimensions; all target assertions concern refusal before region reads.
- **C2** `canonical_artifact_accounting_excludes_nested_diagnostic_perturbation`: `restore from 9ed22bccaa:crates/worth-relational/src/history/commit/artifact_tests.rs`. Pure allocation accounting and diagnostic-size invariance. canonical_payload_bytes() returns u64, so even lines 243-247 compare size, not payload contents. No independent H semantic assertion identified.
- **C2** `memory_frame_quota_and_cancel_refuse_before_position_or_head`: `restore from 9ed22bccaa:crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs`. Independent finite frame-size quote, one-byte-short capacity refusal, exact requested bill and charged-memory refund; move the frame quota campaign.
- **C2** `clearing_reconstructed_last_aspect_retains_no_state_bill`: `restore from 9ed22bccaa:crates/worth-relational/src/history/data/codec_retirement_closure/payload_reconstruction/tests/optional_output.rs`. Storage-footprint quote and exact zero retained-state bill/refund assertions.
- **C2** `funded_extra_refuses_copy_and_replacement_before_growth`: `restore from 9ed22bccaa:crates/worth-relational/src/storage/substrate/record_arena/values/tests.rs`. Finite copy/replacement admission, exact retained bill, and funding lifetime after clone. The state-presence check is the positive counterpart of that custody claim.
- **C2** `repeated_dag_reachability_reuses_scratch_under_the_same_finite_policy`: `restore from 9ed22bccaa:crates/worth-relational/src/validation/engine/tests/acyclicity_resources.rs`. 128-node repeated-reachability workload under fixed 1MiB/4M policy, live-scratch budget reuse and final exact refund. Underlying current memory-denial cause remains unknown.
- **C4** `checkpoint_restore_rejects_incomplete_field_revision_column`: `restore from 9ed22bccaa:crates/worth-relational/src/visibility/materialization/read_records/projection/field_revisions/restore_tests.rs`. Intentionally shorten field_revisions SoA column and reseal root; require typed corruption refusal and diagnostic. Recovery already refuses CorruptCheckpoint at generic arena column-length check; detail substring expectation is stale.
- **C2 metadata accompanying already moved private courts**: `runtime.merge.topology_region_detection` and `runtime.merge.execution_verification`, `restore from 73ce80ebfc:crates/worth-relational/src/performance/data/complexity_contract_inventory.rs`. Their only registered proof bodies moved with P-A; do not publish them as Verified until C2 restores the actual counter/verification evidence. Other contracts keep their actual registered proofs. The proof registry checker remains unchanged.
- **C2 source-unit claim closures**: `controlled_field_lowering_refuses_before_copy_and_preserves_typed_causes` (its finite lowerer/readmitter phase-admission block) and `controlled_field_lowering_releases_scratch_before_retaining_small_patch` (all private exact/backing/peak assertions), `restore from 73ce80ebfc:crates/worth-relational/src/authority/mutation/intents/field_authoring_candidate/tests.rs`; restore the same-SHA `tests/storage_footprint.rs` and `tests/fixture.rs`. Original winner, typed cancellation/zero-memory refusal, scalar/struct content and portable wire-copy assertions remain in their semantic unit courts.
- **C2** `cleared_entity_and_relation_extras_keep_only_their_fixed_original_bills`: `restore from 73ce80ebfc:crates/worth-relational/src/storage/substrate/record_arena/values/tests/optional_output_funding.rs`, with its same-SHA parent fixtures. This is the exact private EntityExtra/RelationExtra bill and last-drop campaign. The independent optional reconstruction semantics remain in their own unit court with a lawful optional contract.

### Root final-partition cleanup moves

These claim-level moves preserve the independent H assertions named below. The original red run remains red; fixture repairs alone do not certify the masked claims.

- **C2** `checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/checkpoint_stage.rs`. Exact checkpoint bytes, per-byte charge, scope count, consult ordinals or memory-boundary campaign. The independent typed work/memory/cancellation refusals, successful counterpart, and unchanged publication remain in checkpoint_refusals_preserve_request_custody_and_publication. Restore same-SHA checkpoint_stage/fixture.rs for original byte-walk support.
- **C2** `checkpoint_capture_sets_up_exactly_one_stage_whatever_the_row_count`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/checkpoint_stage.rs`. Exact checkpoint bytes, per-byte charge, scope count, consult ordinals or memory-boundary campaign. The independent typed work/memory/cancellation refusals, successful counterpart, and unchanged publication remain in checkpoint_refusals_preserve_request_custody_and_publication. Restore same-SHA checkpoint_stage/fixture.rs for original byte-walk support.
- **C2** `empty_checkpoint_work_ceiling_bounds_the_whole_capture`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/checkpoint_stage.rs`. Exact checkpoint bytes, per-byte charge, scope count, consult ordinals or memory-boundary campaign. The independent typed work/memory/cancellation refusals, successful counterpart, and unchanged publication remain in checkpoint_refusals_preserve_request_custody_and_publication. Restore same-SHA checkpoint_stage/fixture.rs for original byte-walk support.
- **C2** `empty_checkpoint_memory_boundary_is_its_bytes_plus_one_stage`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/checkpoint_stage.rs`. Exact checkpoint bytes, per-byte charge, scope count, consult ordinals or memory-boundary campaign. The independent typed work/memory/cancellation refusals, successful counterpart, and unchanged publication remain in checkpoint_refusals_preserve_request_custody_and_publication. Restore same-SHA checkpoint_stage/fixture.rs for original byte-walk support.
- **C2** `fixed_kind_read_is_flat_in_unrelated_regions_and_history`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/kind_directory/cold.rs`. Cold I/O and packet-query work growth; retain exact kind IDs and query results at the same region/history workloads, using same-SHA kind_directory/query.rs for restore.
- **C2** `lazy_pin_and_fixed_read_do_not_grow_with_unrelated_regions_or_history`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/lazy_pin.rs`. Member-growth, pin-I/O/symbol residency, and exact reuse-count assertions; retain selected entity/symbol/text, relation ID, and repeated read semantics at the original workloads.
- **C2** `wide_frontier_does_not_escalate_to_unreached_regions`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/lazy_pin.rs`. Member-growth, pin-I/O/symbol residency, and exact reuse-count assertions; retain selected entity/symbol/text, relation ID, and repeated read semantics at the original workloads.
- **C4** `lazy_pin_and_fixed_read_do_not_grow_with_unrelated_regions_or_history`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/lazy_pin.rs`. Injected damaged and unavailable source subcampaign, original helper lines197-269. Restore same-SHA cold_checkpoint.rs.
- **C2** `equal_payload_jumps_charge_the_same_receiving_owner_work_at_near_and_distant_history_nodes`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/relational_certification/retained_entity_selection.rs`. Charged-work, peak-memory, copy/route counter equality across retained-history distances; H keeps both published selected values and retained current value.
- **C2** `count_only_read_does_not_grow_with_model`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/snapshot_counts.rs`. Measured count-only I/O and model/history growth bounds; retain independent authored cardinalities and whole-row agreement at the same workloads.
- **C2** `count_only_read_does_not_grow_with_history`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/snapshot_counts.rs`. Measured count-only I/O and model/history growth bounds; retain independent authored cardinalities and whole-row agreement at the same workloads.
- **C2** `renewed_cell_symbol_admission_is_flat_in_model`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/symbol_admission.rs`. Symbol-admission phase/accounting and model/history work bounds; retain full/non-full cardinality and measured-next-value readback at the original workloads.
- **C2** `renewed_cell_symbol_admission_is_flat_in_history`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/symbol_admission.rs`. Symbol-admission phase/accounting and model/history work bounds; retain full/non-full cardinality and measured-next-value readback at the original workloads.
- **C2** `a_fixed_edit_does_not_pay_the_previous_large_edit_again`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/symbol_admission.rs`. Previous-bulk-size versus fixed-edit symbol-admission work growth campaign, including both native and declared input text. No independent final-value oracle was present.
- **C2** `cold_inline_bytes_head_refuses_after_member_read_then_retains_exact_payload_through_edit`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/source_initial_inputs/checkpoint_admission/cold_materialization/byte_payloads.rs`. Injected memory-room pressure, small/large denial, retained-backing and final-owner funding assertions. H keeps real reads, exact scalar/Struct bytes and historical adjacency at 8 and64KiB payloads.
- **C2** `cold_inline_bytes_history_refuses_after_read_then_retries_full_with_exact_old_values`: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/relational/source_initial_inputs/checkpoint_admission/cold_materialization/byte_payloads.rs`. Injected memory-room pressure, small/large denial, retained-backing and final-owner funding assertions. H keeps real reads, exact scalar/Struct bytes and historical adjacency at 8 and64KiB payloads.
- **C2** `forked_topology_write_copies_only_touched_regions`: `restore from 73ce80ebfc:crates/worth-court-relational-branch-reference-compile-time/tests/worth_court_relational_branch_reference_compile_time/oracle/relational/relational_certification/root_branch_copy_on_write/mod.rs`. All exact locators/root sharing, physical dedup/copy/accounting and unrelated-branch cost assertions. H keeps independent baseline and rewire oracle plus ancestor/sibling branch observations.
- **C2** `prove_named_delta_cow`: `restore from 73ce80ebfc:crates/worth-court-relational-branch-reference-compile-time/tests/worth_court_relational_branch_reference_compile_time/support/relational_certification/ordinary_context/root_named_delta_cow/declarations.rs`. Exact region/copy/allocation and unrelated cost assertions used by the eight named-delta campaign; H retains original eight independent semantic oracle comparisons. Restore same-SHA oracle/.../root_named_delta_cow/mod.rs and helpers.
- **C2** `production_write_footprints_report_exact_records_and_local_publication_cost`: `restore from 73ce80ebfc:crates/worth-court-relational-branch-reference-compile-time/tests/worth_court_relational_branch_reference_compile_time/cost/relational/relational_certification/root_cost_scale_axes/publication/mod.rs`. Exact physical publication counters/allocation and scale/path sharing. H keeps actual production patch-record counts at1,64,4096 authored records and baseline oracle; restore same-SHA direct helper closure.

- **C2 fixture closure** for the moved selected-member funding/retirement and byte-pressure claims: `restore from 73ce80ebfc:crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/selected_source.rs` and its same-SHA `selected_source/custody.rs`. Remove their now-unreachable pressure variant and custody-control owner; retained read and retirement ports forward to the real medium.

### Landing 2a composition of previously moved Root claims

- **C2:** the original-account invoice remainder of
  `equivalent_proposal_entries_preserve_mutation_results` is preserved at
  `restore from a98ed66840:crates/worth-relational/src/authority/commit/pipeline/request_checkpoints/work_account.rs`.
  The same mutation and successor-version comparisons remain in the happy tree;
  measured positive/bounded work belongs with the existing C2 account courts.
- **C4:** H5's updated `forged_cross_region_kind_commitment_is_refused_before_region_payload_reads`
  remains deferred with its original cross-region corruption claim:
  `restore from a98ed66840:crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs`.
  It complements the existing `9ed22bccaa` restore entry; merging H5 does not
  resurrect a deliberately deferred corruption court.

### Landing 2a private fixture funding splits

The following complete original funding campaigns remain C2 obligations. Their
independent value semantics stay active in the same owning source modules;
these splits do not claim equivalent funding coverage from a related public court.

- **C2:** `extracted_content_snapshot_retains_original_request_bill_until_last_clone`;
  restore `669ac9fcd4:crates/worth-relational/src/content/schema/retention_tests.rs`.
  Preserve extracted-versus-projection charge, serialization with original custody,
  clone ownership and final refund. The value serialization court stays on H.
- **C2:** `exact_readmission_retains_original_compiler_funding_until_last_child`;
  restore `669ac9fcd4:crates/worth-relational/src/branch/root_schema/readmission/tests.rs`.
  Preserve zero-budget refusal/refund, exact compiler quote, escaped-plan child and
  parent/registry/final-child drop sequence. Declared kind/plan counts and authority
  equality remain on H through configuration-owned exact readmission.
- **C2:** `controlled_patricia_refuses_exact_leaf_and_late_parent_and_refunds_children`
  and `escaped_patricia_leaf_keeps_only_its_original_allocation_grant`, including
  `controller_memory`; restore
  `669ac9fcd4:crates/worth-relational/src/storage/substrate/digest_index/funding_tests.rs`.
  Preserve both failure positions and typed fields, controlled/uncontrolled digest
  equality, allocation ownership and exact refunds. Five-node topology, escaped
  leaf shape and original key remain active private value evidence.

### Landing 2a historical handler-key custody split

- **C2:** restore the complete original
  `historical_key_aliases_retain_and_refund_the_original_authoring_payment`,
  `refused_normalization_never_yields_historical_keys_or_a_retained_bill`, and
  their two-allowance `reads` fixture from
  `669ac9fcd4:workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/invariant_projection/operation_reader/decision_reads/expected_keys/historical_retention_tests.rs`.
  Preserve normalization charging, historical conversion/alias custody, exact
  zero-memory error category and every refund observation. Active H evidence
  separately exercises the exact handler-only 1/2 allowance contrast through
  the already permitted public normalizer court
  `conflicting_handler_adjacencies_keep_the_strongest_allowance`; the private
  value court `historical_conversion_and_aliases_preserve_exact_handler_keys`
  exercises actual conversion and aliases with the complete expected key.
  Neither H court claims funding coverage or a public historical-conversion door.

### Landing 2a selected-payload funding split

- **C2:** restore the complete original
  `669ac9fcd4:crates/worth-relational/src/storage/substrate/record_arena/values/tests/historical_selection_funding.rs`
  and its `mod historical_selection_funding` mount under values/tests.rs.
  Preserve both entity/relation kinds, absent/existing targets, zero-budget
  request denial, unchanged slot counts, zero charge/refund, distinct allocation
  identity, exact payload equality, reachable bounds, latest-history carrier,
  retained-bill contrast and every source/target/installed drop observation.
  Its source fixture closure is the original parent values/tests.rs `patch`,
  `extra`, `expected` and `request` functions at the same SHA. Active private H
  courts exercise the same production selection kernel, retain both payload
  kinds and target postures, and verify latest-history carrier and retained
  values after source and target drop. They do not claim funding coverage.

### Landing 2a exact Home-step invoice campaign

- **C2:** restore the complete
  `home_step_body_and_candidate_share_the_original_work_ceiling` campaign and
  `trial` fixture from
  `0890a7800a:workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_indexes/work_account.rs`.
  Restore its same-SHA `mod work_account` mount in `transition_indexes.rs`, the
  COMPLETE `transition_indexes/` fixture tree (including `target.rs`),
  `no_pending_owners/transition_opening.rs` and its child fixture closure, and all
  referenced parent imports, actual Home transition-plan/step-grant/World-open/
  Native preparation owners and work-account controls. Preserve the actual SDK
  invoice, exact and one-unit-short requests in both wrapped/unwrapped forms,
  typed WorkExhausted, discarded candidate and unchanged Native basis assertions.
  Restore through the current genuine Home and index-maintenance budget contract;
  never revive an obsolete signature or fabricate original-step/source admission.
  Ordinary transition/index and step-affinity courts remain active under H.

### Landing 2a Memory invoice and deferred complexity proofs

- **C2:** restore the complete
  `memory_original_frame_survives_export_checkpoint_and_last_alias` campaign
  from `0890a7800a:crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs`.
  Restore its same-SHA `envelope_residency.rs` mount, full
  `history/commit/envelope_residency/` tree, canonical-artifact/immutable-byte
  owners and `crates/worth-relational/src/tests/support.rs` with its complete
  support tree. Preserve every original payload charge, no-extra-charge export,
  exact retained bill, alias lifetime and zero-after-last-drop assertion.
  The active H counterpart keeps original-frame identity, exact bytes before
  and after checkpoint, and usable exported bytes after runtime and alias drops.
- **C2:** the four complexity declarations `runtime.schema_transition.classification`,
  `runtime.subscriber_resume.continuity`, `runtime.merge.planning`, and
  `runtime.merge.execution_commit` are explicitly Debt while their complete
  proof campaigns above are moved out. Restore their Verified declarations and
  original proof lists from
  `0890a7800a:crates/worth-relational/src/performance/data/complexity_contract_inventory.rs`
  together with the already recorded complete schema-continuity and merge
  campaigns at `86e84dc5cd738882bf50fced389fc7deaa31567f`, including their exact
  original same-SHA fixture, parent, implementation and counter/control closure.
  Restore the universal nonempty-proof assertion from
  `0890a7800a:crates/worth-relational/src/tests/complexity/contracts/commit_budgets/contract_registry.rs`
  when those proofs are registered again. No moved proof is represented as
  currently executed or verified; every remaining named proof must still name
  a registered test and every Verified declaration must have proof.

### Landing 2a foreign-node injection campaign

- **C4:** restore the complete `ordered_chain_foreign_node_owner_cannot_enter_a_birth_edit`
  wrapper from `0890a7800a6b1a3d4becef3299915c0bf5aec4bd:workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/court/prefix_growth/ordered_chain.rs`
  and its full `events/chain/foreign_node.rs` body and `events/chain.rs` module
  mount/imports/append/principal closure from the same immutable managed_index
  tree. Restore the complete enclosing maintained/events/court/prefix-growth
  mounts, `events/court.rs` and `court/fixture.rs`, and the genuine events schema,
  query, shape, paths, prefix, rule, composition, authoring, history, mutation,
  plain, `maintained/nested/edit.rs`, registered host construction and directly
  used authentication/resource fixtures at that SHA.
  The original receiver-owned `observed_root` and `foreign_node` fields, genuine
  provider build-root observation and update-time foreign lookup are at
  `3656bb1cc3dc7449efed2f4788b8c5d0f362a502:workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs`
  with its complete same-SHA `prefix/` child closure. Use the current shared
  `prefix_observation.rs`, `prefix_provider.rs` and children and execution
  managed-index Node issuance/edit/validation, controls/stops and provider-failure
  owners from `0890a7800a6b1a3d4becef3299915c0bf5aec4bd` as the surviving owner
  reference. Adapt these restoration sources to current genuine contracts;
  never overwrite landed owners with historical replicated providers or retired
  construction routes. Preserve the actual unrelated U node, arm/disarm, exact
  Controlled(ForeignNode) failure chain, unchanged head and A contents, and the
  lawful cleared-arm append producing E with x/e. The original retry uses a new
  idempotency key; it does not prove same-key retry or rollback after effects.
  Ordinary foreign-Source-slot and current Node validation evidence stays active.
  The optional-aspect provider receiver, watch and complete Source campaigns stay
  active under H through their actual borrowed SourceRows; they are not C2/C4 moves.

### Parallel work

H items whose dependencies have merged may be built in parallel where their write scopes do not overlap. Each one merges into the integration branch when its reviewer approves and its Tier-2 run there is green. A C item may start before landing 2 only where the coordinator assigns it and it does not slow an H item.

### Verification tiers by track

| Track | What runs |
|---|---|
| H | Tiers 1–2 as below, plus the graceful oracle when the diff reaches its subject |
| C | Adds the crash oracle at Tier 2 |
| Tier 3 (the full matrix and the exhaustive cumulative oracle) | At landing 3 |

## Adversarial constraint

The cleanup track certifies this constraint. The happy track meets the graceful constraint in Delivery tracks.

A home is killed at any durable I/O boundary of any supported operation. That
includes every owner's publication, product movement, program transitions,
checkpoints, close and recovery itself. The home is then reopened any number
of times, in a fresh process, under the same or a changed application build
identity.

Each reopen must:

- recover exactly the acknowledged prefix of every owner, plus at most the
  in-flight operation, decided from durable state;
- reach owner state equal to a memory-only reference host executing the same
  operation sequence at that prefix. This covers graph records, World
  occurrences and custody, product heads, idempotency outcomes, Signal and
  Bridge definitions, programs, workflow state, external custody and
  correction lineage;
- answer every retry of an admitted key with its original occurrence, its
  recovered custody posture, or its typed no-effect posture, and never with a
  second effect;
- never reissue a committed identity of any owner;
- refuse before service, with a typed reason, when it cannot do the above.

A crash between owner effects preserves which owners acted and whether the
product moved. Unresolved effects keep their recovery posture until their
owner establishes the fate. Loss of process memory, cancellation and caller
abandonment are never an implicit completion, rollback or no-effect.

## Decisions

**D1. One application home, Store-owned, host-opened.**

A new Store-workspace crate, `worth-store-application`, owns the application
home. It handles open, initialization, recovery sequencing, checkpoint,
resolve and close for every owner (Milestone 1 D1). It replaces
`RelationalStoreHome` as the only home type. `worth-store-relational` keeps the
Relational medium adapter and cold source, which become the Relational
section of the home.

The Query host is the composition root. `worth-query-host` opens the home
through one call. Applications never construct, open or synchronize a Store
runtime, and they import only `worth-query-decl` and `worth-query-host`.

This requires an explicit boundary ruling. It is recorded as
`dependency_denials` and allowlist entries in
`tools/boundary-check/config/road1.toml`; band rules alone do not cover crates
outside `worth-contracts`. The ruling has three parts:

- **`worth-query-host`** may depend on `worth-store-application`, and only its
  production composition module may name it. No other `worth-query*` crate may
  depend on any `worth-store*` crate.
- **`worth-store-application`** may depend on the owner crates whose ports it
  implements: Relational, World, Signal, Bridge, and the Query crates that
  define Query-owned artifacts. It must never depend on `worth-query-host`.
- **`worth-store-testkit-application-oracle`** (D14) is the fixture/reference
  library and may depend on `worth-query-host` and `worth-store-application`.
  It has no tests or integration target. `worth-store-court-application` is
  the canonical standard-harness leaf containing assertions and child-process
  reexecution. No production crate may depend on either package.

**Build cost.** From now on, the Store workspace builds the Query, World and
Signal stacks for its application crates, and the host builds the Store
stack. Phase 3 records the measured build-time impact on every lane in its
evidence. Store lane partitions keep the physical-only suites off the
application crates.

**D2. Owners define artifacts and ports; Store implements them.**

Each owner defines, in its own crate:

- its durable artifacts: frame, version, digest and readmission;
- a durable medium port with the Milestone 2 answers `Committed`,
  `ProvenNoEffect` and `Indeterminate`;
- a recovery source it readmits from.

Store never decodes an owner payload. `worth-store-application` implements
every port over named journals and anchored records. Owner ports are plain
traits with no filesystem or Store dependency, so Signal still compiles for
`wasm32`. An owner's readmission is its only door from bytes to authority.

**D3. Store capacity: root schema 12 and a journal registry.**

- **Anchors.** Root manifest schema 12 widens the inline anchor table to 32
  anchors. The payload is 3,760 bytes, within the smallest page.
- **Unchanged.** The WAL advance codec, recovery projection and successor-root
  carry-forward rules do not change.
- **Journals.** The home crate declares the ordered-journal registry as a
  closed, versioned set of names. Each journal carries its own unresolved
  state.
- **Clean cutover.** Schema 11 roots refuse at open with
  `UnsupportedRootSchema`, and nothing reads them. No production home exists to
  preserve.

**D4. Owners publish in order; product movement is its own durable step.**

No root decision moves more than one anchor. A composite operation publishes
each owner's effect as that owner's own durable step, in World's fixed owner
order, durable before live (Milestone 2 D2). Product movement is World's
publication of a `RuntimeWorldOccurrenceArtifact` to `world.occurrences`. The
live product compare-and-swap happens only after that publication answers
`Committed`.

- `ProvenNoEffect` on product movement leaves the owner effects as
  `ProductUnpublished` custody, with their existing next actions.
- `Indeterminate` returns `ProductUnpublished(ProductPublicationUnresolved)`.
  Store's unresolved state is per journal and `world.occurrences` is one
  journal, so this blocks product movement on every branch of the home until
  reopen. Owner effects that do not move a product remain admissible.
  Recovery decides the fate.

No competing transaction coordinator, idempotency table or recovery authority
is added. World stays the composite coordinator, and its existing catalogs gain
durable backing.

**D5. Owner effects carry their attempt association; custody dispositions are
durable.**

Before Native candidate preparation seals an envelope, World records an
`OpenAttempt` in its existing `world.custody` journal. It retains the original,
independently issued attempt, composite-commit and custody identities, exact
expected product head, branch/incarnation, provenance, program revision and
component intent. Only actual commitment of that open permits owner effects;
no-effect or indeterminate opening starts none. The existing preparing-operation
guard and reserved resources move through this progression once. Native consumes
the original validated proposal once, binding its opaque association before the
existing single preparation algorithm seals the candidate.

After the open commits, preparation refusal, cancellation or unwind retains or
terminalizes that same durable open; it cannot erase the record, claim total
World no-effect, reissue its identities or replay a handler. Completion verifies
the sealed candidate association against the original open and exact branch and
incarnation before effects. Recovery resumes issuer frontiers beyond every
durably recorded identity, including unfinished opens.

Every owner effect published inside a composite attempt records the attempt's
World-defined `CompositeAttemptAssociation`: the home-scoped attempt identity,
product branch identity and incarnation. It sits in the owner's own artifact:

- a Relational commit carries it in a Relational-owned, opaque, size-bounded
  caller-association field. The canonical envelope also records the original
  durable semantic result defined in Phase 4. Both are sealed during original
  preparation, under canonical artifact v3 and native checkpoint CHK16;
- a Signal advance carries it in its advance artifact (Phase 5);
- later owners follow the same pattern.

Relational's existing commit index carries required descriptive association
discovery metadata derived from that same envelope: `Unassociated` or
`Associated(original opaque association)`. An absent field is malformed
current-format data, never `Unassociated`. This selects subjects for actual
readmission; it grants no publication, settlement, root or recovery authority.

Recovery joins owner effects to occurrences by association. An owner effect
whose association has neither an occurrence nor a disposition is recovered as
durable `ProductUnpublished` custody with cause
`RecoveredWithoutProductMovement`.

Its identity bundle and predecessor come from the original open. A purported
World association with missing or inconsistent original-open evidence refuses
recovery; it never manufactures the missing identities. An open alone does not
prove that an owner effect happened.

Continuing or releasing custody publishes a `RuntimeWorldCustodyDisposition`
to `world.custody` before its live effect. Released custody therefore never
reappears on reopen. Open custody is carried in World's checkpoint section
(D12). Nothing is folded silently into a new root or re-executed.

For Phases 3–4, abandoning a live directory preparation retains its actual
Query/World custody and answers `ReopenRequired`; it does not grant live
no-effect release. Completed Native recovery and complete associated-effect
reconciliation establish the recovered fate. The exact authority, next-action
and production-court contract is in
[Shared durable owner contracts](milestone-3-durable-owner-contracts.md#live-abandonment-and-recovered-disposition).

**D6. Durable, home-scoped identities.**

Every identity that a durable artifact or durable row records becomes
home-scoped and runtime-neutral. For World these are owner, branch,
incarnation, reference generation, composite commit, publication attempt,
bootstrap attempt and `ProductUnpublishedOwnerEffects`. The equivalent Signal,
Bridge and Query identities follow in their phases.

- **Recording.** The home descriptor records a home identity at
  initialization.
- **Frontiers.** Issuers resume one past the largest durable value (Milestone 2
  D7) and issue deterministically in order.
- **Process-local identities.** `runtime_instance_id`, session ids and
  graph-instance ids remain only for values no durable artifact or row
  records. The Phase 1 audit lists every durable row that records one today.
  The phase that makes the row's family durable replaces that field with
  home-scoped identity.

**D7. Single writer and incarnation fencing.**

- **Writable opens.** Every open is writable; this milestone provides no
  read-only open.
- **Lease.** Store's OS mutation lease (`MutationOwnershipLease`) is held
  continuously from the start of recovery through serving. The handoff
  transfers the held lease; it never drops and reacquires it.
- **Incarnation.** Each open durably advances the home incarnation in the home
  descriptor before service. Values that carry authority are bound to that
  incarnation: capabilities, handles, sessions, leases and recovery claims. A
  value from an earlier incarnation refuses with `StaleIncarnation` before any
  effect.
- **Durable data is not fenced.** Receipts, occurrence identities, commit
  identities and history cursors are resolved by home identity and affinity,
  and remain valid across reopen.
- **New homes.** Initialization happens under the lease. A location is new
  only when the lease holder observes no Store at all. A Store left partly
  initialized is completed or refused (Milestone 2 D5), never treated as an
  empty home.

**D8. Recovery order is Store-sequenced.**

Open runs these steps in order:

1. Store physical recovery.
2. The home descriptor, then the incarnation advance.
3. Relational readmission.
4. Signal and Bridge readmission.
5. World readmission and reconciliation (D5).
6. Query readmission: idempotency, programs, workflows and custody.
7. Dispatch and correction custody.
8. Completion of a pending program transition (D16), or the transition the
   open request carries.
9. Service.

Each owner readmits only its own artifacts, after the owners it depends on.
Any refusal is typed and happens before service. Steps 1–7 are recovery: they
never republish, re-execute a handler, or issue a replacement commit or
occurrence. Step 8 is not recovery. It is the transition's own forward
progress, run against fully readmitted owners before service, under D16's
plan-digest check.

**D9. Idempotency and receipts resolve from durable state.**

Query's application idempotency binding is recorded in the same Relational
commit as the effect, as it is today. It now also records product branch
affinity (identity and incarnation) and the attempt association. A retry is
answered by reading the durable binding, the World occurrence or custody, and
the owner artifacts. The original receipt is rebuilt from those, field by
field, as Phase 4's receipt table defines.

- **Evidence store.** `WorthQueryCompletedCommitEvidenceStore` stops being an
  authority. It becomes a derived in-process index:
  - it is filled at commit and rebuilt on demand from durable state after
    reopen;
  - it is never the only source of an answer;
  - its session index stays process-local and incarnation-fenced;
  - its dispatch-correlation index is rebuilt from durable outbox rows.
- **Deleted.** `CommittedReceiptNotRetained` is deleted, and the
  retained-receipt-basis cap no longer bounds answers. `worth-server`'s
  in-memory retry caches are deleted; Server retries go through Query's
  contract.
- **Affinity check.** The `runtime_instance_id` equality in resolution is
  replaced by home identity plus affinity.
- **Concurrency.** Same-key concurrent retries are serialized by the existing
  branch commit lane under the single writer (D7).

**D10. Retention classes.**

Every durable artifact belongs to exactly one class:

- **Authoritative.** Never expires while its home exists.
- **Mandatory obligation.** Retained until its owner discharges it.
- **Optional retained material.** May expire under a declared policy. Expiry
  answers a typed `RetainedMaterialExpired` and never authorizes re-execution.
- **Derived.** Rebuildable, and never authoritative.

Idempotency bindings, occurrences and custody dispositions are authoritative.
Reclamation is Milestone 4; this milestone declares the classes and never
reclaims authoritative or obligation data.

**D11. Close.**

`close` stops admission, then settles or durably retains accepted work within
its request deadline. It answers `Closed` or `CloseRefused { posture }`.

- On refusal, the home stays open and serving, or it enters an explicit
  recovery posture the caller can inspect. It never silently seals a runtime
  the caller was told remains open.
- Acknowledged results never depend on close; durability is reached at
  publication.
- Close attempts a home checkpoint before releasing the lease (D12). A failed
  checkpoint never refuses close. It is reported in `Closed`, and the next open
  replays the longer tail.

**D12. One home checkpoint, taken automatically.**

A home checkpoint is one manifest under the `home.checkpoint` anchor,
published as one root decision.

- **Sections.** The manifest has one section per owner. Each section uses the
  owner's own checkpoint format, with content-addressed members as in
  Milestone 2 D4. The Relational manifest becomes the Relational section
  using its current canonical format and declared clean version cutover.
- **Coverage and crashes.** Each section covers a position of its owner's
  journals, and recovery replays only the tails. A crash at any point of a
  checkpoint leaves either the previous or the new checkpoint selected.
- **Exact Native component obligations.** Capture selects current branch cells
  and live owner-issued `ExternalComponentBasis` obligations at one coherent
  owner capture boundary. It retains their actual roots, schema authorities
  and source custody through checkpoint selection. It does not inherit the
  previous checkpoint's entire physical union or retain unrelated observations,
  transactions or prepared candidates merely because they share a root.
  Acquiring or releasing an external obligation concurrently with capture has
  an owner-linearized result. A selected obligation remains protected after
  its original lease is released. A later checkpoint may omit it when no
  current cell or live external obligation requires it.
- **Cross-owner capture cut.** Before Native section capture, World's existing
  checkpoint owner freezes its exact root/open-custody snapshot and the actual
  external Native component leases needed by that snapshot. Retain that same
  snapshot and leases through Home manifest selection; encode the World section
  from this cut. Later admitted World publications remain journal tails. No
  owner mutex spans I/O, and a Native header snapshot alone never certifies a
  coherent Home cut or permits World to add an uncaptured historical reference.
  In the after-effect/caller-loss window, the cut refuses while an attempt is
  executing or loaned without World's actual successor component retention.
  Settle the existing retained handoff and acquire that real obligation before
  capture; artifact bytes cannot substitute for missing root custody. Select
  the Home checkpoint while Native is still Serving, then release capture pins
  before Native `prepare_close`; never bypass its genuine read-custody refusal.
- **No partial Home manifest.** Until World's checkpoint section is implemented,
  an attached or unclaimed Home with actual committed World journal heads
  refuses checkpoint with `WorldSectionRequired` before Native capture or
  manifest publication. Absence of heads grants no World reconciliation proof.
  Close reports this failed checkpoint and follows D11, retaining journal tails.
- **Triggers.** The host checkpoints automatically:
  - when any journal's tail since the last checkpoint passes a threshold
    (count and bytes, with defaults in the host limits);
  - at close.

  Applications may also checkpoint explicitly. Reopen work is proportional to
  the manifest plus the bounded tails, not to history.

**D13. One durable path; the memory-only mode stays explicit.**

The host has one open call, with two locations:

- **A directory.** Durable.
- **Memory-only.** Explicitly non-durable, with unchanged semantics.
  Memory-only evidence never qualifies durability.

Application-owned checkpoint transport is retired by clean cutover in Phase
10. That covers:

- `capture_application_checkpoint*`;
- `WorthQueryApplicationCheckpoint` byte transport;
- the `*_from_checkpoint*` constructors;
- `repair_to_checkpoint*`;
- Relational's embedded-checkpoint placement.

Until then it is a named interim path for callers whose state families are
not yet durable.

- **One door.** Phase 3 collapses it into a single door,
  `WorthQueryInterimCheckpointRestore`, taking the new home limits. Nothing in
  the interim path keeps an `in_memory` name or `InMemory` type.
- **Frozen callers.** Its callers are frozen in
  `tools/boundary-check/config/interim-checkpoint-callers.txt`. A new caller
  fails boundary-check, and callers are only ever removed.
Checkpoint-transition and program-adoption contracts move onto the open home
(D16, Phase 6). The roadmap's former "explicit embedded-save" mode is retired
with this path.

**D14. Interruption controls and the host oracle.**

- **Store faults.** Store faults and pause gates use Store's
  `certification-test-authority` feature. `worth-store-application` forwards
  it, as `worth-store-relational` does, inside Store's one-build exemption.
  No feature is declared outside the Store workspace.
- **Owner boundaries.** Every owner effect, custody disposition, transition
  step and product movement is a Store publication, so Store's mutation seams
  interrupt every owner boundary.
- **Oracle location.** The host oracle lives in
  `worth-store-court-application`, using the fixture/reference library
  `worth-store-testkit-application-oracle`. Court children re-execute the test binary,
  as Milestone 2's relational_home children do, and run the real production
  host.
- **Derived-state door.** Discarding derived state uses a sealed certification
  door on the home (`discard_derived_for_test`). Its callers are recorded in
  `control-callers.txt`.
- **Owner unit courts.** Owners outside Store use `cfg(test)` in-crate media.
  Those are not a second durable path.
- **The oracle's identity rule.** The reference is the ordinary MemoryOnly
  host, issuing normal ephemeral identities without a durable binding,
  Home-scoped identity, frontier snapshot or reset. One typed run-wide
  `DurableIdentityCorrespondence` bijection relates Home durable identities
  to reference identities across every reopen. Its `admit(home, reference)`
  refuses either a Home identity mapped to a second reference identity or a
  reference identity reached from a second Home identity. Admit identities
  only when they first appear in compared durable graph state, occurrences,
  custody, product heads or idempotency outcomes. Process-local identities and
  identities consumed by attempts that never became durable stay out.
  Identity meaning is compared by correspondence, which needs no durable
  reference. The separate required test that identities issued after reopen
  never collide with recovered ones remains unchanged.
- **Seeds and lanes.** A failing seed reproduces deterministically. Focused
  lanes run a small seed range. A scheduled lane runs the broad campaign under
  a stated time and memory budget.

**D15. Typed unsupported families, for directory homes.**

A `DurableStateFamily` enumeration names every durable family from the Phase 1
audit. Each family maps to the operation kinds and declared capabilities that
can create it. The mapping includes capabilities that create the family only
on failure, such as managed resources, which can leave managed cleanup.

Until the phase that supports a family lands, on a directory home:

- an operation whose kind or declared capabilities can create the family
  refuses at operation admission with `UnsupportedDurableFamily { family }`;
- opening a directory home that holds the family refuses before service.

Memory-only homes are never refused. Each phase's "Interim refusals" lists
exactly which operation kinds refuse until it lands. It also states, for each
existing caller (bank world, Server, examples, fixtures), which
location that caller uses meanwhile. Phase 10 deletes every variant.

**D16. Program transitions are durable and resumable.**

Opening a home with a program or schema revision different from the one
recorded requires a transition. The open request carries it as an optional
part: predecessor, transition author, resources and dispositions. Opening
without a required transition refuses with `TransitionRequired`, naming the
recorded and requested revisions.

A transition proceeds in this order:

1. **Plan.** The transition author computes its complete step plan against
   the readmitted predecessor state.
2. **Intent.** Publish a `HomeProgramTransitionIntent` to `home.transitions`.
   It names the predecessor, the successor, the transition identity and the
   digest of the complete step plan.
3. **Steps.** Publish the migration steps: Relational schema transitions,
   commits and dispositions. Each step is positioned, and each carries the
   transition association.
4. **Binding.** Publish the descriptor's program binding.

At open step 8 (D8), an intent without a binding is pending. Open recomputes
the plan from the same predecessor state and checks it against the recorded
digest:

- **The plan matches.** The transition resumes from the first step that is not
  durable. Durable steps are never re-run.
- **The plan differs.** Open refuses with `TransitionPlanDrift`. A developer
  who edited the migration after a crash cannot mix two migrations.
- **The successor differs.** Opening with a different successor refuses with
  `TransitionPending`.

A pending transition can only be finished with its original plan, or
discarded with the home.

The supported development workflow is:

- **Edit and reopen.** After editing a program or schema, reopen with the
  transition.
- **Discard.** Discard the home through the host's `discard_home(location)`.
  - It does not open the Store. It takes the filesystem mutation lease, so an
    open home cannot be discarded.
  - It first writes a discard tombstone at the location, then removes the
    Store.
  - It is idempotent. It completes a partly discarded or damaged location.
  - Opening a tombstoned location refuses with `DiscardPending` until
    `discard_home` completes.

**Cleanup track: moved courts.** The following transition-source recovery
courts leave the happy-path tree under the M3 track split. Restore and migrate
them against real Store Home/World authority; do not ignore or gate them.

- `transition_source_recovers_the_exact_durable_prefix_and_original_planning_root`: Recovers the exact durable transition prefix and original planning root, keeps that historical root distinct from the current execution basis, and readmits successor schema version 2.
  `restore from 34edd18ff6:crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/home_program_transition_source.rs`. Migration needs catalog-grant access before
  cutover and a replacement for the expected-zero prefix scenario, because
  zero-step plans are prohibited.
- `transition_source_reads_its_planning_predecessor_from_the_selected_admission_image`: Reads the original planning predecessor from the selected admission image, preserving one predecessor row and two current rows after the durable migration tail.
  `restore from 34edd18ff6:crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/home_program_transition_source.rs`. Migration needs catalog-grant access before
  cutover and a replacement for the expected-zero prefix scenario, because
  zero-step plans are prohibited.
- `transition_source_refuses_a_missing_or_foreign_prefix_before_runtime_cutover`: Refuses a missing or foreign prefix with a typed refusal before runtime cutover; the receiver remains empty.
  `restore from 34edd18ff6:crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/home_program_transition_source.rs`. Migration needs catalog-grant access before
  cutover and a replacement for the expected-zero prefix scenario, because
  zero-step plans are prohibited.

**D17. Type-enforced invariants, placed by the vocabulary rule.**

The decisions above are invariants. Each one is enforced by the compiler
wherever the arch laws require it (laws 3, 5, 9, 10, 13 and 16). A prose rule,
a review finding or a test is enough only where this decision says so.
Implementers are mostly AI agents. For them, a compile error at the edit site
is reliable. An invariant stated 1,400 lines away in a specification is not.

*Placement.* Before a new type is designed, it is placed by the AGENTS.md
rule:

- **`worth-proof`** holds anything that decides whether an operation is
  legal: witnesses, phase proofs and fenced authority.
- **`worth-foundational`** holds anything that means the same thing when it
  crosses a boundary into another runtime: identities, associations, fates,
  classes and families.
- **The owning runtime** holds anything that needs a clock, a counter, a live
  table or `Drop`: issuers, the lease, the incarnation advance and custody
  registries.

New vocabulary reuses what these crates already provide:

- `worth-proof`'s `AuthorityWitness`, `CapabilityWitness` and minting
  patterns;
- `linear::LinearResource`, for values that must end exactly once;
- `phase` markers;
- `brand`, only within a scope. Its documented limit applies: a long-lived
  carrier uses a runtime-owned identity value instead.
- `worth-foundational`'s `boundary_protocol` versioning and
  unsupported-version refusal, for every artifact frame (law 21).

The implementer verifies that the crate dependency exists before concluding
that vocabulary is missing. No parallel vocabulary is introduced.

*Invariants and their mechanisms.* The table names the decision each
invariant comes from.

| Invariant | Mechanism | Placement |
|---|---|---|
| Durable artifacts record only durable identity (D6, R17) | A sealed `DurableIdentity` marker trait. Artifact codecs accept only fields whose types implement it. Process-local identities (runtime instance, session, graph instance, participation pointer, installed-runtime ordinal) never implement it, so recording one does not compile. | Trait and home-scoped identity values: `worth-foundational`. Issuers: owning runtimes. |
| Durable before live (D2, D4) | The live cutover and the live product compare-and-swap each take a `DurablyCommitted` witness, move-only and consumed by the cutover. Only the owner's medium binding mints it, and only from a `Committed` answer. `ProvenNoEffect` and `Indeterminate` mint nothing. | Witness type: `worth-proof`. Minting: owner runtime. |
| Publication fates (D2) | One owner-neutral `DurablePublicationFate`: `Committed`, `ProvenNoEffect`, `Indeterminate`. Every owner port answers with it, and it never collapses to `Result` or a boolean. Owner-specific detail rides in typed payloads. M2's `RelationalDurablePublication` adopts it. | `worth-foundational`. |
| Recovery order (D8) | Typestate with plain, named proof structs, one per step: `StoreRecovered`, `HomeAdmitted`, `RelationalReadmitted`, `SignalReadmitted`, `WorldReconciled`, `QueryReadmitted`, `CustodyReadmitted`, `TransitionSettled`. Each step consumes its predecessor's proof. Service is constructible only from the last one. Adding an owner breaks compilation at every composition site until it is wired in (law 9). | Proof structs: `worth-proof` phase markers. Sequencing: `worth-store-application`. |
| Fencing (D7) | Authority values and durable data are distinct sealed types. Authority values are capabilities, handles, sessions, leases and recovery claims. They carry the home incarnation and are never `DurableIdentity`. Durable data is receipts, occurrence and commit identities, and cursors. It carries no incarnation and is never authority. The cross-process check stays a run-time comparison, because old in-memory values die with their process. | `HomeIncarnation` value: `worth-foundational`. Fenced-authority shape: `worth-proof`. Advance: Store home. |
| Receipts from durable state only (D9) | The original-receipt constructor takes only readmitted durable inputs: the binding, the occurrence and the owner artifacts. Building a receipt from the evidence index is uncallable. | Query. |
| Unsupported families (D15) | One declaration: an exhaustive `match` over `DurableStateFamily` returning `Supported` or `RefusedUntil(phase)`, with no wildcard arm. Operations declare their capabilities as types. Admission computes refusal from those declarations, never from hand-written checks at call sites. | `DurableStateFamily`: `worth-foundational`. Support table: `worth-query-host` composition. |
| Retention class (D10) | An associated constant on every artifact type, matched exhaustively wherever retention is decided. | `RetentionClass`: `worth-foundational`. |
| Custody that ends exactly once (D5, D11, R11) | Custody dispositions, transition intents, managed-cleanup obligations and pinned correction material are `LinearResource`s. Their terminal transition takes `self`. | `worth-proof` `linear`. Owners implement it. |
| Attempt association (D5) | `CompositeAttemptAssociation` is one `DurableIdentity` value carried by every owner effect. Owners treat it as opaque. | `worth-foundational`. |
| Continuous lease (D7, Phase 2) | The lease owner value moves from recovery into serving. No code path holds neither. | Store. |

*Agent-oriented type style* for all new durability code:

- **Plain named types.** Prefer concrete, named proof and witness structs
  over generic proof towers such as `Proof<Stage<Owner<..>>>`. Keep generics
  shallow, and add no new macros or blanket impls.
- **Exhaustive matching.** Every `match` over a durable family, fate, class,
  retained decision (R9) or recovery step is exhaustive, with no `_` arm, so
  that adding a variant lists every site to update.
- **One meaning per name** (law 10). Names are unique and greppable.
- **Enforcement or a traced test.** Every invariant in this specification is
  either compiler-enforced or covered by a test whose name states the
  behaviour, such as `live_cutover_requires_durable_commit` (iteration law 16:
  no milestone, decision or ticket ids in names). Each phase's requirement map
  traces requirement ids to those tests.
- **Explicit over clever.** Explicit, repetitive code is acceptable. An
  abstraction is acceptable only when it shares semantic authority, lifecycle,
  failure topology and cost class (law 12).

*Phase binding.*

- **Phase 2:** the continuous-lease row.
- **Phases 3 and 4:**
  - placement;
  - `DurableIdentity`;
  - `DurablyCommitted`;
  - `DurablePublicationFate`, including the adoption by Relational's port;
  - the recovery typestate, with the owners that exist by then;
  - fencing types;
  - the receipt constructor;
  - the `DurableStateFamily` declaration;
  - `RetentionClass`;
  - `CompositeAttemptAssociation`;
  - `LinearResource` custody for dispositions and transitions.
- **Phases 5 to 9:** each extends the typestate, the family declaration and
  linear custody for its owners. A phase is not complete while any of its
  invariants is enforced only by prose.

## Anchors and journals

These are the closed registry of D3. A name is final when the phase that
creates it lands.

| Name | Kind | Owner (format) | Class | Phase |
|---|---|---|---|---|
| `home` | Anchored record | Store home: identity, incarnation, program/schema/build binding | Authoritative | 2–3 |
| `home.checkpoint` | Anchored record | Store home manifest; owner sections | Derived | 3 |
| `home.transitions` | Journal | Store home: transition intents (D16) | Authoritative | 3 |
| `relational.canonical-commits` | Journal | Relational (with caller association) | Authoritative | M2, 4 |
| `relational.branch-transitions` | Journal | Relational | Authoritative | M2 |
| `world.occurrences` | Journal | World occurrence | Authoritative | 4 |
| `world.branches` | Journal | World product branch lifecycle | Authoritative | 4 |
| `world.custody` | Journal | World custody dispositions | Authoritative | 4 |
| `signal.definitions` | Journal | Signal definition transitions (with Bridge section) | Authoritative | 5 |
| `signal.advances` | Journal | Signal branch advances | Authoritative | 5 |
| `query.obligations` | Journal | Query: support lifecycle, adoption, required-output custody (R8), managed, conditional, owned-async custody, original-owner custody | Obligation | 6–7, 9 |
| `query.dispatch` | Journal | Query: dispatch attempts, inbound custody, recovery claims | Obligation | 8 |
| `query.corrections` | Journal | Query: retained correction material and custody | Per contract | 9 |

The M2 `relational.home` and `relational.checkpoint` anchors are replaced.
`relational.home` is replaced by `home`, and `relational.checkpoint` by the
Relational section of `home.checkpoint` (D12).

Query's idempotency binding is not a journal. It travels inside the
Relational commit (D9).

---

## Phase 1: State survival audit

**Goal.** Classify every production-owned state family before any of it is
made durable. This gives every later phase an exact scope and every refusal an
exact name.

**Requirements**

- **Inventory.** Cover every state family held by Relational, World, Signal,
  Bridge, Query (installation, execution, publication), Server and the bank
  world. For each one, record:
  - its owner and owning type, with path;
  - its survival class (D10), or that it is process-local;
  - its current persistence: none, a Relational row, or in-memory;
  - the process-local identities it records (D6);
  - the phase that makes it durable. If it stays process-local, record how it
    is reacquired through fresh authority.
- **Unsupported families.** Name the families that become
  `DurableStateFamily` variants. Map each to the operation kinds and
  capabilities that can create it (D15).
- **Durable rows with process-local identities.** List every durable row
  recording `runtime_instance_id` or any other process-local identity, with
  the phase that replaces it. This includes digests derived from such
  identities, such as transport causal digests, workflow receipt digests and
  runtime source commitments. The causality rows' `world-owner` is an
  example.
- **Known defects.** Record each with its target phase:
  - the temporal idempotency key is derived from per-process runtime
    identities;
  - `PayloadRejected` is classified as unresolved;
  - support lifecycle resets to Active at install;
  - only the main branch's program activation is verified on restore.
- **Interim-path inventory.** List every public entry path retired by D9 and
  D13, with its callers. For each caller, give the location it uses in each
  interim phase.

**Deliverable.**

- The index is `plans/database-foundation/milestone-3-survival-audit.md`. It
  holds:
  - the `DurableStateFamily` list and its capability mapping;
  - the table of durable rows that record process-local identities;
  - the defect table;
  - the interim-path table.
- Per-owner inventory parts live in
  `plans/database-foundation/milestone-3-survival-audit/`.

**Required tests.** None; this phase writes no code.

**Exit criteria**

- An independent reviewer samples at least twenty audit rows against the code
  and finds them accurate.
- Every family and every row with a process-local identity that a later phase
  names appears in the audit, and no family is unclassified.

### Audit resolutions

The Phase 1 audit raised eighteen gaps in the specification. Each one is
resolved below and tagged with the phases it binds. A phase's requirements
include every resolution tagged with it.

- **R1 (Phase 1, index).** Part A audits Relational as the carrier of rows.
  The rows themselves are written by callers (idempotency, outbox, workflow,
  causality and others), and Parts B and C audit those. The index joins
  carrier and caller into one family per row kind, named after the caller.
  There is no generic "caller rows" family.
- **R2 (Phase 5).** Signal's authored topology is the declared node
  definitions plus the static dependency edges those definitions declare.
  Everything else is derived and rediscovered by evaluation:
  - dependency snapshots;
  - subscriber and evaluation edges;
  - dynamically discovered dependencies;
  - cause sets;
  - diagnostics.

  The Phase 5 authored-image format states this split before it is final.
- **R3 (Phase 5).** The `worth-signal-wasm` facade's `RuntimeStore` and
  portable envelope are a separate product surface. Neither is opened through
  the Query host, and both are outside this milestone. JS callbacks are never
  serialized. D15 does not apply to the facade.
- **R4 (Phase 5).** Some records are process-local diagnostics, not durable
  state: Bridge's writeback execution, replay and comparison records, and
  Relational's diagnostic artifact vector.
  - Their accessors gain a typed answer that distinguishes three cases: never
    retained, expired under count limits, and not retained across reopen.
  - Any evidence that recovery needs must be derivable from durable commits.
    For writeback, Phase 5 proves this.
  - `SignalSnapshotV1` stops being a restore authority, because the Phase 5
    artifacts replace it. Its restore door is deleted, or stays memory-only
    if the audit shows a memory-only consumer. It is never a second durable
    path.
- **R5 (Phases 5 and 7).** Two Signal structures hold real request custody and
  are not derived: `ResourceRuntimeState.in_flight_by_request` and native
  managed admission.
  - Until Phase 7, operations that admit Signal resource requests refuse on
    directory homes under D15.
  - Phase 7 extracts that custody into a durable form before any remaining
    resource state is treated as derived.
- **R6 (Phases 1 and 8).** The outbox wording is corrected above. Phase 8
  removes the process-local identity from every row and digest that the audit
  lists for it.
- **R7 (Phases 7 and 9).** Some run kinds retain opaque process objects:
  provider checkpoints, memory arenas, artifact contexts and `Arc<dyn Any>`
  commit authority. Serializing such an object is never allowed. Each of
  these run kinds takes one of two routes:
  - an owner-defined, versioned data contract and readmission;
  - a terminal posture that its existing lifecycle contract already defines.

  The phase's first deliverable is a per-kind table naming the route each run
  kind takes.
  The [Phase 7 custody routes](milestone-3-long-running-custody.md) select those
  routes and inventory every retained conditional decision. They distinguish
  existing lifecycle contracts from the durable consumers still to implement.
- **R8 (Phase 6).** General required-output custody is a durable obligation in
  `query.obligations`. That covers `DemandRecord` performed obligations and
  prerequisites, and `SourceCustody`. Ready outputs remain derived.
- **R9 (Phase 7).** Every variant of the conditional retained decision is
  classified:
  - **Durable obligations**, because they carry an effect or an obligation:
    CleanupRequired, Retryable, CommitRetryable, ExecutionControlRetryable,
    ProductStale, ProductUnpublished, Indeterminate and
    InterruptedDuringReentry.
  - **Process-local**, because they can be re-derived from durable state and
    host time: Eligible, Deferred, Suppressed and Backpressured.
  - **Terminal**: ControlStopped, TerminalFailure, Failed and NoEffect. These
    are recorded durably exactly when a caller can still ask for them.

  Phase 7 publishes this table with its evidence.
- **R10 (Phase 7).** Workflow proposal coverage is a set of graph records: it
  is authored, and it recovers with Relational. Installed vocabulary coverage
  is derived and is recreated from the supplied program. An installed
  interpreter is never persisted as authored coverage.
- **R11 (Phase 9).** Each retained use of correction material declares its
  D10 class.
  - While a prepared correction or a recovery binding pins the material, it is
    a mandatory obligation.
  - Unpinned material is optional.

  Pinning, and transfers of a pin, are recorded together with the class.
- **R12 (Phases 4, 5 and 7).** D15's capability mapping splits branch cleanup
  by component. World native cleanup is supported from Phase 4, Signal cleanup
  from Phase 5, and conditional-assembly cleanup from Phase 7. On a directory
  home, a branch operation refuses if its cleanup includes a component that
  is not yet supported.
- **R13 (Phase 8).** Revocation and retirement of an inbound source are
  home-level security facts, and they are durable. Reopening never resets a
  revoked source to Active. Verifier installation is fresh host configuration
  applied on top of that durable posture.
- **R14 (Phase 8).** The bank external rail is the external party. Its own
  survival is outside WORTH, and Phase 8 does not make it durable.
  - Phase 8's rail courts kill the application, not the rail.
  - One court restarts the rail with its state lost. The application must not
    claim completion: the effect stays indeterminate and is reconciled under
    the rail's admitted contract.
- **R15 (Phases 3, 4 and 9).** `BankHttpMutationContinuation` is transport
  custody. It is re-created fresh at open.
  - The durable original owner is the per-operation record in its worker
    records and in `mutation_custody`. Phase 9 makes that record durable.
  - Until then, actions that retain original-owner custody refuse on
    directory homes under D15.
- **R16 (Phases 3 and 9).** Recovery of HTTP state:
  - HTTP recovery and continuation tokens carry authority, so they are fenced
    by incarnation (D7). After reopen they answer `StaleIncarnation`. A fresh
    authentication then reaches the recovered durable custody by its
    operation identity.
- **R17 (Phase 6).** Installation splits its canonical identities into two
  kinds:
  - **Semantic identity** is derived from content and may be durable.
  - **Authority identity** (installed-runtime ordinals and participation
    pointers) is process-local and fenced.

  Durable artifacts record only semantic identity. Equivalence checks compare
  semantic identity, plus the authority re-established at open. They are
  never dropped.
- **R18 (Phase 3).** Server upload staging is process-local. Interrupted
  upload sessions end at reopen with a typed `UploadSessionExpired`. Durable
  ingress is outside this roadmap.

---

## Phase 2: Store capacity, single writer and incarnation

**Goal.** Give Store what an application home needs, before any owner uses it.

**Requirements**

- **Schema 12.** Root schema 12 with 32 anchors (D3):
  - successor-root builders, maintenance and residue publications carry the
    table forward unchanged;
  - schema 11 is refused with `UnsupportedRootSchema`;
  - the offline integrity observer reads schema 12.
- **Journal registry.** The journal registry is declared by the home crate. A
  publication to an unregistered name refuses before any effect.
- **Lease handoff.** The lease is handed off continuously from recovery to
  serving (D7). The drop-and-reacquire window at `physical_store.rs` recovery
  is deleted.
- **Incarnation.** The home incarnation is recorded in the descriptor and
  advanced durably on every open, before service.
- **New-home detection.** Initialization happens under the lease (D7).
  `location_is_unused` stops deciding newness without the lease.
- **Milestone 2's home.** It runs on schema 12, with no other behavioral
  change.

**Required tests**

- **Anchor table:**
  - 32 anchors publish, recover and carry forward through maintenance;
  - a 33rd anchor refuses with `AnchorTableFull`;
  - the WAL recovery projection replays advances of high-index anchors.
- **Refusals before any effect:**
  - a schema 11 root refuses at open;
  - a publication to an unregistered journal refuses.
- **Two processes open the same location concurrently.** Exactly one serves;
  the other refuses with `MutationOwnerContended`. Covered for an empty
  location, an initialized home and a home that needs recovery.
- **Kill and reopen.** A process holding a home is killed. The next open
  recovers, and the incarnation increases by exactly one per open.
- **Lease held through recovery.** An observer process fails to acquire the
  lease at every recovery yieldpoint and at the handoff.
- **Offline integrity observer.** It reads a schema 12 root.

**Oracle extension**

- Milestone 2's relational_home oracle runs once on schema 12: media sweep,
  process kills and recovery interruption.
- New media and process cases cover the incarnation advance at open.

**Docs.** The Store README and the `worth-store-relational` guidance cover the
schema, the lease and the incarnation.

**Exit criteria**

- Landing 1's Tier-2 run is green on its head (see "Verification tiers"),
  with the Store crash oracle run exhaustively.
- Final-gate certification.

---

## Phase 3: Application home and host lifecycle

**Goal.** The production Query host opens, recovers, serves, checkpoints and
closes a file-backed home through one call, with every owner composed by the
home.

**Requirements**

*Home and boundary*

- Create `worth-store-application` and apply the D1 boundary ruling.
- Delete `RelationalStoreHome`. Its tests and the Milestone 2 relational_home
  oracle move to the application home's Relational section in
  `worth-store-court-application`, using
  `worth-store-testkit-application-oracle` for shared authored fixtures.
- Record the measured build-time impact on every lane in the phase evidence
  (D1).

*The open call*

- `worth-query-host` exposes one open call. Its home request carries:
  - the location, either a directory or memory-only;
  - the validated program and support roster;
  - the application build identity;
  - trusted runtime services (time source and World resources);
  - the execution authority;
  - limits, including the checkpoint thresholds (D12);
  - `initial_state`;
  - an optional program transition (D16).
- The `in_memory*` constructors are deleted. Their memory-only callers move to
  the open call, and "InMemory" disappears from the limits, profile and denial
  names.
- The `*_from_checkpoint*` constructors collapse into the single
  `WorthQueryInterimCheckpointRestore` door (D13). Its callers are frozen in
  boundary config.
- A new home is distinguished from an existing one only through Store (D7).
  `initial_state` runs only for a new home.
- `discard_home(location)` (D16).

*Program and build binding*

- **Descriptor.** The home descriptor records the program identity and
  revision, the rostered revisions, and the schema revision.
- **Program transitions** follow D16. An incompatible program refuses before
  service.
- **Changed build identity.** Opening under a changed build identity is
  admitted. Authored truth is unchanged. Recorded derived outputs whose
  producing implementation differs are stale under the existing lineage
  contract, and nothing regenerates eagerly. Every rebuild mints a new build
  identity, so this is the ordinary local-iteration path.

*Close, fencing and checkpoints*

- Close follows D11; incarnation fencing follows D7; automatic checkpoints
  follow D12.

**Interim refusals (Phases 3–4).** On a directory home, these refuse per D15:

- operations that touch Signal or Bridge state;
- conditional, temporal and managed operations;
- workflow runs;
- dispatch and inbound completion;
- corrections;
- program adoption.

**Unchanged initial components for the Relational-only lane.** A directory
Home may compose an owner-proven initial Empty Signal root and an owner-proven
Bridge baseline without enabling any Signal/Bridge mutation. Arbitrary authored
Signal graphs, even if an operation would leave them unchanged, remain
unsupported until Phase 5. Refuse an unsupported seed before the first durable
World bootstrap or owner effect; successful live bootstrap cannot create a
Home whose required component meaning is already unrecoverable.

Signal owns a versioned `RecordedSignalInitialEmptyRoot` description and its
admission. The sole `SignalOwnerPartition` to `SignalOwner` transfer can mint
the private initial-root witness only after inspecting actual state: one
original root, initial generation, no restore/snapshot/fork/retirement/merge
history, no allocated/authored nodes or operators, no authored values or
installed definition, and no accepted Signal movement. The witness remains
bound to that exact owner/cell/incarnation and actual initial semantics.
Existing owner admission and cell access guard later export/readmission;
closed, replaced or advanced cells refuse. Do not add an operation counter,
second registry or a caller-supplied emptiness flag. The folded `definition_basis`
alone does not prove any of these facts.

The recorded description has a required owner schema version and InitialEmpty
tag with its owner-defined stable schema/visibility/definition commitments.
Its meaning is the completely unmodified initial Empty root, not an old
process's admitted descriptor or pointer. On reopen, Signal verifies this
recorded contract against the actual newly composed initial owner and mints a
fresh runtime-local admitted basis with real retention. Neither World nor
Store rewrites an old graph/runtime ordinal or substitutes an arbitrary
current basis. Data is descriptive until this owner admission succeeds.

Bridge owns a versioned `RecordedRuntimeWorldBaseline` description and
readmission through its existing correspondence admission owner. Export
requires the actual admitted baseline with `dependency=None`, no installed
correspondence meaning, and the actual registered Native source. Record the
stable adapter semantic identity and baseline role/semantic commitments;
process runtime keys and admission identities are not durable authority.
Readmission requires the same private Store-composed Native owner, its actual
registered source/profile, and Signal's newly owner-admitted InitialEmpty
basis. Bridge compares stable meaning and issues fresh Bridge admission using
those actual owners. A missing source, changed adapter/role, installed
dependency, unsupported version or stale/foreign live owner refuses. There is
no public raw Native/Bridge exposure, graph-ID rewrite, generic restore trait
or caller-provided source/fate pairing.

World records both owner descriptions in its original Branch artifact, bound
to original product identity, program revision and exact Native predecessor.
Use World artifact v3 with mandatory seed fields; earlier v1/v2 development
frames refuse before effects, with no defaults or migration. Store privately
sequences Native recovery, Signal seed admission, Bridge baseline admission,
then World reconciliation. Original World identities, heads, attempts and
custody are unchanged. General nonempty Signal/Bridge authored recovery
remains Phase 5's owner artifact contract and does not reuse this Empty proof.

Place Signal meaning/export at existing `branch/recorded_initial_root.rs`
and owner witness/admission beneath `branch/owner_services/`; expose only its
branch facade. Place Bridge recorded baseline beneath existing
`correspondence/runtime_world_admission/`; expose its existing facade. World
seed encoding/readmission stays under its `durability/` owner; filesystem
composition stays in Store's existing `world_section/` stage. Create only
populated responsibilities, with no generic seed registry or placeholder tree.

The real kill/reopen Home journey uses an actually Empty Signal program. It
must recover the same World head and original unpublished Native custody with
new owner-admitted local component bases, release or continue, then reopen
again without handler execution or effect republication. Nonempty and
modified-then-empty Signal worlds, snapshots/forks, installed Bridge
dependencies, altered adapter/role, missing seed fields and foreign live
owners must fail before effects for the intended reason. Actual seed bases
must retain/release owners honestly under Home close. Owner tests and public
construction fences support, but do not replace, this filesystem proof.
Signal/Bridge READMEs, capability index and the Home build guide describe the
Empty-only supported lane and typed refusal of authored state until Phase 5.

Meanwhile, the bank world, Server and Signal-dependent examples
use memory-only homes or the interim checkpoint path, as the audit lists.

**Required tests**

- **Open and reopen:**
  - opening an empty directory initializes the home and serves;
  - reopening recovers;
  - a damaged descriptor, a foreign home, an unknown root schema and an
    incomplete recovery each refuse before service, with typed reasons.
- **Competing opens at host level.** Two processes open one directory through
  the host. One serves; the other gets a typed contention refusal.
- **Rebuild.** Same program and schema, new build UUID: the home opens with no
  refusal, and stale outputs are reported without being regenerated.
- **Program transitions:**
  - a schema edit without a transition refuses with `TransitionRequired`;
  - with a transition, it migrates and serves;
  - a process killed at each seam of the transition (intent, each step,
    binding) reopens and resumes from the first undurable step, without
    re-running a durable one;
  - a resume whose recomputed plan differs from the recorded digest refuses
    with `TransitionPlanDrift`;
  - opening with a different successor while a transition is pending refuses
    with `TransitionPending`;
  - an unrostered or incompatible program refuses.
- **Discard:**
  - `discard_home` on an open home refuses;
  - on a closed home it removes the home, and the next open initializes;
  - a process killed during discard leaves a tombstoned location;
  - opening that location refuses with `DiscardPending`;
  - running `discard_home` again completes, including on a damaged Store.
- **Frozen interim callers.** Adding a caller of the interim checkpoint door
  fails boundary-check.
- **Fencing and durable data.** A capability, handle and session from
  incarnation N refuse with `StaleIncarnation` at N+1. A receipt and an
  occurrence identity from N still resolve at N+1.
- **Close:**
  - with accepted work in flight, close either settles it or refuses with an
    inspectable posture;
  - a refused close leaves a serving home;
  - acknowledged commits survive a kill without close.
- **Checkpoints.** After more commits than the threshold, a checkpoint exists.
  The reopen journal tail stays at or below the threshold, measured through
  the home's read counters with headroom.
- **Memory-only.** Memory-only mode passes its existing semantic suites through
  the new call.

**Oracle extension.** `worth-store-court-application`, using the shared
`worth-store-testkit-application-oracle` library, covers home open,
initialization, reopen, program transition, automatic checkpoint and close.
That includes the media sweep and process kills at every Store seam of each,
and recovery interruption at every stage.

**Docs**

- The host README sections on open, close, checkpoints and transitions.
- `build-an-application.md` §4 "Install or reopen", including the development
  workflow (D16).

**Exit criteria.** This phase lands with Phase 4, under Phase 4's exit
criteria.

---

## Phase 4: Durable occurrences, interrupted publication and original-result retry

**Goal.** The decisive journey: a lost response after a durable commit, a
kill, a reopen and a retry return the original occurrence, without
re-executing anything.

**Requirements**

*World becomes durable*

- World defines and readmits these artifacts:
  - `RuntimeWorldOccurrenceArtifact`. It records:
    - the composite commit identity and parent;
    - the home identity and the publishing home incarnation;
    - the product branch identity, incarnation and generation;
    - the attempt identity and provenance;
    - the program revision, populated from this phase on;
    - caller correlation;
    - an extensible, owner-tagged list of component evidence.

    Phase 4 defines the Relational entry: commit id, position and digest.
    Later phases add tags, never fields.
  - `RuntimeWorldProductBranchArtifact`: create, retire and incarnation.
  - `RuntimeWorldCustodyDisposition`: continue or release (D5).
- World's identities become home-scoped (D6). The process-static owner
  ordinal no longer reaches any artifact or row.
- Product movement is durable before live (D4).
- On reopen, World rebuilds its history catalog, product heads, ancestry and
  open custody from its checkpoint section plus its journal tails.
  Pre-crash history pages, ancestry tracing and `recover_performed` work
  after reopen.
- Reconciliation follows D5. Pending settlement is covered: a Relational
  commit that is durable but unsettled is recovered by Relational, and World
  holds its custody. `continue_effects`, `release_effects` and
  `prepare_settled_relational_adoption` operate on recovered custody.
- Milestone 2's unresolved forks retained as `ProductUnpublished` survive
  reopen as the same custody, with their native fate decided by recovery.

*Relational association*

- **Original occurrence backing.** Before Native effects, reserve the actual
  occurrence frame's charged backing/capacity through existing
  `ExecutionByteBuffer` under the original execution/allocation policy. Retain
  that same buffer in the original `ActiveAttemptResources` beside its durable
  OpenAttempt and independently issued World identities. Candidate-known
  receipt/root data may inform admission; Native's actual position and artifact
  digest are unavailable until publication and must never be predicted.
  After actual Native settlement, fill and seal that same buffer from the
  original positioned result and retained original successor basis using
  Native's descriptive retained-component export. A later Native head advance
  cannot rebind the occurrence. Do not replace stopped backing, request fresh
  grants to revive it, issue replacement World identities, or substitute the
  latest Native basis. Refusal/unwind retains original unpublished custody and
  actual owner evidence. Publish the World occurrence durably before live CAS;
  recovery uses distinct owner capability adoption, never a fabricated live
  `CommitResult`.

- The canonical envelope gains caller association and the original semantic
  result under artifact v3; both authoritative and embedded checkpoint
  projections carry them. CHK16 requires association discovery in its existing
  commit directory. Earlier formats refuse before effects (clean cutover).

**Native semantic result and recovery admission**

The adversarial case is a Native effect that advanced its branch but never
moved the product. After a kill, World must keep its original predecessor
product basis while recovering that exact effect. Substituting Native's latest
basis, fabricating a live result, or loading all historical bodies to discover
associations fails this contract.

One immutable, Native-owned `RelationalCommitSemanticResult` is assembled
during original preparation, before envelope sealing. The same semantic value
feeds live result assembly and the authoritative recorded image; live-only
fields stay separate. Do not serialize `CommitResult`, `CommitLog` or
`CommitSummary` wholesale, or introduce another result policy or ledger.

The existing producer already has the needed data. `AssembledCommitExecution`
owns structural summary, invariant executions, version, history and create
bindings. `prepare_commit_publication_execution` combines these with the
admitted log's semantic summaries and canonicalized changed records, then
installs the semantic value before the existing envelope `Arc::new` seal.
`CommitResultMaterial` and `CommitResultSeal` consume that same value for the
live result. Source-backed recovery exposes a distinct
`RecoveredRelationalSemanticResult`, not a live seal or result handle.

The durable projection has this explicit field boundary:

| Original source | Recorded semantic meaning | Excluded live or diagnostic data |
|---|---|---|
| Commit reference and version | Exact original receipt/version, bound to the enclosing envelope | Process-local transaction identity |
| Created entity bindings | Every canonical correspondence: partition, kind, resolved client-key text to assigned `EntityId` | Process-local symbol representation and lookup estimates |
| Created relation bindings | Every canonical correspondence: partition, kind, resolved client-key text, exact endpoints to assigned `RelationId` | Process-local symbol representation and lookup estimates |
| Created endpoint reference | `Existing(EntityId)` or `Created(partition, kind, resolved client-key text)` | Guesses from allocation order, later state or patch contents |
| Prepared changed records | Exact canonical ordered `Vec<RecordRef>` | A later graph scan |
| Structural summary | Invariant groups, commit topology, ordered touched partitions, descriptor semantics version | Bulk entity/relation slots reserved |
| Schema summary | Original transition and descriptor semantics version, consistent with the schema artifact | Current-policy reconstruction |
| History summary | Target branch, requested/effective merge-parent counts, parent/merge-base counts, previous-head presence | Later branch state |
| Change/aspect summaries | Original changed-record/adjacency and entity/relation aspect counts, touched aspects, opaque/zero-aspect structural deltas | Evaluation/emission traces |
| Patch/publication summaries | Actual patch-record count represented once, lineage-event and merge-parent counts | Preparation limit, diagnostic-entry count, snapshot ID |
| Validation executions and decisions | Original ordered metadata, checks, results and verdict meaning described below | Validator re-execution and process-local proposal identity |
| Validation summary | Read-only projection of the retained original validation meaning | Independent counters or another accounting authority |
| Retention | Authoritative root/member references and real owner pins remain in their existing owners | Timing, visit counts, charges and residency bookkeeping |
| Execution, snapshot and diagnostics | Nothing | Snapshot handles/IDs, timing, complexity, work/CPU/allocation measurements, late interruption and omitted traces |

Validation retains original execution point, observation kind, version/current
version, consumed/applicable groups, declared cost class, disposition, plan
contract and merged-plan presence. Preparation strategy/failure provenance and
proof scope/widening causes may be retained as historical categories, never as
current execution authority. Packet/traversal bookkeeping is excluded. Checks
retain execution point, failure effect, rule, canonical witness, groups, declared
cost class, custom provenance and complete verdict. Decisions retain those
meanings and their original decision. Declared cost classes describe the original
validation contract; they are not measured work. Process-local
`proposal_identity` is excluded. Numerical verdict output keeps its original
semantic value.

Canonical reference text comes from the original admitted symbol dictionary,
never display formatting of an unresolved symbol. Preserve every original
correspondence's meaning. Exact/stable aliases of one canonical key must agree;
conflicting assignments, missing text or incomplete semantics refuse preparation
before effects. Readmission validates ordering, correspondence consistency,
receipt/version/schema agreement, changed-record/count agreement and validation
result/decision agreement. It does not run handlers or validators.

There is no serialized `Published` flag or pre-recorded completion log event.
Receipt/version can bind before sealing; assigned journal position comes from
actual positioned evidence. Artifact digest is computed after encoding and is
not placed inside its own semantic image. Actual publication or verified recovery
source establishes commitment. Excluded live fields are unavailable after
recovery, never zero-filled or invented.

**Lazy association discovery (D12)**

The existing `RelationalCommitIndexEntry` gains one required, non-optional
`RelationalAssociationDiscovery` value, with variants `Unassociated` and
`Associated(RelationalCallerAssociation)`. It is derived from the actual
envelope when publication becomes indexed or its journal tail is readmitted.
The existing checkpoint commit directory carries it without demanding cold
envelopes. Missing metadata is malformed current-format data; no default can
hide an associated effect. Existing index/envelope checks compare discovery
whenever both are available.

World selects original subjects from this admitted directory and its opens,
occurrences and dispositions. Only selected artifact bodies and the predecessor
and successor roots actually required are demanded. Their source artifact must
match receipt, position, digest and exact association before capability issuance.
Neither discovery metadata nor a memory-exported artifact certifies durable
origin. Reopen cost is proportional to the manifest, bounded tails and demanded
root/artifact material. Unrelated historical bodies stay cold. Ordinary
publication performs no reconstruction scan. This adds no registry, control
publication or accounting framework.

**Exact Native references and recovered capability**

Recorded component references distinguish:

- `Empty`: branch identity, reference generation, truth version, lifecycle,
  schema and visibility commitments; no commit, position, digest or zero receipt.
- `Canonical`: original full receipt, position, artifact digest, root descriptor,
  branch/reference axes, truth version, lifecycle, schema and visibility.

Reuse Native's canonical runtime-frame conversion and
`FoundationalBranchTarget::Empty`/`Basis` meanings. World records its predecessor
before Native preparation; it cannot predict a future Native commit ID.
Descriptions never admit a basis. No path rewrites an old runtime ID or falls
back to the latest root.

**Checkpoint coverage of exact Native components**

Every canonical `RelationalCheckpointRoot` carries required, nonempty
`recorded_components: Vec<RelationalBranchBasisDescriptor>`. These are exact
current-cell or external-component observations in Native's existing canonical
runtime frame. Distinct observations may select the same immutable root;
identical observations are deduplicated and encoded in canonical order. Each
must agree with that root's actual commit descriptor and schema authority.

The manifest separately carries required `retained_empty_components`. Each
entry contains an exact Empty descriptor and `schema_carrier_digest`. Empty
has no invented commit, position, artifact digest or canonical root target.
The referenced schema carrier must exist and match its schema commitment.

The root directory contains exactly current canonical heads plus canonical
roots required by captured external-component obligations. Every current
canonical head is present; every additional root has an exact recorded
component. Missing fields, duplicate or conflicting observations, foreign
frames, target/root/schema disagreement and unbacked extra roots refuse.
Capture obtains external descriptors from the same actual observations that
created their existing retention obligations. Root equality, weak-registry
membership and copied caller descriptions cannot replace this provenance.
Extend existing retention records and capture ownership; add no registry,
control-journal publication or independent authority.

Readmission admits the directory and current heads without loading historical
retained artifact bodies. The existing recovered-root inventory combines
checkpoint component headers, current images and actual tail replay
observations. Historical entries retain exact admitted directory/source
identity and schema commitments; decoding does not make them operational bases.
Exact component admission selects the original header, verifies its complete
source artifact against receipt, position, digest, association and
root/schema/reference commitments, then materializes only the demanded
immutable region directory through existing root readmission. Real observation
and component pins are acquired before exposure. Refusals retain original
evidence and acquired custody. Unselected historical bodies and region payloads
stay cold. Checkpoint coverage neither proves settlement nor authorizes World
adoption.

Capture remains in `durability/authority/checkpoint_capture/`, with exact
obligation selection in existing `history/retention/`. Required headers and
validation remain in `durability/{data,log/native_checkpoint_codec}/`. Lazy
checkpoint/tail inventory and selected materialization belong under existing
`durability/authority/runtime_rebuild/`. Existing recorded-component/publication
admission remains the sole capability producer. Native README and capability
index explain this lifetime, selective admission and version cutover.

The adversarial proof externally retains an exact canonical predecessor,
advances Native without publishing World's occurrence, checkpoints, kills and
reopens. It admits the original predecessor and successor with original
semantics and real pins, releases or continues actual World custody, and
reopens again. Repeat with an externally retained Empty predecessor. Deleting
an unrelated historical body cannot prevent selected admission; missing or
corrupt selected material must refuse. Altered component axes/schema/root/source,
missing headers, latest-head substitution or checkpoint-as-settlement must
fail. Releasing the last external obligation permits a subsequent checkpoint
to omit that historical component. Owner tests support these boundaries but
do not replace the real Home filesystem journey.

Extend the existing `DurabilityRecoveryAuthority` with
`recover_with_authority(execution, plan)`, issuing existing
`RecoveredRelationalRuntimeAuthority` only after actual reconstruction. Extend
that owner with exact recorded-component/publication admission. Its existing
current-image admission remains distinct. Select the exact Journal entry/source,
release owner locks, demand and readmit that source's artifact, and compare all
receipt/position/digest/association/schema/root/reference commitments. Admit the
actual empty or canonical predecessor and successor through existing root/member
owners, acquiring real observation/component pins before exposure. Missing
historical material refuses; it does not justify a synthetic basis.

World may have only its recorded predecessor when a kill precedes occurrence
publication. Existing recovered Native authority therefore exposes
`recorded_publication_successor(execution, runtime, readmitted_effect)`: a
description derived from that exact effect's source/index and this
reconstruction's actual tail/current/checkpoint component image. It returns
Native's existing recorded-reference type, not an operational basis. Exact
original receipt, position, digest, target branch and reference/root/schema
axes must agree. Missing or ambiguous images refuse while retaining the
original effect; neither latest-current export nor guessed axes are allowed.
The existing recovered-publication capability still performs real owner
admission and pin acquisition. Its proof must recover without a caller-held
successor reference, including later Native head advancement.

Use one named move-only proof phase in `worth-proof`,
`RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>`.
Native owns the concrete private payload, its actual recovering runtime/source
binding, original semantic result and admitted predecessor/successor bases/pins.
Store/World bind home identity and incarnation through the actual private Home
stage; Native need not interpret World's association or invent a Home binding.
Only successful Native owner admission constructs the payload. World consumes
that concrete specialization through a distinct recovered-adoption branch and
moves custody into its existing retained owner. Generic markers, receipt data,
index entries and caller-provided fate pairs cannot construct it. Refusal retains
the original subject/evidence and any acquired custody.

Settlement observations distinguish actual matching performed-but-pending and
performed-and-settled route states. Existing pending repair acts on that same
record/route and remains receipt-only; World's live result stays `None`.
`CheckpointReadmitted` reports exact admitted directory/source coverage only,
not a preserved live result or settlement event. It authorizes no adoption by
itself. The capability requires actual source/body/root admission and the required
recovered owner state and pins. Pending work must be settled by its existing owner
before adoption; pending-record absence is never settlement proof.

Memory-only backing, interruption, stale/foreign binding, source/index mismatch,
wrong roots/schema/references, missing semantics and unavailable roots have typed
refusals. Recovery neither republishes an effect nor executes a handler, constructs
`PerformedRelationalCommit`, fabricates `CommitResultSeal`, or supplies a recovered
`Arc<CommitResult>`. Arbitrary recovery bytes or public traits do not manufacture
the concrete Store-private `WorldReconciled` Home stage.

The destination within the existing owners is:

```text
worth-relational/src/
  transactions/data/outcomes/
    commit_semantic_result.rs                 # shared immutable meaning
    commit_semantic_result/{creation_correspondence,validation}.rs
    commit_result.rs                          # live-only seal plus same meaning
  authority/commit/pipeline/
    semantic_result.rs                        # original producer, before seal
    publication_execution/preparation.rs      # install image before Arc seal
    result_assembly.rs                        # consume same value
  history/commit/index_entry.rs               # required discovery metadata
  history/data/canonical_commit_envelope/     # both existing projections
  durability/canonical_commit_artifact/       # v3 strict readmission
  durability/log/native_checkpoint_codec/     # CHK16 strict directory/image
  durability/authority/recovery/
    recorded_component.rs                    # exact source-backed roots
    recovered_publication.rs                 # private capability custody/minter
  branch/{recorded_basis,canonical_runtime_frame}.rs
  facade/durability.rs                        # narrow owner admission exports
worth-proof/src/recovered_publication.rs      # legal consuming phase shape
worth-runtime-world/src/
  durability/                                # recorded basis/reconciliation
  publication/recovered_adoption.rs           # consumes Native capability
  recovery/product_unpublished/               # original composite custody
worth-store-application/src/world_section/readmission.rs
```

These are current-slice destinations, not empty placeholders. Pure semantic
definitions remain Native-owned; legal phase shape belongs in Proof; clocks,
source, pins and `Drop` stay in Native runtime custody. World owns composite
reconciliation; Store owns filesystem sequencing. No generic recovery bag or
second authority is introduced.

The intended new facade sequence is:

```rust
let (_, mut recovered) = native.durability_recovery()
    .recover_with_authority(execution, plan)?;
let subjects = recovered.associated_publication_subjects()?;
let admitted = recovered.readmit_publication(
    execution, original_subject, recorded_predecessor,
)?;
world.recovery().prepare_recovered_relational_adoption(
    original_custody, admitted,
)?;
```

Names describe the required new contract, not APIs that already exist. Discovery
does not load historical bodies; only owner admission grants the capability.
Continuation/release then follows the existing durable-disposition-before-live
protocol. Recovery itself publishes no occurrence or disposition.

Artifact v3/CHK16 are a clean undeployed-format cutover: older versions,
including CHK15, refuse before effects; no missing-field default,
reinterpretation, in-place repair or automatic reinitialization. Canonical
artifact v3 remains unchanged by the exact-component checkpoint amendment.
Implement shared semantic production/versioning,
then exact source/historical admission, then World adoption, then the real
filesystem journey within existing Phases 3-4.

The decisive proof kills after Native commitment and before occurrence on both
sides of the first Home checkpoint. Cover empty and cold canonical predecessors,
a journal-tail effect, entity/relation correspondence and nontrivial validation
meaning. Compare authored expectations and the original live semantic projection
with recovered output; a same-encoder round trip is insufficient. Observe original
identities/custody, unchanged handler invocation, no recovery-time republication,
continuation or release, and a second reopen. Missing discovery/semantics,
memory backing, stale/foreign sources, wrong roots and altered correspondence or
validation must fail for the intended reason. Unrelated cold bodies stay cold;
excluded live fields must be unavailable. Local Native tests do not qualify the
filesystem World journey or complete either phase.

*Idempotency and receipts (D9)*

- The binding records affinity and association. Resolution reads only durable
  state.
- Each original receipt field has a durable source:

| Receipt field | Durable source |
|---|---|
| Commit receipt/id, version, journal position and artifact digest | Exact source-backed Relational artifact, readmitted and bound to admitted index/root evidence |
| Original durable semantic result: create correspondences, changed records, structural/schema/history/change/aspect and validation meaning | `RelationalCommitSemanticResult` sealed in the original artifact |
| `changed_record_count` | Exact recorded ordered changed records, consistent with recorded change summary |
| Live `CommitResult`, process transaction/proposal identity, snapshot handle/ID, timings/complexity and late interruption | Not reconstructed. Recovered receipts expose distinct `RecoveredRelationalSemanticResult`; live-only fields are unavailable |
| commit evidence: idempotency binding | Query idempotency binding row |
| commit evidence: outbox seal | durable outbox rows in the same commit |
| commit evidence: provider session binding | Not recovered: sessions are process-local and incarnation-fenced (D7). A recovered receipt carries the publishing home incarnation in its place, and nothing resolves through the old session. |
| `basis_descriptor` | Relational artifact (runtime-neutral, Milestone 2 D11) |
| occurrence, old and new product head, attempt | World occurrence |
| outcome identity, emitted effect count | Query idempotency binding row |
| exact output settlement | Query outcome row in the same commit (new field) |
| aftermath causality | Relational causality rows |
| publishing runtime | home identity and incarnation, recorded in the occurrence |

Recovered equality means equality of this explicit durable semantic projection,
not live execution observations. Receipt constructors consume the readmitted
binding, World occurrence/custody and Native owner-admitted inputs; the derived
evidence index grants no constructor authority. The existing capability index,
build guide and Host README must explain this distinction and use the real facade.

- These are deleted: the receipt authority of the evidence store,
  `CommittedReceiptNotRetained`, and the Server retry caches.
- **Changed intent and foreign affinity.** A retry with changed intent refuses
  with `IdempotencyIntentDrift`. A foreign product branch or incarnation
  refuses with the existing affinity refusal, now evaluated against durable
  affinity.
- **Custody and fates.** A retry whose commit is in recovered custody answers
  `ProductUnpublished` with its next actions. An attempt with proven no
  effect executes once on retry. An unresolved attempt answers its unresolved
  posture until recovery decides it.

**Interim refusals.** These are the same as Phases 3–4 above.

**Required tests**

- **The decisive journey**, on the production host and filesystem media:
  1. Perform an ordinary mutation with an idempotency key.
  2. Let it durably complete, then drop its response.
  3. Kill the process without closing it.
  4. Reopen in a fresh process and retry the same request.

  The retry returns the original occurrence and exact branch state. The
  mutation handler's invocation count is unchanged, and there is no duplicate
  domain effect. The journey runs on both sides of the first home checkpoint.
- **Retry refusals:**
  - a changed-intent retry after reopen refuses;
  - a retry on a sibling product branch refuses;
  - a stale capability from before the reopen refuses.
- **Durable data across reopen:**
  - a pre-crash receipt used for history and ancestry lookups succeeds after
    reopen;
  - recovered service then performs another ordinary mutation.
- **Every receipt field** in the table equals the reference after reopen.
- **Crash between the Relational commit and product movement.** Recovery shows
  custody: the Relational effect retained, the product unmoved, and a retry
  answering `ProductUnpublished`. Continuing publishes the original commit
  without re-execution.
- **Release, kill, reopen.** The custody stays released.
- **Caller abandonment and cancellation.** These after an owner effect leave
  custody that survives reopen.
- **Proven no effect.** An attempt with proven no effect executes exactly once
  on retry after reopen.
- **Indeterminate product movement.** It blocks product movement on every
  branch until reopen (D4), and recovery decides it.
- **Concurrent same-key retries** produce exactly one occurrence, and every
  caller receives it.
- **History and identities.** World history pages and ancestry after reopen
  equal the reference. Identities issued after reopen never collide with
  recovered ones.
- **Server.** The retry route survives a restart of the Server process.

**Oracle extension**

- **Reference and script.** A memory-only host running the same program
  executes a seeded host script. The script covers keyed mutations, including
  distinct keys, same-key retries, changed-intent retries and concurrent same-key
  retries; response loss, branch creation, product movement on several
  branches, settlement failure, custody continue and release, caller
  abandonment, and checkpoints. Every seed in the tested range generates.
  Application mutations require an idempotency key by contract (build §5.3), so
  there is no unkeyed route to exercise.
- **Interruption points.** The oracle interrupts at every durable boundary:
  - before commitment;
  - after durable Relational publication;
  - between the owner effect and product movement;
  - after product movement, before the response;
  - during custody dispositions;
  - during checkpoints.
- **Comparison after each reopen.** Graph state, occurrences, custody, product
  heads and idempotency outcomes are compared with the reference prefix. The
  oracle checks the distinct no-effect, committed and unresolved outcomes. It
  never expects a committed receipt for an attempt that never committed.
- **Continuation.** The remaining script then runs to the reference's final
  state.
- **Fault coverage.** A media sweep of every Store operation of a mutation, a
  product movement, a disposition and a checkpoint; real process kills at
  every Store seam; and recovery interruption at every stage, including World
  reconciliation.

**Docs**

- `capability-index.md`, rows "Retry a mutation safely" and "Survive a
  restart".
- `build-an-application.md`: commitment, retry, close, unsupported homes and
  recovery, at the caller's level.
- The host README.
- The bank docs' process-death disclaimers, for Relational-only flows.

**Exit criteria (Phases 3 and 4 together)**

- **Landing 2's Tier-2 run is green on the combined head.** This diff
  reaches nearly every workspace except worth-ui, so the run is close to the
  full matrix by construction.
- **Oracles.** The cumulative oracle runs exhaustively, because the diff
  reaches the home and owner durability.
- **Certification.** The coordinator certifies at the final gate.

---

## Phase 5: Signal and Bridge authored state; mixed publication

**Goal.** Signal and Bridge recover their authored state through their own
readmission. Mixed Relational/Signal operations recover at the true owner
boundary.

**Requirements**

*Authored and derived state*

- Signal separates authored state from derived and scheduling state:
  - **Authored:** installed definitions, runtime-published definitions,
    topology and branch structure.
  - **Derived:** values, dirty and aspect versions, pending causes, traces,
    telemetry, and the resource and temporal runtime.
- The authored image is a Signal-owned artifact with a versioned, checksummed
  byte codec, not serde snapshots.

*Signal port and artifacts*

- Signal defines `SignalDurableMedium`, a wasm-safe trait with the D2
  answers.
- `SignalDefinitionTransitionArtifact` records a change to authored
  definitions. It includes Bridge's conditional definitions and lowering
  claims as a Bridge-defined section.
- `SignalBranchAdvanceArtifact` records the branch, generation, attempt
  association (D5) and definition-transition reference. It never carries
  derived values.

*Identities*

- Signal's durable identities become home-scoped (D6): owner, branch, cell
  incarnation, generation, snapshot, admission and conditional contract
  occurrence.
- Descriptor readmission accepts a recovered owner and refuses a foreign one.

*Readmission*

- Multi-branch owner readmission installs program definitions and applies
  durable definition transitions in order. It restores branch structure and
  generation frontiers for every World branch, and leaves derived state cold,
  recomputed on demand.
- A derived value never stands in for a lost definition.

*Mixed publication*

- World occurrences carry a Signal component-evidence tag.
- A crash between the Relational and Signal effects recovers custody naming
  exactly the acting owners, with the product unmoved.

*Bridge*

- Conditional definitions recover through the Signal transition section.
- Bridge's lowering registry and retention ledger are derived, rebuilt at
  installation.
- Writeback replay records follow their audit classification.
- Owned-async custody belongs to Phase 7.

*Build identity*

- Signal derived state recorded under one build is stale under another. It is
  recomputed, never reused unconditionally.

**Interim refusals lifted.** Signal-touching and mixed operations, and
conditional-definition publication.

**Still refused until later phases.** Conditional and temporal operations,
workflow runs, managed and owned-async work, dispatch, corrections and
program adoption.

**Required tests**

- **Signal readmission refuses** a damaged frame, a foreign owner, an unknown
  version and a definition-basis mismatch.
- **Reopen with several branches.** A home with several branches, published
  definitions and conditional definitions reopens to authored state equal to
  the reference.
- **Derived values.** They recompute to the reference values on first demand,
  and none is read from durable bytes.
- **Mixed operation killed between owner effects.** It recovers custody with
  exactly the acting owners. Continuing completes it without re-executing
  either effect.
- **wasm.** `worth-signal-wasm` checks for wasm32 with the port present.

**Oracle extension.** The host script adds Signal-only and mixed operations,
conditional-definition publication, and Signal forks with branch creation. It
interrupts at every Store seam of each Signal publication, and between owner
effects.

**Docs**

- The Signal README durability section and the Bridge README.
- The capability index.
- The derived-recovery section of `build-an-application.md`.

**Exit criteria**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 6: Programs, adoption and build identity

**Goal.** Program lifecycle and adoption obligations survive restart, and a
changed build identity is an oracle dimension rather than a single case.

**Requirements**

- **Support lifecycle.** Program support status (Active, Retiring, Retired)
  and its custody counts become durable obligations in `query.obligations`.
  Counts derive from durable obligations and are never reset to Active at
  open.
- **Program activation.** Activation is verified on every branch at open, not
  only on main. Each occurrence's program revision (recorded since Phase 4) is
  checked against the branch's activation history.
- **Adoption custody.** These become durable custody:
  - `WorthQueryBranchAdoptionRecovery`, with its support custody and
    unpublished recovery;
  - `WorthQueryStoppedBranchSetAdoption`.

  Both resume through `recover_branch_adoption` and the branch-set resume
  after reopen. They are no longer caller-held values whose loss forgets the
  obligation.
- **Build identity.**
  - Authored source and references survive a changed build.
  - Incompatible derived outputs are marked stale under their reuse
    contracts. They are recovered or regenerated through the normal demand,
    dependency, invalidation and settlement owners, never eagerly at open.
  - Same-build reuse keeps its equivalence and ownership checks.
- **Derived caches.** Discarding derived caches or materializations loses no
  authored truth, history or custody.

**Interim refusals lifted.** Program adoption and support retirement.

**Required tests**

- **Producer directory build identity.** Restore
  `directory_changed_build_reuses_original_output_until_new_implementation_is_demanded`
  (restore from `5358b31d3083dc88eea961af716e948f759eac96:workspaces/worth-query/crates/worth-query-court-decl-host-package-archive-replay-application-graph/tests/worth_query_court_decl_host_package_archive_replay_application_graph/certification/application_graph/producer_predicate_checkpoint/changed_build.rs`): produce under A, reuse the
  original accepted output under A with its equivalence and ownership checks,
  reopen under B without producer execution, then regenerate only on demand
  while keeping the original entities. The filesystem court is moved out of
  the Phase 3+4 tree until the Phase 6 authored Signal owner is available;
  its uncommitted restoration has been removed without gating or weakening it.
- **Retirement survives restart.** Retire support, kill, reopen: the status is
  still Retired, and interpretation refuses where the retirement requires it.
- **Adoption resumes.** Adoption interrupted at each stage across a kill
  resumes to the reference result, with no repeated effect.
- **Unrostered activation.** A non-main branch whose activation names an
  unrostered revision refuses at open.
- **Changed build.**
  - Stale outputs are reported stale.
  - Outputs regenerate only on demand, and keep their original entities.
  - Required-output obligations keep their execution and failure semantics.
- **Discarded derived state.** Discard derived state (D14 door), reopen:
  authored truth and custody are exact.

**Oracle extension.**

- From this phase on, two dimensions apply to every generated sequence:
  - build identity changes between reopens;
  - derived material is discarded between reopens.
- Adoption and retirement operations join the host script.

**Docs.**

- The program adoption guide.
- The "Build identity" sections of `build-an-application.md`.
- The capability index.

**Exit criteria.**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 7: Workflows, conditional operations and long-running work

**Goal.** Workflows, conditional and temporal operations, managed recovery
and owned-async work resume after process death, or terminate exactly where
their existing lifecycle contracts define a terminal posture.

**Requirements**

- **Workflows.**
  - Workflow definitions, instances, transitions, approvals and coverage are
    graph records, and recover with Relational.
  - Progress and replay retention are derived, rebuilt by reconstruction.
  - A workflow interrupted mid-run resumes from durable state under its
    original keys. Its required approvals remain required.
- **Workflow migration on transition.** A home holding workflow records
  transitions (D16) under the same workflow dispositions that live program
  adoption applies (`program_adoption/preparation/dispositions.rs`).
  `checkpoint_transition/selection.rs` refuses workflow records today; this
  phase deletes that refusal.
- **Managed cleanup.** Cleanup required after a failed commit becomes a
  durable obligation, recording the run identity and failure kind. Its
  process resources are reacquired through fresh authority. It is persisted
  even when clean close would have refused.
- **Conditional decisions.**
  - These retained decisions become durable obligations: CleanupRequired,
    Retryable, CommitRetryable and ProductStale.
  - Eligible, Deferred and Suppressed wakes are process-local. They are
    re-derived from durable state and host-fed time after reopen.
  - Clock leases are reacquired.
- **Temporal operations.** The temporal idempotency key derives only from
  durable, home-scoped identities, never from per-process runtime binding
  identity. A temporal intent re-executed after restart resolves to its
  original occurrence.
- **Managed runs and owned-async custody (Bridge and Query).**
  - Each yield, suspension and mandatory in-flight request is resumed from a
    durable record.
  - Termination at reopen is allowed only where the existing lifecycle
    contract defines a terminal posture for that kind. The termination is
    then typed and observable.
  - Loss of memory is never success.
- **Resources.** Resources are process-local and are reacquired through fresh
  authority.

**Interim refusals lifted.** Workflow runs, conditional and temporal
operations, and managed and owned-async work.

**Required tests**

- **Workflow killed mid-run.** It reopens, resumes under its original keys and
  reaches the reference terminal state. Its approvals survive.
- **Workflow migration.** A transition of a home holding workflows applies the
  migration dispositions. A kill at each transition seam resumes correctly.
- **Managed cleanup.** Kill with cleanup outstanding. The reopened home shows
  the obligation. Discharge it, and close then succeeds.
- **Temporal operation.** Kill it between its commit and its response.
  Re-execution after reopen yields one occurrence.
- **Managed-run suspensions.** Each kind either resumes, or terminates with
  the typed posture its contract defines.
- **Owned-async requests.** Request custody survives a kill and resumes or
  terminates per its contract.

**Oracle extension.** The host script adds:

- workflow runs stopped mid-progress;
- conditional and temporal operations;
- managed cleanup;
- owned-async requests;
- workflow-bearing transitions.

It interrupts at every durable boundary, including between workflow steps.

**Docs.**

- The workflow and conditional-operation guides.
- The capability index rows for managed and long-running work.

**Exit criteria.**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 8: External dispatch and custody

**Goal.** External work keeps its custody across process death. Unknown
external effects stay unknown until their owner resolves them, and completed
external effects are never repeated.

**Requirements**

- **Outbox contract on reopen.** The outbox's inbound contract is rebound from
  the installed program on reopen. It is no longer `None` after restore.
  Outbox rows, transport causal digests and runtime source commitments carry
  no process-local identity. The audit's identity table for Phase 8 lists
  them.
- **Dispatch custody.** A dispatch attempt is recorded durably in
  `query.dispatch` before the external call. Its outstanding custody,
  correlation and idempotency association survive.
- **Crash after dispatch.** If the process dies after dispatch and before the
  response is recorded, the external posture is indeterminate. That posture
  survives reopen until the owner resolves it.
- **Proven no effect.** Proven no effect (for example `PayloadRejected`) is a
  first-class durable outcome. It is distinct from unresolved, compensation
  and completed.
- **Inbound custody and deduplication** are durable: message identity,
  terminal, pending and expiry.
  - A duplicate delivery after reopen is recognized as a duplicate.
  - Terminal acknowledgement settlement is durable.
  - An authentic callback for a pre-crash dispatch completes it, rather than
    answering "unknown correlation".
- **Recovery handles.** Their claim status is a durable obligation. Recovery
  progression (compensate, reconcile, redispatch, safe retry, resolve, expiry)
  operates on recovered custody.
- **Rebuilt indexes.** The inbound terminal index and the evidence store's
  correlation index are rebuilt from durable World history and Relational
  rows.
- **Deleted postures.** The `StoreCapabilityRequired` postures are deleted
  from `WorthQueryDispatchOutboxDurabilityPosture` and
  `WorthQueryRecoveryDurabilityPosture`.
- **No exactly-once claim.** There is no claim of remote exactly-once beyond
  the external owner's admitted contract.

**Interim refusals lifted.** Dispatch and inbound completion.

**Required tests**

- **Bank external rail under application kills.** With the bank external rail
  as the external party, these cases pass while the application process is
  killed:
  - exactly-once completion;
  - lost response;
  - duplicate acknowledgement;
  - completion after timeout;
  - disappearance mid-dispatch;
  - semantic rejection, as proven no effect.
- **Crash between dispatch and response recording.** Reopen shows the
  indeterminate posture. Reconcile resolves it without redispatching a
  completed effect.
- **Duplicate inbound delivery across a reopen** is deduplicated.
- **Authentic callback after reopen** completes the original dispatch.

**Oracle extension.** The host script adds dispatching operations and inbound
completions against a deterministic in-test external party. It interrupts at
every durable boundary, both before and after the external call, and before
inbound results are recorded.

**Docs.**

- The external work and recovery guides.
- The process-death disclaimers in `http-owning-money.md` and
  `public-consumer-contract.md`, rewritten to describe the supported behavior.
- The capability index.

**Exit criteria.**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 9: Correction and original-owner advancement

**Goal.** 9.18 corrections and their retained material survive restart, and
the original owner keeps editing after reopen.

**Requirements**

- **Lineage readmission.** Correction occurrences, alternatives, causal links
  and lineage are Relational causality rows. They readmit against durable
  World identities, which they record as home-scoped identity since Phase 4.
- **Retained material.** Retained governed input, pre-images and original
  authority become durable in `query.corrections`, under their retention and
  disclosure contracts. Each is optional or an obligation, as its contract
  says. Expired optional material answers typed unavailability (D10). Recovery
  never guesses an inverse or reinterprets retained bytes.
- **Custody.**
  - Correction idempotency resolves through Phase 4's durable idempotency.
  - Required-output custody for corrections, meaning the pending publication
    registry's correction entries, is a durable obligation.
  - Authored correction and required derived settlement stay distinct,
    including recovery when only one has completed.
- **Divergent branches.** Correction meaning and the selected lineage are
  preserved through reopen. Recovery never consumes or substitutes an
  alternative from another branch.
- **Original-owner advancement contract.**
  - **The original owner** is the retained native operation owner, in the
    platform's existing sense: the continuation that fresh authenticated
    commands advance and close. Examples are the bank's
    `BankHttpMutationContinuation`, `capability-index.md` "advance and close
    each original owner", and `build-an-application.md`'s original-owner
    guidance. Today it is a caller-held process value.
  - **Durable custody.** The owner's continuation becomes durable custody in
    `query.obligations`. It records:
    - its original operation identity and its occurrence;
    - its native ownership material, meaning the aggregate identities and
      incident relation facts it governs;
    - its lifecycle position.

    Phase 7 supplies the managed and owned-async custody it builds on.
  - **After reopen.** The custody readmits, and the host issues a fresh owner
    value, bound to the new incarnation (D7), for the same original identity.
    Fresh authenticated commands advance and close it exactly as before the
    crash. The old process's value refuses with `StaleIncarnation`.
  - **A dependency edit followed by advancement.** Advancement continues the
    same original owner. Aggregate identities stay continuous under the
    existing keeper, split and merge laws (`aggregate_identity/transition.rs`):
    membership may change exactly as those laws allow, and no identity is
    re-minted outside them.
  - **What reconstruction must not do.** It never replaces the owner, and it
    never creates a new authoritative object to make the edit succeed. No new
    lineage kind is added.
  - **Missing material.** If the required ownership material is unavailable,
    advancement refuses with `OwnershipMaterialUnavailable`.

**Interim refusals lifted.** Corrections.

**Required tests**

- **Correct, kill, reopen.** Correction history, alternatives and lineage equal
  the reference.
- **Reapply after reopen** selects the recovered alternative.
- **Expired optional pre-image** answers typed unavailability.
- **Divergent branch.** A correction on a divergent branch, killed and
  reopened, keeps its branch's lineage.
- **Split completion.** A correction killed with the authored correction
  complete but derived settlement pending recovers that exact split.
- **Advancement after reopen.** An original owner is held across a kill and
  reopen. A fresh authenticated command, then a dependency edit, then
  advancement and close of the original owner, all match the reference:
  aggregate identities, membership and incident relations are equal, and no
  identity is re-minted outside the keeper, split and merge laws.
- **Stale owner value.** The pre-crash owner value refuses with
  `StaleIncarnation`.
- **Missing ownership material** refuses with
  `OwnershipMaterialUnavailable`.

**Oracle extension.** The host script adds corrections, reapplies and
divergent-branch corrections, and a dependency edit plus original-owner
advancement after a reopen.

**Docs.**

- The correction guide.
- The capability index rows.

**Exit criteria.**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 10: Public cutover and milestone certification

**Goal.** One supported durable path, honest public guidance, and
certification of the whole milestone.

**Requirements**

- **Retire the interim checkpoint path (D13).** Delete it entirely:
  - capture;
  - byte transport;
  - the `*_from_checkpoint*` constructors;
  - `repair_to_checkpoint*`;
  - Relational's embedded-checkpoint placement.

  Its callers move to the open call.
- **Delete `UnsupportedDurableFamily`.** Every family in the roadmap's
  Milestone 3 requirements is supported. No family within that set remains
  refused.
- **Re-audit every Phase 1 family against the code.** Confirm three things:
  - no competing durable publication or recovery path remains;
  - no durable row records a process-local identity;
  - no capability claim overstates what reopening resumes.
- **Update every ordinary guide** with supported restart behavior, failures
  and valid next actions:
  - the application, workflow, external recovery and correction guides;
  - the host README;
  - the capability index, including "Known gaps".
- **Executable examples** use the public host and its real owners:
  `observable_workflow`, `authored_workflow`, `shared_execution` and the bank
  world.

**Required tests**

- **The full journey,** on the production host and filesystem media, in one
  run:
  1. mixed-owner publication;
  2. a deferred settlement;
  3. a workflow mid-run;
  4. an ambiguous external completion;
  5. a correction on a divergent branch;
  6. kill and reopen;
  7. recovery of original occurrences and custody, without repeated effects;
  8. a dependency edit, then advancing the original owner.
- **No caller of a deleted path remains.** Boundary-check enforces this.

**Oracle.** The cumulative oracle runs once, exhaustively.

- Generated sequences mix every family, interrupt at every durable boundary,
  and reopen in a fresh process.
- Build-identity change and discarded derived state are dimensions across the
  whole run.
- Recovery interruption runs at every stage.

**Completion.** Milestone 3 is complete when:

1. **One durable home.** The host opens one durable home. No
   application-owned checkpoint transport, in-memory receipt authority or
   parallel retry cache remains.
2. **Every audited family is accounted for.** Each is either durable, or
   process-local with reacquisition through fresh authority that is
   documented and tested.
3. **Journeys pass.** The decisive journey (Phase 4) and the full journey
   (Phase 10) pass on both sides of the first home checkpoint.
4. **Oracle passes.** The cumulative oracle's generated sequences pass across
   build-identity change and discarded derived state.
5. **Every lane is green.**
   - Not merely free of new failures. The lanes are Relational, World, Signal
     (including wasm), Bridge, Server, Query (library, integration, examples,
     doc and fixture gates), bank world and Store, plus boundary-check,
     agent-context, clippy, formatting and line caps.
   - Failures that existed before the milestone are fixed honestly.
6. **Certified.** An independent reviewer certifies the milestone against this
   specification and the roadmap.

## Out of scope: worth-ui

`workspaces/worth-ui` is not ready, so it is left out of this milestone
entirely. No phase migrates `worth-ui-query-binding`, and no landing gate
includes the UI lanes: the `ui`, `check-ui-workspace` and `clippy-ui-binding`
steps of `query_gate.py`. The binding stops compiling against the host once
Phase 3 replaces the `in_memory*` constructors. When worth-ui is picked up,
its own work adopts the open call; this milestone adds no shim for it.

## Verification tiers (every phase)

Verification cost is the milestone's main expense. It is spent where a diff
can actually break something.

| Tier | When | What runs |
|---|---|---|
| 1 | Each slice | `cargo check` on touched crates, plus the tests the slice adds or changes, selected by name. |
| 2 | Each phase merge into the integration branch, and each landing | Only the lanes the diff can reach, computed mechanically by `affected_lanes.py` in the orchestration tooling. A lane is reached when the `cargo metadata` resolution of its workspace includes a changed package, or when the diff changes the lane's non-crate inputs: docs for the doc gates, scripts and workflow steps for their lanes. The constitution check (boundary-check and agent-context), line caps in diff mode, formatting of touched workspaces, and lockfile alignment when Cargo files change are always included. Tool self-tests run only when the tool changed. |
| 3 | Landing 3 (Phase 10), and idle background runs on master | The full lane matrix and the exhaustive cumulative oracle. |

- **Crash oracle at Tier 2.** The oracle runs exhaustively when the diff
  reaches its subject: Store publication, recovery, root or journal code, the
  home, or any owner's durable artifacts, readmission or recovery. Otherwise it
  runs its focused seed range plus every case the diff adds or changes.
  Failing seeds reproduce deterministically.
- **Evidence.** Logs record the command, HEAD, a clean-tree flag, times and
  the exit code. A lane with a green log on the same HEAD is reused, never
  rerun. Reviewers verify evidence and probe; they do not rerun suites.
- **Every lane green.** This is enforced at Tier 3. A Tier-3 failure is fixed
  before the milestone closes, whichever phase caused it.
- **Merging to master.** Each landing merges the current master in, runs the
  Tier-2 lanes that merge reaches, then fast-forwards master. Nothing is ever
  force-pushed.
