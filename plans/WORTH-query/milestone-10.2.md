# Milestone 10.2: Typed Operation Progression And Delivery

**Status:** design brief; not implemented. Parent: [Milestone 10](milestone-10.md).
Requires the owning stage handoff in [10.1](milestone-10.1.md).

The existing source/fixed/discovered/workflow owners already solve ordinary
continuation. Make their shared construction and legal transitions compiler-total
so new composition cannot reconstruct them from posture labels or receipts.
Keep the existing Decl/Host operation entries; do not add a second operation API.

| Phase | Usable result | Requires | Enables | Parallel constraints |
|---|---|---|---|---|
| 1 | Source and fixed owners implement one sealed progression contract. | 10.1; existing action/delivery declarations. | Complete original-owner handoff. | Shared protocol freezes before other adapters. |
| 2 | Discovered and workflow adapters preserve their distinct authority. | Phase 1. | Full operation-family composition. | Separate adapter work can proceed in parallel. |
| 3 | Settlement, delivery and replay remain independently typed. | Phases 1–2. | 10.3 disposal and 10.4 composition. | Delivery integration consumes complete family adapters. |

## Phase 1: One complete operation adapter contract

Extend the existing mode selected by `ApplicationActionDelivery::Mode`.
Generalize `operation/history_mode.rs` and its sealed driver at the operation
owner; history consumes that adapter rather than supplying group-specific hooks
to otherwise ordinary operations. Reuse `native_publication.rs` for shared
Native facts, not an independently reconstructed publication state machine.

Proposed platform-only shape, parameterized by its original schema/action
binding and existing ordinary request type:

```rust
pub trait OperationLifecycle: sealed::OperationLifecycle {
    type Running;
    type Completed;
    type Terminal;
    type Disposal;
    type CloseReceipt;
    type CloseDenial;
    type Request<'r>;
    type Denial;

    fn advance(
        owner: Self::Running,
        request: Self::Request<'_>,
    ) -> OperationAdvance<Self::Running, Self::Completed, Self::Terminal, Self::Denial>;

    fn into_disposal(owner: Self::Running) -> Self::Disposal;
    fn terminal_into_disposal(owner: Self::Terminal) -> Self::Disposal;
    fn completed_into_disposal(owner: Self::Completed) -> Self::Disposal;
}

pub enum OperationAdvance<O, C, T, D> {
    Progress(O),
    Refused { owner: O, denial: D },
    Completed(C),
    Terminal { owner: T, denial: D },
}
```

This is a sealed normalization of existing outcome shapes, not an application-
implemented executor. The request association is a borrow of the existing fresh
ordinary request; it cannot capture principal, grants or cancellation in the
retained owner. New protocol members have no permissive default implementation.
Implementing a new delivery mode without a required owner transition fails
compilation; unsupported installed combinations refuse before effects.
The initial adapters delegate existing lawful native disposal. 10.3 completes
disposal of unresolved states and application close; it does not supply a
missing executor needed for this phase's first working journey.

Prepared source execution consumes actual prepared mutation/native proofs.
Completed and terminal types do not expose `advance`; a terminal outcome cannot
be made retryable by matching a descriptive enum. A fresh business attempt
requires the existing owner-issued no-effect proof and fresh admission. Private
exhaustive sums remain legal for pumping dynamically discovered work. Do not
create a generic public type for every internal state or erase all causes to
`NotReady`.

Architecture 9/13 require exhaustive mode construction and one declared
membership; 10/16 require distinct original meaning, admission and completion;
DX 8/19 require valid next actions without private-state knowledge.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Ordinary source/fixed | Retain real action, advance, stop at post-effect handoff, then resume. | Same original intent/result/output owner; one source publication. | Query compiled consumer, CI. |
| Legal transitions | Attempt to advance completed/terminal owner, forge prepared proof or implement an incomplete mode; valid counterparts. | Compiler rejects the intended missing capability/constructor. | Existing consolidated public compile lane. |
| Current authority | Revoke only mutation grant before first effect, then restore; separately revoke after a performed source. | Fresh work refuses before effect; original recovery follows its own admission without pretending a new effect is authorized. | Real Query capability fixture, CI. |
| Native no-effect | Resolve indeterminate key as unseen without proven abort; contrast actual proved abort. | No handler replay from unseen alone; genuine retry path stays available. | Existing native-publication/consumer recovery courts. |

Close with source/fixed adapters in production and their old independent
construction/progression wiring removed. The ordinary caller still retains the
returned operation and supplies only a fresh request on retry.

