# Milestone 9.17.6.4: Framework-Owned Operation Continuations

> **Status:** Planned. Immediate successor to the completed
> [9.17.6.3](./milestone-9.17.6.3.md). This milestone makes Query own the
> ordinary application's operation continuation over its existing publication,
> recovery, output and currentness owners. It does not introduce another
> execution engine. Implementation and certification remain outstanding.

## Goal And Roadmap Placement

A framework-grade application entry retains the original operation description
and every live native obligation while its owning continuation is retained,
through honestly classified delivery and lawful close. The application supplies
domain intent, starts the
operation and pumps genuine pending work with fresh admission. It should not
rebuild an idempotency request, guess which publication phase survived, recover
native custody by receipt, or lose settled output when delivery is refused.

The default public API must make those mistakes mechanically difficult. Move-only
owners, sealed state views, constrained source modes and typed outcomes are the
contract; documentation and caller discipline are supporting evidence.

9.17.6.3 already supplies exact invalidation, native parallel advancement and the
shared execution authority. This successor composes that machinery with the
completed application-program and workflow contracts. It preserves the completed
milestones and their historical certification. It does not retroactively make
9.17.7 or the 9.17 umbrella depend on unfinished work.

The first vertical slice is a fixed required-output operation through the real
public consumer fixture, including genuine partial publication, fresh recovery,
settlement, refused delivery and successful retry. Discovered outputs, source-only
operations, workflow adapters, complete cleanup proof and public consumer cutover
remain required scope. Private application adoption is a successor obligation
against a certified public revision, not evidence this public milestone can claim.

## Current Boundary: What Exists And What Is Missing

Paths below are relative to `workspaces/worth-query/crates/`.

| Existing boundary | Present guarantee | Composition gap this milestone closes |
| --- | --- | --- |
| `worth-query-declaration/src/application_program/feature/authoring/capsule.rs` and `application_program/output_graph.rs` | Canonical feature membership and typed required/discovered output meaning; declarations grant no execution authority. | Bind typed delivery meaning to the existing canonical action/program contract, rather than adding a second registry. |
| `worth-query-publication/src/application_entry/mutation/request.rs` | Typed input, preconditions, row/result-set source expectation and borrowed idempotency key. | Own the original typed descriptor beyond the initiating borrowed request; retain no authority grant. |
| `application_entry/mutation/performed.rs`, `performed/unpublished.rs`, `performed_source/recovery.rs` | Move-only partial publication and a shared native source recovery phase. | Retain and compose these owners through the ordinary operation instead of making callers reconstruct request and phase. |
| `application_entry/mutation/performed/recovery/retention.rs` | Fixed committed-retention failure can retry using its original carrier. | Add the corresponding discovered committed-retention entrance, preserving its original discovery and custody. |
| `application_entry/mutation/discovered/recovery.rs` | Receipt-based recovery of an already retained discovered source. | A receipt cannot recover an unregistered carrier. Do not present this entrance as the missing retention retry. |
| `application_entry/mutation/performed_outputs.rs`, `discovered/outputs.rs`, `program_output_continuation.rs` | Native fixed/discovered traversal, settlement, retained admission prefixes and genuine pending work. | Delegate these owners through one ordinary owning continuation; do not copy their cursors or state machines. |
| `application_entry/request/output_currentness.rs` | Currentness checks native settlement lineage at an exact request observation. | Couple complete settlement evidence and typed query delivery at one authentic observation, retaining both after read refusal. |
| `application_entry/workflow/operation.rs`, `workflow/assessment.rs`, `workflow/ordinary/run.rs` | Workflow operation authority, recovery, assessment and bounded business progression. | Provide adapters to those owners without inventing a generic workflow recovery or readiness authority. |

The public certification fixtures under
`worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/`
already exercise real fixed and discovered partial publication. In particular,
`required_unpublished.rs`, `discovered/unpublished.rs`,
`discovered/initial_unpublished.rs`, `readiness_recovery/ready_read.rs` and
`settlement.rs` supply the first production-valid courts and consumer cutover.
Reuse their installed programs, fault boundaries and independent product queries.

Existing managed direct/workflow runs own provider resource execution, safe
points, convergence and native resource leases. An operation continuation owns
the application entry's outstanding obligations. Neither replaces the other.
Where a native operation uses a managed run, delegate that run unchanged.

The joined upstream foundation also includes typed demand-only roots,
`WorthQueryApplicationOutputDemandContinuation` and ordinary retained-current
reads. Source-only followed by independent assessment reuses these owners;
this milestone must not rebuild them. Implementation begins from the joined,
certified public foundation with these native contracts preserved.

