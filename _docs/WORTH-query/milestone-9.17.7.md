# Milestone 9.17.7: Inbound Occurrences And External Effect Completion

> **Status:** Planned successor to [9.17.6](./milestone-9.17.6.md).
> 9.17.6 supplies the complete dynamic workflow kernel but deliberately exposes no
> callback, resume-message or inbound-completion API.

## Goal And Boundary

An authenticated external message becomes one bounded immutable inbound occurrence,
correlates to one committed dispatch effect, and is consumed exactly once by the
external-effect aftermath owner through World publication. A workflow awaiting that
effect progresses only from the owner's performed completion posture. Raw payloads,
identifiers, transport acknowledgements, status fields and workflow commands carry no
completion authority.

This milestone owns inbound protocol meaning, authentication, admission, correlation,
bounded custody and consumption. Product/server adapters own HTTP, broker and rail
transport. Query's existing dispatch/outbox and application-aftermath external-effect
owners remain the sole effect identity, retry and completion authority. Signal wakes;
Store later owns restart-safe rediscovery. This milestone claims process-local
durability and does not serialize live authority.

The installed family is named `inbound_occurrence`. Existing 9.17.4 `external_input`
means pull-style provider input and is unchanged.

## Decisive Production Courts

### Static effect completion first

Through Bank's real process root, commit an ordinary outbound rail dispatch without a
workflow. Lose its synchronous response, then receive a valid callback. Admission
commits one application-scoped occurrence. Correlation finds the exact dispatch,
branch incarnation and external-owner identity. Consumption performs one compare-and-
set on the dispatch-effect posture and publishes it through World.

Race safe redispatch against callback consumption. Exactly one owner posture settles;
the losing path observes that result and cannot emit another logical effect. Deliver
duplicates, reordering, an unknown/foreign correlation, invalid authentication,
unsupported protocol/version, malformed/oversized payload, late delivery and
retention exhaustion. Deny before effect where possible; retain valid blocked truth.
Observe transport receipt, occurrence fact, correlation, external-owner posture and
World result independently.

Fork after the dispatch. Only its exact dispatching branch incarnation may consume the
occurrence. The fork may inspect the performed effect as history but receives typed
`ForeignDispatchOccurrence` if it attempts completion.

### Workflow integration second

Install `await_inbound` only after the static court passes. Its definition names an
installed inbound contract and the effect result it awaits; it cannot contain a URL or
caller-selected correlation target. The node is satisfied by the external owner's
performed posture arriving through the 9.17.4 typed publication return path, never by
reading or consuming the occurrence directly.

Lose a workflow step's synchronous response, consume its callback, drop projections
and handles, then rebuild and advance. Require one occurrence, one posture transition,
one lawful workflow successor and no redispatch. Duplicate/reordered/post-cancellation
callbacks cannot reopen a cancelled instance. A callback without a live authenticated
advance request may publish external truth and wake readiness, but executes no workflow
node.

## Authority And Lifecycle

Compiler-visible owner progression is:

~~~text
ReceivedInboundEnvelope
  -> AuthenticatedInboundEnvelope
  -> AdmittedInboundOccurrence
  -> CorrelatedInboundOccurrence
  -> ConsumedInboundOccurrence
  -> PerformedExternalEffectPosture
~~~

| Phase | Owner proof and limitation |
| --- | --- |
| Received | transport bytes plus installed protocol identity; no authentication |
| Authenticated | installed source/protocol proof over exact bytes/version; no admission |
| Admitted | performed immutable occurrence publication under application custody |
| Correlated | exact committed dispatch/effect, branch incarnation and remote identity |
| Consumed | owner-issued single-use compare-and-set result; not workflow authority |
| Performed posture | sole completion truth consumed by recovery and workflow eligibility |

No phase has public fields, `from_identity`, a proof codec or a deserialize-proof path.
Protocol identity and version are stable boundary artifacts; unsupported versions deny
before decoding domain payload or contacting World. Deduplication binds protocol,
authenticated source, external message identity and correlation family. Correlation
binds exact dispatch/effect identity and branch incarnation. Acknowledgement, remote
completion, occurrence admission, consumption and workflow advancement remain distinct.

Admission custody is application-scoped because the branch is not trusted until
correlation. Bounds cover envelope bytes, authentication/decoding work, admitted and
unconsumed occurrences, pending correlation, duplicate contacts, retention age and
cleanup work. Exhaustion never drops an admitted valid occurrence or converts missing
truth into completion. Closing the last observer does not erase mandatory custody.

