# Home close and owner retirement

This P-B slice implements Milestone 3 D11, D12's close trigger and D17's
consuming custody under the 2026-11-02 coordinator handler-retirement ruling.
The authoritative specification remains `milestone-3.md`; the owner details
remain in `milestone-3-home-lifecycle.md`.

## Boundary review

The adversary holds an original invocation, observation, provider or source
alias while close is interrupted. A refusal must retain the original Home,
and successful close must release physical custody without destroying retained
supplier payloads or silently completing work.

World owns installed-call admission and seal. Query owns handler invocation,
the serving graph, provider inventories and registered-source custody. Bridge
owns its source aliases, subscriptions, observations and assemblies. Native
owns its runtime aliases, read/publication fences and physical-member custody.
Store owns checkpoint selection, the continuously held mutation lease and
physical shutdown. Host orchestrates these owners and retains refused custody.
Descriptions, reference counts and handler completion prove none of another
owner's obligations.

The starting source (`6750f1376f`) rejects every supplier handler before its
hooks; registered-source retirement unconditionally refuses. Query therefore
cannot reach the independent lower Native/physical close gates. The original
Store composition still drops Bridge and Native allocations during teardown;
retaining a whole operational Query would instead keep Native aliases alive.
Both are obligations to cut over, not alternate ways to claim Closed.

## Implementation order and topology

1. Apply the coordinator ruling in the handler trait and original registry
   retirement. Preserve unique original allocations, explicit hook refusal,
   compiled-body hooks and the sealed Host-only post-World handoff.
2. Add consuming source/serving/provider retirement at their existing Query
   owners. Delegate each capability to its original owner port, retain exact
   partial progress, and retain inert payloads until explicit reclaim.
3. Complete missing Bridge/Native/Store retirement handoffs at those owners.
   Capture and settle the Home checkpoint while Native serves; release capture
   pins before Native admission. Preserve external readers' genuine vetoes.
4. Integrate Host close/retry and failure-close, preserving pre-seal resume
   and explicit post-seal recovery custody through deadline/cancellation.
5. Extend behavior-named close courts and qualify all six unchanged Store
   native-artifact acceptance courts. Run the Host controls directly with
   RUST_MIN_STACK absent, then affected normal targets and completion gates.

Source retirement stays under `product_world/source_owner/`; serving graph
retirement under `primary_graph/root/`; provider retirement under
`primary_graph/provider/`; Host orchestration under `application_home/close/`.
Native and Bridge capabilities are retired at their owning runtime modules.
No generic cleanup registry, compatibility path or fake successful refusal
translation is permitted. New public owner ports use their existing facades.

## Requirement map

| Requirement | Acceptance evidence |
|---|---|
| D11 ordinary handler custody and retained payload | `default_supplier_hooks_retire_invocation_without_destroying_shared_payload`, `ordinary_supplier_retirement_retains_payload_with_an_exhausted_memory_budget`, and Host `handler_retirement` courts; coordinator ruling retires the old ordinary-supplier Unsupported assertion |
| D11 original slot/handler aliases refuse Busy | `original_erased_slot_and_handler_aliases_preserve_the_same_handler_and_payload` and `erased_slot_and_active_handler_aliases_refuse_while_the_same_query_serves`; Host `program_alias_refusal_resumes_the_same_query_and_ordinary_handler_retires` |
| D11 active call refuses before seal | `active_installed_call_refuses_before_world_seal_and_resumes_the_same_home` |
| D11 registered source, serving graph and provider custody | Six unchanged `query_readmission::native_artifacts` acceptance courts named in the P-B brief, plus owner retirement controls |
| D11 original request deadlines and refusal custody | `expired_host_close_preserves_the_same_serving_home_and_a_live_retry_physically_closes`, with the original owners retained |
| D11/D12 failed close checkpoint does not prevent Closed | `host_checkpoint_memory_failure_is_reported_while_original_retired_owners_remain_physically_closed` |
| D11 kill without close preserves acknowledged results | `killed_host_without_close_reopens_its_acknowledged_value_and_original_idempotency_record`; exact P-A refusal is retained if blocked |
| D11 cold/materialized Native and independent inert metadata | `cold_and_materialized_original_sources_physically_close_while_metadata_and_retired_payloads_live` |
| D11 authentic source-capable fork veto and retry | `an_original_source_capable_fork_refuses_before_native_seal_and_same_home_retry_closes` |
| D17 original member callback custody | `completed_member_callback_survives_a_busy_original_return_slot_without_replay`, `cancelled_member_handoff_returns_original_custody_and_a_live_request_continues_it` |
| D17 prepaid close storage | `prepaid_close_storage_retains_original_backings_through_exhaustion_and_public_moveout`, `prepaid_slot_denial_targets_actual_backing_and_preserves_cancellation` |
| D17 no service after seal; terminal custody consumed once | Prepared/retiring/retired owner types and sealed `query-home-retirement` caller |