## Adversarial Courtroom

The plausible false implementation is a happy-path wrapper which retries a
mutation, rebuilds its request from a receipt, drops settlement on a read error,
labels a historical result current, or treats successful delivery as cleanup.
The decisive evidence must convict each of those substitutions.

Use the real Query composition root and installed public consumer program. The
fixture's actual append failure must produce native partial publication; a mock
status enum or pre-effect error is not partial-effect evidence. Retain the owner,
then apply the following hostile sequences through the ordinary public entrance.

| Court | Hostile sequence | Required outcome and surviving state |
| --- | --- | --- |
| Fixed partial publication | Interrupt at the existing append fault after the source effect, then recover with a fresh admitted request. Try a wrong key, altered input/source, foreign runtime and lost authority first. | Typed refusal preserves the complete original owner. Correct recovery transfers the native carrier and completes required outputs without re-running the original source handler. Product history and queried outputs independently prove no duplicate source effect. |
| Complete descriptor retention | Use a real source-bound result-set operation with nondefault typed preconditions, then encounter partial publication. Attempt native recovery with changed/dropped preconditions, a substituted row or altered result set, or a substituted newer source observation. | Advanced hostile recovery refuses and preserves exact custody; the original descriptor with fresh lawful authority succeeds. The ordinary facade makes descriptor substitution unexpressible. Result-set expectation and precondition identity cannot silently become defaults or a row-only approximation. |
| Replay and resumption | Lose the reply after a genuine commit and initiate the same key/action/input again. Exercise live registered receipt recovery, an unavailable registration and a genuinely historical receipt on fixed and discovered lanes. | Registered replay resumes through native receipt recovery without a new source effect. Unrecoverable registration and historical replay return terminal historical evidence with remaining cleanup; neither manufactures the original handler result nor reports required outputs complete. |
| Discovered committed retention | Produce a real source effect whose discovered carrier could not be registered. Refuse retention again before a lawful retry. | The original discovered roots, typed result and unregistered custody survive every refusal. Retry promotes that carrier; receipt-only lookup and rediscovery cannot substitute. Existing settled roots and admitted prefixes remain owned. |
| Delivery refusal | Settle outputs, then deny the final read or exhaust its native admitted work. Retry with a fresh lawful request. | Settlement and immutable descriptor survive. Retry proves currentness and queries at one authentic observation. It does not publish or compute the original operation again merely because delivery failed. |
| Stale/current distinction | Advance an unrelated domain and then a consumed fact between settlement and delivery; also request a lawful historical observation. | Unrelated progress follows native exact invalidation. A changed required settlement returns typed Stale with original history/cleanup; it cannot yield a current result or advance a closed handle. A later observation is current only if the original settlement passes the complete native check. Historical delivery stays explicitly historical. |
| Independent cleanup | Leave a real native cleanup obligation while source/output work succeeds; refuse cleanup admission, then retry or explicitly close. | Delivery and cleanup have separate outcomes. Refusal retains remaining cleanup custody. Neither drop nor a delivered result claims external cleanup or durability. |
| Pending and stop boundaries | Use actual native deferred output work, cancellation and exhausted admission at each effect-bearing handoff. | One advancement respects its single native phase and latched stop. Pending remains pumpable; interrupted/refused states preserve every live obligation. No hidden polling, replacement request or larger budget evades the boundary. |
| Source and workflow distinctions | Execute a lawful source-only save and independent assessment; exercise a workflow operation recovery with its original required transition authority. | Source-only completion cannot stand in for required output completion. Assessment cannot authorize workflow business progression. Generic mutation recovery cannot erase `WorkflowUnsupported`. |

Choose the smallest real fixtures that expose each boundary, then increase actual
discovered fan-out and unrelated retained population independently. Under the same
finite admitted policy, composition must perform the same native work as the
equivalent correctly coordinated journey, plus bounded descriptor/dispatch work.
Unrelated retained population must not introduce a registry scan. Fan-out cost
may grow with the actual required set and typed delivery batch. Use measurements
only where needed to establish these growth/equivalence claims, not exact
incidental work counts or universal quotas.

Mutation probes must make these courts fail when they: rerun the source handler;
discard custody on refusal; accept an altered original descriptor; omit/default
its nondefault preconditions or result-set source identity; bypass fresh
admission; omit one discovered settlement from currentness; query at another
observation; classify old evidence as current; clear cleanup on delivery; or
replace an exhausted request. Compile-fail evidence rejects invalid source modes,
counterfeit settled views and unauthenticated state transitions. Preserve existing
advanced negative courts instead of weakening them to fit the new facade.

