# Milestone 9.16: Authenticated Async Bank World And The Ordinary Query Front Door

> **Current execution posture:** Runtime Hardening Phase 7 is closed through
> Phase 7.7 Gate D. Runtime Phase 8's accepted application-aftermath,
> external-effect, recovery, retention, and publication foundation is
> **closed through C8 (2026-08-12)** under the
> [Runtime Phase 8 finish plan](./milestone-9.16-runtime-phase-8-finish-plan.md); see the
> [Phase 8 closure ledger](./milestone-9.16-runtime-phase-8-closure-ledger.md)
> for closure evidence and the
> [feature guide](../../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md) for the
> supported developer surface. Historical gate labels remain evidence only.
> The present undo/redo implementation is provisional: it may remain in the
> tree, but its product semantics and final public contract belong to
> [Milestone 9.18](./milestone-9.18.md).
> [Milestone 9.16.1](./milestone-9.16.1.md) is closed, and its canonical
> graph-progression substrate remains inherited. Gates A-C and the executable
> release/disbursement slices remain historical prerequisites. Bank World
> Phase 5 is closed by the real Docker-backed, separate-process transport court
> (2026-08-13). Runtime Phase 9's host-installed conditional-operation path and
> Runtime Phase 10's public cutover are closed by their
> [Phase 9](./milestone-9.16-runtime-phase-9-closure-ledger.md) and
> [Phase 10](./milestone-9.16-runtime-phase-10-closure-ledger.md) ledgers.
> Milestone 9.16 itself remains open for Bank Phase 6 and Closure Phase 1.
> The independently governed
> [Milestone 9.16.2](./milestone-9.16.2.md) portable-package and fresh-
> readmission foundation must also close before 9.17.

## Goal

Prove that a small team can build a legitimate authenticated, multi-user,
asynchronous application through the ordinary Query API without reconstructing
runtime authority, writing stringly semantic adapters, or reaching into Query
internals.

The proving application is an in-memory bank and person-to-person payment world.
It has real users, personal, business, institution, branch, and estate scopes,
capability-constrained customer and employee authority, field-level disclosure,
double-entry monetary effects, deposits, withdrawals, transfers, approvals,
compensating reversals, provisional linear undo and redo experiments, concurrent requests, live
updates, and a real Authentik OIDC boundary. Its hostile authorization world
includes an employee whose institutional power conflicts with a personal
interest in a deceased relative's estate, plus a governed break-glass path that
cannot become superuser authority.

The application is not a showcase shell. It is the adversarial consumer that
defines whether Query has a front door.

## Roadmap Placement

Milestone 9.15 ends with an installed operation whose proposed state has passed
real invariants. Milestone 9.16 adds the authority and public composition needed
to make that work useful to an ordinary application:

```text
schema-bound typed intent
    -> authenticated principal
    -> installed capability and disclosure context
    -> scoped authorization or governed elevation
    -> admitted touched graph
    -> canonical application query / operation basis
    -> prepared and invariant-approved proposal
    -> provider compare-and-commit
    -> typed outcome
    -> ordinary read / mutation / workflow / history / live result
    -> governed recovery / compensation
    -> provisional undo / redo experimentation (not accepted product closure)
```

The independently governed Milestone 9.16.2 portable-package interstitial also
closes before Milestone 9.17. It establishes stable package identities, typed
decomposition, fresh reconstruction, and a neutral archive without adding
application-state persistence. The Milestone 9.17 sequence remains in memory:
9.17.1 adds exact component bases and Relational branch-local MVCC, corrective
9.17.1.1 closes Relational publication ports and shared lifecycle, 9.17.1.2
completes owner services and Signal independent progress, 9.17.2 adds
dedicated Runtime World composite history/publication, and 9.17.3 completes Query carriage and
performed-publication-gated live-runtime dispatch.
Milestone 9.18 accepts tree-based semantic undo/redo over that completed
composite history.
Advanced access and computation begin in Milestone 9.19 and must use the same
public front door.

## Work Types And Phase Identity

Milestone 9.16 contains three visibly different tracks. Each track has its own
ordinary phase sequence:

```text
Runtime Hardening Track   Phase 1 -> Phase 2 -> ... -> Phase N
Bank World Track          Phase 1 -> Phase 2 -> ... -> Phase N
Closure Track             Phase 1 -> Phase 2 -> ... -> Phase N
```

The Runtime Hardening Track owns generic Query product work:

- Runtime Phase 1 establishes schema-bound typed application references;
- Runtime Phase 2 establishes generic authenticated-principal admission;
- Runtime Phase 3 establishes installed permission and touched-graph authority;
- Runtime Phase 4 establishes provider compare-and-commit;
- Runtime Phase 5 establishes ordinary read, mutation, workflow, and live
  facades;
- Runtime Phase 6 establishes installed application queries, basis controls,
  Query-owned continuations, and one-shot/history/live/preview parity;
- Runtime Phase 7 establishes purpose-bound capabilities, field disclosure,
  conflict-of-interest, delegation, and break-glass authority;
- Runtime Phase 8 establishes the accepted aftermath, external-effect,
  recovery, retention, and publication foundation; its existing undo and redo
  lane remains provisional for Milestone 9.18;
- Runtime Phase 9 establishes host-installed conditional providers, managed
  clocks, Signal-owned temporal wakes, and reconstruction from authoritative
  domain truth; and
- Runtime Phase 10 performs public policy cutover and workaround deletion.

Bank phases consume and pressure those runtime capabilities:

- Bank Phase 1 freezes the product, courtroom, and gap ledger;
- Bank Phase 2 builds the Authentik adapter and dynamic identity fixture;
- Bank Phase 3 installs the banking domain, accounting, roles, and invariants;
- Bank Phase 4 adds estate administration, capability delegation,
  conflict-of-interest, emergency elevation, and aftermath semantics;
- Bank Phase 5 builds the temporary HTTP boundary and independent user nodes;
  and
- Bank Phase 6 proves the complete consumer journeys.

Closure Phase 1 runs hostile milestone certification and permanent
prohibitions. Track and phase together form the phase identity; the interleaved
order below describes causal execution across tracks.

## Discovery Intake And Phase Amendment Rule

The bank world is expected to discover missing behavior. Discovery is an input
to the milestone, not permission for an ad hoc fix inside whichever phase
happened to expose it.

Before implementing a discovered requirement, update the gap ledger and
classify it:

1. **A completed runtime guarantee needs stronger composition.** Add the next
   corrective Runtime phase or an interstitial milestone and block unfinished
   dependents on it. Do not reopen or rewrite completed milestone or ledger
   history.
2. **New generic Query behavior, API, authority, lifecycle, or performance
   capability is required.** Add the next appropriately sized phase to the
   Runtime Hardening Track before implementation.
3. **Bank-domain meaning or behavior is missing.** Add the next appropriate Bank
   World phase. Banking semantics must not be generalized into Query.
4. **Authentik, HTTP, process-fixture, or user-node mechanism is missing.** Add
   a Bank World adapter/fixture phase. Transport mechanics do not
   become runtime authority.
5. **The discovery changes public cutover, deletion, documentation, or the
   decisive courtroom.** Add the next Closure phase.
6. **The discovery has an independent advanced-computation purpose.** Assign it
   to Milestones 9.19 through 9.22 rather than expanding the bank milestone.

A new phase must be an appropriate vertical slice with one causal guarantee,
not a ticket-sized patch. It states what proof it consumes, what architecture it
establishes, what it mechanically forbids, what evidence closes it, and which
later bank or runtime phase it unblocks.

The unfinished bank or runtime phase that exposed a generic gap remains blocked
until the corrective phase or milestone closes. Previously completed rows remain
historical inputs rather than being relabeled. The application may not carry a
local workaround forward to keep the demo moving.

### Milestone 9.16.1 Interstitial Contract

Milestone 9.16.1 reconciles the graph-obligation, graph-read-planning, and
provider-session authority introduced across Milestones 9.9, 9.10, 9.11, 9.15,
and this milestone. Reconciliation is per semantic surface:

1. consume an earlier owner and proof unchanged when they already satisfy the
   stronger progression;
2. otherwise preserve the existing production path as sole authority while a
   destination successor proves feature, denial, lifecycle, receipt, and cost
   parity;
3. atomically cut covered consumers to the successor and retire only the exact
   predecessor authority in the same slice; and
4. record any broader architectural discovery as an explicit phase amendment
   or successor milestone rather than treating it as permission to decompose
   the rest of `worth-query`.

The absence of competing authority is the invariant. File count, crate age,
and monolith placement are not themselves defects. One-way lowering from the
ordinary Query facade into the sole destination authority is lawful and keeps
the Phase 6 caller contract stable. A second independently executable selector,
planner, session, invariant completion, or receipt path is not lawful.

Runtime Phase 7.3 consumes the 9.16.1 session and graph-plan authority while
retaining Phase 6 query identity, parameters, bases, continuations, history,
preview, live delivery, result shaping, and publication contracts. A 9.16.1
cutover is incomplete if the Query monolith, public declarative journeys,
Worth UI Query binding, or the Bank consumer loses covered behavior.

### Milestone 9.16.2 Interstitial Contract

Milestone 9.16.2 owns the portable-package gap discovered by Workflow Editor.
It consumes the validated package meaning built here and does not redefine Bank
behavior, outbox payload meaning, or Query/Relational publication authority.

Its boundary is exact: Query owns package records, reconstruction, validation,
and semantic identity; the neutral archive owns deterministic transfer bytes;
hosts own Git, signatures, expected release selection, providers, secrets, and
activation. Package records, archive bytes, checksums, signatures, filenames,
and repositories carry no installed or runtime authority. Application-state
durability remains Worth Store integration work.

Milestone 9.16 Bank Phase 6 and Closure Phase 1 may continue beside 9.16.2 when
their touched boundaries do not overlap. The roadmap handoff to 9.17.1 waits for
both milestones rather than hiding a Workflow Editor workaround inside the
Bank consumer.

## Why This Milestone Exists

Milestones 9.12 through 9.15 built substantial authority, lifecycle, workflow,
replay-certification, artifact, and pre-commit machinery. Dogfooding began with
complex consumers, so it was possible for those consumers to tolerate local
builders, raw aspect strings, implicit principals, handwritten permission
checks, or deep runtime knowledge.

That is the unfinished front face of the house. A real bank world makes the
missing fundamentals impossible to hide:

- a caller identity must come from an external authentication boundary;
- authorization must depend on current graph relationships and the exact graph
  the operation can touch;
- concurrent money movement must commit atomically or fail with an honest
  outcome;
- retries must not duplicate effects;
- reads and live updates must not disclose unauthorized accounts or postings;
- business authorization must support multiple users with different scopes;
- employee powers must be institution-scoped and distinct from customer powers;
- a role name must not silently grant every action, field, purpose, or resource
  inside its organizational scope;
- a principal whose employee authority conflicts with a personal interest must
  not gain power by combining otherwise valid relationships;
- emergency access must be time-, purpose-, field-, action-, and
  resource-bounded and must remain incapable of authorizing unrelated money
  movement;
- named application queries, their cursors, and their current, historical,
  live, and preview forms must retain one canonical meaning;
- indeterminate work must expose a lawful recovery action rather than a
  dead-end status;
- undo must preserve committed history through a declared inverse or
  compensation, and redo must be a freshly authorized execution rather than
  ordinary replay;
- application code must use schema-derived types instead of string aspect keys;
- transport adapters must translate rather than own policy or runtime
  authority; and
- several independent processes must communicate asynchronously over real
  network boundaries.

If implementing this world still requires local Query folklore, the milestone
remains open and the workaround becomes a Query requirement or an explicitly
owned non-Query concern.

## Adversarial Constraint

The fixture creates its bank, users, employees, accounts, role assignments, and
OIDC subjects dynamically. No application principal, account, role assignment,
balance, or authorization outcome is hard-coded into product logic.

Each fixture participant runs in a separate user-node server process with its
own async runtime and network listener. User nodes authenticate through a real
Authentik issuer and call one authoritative in-memory bank server over TCP.
Requests race, retry, disconnect, reconnect, lose authorization, and observe
live changes.

At minimum, the courtroom must include:

- two unrelated personal-account customers;
- a joint personal account whose owners have non-identical transaction and
  disclosure authority;
- one business with an owner, an initiator, an approver, and a read-only user;
- a delegated signer whose authority is narrower than the account owner's and
  expires during the fixture;
- a bank teller, branch manager, estate specialist, compliance officer, and
  bank auditor with non-equivalent branch, case, institution, action, and
  disclosure scopes;
- a user who has both customer and employee relationships;
- a deceased customer with an estate case, a court-recognized executor, at
  least two beneficiaries, restricted identity and account fields, and an
  account subject to probate controls;
- a branch manager who is also a beneficiary of that deceased customer's
  estate and therefore has a real conflict of interest;
- a non-conflicted compliance officer capable of approving only the exact
  exceptional access or estate action allowed by the case;
- two concurrent transfers competing for the same available funds;
- an idempotently retried transfer whose response is lost after commit;
- a business payment that requires approval by a different authorized user;
- authorization revoked while a live subscription is active;
- a purpose-bound capability narrowed to one estate case, one account family,
  selected fields, allowed operations, and a finite validity interval;
- an attempted use of branch-manager authority against the manager's own
  beneficial interest, including attempts to combine employee and beneficiary
  grants;
- a break-glass request for an urgent account-preservation investigation,
  including approval, expiry, revocation during live delivery, field masking,
  and attempted escalation from restricted read/freeze authority into
  disbursement;
- an inserted relationship that changes a previously negative authorization
  decision;
- a stale request whose read-set is relevant to the concurrent mutation;
- an unrelated mutation that must not cause a false conflict; and
- a committed transfer undone through a compensating journal entry, a redo
  attempted after intervening relevant drift, and an irreversible operation
  that exposes no fabricated undo action;
- an indeterminate response that returns an actionable recovery capability
  without granting commit, rollback, or retry authority to copied receipt data;
  and
- a crashed or disconnected user node that cannot leak a session, queue, or
  executable authority.

The system must fail closed. Authentication success is not authorization,
permission declaration is not permission evaluation, and policy evaluation is
not commit authority.

### Conditional-operation host courtroom

The conditional-operation courtroom installs one exact application operation
with one domain-specific conditional node, one temporal node, and one named
host clock source through `worth-query-host`. Its authoritative Relational
truth contains future, already-due, cancelled, superseded, and completed
temporal intents. The same courtroom must survive:

- a Query application-runtime reinstallation after durable intent commit but
  before a wake is scheduled;
- reinstallation after Signal makes a wake eligible but before Query invokes
  the operation;
- reinstallation after the operation commits but before derived wake
  completion is observed;
- duplicate, reordered, stale, and foreign-source clock observations;
- a provider replacement, an installed-operation generation change, and a
  conditional-node identity substitution; and
- unrelated domain, operation, node, clock, and temporal-intent growth.

Current authoritative truth, not a retained host timer or copied wake handle,
decides what is reconstructed. Query must re-admit the exact installed
application operation against current authentication, capability, purpose,
disclosure, invariant, idempotency, and provider truth. Signal alone decides
wake eligibility and suppression. A cancelled or completed intent causes no
invocation; a commit-before-observation fault cannot duplicate application
effects; and stale or foreign bindings fail before predicate, scheduling, or
operation work.

Any implementation is convicted if the host constructs a `SignalGraph`,
imports `worth_runtime_bridge` or `worth_signal`, returns a Signal decision,
runs a timer/scheduler loop, invokes an operation callback directly, treats an
in-memory wake table as durable truth, or scans all application truth on each
clock observation.

## Product Decision Lock

### Application boundary

1. The bank world is production-shaped example code with the same public
   dependencies available to an ordinary consumer.
2. The bank domain owns banking vocabulary, schemas, operations, invariants,
   authorization contributions, and presentation-independent application
   semantics.
3. Query owns canonical declarations, installation, admission, touched-graph
   policy composition, execution progression, compare-and-commit progression,
   result meaning, live-query meaning, and explanation contracts.
4. The temporary HTTP layer owns HTTP, OIDC transport adaptation, request
   deadlines, cancellation, serialization, and status-code mapping. It owns no
   banking policy and mints no Query or graph authority.
5. The fixture owns dynamic environment provisioning and teardown. It cannot
   introduce a privileged mutation or authentication bypass.

### Identity and authentication

6. Authentik is the authentication authority for the reference world.
7. An external principal is identified by the typed pair `(issuer, subject)`.
   Email, username, display name, token text, and locally assigned fixture
   names are not identity.
8. JWT validation requires issuer, audience, signature, time, and nonce/state
   posture appropriate to the selected OIDC flow.
9. The Authentik adapter produces an authenticated external-principal proof. It
   does not produce bank authorization or Query execution authority.
10. Mapping an external principal to a bank principal is an installed,
    graph-backed operation. Unknown, disabled, ambiguous, or stale mappings fail
    closed.
11. Automated tests acquire tokens through a real supported OIDC flow. A test
    helper may provision users and clients but may not mint, decode-and-trust,
    or bypass validation of identity tokens.

### Authorization

12. Authorization is capability-, relationship-, operation-, and context-
    scoped. A role may contribute capabilities but is never itself ambient
    permission. Flat role strings, route-local conditionals, caller booleans,
    and token claims open no application action.
13. An installed capability grant names the semantic action family, resource
    scope, admissible field or relation scope, purpose constraints, validity
    interval, delegation posture, grant provenance, and any amount,
    cardinality, or workflow-stage limit that changes what it authorizes.
    Omitted dimensions do not default to global authority.
14. Customer powers derive from current personal, business, executor,
    beneficiary, and authorized-user relationships. Employee powers derive
    from institution, branch, case, teller, auditor, manager, and compliance
    assignments. No grant silently crosses those scope or authority families.
15. Query composes explicit allow, deny, separation-of-duty,
    conflict-of-interest, distinct-actor, purpose, and field-disclosure
    predicates. A principal holding several valid grants receives only their
    lawful composition under installed policy; incompatible grants cannot be
    stacked into a stronger unnamed authority.
16. Relational owns authoritative graph facts and exact touched-graph evidence.
    The runtime bridge owns installed aspect correspondence and lowering.
    Signal owns local policy-node evaluation and decision evidence. Query owns
    their legal composition into operation admission.
17. No Query-local “super permission,” host callback, token claim, or route
    middleware result may replace that composition.
18. Authorization is evaluated against the graph the operation can actually
    read or affect. The executor cannot touch an entity, relation, aspect,
    field, case, or purpose outside the admitted capability and touched graph.
19. Negative authorization facts, membership dependencies, grant validity,
    purpose, conflict relationships, case state, approval state, and
    revocation enter the decision read-set whenever they influenced admission,
    so every authority-widening or authority-narrowing mutation is causally
    visible at commit and delivery.
20. Read projection, mutation admission, explanation, activity history,
    cursor continuation, preview, and live delivery distinguish internal
    computation authority from consumer disclosure authority. A protected
    fact may influence an installed policy, predicate, ordering, or aggregate
    only under separately admitted internal access and noninterference
    contracts; that access does not authorize disclosure. Consumer-visible
    result construction applies field-disclosure scope before projection or
    serialization. Masked or omitted fields carry typed posture; post-
    projection redaction is forbidden.
21. Break-glass is a governed state transition with requester, approver,
    reason, purpose, exact scope, allowed actions, disclosed fields, issue and
    expiry time, revocation, audit, and mandatory review posture. It cannot
    erase conflict-of-interest, distinct-actor, invariant, or commit
    requirements; the bank reference path is read/freeze-limited and cannot
    authorize estate disbursement or personal benefit. Revocation or expiry
    closes or narrows active delivery before the next payload.

### Typed schema and aspects

22. Installed schema declarations generate or bind typed entity, relation,
    aspect, field, operation, and policy references for ordinary consumers.
23. Application code does not pass semantic aspect names as strings.
24. Values use Foundational-native exact scalar and struct families. Money uses
    an exact typed minor-unit representation with an explicit currency.
25. Dynamic extension, if retained, uses an explicitly dynamic key type and a
    checked schema-readmission boundary. It is not an escape hatch in generated
    application code.
26. Typed references are runtime- and schema-version-bound where use could
    otherwise cross installations.
27. Generated or derived APIs expose only legal operators and writes for the
    referenced schema capability.

### Foundational boundary law for Runtime Phases 6-8

F1. Query remains the strongest owner of application-query installation,
    planning, execution, authorization, disclosure, recovery, undo, redo, and
    runtime-local receipts. Foundational vocabulary cannot mint, replace,
    weaken, or reconstruct any of those authorities.
F2. Meaning that crosses declaration, installation, execution, publication,
    support, or adapter crates uses `worth-foundational` rather than a
    Query-local duplicate: canonical basis and comparison, aspect contracts and
    masks, boundary categories and roles, diagnostic vocabulary, provenance and
    lineage posture, profile progression, and performance claim vocabulary.
F3. Canonical basis is the semantic identity surface. A digest is derived
    compression downstream of a ready canonical basis and never executable
    authority. Debug formatting, joined strings, raw enum `Debug` output, or
    independently maintained hash encodings cannot define cross-crate query,
    capability, disclosure, or aftermath identity.
F4. Query-specific canonical material uses one named
    `CanonicalBasisDomain::Future(...)` constant and one explicit
    `CanonicalizationRuleVersion` per semantic family until Foundational owns a
    named general domain. Raw domain strings may not proliferate across
    declaration, installation, execution, receipts, or tests.
F5. Typed Query result selectors and hot runtime indices remain Query-owned and
    must not rebuild canonical basis or derive digests during projection,
    continuation, authorization, or delivery. Installation prepares canonical
    meaning once; warm paths carry typed or indexed proof of that installed
    meaning.
F6. Foundational `ProjectionMask`, `MutationMask`, and `DiagnosticMask` express
    contract-level field legality. Query capability and disclosure admission
    decides who may compute, disclose, or mutate under that legal upper bound.
    A mask alone opens no Query authority.
F7. Query receipts remain the exact executed runtime evidence. Only an explicit
    boundary, support, diagnostic, or performance adapter may lower them into
    Foundational receipt, provenance, lineage, profile, or counter-backed
    vocabulary; planning claims may not masquerade as executed evidence.
F8. Foundational transition and lineage vocabulary may describe committed or
    completed aftermath, but Query owns the actual recovery, inverse,
    compensation, and fresh redo transitions. Foundational branch, merge, and
    cherry-pick authority is not a hidden implementation path for linear
    aftermath.
F9. Canonical closure is family-local, not milestone-wide. Every later phase
    that introduces new identity-bearing meaning -- including mutation
    preconditions, capability and disclosure contracts, elevation, aftermath,
    external-effect causality, and linear lineage -- prepares its own ready
    Foundational canonical basis and proves structured comparison. An earlier
    query-identity proof does not authorize a later private grammar.
F10. When Query needs a compact key for cross-crate semantic meaning, it admits
    the ready basis through Foundational's typed digest slot and then carries
    the derived digest inside a stronger Query-owned installation or attempt
    artifact. Query may not choose the digest input grammar, algorithm slot, or
    comparison rule through a direct `sha2` call. Runtime-local opaque hashing
    that does not define cross-crate meaning remains distinct and may not be
    presented as canonical identity.