This map records intended evidence, not completed verification. Lane logs are
retained under `target/lane-evidence/`. Opening, durability formats, transitions,
discard, idempotency, World call-admission and Server changes remain with their
other owners. Any required edit there is reported before touching it.

## Retired assertion

The coordinator's 2026-11-02 handler-retirement ruling retires
`opaque_supplier_completion_override_cannot_admit_or_advance_retirement`'s
blanket supplier-refusal claim and the ordinary-supplier refusal portion of
`original_program_alias_and_opaque_handler_refusals_resume_the_same_directory_query`.
The replacement controls require ordinary supplier retirement, preserved
original payloads, actual alias Busy and explicit handler-declared refusal.
No acceptance court is removed, ignored or allowed to accept an arbitrary error.

## Native and Store retirement ownership

Native close consumes the authentic outer runtime Arc (or the still-unshared
pre-service owner), preserving its inner state Arc and dormant Weak services.
It validates original live and retired roots, both mirror directories, nested
symbol tables, empty lineage registrations, history-prefix registrations,
deferred-index detached history and canonical source placements. An original
medium binding is mandatory. Its successful consuming extraction seals the
phase; subsequent Busy/stop/refusal retains the same owners and callbacks.

A callback task and its returned observation remain in the current owner cursor
until the exact original slot accepts their nonwaiting return. Retry returns the
cached observation before another callback. Retired original payloads remain
in those owner slots. Native-owned region leases retire once; independent
resident metadata leases keep their normal clone/Drop accounting.

The retired-root owner now uses its own RwLock/BTreeMap. Ordinary sweeps copy
one entry's generation/root/bindings, inspect lane liveness outside the primary
gate, and revalidate generation plus binding identity before pruning. This adds
real global write serialization. Close uses successor lookup over original
entries and a fixed radix path, without invoking the ordinary reclaimer.

Home's existing pre-descriptor close storage prepays the Native close frame and
possible reversible outer allocation from the original opening request. A
refusal returns the same vacant preparation; no allocation is admitted at close
merely because a checkpoint exhausted memory. Each admitted checkpoint directory
also prepays its own finite retirement Box before its physical custody is issued.
These are real extra cold allocations and inline state, not free bookkeeping.
The fixed Home result Box and Native frame Box are distinct responsibilities;
each allocation dies before its own reservation is refunded.

Store retires its original installed selected read through the existing physical
read-membership port. A shared external source keeps its own protection and
prevents physical close. A unique local source retains its original data and
supplier payload while releasing only matching physical capabilities. Actual
physical Closed moves the retired Native, provider, Bridge and Home allocations
into the existing retired Home result; explicit reclaim destroys them later.

The P-B brief authorizes these missing in-memory Native retirement ports.
Persisted formats and reopen reconstruction remain with P-A. No opening or
format grammar is changed by this slice.

## Verification scope