Wrong-key/input/source substitution is compiler-denied or unexpressible through
the immutable ordinary facade. Use existing advanced native hostile entrances
for runtime mismatch courts; do not add a descriptor mutation or attach-owner
escape hatch solely to exercise a negative test.

Crash behavior is limited to the native checkpoint/recovery contract. Test its
actual partial-effect recovery entrances where applicable. This milestone does
not promise that a live continuation can be serialized or survives process loss.

## Product Decision Lock

### One Original Description; Fresh Authority On Every Advance

Query constructs an immutable owning descriptor from the typed initiating entry:
the original key, intent/input, preconditions, source expectation and branch,
schema and installed-program affinity. A source expectation is the existing row
or result-set form, or the declaration-authorized no-source mode. The key remains
typed; it is not converted into a universal string. Do not retain a principal,
request scope, credential or permission grant.

The descriptor proves original meaning and identity, not permission or successful
publication. A fresh ordinary application request supplies current principal,
scope and native policy for every advance, delivery and cleanup attempt. It is
the existing `WorthQueryApplicationRequest`, not a rebuilt mutation request:
callers supply no repeated key, intent, preconditions or source expectation.
Query combines that fresh context with the retained descriptor inside the native
phase and obtains descriptor-bound native admission. Any native recovery request
that needs original typed fields is constructed there by Query. This adds no
stored authority grant or admission registry. Existing native admission validates liveness, identity,
program/source/branch affinity and authority. Descriptor mismatch refuses before
effects; refusal preserves custody. An installed-program change follows the
existing evolution/compatibility and fresh admission rules, never a caller-selected
substitute action with the same label.

The descriptor's branch is fixed. A fresh request on another branch produces a
typed foreign-branch admission refusal and preserves the complete owner; Query
must not follow the host's newly current world or silently rebind authority.
Callers may select the owner's exposed branch when creating fresh ordinary
context; no mutation descriptor is reconstructed. The branch-switch court changes
the host's default world between calls and proves no effect/read on that branch.

Choose an internal borrowed request view for native materialization. Native
staging/recovery borrows the immutable typed intent, key and preconditions;
retained observed-source loans use their existing Arc-backed charge. Do not add
an `Intent: Clone` requirement or reconstruct/clone the domain input per pump.
Preserve existing binding input Clone bounds and native governed-input ownership,
snapshots and accounting; this is not a promise of zero native clones.
Refactor the native staging view where today's single-use owned request takes
these fields. Both owned advanced requests and the borrowed ordinary view delegate
one native admission/staging implementation; no copied validation or schema.
The view is an internal borrowing seam over the actual installed binding/input;
it does not implement a short-lived public `ApplicationMutationIntent`, weaken
that trait's existing static bound or introduce a replacement public intent type.

### Compose Owners, Do Not Replicate Engines

Use move-only lane-specific fixed, discovered and source-only continuations. A
small sealed dispatch may select a lane, but a broad status union must not copy
native publication phases, output cursors, producer state or workflow state.
Progress is a descriptive view derived from the actual owner, not another source
of truth. Public constructors cannot manufacture native receipt or settlement
authority. Mark outstanding owners `must_use`; that warning supplements ownership
and typed method availability rather than serving as the only protection.

One `advance` opens one genuine native advancement phase. Immediate handoffs
delegate native internal seams within that phase, adding missing seams below. Existing
one-advance output traversal owns fan-out and parallel work. Return genuine pending
work when it cannot finish. Do not add application polling, a scheduler, an outer
retry cap, copied settlement attempt constants, nested public phase creation, or
a fresh/larger request to reset exhausted work or cancellation.

Delivery and close also use one native phase per attempt. Currentness and the
declared query/batch share that phase as well as the exact observation, through
native internal seams; separate public wrappers must not reset work or latched
stopping between check and read. Add a narrowly scoped internal delegation seam
where the current public currentness wrapper opens its own phase.

The required seam inventory is explicit: initial fixed/discovered performed
execution; fixed and discovered unpublished recovery; fixed and discovered
registered receipt recovery; retention retry visibility; currentness and declared
query/batch execution; workflow assessment advancement; and native cleanup/release
retry. Several current public wrappers open their own phase and have no callable
composition seam. Add owner-local crate-visible delegates and let the existing
public advanced wrappers open their phase and call those same delegates. Do not
return artificial Pending between immediate handoffs to avoid adding a seam.

