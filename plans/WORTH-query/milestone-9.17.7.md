# Milestone 9.17.7: Inbound Occurrences And External Effect Completion

> **Status:** Phases 1–3 are implemented and independently certified. Whole-
> milestone QA found follow-up defects and evidence gaps; the corrections and
> focused regressions are complete. Process-restart durability remains outside
> this milestone's process-local custody guarantee.
> Design grounded in Query, World, Bank and Proprietary boundaries.
> [9.17.4](./milestone-9.17.4.md) and
> [9.17.6](./milestone-9.17.6.md) supply publication delivery and workflow custody;
> they did not supply inbound completion at this milestone's start. New names
> and call shapes below specify the delivered destination.

## Goal And Roadmap Placement

Complete an already committed external effect from authenticated remote evidence,
through the existing external-effect aftermath owner and one World publication.
First prove Bank's static estate death notification, then an approved-payment
workflow consuming the same owner result. An occurrence is consumed at most once;
successful recovery exposes the one performed completion. Delivery is repeatable
and may remain blocked. A callback, transport ACK, status, receipt or workflow
command grants no completion authority.

Query owns installed inbound meaning, admission, exact correlation and custody.
The remote service owns whether its effect happened. World alone publishes product
currentness. Product adapters own transport; fresh workflow advance observes
owner completion. Signal retains its existing conditional-consumer role.
`inbound_occurrence` is the installed family; `external_input` remains pull-style
provider input. This slice serves completion of declared outbound effects, not
general webhooks or remote workflow RPC. [9.18](./milestone-9.18.md) consumes its
performed and unresolved custody; correction cannot rewind a remote effect.

The guarantee is **process-local retained custody**, not restart durability.
Store's later reconstruction integration must preserve canonical occurrence/effect
records and re-admit authority. Do not serialize live proofs or describe an
in-memory acknowledgement as durable delivery.

## Starting Boundary

Query paths below are relative to `workspaces/worth-query/crates/`.

| Existing owner | Current fact and required change |
| --- | --- |
| Execution `domain_computation/application_aftermath/external_effect/` | Owns outbox, correlation, causal events, dispatch and transport classification. Completed differs from acknowledged, lost response and timeout. Add inbound evidence and terminal consumption here. |
| Execution `primary_graph/provider/committed_dispatch_outbox.rs` and `application_runtime/external_dispatch_attempt.rs` | Observe exact committed outbox and admit runtime/World-affine physical attempts. Extend exact indexed lookup and retained provenance; a wire correlation remains a selector. |
| Execution `application_aftermath/{recovery_handle,recovery_progression}/` | Owns discovery, fresh retry authority and recovery transitions. Make synchronous completion, safe retry and inbound completion converge on one terminal effect record. Current receipt-local observations do not establish that convergence. |
| Publication `application_entry/workflow/operation/{owner,recovery}.rs` | `accept_operation_from_owner` resolves operation custody; recovery can redispatch. Extend acceptance to completed inbound owner results without requiring redispatch. |
| Repository `crates/worth-runtime-world/src/{publication,history,recovery}/` | Supplies `Performed`, `NoEffect`, `ProductUnpublished`, retained delivery and owner-effect custody. Reuse this engine. |

Bank installs `WorthQueryExternalEffectTransport` through
`BankIdentityRuntime::install_external_effect_transport` and reads outbox through
`observe_committed_dispatch_outbox`. Its existing process court exercises
`/v1/estate/notify-death`, response loss and safe retry. The separate
`bank-external-rail` independently decodes the notice and records its actual
consequence in `src/server/completed_effects.rs`, separately from dispatch status.
The authenticated callback sender and Bank callback endpoint do not exist yet;
building them is part of this milestone.

Proprietary's `worth-cad-entry/src/application/workflow/assessment.rs` declares
reviewed geometry with `no_external_effect`; House uses Query assessment demand.
Its `docs/house/query-platform.md` places deferred CPU completion inside the
managed producer lifecycle and forbids a second scheduler or callback publication.
Do not invent a remote CAD solver for this milestone. A future CAD remote effect
must identify its actual effect/adapter and prove this contract when adopted.
Local numerical completion does not inherit inbound authority. No Proprietary
implementation or dependency-pin change is required for this Bank capability.

## Adversarial Courts

