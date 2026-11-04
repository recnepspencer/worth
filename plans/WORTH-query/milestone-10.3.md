# Milestone 10.3: Disposal, Abandonment And Shutdown

**Status:** design brief; not implemented. Parent: [Milestone 10](milestone-10.md).
Requires [10.1](milestone-10.1.md) and the owner interfaces of [10.2](milestone-10.2.md).

Make releasing a retained operation an ordinary, typed lifecycle operation that
cannot accidentally retry its business action. Reuse World active custody,
recovery catalogs, component retirement and Query's existing cleanup owners.
Do not build another global obligation registry or promise that Drop finishes
external work.

| Phase | Usable result | Requires | Enables | Parallel constraints |
|---|---|---|---|---|
| 1 | Every operation can consume its continuation into disposal-only ownership. | 10.2 protocol; 10.1 handoff. | Safe abort and explicit close. | Family adapters share a settled disposal result contract. |
| 2 | Abandonment preserves exactly the original native obligations. | Phase 1 and existing World/Query registries. | Reliable residual recovery after lost handles. | Investigate Native/managed adapters independently. |
| 3 | Session/World close returns complete, typed residual work. | Phases 1–2. | 10.4 composite abort and 10.5 managed-session integration. | Integrate after concrete residual types exist. |

## Phase 1: Consume into disposal without business authority

Generalize the real behavior in `operation/cleanup.rs`, source-only disposal and
fixed-output disposal. Group-specific entry names move behind the shared
operation protocol; a history group should not decide which native posture is
safe to advance merely to discover whether it may close.

Target Decl/Host shape on each existing running/completed/terminal owner:

```rust
pub fn into_disposal(self) -> WorthQueryOperationDisposal<Mode>;

pub enum WorthQueryDisposalAdvance<D, R, E> {
    Pending(D),
    Refused { disposal: D, cause: E },
    Closed(R),
}

impl<L: OperationLifecycle> WorthQueryOperationDisposal<L> {
    pub fn advance(
        self,
        request: L::Request<'_>,
    ) -> WorthQueryDisposalAdvance<Self, L::CloseReceipt, L::CloseDenial>;
}
```

`Mode` is the sealed lifecycle association selected by the original action's
delivery declaration in 10.2, including its close receipt/denial types. Entering disposal performs no
I/O or release and retains all original obligations, even if their fate is
unknown. It consumes ordinary advancement authority. The disposal owner has no
business `advance`, `deliver_current`, candidate-adopt or workflow-dispatch
method. Its `advance` means only the named disposal transition above.

Each attempt needs fresh request controls and exact runtime affinity. A revoked
business mutation grant, retired original source or superseded selected head
does not itself authorize abandoning already-owned cleanup. Existing native
recovery/cleanup admission still decides legal settlement or release. Where
external fate cannot be resolved without another authority, return a typed
retained obligation; never dispatch the effect to make cleanup finish.
Successful partial component retirement does not erase remaining work.

Architecture 10/16 require disposal capability distinct from business execution;
14 forbids hidden compensation/rollback; 19/22 preserve pending ownership.
Place it under ordinary operation ownership with concrete native adapters, not
under history. Keep domain-specific external reconciliation with its owner.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Partial source | Perform first candidate edit, refuse second, revoke mutation grant and move active head; dispose. | Active document unchanged by disposal; actual first effect closed/reported through original owners; no handler replay. | Real Query grouped/ordinary public fixture, CI. |
| Cleanup refusal | Refuse original cleanup admission, then retry with a lawful fresh request. | Same disposal owner and unfinished work returned; prior successful releases are not repeated as new business effects. | Native/Query cleanup courts, CI. |
| Missing capability | Attempt business advance/dispatch or adoption on disposal owner; valid disposal twin. | Compiler rejects prohibited method at the intended public boundary. | Consolidated compile-contract lane. |
| Unknown fate | Hold genuine indeterminate publication/dispatch, enter disposal. | Original resolution owner retained; no fabricated no-effect and no unconditional close receipt. | Existing native/keyed and public Bank recovery courts. |

Close after running/completed/terminal families share this contract and the
old caller-written posture whitelist is insufficient to perform cleanup.

## Phase 2: Keep abandonment in existing owners

World `ActiveAttemptCustody` already preinstalls an `ActiveAttemptRecord` in its
recovery catalog. Drop preserves actual partial effects. Query managed output
failures can also hold obligations not represented by a World publication
record. Audit and connect these specific owners rather than assume every
operation can be recovered from a commit receipt.

Target transfer remains an owner-issued, consuming operation:

```rust
pub fn abandon(self) -> WorthQueryAbandonedOperation;
```

The returned value describes the disposition; it is not a new recovery permit.
The installed runtime retains native residuals before the caller capability is
released. Original typed completion/result custody is preserved only where its
existing owning storage can retain it; otherwise the outcome reports result
unavailable while preserving actual native obligations. Never keep arbitrary
borrowed generic values in a new global `Any` registry.

Register a recoverable native duty before the effect that creates it. Use
World's current catalog and branch custody, the managed-run cleanup owner and
output-demand registry for their respective duties. A missing adapter or
unrepresentable duty refuses before effects. Losing the caller handle cannot
mint another caller's recovery authority. Only a granted recovery entrance may
act on inspected residuals.