Native recovery posture controls termination. Retryable refusal returns the
owning continuation; terminal native posture transfers custody into a terminal
outcome with no `advance` method. In particular, a retention failure with no
recoverable carrier is terminal source-committed/required-outputs-unattainable,
not endless BindingMismatch retry. Preserve receipt, any actually retained
handler result, original descriptor and cleanup. Terminal outcomes still support
lawful cleanup/close and their permitted historical delivery. No fitted retry cap
or caller classifier substitutes for native Retryable/Terminal meaning.

Classify fresh caller-context mismatches before delegating destructive terminal
handoffs. Wrong runtime/branch/program context returns typed refusal with unchanged
custody; a correct fresh context remains usable. A native denial labeled Terminal
for such a mismatch does not prove the underlying retained source or carrier is
unrecoverable. Authoritative terminal source/custody posture is distinct from an
invalid caller attempt; the terminal court must prove the former.

`AlreadyCommitted` composes registered receipt recovery for the original typed
action and root demand derived from the descriptor. Fixed and discovered adapters
use their existing lane-specific receipt recovery; native ownership still decides
whether the registration is available. Progress is recovered, never fresh handler
completion. `PreviouslyCommitted`, or registered recovery reporting a terminal
missing registration, transfers to terminal historical evidence retaining cleanup.
The facade must not leave callers to rebuild this ordinary replay continuation.

### Discovered Retention Is A Real Owning Entrance

Replace the ordinary discovered committed-retention failure's exposed tuple with
an opaque move-only owner. It retains the original native carrier, typed source
result, discovery, descriptor and any cleanup. Fresh checked retry consumes that
failure, promotes the original carrier using the existing native retention owner,
and returns the complete failure/remaining owner on refusal. Registered receipt
recovery remains a distinct entrance. Neither handler replay nor re-discovery may
reconstruct an unregistered source carrier.

### Delivery Meaning And Exact Currentness

Bind typed delivery meaning to the existing canonical action/program contract:
a typed query or bounded typed query batch, with selectors derived from original
semantic input and/or actual native settlements. It is installed and participates
in existing program meaning/version checks. There is no second delivery registry
or duplicated schema. A delivery declaration grants no authority.

An action binds one delivery contract in its canonical meaning; callers cannot
swap the root or query contract at runtime. An action which promises settlement
without a returned projection has an explicit typed no-projection delivery mode,
not a dummy query. Existing actions without delivery meaning remain valid on
their explicit advanced surfaces during cutover; they cannot claim the new
ordinary delivered-result contract. Changed delivery meaning follows the existing
program revision/adoption contract, with old installed occurrences retaining
their original interpretation. No ambient latest-delivery lookup is permitted.

The owning constructor selects a typed canonical action reference containing
composition instance, feature and binding. Installation resolves that complete
identity, including required root and delivery. The same binding may appear in
several instances; first-binding lookup cannot select an action or delivery.
Prove two instances using one binding and different lawful delivery contracts;
each typed action must deliver only its own contract. The reference is pure
meaning, not an authority grant or a new registry.
Carry that exact action identity through descriptor, installation lookup, source
execution/admission, recovery and delivery. Selecting delivery precisely while
source execution still first-matches the binding is forbidden; the two-instance
court must exercise different lawful action meaning as well as returned queries.

Use the existing query declarations and typed selection machinery. Pure selector
construction may use their existing authoring mechanisms; it must not become an
arbitrary runtime callback pipeline, caller readiness classifier or application
effect hook. Domain mapping of actual returned rows remains application code.

A sealed borrowed settled view exists only when the native required set is
settled. Its current delivery operation validates every relevant actual settlement
and executes the declared query/batch at the same authentic observation. For a
discovered aggregate, checking only a source row or one root is insufficient;
native superseded-root semantics remain authoritative. Add a narrowly owned
native aggregate check if the existing API cannot express the complete check.

Read refusal leaves settlement, descriptor and cleanup in the continuation.
The caller cannot accidentally turn the ordinary path into a bare `Result<T,
String>` that discards these owners. Current delivery exposes its exact basis
and native evidence. Historical delivery and recovered projection are distinct
from a freshly executed handler result. Do not reconstruct an original result
that native replay/recovery does not retain or prove.

