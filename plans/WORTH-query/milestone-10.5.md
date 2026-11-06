# Milestone 10.5: Managed Computations, Outputs And Live Lifetimes

**Status:** design brief; not implemented. Parent: [Milestone 10](milestone-10.md).
Requires owning handoff [10.1](milestone-10.1.md), disposal/close
[10.3](milestone-10.3.md), and [10.2](milestone-10.2.md) for operation adapters.

Extend the same ownership discipline to pending/retained work. Existing managed
runs, provider sessions, demand registry, live leases, Bridge finalization and
Signal resources remain their own engines and authorities. Their lifetime
transitions must compose without a feature author writing another cleanup loop.

| Phase | Usable result | Requires | Enables | Parallel constraints |
|---|---|---|---|---|
| 1 | Managed runs and yielded provider work retain real continuation/cleanup owners. | 10.1 and 10.3. | Safe stop/resume/dispose of computation. | Provider adapters can proceed beside live integration. |
| 2 | Output interests and live leases expose retained termination ownership. | Phase 1 handoff contract; current demand/live owners. | Total application lifetime close. | Separate demand/live slices share one disposal result contract. |
| 3 | Installed application close and new runtime extensions are mechanically complete. | Phases 1–2; 10.3 close owner. | Group-wide runtime lifecycle adoption. | Final integrated qualification follows adapter cutover. |

## Phase 1: Keep yielded work distinct from interrupted work

Managed runs already own provider sessions, checkpoint suspension, Bridge basis
and artifact production. Yield/checkpoint resumption, hard request interruption,
domain refusal and successful completion have different continuation rights.
Use the native managed-run variants rather than normalizing all of them into
"pending" or authoring another request scope.

Target consuming disposal on existing retained run/suspension owners follows
10.3. Existing installed single-kernel `prepare`/`compute`/`complete` and
`checkpoint.poll()` remain the ordinary computation path:

```rust
fn compute_chunk(
    input: &[u64],
    checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
) -> Result<u64, WorthQueryManagedComputationCheckpointDenial> {
    let mut total = 0_u64;
    for value in input {
        checkpoint.poll()?;
        total = total.wrapping_add(*value);
    }
    Ok(total)
}
```

This illustrative local kernel declares no effect or independent oracle claim.
The framework checks original request controls before/after compute and at
handoff. A yielded owner resumes only from its actual provider checkpoint under
the required fresh admission. An interrupted run cannot quietly resume inside
the cancelled scope. A blocking numerical/library/network call remains bound
to its own cancellation support; polling before it is not preemption.

Provider registration supplies concrete continuation, terminalization and
disposal behavior once. A new provider missing one required transition fails
compilation/installation. Shared plumbing carries the original execution policy;
no provider-local worker pool or fallback unbounded request is allowed.
Architecture 9/16/19/22 enforce lifecycle completeness, legal progress and honest
interruption; DX 4 keeps long-running work owned.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Genuine yield | Yield an actual provider run with retained checkpoint/artifact/basis, then resume. | Original continuation and dependency meaning retained; no fresh run disguised as resume. | Existing managed-run yield/recovery tests and public producer fixture, CI. |
| Hard stop | Cancel parent during compute and ignore one checkpoint failure; lawful twin completes. | Latched stop prevents success, no result disclosure, original cleanup survives. | Existing Execution nested-stop + Query managed public case, CI. |
| Freeze/close refusal | Refuse actual checkpoint freeze or Bridge finalization during yield/dispose. | Native refusal and original suspension/cleanup owner retained. | Existing managed-run yield-cleanup owner tests, focused/CI. |
| Provider omission | Implement provider without required terminal/disposal adapter; valid counterpart. | Compile/install rejects before provider execution. | Existing provider installation/compile lane. |

Close with actual adapters in production, not a parallel generic managed-run
state machine. Preserve existing domain error meanings and reuse semantics.

## Phase 2: Own demand and live termination failures

`WorthQueryApplicationOutputDemandContinuation` already owns the admitted
interest, retains no principal/scope and exposes managed cleanup. Its `close`
releases an interest; that is not complete output settlement or failed producer
cleanup. The live lease already releases its queue, Bridge basis and graph read
proof, but termination can fail and Drop cannot expose an owned retry to a caller.

Target public live close extends the current `close` result with real retained
custody where termination did not finish:

```rust
pub enum WorthQueryApplicationLiveCloseAdvance<L, R, E> {
    Closed(R),
    Pending(L),
    Refused { closing: L, cause: E },
}

pub fn begin_close(self) -> WorthQueryApplicationLiveClosing;
```

