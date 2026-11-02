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
| D11 ordinary handler custody and retained payload | Handler registry retirement controls and Host `handler_retirement` courts; coordinator ruling retires the old ordinary-supplier Unsupported assertion |
| D11 original slot/handler aliases refuse Busy | `original_erased_slot_and_handler_aliases_preserve_the_same_handler_and_payload` and same-Home Host resume control |
| D11 active call refuses before seal | `active_installed_call_refuses_before_world_seal_and_resumes_the_same_home` |
| D11 registered source, serving graph and provider custody | Six unchanged `query_readmission::native_artifacts` acceptance courts named in the P-B brief, plus owner retirement controls |
| D11 original request deadlines and refusal custody | Host close interruption/retry courts, with the same original owners retained |
| D11/D12 failed close checkpoint does not prevent Closed | Host checkpoint-failure court, observing actual physical shutdown and retained checkpoint failure |
| D11 kill without close preserves acknowledged results | Production Host filesystem process court; exact RootLinkage or CompleteNativeRequired reopen refusal is reported to P-A |
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