A plausible false implementation trusts correlation text, marks a message consumed
before World performs, updates a private status and reruns the business operation
after response loss. The following journeys and owner court must convict it.
Variants share their world and independent observations; do not expand them into
a redundant Cartesian test matrix.

### Static Bank notice first

Extend `bank-courtroom/tests/transport_process_courtroom.rs` and its scenario
modules. Use `TransportProcessWorld`, the authenticated specialist user node,
actual HTTP adapter/server and separate rail process. Commit a lawful estate
death notice through `/v1/estate/notify-death` with `CommitThenLoseResponse`.
Independently read the rail's completed notice and verify estate, notice and
subject; ledger status alone does not prove the consequence.

1. The rail signs completion from its actual completed-effect owner and sends a
   separate HTTP request. Hold exact outbox visibility at its owner lookup
   boundary. Bank returns `RetryBeforeAcceptance`, no custody ACK and no
   completion; the sender retains the same message. Release the hold and retry:
   one immutable occurrence enters aftermath custody.
2. Lose the callback response after acceptance and redeliver the message 100
   times. Return the same occurrence/result without another terminal publication.
   Same message identity with different validly signed meaning returns
   `MessageIdentityConflict`; tampered bytes fail authentication before correlation.
3. Fork after dispatch. Use genuinely authenticated foreign-application and
   inherited-sibling dispatches as negative twins. Exact owner/branch-incarnation
   checks deny completion, while the source-branch twin performs. Equal-looking
   branch names/versions are insufficient.
4. Race freshly admitted safe retry against callback consumption. If completion
   wins before transport admission, retry adds zero contacts. If retry already
   left, it uses the same outbox bytes/correlation: at most one additional physical
   contact, exactly one remote logical effect and one local terminal completion.
   A delayed acknowledgement cannot downgrade completion. Unknown remains unknown
   until actual evidence; a losing attempt cannot fabricate success.
5. Deliver a duplicate after optional payload cleanup, an unsupported version and
   an authenticated unknown correlation. None opens another effect or leaks a
   foreign dispatch. Same-message delivery is duplicate while its signed acceptance
   window is open and expired afterward, never a new occurrence.

Observe independently: rail contacts/admissions/completed consequences, accepted
occurrence provenance, external-owner terminal state and World's completion commit.
HTTP success is not the completion oracle. Bypassing authentication, matching branch
text alone, acknowledging before custody, omitting deduplication or updating only
private status must fail the relevant continuation.

### Publication gap and bounded custody

Use the existing public `application_graph` world with real Query, Relational,
Signal and World owners and genuinely issued dispatches.

- Interrupt after acceptance but before preparation; drop observers. Existing
  owner discovery must still find the exact pending occurrence.
- Interrupt after the Relational effect but before product movement, including a
  competing head publication. Require `ProductUnpublished` with actual custody;
  the current product remains uncompleted. Resolve that work before readmission.
- Interrupt after World performs but before delivery/HTTP response. Drop caller
  handles and derived status/correlation indexes. Recover via existing custody and
  delivery, rebuild indexes in explicit bounded reconstruction, and obtain the same
  result without redispatch or another completion commit. Cancellation preserves
  these same boundary distinctions.
- Saturate the configured custody limit with lawful pending occurrences, then
  deliver one more. Reject before acceptance; never evict accepted truth or ACK
  nonexistent capacity. Settle one, reclaim it lawfully and admit the waiting twin.
  Payload age expiry and observer disposal cannot erase mandatory pending evidence.

Qualify at 10 and 1,000 unrelated dispatches with fixed selected effect and envelope
width; vary duplicate count from 1 to 100 separately. Assert the cost contract
below. Add narrow codec/transition tests only for behavior the journeys cannot
efficiently localize, including malformed/oversized envelope and a valid twin.
Skipping World, setting consumed early, dropping custody with a handle or scanning
the outbox must turn this court red. In-memory interruption is not restart proof.

### Workflow payment second

Use Bank's real approved-payment workflow and `ApprovedPaymentSettlementEffect`,
actual accounts, initiator/approver admission and rail. Lose the operation response,
then close the advancing request before callback delivery.

The callback publishes external completion and records owner readiness, but executes **zero
workflow transitions** without fresh authenticated advance. Revoke advance authority
and verify another callback cannot advance. Restore lawful authority, destroy the
progress projection and reopen: `advance_workflow` consumes exact owner settlement,
releases operation custody and enables one successor without rerunning the payment
or redispatch. Independent account/journal readback, rail effect counts and
authoritative transition history must agree.