The six unchanged native-artifact acceptance assertions remain the requested
Store controls. Ordinary fixture configuration remains its original contacts Arc;
a separate authored fixture supplies the explicit handler refusal. New Host
courts cover expired close/resume, a real bounded-memory checkpoint failure and
a killed acknowledged writer. The kill/reopen court must report an exact P-A
RootLinkage or CompleteNativeRequired refusal if that prerequisite remains red.

This map records intended evidence until the focused courts, affected normal
targets, default-stack direct controls and completion gates qualify the current
committed source. Earlier compilation logs qualify only their earlier dirty
source, and do not certify physical Closed. Independent review remains required.

## Causal test and boundary corrections

The lifecycle plan's Native/Store source-close decision ("Actual physical slots
and retained callback work") requires original source and retirement state to
stay under the existing envelope lock. It deliberately retires the old
`memory_frame_layout_preserves_scalar_tuple_observations` size-equality
assertions against `OldBacking`/`OldCell`: journal backing now occupies 80 rather
than 32 bytes on the qualified Windows target. The replacement keeps all actual
payload-length, digest and scalar-observation assertions. This is an explicit
cold-storage cost, not a claim of unchanged layout or a wire-format change.

Recovery's whole-state comparison now recognizes the retired-root map inside
its new RwLock. It still compares every entry and field and preserves native
address aliasing; only independently issued process addresses are canonicalized.
The field-text controls preserve original-request custody: the one-byte request
now refuses at the genuine symbol-table admission, the retained-backing control
funds its larger actual table, and SerialMemoryBudget custody is reported as
managed, as `StoredSymbolText::admit` already requires.

The private source-member and installed-handler alias courts now live in the
existing source-protection and handler-retirement unit courts. Their registered
runtime construction frontier is unchanged; no source-court baseline exemption
was added. The sole new control caller retains original cfg(test)
lock-observation metadata and invokes no control.

D12's mandatory coupled checkpoint retires the World-owned courts' old
`WorldSectionRequired` expectations. They now require a successful coupled
checkpoint and retain their original external-read and delivery fences. Bare
Native-only reopened Homes still assert `WorldSectionRequired`.

`close_deadline_retains_serving_native_until_its_original_checkpoint_settles`
used a worldless checkpoint control that cannot admit D12 work. Its replacement
`close_deadline_retains_the_original_coupled_checkpoint_and_retry_closes_the_same_home`
drives the actual Home-issued Native/World operation and requires explicit
post-seal recovery custody, the same continuous lease, and successful retry.
The neighboring input-ticket and overlapping-driver courts use that same
issued operation; observer hooks forward through its real checkpoint journey.

The detached-read admission added for source close is local to that source-close
phase. Memory retirement retains its existing incremental read/cache sequencing;
it does not inherit source-close's stronger initial-capture veto.

The handler retirement cursor retains the original static binding rather than
allocating a copied key after checkpoint. The court
`ordinary_supplier_retirement_retains_payload_with_an_exhausted_memory_budget`
traces D11 release after checkpoint memory exhaustion and preserves payload
until the same registry is reclaimed.

The source-close read-custody gate retires the original Native visibility-cache
observations before classifying remaining detached custody and before source
seal. Those cache pins belong to Native; treating them as external is a design
error that makes every queried Home unclosable. Once any cache entry advances,
service restoration is refused and the same closing owner retains its original
cache payloads. A genuine retained Complete capture still vetoes source seal;
its release permits that same closing owner to finish. The behavior court is
`owned_visibility_cache_retires_before_external_capture_recheck_and_source_seal`.

Provider completed-commit evidence also owns performed World publication facts.
`session_commit/evidence.rs::by_commit` retains `WorthQueryPrimaryGraphCommittedApplication`,
whose `committed_product_publication` reaches `product_world/receipt.rs`'s
original consumed World delivery. Its before/after World snapshots retain real
Native admitted bases. Source/runtime alias retirement alone cannot release
those pins. A consuming World receipt retirement port and its Query receipt
handoff are required; dropping evidence or manufacturing retired identities is
not an acceptable close implementation. Ownership of the receipt handoff has
been referred to the coordinator because its files lie outside the listed
P-B source-owner paths. The current Host mutation-close courts remain red.