Drop performs bounded local release/marking and does not allocate or scan all
operations. Existing funded owner admission performs deferred work. `mem::forget`
cannot be made safe by a Drop guarantee: preinstalled duties remain visible,
and close reports/refuses unresolved live custody. Process/runtime destruction
retains only the underlying existing durability promise, never a new one.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Partial abandonment | Drop/unwind after real Native effect but before product publication. | Existing World record names exact effects and original recovery next actions. | World active-custody courts, CI. |
| Managed cleanup | Abandon a genuine failed producer with cleanup not represented by a commit record. | Native managed owner remains registered/reachable or close returns its actual retained obligation. | Query managed-cleanup public court, CI. |
| Registration failure | Exhaust existing relevant registry capacity before starting a new effect; lawful twin. | No effect or invisible resource created; existing duties unchanged. | Actual owner registration court, CI. |
| Scale | Release one handle with unrelated retained records added. | Bounded local Drop work; no global scan, hidden execution or synthetic business retry. | Existing owner cost lane. |

Architecture 9/19 require total lifecycle propagation and tracked resources;
20 keeps descriptive inspection from becoming recovery authority. Extend actual
catalog/managed-owner admission and disposal, not a new root lifecycle service.

## Phase 3: Compose explicit close and residual work

World close already excludes new admission and returns
`RuntimeWorldCloseReport`, retained records and `OwnerRetirementWork`; its
enumeration settles nothing by itself. Query session close must carry those
owners together with demand/live/managed residuals. Add a progress owner for
potentially long draining rather than an unbounded destructor.

Proposed Host entry on the existing owning installed-program runtime handle,
delegating to that exact application's shared lifetime gate:

```rust
impl<S: ApplicationSchema, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn begin_close(&self) -> Result<WorthQuerySessionClosing<S, P>, WorthQuerySessionCloseDenial>;
}

impl<S: ApplicationSchema, P: ApplicationProgramDefinition<S>> WorthQuerySessionClosing<S, P> {
    pub fn advance(
        self,
        request: &WorthQueryRequestScope,
    ) -> WorthQuerySessionCloseAdvance<S, P>;
}

pub enum WorthQuerySessionCloseAdvance<S, P> {
    Pending(WorthQuerySessionClosing<S, P>),
    Refused { closing: WorthQuerySessionClosing<S, P>, cause: WorthQuerySessionCloseDenial },
    Closed(WorthQuerySessionCloseReport),
}
```

`Session` here means the Query-owned resource lifetime of that installed
application, not a new caller-created session or replacement of
`RuntimeWorldOwnerRoot`. Every clone of the same installed backing observes the
same gate; another application and the process execution service remain separate.
The report owns remaining retirement/cleanup work or runtime-issued access to
retained recovery; it cannot be an identifier list whose only owners were dropped.
Admission races use the current World operation/close lock order. Closing denies
new work, preserves genuine in-flight critical sections, and performs owner work
outside lifecycle locks. Refused close retains progress for a fresh request.

Do not terminally close World while a retained duty still needs its open recovery
service. First settle/dispose it or transfer an actual independently usable
owner; otherwise retain `WorthQuerySessionClosing` with a typed recovery-required
refusal. A report naming a record whose only legal recovery entrance was just
closed is not a usable handoff. Read-only observation holders may remain under
their existing contract and must be reported separately from active recovery.

**Holder census.** Where a residual is a shared runtime, medium or physical holder, the report and every share refusal carry [10.7](milestone-10.7.md)'s typed holder census, never an anonymous count. The census is diagnostic and grants no retirement authority.

**Owner lifecycle states as types.** World and Query owners expose their Open, Closing and Closed states as consuming types where the transition is stable:
- operations legal only after close (origin retirement, reclaim) take the closed owner;
- inspection ports are issued only from the open owner.

This deletes runtime refusals that exist only to reject the wrong state, such as `OwnerNotClosed`. Permission, currentness and live availability stay runtime checks at their use boundary, as the parent requires.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Close/admission race | Pause genuine publication and race session/World close in both winner orders. | No admitted effect after successful close; active winner keeps exact custody and produces lawful close refusal/progress. | Existing World close/custody court plus Query session integration, CI. |
| Residual work | Close with unpublished record, failed component retirement and retained observer. | Report identifies actual surviving owners separately; enumeration never claims settlement/reclamation. | Existing World close-report fixture, CI. |
| Recovery still needs World | Retain genuine recovery requiring the open World service, then request terminal application close. | Closing retains runtime and recovery until resolved or lawfully transferred; no dead recovery ID masquerades as usable custody. | World recovery/close plus Query application-close integration, CI. |
| Stop draining | Cancel between two actual close releases, then resume. | Already completed release preserved, unfinished owner returned, no all-or-nothing claim. | Query close integration, CI. |
| Locality | Close one application while another has live demands. | Only targeted lifetime is closed; sibling remains usable. | Public Query multi-runtime fixture, CI. |
| Holder census | Close while distinct local and external shares remain. | Refusal and report name every live role; each disappears after its real release. | 10.7 Home close courts plus Query session close, CI. |
| State typestate | Call a closed-only retirement on an open owner, and an open-only inspection on a closed owner; lawful twins. | Compile failure at the intended boundary; no runtime `OwnerNotClosed` path remains. | Consolidated compile-contract lane. |

10.5 supplies demand/live/provider adapters to this close contract. This phase
qualifies operation/World close; group-wide application close additionally
requires 10.5 and cannot be claimed while an owned resource family lacks its
adapter. Update existing
recovery/close docs with actual residual handling, never an unconditional
"Drop cleans everything" promise. No new storage or crash-recovery guarantee.