Preserve 9.17.6's cancellation/migration/fork-continuation denial while operation
custody is unresolved. After settlement, cancel before the next effectful node;
late and duplicate completion cannot reopen it. A sibling instance or fork cannot
consume the result. Program adoption removing the ordinary operation retains exact
completion/recovery without re-enabling that API. Wake-to-execute, raw occurrence
as transition authority and lost settlement after projection rebuild must fail.
Do not repeat the static wire matrix here.

## Decision Lock

### Installed meaning and exact authority

One declaration binds a typed outbound effect to its inbound protocol, permitted
source identity, supported version, completion meaning and finite limits.
Installation resolves the verifier/decoder and existing aftermath owner. Missing
ownership or incompatible binding denies installation; no parallel registry or
runtime string-selected handler.

The original dispatch co-retains application/World owner, dispatching branch
incarnation, performed composite occurrence, operation/outbox identity, effect,
correlation family and installed remote-source/protocol binding. The current
correlation derivation includes branch text; it is not the full provenance check.
Extend outbox provenance/indexes where necessary. Old outboxes lacking this binding
return `InboundNotSupported` and keep existing recovery; no migration infers trust
from a URL or reinterprets old correlation bytes.

The first Bank completion confirms the exact dispatched payload. It carries no
independent remote result body such as a settlement reference; that meaning needs
its own declared protocol and effect contract before a later product adopts it.

Completion targets the current head of the original branch incarnation after
checking the dispatch belongs to its retained lineage. Unrelated intervening edits
are permitted. A reused branch name, fork, retired incarnation or missing provenance
cannot redirect completion. Retain the exact current Signal definition basis unless
an existing declared owner contract requires a successor. Historical inspection of
a fork does not transfer its parent's completion authority.

An installed source receives authority only to record completion of its exactly
matched effect. It cannot dispatch, compensate, select another branch or invoke a
workflow. Recording remote truth does not require the original user to remain
logged in or retain mutation permission. Redispatch and workflow effects still
require fresh user authority. Source retirement fences new admission but preserves
accepted recovery. Source revocation blocks new consumption with typed security
recovery and retained evidence; it cannot silently discard accepted truth.

### Compiler-visible progression

~~~text
bounded bytes -> public authenticated source evidence -> public exact dispatch correlation
  -> public admitted occurrence in owner custody -> private prepared completion
  -> World Performed | NoEffect | ProductUnpublished
  -> performed owner completion through existing publication delivery
~~~

| Product | Issuer, proof and limit |
| --- | --- |
| Received envelope | Adapter supplies bounded wire structure; no trust or effect permission. |
| Authenticated evidence | Installed verifier checks exact bytes, audience, source/key epoch and validity. Correlation consumes it; it cannot execute an effect. |
| Correlated occurrence | Query proves one actual performed dispatch with full matching provenance. Admission consumes it; IDs cannot construct it. |
| Admitted occurrence | Aftermath owner retains immutable evidence and reserved lifecycle capacity. A caller reference only locates it; custody remains with the owner. |
| Prepared completion | Query privately holds the exact incarnation commit guard while it prepares the current-basis Relational candidate and World publication lease. The prepared carrier cannot escape that guard or become a redispatch permit. |
| Performed completion | Constructed from World's performed carrier; consumption and terminal posture share one publication. Recovery/workflow consume its owner result. |

Public sealed phases stop at admitted custody. `Admitted::execute` privately
prepares and executes the World attempt under the same exact incarnation guard;
the common receive entry composes those phases. The existing prepared Relational
candidate and World publication lease carry the private completion authority.
Do not add a public prepared marker or a callback that could reenter the guarded
runtime. Reuse `worth-proof` legality vocabulary only if a concrete missing grant
is demonstrated; no completion-specific proof type is required here. Runtime issuance,
source binding, clocks, indexes, reservations and `Drop` stay with Query.
An independently created owner cannot satisfy the installed runtime's proof binding.
Portable protocols/versions/bounds reuse `worth-foundational`, already present in
the relevant manifests. No public proof constructors, deserialize-proof path or
identity-to-authority cast. Do not introduce a substrate registry.

### Acceptance, duplicates and ordering

