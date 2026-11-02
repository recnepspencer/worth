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
| D11 ordinary handler custody and retained payload | `default_supplier_hooks_retire_invocation_without_destroying_shared_payload`, `ordinary_supplier_retirement_retains_the_original_payload_until_reclaim`, and Host `handler_retirement` courts; coordinator ruling retires the old ordinary-supplier Unsupported assertion |
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