Place delivery work in a typed per-attempt policy: the runtime host's admitted
profile supplies the ordinary default, including a finite currentness-work bound;
the caller may narrow it. The declared batch is bounded by its installed meaning
and actual native policy. No hard-coded facade constant or unlimited fallback
fills in the native `maximum_work`. After a retryable exhausted delivery, a
separate fresh attempt may change policy only within lawful native host admission.
It may not reset work/cancellation inside the failed attempt or revive a terminal
outcome. The finite-policy court covers both refusal and fresh lawful retry.

Stale currentness returns a typed stale-delivery outcome retaining the original
settlement, descriptor and cleanup. It permits explicitly historical delivery at
the retained basis and lawful close, but no output `advance` on the already closed
native handle. Refresh to a newly computed settlement is out of scope here;
applications may use a separate explicit native demand/assessment journey.
Repeated reads must not secretly open that demand, re-run the mutation or pump
a closed handle. A later authentic observation may yield current delivery only
when the original settlement passes the actual complete native currentness check.

### Publication, Settlement, Delivery And Cleanup Stay Distinct

Typed outcomes preserve: fresh handler completion, exact native replay,
historical receipt/evidence, source-only completion, settled current delivery,
pending work, fresh-admission refusal, domain rejection and native unresolved
publication. Preserve native blocked, settlement-deferred and indeterminate
postures; compose only recovery entrances that actually exist. Unsupported
recovery remains explicit and retains any live custody rather than masquerading
as no effect or completion.

Cleanup is an independent native obligation carried alongside those stages from
the first phase of implementation. A delivered result does not erase it. Explicit
close delegates native close/release contracts and returns typed failure with
remaining custody when incomplete. Drop performs only existing bounded local
releases; it cannot claim funded external cleanup. Dropping or deliberately
forgetting an unresolved owner is abandonment, not lawful completion. Rust cannot
force explicit close. Native independently registered obligations retain only
their existing recovery visibility; an unregistered carrier or wrapper-only
cleanup context has no promised recoverability after abandonment. Source effects
already performed are not rolled back. Document this boundary and test that drop
does not emit completion, cleanup success or a new publication. The guarantee is
custody preservation during retained ownership and all typed handoffs, not durable
custody after arbitrary drop. No new owner codec, crash
durability, aggregate quota or detached operation registry is a prerequisite.
Retained descriptor and delivery state obey existing native resource policies;
implementation must identify their actual admission/accounting before effectful
construction and must not hide unbounded cloning behind the facade.

Publication currently hands prior cleanup and recovery cleanup failure back to
callers; it has no complete ordinary cleanup retry facade. Phase A adds an owning
delegation entrance to the existing execution-level release/owner-cleanup retries.
It retains the real failed cleanup owner and uses the same fresh native phase,
without interpreting a cleanup status as permission or inventing a cleanup engine.

### Source-Only And Workflow Adapters

Source-only is available only for declaration-authorized source-only meaning.
Its source completion can be followed by an independent output demand or
assessment, using existing demand owners. It cannot bypass a required-output
contract or synthesize a dummy root. A required-output action takes its owning
required-output path by default.

Workflow operation and assessment adapters delegate the existing workflow
operation/recovery, transition authority and assessment owners. Assessment is
observation, not business approval. Existing ordinary workflow runs keep step
limits and stopping reasons. Do not remove generic `WorkflowUnsupported`, merge
workflow recovery into mutation recovery, or accept a caller boolean as transition
authority. Provider runs and validated computation completion remain their own
native contracts.

The immutable descriptor holds workflow identity, not a grant. The workflow
lane's native owner separately carries exactly the originally issued single-use
`WorkflowOperationAuthoritySlot` until native preparation consumes it.
It cannot clone/reissue the slot. Subsequent recovery delegates the existing
`for_workflow_operation_recovery` identity binding and original native custody;
it does not reacquire mutation authority from the descriptor. Prove pre-effect
interruption, consumption, refusal/recovery and rejection of reused transition
authority. Assessment delegates its own phase seam and remains observational.
Preserve the actual native take boundary before handler validation/outcome;
cancellation, domain denial or refusal after take cannot restore the grant.
Native idempotency replay precedes take and must retain that ordering.

## Destination Topology And Ownership

The following is a destination, not permission to create empty placeholders.
Create a file when its phase supplies the responsibility. Existing native owners
remain in place. Paths are relative to `workspaces/worth-query/crates/`.