There is no application inbox branch or separate World admission commit.
Application-scoped aftermath custody canonically retains accepted pending evidence.
Authentication and exact correlation precede acceptance. Temporarily unavailable
lookup returns `RetryBeforeAcceptance`; authoritative absence/foreign provenance
denies. Neither acknowledges custody. Thus a callback racing local outbox visibility
remains the rail sender's obligation. Receipt of bytes alone earns no ACK.

Acceptance reserves count, bytes, transition and recovery capacity atomically with
retaining the occurrence. Only then may Bank return `AcceptedPending`, including
when publication is blocked. Response loss resolves to that same record on retry.
Cancellation/observer loss cannot erase it. Admission never reruns the original
business handler.

Dedup identity combines application audience, protocol, authenticated source/key epoch
and source message identity. Immutable comparison includes every signed semantic field,
including validity cutoffs, version, exact correlation family/token and payload.
Retain its canonical meaning digest when optional payload is deleted; expiry cannot
be extended by replaying a changed envelope under the same identity.
Same key/meaning returns the same occurrence;
changed meaning returns `MessageIdentityConflict`. A different message ID for an
already completed effect returns `AlreadyCompleted` after exact meaning comparison,
without allocating an unbounded duplicate log or another terminal transition.
Conflicting evidence returns a typed conflict preserving existing truth.
Completion is monotonic; delayed acknowledgement cannot downgrade it.

### Publication and retry race

The completion candidate co-records occurrence consumption and the one terminal
external-effect posture in existing owner records. World movement is the product
linearization point. No separate consumed flag is committed first. A private
in-flight reservation is custody, never performed truth. Synchronous completion
and safe-retry completion must enter this same terminal owner, preserving existing
public outcomes and causal ladders; otherwise the race is not closed.

| Boundary reached | Required result |
| --- | --- |
| Before accepted custody | Denied/expired/overloaded/retry-before-acceptance; no custody ACK or owner effects. Sender retains the message. |
| Accepted, no owner effects | `AcceptedPending` and bounded rediscovery. Cancellation releases caller interest only. |
| Owner effects, no World movement | Existing `ProductUnpublished` custody; no completion or workflow eligibility. Resolve/clean exact work before readmitting retained evidence. |
| World performed, response/delivery missing | Discover existing result and finish its delivery; no republish or redispatch. |
| Already terminal | Duplicate/`AlreadyCompleted` with original provenance; no new completion effect. |
| Unavailable evidence/revoked source/conflicting report | Typed recovery posture; never guess success or retry the business operation automatically. |

World arbitrates head movement; the external owner arbitrates in-flight
dispatch/completion. Failed comparison never silently retargets a prepared attempt;
bounded readmission is explicit. Callback receipt cannot retract a retry already
on the network. Query promises no fresh logical effect identity; only the installed
rail idempotency contract promises one remote consequence under physical retry.
Unsupported remote idempotency denies safe redispatch while allowing authenticated
completion/reconciliation. No whole-runtime lock is held across I/O.

## Transport, Lifecycle And Cost

Add bounded `POST /v1/inbound/rail-completions` to `bank-http-adapter`.
The principal is the installed rail source, not a user-node OIDC session. It grants
no Bank business-operation capability. Destination and signing configuration come
from process installation, never a message/workflow URL.

The first Bank contract uses an installed Ed25519 verification key and separate
rail-held signing key. A versioned unambiguous encoding signs application audience,
source/key epoch, message ID, issue/expiry times, protocol identity/version,
correlation family/token and exact payload. Use a maintained crypto implementation;
no custom algorithm. Rail encoding and Bank decoding independently implement the
documented wire contract. Unsupported versions, wrong audience, excessive size or
bad signature deny before domain decoding, retained allocation or World contact.
Never retain credentials or expose protected payload through unrestricted diagnostics.

The rail creates a stable completion message only from its completed-effect owner
and retains it until an authenticated Bank custody ACK, permanent denial or declared
deadline. Ambiguous loss and retry-before-acceptance resend the same bytes/ID using
bounded attempts, backoff and concurrency. Exhaustion leaves an observable unresolved
sender obligation for explicit reconciliation, not silent delivery or a new ID.
Authenticate the ACK by binding it to the installed Bank peer (HTTPS with pinned
deployment trust outside the loopback process court); an unauthenticated HTTP success
cannot release sender custody. Shutdown exposes outstanding delivery. Test controls
may delay/reorder the real sender but cannot mint completion through a special route.