The external-effect owner decides whether a remote-idempotency contract permits
redispatch. Callback or reconciliation evidence can propose completion but only the
owner's World-published posture establishes it. Recovery and `await_inbound` consume
that posture; neither maintains another completion/status table.

## Required Public Experience

~~~rust,ignore
let received = bank.inbound_rail().receive(envelope)?;
let authenticated = received.authenticate(&installed_source)?;
let admitted = application
    .admit_inbound(authenticated)
    .idempotency(message_key)
    .execute()?;
let outcome = application.consume_inbound(&admitted.occurrence()).execute();
// Match typed performed, duplicate, blocked, foreign, denied or unpublished/recovery.
~~~

Host entry returns typed received/authentication/admission outcomes without owning
message meaning. Consumption returns performed, already-consumed, blocked,
foreign-branch, denied, unpublished/recovery and indeterminate postures without
flattening external-owner outcomes. Workflow callers continue to use 9.17.6
`advance_workflow`; there is no resume-by-message API.

## Destination Topology

Paths are under `workspaces/worth-query/crates`; E existing, R extend, N new.

~~~text
worth-query-declaration/src/application_schema/inbound_occurrence/
  {binding,protocol,correlation}.rs                         N stable inbound meaning
worth-query-installation/src/application_schema/inbound_occurrence/
  {installed_contract,source_authentication}.rs             N installed implementation
worth-query-execution/src/domain_computation/application_aftermath/external_effect/
  inbound/{admission,correlation,consumption,custody}.rs     N completion beside owner
  {observation,dispatch,recovery_progression}.rs             E/R sole posture/retry owner
worth-query-publication/src/application_entry/inbound_occurrence/
  {receive,admission,consumption,outcome}.rs                 N public host/application entry
worth-query-certification/tests/application_graph/inbound_occurrence/
  {admission,correlation,consumption,workflow}.rs            N grouped public proof
workspaces/worth-query-bank-world/crates/bank-server/src/
  inbound_rail/                                             N transport adapter/composition
~~~

Inbound code never lives under `primary_graph/workflow`; workflow depends only on the
performed external-effect posture. Product adapters depend on host/publication facades,
not execution internals. Visibility and boundary checks prevent application-aftermath
from importing workflow progression and prevent transport adapters from minting owner
postures.

## Ordered Phases

### Phase 1: Static Bank occurrence and owner completion

Ship installed protocol/authentication, immutable bounded admission, exact correlation
and one-shot external-owner consumption through the real Bank process root. Prove the
lost-response/redispatch race, branch affinity, duplicates, hostile envelopes,
retention and cleanup before adding a workflow consumer.

### Phase 2: Workflow await and CAD counterpart

Add the `await_inbound` vocabulary and lower it only to performed-posture eligibility.
Complete the workflow reconstruction/cancellation court and one CAD solver-style
callback using its real product adapter. No product-specific runner or message router.

### Phase 3: Public and operational closure

Finish protocol codecs/compatibility, resource counters, discovery/diagnostics,
documentation and deletion of any experimental callback/resume/status substitute.
Run warning-free public/process tests and repository enforcement. No unfinished inbox
authority reaches 9.18.

## Acceptance And Handoff

The static Bank court must pass without workflow types. The workflow court must then
advance from the same external-owner posture rather than consuming an occurrence.
Mutation-sensitive proof removes or misroutes authentication, branch affinity,
deduplication, the owner compare-and-set, World publication or posture delivery and
turns a named court red. Count admission, World, wake, redispatch and consumption
contacts; duplicate volume is an independent scale axis.

Compile-fail forged phase construction with valid owner-issued counterparts. Runtime
tests use genuinely authenticated foreign/stale occurrences. In-memory interruption
does not claim restart durability; Store recovery remains future authority.

Extend existing aftermath/resource guides and Bank/CAD process references with
protocol, authentication, deduplication, correlation, consumption, recovery, cleanup
and process-local durability. No separate inbox journal or workflow guide.

[9.18](./milestone-9.18.md) receives completed workflow and inbound-effect custody.
Correction preserves exact performed external posture and cannot undo completion by
moving workflow status backward. Cross-runtime merge later reconciles occurrence and
effect provenance without duplicating dispatch or consumption.