F11. Foundational admits SHA-256 as its only digest algorithm. Runtime Phase 6
    removes the former fixture hash and deferred cryptographic-debt surface,
    freezes deterministic vectors and wrong-slot denial at the lower layer,
    and proves no production or certification caller retains the deleted
    algorithm. Runtime Phases 6-8 reuse that zero-debt lower-layer slot; they do
    not add Query-local digest engines.
F12. Every identity-bearing semantic family declares one derivation seam and
    one cost lane. Installation-bound meaning prepares its ready basis and any
    compact digest once when that meaning is installed or rebuilt.
    Request-bound meaning may prepare and derive only once per fresh admission,
    under installed limits for semantic-entry count and canonical encoded
    bytes. Later planning, execution, provider commit, retry resolution,
    recovery, publication, and live delivery carry the retained typed artifact;
    they do not rediscover or rederive it. Each independently received retry
    may prepare its request-bound identity once during fresh admission; retry
    resolution thereafter carries it. Undo and redo may each derive one new
    intent identity, but cannot rehash retained predecessor meaning.
F13. Canonical-basis preparation and digest derivation are mechanically
    observable work. Installation and admission evidence, Query execution
    receipts, and certification counters distinguish installation, admission,
    execution, and publication and count basis preparations, digest
    derivations, canonical bytes encoded, and textual digest materializations.
    Root selection, traversal, predicate and ordering evaluation, projection,
    continuation delivery, live maintenance and delivery, authorization-fact
    evaluation, provider recompare-and-commit, and recovery inspection perform
    exactly zero basis preparations, digest derivations, and textual digest
    materializations. Internal paths carry typed fixed-width digest bytes;
    hexadecimal or other textual forms exist only at an explicit diagnostic,
    publication, provider-wire, or support boundary. If a contiguous canonical
    representation cannot remain within its declared cold or admission budget,
    the Foundational owner must provide bounded incremental encoding; Query may
    not compensate with a private hash grammar.
F14. Descriptive digests and installation authority seals are different
    cryptographic families. A semantic digest remains Foundational-owned and
    carries no authority. An installation authority seal is Query-owned,
    keyed HMAC-SHA-256 over one typed, domain-separated transcript and is
    verified in constant time. The installed-index build obtains one secret
    root key from the operating system's cryptographic random source through
    its existing fallible boundary, derives generation- and package-scoped
    child keys, and exposes none of those keys through public fields, textual
    identities, serialization, or `Debug`. Exact rebuilds retain the same
    root lineage; successor generations derive new package keys; an
    independently created root is foreign authority even if descriptive
    runtime ordinals and package meaning match. Secret-prefix hashing,
    caller-reconstructible "nonces", untagged optional fields, ordinary string
    equality for MAC verification, and raw secret byte arrays in
    proof-carrying artifacts are forbidden.

### Monetary model

28. Monetary effects are represented by immutable balanced postings. Account
    balances are derived from postings rather than independently mutable
    balance fields.
29. Every journal entry balances to zero in one currency and contains at least
    two postings.
30. Transfer, deposit, withdrawal, opening funding, compensating reversal,
    estate freeze, estate release, and estate disbursement each have a distinct
    typed operation, purpose, capability requirement, and aftermath posture.
31. Deposits and withdrawals move value through explicit bank cash or settlement
    accounts; they do not mint or destroy money implicitly.
32. Amounts are positive, currency-compatible, and bounded. Floating-point
    money is forbidden.
33. Available-funds and account-status invariants execute over proposed state.
34. Business payments and estate disbursements may require separate initiator,
    approver, executor, compliance, or beneficiary-exclusion predicates
    according to installed policy. One principal cannot satisfy a
    distinct-actor or conflict-free rule by holding multiple roles or grants.
35. A request-scoped idempotency key is bound to authenticated principal,
    operation family, semantic payload, and institution. Reuse with a different
    payload denies; a retry of the same committed intent returns the same
    semantic result without duplicating postings.

### Compare-and-commit

36. An invariant-approved proposal still cannot mutate authoritative state.
37. Compare-and-commit consumes the provider session, complete decision
    read-set, proposed state, invariant proofs, idempotency posture, and exact
    effect set.
38. The provider proves one atomic transaction or returns a typed non-commit
    outcome.
39. Relevant drift returns stale or denied posture. Unrelated drift may commit
    without widening the declared or realized authority.
40. Committed, stale, policy-denied, invariant-violated, aborted,
    partial-effect, indeterminate, cancelled, and already-committed-idempotent
    outcomes remain distinct.
41. An in-memory provider may honestly prove atomicity within its process. The
    milestone does not turn process memory into a stronger guarantee.
42. A provider that cannot prove atomicity may not return `Committed`.

### Async and transport

43. All network-facing and provider-facing operations are async and carry
    deadlines, cancellation, bounded queues, resource budgets, backpressure,
    overflow posture, and partial-effect meaning.
44. One user node equals one OS process, one async runtime, one network
    listener, and one independent authenticated session boundary.
45. The fixture uses dynamic ports and discovered identities. Shared process
    globals cannot coordinate users.
46. The bank server is the only authoritative application server. User nodes
    are clients/proxies and cannot cache or reconstruct bank authority.
47. Live updates use a real streaming transport such as SSE. Polling may exist
    only as an explicit degraded posture, not as fake live delivery.
48. Disconnect, timeout, and response loss do not imply rollback. The client
    resolves uncertainty through the typed idempotency/outcome API.
49. HTTP response mapping preserves semantic outcome distinctions and never
    treats every denial as `500` or every accepted request as committed.

### Replay and history

50. Ordinary users may query authorized account activity and receive live
    updates through normal Query surfaces.
51. Runtime replay remains certification-only. The bank certification suite may
    reconstruct results from retained in-memory fixture history through
    `worth-query-replay`; product and user-node crates may not import replay
    authority.
52. Replay evidence must agree with ordinary result meaning without becoming an
    application mutation or recovery API.

### Application queries, recovery, and aftermath

53. A named application query is an installed schema-bound definition with
    typed parameters, admitted root, result shape, cardinality, ordering,
    dependency and disclosure contracts, supported truth bases, and declared
    one-shot, history, live, or preview eligibility. A host-local marker plus
    an unregistered projection closure is not the completed front door.
54. Query owns canonical application-query identity and opaque continuation
    identity. A cursor is bound to the installed query, parameters, scope,
    basis, ordering, result shape, and compatibility generation. It carries no
    authorization; every continuation receives fresh admission.
55. Current, pinned historical, live, and admitted preview execution consume
    the same canonical query meaning where supported. A lane may strengthen
    lifecycle or consistency requirements but may not silently change
    projection, disclosure, ordering, or membership meaning.
56. Callers explicitly own result limits, work limits, deadline,
    cancellation, requested consistency, freshness posture where meaningful,
    truth basis, and mutation preconditions. Query turns expected-version or
    expected-fact preconditions into provider-recompared facts rather than
    host-side races.
57. Read and disclosure outcomes identify truth posture, provider version,
    work, result count, truncation, typed omissions, and a governed access
    receipt sufficient to explain what was disclosed under which installed
    authority. The receipt is evidence, not reusable authority.
58. Capability support is inspectable before execution: supported, supported
    with named degradation, provider capability required, Store capability
    required, advanced access product required, or unsupported. Lack of
    support never widens into a scan, host loop, post-read filter, or local
    cache.
59. Installed application-query predicates, filters, ordering, traversal, and
    result shape lower into the existing Milestone 9.10 graph-read proof
    chain: admitted schema references, canonical read graph, selectivity and
    access shape, `PredicateSupport` and `OrderingSupport` requirement rows,
    cost and inventory admission, one admitted access plan, and receipt-backed
    execution. Application queries do not create a parallel planner, index
    inventory, or execution lane.
60. Relationship expansion, nested result construction, filtering, and sorting
    execute through one admitted graph-read access plan or return a typed
    required-capability, streaming, materialization, or denial posture. A
    facade-side loop, per-result neighbor lookup, undeclared
    post-materialization sort, or repeated child query cannot satisfy the
    contract. Covered receipts prove exact-zero caller-owned N+1 work and zero
    undeclared fallback.
61. Every installed mutation declares two things: its correction authority --
    the runtime alone, the runtime together with a distinct actor or external
    truth owner, or none -- and, unless that authority is none, exactly one
    correction mechanism. Query derives the published `Reversible`,
    `Compensatable`, `Reconcilable`, or `Irreversible` posture from that pair.
    A declaration may not state a posture directly, and an omitted or
    contradictory axis fails installation rather than defaulting in either
    direction. The public result exposes only next actions valid for that
    operation, outcome, current authority, and provider posture.
62. Undo never deletes or rewrites committed truth. It is a newly admitted
    operation derived from the exact committed receipt and installed inverse
    or compensation contract, with fresh authentication, authorization,
    policy, decision facts, invariants, idempotency, and compare-and-commit.
63. Bank money movement uses compensating journal entries. Authorization
    changes use explicit inverse operations when still lawful. Account
    creation, approval, audit, or externally escaped effects expose
    compensation, reconciliation, or irreversible posture rather than a fake
    inverse.
64. Redo is not replay and does not reuse prior authority. A successful undo
    may expose a descriptive redo intent for fresh execution against current
    truth. Relevant drift, changed policy, expired capability, or changed
    invariant posture may deny it.
65. Milestone 9.16 historically proposed a linear, current-head,
    receipt-linked undo/redo journey. The implementation is now provisional;
    Milestone 9.18 must accept, revise, or replace it before it becomes a
    guarantee.
    Relational remains the sole owner of its current commit identity, parents,
    branch head, ancestry, and publication. Query owns typed `undo-of` /
    `redo-of` operation meaning but owns no parallel history chain or head.
    Milestone 9.17.1 first adds exact owner component bases and Relational
    branch-local MVCC, 9.17.1.1 and 9.17.1.2 close the Relational and Signal
    service prerequisites, 9.17.2 adds Runtime World-owned composite product
    branches while preserving each component owner, and 9.17.3 completes Query
    carriage and public-facade cutover;
    Milestone 9.18 owns exact composite source/target selection, tree-based
    reversal/reapplication, and fresh Query admission as new composite history.
    Runtime Bridge owns installed component correspondence and Runtime World
    coordinates composite publication; neither owns correction meaning or
    Relational or Signal internal currentness. Semantic merge, rebase,
    multi-parent publication, offline
    synchronization, and distributed recovery remain governed by the
    [cross-runtime merging-and-branching roadmap](../cross-runtime/merging-and-branching-roadmap.md).
66. Indeterminate and externally unresolved outcomes carry a framework-owned
    recovery handle naming the legal next actions: inspect, resolve by
    idempotency, retry safely, compensate, reconcile, or dispose. A bare status
    or copied receipt cannot manufacture any of those transitions.
67. Provider commit, emitted application causality, and external completion
    remain distinct typed facts. Local commit may authorize managed dispatch,
    but it cannot claim that a device, payment rail, notification system, or
    other external authority completed its effect.
68. No operation emits an escaping effect without a committed local fact
    anchoring it. An application may declare an operation with no domain
    mutation; the runtime still commits that operation's dispatch intent in the
    same transaction, and that record is the anchor. There is therefore no
    mutation-free external effect -- only an operation whose sole domain effect
    is its dispatch record. An escaping effect without an anchor has no
    correlation target, no idempotency record, no recovery handle, and no
    authoritative local answer to whether it occurred, so anchoring is a
    property of the runtime rather than a per-operation choice. An operation
    that declares an escaping effect can never publish `Reversible` posture,
    because reversal derives from recorded inverse data without external
    reread.

### Host-installed conditional operations and managed time

69. `worth-query-host` owns the only application-host installation contract
    for conditional execution in the primary-graph application runtime. The
    host binds a provider to an exact installed schema generation, application
    operation, conditional-node identity, provider family, and graph
    participation authority. A label, digest, portable declaration, or equal
    node ordinal cannot substitute for that installed binding.
70. A domain condition provider receives a Query-owned, read-only observation
    set admitted from the node's declared semantic dependencies. It may compute
    only the application predicate result or a typed provider failure. It may
    not return a Signal condition decision, choose eligibility, schedule work,
    mint provenance, widen observations, invoke an operation, or retain an
    executable snapshot or provider session.
71. Query adapts the admitted predicate result into the existing pair-bound
    Runtime Bridge conditional lowering. Runtime Bridge owns installed
    Relational-to-Signal correspondence and lowering; Signal owns condition
    resolution, wake scheduling, eligibility, coalescing, suppression, and
    decision provenance. Query consumes that decision without restamping it.
72. A temporal conditional node binds an exact named host clock source during
    application-runtime installation. The host may submit observations only
    through a Query-owned clock-observation port bound to that source and
    runtime. Query validates source identity, timeline, monotonic progression,
    duplication, reordering, and installation generation before forwarding an
    admitted observation. A host observation is time evidence, never an
    eligibility decision or operation authority. A temporal clock source is
    not the authorization-time source and cannot satisfy authentication,
    capability-expiry, elevation-expiry, deadline, or idempotency time checks.
73. Signal's wake table is derived, volatile runtime state. Reconstructible
    temporal intent -- target operation, node, application input or input
    derivation, due basis, active/cancelled/completed posture, stable intent
    identity, and idempotency relation -- lives in authoritative
    Relational/domain truth under an installed, bounded reconstruction
    contract. A host-local timer, task, cache, or serialized Signal wake is not
    a truth source.
74. Application-runtime installation and reinstallation reconstruct missing
    temporal wakes from the exact current Relational projection before the
    runtime reports conditional readiness. Reconstruction revalidates the
    installed operation, node, provider, clock, graph, generation, and current
    intent posture. It may create a new derived wake identity linked to the
    stable domain intent; it may not treat an old wake identity as authority.
75. A ready wake re-enters the same installed application-operation path used
    by an ordinary request. Wake evidence does not authorize execution. Query
    performs fresh principal or governed system-actor admission, capability and
    purpose evaluation, touched-graph admission, invariant progression,
    idempotency, and compare-and-commit. The operation must atomically consume
    or advance the authoritative temporal-intent posture when its effect
    commits, so reconstruction after commit cannot duplicate effects.
76. Provider registration, clock observation, wake ownership, and reconstructed
    intent resources are bounded and lifecycle-managed. Closing or replacing
    an application runtime cancels its derived work and releases its leases;
    stale tasks, providers, clocks, decisions, and wake handles open no door in
    the successor runtime.
77. Ordinary clock observation and wake promotion are bounded by admitted due
    work and never scan unrelated domain rows, operations, nodes, or clocks.
    Reinstallation reconstruction is an explicitly measured cold path bounded
    by the installed reconstruction projection. Counters distinguish clock
    admission, truth rows inspected, wakes reconstructed, eligibility,
    suppression, predicate contacts, operation admissions, idempotent
    resolutions, and committed invocations.

## Destination Topology

### Query authority packages

```text
worth-query-decl
    schema-derived typed references
    typed external identity and principal-binding declarations
    installed application-query and result-shape declarations
    capability, purpose, disclosure, delegation, and elevation declarations
    operation, read, mutation, workflow, history, live, and aftermath declarations

worth-query-installation
    installed principal-binding and authorization contracts
    touched-graph policy compilation
    query identity, basis, cursor, disclosure, and lane-eligibility contracts
    capability composition, conflict, break-glass, and review contracts
    inverse, compensation, redo, and external-effect posture contracts
    installed conditional-provider, named-clock, temporal-intent, and
    reconstruction contracts
    monetary operation and invariant contribution admission

worth-query-admission
    admitted authentication adapters and sealed external-principal proof
    authenticated-principal-bound query and operation admission
    purpose-bound capability and field-disclosure admission
    governed elevation and current-review admission
    touched-graph and policy decision admission
    compare-and-commit admission

worth-query-execution
    one installed primary Relational graph authority
    indexed external-principal-to-application-principal binding
    policy-evaluated attempts
    canonical application-query basis and continuation progression
    provider compare-and-commit progression
    current capability and disclosure revalidation
    host-provider binding, managed clock observations, conditional re-entry,
    and temporal wake reconstruction
    idempotent typed outcomes and managed recovery
    linear inverse, compensation, and redo progression

worth-query-publication
    capability- and disclosure-scoped read, mutation, activity, explanation,
    history, live, recovery, and aftermath contracts

worth-query-host
    ordinary host composition, conditional-provider installation, and named
    clock observation without raw Relational, Runtime Bridge, Signal, or
    authority exposure

worth-query-certification
    hostile public-consumer, capability, disclosure, elevation, aftermath,
    concurrency, and replay proof
```

### Reference-world packages

The exact package count may follow implementation pressure, but semantic
ownership must remain visible:

```text
workspaces/worth-query-bank-world/
    crates/
        bank-domain/
            banking and estate schemas, typed operations, invariants,
            capability contributions, disclosure meaning, and aftermath posture
        bank-server/
            runtime composition, installed ordinary queries,
            and authoritative in-memory provider
        bank-http-adapter/
            Axum transport and Authentik OIDC adaptation
        bank-user-node/
            independent authenticated client/server process
        bank-courtroom/
            environment provisioning and cross-process certification
```

No `common`, `shared`, `helpers`, or generic “app core” package may obscure
ownership. If a responsibility remains local, it remains in the owning package.

### Cross-runtime authority flow

```text
Authentik
    -> authenticated external-principal proof
    -> installed graph principal binding
    -> typed purpose and requested capability
    -> installed capability, conflict, and disclosure decision
    -> canonical Query query or operation intent
    -> Relational touched-graph facts
    -> runtime-bridge installed lowering
    -> Signal policy decision
    -> Query admitted operation
    -> Relational proposed state
    -> installed invariants
    -> provider compare-and-commit
    -> Query publication
    -> managed recovery / inverse / compensation / redo admission
```

Every arrow changes authority. The receiving boundary validates the proof it
needs; it does not trust an adjacent receipt or reproduce the previous owner's
decision.

## Phase Plan

### Bank World Track — Phase 1: Freeze The Bank World And Build The Gap Ledger

**Requirement**

Define the legitimate product behavior, fixture topology, public consumer
transcript, and requirement/evidence ledger before changing Query.

**Must establish**

- exact bank entities, relationships, operations, invariants, roles, and
  outcome families;
- the supported OIDC flow and authenticated-principal boundary;
- dynamic Authentik, bank-server, and per-user-node topology;
- one ordinary DX transcript for each read, mutation, approval, activity, and
  live path;
- a front-door gap ledger classifying every required workaround as a Query gap,
  bank-domain concern, transport concern, fixture concern, or intentionally
  unsupported capability; and
- the cheapest commands for declaration, installation, execution, consumer,
  cross-process, and hostile certification loops.

**Proof before Runtime Hardening Phase 1**

The ledger covers every courtroom actor and behavior, every requirement has one
owner, and no test plan relies on a hard-coded principal or privileged fixture
mutation.

### Runtime Hardening Track — Phase 1: Schema-Bound Typed Application References

**Requirement**

Make the bank domain expressible without raw semantic strings or caller-cast
values.

**Must establish**

- typed entity, relation, aspect, field, operation, policy, and currency
  references derived from the installed schema;
- typed read/projection/filter and mutation/effect authoring;
- explicit checked dynamic-extension posture, if one exists;
- early operator, value-family, runtime, and schema-version rejection; and
- compile-time denial for representative cross-schema, wrong-value,
  wrong-operation, and illegal-write examples;
- dependency compilation distinguishes primary-logical-graph reads and touches
  from separate-authority provider calls: primary reads and runtime mutations
  retain their native evidence, while remote reads, touches, and commits still
  require exact provider receipts.

**Proof before Runtime Hardening Phase 2 and Bank World Phase 2**

The bank-domain package contains no application aspect strings, the public DX
transcript compiles using only declaration-facing types, and a typed local
mutation with an installed primary read completes dependency compilation
without inventing a provider receipt.

### Runtime Hardening Track — Phase 2: Authenticated Principal Admission

**Requirement**

Define the generic boundary through which a validated external identity becomes
a sealed, time-bounded principal proof eligible for installed graph binding.
Query must not learn Authentik, OIDC transport, JWT, cookie, or HTTP semantics.

**Must establish**

- typed issuer-and-subject identity;
- a collision-free canonical representation of the pair for indexed graph
  lookup, without exposing a raw application string key;
- a first-class principal-binding member in canonical application-schema
  identity, closure, installation, and runtime-generation validation;
- application schemas carried through the ordinary host domain-package grammar
  into the exact installed portable package index; a declaration that exists
  only in a side-built installation index is not host composition;
- one compatible canonical owner grammar across host domain packages and
  application schemas; namespace-qualified owners are allowed without allowing
  dots in schema or member identifiers;
- a sealed proof constructor available only to an admitted authentication
  adapter boundary;
- audience, validation-time, expiry, authentication-method, and adapter identity
  bound into the proof where later admission depends on them;
- one execution-owned primary Relational graph root shared by ordinary runtime
  paths, rather than a fixture registry, parallel map, or separate-provider
  graph-participation lane;
- a purpose-scoped host composition terminal that installs and publishes that
  primary graph without requiring bridge, read, mutation, live, signal, or
  inspection adapters whose authority is not exercised in this phase;
- publishing the purpose-scoped terminal consumes the raw execution root and
  installation authority; unrelated ordinary facade families and explicit
  backend/backend-part assembly are absent from its public type at compile
  time;
- the primary graph retains the Relational schema, invariant, and runtime
  configuration already installed by ordinary Query composition rather than
  replacing it with a freshly defaulted runtime;
- a consumed, installation-only typed bootstrap path for principal and mapping
  rows; no privileged identity mutation path survives runtime publication;
- installed, indexed mapping from authenticated external principal to
  application principal, with exact mapping, relation, and typed
  target-principal identity freshness evidence;
- the target-principal identity field is a first-class, read-only,
  equality-capable principal-binding dimension whose scalar family and Rust
  value type participate in declaration closure, canonical schema identity,
  installation, runtime resolution, and freshness;
- the sealed Query principal proof returns that typed application identity from
  the exact mapped primary-graph row; a parallel external-identity-to-product-ID
  registry is forbidden;
- target-principal identities are canonically unique within each installed
  principal binding, and bootstrap admission of external identity, principal
  key, and typed identity is atomic on denial;
- a Relational-owned bounded equality-index lookup contract whose ordinary
  path examines at most the declared candidate cap and whose cold
  certification path proves the same classification against authoritative
  storage;
- unknown, disabled, ambiguous, expired, and cross-runtime denial;
- request deadline and cancellation carriage; and
- no deserialization, fixture, token-claim, or host assertion path that can mint
  the proof.

**Proof before Bank World Phase 2**

A causally admitted test authentication adapter can produce a usable proof,
while forged data, copied fields, wrong runtime, stale proof, and direct
deserialization open no principal or operation authority. The indexed mapping
must prove storage parity and bounded candidate work. Mutable or wrong-typed
target-principal identity fields and copied binding identifiers paired with a
different identity type must deny. Changing the mapped principal's typed
identity must stale the proof. Two different Principal rows carrying the same
typed application identity must deny without poisoning a corrected bootstrap
retry. Primary identity facts
must not require a graph-participation provider receipt. A lawful
`worth-query-host` consumer must build the purpose-scoped runtime without
constructing unused adapter ceremony, while attempted ordinary read, mutation,
workflow, live, raw execution-root extraction, and replay actions fail to
compile against the purpose-scoped terminal.