Finite installed configuration bounds envelope/payload bytes, verifier work,
outstanding dispatch provenance before any callback, accepted count/bytes,
concurrent publication, discovery page/work, sender attempts/
deadline, replay window and cleanup work. The compiled example supplies a small
complete profile; all are admission ceilings, not advisory metrics.

Reserve outstanding provenance capacity before an effectful commit and carry it
through unpublished World custody. The original commit-basis lease stays with the
external-effect owner until settlement leaves enough canonical terminal provenance
for duplicate and recovery lookup. Accepted-occurrence limits begin later and
cannot authorize FIFO eviction of an unconfirmed dispatch.

- Reserve mandatory evidence until settlement or explicit owner-admitted retirement.
  Request deadlines and optional payload age do not expire pending custody.
  Optional deletion leaves typed omission. If policy cannot retain required
  evidence, deny acceptance or expose blocked retirement; never weaken it silently.
- Retain dedup evidence while its signed message remains acceptable. The installed
  clock enforces its cutoff with zero skew allowance; unavailable time denies
  new acceptance. After cutoff, replay is expired, so reclamation cannot reopen
  consumption. Previously accepted evidence remains recoverable beyond cutoff.
  Terminal truth remains in owner history.
- Branch/program retirement inventories pending obligations through existing owners.
  Observer close releases only interest. Runtime close fences new admission and
  exposes/drains bounded obligations. Forced process death has no survival promise.
- Ordinary exact lookup is O(log D) or better for D retained dispatch/occurrence
  records, touching one selected effect. Authentication is O(B) in bounded bytes B.
  No outbox/history/branch/workflow population scan or hidden index reconstruction.
  After terminal settlement, each duplicate has zero prepares, World publications,
  redispatches and workflow transitions, and adds no retained occurrence.
- Query exposes bytes verified, exact index probes, selected-record visits,
  custody bytes/reservations and completion prepares/publications. World retains
  its own publication contact/comparison counters; Bank's process court observes
  independent rail contacts and workflow transition receipts. These are evidence
  at their respective owners, not one synthetic transport-wide counter. At 10
  versus 1,000 unrelated **inbound-bound** dispatches, selected visits stay
  constant and exact probes obey the declared index bound. The indexed owner
  implementation supplies the O(log D) bound; a probe counter alone does not
  measure tree comparisons. Duplicate volume cannot grow retained state or
  completion publications. Cleanup obeys its supplied page/work budget;
  reconstruction is a separate lane.
- Reuse existing notification coalescing and resource accounting where a
  conditional observer is installed. The callback does not fan out to workflow
  transitions. No-effect ordinary operations acquire no retained inbound
  provenance; a denied empty program is a separate precommit case. No background
  polling or generic inbox scheduler.

## Public Experience And Workflow Integration

Declare through `worth-query-decl`; expose host progression through
`worth_query_host::facade::application_entry`. Both remain exports-only facades.
Bank wraps domain-named entry without exposing its application runtime.
The host installs one typed verifier/source contract; delivery cannot nominate a
trusted source or completion target. Compile this new call shape in Phase 1:

~~~rust,ignore
// Installed contract; scope carries deadline, cancellation and bounded work.
let outcome = application.inbound_occurrences()
    .receive(rail_completion_contract, bounded_envelope, &scope)
    .execute();
// Advanced path: application.authenticate_inbound_occurrence(handle, bytes)?
//     .correlate()?.accept()?.execute(&scope).
// Match denied/retry-before-acceptance, accepted-pending, performed,
// already-completed or retained unpublished/recovery posture.
~~~

The common call composes authentication, correlation, custody and one bounded
completion attempt. Advanced APIs expose sealed authenticated, correlated and
admitted phases plus existing
discovery/recovery controls. No caller-chosen idempotency key: identity is authenticated
protocol meaning. Wire responses are disclosed selectors/outcomes only; operator
inspection/recovery requires fresh authenticated entry and exact lookup.
Canonical envelopes carry protocol/boundary, actual commit/effect posture, mandatory
recovery and cost. Optional sidecars cannot omit facts needed to interpret custody.