The ordinary-stack close correction places each graph's actual source task in
preflight-funded cold storage and prepays the original whole-Query retirement
Box. The prepared and refused carriers carry those boxes, instead of embedding
the future retirement bodies in opening and close results. Completion erases
that same Box without allocating a replacement. Projection and granular graph
copies also run their real source preflight before World seal. Payload backing
is destroyed before its original reservation refunds.

Post-seal Query retirement consults the original request without opening a new
execution scope per step. A fresh scope itself needs framework memory, which
cannot be required after a failed checkpoint exhausts that payer. The handler
owner contains supplier advancement panics while keeping its original payload
and cursor; `handler_retirement_and_panic_payload_destructor_preserve_original_custody_for_retry`
traces that refusal. The memory court exhausts the same Serial payer only after
preflight, through its own reservation port, and advances without fresh memory.

Store's retiring Bridge composition also uses a Box funded and allocated by its
original owner before World seal. The same Box crosses Serving, Retiring and
Retired custody. This keeps the complete Bridge body out of public close
results and the opening failures that retain them; it introduces no post-seal
allocation or payload destruction.

`fresh_read_host_physically_closes_with_original_retired_allocations_alive`
separately qualifies D11's actual physical lease release while the original
retired Query, handler, Native and Bridge allocations remain alive. It preserves
all mutation-close and recovery controls; it does not certify their blocked
receipt or reopen successors.

The physical-release court independently qualifies the real filesystem media
both before close (OwnershipContended) and after Closed (exclusive lease
acquired), while all retired allocations remain alive. It uses that existing
owner admission port directly; it does not require a second Native/World reopen
in order to prove physical custody release. The original recover_home probe
reached Closed, then overflowed inside recovery on the ordinary stack; the
captured stack is close-stack-debug-publics.log. The kill/reopen court retains
its full recovery requirement.

The alias court separates the original mutation phase from its before/after
close carriers, preserving every operation and assertion without another thread
or stack allowance. Its original monolithic body overflowed in Native commit
admission before reaching its final close; close-handler-stack-debug.log retains
the actual stack. The phase split requires direct ordinary-stack qualification.

Store source-close preparation now funds the exact physical-read retirement
backing before descriptor effects, alongside Native's existing preparation.
PreparedRelationalSectionClose carries both original vacant slots. Reversible
refusals return both slots; source retirement and its inert result retain the
same cold Box. No physical read task or retained reader body is embedded in
Home opening/close result carriers, and no replacement backing is allocated
when a failed checkpoint leaves its payer exhausted.

## Qualification and outstanding ownership

At 7b29f21c24, fresh read-only Host close passes both the owned court and direct
ordinary-stack run. The independent filesystem admission refuses before close
and acquires its original mutation lease after close while all retired owners
remain alive. Prepaid close-storage courts and Native-owned-cache versus real
external-capture courts each pass 2/0/0.

The six unchanged Store acceptance courts remain red: five reach
CompleteNativeRequired during readmission; the original cancellation/memory
court reaches ReadCustodyRetained on mutation close. The Host deadline and
checkpoint-memory controls also reach that exact Native refusal. None is
accepted as success. Kill-without-close acknowledges commit 4, then ordinary
reopen overflows; its configured-stack run reaches RootLinkage. Those reopening
paths and durable formats remain with P-A.

The latest alias court separates opening/alias refusal/resume, mutation/read,
and final close into actual sequential phases on the same libtest thread. At
ccac0ab the ordinary stack completes the first two phases and reaches the
remaining ReadCustodyRetained refusal on close; it no longer overflows. Store's
deadline control likewise resumes and reads the same Home before that refusal.
No larger stack, replacement owner, suppressed assertion or new thread is used.