This phase intentionally establishes the narrow primary-graph composition that
later authorization and ordinary-facade phases consume. It does not claim the
general read, mutation, workflow, or live front door owned by Runtime Hardening
Phase 5.

### Bank World Track — Phase 2: Authentik Adapter And Dynamic Identity Fixture

**Requirement**

Adapt real Authentik OIDC identity into the generic Runtime Hardening Phase 2 boundary
and provision all bank identities dynamically.

**Must establish**

- issuer/audience/signature/time validation and key rotation behavior;
- HTTPS is mandatory for the issuer, introspection, and revocation endpoints,
  and secret-bearing control endpoints must share the issuer origin; the cold
  courtroom may trust its ephemeral self-signed certificate only through
  certification-gated clients;
- cryptographic ID-token/access-token pair binding plus exact introspection
  client affinity; two independently valid users' credentials must not compose
  when their token halves are crossed;
- `(issuer, subject)` identity, with display claims retained only as attributes;
- unknown, disabled, ambiguous, expired, revoked, and wrong-audience denial;
- access-token introspection or equally authoritative online revocation
  evidence; decoding a locally valid ID token is not revocation proof;
- a real Authorization Code with PKCE browser flow carrying state and nonce;
- one bounded JWKS refresh-and-retry on an otherwise admissible token whose
  signing key is absent from the installed cache;
- explicit request deadline and cancellation propagation;
- no Authentik-specific type or token claim beyond the adapter boundary;
- no transport or test-only constructor for authenticated proof; and
- dynamic fixture provisioning, signing-key rotation, token revocation, and
  teardown through real Authentik administration and token acquisition
  boundaries;
- teardown proof covers containers, volumes, networks, and the secret-bearing
  fixture directory after both success and an intentionally failed body;
- a cold Docker courtroom lane that runs real Authentik, PostgreSQL, and browser
  automation without entering ordinary bank edit/test loops.

**Proof before Runtime Hardening Phase 3**

Tokens from the real issuer admit the correct principal, forged or malformed
tokens fail, and application code cannot create or deserialize the proof.

### Runtime Hardening Track — Phase 3: Scoped Authorization And Touched-Graph Admission

**Requirement**

Compile installed bank abilities into the existing Relational, runtime-bridge,
Signal, and Query authority chain, then bind each admitted operation to its
exact allowed touched graph.

**Must establish**

- personal, business, and institution-scoped ability declarations;
- schema-typed application-operation requirements that compile into the
  existing installed domain-operation graph-read, touch, effect, invariant,
  and conditional contracts rather than creating a parallel application lane;
- a move-only, schema-typed primary-graph installation phase for the entity,
  relation, and field facts required by policy evaluation, with no retained
  product mutation bypass after publication;
- current relationship-backed role evaluation;
- distinct-actor approval rules;
- positive, negative, membership, and revocation dependencies;
- Relational-owned touched-graph evidence for the policy facts actually read;
- Query-owned binding of the admitted operation to its installed allowed
  touched-graph contract, without pretending that not-yet-proposed record
  identities are realized effects;
- runtime-bridge lowering through installed correspondence;
- Signal-owned decision evidence;
- Query-owned composition into an operation admission proof; and
- retention of the exact authentication lifetime and live request
  cancellation/deadline authority in that operation proof, so later governed
  phases can reject authority that expired or was interrupted after admission;
- a Query-minted stable operation-scope fingerprint over the runtime,
  installed operation, authenticated principal, and typed scope for downstream
  idempotency binding; it excludes the observation snapshot so an equivalent
  authorized retry remains the same intent; and
- one lane-independent admitted-scope proof family that later read, mutation,
  explanation, history, and live facades must consume; Phase 3 does not invent
  those Phase 5 facades merely to claim integration.

**Proof before Bank World Phase 3**

Forged roles, token claims, caller-declared touched sets, route middleware
booleans, cross-account access, role combination, and post-revocation delivery
all fail. Valid customer and employee combinations remain usable.

### Bank World Track — Phase 3: Double-Entry Banking Operations And Invariants

**Requirement**

Implement the bank domain as ordinary installed operations that emit typed
effects and execute real monetary invariants over proposed state. This phase
produces immutable invariant-approved proposals; it does not create a
bank-local commit path ahead of Runtime Hardening Phase 4.

**Must establish**

- personal and business account creation;
- explicit opening funding;
- direct transfer by stable recipient identity;
- deposit and withdrawal through bank accounts;
- business initiation and separate approval;
- immutable journal entries and postings;
- exact balance, available-funds, currency, account-status, and balancing
  invariants; and
- private causal binding of each invariant witness to the exact in-memory
  snapshot basis used to mint it; an independently built snapshot with the
  same descriptive version is not the same basis;
- typed idempotency intent bound privately to the exact installed-operation
  authority identity, authenticated principal and typed scope, mutation
  preconditions, governed application input, and any Query-owned lifecycle
  proposal in fixed, non-aliasing component slots; the caller-supplied key and
  business intent cannot omit or substitute those components.
- no authoritative mutation or `Committed`/`AlreadyCommitted` outcome before
  provider compare-and-commit.

**Proof before Runtime Hardening Phase 4**

Independent accounting oracles recompute proposed balances and journal
conservation.
Overdraft, unbalanced entries, currency mismatch, duplicate approvals,
self-approval where prohibited, and idempotency-key payload drift all deny.
Equivalent payloads produce the same typed idempotency intent; Runtime
Hardening Phase 4 owns retry resolution and `AlreadyCommitted`.

### Runtime Hardening Track — Phase 4: Provider Compare-And-Commit

**Requirement**

Advance an invariant-approved Milestone 9.15 proposal through one
provider-proven compare-and-commit progression.

**Subphases**

- **4.1 — Application-to-provider progression.** Bind the Runtime Phase 3
  admitted application operation to the existing Milestone 9.15
  basis-complete read-set, proposed-state, and installed-invariant
  progression. The resulting commit candidate is a concrete Query authority
  that consumes the exact application admission, proposal, provider session,
  and invariant progression; none of those ingredients can authorize commit
  independently.
- **4.2 — Atomic compare-and-commit.** Recompare only the exact decision facts
  retained by that candidate while the installed provider owns the commit
  serialization boundary, then apply the entire effect program or none of it.
  Relevant drift stales the candidate; unrelated drift does not.
- **4.3 — Idempotent terminal resolution.** Bind the typed operation intent to
  an authoritative idempotency record in the same provider transaction and
  resolve equivalent retries, payload drift, response loss, and ambiguous
  provider outcomes without guessing. Provider intent composition uses fixed,
  domain-labeled slots for the installed-operation authority identity,
  operation scope (runtime, installation, authenticated principal, and typed
  scope), typed mutation preconditions, governed input identity, and lifecycle
  proposal identity. An absent slot is distinct from every present slot, two
  slot kinds containing identical bytes cannot alias, and the same caller key
  and business intent cannot cross installed operations, principals, or
  scopes.
  A governed input identity is materialized exactly once during capability
  admission. Injective fixed-width identities report zero canonical work;
  identities derived through the Foundational canonical digest front door
  carry that exact bounded digest work in the admission phase. Operation
  progression, provider commit, retry resolution, and projection retain the
  admitted bytes and must not rederive the identity or recount its work.
- **4.4 — Derived accounting truth and conflict dependency.** Remove stored
  available balance as an authoritative decision field. Reconstruct monetary
  balance only from the installed journal, posting, and posting-account graph.
  Retain a typed per-account accounting revision as coordination metadata,
  derived from that graph and advanced atomically for every account touched by
  a journal append. The revision may stale a competing attempt, but it never
  carries, replaces, or independently mutates monetary truth. Journal effect
  lowering reads the governed current revision from the exact projected read
  attempt and advances it by the emitted posting count; it never derives a
  successor revision from a deliberately partial domain proposal snapshot.
- **4.5 — Bounded application projection.** Give installed application
  composition a provider-derived, version-bound projection or access strategy
  whose ordinary preparation cost follows the admitted touched graph and the
  smallest honest adjacency granule rather than total provider state.
  Projection state is reconstructible acceleration, never a second truth
  source or commit authority. Its version and invalidation basis must be exact,
  and its structural work must be observable. For signed incoming aggregates,
  Query returns a typed summary containing both the checked scalar and exact
  contributing-source count. The exclusive form proves that each contributing
  source has exactly one relation of the declared kind, so malformed
  multi-target postings cannot be counted as lawful money. Warm maintenance
  examines only authoritative changed sources; a cache miss reconstructs from
  retained journal/posting truth, reports its input breadth, and denies on
  malformed scalar, cardinality, budget, count, or arithmetic state.
- **4.6 — Collision-free created identity.** Created domain objects derive
  stable typed identities from the operation's exact idempotency-key identity
  and an explicit within-operation role. Creation does not scan a global
  maximum, race through a caller-owned counter, or serialize unrelated
  operations merely to mint an identifier. Equivalent retries derive the same
  identities; distinct admitted idempotency identities cannot alias.
- **4.7 — Operation-scoped projection authority.** Bind provider-derived
  invariant projection to one installed operation's declared read capability.
  The locked reader exposes a field or relation only when its typed member
  implements that operation's `OperationReads` capability, and the resulting
  version-bound projection snapshot carries the same operation type into
  application read-set construction. Keep that compile-time projection
  ceiling distinct from the installed decision-read manifest: the former says
  which typed facts may inform bounded invariant construction, while the
  latter names the smaller exact dependency set that every realized attempt
  must observe and recompare before commit. Projection implementation details
  do not become mandatory conflict facts, and omitting a genuine decision
  dependency from the installed manifest remains invalid. A broad schema
  projection, a snapshot for another operation, or composition-root access to
  undeclared facts cannot mint mutation authority.
- **4.8 — Realized touched-graph identity closure.** Bind every
  provider-derived operation projection to the exact admitted operation and
  retain the entity identities actually resolved or reached through its
  bounded typed traversals. The application read attempt consumes that
  move-only closure together with the pinned projection lease: an unprojected
  attempt may observe only the admitted root scope, while a projected attempt
  may additionally observe only identities carried by that exact projection.
  An operation-typed but admission-unbound inspection, a projection bound to
  another admitted scope, or a newly resolved same-type entity outside the
  retained closure cannot become a decision fact or effect target. The
  installed decision-fact budget rejects an additional distinct fact before
  its authoritative value or relation is read and is rechecked when the
  complete read-set proof is minted.
- **4.9 — Current projection and lifecycle authority.** An admitted operation
  projection validates exact runtime and binding affinity plus current
  authentication, cancellation, and deadline authority before its closure can
  read provider truth. Admission-unbound projection is a separate inspection
  result type that carries output and work but no consumable snapshot or
  mutation progression. Framework-owned snapshot pins are released on success,
  denial, early drop, and unwind; a panicking domain projector cannot leak
  retention authority.
- **4.10 — Compiler-visible mutation progression.** Ordinary root-scoped
  application reads and admission-bound projected mutation reads are different
  sealed phase types. Completing an ordinary read set can authorize a read
  result, but cannot expose effect-program construction. Only a complete read
  set that consumed the exact admitted projection may advance to installed
  effect authoring, so raw admission plus a narrow read set is not an implicit
  higher-level mutation authority.
- **4.11 — Exact dynamic decision dependency plan.** The admission-bound
  invariant projection records the exact entity, field, and relation fact
  identities that the domain declares as commit dependencies while it resolves
  the admitted touched graph. The installed manifest remains the compile-time
  ceiling of permitted dependency families; it is not treated as proof that
  one arbitrary instance of each family is complete. Application read-set
  completion requires equality with the projection-carried dynamic dependency
  plan, including multiplicity across distinct entity identities, before
  mutation authority can be minted.
- **4.12 — Pinned, budgeted projection execution.** Every adjacency and
  endpoint lookup explicitly names the retained projection version rather than
  consulting current provider head. Structural counters count the records
  actually read, without self-certifying undercounts, and the installed
  operation contract supplies a distinct hard projection-work limit enforced
  during traversal. Ordinary mutation preparation explicitly disables
  replay/reconstruction; reaching the projection limit denies without widening
  into a whole-world or reconstructive lane.
- **4.13 — Typed admitted-root projection entry.** An admission-bound
  projector receives the exact typed scope entity that authorization admitted,
  already bound to the projector's private occurrence authority and realized
  identity closure. Domain projection begins from that value when the admitted
  scope is its root; it does not re-resolve the root from caller input, pay a
  redundant equality lookup, or risk planning a different same-type entity.
  Inspection-only projection has no admitted root, and no public identifier or
  equivalent entity value can manufacture one.
- **4.14 — Authorization dependency commit closure.** The exact Relational
  authorization observations and Runtime Bridge decision evidence that minted
  an application admission remain proof-carrying commit dependencies. Query
  joins those sealed observations to a separately budgeted installed provider
  decision-fact family and the provider recomparisons their exact relevant
  entity, typed-adjacency, relation, and predicate-field basis before effects
  can commit. A permission-revoking mutation, a newly matching deny path, or a
  removed allow path therefore stales the admitted attempt; unrelated
  relation-kind growth does not create a false conflict. Query neither trusts
  detached dependency identifiers nor replaces the installed
  Relational–Bridge–Signal composition with a host callback or an ad hoc
  policy re-evaluation.

**Must establish**

- complete decision read-set validation;
- mutation preparation from the exact retained branch snapshot, with the
  pinned Relational snapshot and Runtime Bridge authority basis joined before
  provider admission so unrelated head movement does not create a false
  conflict;
- request-local Runtime Bridge execution lanes over the same installed truth
  authority, so thread-affine Signal state cannot reject concurrent server
  requests before provider admission;
- exact proposed-effect evidence from the Bank Phase 3 proposal and proof that
  every realized read and effect is a subset of the operation admission's
  installed allowed touched graph;
- projection capability reads and commit-decision reads remain distinct.
  Ordinary field and relation traversal never capture a commit dependency
  implicitly. The installed decision manifest is the allowed target-family
  ceiling; explicit decision reads seal the exact realized dependency keys,
  and completion requires equality with that dynamic key set rather than one
  arbitrary observation from every allowed family;
- one exact realized identity closure joining the admitted root scope to the
  identities reached by its admission-bound operation projection; schema- and
  operation-compatible identities outside that closure open no read or effect
  authority;
- relevant versus unrelated drift classification;
- atomic effect application by the in-memory provider;
- one terminal attempt outcome;
- response-loss/idempotent resolution;
- explicit abort, partial-effect, indeterminate, stale, cancelled, denied, and
  committed posture; and
- no public construction path from plan, receipt, read-set, invariant result,
  or provider token fragments.
- one installed primary-graph provider authority owns bank decision facts,
  proposed-state comparison, and committed effects. `BankSnapshot` may remain
  an isolated proposed-state projection and independent accounting oracle, but
  it is not a caller-supplied commit basis, a parallel mutable runtime, or an
  authority that can be updated beside the provider graph.
- derived facts such as available balance are computed from the authoritative
  journal and posting state under the installed provider contract. They are
  not independently mutable fields that can diverge from the accounting
  history that proves them.
- every invariant-approved Bank proposal and its Query decision read-set share
  one consumed pinned projection lease; a same-version snapshot reacquired
  after proposal approval is not an equivalent causal basis.
- invariant projection reads are compiler-constrained to the operation's
  declared projection-read capability; every genuine commit dependency is
  separately admitted by the installed decision-read target manifest, and
  the retained projection lease is typed for that same operation before it
  can enter application read-set construction.
- an operation-typed projection that is not bound to the exact admitted
  operation remains inspection-only. The application read-set consumes both
  admission affinity and the bounded set of entity identities realized by the
  retained projection, and enforces the installed distinct-fact budget before
  another authoritative fact is read.
- stale, cancelled, expired, foreign-runtime, or foreign-binding admission
  cannot execute an invariant projection closure. Inspection output cannot be
  converted into a retained projection lease, and every framework-owned pin is
  released if projection unwinds.
- a complete ordinary read set has no effect-program transition. Mutation
  authoring is available only after the exact admission-bound projection,
  realized identity closure, and dynamic decision dependency plan have all
  been consumed into the projected read-set phase.
- the complete realized decision-fact keys equal the projection-carried
  dependency keys, not merely the set of installed target families. If an
  operation depends on the same field family for two accounts, both exact
  account-field facts are mandatory and neither can stand in for the other.
- projection adjacency and endpoint access is explicitly pinned to the retained
  version, hard-limited by an installed projection-work budget, and counted
  according to actual provider reads. Ordinary compare-and-commit planning
  selects a replay-disabled Runtime Bridge lane.
- the admitted projection closure receives its exact typed root scope directly
  from the operation proof. Root-dependent plans neither re-resolve that entity
  through a caller value nor substitute another same-type identity, and warm
  projection work excludes the eliminated lookup.
- authorization observations retained by the admitted operation enter the
  provider progression as their own installed, cardinality-bounded
  decision-fact family. Their exact relevant graph basis is recomparison
  evidence: revocation or a newly effective deny dependency prevents commit,
  while unrelated relation kinds are neither scanned by the warm path nor
  treated as causal drift.
- account accounting revisions are exact conflict dependencies for journal
  mutation, advance in the same provider transaction as their postings, and
  are reconstructible from authoritative journal topology. Bank projection
  requires equality between the revision and Query's exact aggregate source
  count before using the aggregate value.
- ordinary proposal preparation does not repeatedly scan or clone the entire
  installed application world. Provider-derived access paths carry an exact
  version basis, declare their synchronous maintenance and invalidation cost,
  and expose structural counters that distinguish touched work from unrelated
  world size.
- journal, posting, payment, authorization, and account identities created by
  operations have an exact typed derivation basis. Two unrelated proposals
  prepared from the same provider version neither choose the same created
  identity nor acquire a false shared allocation conflict.
- application effect authoring cannot call a raw Relational commit surface.
  Relational mechanics may implement the installed provider, but only the
  sealed invariant-approved provider candidate can reach its transaction
  boundary.

**Proof before Runtime Hardening Phase 5**

Concurrent transfers cannot overspend, unrelated writes avoid false conflicts,
the same idempotent request cannot post twice, and failure injection never
claims atomic commit without provider proof. Destroying or perturbing a derived
balance projection cannot change accounting truth, and rebuilding it from the
committed journal restores the same value and source count. Repeated public
mutations beyond the former history-proportional decision-fact ceiling retain
constant warm aggregate work, while malformed multi-account postings and
revision/count drift deny.

### Runtime Hardening Track — Phase 5: Ordinary Read, Mutation, Workflow, And Live Facades

**Requirement**

Expose the bank's ordinary work through one declarative Query front door whose
valid next actions follow typed phase progression.

Runtime Phase 5 is implemented in the following authority-ordered internal
slices. These are one runtime phase, not independent roadmap tracks.

**5.1 — Typed bounded read front door**

- declare account discovery, account summary/detail, authorized-user, activity,
  pending-payment, and institution-audit reads as typed installed application
  work;
- every read consumes the same Runtime Phase 3 admitted-scope proof family as
  mutation and begins from the exact typed admitted root;
- account discovery starts from the authenticated principal's graph identity
  and traverses only declared visibility relationships. It never enumerates the
  installed account world and redacts afterward;
- current-consistency, deadline, cancellation, maximum-result, and work-budget
  controls belong to the caller; and
- results expose typed data plus provider-read, touched-scope, truncation, and
  degradation metadata.

**5.2 — Complete typed mutation front door**

- transfer, deposit, withdrawal, opening funding, account creation,
  authorization grant/revoke, business initiation/approval/rejection, and
  reversal all consume Runtime Phase 4's sealed projected-mutation progression;
- private per-operation lowering retains compiler-visible installed
  create/delete/write/link/unlink/emit capabilities rather than dynamically
  interpreting a bag of bank effects;
- every invariant-approved proposal effect shape is covered by that exact
  operation's installed program. Proposal/program drift is an installation or
  compile-time failure, never a runtime fallback to untyped effect dispatch;
- typed application emits enter the same atomic provider commit causality and
  recoverable receipt as graph changes. They are never a best-effort
  post-commit side channel; Runtime 5.4 may consume this causality but does not
  retroactively create it;
- mutation decisions seal the exact causal field, edge, and bounded adjacency
  predicates carried by the admitted projection. In particular, a decision
  based on an empty outgoing or incoming relation set must become stale if a
  matching edge appears before commit; exact endpoint-pair observations are
  not accepted as proof of whole-adjacency absence;
- projected dependency completion re-observes only those sealed predicate
  keys. Callers cannot manufacture, widen, omit, or replace an adjacency
  predicate, and provider compare-and-commit validates its current membership
  against the pinned observation before applying effects;
- the installed mutation decision-read manifest is an allowed target-family
  ceiling, while the projection-carried keys are the exact realized
  dependencies. Conditional branches are not forced to invent one arbitrary
  fact from every allowed family; all actual admitted field and adjacency reads
  seal themselves, and completion requires equality with that dynamic set;
- the caller supplies typed input, request controls, and idempotency intent but
  cannot assemble admissions, read sets, provider candidates, receipts, or
  support snapshots; and
- every mutation returns the bank-domain typed outcome family with bounded work
  metadata.
- the published primary-graph provider support envelope admits every installed
  bank mutation contract within its declared decision-fact and effect-program
  ceilings, or publication fails. Per-attempt semantic width remains distinct
  from the fixed concurrent-attempt capacity; a contract must not install
  successfully only to be unconditionally rejected by provider admission.

**5.3 — Workflow continuations and typed explanations**

- business initiation may return a typed pending-payment continuation whose
  only legal decisions are approval or rejection under a newly authenticated
  principal and request scope;
- the continuation is a private-field descriptive payment handle, not retained
  authentication, admission, projection, snapshot, or provider authority. A
  successful or authoritatively recovered initiation and a current
  approval-required payment read may produce it so a separate user process can
  resume the workflow without transferring authority;
- approval and rejection markers contain no caller-selected actor identity.
  Execution derives the deciding principal from the newly supplied
  authenticated proof, consumes fresh request controls, and re-enters the
  installed authorization and invariant progression. A copied or stale payment
  identity therefore opens no workflow authority;
- failed, stale, cancelled, aborted, partial-effect, or indeterminate
  initiation cannot mint a continuation, while an `AlreadyCommitted`
  initiation recovers the same descriptive continuation;
- denial, invariant violation, stale attempt, abort, cancellation,
  partial-effect, and indeterminate outcomes retain distinct typed meaning; and
- explanation data is derived from the governed typed outcome and retained
  decision evidence through one total typed mapping. Invariant violation is not
  hidden inside a generic denial, and no public caller or adapter parses
  diagnostic strings to recover semantics.

**5.4 — Authorized history and live delivery**

- account activity history is a bounded ordinary projection of committed
  journal/posting topology, ordered by an authoritative per-account posting
  sequence committed atomically with the account journal revision. It is not
  replay, does not sort by idempotency-derived identity, and imports no
  certification-only reconstruction surface;