The closing owner exposes only termination/cleanup. Its fresh `advance` consumes
the previous close owner and returns the same result shape. Existing successful
cheap close may remain a convenience only where it really terminates its full
owned lifetime; it cannot silently drop a failed owner. Demand disposal uses
10.3 while preserving separate interest-release and managed-cleanup evidence.
The closing carrier retains the existing live lease's runtime lifetime and
`!Send` restriction. It cannot be parked in a process-global `Any` table or
sent to an arbitrary worker. Queue release, Bridge finalization and graph-read
completion carry their actual successor proofs; a later refusal must not lose
those owners or attempt a terminal transition again from its predecessor.

Stopping delivery cannot widen disclosure: fresh principal, scope, READ and
source basis remain checked at their actual boundary. Preserve immutable
settlements while releasing this consumer's interests. Signal resource
cancellation/supersession uses its installed `ResourceCancellationDecisionPlan`
and existing host-advisory semantics. A host advisory is best effort; actual
external completion remains explicit. Query request cancellation cannot claim
the external task was undone or that another consumer's shared work was closed.

Place adapters under existing demand, live and managed owners; extend current
Bridge finalization, not its read authority. Architecture 12 forbids merging
these lifetime meanings; 19/20 preserve ownership and disclosure boundaries.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Shared demand | Two genuine consumers share work; cancel/dispose one, retain the other. | Only one interest released; surviving consumer and actual settlement remain lawful. | Existing output-demand sharing/cleanup public fixture, CI. |
| Live close failure | Cause actual queue/basis finalization refusal, then retry closing. | Typed retained close owner; queue/read/Bridge obligations neither lost nor falsely reported closed. | Existing live-lease lifecycle fixture + public close, CI. |
| Disclosure stop | Revoke live READ/principal or cancel request before delivery. | No protected delta disclosed; actual lifetime is closed or returned as owned close work. | Existing live delivery/admission public court, CI. |
| Resource cancellation | Cancel/supersede an actual Signal resource and deliver a late completion. | Installed cancellation policy, generation and dependent-resource rules preserved; no stale result becomes current. | Existing Signal resource direct/dependent cancellation court, CI. |

Close with cancellation, supersession and explicit disposal using the same
registered native owners. Retain existing terminal reason distinctions.

## Phase 3: Complete installed application lifetime integration

Use 10.3's installed application close receiver, shared by all clones of
`WorthQueryProgramApplicationRuntime`. It controls that exact application
allocation, not the process execution service. Add no separate user-created
session object required to make an ordinary operation safe.

Construction/fork/install must exhaustively carry the application close gate and
actual owner adapters. Existing World catalogs, managed cleanup, demand interests
and live registration remain the sources of residual truth. A combined report
borrows or owns their issued residuals; it does not replace them with a second
permission-bearing registry. New work observes the close gate before allocation
or effects; in-flight work retains its actual critical-section right.

The current `close_conditional_runtime` requires unique runtime backing and
returns `RuntimeKeepaliveRetained` otherwise. Preserve that safety: draining
framework work must not manufacture `Arc` uniqueness or destroy another genuine
holder. Close reports/pends on remaining holders separately from refusing new
work. It must distinguish closed admission, discharged owned resources and
retained external/observer obligations. No global drain in Drop.

Public caller follows 10.3's `begin_close`/`advance` example. A stopped close
returns its progress owner, while administrative inspection remains descriptive
and audience-scoped. Delivery of a retained result after close follows the
existing current/historical admission contract; it cannot reopen a closed session.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Mixed lifetime | Open genuine operation, failed producer, live lease and yielded run; begin application close. | New work denied; each original duty is discharged or returned as exact residual ownership. | Public Query installed-runtime integration, CI. |
| Retained clone | Keep another runtime/observer holder while close drains; release it independently. | Existing uniqueness guard retained; no fabricated full-close result; independent release enables legal continuation. | Existing runtime-keepalive plus new close journey, CI. |
| New extension | Install a small managed producer/live adapter; omit its termination wiring in a negative counterpart. | Valid ordinary journey works; omitted lifecycle rejected before allocating a live resource. | Existing compiled consumer/provider lane. |
| Scale and build | Release/advance one resource with more unrelated retained resources; build source/fixed/discovered/workflow modes. | Work follows the touched owner; no new pool/test binary. Shared lifecycle mechanics stay in one concrete core rather than a new phase/mode instantiation family. Review the existing codegen report for changed families. | Existing cost and iteration gates. |

Finish with ordinary execution and managed/live examples compiling against
Decl/Host, standard boundary/agent-context/format/line-cap gates, and the affected
native/public courts. Public docs teach supported progress, cancellation and
residual close handling. No test-name manifest, cross-machine qualification or
new independent numerical oracle is required.