Independent review of a78d14a967 returned REJECT: (1) mutated Home close retains
Native read custody; (2) the alias and deadline controls overflow the ordinary
stack. Finding 2 is addressed by original prepaid physical-source storage and
sequential court phases, with direct logs at 7b29f21c24 and ccac0ab. Finding 1
requires the referred Query/World receipt-owner retirement handoff. The
coordinator's ownership question remains unanswered; this is not a completed
P-B or Phase 3 landing.

Every executed command and test count, including failed intermediate builds,
is catalogued in target/lane-evidence/close-evidence-catalog.md and its JSON
counterpart. Each full log records its own HEAD, dirty state, times and exit.
Older normal evidence is retained without certifying a later source: Native
normal recorded 2301/125/31, Bridge 1121/0/0; six of those Native failures were
subsequently corrected and targeted controls passed. The remaining 119 are
unqualified, with exact test names/stdout in native-normal-unqualified.json.
Query execution normal recorded 2208/0/1; Host/installation/testkit unit targets
367/0/0; Store application/section units 36/0/0; Query Host courts 340/12/2.
These recorded failures prohibit a full green-lane claim.

The remaining normal courts exposed another P-B design flaw: the source-close
preflight refused Native's retained immutable indeterminate outcome as though
it were running work. Its original binding and outcome already remain in the
retired Native allocation; the original physical dispatcher owns effect
settlement and inspection. Stopped-business admission still fences active
Native work. Removing the blanket UnresolvedPublication veto permits the
physical owner to return its actual InspectionRequired custody without losing
the original uncertain outcome. The existing failed-completion court remains
unchanged and requires that precise physical inspection result.

The two World close helpers in world_durable_release and
world_occurrence_readmission also still asserted that coupled World checkpointing
was unsupported. D12 retires that expectation; their close branches now require
Ok checkpoint results. The bare Native cleanup branch retains its original
WorldSectionRequired assertion. All reopen and original custody assertions stay.

The handler panic retry court now also makes the supplier panic payload's own
destructor panic. Post-seal retirement uses the existing Execution panic
boundary's second-payload containment pattern, so that destructor cannot unwind
the original Query owner. Original handler payload identity, drop count and
unadvanced cursor assertions remain and the same request retries successfully.


## Coordinator-authorized publication receipt retirement (2026-11-02 13:50)

The coordinator grants Query product_world/receipt.rs, committed_publication.rs,
and the missing World ConsumedCompositePublication owner port. This supersedes
the pending ownership question above. The change is in-memory custody only:
provider evidence indexes and original Native/durable rows remain; no
provider/idempotency or Server path changes are authorized.

Boundary review: the provider owns completed applications; their receipt Arc
is shared by live callers. Arc::get_mut on that exact custody is the pre-seal
external-reader fence. Unconsumed one-shot successor/client/output observations
or fresh history delivery remain pending work and refuse without mutation.
World owns its consumed delivery and canonical envelope. Its before/after and
expected snapshots own shared commits; commits own composite bases; those own
actual Native admitted basis shares. Native alone disarms their original
retention and registry leases. Each owner consumes only its original local
share; independent shares remain real Native close vetoes. Inert facts and
original allocations survive until the retired Home's explicit reclaim.

Implementation plan:
1. Expose Native admitted-basis retirement by reusing original observation
   retirement, with authentic-owner affinity, original request, retained inner
   data and typed busy/stop outcomes. Add the exact facade entries.
2. Add World snapshot/commit/composite-basis consuming steps and a prepaid
   consumed-publication retirement frame. Keep original fact/identity/result
   data, retire every Native share, refuse unsettled or undelivered custody.
   These steps neither write artifacts nor replace facts with copied identities.
3. Add Query receipt preflight and consuming advancement. Provider preflights
   every original receipt before World seal, then retires the completed evidence
   by an ordered cursor before source retirement. All original indexes stay in
   the same nonserving provider. No durable row is removed or rewritten.