- history continuation is a descriptive, account- and provider-version-bound
  cursor rather than retained snapshot authority. A changed provider version
  produces an explicit stale-cursor outcome; continuation never silently skips
  or duplicates entries across a moving graph;
- account and activity live leases retain the exact admitted scope and installed
  Runtime Bridge/Signal authorization evidence;
- provider commit causality enters one bounded delivery source with explicit
  overflow, cancellation, deadline, and closure outcomes; and
- every declared effect payload carries a typed retained-byte contract. Effect
  authoring counts inline representation and recursively owned allocation
  capacity, rejects the cumulative committed payload before provider mutation
  when it exceeds the installed `RetainedBytes` envelope, and seals the
  validated batch as the only input accepted by provider commit causality. The
  bounded delivery source tracks those admitted bytes through publication and
  eviction; a batch-count ceiling alone is not a memory bound;
- every matching typed emission in one committed batch is delivered exactly
  once and in batch order. A lease may pause within a batch at its admitted
  buffer ceiling, but it cannot advance the source cursor past undelivered
  matching causes;
- the bank activity facade binds each delivered typed cause to its exact
  journal identity, account identity, and account posting sequence and performs
  one fresh, authorized, one-item projection for that conjunction. A
  counterparty posting may have the same per-account sequence and is skipped,
  not mistaken for absence. The lease retains the cause until projection succeeds;
  an oldest-first bounded activity snapshot that omits the cause cannot be
  relabeled as that commit's live update;
- a caller may narrow the installed live buffer ceiling but cannot enlarge it
  or manufacture queue capacity outside the installed execution contract; and
- current permission dependencies are re-evaluated before every delivered
  projection, so revocation closes or narrows the lease before the next
  unauthorized event. Queued payloads carry no authority past that check.

**5.5 — Public consumer closure**

- consumer transcripts compile and run through bank-domain types and the
  `worth-query-decl` / `worth-query-host` audience path only;
- no consumer assembles runtime identities, policies, read sets, provider
  sessions, receipts, or support snapshots; and
- the transcript covers every read, mutation, workflow, explanation, history,
  and live family plus cancellation, deadline, consistency, idempotency,
  bounded work, queue overflow, and live revocation.

**Destination module skeleton**

```text
workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/
    ordinary_read/
        controls.rs
        outcome.rs
        projection.rs
    ordinary_mutation/
        controls.rs
        outcome.rs
        progression.rs
    live_delivery/
        controls.rs
        lease.rs
        outcome.rs
        provider_causality.rs

workspaces/worth-query-bank-world/crates/bank-server/src/ordinary/
    read/
        account_discovery.rs
        account_detail.rs
        account_access.rs
        activity.rs
        payment.rs
        institution_audit.rs
    mutation/
        account_access.rs
        account_creation.rs
        business_payment.rs
        money_movement.rs
        reversal.rs
    workflow/
        continuation.rs
        explanation.rs
    live/
        account.rs
        activity.rs
```

Files may be combined only where they retain one semantic responsibility and
remain within the workspace line cap. The stable axes above are not flattened
into `helpers`, `common`, route-local policy, or a second mutable bank runtime.

**Proof before Bank World Phase 4**

Consumer transcript tests compile and run using only `worth-query-decl` and
`worth-query-host`. No consumer assembles runtime identities, policies,
read-sets, provider sessions, receipts, or support snapshots.

### Bank World Track — Phase 4: Estate, Capability, And Emergency-Access World

**Requirement**

Extend the bank into a production-shaped estate and exceptional-access world
that forces capability composition, conflict-of-interest, field disclosure,
delegation, elevation, review, and aftermath to become real domain semantics.
This phase declares the bank meaning and hostile worlds; it does not create a
bank-local authorization or history engine.

The Phase 4 commands remain typed bank-domain semantics rather than installed
executable application operations. Runtime Hardening Phase 7 must install
their capability, purpose, disclosure, conflict, and elevation contracts
before any host can obtain execution authority for them.

**Must establish**

- typed death notice, estate case, executor, beneficiary, joint owner,
  authorized signer, branch, estate specialist, compliance, legal-authority,
  capability-grant, emergency-access, and mandatory-review entities or
  relationships at the narrowest truthful domain boundary;
- capability grants for exact account, estate, institution, branch, operation,
  purpose, field, amount, validity, delegation, and workflow-stage scopes;
- deny, conflict-of-interest, separation-of-duty, distinct-actor, and
  beneficiary-exclusion policy contributions;
- restricted customer identity, beneficiary, document, account, posting, and
  audit fields with domain-owned disclosure classifications and typed
  application result shapes;
- death notification, account freeze, case opening, executor recognition,
  capability delegation/revocation, emergency request/approval/revocation,
  mandatory review, estate release, and estate disbursement operations;
- the branch-manager/beneficiary world in which ordinary employee authority,
  beneficiary status, delegated authority, and emergency elevation remain
  separately attributable and cannot compose into self-benefiting power;
- bank aftermath declarations identifying compensatable money movement,
  explicitly reversible authorization changes, reconcilable external effects,
  and irreversible audit or legal decisions; and
- independent estate, capability, disclosure, conflict, and accounting oracles
  that do not call the production policy or projection implementation.

**Proof before Runtime Hardening Phase 7**

The bank domain can state every legal and illegal case without a superuser role,
route predicate, redaction callback, mutable permission map, generic metadata
bag, or local undo stack. The conflicted branch manager cannot read restricted
estate fields, approve their own elevation, disburse estate funds, or combine
grants to do so; a causally valid non-conflicted path remains expressible.

### Runtime Hardening Track — Phase 6: Installed Application Queries, Bases, And Continuations

**Requirement**

Join schema-bound application meaning to Query's existing canonical read,
collection, graph-read access-planning, cursor, history, live, and preview
machinery so an application declares one query rather than hand-wiring
parallel lane-specific projectors or access paths.

Runtime Phase 6 is implemented through the ordered internal proof gates
6.1-6.9 below. They are not parallel backlog categories: a later gate may
consume only authority and guarantees already proved by every earlier gate.
Discoveries that strengthen a completed gate become an append-only corrective
phase or milestone and block unfinished dependents rather than becoming ad hoc
work inside the current gate.

The durable requirement and finding states for these gates live in
[`milestone-9.16-runtime-phase-6-closure-ledger.md`](milestone-9.16-runtime-phase-6-closure-ledger.md).
The `R6.*` and `Q6.*` identifiers below have no closure meaning outside that
ledger.

**Must establish**

- installed application-query identity with typed parameters, admitted root,
  result shape, cardinality, predicates, ordering, dependency ceiling,
  disclosure contract, basis support, and lane eligibility;
- one ready Foundational canonical-basis artifact for that complete portable
  query meaning, prepared under a named Query domain and rule version, with
  application-query and installed-graph digests derived only from that basis;
  the basis and digest remain descriptive inputs to Query's stronger installed
  authority and never replace it;
- one installed binding from that identity to the existing canonical
  `WorthQueryReadFamily` / `WorthQueryReadGraph` meaning and Milestone 9.10
  graph-read access-planning proof chain;
- parameter-bound predicate normalization and selectivity, plus
  `PredicateSupport`, `OrderingSupport`, traversal, result-buffer, and
  lifecycle requirement rows derived before execution rather than
  rediscovered by an application-query executor;
- domain-owned projection or derived-field semantics behind that installed
  contract without host-local execution dispatch becoming query authority;
- Query-owned opaque continuation bound to query identity, parameters, scope,
  basis, ordering, result shape, and compatibility generation;
- fresh installed scoped-authorization admission on every page, history
  request, live delivery, and preview read; a cursor or prior result carries
  no authority;
- a typed access-context input that Runtime Hardening Phase 7 can strengthen
  with capability, purpose, disclosure, conflict, and elevation proof without
  changing canonical query identity;
- explicit caller controls for current or pinned basis, consistency, freshness
  posture where supported, deadline, cancellation, result count, and work;
- one canonical result meaning across one-shot, historical, live, and admitted
  preview lanes, with typed support or unsupported posture per lane;
- consumption of one admitted graph-read access plan on every covered one-shot,
  continuation, historical, and preview execution, with separately admitted
  live-maintenance posture for the same canonical read meaning;
- expected-version and expected-fact mutation preconditions lowered into exact
  provider-recompared decision facts;
- governed read-access receipts carrying the Milestone 9.10 plan,
  requirement-set, support, strategy, fallback, edge-scan, and
  per-result-neighbor-lookup evidence alongside typed field omissions;
- explicit lowering of receipt work counters and allocation posture into
  Foundational performance vocabulary only at inspection, support, or
  publication boundaries; the ordinary receipt remains Query-owned and no
  Foundational report materialization enters the execution path; and
- deletion or migration of application-local cursor identity, lane-specific
  query meaning, host pagination, host-side post-read filtering or sorting,
  repeated child queries, per-result relation lookup, and duplicated
  one-shot/live/history projectors where Query owns the generic contract.

**Destination module skeleton**

```text
worth-query-declaration/src/application_query/
    canonical_basis/
        mod.rs
        definition.rs
        result_shape.rs
        entry.rs
    definition.rs
    parameters.rs
    invocation.rs
    result_shape.rs
    ordering.rs
    authorization_requirement.rs
    basis_support.rs
    lane_eligibility.rs
    continuation.rs
    live_cause.rs

worth-query-installation/src/application_query/
    canonical_basis/
        mod.rs
        graph.rs
        planning.rs
        continuation.rs
        installed_query.rs
    authority_seal.rs
    installed_contract.rs
    read_family_binding.rs
    planning_contract.rs
    graph_access_contract.rs
    disclosure_contract.rs
    continuation_contract.rs
    live_contract.rs
    lane_support.rs

worth-query-admission/src/application_query/
    parameter_binding.rs
    parameter_canonical_basis.rs
    requirements.rs
    runtime_support_review.rs

worth-query-execution/src/domain_computation/primary_graph/application_query/
    request.rs
    admission.rs
    graph_read_plan_binding.rs
    access_context.rs
    controls.rs
    authorized_read.rs
    read_execution/
    projection/
    one_shot/
    continuation/
    historical/
    preview/
    live/
    access_receipt/
    resource_lifecycle/

worth-query-declaration/src/application_schema/
    mutation_precondition.rs

worth-query-installation/src/application_operation/
    precondition_contract.rs

worth-query-execution/src/domain_computation/primary_graph/application_attempt/
    precondition_binding.rs
    provider_recomparison.rs

worth-query-bank-world/crates/bank-domain/src/queries/
    account/
    payment/
    estate/
    audit/

worth-query-bank-world/crates/bank-server/src/application_query/
    request.rs
    outcome.rs
```

The lane files are committed sibling responsibilities with different lifecycle
and truth-basis posture. They bind lane progression to the existing canonical
read graph and admitted graph-read access plan; they are not separate query
engines. Implementation creates only populated responsibilities but may not
flatten them into one mode-switched executor.

`request.rs` owns the generic typed host progression from a domain invocation
through installed resolution, mapped principal, typed scope, controls, and lane
selection. Bank query-family modules adapt bank identities and result types but
may not repeat the install-admit-execute sequence or dispatch through local
markers. `read_execution/` owns the shared bounded non-live kernel;
`access_receipt/` owns governed observations and never reusable authority; and
`resource_lifecycle/` owns terminal release of admitted plan, basis, buffer,
and lane resources. The mutation-precondition files are operation authority,
not application-query meaning, and therefore remain outside
`application_query/`.

The current mature preview machinery does not authorize a second
application-query preview engine. Runtime Phase 6 must bind the application
query preview lane to the existing Runtime Bridge preview-session authority
through a lower-layer contract consumed by `worth-query-execution`. If the
required proof is currently monolith-owned, Phase 6 moves the minimum owning
authority boundary downward; it may not copy the proof, import the monolith
into execution, or substitute descriptive preview identity. The bank reference
world must prove one admitted estate preview query. Queries or providers that
do not declare or install preview support return typed unsupported posture
before execution.

#### Runtime Phase 6.1: Canonical Query Identity And Installed Authority

Runtime Phase 6.1 closes the complete identity and installation boundary before
later planning or lane work may rely on it.

It must:

- preserve one canonical query identity across declaration, schema, package,
  installed read graph, installed query, and compatibility generation;
- lower every identity-bearing semantic dimension into one ready Foundational
  canonical basis before deriving any compact digest through Foundational's
  admitted sequence-digest lane; use typed canonical values and stable loci
  rather than debug formatting, joined strings, raw-byte hashing, or a second
  hand-written hash grammar;
- complete and consume Foundational's admitted production SHA-256 sequence
  slot before any compact Query identity relies on it; the deleted fixture
  digest and deferred cryptographic-debt surface must have zero source,
  documentation, and compile-surface residue;
- separate those descriptive digests from Query-owned installation authority:
  the installed index owns a cryptographically random, redacted root secret;
  derives generation/package authority keys through domain-separated
  HMAC-SHA-256; seals installed application queries, operations, abilities,
  principal bindings, and conditional dependencies with family-specific typed
  transcripts; and verifies seals in constant time;
- preserve exact authority lineage across a same-generation rebuild, rotate
  derived package authority across a successor generation, and reject an
  independently rooted index as foreign authority even when its public runtime
  ordinal and installed meaning are otherwise equal;
- keep the ready canonical basis and derived digest descriptive while only
  Query's installed proof-carrying artifact authorizes resolution or
  execution;
- derive each newly installed or rebuilt query identity at the installation
  seam exactly once, retain the typed fixed-width digest in the installed
  artifact, and expose counters for basis preparations, digest derivations,
  and canonical bytes encoded;
- make query marker, parameter and result types, typed scope, result slots and
  paths, cardinality, predicate and ordering meaning, dependency ceilings,
  disclosure, basis support, authorization requirement, and lane eligibility
  identity-bearing;
- keep parameter values, request limits, deadline, cancellation, and chosen
  supported basis outside canonical query identity;
- reject undeclared or host-authored definitions, changed query/parameter/
  result/scope types, foreign schema, foreign runtime, stale generation,
  package drift, schema-meaning drift, and forged descriptive identity before
  execution authority; and
- preserve exact rebuilt authority without treating a rebuild as drift.

Proof requires an independent one-axis identity inventory, structured
Foundational canonical comparison for independently produced meaning,
convergence twins for non-identity controls, typed digest-slot admission,
a standard HMAC-SHA-256 vector, injected entropy-failure denial, secret
redaction, wrong-key, cross-family, changed-field, optional-field collision,
same-root rebuild, successor-key rotation, and independently rooted authority
hostility,
a mutation-sensitive residue check against direct SHA/debug-string identity
construction, public compiler denial for cross-schema or host-definition
substitution, same-runtime dynamic hostility for package and generation drift,
and exact counter evidence showing one preparation and one digest derivation
for each newly installed identity artifact and zero rederivation during
resolution or execution. Runtime Phase 6.1 is complete only when ledger row
R6.1 is `PROVED`; a collection of individually closed findings is not a
substitute.

#### Runtime Phase 6.2: Canonical Planning And Runtime Support Authority

Runtime Phase 6.2 joins installed application queries to the one mature graph
read planning chain. It must not stop at shared enum names or equivalent
requirement rows.

Both mature and application query sources must pass through:

```text
canonical graph meaning
    -> requirement derivation
    -> runtime-owned support inventory
    -> cost estimate and budget
    -> plan review
    -> lower execution authority
```

Cross-source one-axis twins must cover relation presence, direction and depth,
predicate posture, ordering, cardinality, fanout, result pressure, and lane
lifecycle. Source-specific graph digest, schema authority, maximum-cardinality
evidence, and installed mechanism may remain truthfully distinct.

The runtime inventory is derived only from installed provider mechanisms.
Caller-authored support rows, inventories, requirement digests, budgets, or
plan reviews are descriptive and cannot mint or substitute executable
authority. Deleting or mismatching an equality index, ordering mechanism,
traversal mechanism, lifecycle mechanism, or work budget must deny at plan
review rather than widen into scanning, host work, or local caching.
Budget dimensions remain semantically disjoint: inline index bytes cover
adjacency, workset, visited, deduplication, predicate, and ordering structures;
proof bytes and result-buffer bytes remain separately estimated; total-memory
evidence includes all three. Proof carriage cannot consume index capacity, and
excluding proof from the index subtotal cannot remove it from total accounting.
The execution-runtime installer owns a typed application-query resource
profile for the operating world. The profile bounds inline index bytes,
per-result buffer bytes, and intermediate-set capacity independently. Request
controls may narrow result and work limits but cannot widen the installed
profile; query declarations, packages, hosts, and callers cannot author a
budget that substitutes for runtime policy. The selected profile and its
intersection with request limits must be visible in plan-review evidence and
must survive runtime-generation progression without becoming digest-only
authority.

Proof closes R6.2 and R6.3 plus every planning/support finding assigned to this
gate.
Source and dependency residue must independently reject a second planner,
copied requirement vocabulary, generic inventory substitution, or lane-local
support engine.

#### Runtime Phase 6.3: Shared Execution Kernel, Projection, Receipts, And Buffers

Runtime Phase 6.3 establishes one lower non-live execution kernel consumed by
one-shot, continuation, historical, and preview lanes.

The kernel owns bounded root selection, batched forward and reverse traversal,
optional-one/exactly-one/many cardinality enforcement, path-and-slot-keyed
result construction, installed ordering consumption, exact work charging,
projection integrity, result-buffer accounting, basis release, and receipt
construction. Lane modules own lifecycle admission and legal next actions but
may not reimplement those semantics.

Canonicalization is cold installation work. The kernel and domain projection
surface consume compact installed slot/path authority and must not derive a
digest, prepare or compare canonical basis, format semantic identity, or
materialize a Foundational report. Result-buffer capacity is charged before
Query allocates or clones owned result material; a post-allocation overflow
check is accounting evidence, not a bound.

Missing truth, scalar incompatibility, cardinality violation, repeated branch
identity, corrupted projection, result overflow, projection denial, abandoned
plans, and nested-buffer failure must produce exact typed outcomes and release
every owned resource. Phase 6 receipts identify the admitted plan,
requirements, support, strategy, provider and basis generations, predicate,
adjacency, ordering and continuation-seek work, total work, result and
truncation counts, fallback count, per-result-neighbor lookup count, and typed
`NoOmission` posture. Runtime Phase 7 later adds governed disclosure omissions;
Phase 6 may not fabricate them.

Proof closes the kernel, projection, receipt, buffer, and compiler-progression
portions of R6.4, R6.9, R6.11, and R6.13. A permissive domain projector,
flattened branch map, lane-local materializer, or unmeasured nested allocation
must make the evidence fail. Deleting pre-allocation charging, adding selector
hashing, or replacing executed Query counters with a Foundational planning
claim must also make the evidence fail.
A scale-sensitive twin that increases roots, edges, candidate rows, nested
results, and projected fields independently must leave kernel basis-
preparation, digest-derivation, and digest-text counters at exact zero.

#### Runtime Phase 6.4: Access Context, Bases, Continuations, And Interruption

Runtime Phase 6.4 completes the execution-owned access and continuation
boundary.

Every request carries the exact mapped principal, typed installed scope, fresh
scoped authorization, coherent basis/consistency/freshness posture, deadline,
cancellation, result bound, and work bound. Query owns the constructors that
make incoherent control tuples unrepresentable.

An opaque continuation retains descriptive query, parameter, scope, basis,
ordering, result-shape, provider, and generation identity only. It is
move-only, holds no authorization or runtime resource, and can be consumed only
by the owning typed query runtime. Resume reacquires the exact basis and reruns
installed-query, parameter, scope, provider, authentication, interruption, and
authorization admission inside the same protected execution boundary used by
ordinary reads.

Request-bound parameter identity is itself a retained ready Foundational
canonical-basis artifact with a named Query domain and rule version. Resume
prepares one fresh artifact and compares the two ready artifacts once through
Foundational structured comparison before support review or basis acquisition;
it does not compare compact digest text, reconstruct query meaning, compare
`Debug` strings, or canonicalize again after plan authority exists.
The installed continuation contract bounds parameter-entry count before
preparation and enforces the canonical encoded-byte budget during bounded
preparation, rejecting before either budget can be exceeded. Resume evidence
reports exactly one fresh parameter-basis preparation, the bytes encoded by
that preparation, and zero digest derivations unless the installed contract
explicitly requires a compact parameter key; any such key is derived once
during the same admission and retained afterward.

Proof includes changed-parameter, stale-installed-query, wrong-provider,
cross-query, cross-scope, foreign-runtime, stale-principal, stale-scope,
authorization-revocation, authentication-expiry, basis-expiry, cancellation,
deadline, result-exhaustion, and work-exhaustion twins. Independent resource
observation must show retained or disposed continuations consume no basis,
provider reservation, buffer, or live capacity. Authoritative live counts and
retained-byte observations use checked acquire/release transitions so overflow
or underflow cannot wrap into a false baseline; lifetime-only diagnostic totals
may saturate. This phase closes R6.5 and R6.6.

#### Runtime Phase 6.5: Historical, Preview, And Live Lane Completion

Runtime Phase 6.5 proves lifecycle parity after the shared planner, kernel, and
access boundary are closed.

Historical is a named installed lane with explicit support, basis, consistency,
and receipt posture; it is not merely current one-shot execution with a
different label. Preview consumes the Runtime Bridge preview-session authority
described above and the same non-live kernel. Live consumes separately admitted
maintenance support for the same canonical read meaning and uses targeted
installed selection rather than a snapshot reread or host filter.

The application runtime owns the Bridge-to-Relational authority join. Historical
admission materializes a typed Bridge truth view through that runtime's installed
Bridge, resolves the observed snapshot against that runtime's Relational source,
and retains one exact move-only execution basis. Preview admission additionally
requires an active typed preview session opened by the same application runtime
before it may mint that exact Query basis. A branch identity, preview-session
identity, truth-view digest, snapshot identity, copied version, foreign Bridge
handle, or Foundational descriptive artifact is not Query basis authority and
must open no historical or preview execution path. The resulting bounded basis
may delegate the exact admitted snapshot into the shared kernel, but it cannot
authorize another snapshot, lane, runtime, provider, schema generation, or
query.

Every supported lane preserves the same installed query, scope, result shape,
membership, projection, ordering, authorization requirement, and disclosure
contract. Every delivery or execution receives fresh current authorization.
Cancellation, deadline, revocation, source closure, overflow, denial, explicit
close, and abandoned disposal return every owned resource to baseline.

Account activity is the primary bank historical-parity courtroom. A
framework-owned query fixture proves admitted preview authority before any
bank-specific preview workflow exists. Unsupported preview or advanced access posture denies
explicitly and cannot silently use the current lane. The real estate preview
courtroom is completed in Runtime Phase 6.7, where the estate query is first
declared and migrated; Phase 6.5 must not invent an earlier host-local estate
query merely to anticipate that proof. This phase closes the account-activity
and generic lane portions of R6.7. The estate-preview portion remains
explicitly open until Runtime Phase 6.7.

#### Runtime Phase 6.6: Account And Payment Query Migration