```text
worth-query-declaration/src/application_program/
  action.rs                                existing canonical action meaning; revised to bind delivery
  action/                                  created canonical action responsibility family
    delivery.rs                            created: typed delivery meaning
    reference.rs                           created: complete typed action-instance meaning
  output_graph.rs                          existing required/discovered meaning
worth-query-installation/src/application_program/
  scoped_action.rs                          revised: exact typed instance/action resolution
worth-query-installation/src/application_installation/program/
  delivery.rs                              created: compile canonical delivery meaning
  output_roots.rs                          revised: bind delivery to actual installed root meaning
worth-query-publication/src/application_entry/
  request.rs                               revised: ordinary entry and explicit advanced selection
  operation/                               created: ordinary owning composition
    descriptor.rs                          immutable original typed meaning, no admission
    request_view.rs                        borrowed descriptor fields into native staging
    admission.rs                           fresh request/native identity admission
    continuation.rs                        sealed lane dispatch and owning public entry
    progress.rs                            derived, non-authoritative progress views
    outputs/
      fixed.rs                             thin adapter to performed/native output owners
      discovered.rs                        thin adapter to discovered/native output owners
      source_only.rs                       thin adapter to declaration-authorized source completion
    delivery/
      current_read.rs                      settlement + exact-observation query composition
      historical_read.rs                   explicitly historical delivery, distinct evidence
    cleanup.rs                             native obligation delegation and lawful close
    workflow/
      operation.rs                         existing workflow operation/recovery adapter
      assessment.rs                        separate observation/demand adapter
  mutation/discovered/
    retention.rs                           created: opaque failed-retention owner and retry
    recovery.rs                            existing registered receipt recovery
  mutation/performed*/                      existing publication/recovery/output authorities
  request/output_currentness.rs             revised only for required aggregate native checks
worth-query-execution/src/domain_computation/primary_graph/application_output_demand/
  ...                                      existing custody/settlement authority; minimal new seams only
worth-query/src/                            existing stable public facade; exports only
worth-query-certification/fixtures/consumer_entry/consumer_root/src/
  application_invariant_acceptance/proof/application_program/
    ...                                    revised real ordinary journeys and adversarial proofs
```

Stable meaning lives in Declaration and its existing installed form. Publication
owns cross-native application entry composition above participating owners;
Execution remains the custody/settlement authority. `operation/outputs` is not
another output graph evaluator. `delivery` holds observation/read composition,
not schema truth or authority grants. `cleanup` cannot decide publication success.
Workflow children separate business-operation authority from assessment.

Existing public Query facades expose constrained entry, owner, views and typed
outcomes. Native state fields and constructors stay private. Enforce dependency
direction and facade exports with the repository's existing topology controls;
do not use Foundation as a runtime bucket. Retention failure ownership belongs
beside discovered recovery, not in an application helper. Future private adoption
adds domain declarations and consumer adapters without relocating these owners.

Delete ordinary consumer request-rebuilding, source/prepared/started/settled status
coordination, settlement polling and receipt-based guesses once the corresponding
lane cuts over. Delete obsolete ordinary wrappers/exports where no lawful
advanced use remains. Preserve explicit advanced native controls and negative
certification courts; they must not become an alternative ordinary default.
Keep domain preflight, algorithms, view mapping and business decisions in their
domain. No production engine, workflow controller or managed run is deleted
merely because the operation facade can delegate it.

Make the ordinary/advanced distinction mechanically locatable: the existing
`worth-query-host` facade exports the owning entry as its ordinary action path;
the ordinary `WorthQueryApplicationRequest::mutate` returns an owning builder
which has no manual execute/recover/retention/start/settle transitions. The existing
manual mutation builder is reachable only through explicit `advanced()` selection.
Covered methods include `execute_performed`, unpublished recovery, fixed/discovered
registered receipt recovery, retention retry and manual output start/settlement.
Native owner methods needed by legitimate advanced consumers remain on those
advanced types. No transparent legacy aliases remain on the ordinary builder.
This type split is established in Phase A; every covered ordinary consumer cuts
over or is deleted in Phase D. Update
`tools/boundary-check/snapshots/facades.toml` for the reviewed exports and extend
the existing `tools/boundary-check/src/query_audience/` checks to deny those
covered legacy exports on the ordinary surface. Compile-fail public examples
prove the ordinary action entry cannot invoke the old manual transitions; real
advanced certification examples prove legitimate native control remains usable.
This is a facade/type boundary, not a new caller allowlist or a regex ban on
ordinary domain code.

## Caller DX Contract