After the static owner works, add `await_inbound` to the existing workflow vocabulary.
It binds a prior operation result's exact effect and installed inbound contract with
typed waiting/deadline behavior. It cannot name a URL, raw correlation, arbitrary
effect or another instance. Definition validation rejects absent/ambiguous origins
on all reachable branches. This is a wait on existing owner completion, not an
occurrence consumer or second scheduler.

The 9.17.4 performed return path updates owner observation/readiness. Fresh
`advance_workflow` accepts exact settlement through existing operation acceptance.
The callback runs no node. Completion also releases the predecessor's dispatch-pending
custody through ordinary fresh advance, so the wait does not become unreachable
behind an unresolved predecessor. No separate workflow terminal table, receipt
forwarding requirement or resume-by-message API.

## Destination Topology

The implementation follows these existing semantic owners. Q =
`workspaces/worth-query/crates`, B = `workspaces/worth-query-bank-world/crates`,
W = repository `crates`. New directories use exports-only private module facades.

~~~text
Q/worth-query-declaration/src/application_schema/inbound_occurrence/
  {binding,protocol,limits}.rs                             stable meaning
Q/worth-query-installation/src/application_schema/inbound_occurrence/
  {contract,validation}.rs                                 supported installation
Q/worth-query-execution/src/domain_computation/application_aftermath/
  external_effect/
    inbound/{verifier,claims,custody,terminal}.rs           authenticated meaning/custody
    {outbox,dispatch,observation,causal_event}.rs            sole effect owner
  {recovery_handle,recovery_progression}/                  existing lifecycle
  correction/                                             9.18 consumes posture
Q/worth-query-execution/src/domain_computation/primary_graph/
  provider/{dispatch_outbox,committed_dispatch_outbox}.rs   owner facts/indexed access
  application_runtime/inbound_occurrence/                   auth, correlation, progress
  application_runtime/inbound_publication/                  prepared World handoff
  application_runtime/external_dispatch_attempt.rs         dispatch/completion arbitration
  application_attempt/provider_execution/external_dispatch.rs terminal handoff
  workflow/{definition,instance}/                          wait meaning/owner settlement
Q/worth-query-publication/src/application_entry/
  inbound_occurrence.rs                                    public host progression
  workflow/operation/{owner,recovery}.rs                   existing acceptance
Q/worth-query-{decl,host}/src/facade.rs                    audience exports
Q/worth-query-execution/src/domain_computation/primary_graph/tests/
  inbound_admission/                                       focused owner/cost court
W/worth-runtime-world/src/{publication,history,recovery}/   existing owners
B/bank-domain/src/schema/                                 notice/payment declarations
B/bank-server/src/inbound_completion.rs                    product composition
B/bank-http-adapter/src/http/
  protocol/inbound_completion.rs                           wire contract
  server/inbound_completion/{authentication,route}.rs       verifier/HTTP boundary
B/bank-external-rail/src/
  completion_wire.rs                                       independent wire encoding
  server/completion_delivery/                              real consequence-to-callback
B/bank-courtroom/tests/transport_process_courtroom/
  {inbound_completion,payment}/                            grouped process journeys
B/bank-courtroom/examples/                                 compiled external consumer
~~~

Axes are stable meaning, installed support, live aftermath authority, publication
coordination and product transport. Installation owns immutable support; Query owns
clock/issuance/custody. Product verifiers are installed mechanisms, not grant issuers.
The prepared Relational candidate and World publication lease stay private
inside the exact owner transition. Public sealed phases stop at admission and
`execute` completes that transition; no inbound marker is added to `worth-proof`.

Inbound never imports workflow execution. Workflow consumes owner results through
the existing publication boundary. Provider storage implements records/access,
not completion policy. World remains cross-component publisher. Bank adapters
cannot mint Query postures. 9.18 adds correction beside external aftermath;
future remote adapters add product leaves and Store adds persistence under its
owner without moving facades. No flat inbox manager, callback map, duplicated
status facade or generic event framework.

Enforce private constructors, audience/tier dependencies, Query-agnostic schema,
`worth-*` not depending on `worthy-*`, and cert-only replay. Group valuable
compile-fail cases for forged phases/grants, occurrence-as-completion and completion-
as-workflow-permit with valid counterparts. Runtime foreign-owner twins cover
affinity not distinguishable by Rust types. No 400-line exemption is granted.

## Ordered Phases

### Phase 1: Static Bank completion