Runtime Phase 6.6 replaces the bank's application-local account and payment
read authorities with installed domain queries.

The migration includes account discovery, account summary, account detail,
authorized account users, pending payments, payment detail, and the already
started account activity family. `bank-domain` owns query definitions,
parameters, typed scopes, result shapes, abilities, and domain projection.
Query owns installed resolution, admission, planning, execution, pagination,
ordering, and receipts. `bank-server` may adapt bank identities and public
outcomes but may not dispatch through a local marker, repeat the generic
install-admit-execute sequence, or become projection authority.

Caller result count limits bound consumer-visible root rows or the explicitly
continued collection. They must not be reused as a cumulative ceiling over
internal nested dependency records needed to construct one result. Nested
dependency expansion remains bounded by the admitted work limit, the installed
dependency ceiling, and the Query-owned result-buffer reservation. A
one-result account summary must therefore remain lawful when its balance
depends on multiple postings, while insufficient work or buffer capacity still
denies before an unbounded materialization can escape.

Account discovery declares a typed union of bounded scope-to-root relation
paths. Every hop preserves schema endpoints and direction at compile time;
installation makes the complete path set, union/dedup posture, root ordering,
and dependency ceiling identity-bearing. Execution begins only from the
admitted principal scope, traverses those installed paths, deduplicates by
entity identity, orders the combined root collection through the installed
root ordering, and applies result/work bounds before broad materialization. A
host or domain projector may not union, deduplicate, sort, truncate, enumerate,
or redact account roots after Query execution.

Pending payments strengthens the same substrate with typed equality guards on
intermediate and terminal path entities. The declared path fixes
`AuthorizationRole == Approver` before traversing from an authorization to its
account and fixes `PaymentStatus == ApprovalRequired` after reaching a payment.
Each guard carries its exact entity, aspect, field, scalar family, and typed
literal value in canonical identity and schema closure. Planning derives its
predicate support before execution. Query evaluates guards against the bounded
frontier as a batch under the admitted work and result-buffer contracts; a
host filter, caller-selectable role/status parameter, global unrelated index
enumeration, or per-candidate field lookup is forbidden.

Proof uses the public caller progression and deletes the corresponding legacy
query declarations, executor branches, cursor identities, repeated child
reads, post-read filtering and sorting, and duplicate projectors. Every nested
account and payment view reports zero fallback and zero per-result neighbor
lookups. A payment detail whose installed shape requires several relation
indices plus one bounded proof artifact remains within the inline index budget
when the index subtotal fits; the proof artifact remains present in exact
proof-byte and total-memory evidence.

#### Runtime Phase 6.7: Estate, Actor, And Audit Query Migration

Runtime Phase 6.7 adds the complex reference queries needed to prove the front
door is not specialized to account activity.

Installed queries cover estate cases and accounts, estate documents and their
relationships, relevant actor and employee assignments, capability and
conflict context, and institution audit views. Phase 6 owns canonical result
shape, scope, traversal, required ability, basis, lane support, and access
receipt. Runtime Phase 7 owns purpose-bound capability composition, field
disclosure, conflict resolution, elevation, and governed omission; Phase 6
must leave an explicit typed strengthening boundary rather than anticipating
those decisions.

The complex estate and audit queries consume the Runtime Phase 6.2
execution-runtime resource profile. Their admission must prove that the
installed operating-world ceiling admits their exact graph-read estimate.
Increasing request work or result limits cannot make an over-profile query
executable, while a deliberately larger installer-owned profile may admit the
same canonical query without changing query identity. Bank-local plan review,
caller-supplied index bytes, or a query-specific budget bypass is forbidden.

Proof executes nested account, payment, estate, actor, and audit views through
public facades with exact admitted-plan evidence, zero fallback, and zero
per-result neighbor lookups. The real estate query must also execute through an
admitted preview session, completing the second bank lifecycle courtroom
without changing its canonical query meaning. Adding Phase 7 authority must
not change query identity or result selectors. Runtime Phases 6.6 and 6.7
together close R6.10 and the remaining estate-preview portion of R6.7.

#### Runtime Phase 6.8: Provider-Recompared Mutation Preconditions

Runtime Phase 6.8 exposes typed expected-version and expected-fact controls on
the public application operation contract and lowers them into the existing
provider decision-fact comparison authority.

The host supplies typed expectations but performs no read-check-write
sequence. Installation binds the allowed fact family and scope. Admission
binds caller values to the exact operation attempt. Provider execution
recompares the fact atomically at commit and records comparison evidence in the
mutation receipt.

The declared precondition targets and the attempt-bound expected values use
Foundational-native aspect locator and value meaning. Admission prepares one
ready canonical basis for the exact precondition set under a named Query
domain and rule version, then derives the compact idempotency-intent digest
through Foundational's admitted sequence-digest lane. The ready basis and
derived digest remain descriptive inputs retained by the stronger Query
attempt artifact. Provider commit compares the already-bound typed locators and
values directly; it performs no canonicalization, digest derivation, digest
comparison, or semantic string formatting on the warm path.
The installed operation contract bounds precondition count before preparation
and enforces total canonical encoded bytes during bounded preparation, before
unbounded allocation or hash work can escape. Admission reports one basis
preparation and one digest derivation for the admitted precondition set.
Idempotency resolution, response-loss retry, provider recomparison, and commit
consume that retained result and report exact zero additional preparations,
derivations, or digest-text materializations.

Proof uses an ordinary bank mutation: a matching precondition commits,
relevant drift denies atomically, unrelated drift remains admissible, and a
lost response plus retry cannot move money twice. Removing provider
recomparison, changing one precondition locus or value without changing
structured canonical comparison, bypassing Foundational digest admission, or
moving the check into `bank-server` must make the evidence fail. Residue
evidence rejects a private precondition hash grammar and a digest-only provider
comparison. This phase closes R6.8.

#### Runtime Phase 6.9: Cutover, Residue, Warm Paths, And Certification

Runtime Phase 6.9 is a proof and deletion phase, not a place to finish product
behavior discovered late.

It must:

- inventory every bank query family and supported lane;
- delete remaining local query markers, cursor identities, mode switches,
  projectors, host filtering/sorting/pagination, repeated child reads, copied
  support, and internal Query imports;
- prove compiler-visible progression from package declaration through
  installation, parameter binding, access admission, plan review, basis,
  canonical execution, continuation, and governed receipt;
- keep compiler certification consolidated rather than creating one target per
  query or hostile dimension;
- preserve narrow declaration, installation, admission, execution, and bank
  consumer feedback paths;
- prove the Foundational SHA-256 slot once at its owning lower layer and prove
  Query's query, parameter, precondition, capability, disclosure, and
  aftermath families consume that slot rather than cloning its mechanics;
- prove the Query-owned HMAC-SHA-256 installation-authority family separately:
  root entropy is fallible, secrets are typed and redacted, family transcripts
  are domain-separated, verification is constant-time, rebuild continuity and
  successor rotation are exact, and no secret-prefix SHA construction remains;
- run exact formatting, patch hygiene, line-cap, strict changed-crate Clippy,
  boundary-check, generated agent-context, public consumer, and cold courtroom
  evidence; and
- reject ad hoc application-query hashes, duplicate canonical grammars,
  direct digest derivation for cross-crate semantic meaning even when its input
  came from canonical material, warm-path basis/digest work, duplicate
  aspect-mask vocabulary, and Foundational artifacts promoted into Query
  execution authority;
- reject deterministic or caller-reconstructible installation keys, raw
  authority-key arrays in proof artifacts, secret-bearing `Debug`, direct
  equality of MAC text, and family seals that can validate under another
  family's transcript;
- certify phase-separated canonical-work counters: bounded installation and
  request-admission counts, exact-zero execution/provider/live/recovery counts,
  canonical-byte budgets enforced before work escapes, and no textual digest
  materialization outside an explicit boundary;
- run independent scale-axis twins in which query roots, graph edges,
  candidates, result rows, projected fields, policy facts, and live fan-out
  grow while the number of installed or freshly admitted semantic identities
  is held constant; canonical-basis and digest counts must remain constant, and
  every fan-out lane must remain at exact zero;
- measure canonical encoding/allocation separately from SHA-256 compression so
  a cheap digest primitive cannot conceal an unbounded materialization cost;
- reject wrapping lifecycle counters or debug-only underflow checks wherever
  terminal resource-baseline evidence depends on those counters; and
- revise the canonical Query AI README and its referenced feature documents so
  the installed application-query front door, supported lanes, controls,
  denials, receipts, and migration rules match production.

Runtime Phase 6 is complete only when every R6 row is `PROVED`, every Q6 finding
is `CLOSED`, the ledger survives a skeptical independent audit, and no high- or
critical-impact finding remains. A green broad test command or a relabeled
legacy path is not closure.

#### Runtime Phase 6 Caller DX

The generic public progression, not one bank-local hand-written wrapper per
query, must support:

```rust
let first_page = bank
    .query(queries::account_activity(account))
    .as_principal(&principal)
    .at(ReadBasis::Current)
    .limits(QueryLimits::new(50, 10_000)?)
    .run()?;

let next_page = bank
    .query(queries::account_activity(account))
    .as_principal(&principal)
    .resume(
        first_page.continuation()?,
        QueryLimits::new(50, 10_000)?,
    )
    .run()?;

let historical = bank
    .query(queries::estate_activity(estate))
    .as_principal(&principal)
    .at(ReadBasis::Historical(version))
    .run()?;

let preview = bank
    .query(queries::estate_activity(estate))
    .as_principal(&principal)
    .at(ReadBasis::Preview(preview_session))
    .run()?;

let live = bank
    .query(queries::account_activity(account))
    .as_principal(&principal)
    .live(LiveControls::bounded(64, 10_000)?)
    .open()?;
```

The exact private mechanics may differ, but implementation may not require the
caller to resolve an installed definition, construct support evidence, choose
an executor, interpret a mode string, perform host sorting or filtering, or
rebuild the projection for another lane.

**Proof before Runtime Hardening Phase 7**

The same installed account or estate query has identical result shape,
ordering, declared access-context boundary, and scope across every supported
lane. Foreign, cross-query, stale-basis, wrong-order, or wrong-generation
cursors deny before projection. Unsupported search, Store, preview, or
access-product posture cannot silently widen into a scan, host loop, or local
cache. Predicate and ordering changes produce the correct Milestone 9.10
selectivity, requirement, inventory, and access-plan evidence. Nested account,
payment, estate, and actor views prove exact-zero covered per-result neighbor
lookups and zero fallback across ordinary and lane-specific receipts. Phase 6
does not fabricate the richer capability and disclosure proofs owned by Phase
7.

### Runtime Hardening Track — Phase 7: Capability, Disclosure, And Governed Elevation

**Requirement**

Make capability-based authorization powerful enough for medical-grade
disclosure and emergency access while preserving exact graph authority,
conflict-of-interest, currentness, and fail-closed phase progression.

Runtime Phase 7 is implemented through the ordered internal proof gates
7.1-7.7 below. They are not parallel policy features: each later gate consumes
the installed meaning and authority proved by the earlier gates. A discovery
that strengthens a completed gate becomes an append-only corrective phase or
milestone and blocks unfinished dependents rather than relabeling completed
history or becoming a local exception in the current gate.

The durable requirement and finding states for these gates live in
[`milestone-9.16-runtime-phase-7-closure-ledger.md`](milestone-9.16-runtime-phase-7-closure-ledger.md).
The `R7.*` and `Q7.*` identifiers below have no closure meaning outside that
ledger.

**Must establish**

- typed semantic capability identity distinct from role, relationship,
  authentication, policy result, and operation authority;
- scope dimensions for action, resource, relation, field, purpose, amount,
  cardinality, workflow stage, validity, delegation, grant provenance, and
  application-defined constrained context;
- compiler-visible installed composition of allow, deny, conflict,
  separation-of-duty, distinct-actor, delegation, and disclosure predicates;
- delegation that can narrow but never widen the grantor's exact authority,
  validity, disclosure, purpose, or downstream delegation posture;
- purpose-bound request context introduced at an explicit entry boundary and
  thereafter Query-carried without ambient or adapter-owned policy;
- an installed validity timeline and Query-owned current-time interpretation
  whose exact sample enters the decision facts; the application host may fix
  one trusted external time source when it publishes the Query application
  runtime, but operation callers and transport adapters cannot supply a
  sample, replace the source, or choose the moment at which a grant is
  evaluated;
- separate typed internal-computation and consumer-disclosure admission,
  including noninterference posture for protected facts that influence
  predicates, ordering, cursors, counts, aggregates, explanations, history,
  or live membership without appearing in the result;
- typed field disclosure and omission before consumer-visible projection or
  serialization, with no post-projection redaction path;
- Foundational projection-mask admission against the installed aspect contract
  as the legal field upper bound, with a separately admitted diagnostic mask
  for explanation material; Query capability, purpose, noninterference, and
  disclosure decisions remain the stronger authority that narrows those masks;
- break-glass typestate from requested, approved, active, expired or revoked,
  and review-required to reviewed, with exact requester, approver, reason,
  scope, purpose, fields, actions, time, and audit identity;
- an installed upper bound on what emergency elevation can authorize; elevation
  cannot bypass conflict, distinct-actor, invariant, irreversible-commit, or
  provider requirements;
- exact authorization and disclosure decision facts carried through commit,
  history, continuation, and every live payload; and
- re-admission of the strengthened access context on one-shot, continuation,
  historical, preview, and live query lanes without changing canonical query
  identity or result meaning;
- typed explanations that distinguish missing capability, scope mismatch,
  purpose mismatch, conflict, separation-of-duty, field omission, elevation
  required, elevation denied, elevation expired, and review required;
- explicit publication lowering into Foundational diagnostic categories,
  provenance, boundary-evidence receipt posture, and target-aware profile
  materialization without replacing the exact Query denial, decision facts, or
  executed receipt; and
- installation of the Bank World Phase 4 estate and emergency-access commands
  only after their complete capability, disclosure, conflict, delegation, and
  elevation contracts are compiler-visible and fail closed.

**Destination module skeleton**

```text
worth-query-declaration/src/application_capability/
    contract.rs
    scope.rs
    disclosure.rs
    delegation.rs
    delegation_transition.rs
    elevation.rs
    elevation_lifecycle.rs
    elevation_transition.rs

worth-query-installation/src/application_capability/
    canonical_basis.rs
    composition.rs
    conflict.rs
    separation_of_duty.rs
    distinct_actor.rs
    disclosure.rs
    delegation.rs
    elevation.rs

worth-query-execution/src/domain_computation/authorization/
    capability_admission.rs
    capability_decision_fact.rs
    retained_capability_support.rs
    authorization_revalidation/
        currentness.rs
    disclosure_admission.rs
    delegation_admission/
        discovery.rs
        transition.rs
    delegation_progression/
        binding.rs
        narrowing.rs
        support.rs
    conflict_admission.rs
    elevation_progression/
        request.rs
        approval.rs
        elevation_close.rs
        mandatory_review.rs
        upper_bound.rs

worth-query-execution/src/domain_computation/primary_graph/application_attempt/
    delegation_activation_program.rs
    capability_revocation_program.rs
    elevation_request_program.rs
    elevation_approval_program.rs
    elevation_close_program.rs
    mandatory_review_program.rs
    provider_execution/
        decision_facts.rs
        delegation_activation.rs
        capability_revocation.rs
        elevation_lifecycle.rs

worth-query-publication/src/application_authorization/
    disclosed_result.rs
    omission.rs
    explanation.rs
    boundary_evidence.rs

worth-query-bank-world/crates/bank-domain/src/schema/estate/
    capability_contract_installation.rs
    capability_elevation.rs
    operation_program_installation/
        lifecycle.rs
        delegation.rs
        capability_revocation.rs

worth-query-bank-world/crates/bank-domain/src/queries/estate/
    emergency_account_details.rs

worth-query-bank-world/crates/bank-server/src/
    application_query/governed_execution/
        estate_governance.rs
        elevated_restricted_estate.rs
    estate_progression/
        request.rs
        approval.rs
        close.rs
        review.rs
        delegation/
            activation.rs
            authorization.rs
            revocation.rs
```

The existing elevation declaration, progression, programs, provider owner, and
Bank lifecycle files remain. `retained_capability_support.rs` is created by
extracting the independently refreshable supporting-authority owner from the
broader capability decision; both elevation and delegation consume it without
erasing their distinct lifecycle bindings. `delegation_progression/support.rs`
is created for only the proposed child, exact parent/lineage, narrowing, and
target-timeline binding. `capability_revocation_program.rs`, its provider
counterpart, the emergency account-details query and governed executor, and the
Bank delegation `activation.rs` / `revocation.rs` siblings are committed
successor files. The current flat dirty delegation progression is replaced by
that directory rather than preserved as a second entry path.

Provider files own final comparison and effect admission, never policy
reconstruction. Bank's governed-query files are public composition roots; Bank
operation-program declarations describe domain effects but cannot construct
Query authority. Flat `helpers`, `common`, parallel Bank authorization
executors, and application-maintained copies of installed obligation meaning
are forbidden.

These are semantic ownership boundaries, not permission-themed bags.
`application_capability/` declares and installs application-defined authority
meaning. `authorization/` consumes installed meaning and current graph truth to
produce attempt-bound authority. `application_authorization/` can publish only
the already-admitted disclosed shape and typed omissions; it cannot inspect
protected values or make policy decisions.

#### Runtime Phase 7.1: Capability Identity, Scope, And Installed Composition

Runtime Phase 7.1 establishes the vocabulary and installed upper bounds that
every later authorization transition consumes. Capability identity must remain
distinct from a role name, authenticated principal, relationship, policy
result, operation identity, or runtime proof.

It must:

- declare typed action, resource, relation, field, purpose, amount,
  cardinality, workflow-stage, validity, delegation, provenance, and
  application-defined constrained-context dimensions without omitted
  dimensions silently becoming global;
- preserve required-versus-optional application field presence through schema
  identity and Foundational aspect lowering, so a capability dimension that is
  not applicable is represented by lawful absence rather than a sentinel value
  with different application meaning;
- bind validity fields to an explicit installed timeline interpretation that
  Query can sample without application-name inference or a caller-authored
  `now` value;
- identify the exact active grant-status predicate and bind grant workflow
  scope to the resource-side workflow field whose current value it constrains;
  current admission may not infer either binding from names, application
  convention, or adapter policy;
- install explicit allow, deny, conflict, separation-of-duty, distinct-actor,
  delegation, and disclosure composition contracts as canonical application
  meaning;
- prepare that portable installed meaning through one Foundational canonical
  basis and structured comparison lane before admitting digest derivation
  through Foundational's typed sequence slot and retaining the derived digest
  inside Query's compact identity; capability identity and digest evidence
  remain non-authority until Query installation binds them to schema, package,
  generation, and operation proof;
- bound installed capability-contract entry count and canonical encoded bytes,
  derive each newly installed capability identity once, and carry its compact
  typed identity through current admission rather than canonicalizing or
  hashing per grant, decision fact, disclosed field, result row, or live
  payload;
- bind those contracts to the same schema, package, generation, and operation
  identities used by the installed operating world; and
- make undeclared dimensions, incompatible scope composition, descriptive
  role substitution, and host-authored policy fail before runtime authority
  exists.

Proof requires the Bank World Phase 4 declarations to install through the
generic contracts, public compiler denial for authority-category substitution,
structured Foundational comparison plus independent one-axis scope twins
proving that every dimension participates in identity and narrowing, and
residue denial for direct hash/debug-string identity grammars or raw digest
derivation. Phase 7.1 is not complete while application meaning still lives
only in bank-specific match statements or test oracles.

#### Runtime Phase 7.2: Purpose-Bound Access Context And Current Admission

Runtime Phase 7.2 turns an authenticated, mapped principal and explicit
application purpose into attempt-bound authorization by consulting current
relational truth and the installed contracts from Phase 7.1.

It must:

- introduce purpose and constrained request context at the governed entry
  boundary and carry them through Query without thread-local, adapter-owned, or
  ambient policy;
- sample grant validity through the Query-owned time source and retain the
  exact trusted sample in the decision facts; a caller-supplied timestamp is
  descriptive input and cannot satisfy currentness;
- resolve current principal, relationship, grant, grant-validity, workflow,
  denial, and negative-membership facts through the canonical touched graph;
- bind every request-varying capability dimension to the exact admitted
  operation input or query parameters before that authority can open the
  consumer; a parallel caller-authored summary cannot understate amount,
  field, relation, purpose, cardinality, or constrained context;
- emit exact typed decision facts and the corresponding authorization read set
  so compare-and-commit, continuation, history, preview, and live progression
  can revalidate the decision; and
- deny a stale, revoked, foreign-runtime, foreign-operation, repurposed, or
  copied access context before it can authorize execution.

Proof requires independent relational and policy oracles, one-axis currentness
drift attacks, caller-time and underreported-input attacks, and compile-fail
evidence that neither authentication nor a descriptive capability value
satisfies the admitted access-context input.

#### Runtime Phase 7.3: Internal Computation, Disclosure, And Noninterference

Runtime Phase 7.3 separates permission to use a protected fact inside governed
computation from permission to disclose that fact to a consumer. The
separation must occur before result projection and serialization.

It must:

- derive the legal field universe from Foundational `AspectContract` and
  admitted `ProjectionMask` / `DiagnosticMask` artifacts rather than copied
  field-name sets; those artifacts constrain legal fields but carry no
  principal or Query execution authority;
- admit protected inputs for predicates, ordering, cursors, counts,
  aggregates, explanations, historical membership, preview results, and live
  membership independently from consumer field disclosure;
- construct a typed disclosed-or-omitted result shape without post-projection
  redaction or publication-time policy;
- ensure protected facts cannot be inferred through row presence, order,
  pagination boundaries, counts, aggregates, explanations, or live changes
  when the installed disclosure contract forbids that influence; and
- carry the exact disclosure decision facts and omissions with the result
  receipt while carrying no reusable authorization authority.

Proof requires adversarial paired worlds that differ only in a protected fact
and independently compare every observable consumer surface. Publication must
accept only the admitted disclosed shape, making accidental serialization of
an internal protected value unrepresentable. Mask-category substitution,
contract-incompatible field requests, and using a diagnostic mask as projection
authority must fail before result construction.

#### Runtime Phase 7.4: Delegation Narrowing, Provenance, And Revocation

Runtime Phase 7.4 installs delegation as a proof-carrying authority transition,
not as a copied grant with a parent identifier.

It must:

- require every delegated dimension to be equal to or narrower than the
  grantor's current scope, purpose, disclosure, validity, and downstream
  delegation posture;
- bind the delegated grant to the exact grantor, grantee, parent grant,
  provenance chain, schema, runtime, and generation;
- re-evaluate parent validity, revocation, relationship, and workflow truth
  whenever delegated authority is admitted; and
- make parent revocation or expiry invalidate every dependent admission without
  scanning unrelated grants or trusting a cached positive decision.