4. Prove receipt-alias refusal resumes the same Home and mutation close releases
   the real physical lease while retired allocations remain alive. Preserve the
   six acceptance courts. Qualify deadline and checkpoint-failure controls on
   the ordinary stack, and report exact P-A reopen refusals separately.
5. Run affected focused/normal owner targets once as applicable, phase gates,
   and a fresh independent adversarial review of the committed result.

Requirement map additions: D11 receipt custody is traced to
held_original_publication_receipt_refuses_close_and_same_home_resumes,
ordinary_mutation_host_physically_closes_with_original_retired_allocations_alive,
and existing expired/checkpoint-memory Host close controls. D17 original Native
basis retirement is traced to original_basis_retirement_preserves_independent_reads_and_retained_payload.
The tests must exercise actual owners, not copied receipt descriptions.

Native basis retirement validates the authentic runtime's stable instance ID
without looking up or cloning the main branch. This keeps the post-checkpoint
step free of that live-table lookup and String allocation, while preserving
its exact ForeignOwner refusal. The original Native guard and registry lease
remain the release authority.

The prepaid World frame carries storage, not retirement authority. Its consuming
begin rechecks the actual publication's settled component and delivered handoffs;
refusal returns both original owners. D11/D17 map to
prepaid_retirement_refuses_another_undelivered_publication_and_returns_both_original_owners.
Unclaimed history and successor observations are one-shot delivery obligations,
not disposable caches: evidence.rs::claim_fresh_history and the live/output/client
observation take ports complete them on the resumed Home. The preflight preserves
those obligations and refuses pending work rather than silently dropping it.

## Receipt retirement qualification and remaining reopening boundary

The receipt-owner ruling is implemented by the consuming World/Native and Query
ports above. The original registered source, graph, provider, handlers and
completed publications remain in the same retired Home allocation until reclaim.
The actual media admission port acquires the physical mutation lease while that
retired allocation is still alive. These direct controls pass with
RUST_MIN_STACK absent: ordinary_mutation_host_physically_closes_with_original_retired_allocations_alive,
held_original_publication_receipt_refuses_close_and_same_home_resumes,
expired_host_close_preserves_the_same_serving_home_and_a_live_retry_physically_closes,
and host_checkpoint_memory_failure_is_reported_while_original_retired_owners_remain_physically_closed.
Their logs are target/lane-evidence/receipt-store-direct-final/*.log. This
supersedes the historical mutation ReadCustodyRetained blocker above.

All six unchanged acceptance courts now reach actual Native readmission and
refuse Relational(CompleteNativeRequired). The kill-without-close court first
records HOST_ACKNOWLEDGED_COMMIT=4, then Host admission refuses
CorruptCheckpoint: recovered source synchronization denied: RootLinkage. Logs:
receipt-store-controls/store-close-acceptance.log (0/6/0),
receipt-store-controls/host-kill-reopen.log (0/1/0), and
receipt-store-controls/native-cold-close.log (0/2/0, CompleteNativeRequired).
These reopening paths remain P-A's scope; their failures are not acceptance
success. No opening or Native durable-format implementation is changed here.

Native's new original-basis behavior court passes (1/0/0); World normal owners
pass (305/0/0), followed by the three affected publication-custody cases after
the consuming-refusal correction (3/0/0). Retired Native basis and consumed World
publication each pass their compile-fail read-access court (1/0/0 each).
The new World behavior reuses the existing close publication-custody court and
its real fixture, rather than adding a source-court baseline exemption.

Narrow new facade entries are Native's RelationalBranchBasisRetirement,
RelationalBranchBasisRetirementDenial, RelationalBranchBasisRetirementProgress,
RetiredRelationalBranchBasis, and World's PreparedConsumedPublicationRetirement,
ConsumedPublicationRetirement, ConsumedPublicationRetirementDenial,
ConsumedPublicationRetirementProgress and RetiredConsumedPublication. This
receipt slice adds no boundary configuration entry or application_home/mod.rs
entry. The earlier exact query-home-retirement caller seal remains unchanged.
