# Milestone 10.4: Declared Composite Operations

**Status:** design brief; not implemented. Parent: [Milestone 10](milestone-10.md).
Requires operation progression in [10.2](milestone-10.2.md) and disposal in
[10.3](milestone-10.3.md). This strengthens composition, not domain semantics.

An installed definition must select complete lifecycle wiring. A new group,
navigation action or workflow adapter should consume existing operation owners
without interpreting their private Native state, rebuilding admission or
managing an invisible cleanup collection.

| Phase | Usable result | Requires | Enables | Parallel constraints |
|---|---|---|---|---|
| 1 | Installed action modes determine complete composition eligibility and lifecycle. | 10.2/10.3 interfaces. | Composite construction without registration folklore. | Freeze installed contract before caller migration. |
| 2 | Groups and navigation use the shared ownership transitions. | Phase 1; accepted 9.18.2 behavior. | Real multi-step adoption/abort. | Navigation and group adapters may be separate slices. |
| 3 | Workflow/external-effect composition and a new extension use the same rules. | Phases 1–2. | Mechanical extension rather than special-case integration. | Can proceed beside 10.5 after common contracts freeze. |

## Phase 1: Derive lifecycle wiring from installed action meaning

Extend `ApplicationActionDelivery::Mode` and the existing installed action
declaration. The sealed runtime implementation supplies 10.2's mandatory owner
types, 10.3's disposal, result provenance and any actual candidate eligibility
proof. Do not add a second lifecycle registration keyed by strings or action
labels, and do not make applications implement authority-bearing drivers.

Proposed typed eligibility boundary under Query's operation owner:

```rust
pub struct WorthQueryCandidateOperation<L: OperationLifecycle> { /* private original owner and constraint */ }

pub fn admit_candidate_operation<L: OperationLifecycle>(
    operation: L::Running,
    constraint: WorthQueryDocumentHistoryOperationConstraint,
) -> Result<WorthQueryCandidateOperation<L>, (L::Running, WorthQueryCompositionDenial)>;
```

The existing exact candidate constraint is the proof to consume; the wrapper
does not reconstruct it. The installed contract fixes which operation modes
may enter that candidate lineage, required outputs, scope and escaped-effect
posture. An external dispatch is not candidate-local because its carrier
resembles a source operation. Missing or unsupported composition refuses before
candidate/member effects. Type-level known incompatibilities are uncallable;
dynamic installed mismatches have typed early refusal.

Keep execution planning separate from fresh authority. Installation selects
the legal lifecycle and output shape; each actual step admits principal, branch,
head and current program through their original owners. No framework callback
may infer a mutation permission from successful READ or metadata admission.

Architecture 3/4/13 require declared membership and lowered strategy rather
than scattered registration; 10/16 require exact sealed constraints; DX 5/19
require one definition preserving composition guarantees.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Missing wiring | Add an action mode missing disposal or required result ownership; lawful complete mode. | Incomplete framework implementation fails compilation; unsupported installation refuses before effects. | Existing declaration/install and compile-contract courts, CI. |
| Candidate identity | Use a constraint issued for another runtime/document/lineage. | Original operation returned with typed affinity refusal; no member handler runs. | Query real candidate fixture, CI. |
| Effect eligibility | Attempt candidate grouping of installed escaped-effect action; lawful local action twin. | Rejection at actual eligibility boundary before external contact. | Query group/public Bank capability boundary, CI. |

Close when adding an eligible action needs its existing declaration and one
complete sealed runtime adapter, without edits to unrelated registries.

## Phase 2: Use the protocol for private groups and navigation

Keep accepted `begin_group`, `advance`, `complete`, `finish`, `abort`, checkout,
`undo_step` and `redo_to(child)` behavior and ordinary entries. Refactor their
composition through the installed adapter, not another group executor.

The internal target interface on an admitted member is:

```rust
impl<L: OperationLifecycle> WorthQueryCandidateOperation<L> {
    pub fn advance(
        self,
        request: L::Request<'_>,
    ) -> OperationAdvance<Self, L::Completed, L::Terminal, L::Denial>;

    pub fn into_disposal(self) -> WorthQueryOperationDisposal<L>;
}
```

Concrete schema/action bounds and the wrapped mode are supplied by the existing
installed action. The composite consumes actual completed member owners to
record its next predecessor; it cannot take an arbitrary receipt as proof that
the candidate reached that state. A completed output mode retains complete
required-output custody, not only its source receipt.