The later Bank activation command must consume this narrowing meaning without
collapsing it into command authorization. Authority to issue the delegation
command, the exact current parent authority, and the complete proposed child
upper bound are separate inputs. A child may be created or activated only by
an installed effect program after the parent, lineage, and every narrowing
dimension have been revalidated with fresh trusted time through idempotency and
provider commit. The installed activation contract must mechanically own its
complete graph-read and create/write/link obligation set; an application-owned
inventory beside that contract is not closure evidence. Revocation must be a
real status transition whose currentness consequence cuts off both the exact
grant and dependent descendants.

Proof requires depth and width attacks, copied-parent attacks, purpose and field
widening, validity extension, provenance substitution, and revocation after a
previously lawful admission. Warm admission work must depend on the declared
delegation chain and touched evidence, not total grant population. Query owns
the delegation transition; any portable provenance attachment lowers afterward
through Foundational provenance vocabulary and cannot be promoted back into a
grant or access context.

#### Runtime Phase 7.5: Conflict, Separation Of Duty, And Distinct Actors

Runtime Phase 7.5 enforces the installed combination rules that no individual
grant can express. A set of individually valid capabilities must not become
collectively authoritative when conflict or actor-separation meaning forbids
the combination.

It must:

- evaluate allow, deny, conflict, separation-of-duty, and distinct-actor
  predicates as one installed decision rather than ordered host checks;
- include conflicting-beneficiary, case assignment, prior actor, negative
  membership, and other relevant absence or presence facts in the decision
  read set;
- preserve exact actor identity across request, approval, mutation, disclosure,
  and review transitions; and
- make self-approval, conflicted benefit, role accumulation, and split-request
  attempts fail at the earliest governed boundary.

Proof requires complete combination-matrix coverage selected from the installed
rules, plus hostile sequences where relevant relationships or conflicts change
between admission and commit or delivery. The proof must avoid a Cartesian
product by covering each independent predicate, each interaction declared by
the contract, and each privileged transition boundary.

#### Runtime Phase 7.6: Governed Emergency Elevation And Mandatory Review

Runtime Phase 7.6 makes emergency elevation a narrow installed state machine
whose states expose only their legal next transition. The state machine is not
the authorization graph: transition-command authority and the governed
authority carried through the lifecycle are two independent installed axes.

It must:

- represent requested, approved, active, expired, revoked, review-required,
  and reviewed posture with exact requester, approver, reviewer, reason, scope,
  purpose, fields, actions, time, grant, and audit identity;
- bind each lifecycle role to one exact command capability and operation while
  preserving a separately authorized, immutable governed upper bound; a grant
  for the upper bound cannot authorize request, approval, revocation, or review,
  and a transition-command grant cannot widen or replace the upper bound;
- retain the governed upper bound as independently refreshable decision
  evidence and revalidate its exact grant, policy path, resource, purpose,
  scope, field, action, and Query-owned trusted-time currentness before every
  authority-increasing or authority-consuming approval, provider-commit,
  active-use, and delivery boundary; immutable meaning does not make an earlier
  allow decision timeless;
- authorize close or expiry and mandatory review through their independent
  command capabilities and exact lifecycle affinity without requiring the
  governed upper bound to remain positive; revoked or expired support must not
  strand authority-reducing cleanup or its legal review obligation;
- type and resolve the transition command target and the governed upper-bound
  target independently: their actions, purposes, resources, scope types, and
  contexts may differ, while exact lifecycle entity slots and the consumed
  receipt bind the command to the transition it may progress; equality of the
  command resource and governed resource is neither authority nor a required
  affinity rule;
- revalidate time-dependent lifecycle decisions after equivalent-idempotency
  recovery and before provider effects: request and approval must still be
  inside the exact request window, and close must still classify the same
  `Revoked` or `Expired` terminal state from fresh Query-owned time;
- require a non-conflicted distinct approver and preserve the ordinary
  capability, disclosure, invariant, irreversible-commit, and provider
  boundaries underneath the elevated upper bound;
- terminate delivery authority on expiry or revocation before the next page,
  history result, preview result, or live payload; and
- require and complete the exact mandatory review without allowing the active
  elevation proof or review obligation to be copied, skipped, or reused.

Published elevation and review outcomes use Foundational boundary category,
diagnostic, provenance, and profile posture only after Query has produced the
exact transition receipt. Requested/admitted/materialized profile progression
may narrow descriptive richness or retention explicitly, but it cannot widen
elevation scope, disclosure, purpose, or authority.

Proof requires the lawful request-approve-use-close-review sequence and hostile
self-approval, scope widening, purpose swapping, expired use, revoked use,
conflicted approval, command/upper-bound grant substitution, copied state,
repeated review, wrong-review selection, and forbidden disbursement attempts.

#### Runtime Phase 7.7: Bank Estate Cutover, Cross-Lane Re-admission, And Certification

Runtime Phase 7.7 installs the Bank World Phase 4 estate and emergency commands
only after Phases 7.1-7.6 have made their complete contracts enforceable.

It must:

- complete the real Bank consumer front door for every installed estate
  capability: the administration-governed estate query must supply and consume
  its installed capability instead of entering the generic ungoverned query
  executor, and every capability-governed mutation key must have a genuine
  installed operation program before its policy is claimed as consumer-proved;
- route ordinary, delegated, conflicted, disclosure-limited, and emergency
  estate operations through the public installed Query progression with no
  bank-local authorization executor;
- derive or verify every capability request dimension against the exact Bank
  command input or query parameter authority consumed downstream; Bank
  adapters cannot submit a narrower authorization summary beside a wider
  operation;
- keep estate-disbursement command authorization and effect integrity
  orthogonal: capability traversal is bounded by the exact estate, source
  account relation, purpose, and amount ceiling, while private governed-input
  identity additionally binds the operation variant, destination,
  beneficiary, and both ordered posting accounts and signed amounts;
- install estate disbursement as a distinct double-entry accounting program,
  not an alias for ordinary transfer: the combined pre-effect decision
  boundary must prove the open estate and its unique source account, distinct
  open source and destination accounts, revision-matched journal-derived
  balances, the selected estate beneficiary's joint ownership of the
  destination, and at least one recognized exact executor authority before
  emitting a balanced `EstateDisbursement` journal, two postings, both
  revision advances, and both account-activity effects; the Bank projection
  owns graph and account-snapshot truth while the pure domain proposal owns
  open/distinct-account and balanced-posting validation;
- treat the same-estate legal-authority set as fail-closed authoritative
  input: unrecognized records do not qualify, malformed candidate records deny
  rather than being hidden by a different valid candidate, and multiple valid
  candidates select one deterministic authority/executor pair whose exact
  estate, holder, and executor relations are reobserved before effects;
- use `AccountingRevision` as the exclusive currentness token for the
  journal-derived account aggregate only while installation proves that every
  lawful `PostingAmount` or `PostingAccount` writer is the shared money-
  movement program, creates immutable posting truth, and atomically advances
  every touched account revision; projection must require aggregate source
  count to equal revision, and provider comparison must stale a retained
  disbursement after any lawful posting change;
- preserve required-versus-optional field presence through application-query
  result declaration, canonical installation, disclosure, and domain
  projection; lawful schema absence is `None`, policy denial is a typed
  `Omitted`, present wrong-family values deny, and neither posture may be
  represented by a sentinel;
- project the administration-governed estate result as exact descriptive Bank
  grant, optional scope, parent-lineage, emergency-access, and mandatory-review
  meaning while keeping that result incapable of opening lifecycle or command
  authority without a fresh Query admission;
- re-admit the strengthened access context for one-shot, continuation,
  historical, preview, and live lanes without changing canonical query
  identity or result meaning;
- install one estate-scoped emergency-access activity query whose naturally
  many lifecycle children and exact lifecycle effect correspondence make all
  five lanes truthful; ordinary account activity or multiple differently
  governed queries cannot be composed to impersonate this proof;
- preserve exact authorization and disclosure facts through commit,
  continuation, history, preview, publication, and every live payload;
- remove or privatize superseded monolith and application-owned authority
  paths, then prove the destination dependency direction; and
- preserve bounded warm authorization work as unrelated grants,
  relationships, fields, cases, and consumers grow.

Phase 7.7 closes through ordered vertical gates. Gate A completes the public
Bank emergency journey before delegation expands the same authority machinery.
Gate B completes effectful delegation activation and revocation. Gate C proves
cross-lane re-admission and delivery cutoff. Gate D closes publication,
explanation, performance, oracle-removal, and residue evidence. A later gate
cannot lend closure to an earlier incomplete authority or effect boundary.

Gate A's decisive public courtroom is one production Bank journey:

1. request an exact restricted-estate field using separate command and governed
   upper-bound grants;
2. prove on a causal twin that revoking or expiring the exact governed support
   after request prevents approval even while an equivalent alternate grant
   remains current, while the lawful branch approves through a distinct,
   non-conflicted command-authorized actor;
3. consume the lawful approved elevation through the real public query facade
   for the exact field while a wider field and forbidden action deny;
4. revoke or expire the exact governed support after approval, deny later
   one-shot use without accepting an equivalent alternate grant, then close the
   elevation through its independently authorized close command;
5. independently read back the exact terminal elevation and mandatory-review
   state;
6. complete the exact review through a distinct authorized reviewer; and
7. deny repeated, copied, substituted, or wrong-review completion.

Every transition in that courtroom must cross command admission, lifecycle
authorization, invariant projection, an exact effect program, provider
compare-and-commit, and authoritative graph readback. A commit receipt without
independent poststate observation, a declaration-only command, or a no-op
program is not consumer closure.

The Gate A caller-facing target is the ordinary Bank query workflow with one
explicit approved-elevation input, not a lifecycle-only executor or a second
query API:

```rust
let requested = bank.request_estate_emergency_access(
    &requester,
    request_action,
    request_idempotency,
    &scope,
)?;
let requested = match requested {
    WorthQueryElevationRequestOutcome::Requested(receipt)
    | WorthQueryElevationRequestOutcome::AlreadyRequested(receipt) => receipt,
    stop => return publish_request_stop(stop),
};
let approved = bank.approve_estate_emergency_access(
    &approver,
    requested,
    approval_action,
    approval_idempotency,
    &scope,
)?;
let approved = match approved {
    WorthQueryElevationApprovalOutcome::Approved(receipt)
    | WorthQueryElevationApprovalOutcome::AlreadyApproved(receipt) => receipt,
    stop => return publish_approval_stop(stop),
};
let disclosed = bank
    .query(estate_emergency_account_details(estate, access))
    .as_principal(&requester)
    .controls(read_controls)
    .execute_with_approved_elevation(&approved)?;
```

`execute_with_approved_elevation` borrows the exact approved receipt and lowers
through the same installed query and lane progression as ordinary governed
execution. It can open only the exact requested field and purpose. It neither
consumes close authority nor manufactures a reusable access token.
The typed outcome matches are part of the contract: stale, denied, cancelled,
partial, and indeterminate postures must remain visible instead of being hidden
behind an unchecked success extractor.

Gate B must independently deny missing delegation-command authority and
missing, stale, or unlawful parent authority. It must bind the complete child
proposal into idempotency, refresh the exact parent and lineage before commit,
create or activate the child only after narrowing succeeds, perform an exact
`Active -> Revoked` transition, and prove immediate direct and descendant
cutoff through authoritative readback. The new semantic surface must re-prove
the inherited Milestone 9.16.1 obligation-closure, session, exact-read-set, and
provider-currentness contracts; historical closure of those contracts for
earlier surfaces is not evidence for delegation activation.

Gate C owns one canonical `EstateEmergencyAccessActivityQuery` across one-shot,
continuation, history, preview, and live cutoff. Its capability field is the
new highly restricted `EmergencyAccessActivity`, permitted only for
`EmergencyProtection`; it must not widen `PostingHistory` or borrow ordinary
`ViewAccount` authority. The query is scoped to one exact estate and projects
its naturally many emergency-access lifecycles through a direct authoritative
`EmergencyEstate` relation. Each activity projects only the access identity,
reason, status, issued-at and expires-at bounds, plus its exact mandatory-review
identity and status. Requester, approver, and reviewer identities remain outside
this result. Ordering is issued-at followed by access identity, and
continuation targets the many emergency-access relation rather than a synthetic
event log or a singular estate relation. Because those two fields control the
stable order consumed by every supported lane, their disclosure contracts
permit exactly `Ordering`, `Pagination`, `HistoricalMembership`, `Preview`, and
`LiveMembership`; this is ordering-influence authority, not permission to
disclose another result field. The estate identity permits only
`LiveMembership`, the many relation permits only `Pagination`, and every other
projected field or relation forbids observable influence.

`EmergencyEstate` is not a caller-authored convenience edge. The elevation
definition declares it as the exact governed-resource relation, Query creates it
inside the typed request program, and approval, close, review, and governed-use
currentness revalidate the same edge. Missing, duplicated, or retargeted estate
ownership denies; Bank cannot install or mutate a parallel ownership link.

Request, approval, close, and mandatory-review completion each change this
query's result and therefore emit one exact
`EstateEmergencyAccessActivityEffect { estate, access }`. Query installation
must include that typed emission in the framework-owned lifecycle program.
Query derives its payload from the already-bound transition input and retained
lifecycle lineage; Bank cannot append, omit, retarget, or otherwise author an
emission beside the typed lifecycle program. The live cause binds the effect's
estate identity to the query scope and its access identity to the exact result
child. An operation without the installed effect obligation, an extra caller-
authored effect, or a mismatched estate/access payload denies before commit.

For every lane, the same installed query, scope, parameters, result shape,
ordering, disclosure contract, capability purpose, and approved elevation must
be retained. The Bank request carries both the estate and exact emergency-access
identity used to compare the borrowed approved receipt; neither is accepted as
a second loose executor argument. Historical and preview support for the exact-one emergency
account-details query and ordinary AccountActivity continuation/live tests are
supporting lower-bound evidence only; they cannot compose to close Gate C or
R7.14 because their query identity or authority meaning differs.

The continuation cutoff must readmit the next page, revoke or expire the exact
governed support while an equivalent alternate remains active, and then obtain
typed stale-authorization denial before page execution returns any row. The
live cutoff must open under the same approved elevation, queue a real matching
lifecycle cause, remove the exact support before polling, deny before the queued
payload, transition the lease to `Closed`, and restore basis, buffer, and live-
lease resource baselines. The lawful twin aggregates every continuation page
and compares one-shot, historical, preview, and live result meaning plus exact
installed query identity and lane posture. Gate A's one-shot courtroom cannot
lend closure to those contracts.

The intended production surface remains Bank-shaped while retaining Query's
opaque move-only authorities:

```rust
let activity = bank
    .query(estate_emergency_access_activity(estate, access))
    .as_principal(&requester)
    .controls(read_controls);

let first = activity.page_with_approved_elevation(approved.approved()?)?;
let next = bank
    .query(estate_emergency_access_activity(estate, access))
    .as_principal(&requester)
    .resume_with_approved_elevation(
        approved.approved()?,
        first.continuation()?,
        resume_controls,
    )?;
let live = bank
    .query(estate_emergency_access_activity(estate, access))
    .as_principal(&requester)
    .subscribe_with_approved_elevation(approved.approved()?, live_controls)?;
```

The destination topology is responsibility-shaped:

```text
bank-domain/src/queries/estate/emergency_access_activity.rs
bank-domain/src/queries/estate/emergency_access_activity/
    selectors.rs
    shape.rs
    projection.rs
    live_cause.rs
bank-domain/src/schema/estate/effects/
    death_notification.rs
    emergency_access_activity.rs
bank-server/src/application_query/governed_execution/emergency_access_activity/
    admission.rs
    bounded.rs
    continuation.rs
    live.rs
bank-server/src/estate_capability_admission/elevated_activity_lanes/
    parity.rs
    continuation_cutoff.rs
    live_cutoff.rs
```

Gate D's estate-release consumer is an executable Query mutation, not a Bank
oracle verdict or a declaration-only capability. `ReleaseEstate` names one
exact estate plus the executor, legal-authority, and completed-release-review
witnesses selected for that attempt. Those witnesses are retained operation
input and effect-integrity evidence. They are not capability context, command
authority targets, or lifecycle authority, and their existence must not be
resolved while authorizing the estate-scoped release command.

The installed release program must:

- derive the selected witnesses from the input retained inside the admitted
  operation rather than accepting a second copied command beside admission;
- privately bind the exact estate, executor, legal-authority, and review input
  tuple into idempotency in addition to the installed release-operation
  identity, so changing any selected witness cannot recover an earlier commit
  before projection;
- require the exact estate to be `Open`, the selected principal to have the
  exact executor relation to that estate, and the selected recognized legal
  authority to have exactly that holder and estate;
- require the selected review to have kind `EstateRelease`, status `Completed`,
  exactly one relation to the target estate, and exactly one reviewer;
- retain every field and exact relation that influenced readiness through
  provider comparison, then write only `EstateCaseStatusField` from `Open` to
  `Released`;
- admit lawful co-executors and unrelated reviews without enumerating them into
  the selected release proof; and
- publish typed integrity denial for a missing, unrecognized, mismatched,
  incomplete, wrong-kind, retargeted, or malformed selected witness rather than
  collapsing those failures into command authorization or a Bank-local boolean.

The declared projection ceiling applies only to causally selected witness
truth. Unrelated executors, authorities, cases, grants, reviews, fields, and
consumers must contribute zero release decision facts and cannot exhaust that
ceiling. External proof must include authoritative exact-estate readback,
equivalent retry, intent drift, conflict and separation denial, malformed
witness denial, related and unrelated scale, provider currentness, and
exact-zero warm canonical-basis and digest work. Static installation proof must
enumerate the complete role composition, exact decision-read inventory, and
sole status-write obligation rather than asserting only their counts.

Gate D publication must preserve a closed eleven-family outcome taxonomy:
missing capability, explicit policy denial, explicit safe scope mismatch,
purpose mismatch, conflict, separation of duty, field omission, elevation
required, elevation denied, elevation expired, and review required. These are
authorization, disclosure, elevation, and review outcomes, not eleven
interchangeable denials. An installed explicit deny is not missing authority
and may not be relabeled as missing capability. Field omission is a successful
governed disclosure outcome, and review required is the successful result of a
close transition. Publication must not turn either into a failed operation.

Query execution must mint the explanation cause while it still owns the exact
installed rule and retained decision evidence. Signal must retain the exact
per-rule evaluation result it owns, Runtime Bridge must preserve that result
with its correspondence evidence, and Query must map the installed semantic
rule role and exact evaluated result into one closed non-authoritative cause.
Missing-capability, purpose, conflict, or separation meaning may not be guessed
from `PermissionDenied`, a subject string, diagnostic code, or a later graph
inspection. The public result retains the original typed Query outcome and its
exact outcome identity beside the descriptive publication.

When more than one rule rejects the same attempt, Query publishes one primary
cause without erasing the ordered exact cause set retained internally. Request-
shape causes precede graph causes because they are known without observation.
For graph decisions, absence of the exact required grant precedes prohibited
rule matches so a caller without baseline authority cannot learn protected
conflict relationships. Otherwise installed semantic order is deny, conflict,
separation of duty, distinct actor, and elevation-specific posture. Distinct-
actor remains an exact internal subkind of the public separation-of-duty
family; it may not be mislabeled as conflict.

`ScopeMismatch` is publishable only when an explicit mismatch is already
present in admitted request or grant evidence. Query and publication must not
search unrelated grants merely to distinguish missing authority from authority
held over another scope; such a search would add undeclared work and disclose
protected grant existence. Purpose mismatch follows the same retained-evidence
rule. Field-omission explanation may reveal only its already-admitted typed
omission posture, never the protected value or a wider disclosure decision.

Foundational lowering must distinguish two independent facts for a denial: the
governed operation or query did not execute, while publication of its
description did complete. A denied or blocked closeout therefore cannot report
the `Executed` posture or attest effects. Query publication must use the
truthful Foundational closeout category, diagnostic outcome and denial class,
provenance, target-aware profile, and denied receipt disposition. Exact or
narrowed profiles may materialize; widening must fail before publication.
Foundational material remains descriptive and cannot be promoted into Query
capability, admission, operation, elevation, review, disclosure, provider, or
receipt authority.

Proof requires an exhaustive Query-publication table over all eleven families,
one-axis cause and non-aliasing twins at the real decision boundary, exact
Foundational category/diagnostic/provenance/profile/receipt assertions,
string-independence, no post-denial graph work, protected-value
noninterference, and compile/runtime non-promotion. The public Bank transcript
must exercise ordinary missing-capability, conflict, and separation outcomes;
a governed query with an actually omitted field; and an emergency journey
covering an explicitly enumerated elevation-denied subkind, elevation expired,
and the successful review-required close result. Bank installs no product
explicit-deny rule, so the exact explicit-policy-denial family is proved at
Query's real installed-composition boundary rather than by inventing Bank
policy. Bank's product builders fix scope and purpose rather than exposing raw
mismatch knobs, so safe scope and purpose mismatch are proved at Query's public
generic admission boundary; Bank separately proves those dimensions cannot be
caller-substituted and a wrong-scope or wrong-purpose grant remains the
privacy-preserving missing-capability family without alternate-grant search.
Bank's approved-elevation query intentionally has no no-elevation overload, so
compile-fail consumer evidence proves elevation cannot be omitted at that
front door; the exact `ElevationRequired` runtime outcome is likewise proved at
Query's public generic admission boundary rather than by inventing a second
Bank API.

The elevation-expiry courtroom must use the same production application-runtime
publication boundary as a real host. Query exposes one non-authoritative
authorization-time source port at that installation boundary and retains the
system source as the default adapter. The chosen source is fixed for the
runtime lifetime; it is not part of capability identity, graph authority,
request input, or a lower-runtime capability, and source failure maps to the
existing fail-closed trusted-time denial. Query alone converts the external
sample into the installed timeline and decides validity. Tests may implement
that production port with a controllable source, but no `cfg(test)` source,
post-install clock replacement, caller-authored `now`, wall-clock sleep, or
synthetic elevation receipt can satisfy this proof. Bank must request and
approve a real short-lived elevation at a fixed instant, prove lawful use
before expiry, advance only the installed source to the exact expiry boundary,
and obtain and publish `ElevationExpired` through the public Bank query with no
disclosed result or governed effect.
Denied mutations must prove zero effects. Phase 8 non-authorization outcomes
and Gate D's complete scale, residue, and external certification remain
outside this focused publication slice.

The hostile consumer evidence must exercise purpose, field, missing-resource,
related-entity, amount-ceiling, and context-conflict attacks at their truthful
boundary. Static request-shape mismatches may fail during non-observing
preparation; every graph- or grant-dependent mismatch must prepare without
authority and then fail only after the real read or mutation session consumes
it. A preparation-only denial test does not prove capability enforcement.

Certification must compare the independently produced capability,
disclosure-mask, and publication boundary meaning through Foundational
canonical comparison and must admit any compact semantic digest through the
matching Foundational digest slot, while compile and runtime evidence
separately proves that no Foundational digest, profile, provenance row,
diagnostic bundle, or boundary receipt opens Query authority.

The complete estate courtroom, public consumer compilation, boundary checks,
residue searches, lifecycle probes, and warm-path measurements must all close
before Runtime Phase 8 begins. A green bank-specific oracle is insufficient if
the public facade or another installed query lane can bypass the same meaning.
The warm-path measurements must also prove exact-zero canonical-basis
preparation, digest derivation, and digest-text materialization while unrelated
grants, relationships, fields, cases, result rows, and live consumers grow.