Today's public courts coordinate the borrowed mutation request, unpublished or
retention failure, recovered source, output handle, settlement and current read.
The owning entrance must remove that coordination from ordinary application code.
The following is **proposed DX, not an existing or compiled API**. Names are
illustrative; the ownership and authority boundaries above are fixed decisions.

```rust,ignore
// Capture typed key, intent, preconditions and source expectation once.
// The declaration supplies RequiredDelivery; no caller readiness callback.
let operation = original_entry
    .idempotency_owned(key)
    .retain_required_operation::<Program, TypedActionInstance>()?;

// fresh_request is the ordinary application context, with no repeated mutation
// key, intent, preconditions or source. Query binds it to the retained descriptor.
// Each call uses fresh authority. A real Pending returns control to the host.
match operation.advance(&program, &fresh_request, native_policy) {
    Advance::Pending(operation) | Advance::Retryable { operation, .. } => {
        host.retain(operation); // Pump later with fresh admission, not a poll loop.
    }
    Advance::Settled(mut settled) => {
        // Read refusal retains the actual settlement and cleanup in this owner.
        let delivery = settled.read_current(&program, &fresh_request, delivery_policy);
        match delivery {
            Delivery::Current(result) => {
                consume_current_result(result);
                retain_or_finish_close(settled.close(&program, &fresh_request));
            }
            Delivery::Retryable(cause) => host.retain_delivery(settled, cause),
            Delivery::Stale(evidence) => host.retain_historical(settled, evidence),
        }
    }
    Advance::Terminal(terminal) => {
        // Receipt/history/cleanup remain owned. Terminal has no advance method.
        retain_or_finish_close(terminal.close(&program, &fresh_request));
    }
}
```

The effect-free owning constructor may reject invalid declared meaning before
effects. Runtime admission remains fresh at advance. The final implementation
must specify borrow lifetimes and error ownership concretely, and publish compiled
fixed, discovered, source-only and workflow examples. A source-only example shows
a separate assessment demand; a workflow example binds original workflow authority.
Do not accept a demonstrated happy path which still requires a caller's custom
phase machine or a `wait_until_ready` loop to restore the guarantees.

## Ordered Phase Plan

### Phase A: Fixed Owning Journey And Exact Delivery

Establish descriptor ownership, canonical typed delivery meaning/installation,
fresh admission, the fixed adapter, independent cleanup custody and lawful close.
Establish the ordinary/advanced builder split, borrowed request view, exact typed
action selection, terminal native posture and registered fixed replay continuation.
Add the fixed phase seams and typed delivery policy. Stale delivery exposes
history/close and never advances a closed native output handle.
Run the existing real fixed consumer from initial request through partial source
publication, retention/promotion, output start, settlement and current delivery.
Refused delivery must retry without losing settlement. This first usable vertical
slice includes the fixed cleanup/refusal proof; cleanup cannot be deferred behind
a facade that claims completion. Compiler tests deny counterfeit settled views
and invalid source modes. Include the real nondefault-precondition/result-set
descriptor court and its omission/defaulting mutations in this phase. Further
required courts include fixed reply-loss replay, unavailable registration,
terminal carrier absence, branch switching, action-instance selection and finite
delivery policy. Each ordinary journey runs through the public owning entrance,
with existing advanced hostile courts and compile-fail checks as specified above. The next
phase may trust original identity ownership,
one-phase delegation and recoverable final delivery.

### Phase B: Discovered Retention And Complete Required-Set Delivery

Consume Phase A's descriptor/admission/delivery boundaries. Add the actual missing
opaque discovered committed-retention owner and retry entrance, then compose
existing discovered admission and traversal. Preserve original discovery and
already settled roots across refusal; certify complete-set currentness at delivery
with genuine fan-out and native supersession. The discovered partial-effect court
must prove promotion of original unregistered custody, not receipt reconstruction.
Add discovered native phase seams and registered replay, with historical/missing
registration terminal counterparts and no original handler-result fabrication.
The next phase may trust both required-output lanes with equivalent ownership.

### Phase C: Source-Only And Workflow Composition

Consume the required-output default and existing source-only/workflow contracts.
Implement declaration-constrained source-only completion and separate assessment,
then workflow operation and assessment adapters over their native authority.
Prove fresh admission, original workflow binding, honest stopping outcomes and
independent cleanup. Reject required-output bypass, observer-as-approval and generic
mutation recovery of workflows. No second workflow or managed-run state machine
may appear. Ordinary consumers can now select every supported semantic lane.

### Phase D: Consumer Deletion, Cost And Compatibility Closure