## Phase 2: Preserve discovered and workflow distinctions

Add implementations of Phase 1's exact protocol for the existing
`WorthQueryDiscoveredOperation` and `WorthQueryWorkflowOperation`. Extend their
current owning modules. Their typed completion/terminal owners remain distinct.

For discovered outputs, the original unregistered carrier, completed members,
pending members and discovered membership belong to the owner. Receipt lookup
cannot substitute for failed carrier registration. For workflow, native issued
transition/approval authority and spent/indeterminate dispatch outcomes remain
in their current owners. A source-only adapter cannot impersonate workflow
progression, and workflow-instance cancellation is an admitted mutation, not a
request cancellation flag.

| Transition | Exact input and grant | Output / next permitted use | Refusal and effect posture |
|---|---|---|---|
| Discovered source to retained outputs | Original carrier + installed discovered delivery contract. | Existing discovered output owner with complete membership. | Failed retention returns the carrier; no rediscovery or replay. |
| Workflow source/dispatch | Original issued workflow authority + current owner admission. | Existing completed/pending/terminal workflow owner. | Spent authority/unknown external fate retain native posture; no generic retry. |

The caller uses the same `advance` shape as Phase 1. Protocol association, not a
caller-supplied mode or string, selects the implementation. Architecture 12
allows shared ownership mechanics while forbidding shared fake recovery power;
Architecture 14 keeps external-effect recovery distinct.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Discovered retention | Commit source, refuse registration of real discovered carrier, stop and retry. | Original membership, result and completed prefix survive; no source replay. | Existing discovered consumer fixture, CI. |
| Workflow approval | Revoke a real approver grant before execution; lose delivery after lawful execution in the twin. | Denied case has no effect; performed twin resumes exact native custody without reissuing authority. | Existing workflow/approval public courts, CI. |
| External indeterminacy | Lose real public Bank dispatch acknowledgement and attempt retry/cancellation. | Native dispatch/reconciliation owner remains; no duplicate payment or false rollback. | Existing public Bank external-effect court; scheduled if process-expensive. |

Close when the shared protocol supports all four operation families without
`Any` payload extraction or a history-specific progression backend.

## Phase 3: Separate settlement, delivery and replay

Reuse `WorthQuerySettledFixedOperation`, `WorthQuerySettledDiscoveredOperation`,
`WorthQueryCompletedSourceOperation`, `WorthQueryClosedFixedDelivery` and native
workflow completion types. Preserve the existing public delivery calls. Target
shape for a delivery-bearing owner is:

```rust
fn deliver_current(
    &mut self,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> Result<Delivered, WorthQueryOperationDeliveryDenial>;

fn close_for_delivery(
    self,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> Result<ClosedDelivery, (Self, WorthQueryOperationCloseDenial)>;
```

`Delivered` and `ClosedDelivery` are the existing mode-associated typed results,
not universal serialized values. The delivery owner retains settlement after a
refused read. Current delivery checks original full output lineage and actual
permission at one authentic observation. A completed source does not imply
complete required outputs. Recovery of a receipt is descriptive committed
evidence unless actual result/registered output custody is available; it cannot
fabricate the fresh handler return. Close and successful delivery are separate.

Architecture 5/7 preserve failure and commit posture; 17 requires exact reuse
meaning; 20 prevents retained settlement from granting disclosure. Same-scope
stable facts are carried, while actual grant/head/program changes require
owner-controlled revalidation at their use boundary.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Refused delivery | Settle outputs, revoke read grant or cancel final delivery, then retry lawfully. | Original settlement survives; no producer/source rerun solely to redeliver. | Existing fixed/discovered consumer courts, CI. |
| Replay provenance | Lose reply after real commit; replay with registered custody, then with only historical evidence. | Registered owner resumes; unavailable/historical case is explicitly typed and cannot return invented domain result. | Existing replay/retention courts, CI. |
| Currentness | Change a consumed fact and an unrelated fact separately before delivery. | Consumed drift refuses current claim; unrelated change follows native exact validation. | Existing output-currentness fixture, CI. |
| Build locality | Instantiate source/fixed/discovered/workflow examples in existing targets. | Shared non-generic core; test-only edits do not rebuild production; no new per-example binary. | Existing build/court topology gates, CI. |

Update compiled operation/recovery examples in the existing application guide.
Do not declare completion from facade type existence: actual stopped delivery,
replay and partial publication journeys plus displaced-path deletion are gates.