**Proof before Runtime Hardening Phase 8**

The complete estate courtroom proves lawful ordinary, delegated, and emergency
paths while every grant-combination, conflict, field-widening, purpose-swap,
self-approval, expiry, revocation, and copied-elevation attack fails at the
earliest governed boundary. Every installed query lane applies the same
capability, purpose, disclosure, and conflict meaning. Growing unrelated
grants, relationships, fields, or cases does not widen warm authorization
work.

### Inherited Branch-Affinity Contract After Runtime Phase 7

Milestone 9.16.1 makes typed branch identity part of the canonical provider
session and branch-qualifies every snapshot and version basis. Runtime Phases
8-10, Bank World Phases 5-6, and closure inherit that identity without creating
another branch-selection surface.

From this point forward:

- decision read sets, proposed state, invariants, compare-and-commit,
  idempotency, recovery, undo/redo, continuation, history, preview, live
  delivery, disclosure, receipts, and publication carry the exact branch from
  the admitted session;
- equal version or snapshot ordinals from different branches are never
  equivalent and cannot satisfy currentness, retry, recovery, aftermath, or
  publication affinity;
- HTTP routes, Bank adapters, recovery callers, and publication code cannot
  choose, default, replace, or deserialize branch authority;
- a temporary global commit coordinator is permitted only as a conservative
  implementation limit; it is not Query meaning, receipt meaning, or the
  correctness contract, and it must be replaceable by branch-local
  coordination without changing public authority types; and
- no code introduced after Runtime Phase 7 may infer the ordinary branch from
  the string `"main"`, treat a version as globally unique, or mint a
  recovery-, aftermath-, disclosure-, transport-, or publication-local branch
  identity.

Milestone 9.16 still has one ordinary installed branch and does not implement
multiple branch heads, branch-local version allocation, concurrent writers on
different branches, branch creation, merge, rebase, or branch-local inversion.
Exact Relational/Signal component bases and Relational branch-local MVCC begin
in Milestone 9.17.1, Relational publication-service correction closes in
9.17.1.1, final owner services and Signal progress close in 9.17.1.2,
composite product-branch creation begins in 9.17.2, and the ordinary Query lane
closes in 9.17.3; tree-based semantic
reversal and reapplication begin in Milestone 9.18. Semantic merge, rebase,
multi-parent publication, offline synchronization, and distributed recovery
remain in the cross-runtime merging-and-branching roadmap. The prohibition on
branch-shaped aftermath below does not permit branch affinity to be omitted
from ordinary execution evidence.

### Runtime Hardening Track — Phase 8: Application Aftermath, External Effects, And Recovery

**Requirement**

Expose installed application aftermath, recoverable external effects, exact
retained prior truth, and closed publication without rewriting history,
importing certification replay, or pretending that a local commit proves an
external effect.

Runtime Phase 8 is implemented through the ordered internal proof gates
8.1-8.6 below. They are not parallel aftermath conveniences: each gate may
expose only next actions justified by the installed posture and authority
proved before it. A discovery that strengthens a completed gate becomes an
append-only corrective phase or milestone and blocks unfinished dependents
rather than adding an exceptional rollback path.

The refined requirement text, lower-runtime gap inventory, carrier repairs,
counter contract, and `R8.*` requirement identifiers for these gates live in
[`milestone-9.16-runtime-phase-8.md`](./milestone-9.16-runtime-phase-8.md).
That document specifies; it does not relax or renumber the gates below.

**Current scope amendment:** the
[Runtime Phase 8 finish plan](./milestone-9.16-runtime-phase-8-finish-plan.md)
governs cleanup. The undo- and redo-specific bullets below record the historical
design and are provisional successor requirements, not current Phase 8 closure
requirements. Existing code may remain, but only accepted aftermath, external
effect, recovery, retention, authority, and publication foundations close here.

**Must establish**

- installed reversible, compensatable, reconcilable, and irreversible
  aftermath contracts with operation-specific next-action types;
- a framework-owned recovery handle for indeterminate outcomes, bound to exact
  runtime, branch, operation, attempt, principal scope, idempotency identity,
  provider posture, and expiry or disposal lifecycle;
- typed inspect, resolve, safe-retry, compensate, reconcile, and dispose
  transitions, exposing only those admitted by the outcome and installed
  contract;
- undo as a fresh admitted inverse or compensation operation that consumes the
  exact committed receipt and re-enters current capability, policy,
  touched-graph, invariant, idempotency, and compare-and-commit progression;
- compensating journal entries for bank money movement, explicit inverse
  operations for eligible capability changes, and honest denial for
  irreversible legal, audit, approval, or escaped-effect cases;
- redo as a descriptive intent available only after a proved undo, requiring
  fresh authority and current-truth validation and never importing replay;
- Query-owned typed `undo-of` / `redo-of` semantics co-committed through the
  ordinary Relational transaction, with Relational remaining the sole owner of
  commit identity, parents, branch head, ancestry, and publication;
- explicit redo invalidation when Relational's authoritative current head
  diverges from the head bound by the redo intent, enforced again as an atomic
  compare-and-commit precondition rather than by a Query-owned chain; and
- provider commit, emitted application causality, dispatch, external
  acknowledgement, external completion, compensation, and reconciliation as
  distinct typed postures; and
- one Foundational canonical basis for portable aftermath meaning; explicit
  lowering of completed Query outcomes into Foundational boundary categories,
  executed/completed receipt posture, provenance, linear-lineage,
  degraded-recovery, diagnostic, and performance vocabulary without making
  any lowered artifact executable authority.

**Destination module skeleton**

```text
worth-query-installation/src/application_aftermath/
    canonical_basis.rs
    posture.rs
    next_action_contract.rs
    inverse_contract.rs
    compensation_contract.rs
    recovery_contract.rs
    external_effect_contract.rs

worth-query-execution/src/domain_computation/application_aftermath/
    recovery_handle.rs
    recovery_progression.rs
    undo_admission.rs
    undo_progression.rs
    redo_intent.rs
    redo_admission.rs
    causality/
        undo.rs
        redo.rs
        committed.rs
        current_head.rs
    external_effect.rs

worth-query-execution/src/domain_computation/primary_graph/provider/
    application_causality/
        prepare.rs
        commit_fact.rs
        lookup.rs

worth-query-publication/src/application_aftermath/
    outcome.rs
    explanation.rs
    access_and_disclosure.rs
    boundary_evidence.rs
```

The installation package owns operation-specific aftermath meaning and legal
next actions. Execution owns attempt-bound progression and consumes current
runtime authority. Publication describes the resulting posture and available
next actions but cannot manufacture a recovery, undo, reconciliation, or redo
handle from identities carried over the wire.

Relational remains the sole owner of commit identity, ordered parents, branch
head, ancestry, serialization, and canonical publication. Query owns only the
operation-semantic statement that an admitted correction is `undo-of` or
`redo-of` an exact committed target; it prepares that fact and co-commits it
with the ordinary mutation. Query owns no parallel commit chain or mutable
history head. Runtime Bridge owns installed inverse correspondence and may
transport completed admitted causality for a real cross-runtime consumer, but
it cannot decide undo/redo legality, Relational currentness, or history
publication.

Branch-shaped aftermath is intentionally absent from this topology. Its
semantic-history, reference, inversion, merge, publication, recovery, and
product-surface responsibilities belong to the cross-runtime
merging-and-branching roadmap and do not enter through a dormant Query
directory. Ordinary aftermath nevertheless retains the exact branch affinity
of its Milestone 9.16.1 session; omitting that affinity or treating a version
ordinal as global is forbidden. Foundational branch, merge, scoped-merge, and
cherry-pick artifacts are likewise forbidden as implementation authority for
this linear Query aftermath. Only completed Query-owned transitions may lower
into the portable Foundational transition or lineage vocabulary appropriate to
their boundary role.

#### Runtime Phase 8.1: Installed Aftermath Classification And Legal Next Actions

Runtime Phase 8.1 installs one canonical aftermath contract for every mutation
before execution can advertise recovery or reversal behavior.

It must:

- classify each installed operation as reversible, compensatable,
  reconcilable, or irreversible, with operation-specific typed next actions;
- distinguish a semantic inverse, compensating operation, reconciliation
  procedure, and terminal denial instead of representing them as one generic
  rollback callback;
- prepare the portable classification, next-action, inverse, compensation,
  recovery, and external-effect contract through one Foundational canonical
  basis and structured comparison lane before admitting compact digest
  derivation through Foundational's typed slot and retaining that digest inside
  Query's installed aftermath identity;
- bound the installed aftermath contract's canonical entry count and encoded
  bytes, derive each newly installed aftermath identity once, and carry the
  resulting typed identity through recovery, undo, redo, and publication
  without rederiving predecessor meaning;
- bind classification and next-action meaning to the exact operation, schema,
  package, compatibility generation, commit posture, and result contract; and
- reject missing, contradictory, host-authored, or changed aftermath meaning at
  installation rather than defaulting to irreversible or locally reversible.

Proof requires a complete operation inventory, Foundational structured
comparison plus one-axis contract drift attacks, residue denial for direct
hash/debug-string identity grammars or raw digest derivation, and public type
evidence that an outcome exposes only the next actions installed for its exact
posture. The Bank World aftermath declarations must install through the generic
contract instead of remaining descriptive enums.

#### Runtime Phase 8.2: External-Effect Causality And Indeterminate Posture

Runtime Phase 8.2 separates facts the local runtime can prove from facts owned
by an external system. Local commit, application-causality emission, dispatch,
external acknowledgement, external completion, compensation, and
reconciliation are distinct typed postures.

It must:

- assign stable exact identities and causal links to the provider commit,
  emitted application event, dispatch attempt, external acknowledgement, and
  external completion;
- classify timeout, disconnect, lost response, duplicated acknowledgement, and
  unknown provider outcome without guessing whether the effect occurred;
- preserve idempotency and provider correlation evidence required for later
  inspection or safe retry; and
- prevent local success, transport success, or receipt possession from
  satisfying external-completion authority.

A genuinely new dispatch or external-causality event may receive one newly
admitted identity. Delivery, acknowledgement, timeout classification,
inspection, retry resolution, and completion carry that identity and may not
rehash the same semantic event at each transition.

Query owns these exact effect postures and correlation authority. Publication
may lower them into Foundational provenance, execution/completion receipt, and
support-grade freshness vocabulary only after the Query boundary is known;
Foundational completion wording cannot upgrade an indeterminate external
effect.

Proof requires a controllable real external-effect boundary that can commit
then lose the response, acknowledge without completing, complete after timeout,
and duplicate a message. An in-process fake that shares the runtime's truth
source cannot prove this gate.

#### Runtime Phase 8.3: Recovery Handle And Resolution Lifecycle

Runtime Phase 8.3 mints a framework-owned linear recovery handle only for an
installed outcome whose exact posture permits recovery work.

It must:

- bind the handle to runtime, schema, typed branch, operation, attempt,
  principal scope, idempotency identity, provider posture, correlation
  evidence, compatibility generation, and expiry or disposal lifecycle;
- expose only typed inspect, resolve, safe-retry, compensate, reconcile, or
  dispose transitions admitted by the current outcome and installed contract;
- re-establish current provider truth and current application authority before
  a transition produces effect authority; and
- consume, expire, or dispose the handle linearly so copying an identity,
  repeating a transition, or crossing runtimes opens no door.

Inspection and resolution use the retained typed recovery and correlation
identities. Handle lookup, provider inquiry, and repeated inspection perform no
canonical-basis preparation, digest derivation, or digest-text comparison.

An unresolved or degraded recovery publication uses Foundational support-truth
and basis-disclosure vocabulary while retaining the Query recovery handle as
the sole next-action authority. A support artifact or opaque wire identity
cannot be readmitted as a handle.

Proof requires lost-response recovery, already-completed recovery, unresolved
external posture, expiry, disposal, copied-handle, foreign-principal,
foreign-runtime, and duplicate-transition attacks. The public wire boundary may
carry opaque recovery identity and descriptive posture but never the runtime
authority object.

#### Runtime Phase 8.4: Fresh Undo, Inverse Operations, And Compensation (provisional history)

The requirements below describe the current experiment. They remain regression
evidence only and do not establish a supported undo product.

Runtime Phase 8.4 implements undo as a new admitted operation derived from an
exact committed receipt, never as history mutation or direct provider repair.

It must:

- consume the committed receipt and installed aftermath contract to derive the
  exact inverse, compensation, or reconciliation request;
- re-enter current capability, purpose, disclosure, conflict, touched-graph,
  invariant, idempotency, provider, and compare-and-commit progression;
- produce compensating debit and credit journal entries for money movement and
  explicit inverse operations for eligible capability changes while preserving
  the original operation and causality; and
- deny irreversible legal, audit, approval, released-estate, escaped-effect,
  stale, conflicted, or already-consumed outcomes without a fallback mutation.

Undo is a fresh admission and may derive one new bounded intent identity for
the inverse or compensation request. It carries the original committed and
aftermath identities and cannot regenerate them per posting, decision fact, or
co-committed causal fact.

The original, inverse or compensation request, and resulting Query receipts
remain the authority chain. Foundational transition/provenance rows may
describe the completed relationship afterward, but a Foundational committed
artifact, transition bundle, or no-op cause cannot substitute for fresh Query
admission.

Proof requires one and only one compensating transfer with both original
journals preserved, current-policy denial after drift, idempotent retry after a
lost response, inverse capability progression, and rejection of copied,
foreign, irreversible, or twice-consumed receipts.

#### Runtime Phase 8.5: Fresh Redo Intent And Relational-Head-Bound Causality (provisional history)

The requirements below describe the current experiment. They remain regression
evidence only and do not establish a supported redo product.

Runtime Phase 8.5 derives descriptive redo intent only from a proved undo and
runs it as a fresh operation against current truth.

It must:

- bind redo intent to the original operation meaning, proved undo receipt, an
  owner-observed projection of the exact Relational branch head, principal
  scope, and compatibility generation without embedding runtime authority or
  replay state;
- require fresh capability, policy, conflict, touched-graph, invariant,
  idempotency, provider, and compare-and-commit admission;
- represent the original only by its Relational-backed Query commit receipt and
  co-commit exactly one private typed Query causal fact for each undo or redo;
- take the child commit identity, ordered parents, branch, and publication order
  only from the Relational commit result; and
- invalidate redo when a divergent operation advances Relational's current
  head, with the expected head consumed atomically by compare-and-commit and no
  Query-owned chain, mutable head, branch object, merge placeholder, or hidden
  alternate lineage.

Redo is likewise one fresh bounded admission identity. Current-head checks,
co-committed causal-fact preparation, provider comparison, and publication
carry the original, undo, and redo identities without rehashing them per edge
or transition.

Completed aftermath causality lowers into Foundational attested-continuity or
completed-receipt vocabulary only after the Relational commit succeeds. Runtime
Bridge may transport that owner-admitted projection only when causality crosses
runtimes; Bridge admission cannot upgrade it into Query legality or Relational
currentness. Replayed, reconstructed, restored, branch-local, partial, or
promoted lineage postures cannot be relabeled as ordinary linear aftermath.

Proof requires lawful redo, stale or newly unauthorized redo, copied intent,
foreign principal, changed operation meaning, duplicate redo, and divergence
attacks. A hostile schedule must allow two operations to observe the same
Relational head, commit one intervening operation, and prove the stale redo
cannot commit even if its Query precheck already passed. Residue checks reject
a Query-owned lineage chain/head, raw append APIs, and ordinary Phase 8 Bridge
legality or history authority. Certification replay may verify evidence but
must not be imported into the ordinary redo path.

#### Runtime Phase 8.6: Bank Aftermath Cutover, Publication, And Certification

Runtime Phase 8.6 moves the bank's real transfer and estate aftermath through
the public installed progression after Phases 8.1-8.5 are proved.

The accepted cutover is committed aftermath, external-effect, recovery, exact
retention, and closed publication. References below to undo, redo, or
receipt-linked correction causality describe provisional regression coverage,
not a supported product facade.

It must:

- expose committed outcome, recovery, compensation, reconciliation, undo, and
  redo through typed public facades and operation-specific legal next actions;
- preserve authorization and disclosure when publishing outcome,
  explanations, recovery posture, receipt-linked lineage, and exact inherited
  branch affinity;
- keep the temporary HTTP boundary descriptive, asynchronous, and incapable of
  deserializing authority or making route-local recovery decisions;
- remove or privatize superseded monolith, bank-local, and generic rollback
  paths, then prove destination dependency direction; and
- preserve bounded ordinary commit cost when no external or recovery work is
  required, while measuring reconstructive inspection and compensation
  separately.

Query-owned counters prove execution. Publication and certification lower
ordinary, recovery, inspection, compensation, and reconciliation costs into
Foundational performance claims and counter-backed receipts with explicit
temperature, included work, excluded work, freshness, fallback debt, and
report-materialization boundaries. That lowering begins from the stronger
Query receipt and its exact counters; it cannot remeasure execution, derive a
parallel semantic digest, or make report materialization part of the ordinary
commit path. A descriptive claim or policy-admission receipt cannot satisfy
executed-cost evidence.

Certification separately reports canonical-basis preparations, digest
derivations, canonical bytes encoded, and textual digest materializations for
installation, fresh admission, ordinary commit, external dispatch, recovery
inspection, undo, redo, and publication. Ordinary commit and all fan-out work
must remain at exact zero; a fresh undo or redo may pay only its one bounded
admission derivation, independent of posting count, decision-fact count, and
lineage length.

The bank transfer and estate aftermath courtroom, real external-boundary fault
matrix, public consumer compilation, boundary checks, residue searches,
lifecycle probes, and ordinary-versus-reconstructive measurements must all
close before Bank World Phase 5 begins.

**Proof before Bank World Phase 5**

A committed transfer produces one compensating reversal and preserves both
journals; an equivalent retry does not compensate twice. Redo after the proved
undo is freshly authorized and can stale or deny after relevant drift. A copied
receipt, foreign principal, expired capability, conflicted beneficiary,
irreversible operation, lost response, or unresolved external effect cannot
manufacture undo, redo, rollback, or completion authority. Independently
produced aftermath contracts compare through Foundational canonical basis, and
no Foundational digest, committed artifact, transition bundle, support report,
lineage artifact, or boundary receipt can be promoted into a Query recovery,
undo, or redo input.

### Bank World Track — Phase 5: Temporary HTTP Boundary And Per-User Async Nodes

**Requirement**

Run the ordinary public Query surface across real asynchronous process and
network boundaries.

**Must establish**

- one authoritative bank-server process;
- one independently authenticated user-node process per fixture participant;
- an Axum adapter that maps HTTP and SSE onto the public Query facade;
- typed wire representations for query identity, basis, opaque continuation,
  capability purpose, disclosure omissions, elevation progression, and
  recovery without serializing runtime authority;
- bounded request and stream queues, cancellation, deadlines, backpressure, and
  disconnect handling;
- dynamic ports, health/readiness, deterministic teardown, and leak detection;
- typed wire representations for semantic outcomes and legal next actions;
- no route-local banking policy or direct provider access; and
- no route-local branch selection, defaulting, or reconstruction from wire
  strings or version ordinals.

**Proof before Runtime Hardening Phase 9**

The full courtroom runs over TCP with separate process IDs and runtimes.
Disconnects, restarts of non-authoritative user nodes, response loss, queue
saturation, token expiry, and live revocation preserve semantic outcomes.

### Runtime Hardening Track — Phase 9: Host-Installed Conditional Operations, Managed Time, And Reconstructible Wakes

**Requirement**

Complete the primary-graph application runtime's conditional-operation front
door. An ordinary host must be able to install application predicate providers
and named clock sources, submit clock observations, and let Query invoke the
same installed application operation when Signal admits a wake, without
importing Runtime Bridge or Signal or owning a scheduler.

**Discovery classification**

This is a generic Query API, authority, lifecycle, and reconstruction gap under
the milestone's phase-amendment rule. The portable contract is already visible
through `worth-query-host`, but the only implemented provider installation path
is the legacy
[`WorthQueryRuntimeBuilder::conditional_node(...)`](../../workspaces/worth-query/crates/worth-query/src/runtime/builder/conditional_execution.rs),
which accepts a raw Signal graph and `BridgeConditionalProviderSet`. At the same
time, Query's
[`installed-operation` consumer-residue registry](../../workspaces/worth-query/crates/worth-query/src/consumer_kit/consumer_residue/registry/installed_operation_rows.rs)
forbids direct `worth_signal` and `worth_runtime_bridge` use. The
primary-graph application runtime creates and retains its managed Runtime Bridge
but exposes no corresponding host installation port. A consumer therefore
cannot satisfy both the feature contract and the dependency law. Phase 9 closes
that contradiction.

**Consumes**

- Milestone 9.14 portable conditional declarations, semantic dependencies,
  pair-bound Bridge lowering, and Signal decision provenance;
- Milestone 9.16 Runtime Phase 6 installed application-operation identity and
  the primary-graph application runtime;
- Runtime Phase 7 authorization, disclosure, purpose, governed actor, and
  trusted-time rules;
- Runtime Phase 8 idempotency, aftermath, recovery, and retained authoritative
  application truth; and
- Milestone 9.16.1 exact branch, graph, provider-session, installation, and
  generation affinity.

**Public host contract**

`worth-query-host` must expose one typed conditional-operation installation
surface under its existing runtime facade. The surface must let the host:

1. resolve an installed application operation and one of its installed
   conditional nodes without a string lookup, raw node ID, graph handle, or
   lower-runtime identity;
2. bind a typed domain condition provider whose input is Query's immutable,
   dependency-indexed observation view and whose output is only satisfied,
   unsatisfied, or a typed provider failure;
3. bind a named host clock source to the exact temporal node and obtain a
   runtime-bound observation port after successful publication;
4. submit typed clock observations carrying source, timeline, sequence, and
   observed-time evidence, while receiving typed accepted, duplicate, stale,
   reordered, foreign, closed, or failed posture;
5. declare the exact bounded Relational projection from which active temporal
   intents, due basis, operation input, stable intent identity, and idempotency
   relation are reconstructed; and
6. inspect non-authoritative lifecycle and work evidence without receiving a
   provider session, Signal wake, scheduling capability, or executable
   operation authority.

The intended host journey is one installation progression, not a collection of
independently valid ingredients:

```text
installed application schema
    -> installed application operation
    -> installed conditional node
    -> admitted host predicate provider
    -> admitted named clock and temporal-intent reconstruction contract
    -> published primary-graph application runtime
    -> runtime-bound clock-observation port
```

Publication fails atomically if a declared conditional node has no exact
provider, a provider has no declaration, a temporal node lacks its clock or
reconstruction contract, or any operation/node/provider/clock/projection axis
is foreign, stale, ambiguous, unsupported, or duplicated.

**Destination topology**

Implementation names may become more specific, but responsibilities must land
in this semantic shape rather than in a facade or helper bag:

```text
worth-query-installation/src/
    domain_operation/conditional_node/
        host_provider_contract.rs
        named_clock_contract.rs
        temporal_intent_contract.rs

worth-query-execution/src/domain_computation/primary_graph/
    conditional_operation/
        installation.rs
        predicate_observation.rs
        predicate_admission.rs
        clock_observation.rs
        temporal_intent_projection.rs
        wake_reconstruction.rs
        signal_decision_reentry.rs
        application_operation_invocation.rs
        lifecycle.rs
        work_evidence.rs

worth-query-host/src/facade.rs
    re-export the narrow installation, provider, observation, clock, denial,
    and inspection contracts from their semantic owners

worth-query-certification/
    host-only conditional consumer and reinstall courtroom
```

Runtime Bridge's existing conditional-execution owner remains the only
Relational-to-Signal correspondence and lowering lane. Signal's existing
temporal owner remains the only wake scheduler and eligibility/suppression
authority. If those owners require a narrower upstream port, that port belongs
to their existing conditional or temporal modules and must not expose raw
Signal decisions through `worth-query-host`.

**Must establish**

- one application-runtime-owned conditional registry keyed by exact installed
  operation, node, installation generation, graph, provider, branch, and clock
  affinity;
- Query-owned observation types that preserve declared dependency ordinal,
  exact previous/current Foundational contract values, truth basis, and
  absence posture without exposing Bridge resolver context or Signal masks;
- a host predicate adapter that translates satisfied/unsatisfied into the
  installed Bridge provider contract internally, with Query performing no
  second predicate evaluation and no eligibility restamping;
- one named clock source per declared clock binding, immutable for the
  application-runtime lifetime, with duplicate idempotence, monotonic sequence
  and timeline enforcement, source isolation, bounded admission, and explicit
  close/failure posture;
- typed separation between temporal clock observation, authorization time,
  request deadline time, and provider commit time so no source can be
  substituted for another;
- clock observations delivered to the Bridge-owned Signal runtime so Signal
  alone promotes due wakes, coalesces or suppresses work, and emits exact
  decision provenance;
- an installed, branch-affine Relational access plan for active temporal
  intents; ordinary clock delivery must use the derived due index, while
  runtime reinstallation may execute the separately budgeted reconstruction
  plan;
- readiness that remains closed until provider binding, clock binding, current
  intent reconstruction, Signal wake installation, and lifecycle publication
  all succeed atomically;
- fresh Query application-operation admission for each eligible wake, using
  the operation's normal governed principal/system-actor, capability, purpose,
  touched-graph, invariant, idempotency, compare-and-commit, aftermath, and
  publication path;
- atomic consumption or advancement of the durable temporal intent in the
  same application operation that commits its effects, so duplicate wakes and
  commit-before-observation faults resolve idempotently;
- typed provenance joining stable temporal intent, reconstructed wake, Signal
  decision, application-operation attempt, and terminal outcome without making
  any descriptive row reusable authority;
- successor-installation behavior that either proves exact compatibility and
  reconstructs current work or fails closed with a typed rebind requirement;
- bounded queues, cancellation, deadlines, shutdown, provider failure,
  predicate panic isolation, and lease cleanup; and
- public documentation and migration guidance that replace the legacy raw
  builder example for primary-graph application hosts.

**Mechanically forbid**

- `worth_signal`, `worth_runtime_bridge`, `SignalGraph`,
  `BridgeConditionalProviderSet`, or lower-runtime condition-decision types in
  host-consumer manifests or source;
- application-local temporal scheduler, timer wheel, wake registry, scheduler
  task, or direct operation callback;
- a provider returning raw Signal eligibility, suppression, provenance, node,
  aspect, or wake values;
- caller-authored `now`, post-publication clock replacement, wall-clock sleep
  as temporal proof, and cross-source clock substitution;
- reconstruction from serialized wake handles, process memory, logs, or
  copied receipts instead of current authoritative domain truth;
- invocation that bypasses ordinary application-operation admission or treats
  wake eligibility as authorization;
- full-domain, full-operation, or full-node scans on ordinary clock
  observation; and
- parallel primary-graph conditional installation through the legacy
  `WorthQueryRuntimeBuilder::conditional_node(...)` path.

The legacy builder may remain only for its separately governed pre-primary-
graph runtime while live consumers still require it. It is not the host
contract, must not be re-exported by `worth-query-host`, and cannot satisfy this
phase's acceptance evidence.

**Proof before Runtime Hardening Phase 10**

The conditional-operation courtroom closes every crash/reinstall boundary
listed above with independently asserted effects and non-effects. Host-only
compile tests install the provider, clock, reconstruction contract, and
application operation using `worth-query-host`; compile-fail and residue tests
prove that raw Signal/Bridge types, local scheduling, raw decision return, and
direct callback invocation are unavailable.

The proof compares the host path with the existing internal conditional oracle
for satisfied, unsatisfied, failed, future, due, cancelled, superseded,
completed, duplicate-clock, reordered-clock, provider-replaced, and
generation-changed cases. Signal provenance and Query terminal outcomes must
agree without Query or the host reproducing the lower-runtime decision.
Reinstallation from retained authoritative Relational truth restores exactly
the active wakes, restores none for cancelled or completed intents, invokes
the exact installed operation, and cannot duplicate a committed effect.

Work evidence holds installed operation/node/clock counts constant while
growing unrelated rows and proves that ordinary observation cost follows only
the admitted observation plus due wake fan-out. Reconstruction cost is reported
separately and follows only the installed temporal-intent projection. Lifecycle
tests close, replace, cancel, and drop runtimes with exact-zero leaked provider,
clock, wake, task, queue, operation-attempt, and lease resources.

### Runtime Hardening Track — Phase 10: Public Policy Cutover And Workaround Deletion

**Requirement**

Make the proven front door canonical and delete the local reconstruction paths
that the bank world or existing consumers no longer need.

**Must establish**

- contracted declaration and host facade snapshots;
- public API documentation for typed schema use, authentication adaptation,
  installed application queries, capability and disclosure composition,
  break-glass progression, mutation outcomes, recovery, accepted aftermath and
  publication, host-installed conditional providers, named clocks, temporal
  reconstruction, and the provisional status of undo/redo, history, preview
  posture, and live delivery;
- `AI_README.md` orientation links that lead agents from the runtime model to
  the relevant feature documents;
- migration of relevant Worth UI or other reference-consumer workarounds where
  the new surface owns the capability;
- deletion of raw aspect strings, manual permission registries, local Query
  authority builders, application-local generic cursors, lane-specific query
  copies, undo stacks, break-glass booleans, post-projection redaction, and
  duplicate outcome assembly, local temporal scheduler, and raw conditional
  provider assembly; and
- residue checks that prevent their return.

**Proof before Bank World Phase 6**

A fresh consumer can discover and implement the supported bank paths from the
public facade and docs without architectural archaeology.

### Bank World Track — Phase 6: Complete Consumer Journeys

**Requirement**

Assemble the runtime phases and bank mechanisms into the complete public bank
application without internal imports, local authority, or fixture shortcuts.

**Must establish**

- authenticated account discovery and scoped account detail;
- personal transfer, deposit, and withdrawal;
- business payment initiation and distinct-user approval;
- employee teller and auditor journeys with non-equivalent authority;
- estate-case discovery, executor and beneficiary views, restricted-field
  disclosure, account freeze/release, and conflict-free disbursement;
- branch-manager conflict-of-interest, narrowed delegation, governed
  break-glass approval, expiry, revocation, and mandatory review;
- concurrent transfer, idempotent retry, response-loss, and stale outcomes;
- authorized activity history and live delivery;
- permission grant, revocation, and live-stream narrowing; and
- one installed query exercised as one-shot, paged continuation, historical,
  live, and admitted preview work wherever its support posture permits;
- typed explanations and actionable recovery for denial, conflict,
  cancellation, disclosure omission, elevation, and indeterminacy.

Linear undo/redo journeys are explicitly excluded from Bank Phase 6 and
Milestone 9.16 closure. Existing experiments may remain as non-authoritative
regression coverage, but Milestone 9.18 owns their product semantics, public
contract, and acceptance evidence.

**Proof before Closure Track Phase 1**

Every basic front-door question is answered through public Query and adapter
surfaces, every process boundary is real, and the gap ledger contains no
unclassified workaround.

### Closure Track — Phase 1: Hostile Certification And Permanent Prohibitions

**Requirement**

Close the milestone through consumer-real, adversarial evidence and permanent
enforcement.

**Must establish**

- the complete requirement/evidence closure ledger;
- cross-process public-consumer tests;
- compile-fail authority and phase-order probes;
- hostile OIDC, policy, touched-graph, invariant, concurrency, idempotency,
  cursor, basis, disclosure, delegation, conflict-of-interest, break-glass,
  aftermath, live-revocation, and transport scenarios;
- certification-only replay parity for authorized bank results;
- warm-path work counters and targeted compile/test timing;
- boundary, context, facade, dependency, residue, and file-size enforcement;
  and
- explicit ownership for every discovered but deferred capability.

**Closure rule**

A green suite does not close an unproved ledger row. Every high- or
critical-impact finding blocks the unfinished guarantees it can invalidate and
is assigned to an append-only corrective phase when its source guarantee has
already closed.

## DX Target

The application-facing target should be recognizable to a normal framework
consumer while preserving typed authority:

```rust
bank.operation(operations::send_money)
    .as_principal(request.authenticated_principal())
    .with_input(SendMoney {
        from: accounts::AccountRef::from_path(request.path_account()),
        to: customers::PaymentHandle::parse(request.body.recipient)?,
        amount: Money::<USD>::from_minor(request.body.cents)?,
    })
    .idempotency(request.idempotency_key())
    .deadline(request.deadline())
    .execute()
    .await?
```

Business approval should expose the legal next action from a typed result:

```rust
match payment.initiate().await? {
    InitiatedPayment::Committed(receipt) => publish(receipt),
    InitiatedPayment::ApprovalRequired(pending) => {
        pending.approve_as(request.authenticated_principal()).await?
    }
    InitiatedPayment::Denied(reason) => deny(reason),
}
```

Authorized live delivery should remain query-shaped:

```rust
let activity = bank
    .query(queries::account_activity(account))
    .as_principal(request.authenticated_principal())
    .subscribe(LiveDelivery::server_sent_events())
    .await?;
```

One installed query should retain its meaning across supported bases and
lifecycles:

```rust
let query = queries::estate_activity(estate_case)
    .ordered_by(activity::committed_sequence())
    .page_size(50)?;

let current = bank.query(query.clone())
    .as_principal(principal)
    .purpose(purposes::estate_administration())
    .at(ReadBasis::Current)
    .run()
    .await?;

let historical = bank.query(query.clone())
    .as_principal(principal)
    .purpose(purposes::estate_administration())
    .at(ReadBasis::Historical(current.version()))
    .run()
    .await?;

let live = bank.query(query)
    .as_principal(principal)
    .purpose(purposes::estate_administration())
    .live(LiveControls::bounded(64)?)
    .open()
    .await?;
```

Emergency elevation should remain narrower than ordinary institutional power
and should expose mandatory review as a legal next action:

```rust
let requested = bank.break_glass(emergency::preserve_estate_account(account))
    .as_principal(branch_manager)
    .for_purpose(purposes::suspected_asset_dissipation())
    .fields(disclosure::restricted_account_preservation())
    .expires_within(Duration::minutes(20))
    .request()
    .await?;

let active = requested
    .approve_as(non_conflicted_compliance_officer)
    .await?;

let inspection = active.query(queries::estate_preservation_view(account)).await?;
let review = active.close()?.mandatory_review();
```

The following undo/redo sketch is retained only as historical input to
Milestone 9.18. It is not a Milestone 9.16 DX target or acceptance surface:

```rust
let committed = bank.mutate(commands::send_money(input))
    .as_principal(principal)
    .controls(controls)
    .run()
    .await?
    .require_committed()?;

let undone = committed
    .aftermath()
    .compensate_as(principal, undo_controls)
    .await?;

let redo = undone.redo_intent()?;
let redone = bank.redo(redo)
    .as_principal(principal)
    .controls(redo_controls)
    .run()
    .await?;
```

These blocks are design targets. The plan for each phase must reconcile them
against the real public APIs and make any necessary semantic differences
explicit before implementation.

## Basic Front-Door Questions

The completed application must answer, through public typed APIs:

- Who am I in this bank?
- Which personal and business accounts may I see?
- Which estate cases, accounts, documents, and fields may I see for this exact
  purpose?
- What is the exact current and available balance of an account?
- Which capability grants, denials, conflicts, delegations, employee
  assignments, and case relationships affect this operation?
- May I initiate, approve, deposit, withdraw, or transfer from this account?
- May I freeze, release, inspect, or disburse this estate account, and which
  distinct actor must act next?
- Why is a field omitted even though the containing account is visible?
- May a protected fact influence an authorized filter, ordering, count, or
  policy decision without becoming visible, and what prevents it from leaking
  through membership, rank, cursor, summary, or timing posture?
- Can I delegate this capability, how narrowly, for how long, and with what
  downstream delegation posture?
- Does my personal interest conflict with my employee authority?
- What exact emergency access may I request, who must approve it, when does it
  expire, and what review remains mandatory?
- Why was an operation denied?
- What payments are pending my approval?
- Did my request commit, fail, become stale, or remain indeterminate?
- If I retry after losing the response, will money move twice?
- What account activity may I inspect?
- Can I ask the same installed query at current, historical, live, or preview
  basis without its meaning or disclosure changing?
- Is this cursor valid for this exact query, ordering, basis, and scope?
- Which consistency, freshness, result, work, deadline, and cancellation
  controls belong to me?
- Can I subscribe to the same authorized result and receive query-shaped
  changes?
- Did this filtered, sorted, or relationship-expanded result consume one
  admitted graph-read access plan without per-result child reads or hidden
  fallback?
- What changes when my role is granted or revoked?
- Can two users race without overspending or observing impossible balances?
- Can an auditor inspect allowed evidence without gaining mutation power?
- Can an employee who is also a customer keep those authorities distinct?
- If the employee is also a beneficiary, can conflict-of-interest prevent
  self-benefiting access even though each relationship is individually valid?
- Can my host bind this domain predicate to the exact installed operation and
  conditional node without importing Signal or Runtime Bridge?
- Can my host submit an observation from this named temporal clock while
  Signal, not my application, owns wake eligibility and suppression?
- If the application runtime is reinstalled, which active wakes reconstruct
  from current domain truth, and why can none duplicate a committed operation?
- If the outcome is indeterminate, what typed recovery action remains legal?
- Which operations are reversible, compensatable, reconcilable, or
  irreversible?

If one of these requires an internal import, local registry, raw string, hidden
callback, or result reinterpretation, the front door is not finished.

## Must Preserve

- Milestone 9.15 prepared-state and invariant authority;
- one installed operating-world root;
- Query-agnostic schema and domain meaning;
- Relational ownership of authoritative graph facts and transaction mechanics;
- runtime-bridge ownership of exact installed correspondence;
- Signal ownership of policy-node evaluation truth, temporal wake scheduling,
  eligibility, and suppression provenance;
- Query ownership of exact installed conditional-provider binding and
  application-operation re-entry through the host facade;
- authoritative Relational/domain temporal intent distinct from volatile,
  reconstructible Signal wake state;
- authentication distinct from authorization;
- roles and relationships distinct from scoped capability authority;
- ordinary capability distinct from governed emergency elevation;
- entity visibility distinct from field disclosure;
- one canonical application-query identity across supported execution lanes;
- exact Foundational value, aspect-contract, mask, canonical-basis,
  boundary-evidence, diagnostic, profile, lineage, and performance meaning at
  the cross-crate boundaries where those vocabularies apply;
- the strongest-owner rule: Foundational meaning can constrain or describe
  Query work but cannot mint Query installation, execution, authorization,
  disclosure, recovery, undo, redo, or receipt authority;
- cert-only replay imports;
- typed branch affinity carried from admission through terminal publication,
  with snapshot and version identity interpreted only within that branch;
- typed outcomes over exceptions or generic error strings; and
- bounded ordinary warm paths with cold certification isolated.

## Explicit Non-Goals

- loans, credit scoring, interest, exchange rates, card networks, chargebacks,
  fraud modeling, or regulatory reporting;
- production identity administration or a custom identity provider;
- replacing the final WORTH Server architecture;
- browser UI polish;
- multi-currency conversion;
- distributed consensus or multi-bank settlement;
- accepting the current linear undo/redo experiments as product semantics,
  public contract, or closure evidence before Milestone 9.18;
- branch- or tree-shaped undo/redo navigation and branch-local inversion before
  Milestone 9.18, or semantic merge, rebase, and conflict resolution before
  their cross-runtime milestones;
- multiple branch heads, branch-local version allocation, and concurrent
  writers on different branches before the Milestone 9.17.1 handoff;
- durable recovery handles, restart-stable cursors, or restart-stable
  undo/redo history before the Store handoff;
- persistence or replay of Signal wake handles, a Query-owned durable timer
  journal, or provider-process recovery before the Store handoff; Phase 9
  reconstructs derived wakes only when the authoritative Relational/domain
  truth source survives application-runtime reinstallation;
- advanced domain access products, correlated paths, conflict partitions,
  decision evidence, reuse, or provider certification governed by Milestones
  9.19 through 9.22.

The absence of those capabilities cannot justify fake authentication, fake
money, fake concurrency, or fake authorization in the supported world.

## Acceptance Evidence

Milestone 9.16 closes only when:

- the bank courtroom runs against a real Authentik issuer, one bank server, and
  separate user-node processes over TCP;
- all actors and relationships are provisioned dynamically;
- public consumer code contains no semantic aspect strings or internal Query
  imports;
- Query/application/capability/aftermath identities are reproducible through
  ready Foundational canonical bases and structured comparison, while residue
  checks reject direct SHA/debug-string identity grammars and warm-path
  canonicalization;
- phase-separated structural counters prove bounded canonical entry and byte
  work at installation or fresh admission, exact-zero basis preparation,
  digest derivation, and digest-text materialization in execution, provider
  commit, projection, authorization fan-out, live delivery, and recovery
  inspection, and no rederivation during retry resolution;
- independent fan-out twins grow roots, nodes, edges, candidates, result rows,
  projected fields, policy facts, postings, lineage edges, and consumers while
  holding installed and freshly admitted identities constant; canonical and
  digest work must not grow, and SHA-256 compression measurements must remain
  distinct from canonical encoding and allocation;
- the Relational -> runtime-bridge -> Signal -> Query authorization chain is
  exercised and independently challenged;
- a host-only consumer installs an exact conditional provider and named clock,
  submits clock observations, receives Signal-owned eligibility/suppression
  provenance, and invokes the same freshly admitted installed application
  operation without importing Runtime Bridge or Signal;
- active temporal wakes reconstruct exactly from current authoritative
  Relational/domain truth after application-runtime reinstallation, while
  cancelled, superseded, completed, stale-generation, and foreign-clock cases
  invoke nothing and commit-before-observation cannot duplicate effects;
- capability scope, purpose, delegation, conflict-of-interest, field
  disclosure, break-glass approval, expiry, revocation, and review are
  exercised and independently challenged;
- Foundational projection and diagnostic masks constrain field legality while
  compile and runtime hostility proves masks, digests, profiles, diagnostics,
  provenance rows, lineage artifacts, and boundary receipts open no Query
  authority;
- one installed application query preserves canonical identity, result shape,
  ordering, disclosure, and scope across every supported one-shot, continuation,
  historical, live, and preview lane;
- installed-query filters, sorts, traversal, and nested expansion lower through
  the existing Milestone 9.10 requirement, inventory, budget, admitted-plan,
  and receipt chain, with exact-zero covered per-result neighbor lookups and
  zero undeclared fallback;
- read, mutation, explanation, history, and live surfaces enforce identical
  scoped authority;
- monetary invariants and an independent double-entry oracle agree;
- concurrency, stale detection, idempotent retry, response loss, and failure
  injection produce honest typed outcomes;
- every post-Phase-7 authority and receipt retains the exact admitted branch,
  rejects equal-version cross-branch substitution, and contains no hard-coded
  ordinary-branch authority;
- revocation prevents subsequent unauthorized live delivery;
- every indeterminate outcome exposes an actionable governed recovery posture;
- Query-owned execution counters lower into honest Foundational
  counter-backed performance evidence at explicit support/certification
  boundaries, with ordinary and reconstructive work kept separate;
- certification-only replay agrees with ordinary authorized result meaning;
- all workaround deletions and permanent prohibitions are enforced; and
- the closure ledger has no unresolved high- or critical-impact row.

## Handoff To Milestones 9.17 Through 9.22

This handoff additionally requires Milestone 9.16.2 closure. Milestone 9.17 may
consume stable package reconstruction and fresh readmission. It remains an
in-memory runtime milestone: branch-local MVCC, product-world history, runtime
authority, and existing-outbox eligibility are live-process state rather than
archive or persistence content. Worth Store integration later owns durable
component and composite recovery without changing Query package meaning.

The Milestone 9.17 sequence replaces the conservative single-product-branch
and global-coordinator limits. Milestone 9.17.1 establishes exact owner-issued
Relational and Signal component bases plus Relational branch-local MVCC;
9.17.1.1 and 9.17.1.2 close the required Relational and Signal owner-service
boundaries; 9.17.2 establishes Runtime World-owned composite product branches
and coordinated publication; 9.17.3 carries the completed authority through Query
and cuts over the public facade. Query continues to carry the branch-affine
authority established here; it does not become the owner of component truth,
composite correspondence, version allocation, or conflict mechanics. Product
branches may share one immutable Signal basis while their Relational branches
diverge, and unrelated branches must progress concurrently.

Milestone 9.18 then accepts tree-based semantic undo and redo over exact source
composite commits and target product-branch heads. Corrections coordinate
owner-local retain, inverse, compensation, reapplication, and Signal
reconciliation posture, publish freshly admitted composite commits, and
preserve every prior history alternative. Query owns correction semantics and
DX but no component or composite history head. Merge, rebase, multi-parent
publication, offline synchronization, and distributed recovery remain governed
by the cross-runtime merging-and-branching roadmap.

Milestones 9.19 through 9.22 add advanced computation only through the public
typed declaration, admission, execution, publication, and certification path
proven here and the accepted branch/aftermath paths above. Advanced search,
spatial access, membership, paths, bulk execution, decision attachments, and
reuse bind to the installed application-query, capability, disclosure, basis,
recovery, and aftermath contracts. They may extend that path; they may not
reintroduce a specialist-only authority lane, provider-owned cursor,
field-disclosure bypass, or replay disguised as redo.

Geometry and other high-fan-out kernels consume installed typed slots, paths,
masks, plans, and fixed-width semantic identities. Adding cells, features,
nodes, edges, candidate pairs, traversal steps, projected fields, or result
rows must not increase canonical-basis preparation, digest derivation, or
digest-text materialization. New geometry-specific semantic families may
prepare identity only at their declared installation or bounded request-
admission seam and must carry that proof through execution. Any design that
requires per-cell, per-feature, per-node, per-edge, per-candidate, or per-result
semantic hashing is outside the ordinary contract and must be rejected before
kernel execution rather than accepted as undocumented optimization debt.