The group still owns its actual semantic decisions: private candidate lineage,
ordered membership, original attempt identity, one visible history edge and
guarded final adoption. Navigation still owns its retained historical target
and expected selection. The shared protocol handles member interruption,
continuation, original-result retention and disposal.

Before adoption, failure leaves the visible document unchanged and preserves
real candidate effects. After adoption, stopped delivery retains the original
selection result. Retry resolves that result without moving today's head again.
Abort uses disposal-only members; no posture whitelist may invoke ordinary
business `advance` as a cleanup shortcut. Competing adoption uses the existing
document/World/Native guards. These are runtime checks, not permanent typestate
claims about a mutable head.

| Transition | Input/proof and granting owner | Output / next consumer | Refusal and effects |
|---|---|---|---|
| Execute member | Candidate-constrained original operation + fresh admission. | Actual completed member owner. | Refusal keeps original custody; no hidden prefix adoption. |
| Adopt composite | Completed member endpoint + original group/navigation reservation. | Existing selected-step owner/result. | Guard loser retains candidate and cleanup; winner owns delivery. |
| Abort | Original members consumed into 10.3 disposal. | Original disposal/close progress. | No business re-execution, compensation or false rollback. |

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Private group | Execute two real edits, reading active and staged state between them. | Old visible state until adoption; final expected values and exactly one authored edge. | Existing public document-history group court, CI. |
| Late refusal | First edit publishes, second reaches installed invariant refusal; revoke mutation and move active head before abort. | Earlier candidate effect retains lawful disposal; active state is not partially adopted or rolled back. | Same real group fixture with actual cleanup readback, CI. |
| Adoption/delivery | Cancel immediately after successful adoption, retry after a later navigation. | Original result delivered; no duplicate step or repeat movement of current head. | Existing group/navigation handoff courts, CI. |
| Race | Prepare real group completion and navigation at selection; exercise both winner orders. | Exactly the lawful guard winner adopts; loser retains actual owners. | Existing document-selection race fixture, CI. |

Close with the old group-specific Native posture/extraction wiring deleted
from ordinary operation definitions. Compiled application examples retain the
same supported behavior and teach actual abort/refusal handling.

## Phase 3: Preserve workflow and external-effect ownership

Use the existing workflow action/transition declaration, issued approval and
dispatch/reconciliation owners as adapters to the same composition contract.
Do not turn candidate grouping into a generic saga, or turn cancelling a request
into an admitted workflow-instance cancellation.

Target workflow caller continues using its existing owner:

```rust
use worth_query_host::facade::application_entry::WorthQueryWorkflowOperationAdvance;

match operation.advance(&request) {
    WorthQueryWorkflowOperationAdvance::Progress(next) => retain_pending(next),
    WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
        retain_refusal(operation, denial)
    }
    WorthQueryWorkflowOperationAdvance::Completed(completed) => retain_completed(completed),
    WorthQueryWorkflowOperationAdvance::Terminal(terminal) => retain_terminal(terminal),
}
```

The `retain_*` functions here denote the application's existing storage/UI loop;
they confer no authority. Shared adapter implementation must carry original
workflow provenance, exactly-once native attempt identity where already promised,
and external uncertainty. It must not lower these to a generic successful source
completion. All local/irreversible/compensatable distinctions remain installed
domain contracts. No new dispatch occurs during disposal or read-only delivery.

Prove extension with one new small public action/composite in the existing
consumer fixture. It must enter through an ordinary declaration and supported
adapter, without editing fixed/discovered/workflow dispatch tables outside the
semantic mode owner or creating a feature-local cleanup list. This is an actual
compiled integration exercise, not a scanner counting declarations.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Workflow authority | Lose approval/current transition after preparation; lawful twin advances. | Existing workflow authority refuses stale work; wrapper cannot restore it. | Existing workflow public consumer, CI. |
| External callback | Cancel local request after actual dispatch, then receive/repeat callback. | External/native fate remains exact; no false rollback, duplicate dispatch or reopened terminal workflow. | Existing public Bank transport process court, scheduled if expensive. |
| New extension | Implement small ordinary action/composite, then omit a required lifecycle method. | Valid journey runs; omitted method fails compile/install at its own boundary, not an unrelated fixture gate. | Existing compiled consumer and consolidated compile lane. |

QA must challenge adapter omission, owner loss, source/result substitution and
an advanced/public bypass. Reuse existing court targets and boundary/dependency
checks. Do not add a source inventory or a generic recursive certification
framework. Architecture 12/14 and DX 19 govern the different effect meanings.