Cut over the real neutral public consumers and compiled documentation examples.
Record and complete a scoped deletion inventory of their lifecycle coordination
and false ordinary wrappers. Complete the shared cancellation/resource/cleanup
courts at all effect-bearing seams, the actual native unresolved postures and
existing checkpoint recovery compatibility. Prove bounded descriptor retention,
no unrelated-population scan and equivalent native work under the same policy.
Run facade/topology enforcement and the appropriate Query gate suites, including
the existing public invariant, recovery, workflow and output courts affected by
the change. Both correctness and structure reviews must clear the final joined
implementation before marking any phase or milestone complete.

## Documentation Deliverables

Revise the existing authoritative documents; implementation does not create a
competing milestone-shaped usage guide. Repository-relative paths:

| Audience and document | Required change and check |
| --- | --- |
| Application authors: `docs/build-an-application.md` | Default owned fixed/discovered/source-only/workflow journeys; typed delivery declaration; deletion of handwritten lifecycle coordination. Examples are compiled through the real public facade. |
| Framework readers: `docs/how-it-works.md` | Original descriptor versus fresh authority; publication, settlement, observation/delivery and cleanup; existing native engines and execution authority remain owners. Check against actual module/dependency boundaries. |
| Query authors and AI: `workspaces/worth-query/crates/worth-query/docs/AI_README.md` and `docs/foundations/programs-and-adoption.md` beneath that Query docs root | Safe ordinary entry, invalid transitions, meaning/version affinity and the advanced boundary. Compile examples and compile-fail misuse; remove false ordinary request-rebuild guidance. |
| Recovery users: Query `docs/execution/application-aftermath-and-recovery.md` | Owning fixed/discovered retention retry, registered receipt recovery distinction, delivery refusal and independent cleanup/close; unresolved publication and checkpoint limitations. Exercise every claimed recovery entrance. |
| Workflow users: Query `docs/domain-capabilities/workflow/README.md` | Operation versus assessment, original transition authority and stopping outcomes; compiled adapter example and negative authority proof. |
| Runtime hosts: Query `docs/domain-capabilities/execution-resource-admission-and-managed-runs.md` and `workspaces/worth-query/README.md` | Fresh admission, caller pumping, native pending and resource accounting; operation continuation versus provider managed run; no new polling constants or hidden budgets. Verify against existing native policies and the finite-policy courts. |
| Entry-band consumers: `workspaces/worth-query/crates/worth-query-host/README.md` | Default owning entry and exported typed outcomes, retained original intent, delivery refusal and close custody; correct the manual ordinary lifecycle instructions while retaining explicitly advanced native use. Compile public examples and run facade enforcement. |

## Must Ship, Must Preserve And Closure

Completion requires all of the following, not only new types or compile evidence:

- The ordinary public facade owns the immutable descriptor and every live native
  obligation for all supported lanes; fresh authority is required at every effect.
- Discovered committed-retention retry promotes original custody and returns it
  intact on refusal. Native registered receipt recovery remains distinct.
- Required delivery proves the complete actual required set and queries at one
  authentic observation; read refusal retains settlement and cleanup.
- Typed outcomes preserve publication uncertainty, historical/current distinction,
  source-only/workflow meaning and original-result versus recovered-projection truth.
- Lawful close returns remaining custody on failure; drop and successful delivery
  make no false cleanup or crash-durability claim.
- Real public consumers delete repeated request rebuilding and lifecycle control;
  domain algorithms and mappings remain domain-owned. The default documentation
  and facade cannot quietly steer callers back to the fragile ordinary path.
- The courts and mutation probes above pass through real native owners, with
  independent product observations. Existing exact invalidation, native parallel
  execution, finite admission and evolution/checkpoint guarantees stay green.
- Final joined correctness and structure review, scoped deletion/facade checks,
  compiled examples and affected Query gate suites all agree. No production claim
  is closed solely by this specification or a private adopter's compile result.

## Successor Handoff

Application adopters, including M2.7.1, consume the certified public revision and
pin their actual dependency to it. Their proof must use their real fixed/discovered
or source-only/workflow lane, delete redundant lifecycle coordination, and retain
domain intent, computation and presentation. A public neutral fixture establishes
the framework contract; it cannot certify those private journeys in advance.

Validated computation completion, domain correspondence, layout authority and
provider algorithms remain separate successor responsibilities. They may use this
owning entry without rebuilding its state machine. Later portable/distributed
execution plugs into 9.17.6.3's existing ports; it does not become a prerequisite
for this application API correction.
