# Milestone 9.17.6.4: Framework-Owned Operation Continuations

> **Status:** Planned. Immediate successor to the completed
> [9.17.6.3](./milestone-9.17.6.3.md). This milestone makes Query own the
> ordinary application's operation continuation over its existing publication,
> recovery, output and currentness owners. It does not introduce another
> execution engine. Implementation and certification remain outstanding.
>
> **Amended 2026-10-13.** The non-output lane decision and the workflow funnel
> decision are now closed (see [Decision: One Ordinary Retry
> Style](#decision-one-ordinary-retry-style)). [Phase 0: Truth
> Guards](#phase-0-truth-guards) is added: the unauthorized-executor court,
> authoring-time validation of guarded operation inputs, and CI enforcement that
> documentation examples are compiled or visibly illustrative land before Phase A;
> authorization of program adoption, which today admits any authenticated
> principal, lands beside Phase A and before Phase B. Deferred workflow-observation work belongs to
> [9.17.6.5](./milestone-9.17.6.5.md).

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
settlement, refused delivery and successful retry. The source-only (no-output)
lane lands beside it in Phase A (see [Decision: One Ordinary Retry
Style](#decision-one-ordinary-retry-style)). Discovered outputs, workflow
adapters, complete cleanup proof and public consumer cutover remain required scope. Private application adoption is a successor obligation
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
this milestone must not rebuild them. The initial review at `3129c8b7fb`
predated upstream `9bd4509e99`; implementation begins from the final joined and
certified public revision, with these additions preserved.

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
| Stale/current distinction | Advance an unrelated domain and then a consumed fact between settlement and delivery; also request a lawful historical observation. | Unrelated progress follows native exact invalidation. A stale required settlement cannot yield a current result. A historical read is explicitly historical; a newer current delivery identifies its actual basis and does not counterfeit the original handler result. |
| Independent cleanup | Leave a real native cleanup obligation while source/output work succeeds; refuse cleanup admission, then retry or explicitly close. | Delivery and cleanup have separate outcomes. Refusal retains remaining cleanup custody. Neither drop nor a delivered result claims external cleanup or durability. |
| Pending and stop boundaries | Use actual native deferred output work, cancellation and exhausted admission at each effect-bearing handoff. | One advancement respects its single native phase and latched stop. Pending remains pumpable; interrupted/refused states preserve every live obligation. No hidden polling, replacement request or larger budget evades the boundary. |
| Source and workflow distinctions | Execute a lawful source-only save and independent assessment; exercise a workflow operation recovery with its original required transition authority. | Source-only completion cannot stand in for required output completion. Assessment cannot authorize workflow business progression. Generic mutation recovery cannot erase `WorkflowUnsupported`. |
| Source-only lane, one retry style | Run a plain (no-output) program action and a capability-bound one through the owning operation. Interrupt each at the real append fault after the source effect; lose a reply after a genuine commit; hold a pinned source that moves before retry. | Unpublished partials recover internally without a second source effect, capability-bound recovery goes through capability admission, a lost reply resolves through native idempotency, and the moved pinned source is terminal. No step requires the caller to rebuild a key, intent, preconditions or source, and the ordinary builder offers no method that would let it. |
| Approval revoked before first advance | Bind a workflow operation owner to issued authority whose installed requirement or authored `approval_authority` edge needs an approval. Revoke the approver's real grant after the owner is created and before its first effectful advance. | The first advance is denied by the existing approval recheck before any business effect or node settlement. The owner neither reissues nor clones the slot, and the retained descriptor cannot restore it. Continuing requires fresh approval and a fresh workflow advance through existing owners. Product history independently shows no effect. |
| Clean cutover | Compile a consumer against a retired entry/type, try an obsolete export through `advanced()`, and reconstruct a definition or checkpoint whose affected contract is unsupported. Exercise current-contract counterparts. | Retired APIs do not compile on any public surface. Unsupported contracts receive typed refusal before effects or executable readmission, with original bytes preserved. Current-contract execution and recovery succeed without a legacy decoder, shim, conversion or fallback. |

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

### Clean Cutover: No Legacy Or Backward Compatibility

The product decision for this milestone is **no legacy support and no backward
compatibility**. The supported window for affected APIs, declarations, installed
meaning and persisted contracts is the current contract only. This explicit
decision governs this milestone over generic compatibility or deprecation
guidance; it does not expand the task into unrelated formats or subsystems.

- Breaking changes are permitted. Update all covered consumers, fixtures,
  examples and documentation to the destination contract in the phase that
  replaces their lane, and delete the displaced implementation and exports.
- Do not retain deprecated aliases, forwarding wrappers, compatibility overloads,
  legacy modules, dual implementations, old-format decoders, version fallback,
  automatic conversion, compatibility flags or a deprecation grace period.
  Keeping an old name or path working is not an acceptance requirement.
- `advanced()` exposes deliberate current native control. It cannot shelter
  retired APIs, reproduce the old ordinary lifecycle as a compatibility facade,
  or re-export an obsolete wrapper. Each retained advanced entrance must have a
  present native responsibility and current-contract evidence independent of
  preserving old callers.
- An incompatible predecessor definition, installation or checkpoint is refused
  through a typed validation or unsupported-contract outcome before effects or
  executable readmission. Preserve its original bytes and truth; never silently
  reinterpret, erase, reinitialize, downgrade or migrate it. No cross-version
  restore, legacy-definition coexistence or historical format support is
  promised by this milestone.
- Current-contract recovery, idempotency, historical evidence, exact observation,
  native custody and cleanup remain required. They describe lawful runtime truth
  and failure handling within the supported contract, not backward compatibility.
- Phase ordering permits a lane's existing implementation only until its owning
  replacement lands. That phase performs the consumer cutover and deletion;
  phase and milestone closure cannot leave both generations supported.

Enforce the decision through a scoped deletion inventory, facade/topology checks,
compile-fail cases for retired imports and methods on ordinary and advanced
surfaces, and real unsupported-contract refusal with current-contract recovery
counterparts. Reintroducing a retired alias or fallback must make the relevant
enforcement fail. No compatibility branch earns an exemption by passing an old
test; update or remove the obsolete test instead.

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
current program revision/adoption and fresh admission rules, with unsupported
contracts refused under the clean-cutover decision, never a caller-selected
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
Refactor the native staging view where today's single-use owned request takes
these fields. Both owned advanced requests and the borrowed ordinary view delegate
one native admission/staging implementation; no copied validation or schema.

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
not a dummy query. Covered actions adopt the current delivery declaration in
their cutover phase; no legacy default or advanced compatibility path supplies
missing delivery meaning. Changed delivery meaning follows the current program
revision/adoption contract. Unsupported predecessor meaning is refused rather
than translated; retained historical evidence is never reinterpreted using the
new delivery contract. No ambient latest-delivery lookup is permitted.

The owning constructor selects a typed canonical action reference containing
composition instance, feature and binding. Installation resolves that complete
identity, including required root and delivery. The same binding may appear in
several instances; first-binding lookup cannot select an action or delivery.
Prove two instances using one binding and different lawful delivery contracts;
each typed action must deliver only its own contract. The reference is pure
meaning, not an authority grant or a new registry.

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

Source-only is available only for declaration-authorized source-only meaning: a
program action with no output root (see [Decision: One Ordinary Retry
Style](#decision-one-ordinary-retry-style)). Its source completion can be followed by an independent output demand or
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
`WorkflowOperationAuthoritySlot` until the first effectful advance consumes it.
It cannot clone/reissue the slot. Subsequent recovery delegates the existing
`for_workflow_operation_recovery` identity binding and original native custody;
it does not reacquire mutation authority from the descriptor. Prove pre-effect
interruption, consumption, refusal/recovery and rejection of reused transition
authority. Assessment delegates its own phase seam and remains observational.

### Decision: One Ordinary Retry Style

The ordinary API has exactly one way to run, retry and recover an operation: the
owning operation. No ordinary lane retries by having the caller rebuild a key,
intent, preconditions or source expectation. A plain mutation and an output
mutation must not teach two different recovery habits; that would be the second
way this milestone exists to remove.

A program action with no output root is the source-only lane of the owning
operation. The existing program declaration already authorizes it (the action is
owned and is not an output source, `program_owner/occurrence_commit.rs`), so no new
flag is added. Its descriptor is the same as every other lane's. Capability-bound
authorization follows the binding type. Keeping the commit observation is a
declared delivery mode, not a separate way to run.

On that lane, `advance` performs prepare-and-commit. It returns live replay as
recovered progress, and `PreviouslyCommitted` and intent drift as terminal
outcomes. On `Commit(ProductUnpublished)` it recovers the unpublished partial
internally, and on indeterminate or aborted publication it resolves the keyed
outcome internally. Native posture alone decides whether a no-effect failure is
retryable with the same descriptor or terminal. A retry after a stale source
reruns the handler with the original intent only for an unpinned source; a pinned
source that moved is terminal.

| Lane today | Placement |
| --- | --- |
| `execute_in_program`, `execute_in_program_report` | Ordinary: the source-only owning operation. Manual call: `advanced()` only. |
| `execute_capability_in_program` | Folded into the owning operation; capability authorization follows the binding. Manual call: `advanced()` only. |
| `execute_retained_in_program` | Owning operation with a declared retained-observation delivery. Manual call: `advanced()` only. |
| `recover_unpublished_in_program` | Performed internally by the owner. `advanced()` only. |
| `resolve_idempotency_in_program` | Performed internally by the owner for indeterminate, aborted and lost-reply outcomes. `advanced()` only. |
| `prepare_in_program` + `candidate.commit` | Deliberate two-phase native control with no owner equivalent. `advanced()` only. |
| `execute_performed*` and all output transitions | Owning operation (fixed and discovered lanes). `advanced()` only. |
| Non-program `execute`, `execute_retained`, `prepare_program_migration` | No typed action reference to own. `advanced()` only. |
| `for_workflow_operation(..).execute_in_program(..)` | The workflow operation adapter is ordinary; it binds through `for_workflow_operation(..)` on the owning builder and holds the issued slot until its first effectful advance. The native funnel stays the single execution path underneath and is reachable as advanced substrate. |

After process loss, the only ordinary path is to start the same descriptor again;
native idempotency routes it to replay, history or unpublished recovery.

**A runtime without a program is not an ordinary application configuration.**
The ordinary application declares and installs a program
(`in_memory_program` or `in_memory_rostered_program`); every journey in
`docs/build-an-application.md` does. A runtime installed with
`application_installation::in_memory` has no typed action reference for an owner
to hold, so its `execute`, `execute_retained` and `prepare_program_migration`
are advanced native control. They move behind `advanced()` in Phase A with the
two-phase `prepare_in_program` lane. In the same slice, the build guide (§0, §4,
§5.3 and §8), `docs/api.md` and the capability index stop presenting the
program-less runtime or its plain `execute` as an ordinary path and describe it
under an Advanced heading.

**No lane leaves the ordinary surface before its owner lands.** At no phase
boundary may an ordinary task require `advanced()`. Phase A therefore lands the
source-only (no-output) owning adapter together with the fixed adapter, and moves
those lanes' manual calls behind `advanced()` in the same slice. The
workflow-bound builder returned by `for_workflow_operation(..)` keeps its native
execute until Phase C replaces it with the adapter. From Phase A until Phase C,
Query `docs/foundations/workflows.md` states that this native funnel is still the
ordinary workflow execution path, that the workflow operation adapter replaces it
in Phase C, and that callers must not wrap it in their own continuation or retry
owner. The capability-bound `ProductUnpublished` case has no
ordinary recovery today (`recover_unpublished_in_program` authorizes only the
conventional way); the owner routes its recovery through capability admission,
and a Phase A court proves it.

`workspaces/worth-ui` (another team's root) calls `execute_retained_in_program`
in `worth-ui-query-binding/src/product_projection/application_action_owner.rs` and
`application_source_owner.rs` (plus the `status_integrity/lease_tests.rs` test).
Its cutover is coordinated with that team; until it lands, those calls use
`advanced()` explicitly, recorded as a tracked successor obligation, never as a
re-export on the ordinary builder.

Capability delegation and revocation commands
(`execute_capability_delegation_in_program` and its revocation sibling) retry by
re-issuing and matching `AlreadyCommitted`. They sit on the request, not on
`mutate()`, and are out of this milestone's scope. They are recorded as a
remaining retry style owned by [9.17.6.5](./milestone-9.17.6.5.md), item 2 of its
consumer guidance hardening, so they are tracked rather than silently a second way.

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
    source_only.rs                         thin adapter to the no-output program action lane
    outputs/
      fixed.rs                             thin adapter to performed/native output owners
      discovered.rs                        thin adapter to discovered/native output owners
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
lane cuts over. Delete displaced ordinary wrappers/exports. Preserve genuine
current advanced native controls with an independent native responsibility and
negative certification courts; do not preserve an obsolete wrapper merely by
moving it behind `advanced()`. These controls must not become an alternative
ordinary default.
Keep domain preflight, algorithms, view mapping and business decisions in their
domain. No production engine, workflow controller or managed run is deleted
merely because the operation facade can delegate it.

Make the ordinary/advanced distinction mechanically locatable: the existing
`worth-query-host` facade exports the owning entry as its ordinary action path;
the ordinary `WorthQueryApplicationRequest::mutate` returns an owning builder
which has no manual execute/recover/retention/start/settle transitions. The existing
manual mutation builder is reachable only through explicit `advanced()` selection.
Covered methods are every lane in [Decision: One Ordinary Retry
Style](#decision-one-ordinary-retry-style): the plain, capability and retained
program lanes, `execute_performed`, non-output and output unpublished recovery,
idempotency resolution, two-phase prepare, fixed/discovered registered receipt
recovery, retention retry and manual output start/settlement.
Native owner methods needed by legitimate advanced consumers remain on those
advanced types. No legacy aliases or compatibility wrappers remain on either
the ordinary builder or the advanced surface.
The ordinary/advanced type split is established in Phase A. Each lane's manual
calls move behind `advanced()` in the phase that lands its owner: fixed and
source-only in Phase A, discovered in Phase B, workflow in Phase C. Lanes with no
owner equivalent (two-phase `prepare_in_program` and the program-less runtime's
calls) move in Phase A. Every covered
ordinary consumer cuts over or is deleted by Phase D. Update
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
        consume_typed_delivery(delivery);
        retain_or_finish_close(settled.close(&program, &fresh_request));
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

### Phase 0: Truth Guards

Four corrections land before or beside the continuation work. Each closes a place
where the platform currently says something its code does not prove, lets a
consumer discover an error late, or admits an action without authority. None
depends on the owning operation. 0.1 to 0.3 are small and complete before Phase A
begins. 0.4 touches a different owner, may run beside Phase A, and must close
before Phase B begins.

**0.1 Permission to pump is not permission to execute.** The docs state that a
guarded workflow operation still needs ordinary operation authorization even
while the caller holds valid workflow occurrence authority. No test proves it: the
`76e1890143` courts in
`worth-query-certification/tests/application_graph/workflow_execution*.rs` run one
principal, and the retention fixture's `SetRetention` has no authorization policy.
Add the court in
`workspaces/worth-query-bank-world/crates/bank-server/tests/approved_payment_workflow/`,
where `ApprovePaymentMutationBinding` is guarded and requires the
`ApproveBusinessFunds` ability (new file, or `actor_handoff.rs` if it stays under
400 lines):

1. A second approver pumps the instance to `AwaitingOperation(operation)`; the
   starter's account authorization is revoked.
2. The revoked starter executes `operation` with the exact proposed input and a
   fresh key. Binding succeeds. The result is
   `OperationMutation(Authorization(d))` with `d.kind() == Refused(PermissionDenied)`;
   never `RequirementMismatch`, `WorkflowAuthoritySpent` or `Committed`.
3. No settlement rail attempt exists, the payment is not committed, and the
   instance still awaits the operation.
4. The second approver executes the same `operation` with a new key and commits
   once, proving the refused attempt spent no authority (authorization runs before
   the slot is taken in `worth-query-publication/src/application_entry/mutation/preparation.rs`).

Mutation probes: moving the authority take above authorization turns step 4 into
`WorkflowAuthoritySpent`; skipping operation authorization when workflow authority
is present must invalidate step 2's exact `Authorization(PermissionDenied)` outcome.
Bank's independent authenticated-actor handler may deny later; that later denial
does not satisfy the required ordinary authorization refusal.

**0.2 A guarded operation's input is validated when it is authored.** A guarded
Operation node needs exactly one `operation_input` source, but
`worth-query-declaration/src/application_program/workflow/validation/connections.rs`
does not check it. A definition missing it publishes, starts, and then every
advance into the node fails with `WorkflowTransitionAffinityMismatch`
(`workflow_transition_program/operation.rs`): a stuck instance discovered at run
time. Add `ApplicationWorkflowValidationDenialKind::MissingOperationInput`, raised
in the Operation arm after the authority check when a guarded node has zero or
more than one `operation_input`. Unguarded proposal slots are unconstrained.
Validate at authoring, ordinary publication and archive draft import, and enforce
the current contract at installed-definition reconstruction/readmission. Decoding
an older definition cannot bypass the newly required input shape. Definitions
that satisfy the current contract reconstruct with their authentic identity;
unsupported predecessor definitions are refused without conversion or rewriting
their stored bytes. Update
`worth-query-declaration/src/application_program/workflow/tests/validation.rs`
(`data_cannot_cross_a_mutually_exclusive_control_arm` adds the missing input),
the validation kinds in `docs/build-an-application.md` §6.3, the validation
minimums and `operation_input` row in Query `docs/foundations/workflows.md`.
Prove zero and two inputs are denied, every shipped guarded example still
validates, current-contract definitions reconstruct, and an authentic predecessor
missing the required input is refused before executable readmission.

**0.3 Documentation examples are compiled or visibly illustrative.** Of 125 Rust
fences in consumer documents, 4 are checked by CI (the `<!-- compiled -->` blocks of
`docs/build-an-application.md`), 43 are `ignore` (which readers cannot see), and 78
are plain (`rust`, or `rust,no_run` in 3 cases) and never compiled. CI runs no doctests, so the front door's compilable
blocks (it is the rustdoc of `bank-estate-certification`) are never compiled.
Drift of exactly this kind (a missing allocation argument, a node-kind count of six
instead of seven) reached consumers before 2026-10-13. Deliver:

- **Marker grammar**, on the line before each fence:
  `<!-- compiled: <path under topology_entry/src> -->` with a `rust` fence
  (unchanged); `<!-- doctest: <crate> -->` with a `rust` or `rust,no_run` fence,
  valid only when that crate includes the document and CI doctests it; and
  `<!-- illustrative -->` with a `rust,ignore` fence whose first line is
  `// Illustrative: not compiled.` so the label is visible when rendered.
- **Checker** in `tools/boundary-check` (a `consumer_docs` module; files under 400
  lines; no Cargo features), configured by an exact document list in
  `tools/boundary-check/config/consumer-docs.toml`. New codes: `BC8006` an
  unclassified fence count exceeds its debt; `BC8007` debt is stale (count fell,
  lower it); `BC8008` an invalid marker (missing compiled source or region, a
  doctest crate that does not include the document, an illustrative fence without
  `ignore` or caption, or a compiled fence marked `ignore`); `BC8009` a broken
  relative link or `#anchor` under GitHub heading slugs. Each code has a failing
  fixture test.
- **Shrink-only debt** in `tools/boundary-check/snapshots/consumer-doc-fence-debt.toml`,
  one row per document, seeded with today's unclassified count. It may only fall.
- **Generalized drift test**: `compiled_example_drift.rs` iterates the configured
  documents; its pinned marker list equals the set of `compiled:` markers across
  them.
- **CI**: a doctest job running `cargo test --doc` for the Query root
  (`worth-query-host`, `worth-query-decl`, `worth-query`, `worth-query-publication`,
  `worth-query-execution`, `worth-query-installation`) and the Bank root
  (`bank-estate-certification`), mirrored in `scripts/ci/query_gate.py`'s `doc`
  suite. One build per root is preserved because each manifest is its own root.

**0.4 Program adoption is authorized.** `WorthQueryApplicationProgramsRequest`
(`worth-query-publication/src/application_entry/programs/request.rs`) and
`WorthQueryApplicationBranchSetProgramsRequest` (`programs/branch_set.rs`) store
the authenticated principal as `_principal` and never use it. The code defers
"principal-specific adoption authorization" to a "Batch 3 custody" that no plan
owns. Today any authenticated principal holding a request scope can `compare`,
`adopt`, `prepare` and publish a new program revision on a branch, including
branch-set adoption. For any regulated consumer that means changing the governing
program without authority. Close it by composition:

- The program declares the capability that authorizes adopting a program revision
  on a branch. It is admitted through the existing capability admission with the
  request's principal and scope, on every effectful programs entrance
  (`adopt`/`prepare`/publication, single branch and branch set), fresh at each
  phase, and rechecked at publication like any other admitted operation.
- No new permission system, role table, adoption allowlist or host-side check.
  No default that admits every principal: a program that declares no adoption
  capability cannot be adopted through the ordinary programs request, and the
  denial says so.
- Read-only `compare` is gated by the same capability or a declared read
  capability; it must not disclose instance inventories (see
  [9.17.6.5](./milestone-9.17.6.5.md), which gates the remaining inventory
  routes).
- Courts: an authenticated principal without the capability is refused at
  `adopt`, `prepare` and publication with a typed authorization denial and no
  effect, on a single branch and a branch set; a principal whose grant is revoked
  between `prepare` and publication is refused at publication; an authorized
  principal adopts exactly as today. Mutation probes: removing the admission call
  from any entrance turns its court red.
- Update the build guide §7, Query `docs/foundations/programs-and-adoption.md`
  and the capability index in the same slice.

Phase 0 closes when the court and its probes pass, the new validation kind and its
current-contract reconstruction and unsupported-predecessor refusal proofs pass,
the checker and doctest job are green in CI with zero
broken links, every consumer document is listed in the checker configuration, and
(before Phase B begins) program adoption is authorized with its courts passing.

### Phase A: Fixed Owning Journey And Exact Delivery

Phase 0 is complete before Phase A begins. Apply [Decision: One Ordinary Retry
Style](#decision-one-ordinary-retry-style). Establish descriptor ownership,
canonical typed delivery meaning/installation, fresh admission, the fixed adapter,
the source-only (no-output) adapter, independent cleanup custody and lawful close.
The source-only adapter's courts land here: a real no-output
`Commit(ProductUnpublished)` recovered internally without a second source effect,
indeterminate/aborted publication resolved internally through native idempotency,
capability-bound unpublished recovery routed through capability admission, and a
stale pinned source returned as terminal. Cut over Bank's plain mutation consumers
(for example `bank-server` money movement) in this phase so no ordinary consumer is
left on a manual lane whose owner has landed.
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
descriptor court and its omission/defaulting mutations in this phase. The next
required courts include fixed reply-loss replay, unavailable registration,
terminal carrier absence, branch switching, action-instance selection and finite
delivery policy. All must run through the actual public owning entrance. The next
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

Consume the required-output default, Phase A's source-only lane and the existing
workflow contracts. Compose source-only completion with a separate assessment
demand, then implement the workflow operation and assessment adapters over their
native authority. Move the workflow-bound builder's native execute behind
`advanced()` in this slice and remove the interim workflow notice.
Prove fresh admission, original workflow binding, honest stopping outcomes and
independent cleanup. Reject required-output bypass, observer-as-approval and generic
mutation recovery of workflows. No second workflow or managed-run state machine
may appear. The same slice updates the workflow operation sections of Query
`docs/AI_README.md` and `docs/foundations/workflows.md`, per the workflow
coordination below. Ordinary consumers can now select every supported semantic lane.

### Phase D: Consumer Deletion, Cost And Clean-Cutover Closure

Cut over the real neutral public consumers and compiled documentation examples.
Record and complete a scoped deletion inventory of their lifecycle coordination
and false ordinary wrappers. Complete the shared cancellation/resource/cleanup
courts at all effect-bearing seams, the actual native unresolved postures and
current-contract checkpoint recovery and unsupported-contract refusal. Complete
the deletion inventory on ordinary and advanced surfaces; no legacy support or
backward-compatibility path survives. Prove bounded descriptor retention,
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
| Application authors: Query `docs/foundations/ordinary-application-front-door.md` | Its "Preparing a program action before publication" section and "Branch-resolved mutation lanes" table present `prepare_in_program`, `execute_retained_in_program` and `execute_performed` as ordinary lanes. Reclassify each covered lane per [Decision: One Ordinary Retry Style](#decision-one-ordinary-retry-style); the owning entry becomes the ordinary path for required-output and source-only actions. |
| Workflow users: Query `docs/foundations/workflows.md` | The application workflow guide. Its "Operations" section ("Execute through the same funnel") documents `for_workflow_operation(..).execute_in_program(..)`. Phase A adds the interim funnel notice required by the decision; Phase C updates it per the workflow coordination below. The existing `docs/domain-capabilities/workflow/README.md` row targets an internal-engine (`WorthQueryWorkspace`) page; application workflow users read this guide. |
| Crate selection: `docs/api.md` | §3.2 names the host facade's ordinary entry points. Name the owning entry as ordinary and list covered manual lanes only as advanced. |
| AI consumers: `docs/capability-index.md` | The task index rows for running a mutation, mutations with required outputs and retries. Point each at the owning entry once its lane lands; keep the "Never build" column. Add a row for any new capability this milestone ships. |

Do not revise Query `docs/domain-capabilities/installed-operation-reexecution-and-replay.md`
for this milestone. It documents re-execution of installed domain operations on
the internal engine (`WorthQueryWorkspace`) and certification replay. It is not
the application mutation retry, idempotent replay or operation continuation this
milestone changes. Where a revised document mentions replay, distinguish the two.

**Documentation lands per phase.** Each phase updates the documents for the
lanes it cuts over in the same slice; Phase D does not batch them. A lane's
manual calls move behind `advanced()` in the phase that lands its owner, so its
manual lifecycle sections move under the Advanced heading in that same slice and
its ordinary sections teach the owning entry. A manual section that is still the
ordinary path because its owner has not landed (the workflow funnel from Phase A
until Phase C) carries a short interim notice naming the owning entry that will
replace it and the phase that lands it. Remove each notice when its section
moves under the Advanced heading.

**Non-output lanes.** Closed by [Decision: One Ordinary Retry
Style](#decision-one-ordinary-retry-style). Every non-output lane leaves the
ordinary builder; a program action with no output root is the source-only lane of
the owning operation.

**Advanced entrances.** Documents present a manual entrance only under an
explicit "Advanced" heading. Each such section opens by linking back to the
ordinary entry it is an alternative to. A manual entrance never appears in an
ordinary journey, quick-start or task table without that label.

**Compiled examples need a mechanism.** "Compile examples" means a check that
fails CI: a block bound by a `<!-- compiled: X -->` marker to fixture source, or
a `<!-- doctest: crate -->` block that CI runs. [Phase 0](#phase-0-truth-guards)
delivers that mechanism for every consumer document before Phase A. A
`rust,ignore` block, a shape excerpt or reviewer reading does not satisfy a
compile requirement in the table above. Each phase names the mechanism for every
example it claims compiled and extends the marker list or doctests accordingly,
and lowers the fence debt for every document it touches.

**Workflow coordination.** `plans/WORTH-query/workflow-operation-execution.md`
keeps `for_workflow_operation(..).execute_in_program(..)` as the single execution
funnel. Closed by [Decision: One Ordinary Retry
Style](#decision-one-ordinary-retry-style): the workflow operation adapter is the
documented ordinary path, and the native funnel is the advanced substrate it
delegates to. The funnel's admission, occurrence authority and approval rechecks
remain the native owners. Phase C updates the operation sections of Query
`docs/AI_README.md` and `docs/foundations/workflows.md` in the same slice as the
adapter, and adds the approval-revocation court above.

## Must Ship, Must Preserve And Closure

Completion requires all of the following, not only new types or compile evidence:

- The ordinary public facade owns the immutable descriptor and every live native
  obligation for all supported lanes; fresh authority is required at every effect.
- The clean-cutover decision holds: covered consumers use the current contract,
  displaced APIs and implementations are deleted, and no aliases, shims, legacy
  decoders or compatibility paths survive on ordinary or advanced surfaces.
  Unsupported predecessor contracts refuse before effects/readmission without
  rewriting original bytes; current-contract recovery remains proven.
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
  independent product observations. Exact invalidation, native parallel execution,
  finite admission and current-contract evolution/checkpoint guarantees stay green.
- Every document in the Documentation Deliverables table, including the front
  door, workflows guide, API map and capability index, teaches the owning entry
  as the ordinary path for every lane the recorded decisions place under it.
  Its documents landed with their phase, and no interim
  notice remains. Manual entrances appear only under an explicit Advanced
  heading that links back to the ordinary entry.
- The ordinary builder, facade snapshot and documents match [Decision: One
  Ordinary Retry Style](#decision-one-ordinary-retry-style): every lane in its
  table sits where it says, the ordinary surface exposes no rebuild-the-request
  recovery for any lane, and at no phase boundary did an ordinary task require
  `advanced()`. The workflow operation adapter is documented as ordinary and the
  native funnel as its advanced substrate.
- Phase 0 holds: the unauthorized-executor court and its probes pass,
  `MissingOperationInput` is enforced at authoring and reconstruction/readmission
  with current-contract success and unsupported-predecessor refusal proofs,
  the consumer-doc checker and doctest job run in CI, and every effectful programs
  entrance admits the declared adoption capability with its courts passing.
- Every example the documents call compiled is checked in CI by a marker-bound
  fixture block or a doctest; every other Rust fence in a consumer document is
  visibly illustrative or counted in the shrink-only debt, and the debt is no
  higher than at Phase 0 for any document; the approval-revocation court passes.
- The `worth-ui` cutover and the capability delegation/revocation retry style are
  recorded as successor obligations, with the latter owned by
  [9.17.6.5](./milestone-9.17.6.5.md).
- Final joined correctness and structure review, scoped deletion/facade checks,
  compiled examples and affected Query gate suites all agree. No production claim
  is closed solely by this specification or a private adopter's compile result.

## Successor Handoff

Application adopters, including M2.7.1, consume the certified public revision and
pin their actual dependency to it. Their proof must use their real fixed/discovered
or source-only/workflow lane, delete redundant lifecycle coordination, and retain
domain intent, computation and presentation. A public neutral fixture establishes
the framework contract; it cannot certify those private journeys in advance.

[9.17.6.5](./milestone-9.17.6.5.md) makes workflow truth observable: governed
instance discovery and re-acquisition, read-only observation at an exact basis,
admitted actor attribution and separation of duty over workflow actors, plus the
consumer guidance hardening deferred from the 2026-10-13 documentation review.
Its workflow reference example consumes this milestone's workflow adapter.

Validated computation completion, domain correspondence, layout authority and
provider algorithms remain separate successor responsibilities. They may use this
owning entry without rebuilding its state machine. Later portable/distributed
execution plugs into 9.17.6.3's existing ports; it does not become a prerequisite
for this application API correction.