Build installed meaning/proof placement, exact provenance, indexed correlation,
bounded custody and the common terminal World path. Add Bank's endpoint and the
real rail sender. Complete the static journey and publication-gap court, including
sender retry/no-premature-ACK, partial effects, races and saturation. This is one
vertical capability: recovery/authentication cannot be deferred behind a happy-path
facade. Later phases trust exact performed/pending/unpublished meaning and compiled DX.

### Phase 2: Workflow consumption

Add validated `await_inbound` and lower to existing owner settlement/fresh transition
admission. Complete the payment journey, projection reconstruction, cancellation and
program-retirement continuations. Deny raw-message resume mechanically. Later work
may trust that external truth settles without a live user request while workflow
effects still require fresh authority. CAD remote-service adoption is not this gate.

### Phase 3: Resource and public closure

Complete bounded discovery/cleanup qualification, protocol fixtures, documentation
and affected constitutional checks. Remove experimental callback/status/duplicate
terminal paths introduced during implementation. Deliver complete custody to 9.18,
without a speculative service, persistent inbox or test-count target. Each phase
adds adversarial proof when its boundary first becomes real.

## Documentation, Acceptance And Handoff

Revise existing guides during implementation; no milestone journal or proof ledger.

| Audience | Authoritative document and obligation |
| --- | --- |
| Application authors/operators | Query `docs/execution/application-aftermath-and-recovery.md`: authority, correlation, duplicate/late outcomes, synchronous/callback convergence, recovery, expiry and process-local ACK. |
| Ordinary/workflow callers | Query `docs/foundations/ordinary-application-front-door.md` and `docs/foundations/workflows.md`: compiled host/wait examples, fresh advance, cancellation and recovery. |
| Runtime hosts | Query `docs/domain-capabilities/execution-resource-admission-and-managed-runs.md`: reservation ceilings, shutdown, backpressure, reconstruction and cost. |
| Bank maintainers/operators | Bank workspace `docs/process-transport.md` and `docs/public-consumer-contract.md`: wire/version/authentication setup, custody ACK/retry, replay window and sender exhaustion. |
| Platform maintainers | Query `docs/AI_README.md` and decl/host READMEs: owner locations and compiled examples; correct claims that receipt-local posture alone suffices. |

Query guide paths are relative to `workspaces/worth-query/crates/worth-query/`.
Check snippets against compiled public examples and transport examples against real
process journeys. Independent wire fixtures pin semantic bytes/version. The first
inbound compatibility window is v1 only; new versions require explicit coexistence/
retirement and negotiation, never reinterpreted v1 bytes or caller-selected downgrade.

Use existing grouped Query `application_graph`, public compile lanes and Bank
`transport_process_courtroom`; discover exact target names before expensive runs.
Focused semantic/codec/compile checks belong in CI. The process lane uses the real
identity/process harness with finite resources/deadlines. An unavailable environment
leaves that gate unexecuted; a mock cannot close it. Cost evidence uses the stated
small scales. Mutation probes target the disputed enforcement, not tests for tests.

Implementation runs affected owner/public/process tests, formatting, dirty line caps,
`cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` and
`cargo run --manifest-path tools/agent-context/Cargo.toml -- check`.
A design-only revision checks links, path/API claims and `git diff --check`;
it does not claim runtime proof.

QA rejects forged source grants, token-only correlation, premature ACK/consumption,
republishing after success, abandoned accepted truth, ordinary population scans,
workflow resurrection and diagnostic disclosure. Review independent observations
and lawful twins before increasing test volume. Preserve 9.17.4's typed delivery
and 9.17.6's operation-custody cancellation rules.

[9.18](./milestone-9.18.md) Phase 1 may proceed independently. Its external-effect
integration and final acceptance require this milestone's completed custody courts.
It receives immutable occurrence provenance, one terminal
effect owner, bounded unresolved recovery and performed workflow eligibility.
Correction preserves completed/unknown external truth and requires separately
admitted compensation/reconciliation. The
[cross-runtime roadmap](../cross-runtime/merging-and-branching-roadmap.md) later
reconciles provenance without duplicating effects. Durable sender/receiver recovery
and future real CAD remote-effect adoption have separate product gates; neither
is claimed by this process-local Bank milestone.
After forced process death, the local outstanding-dispatch provenance is not
reconstituted here. An authentic callback can therefore receive unknown
correlation rather than completion until a future durable owner rebinds that
provenance; it is not an authenticated permanent denial to the rail.
