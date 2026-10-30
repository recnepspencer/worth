# Shared durable owner contracts

This design binds D2–D8, D10, D12 and D17 for the next home batch and eventual
disjoint World work. Names describe intended ports, not implemented APIs,
except the approved `DurablePublicationFate` and `DurablyCommitted`.

## Placement and trust

Foundational holds immutable identities, associations, families, retention and
protocol meaning. Proof holds legality and progression. Store Application
owns lease/incarnation/lifecycle; each runtime owns its counters, live catalog,
custody and readmission. Proof must not gain a dependency on Foundational or
an owner. Reuse boundary_protocol identity/version/unsupported-version types,
owner-marker sealing, LinearResource and the approved publication witness.

## Identity and issuer contract

HomeIdentity represents the canonical 16 bytes of the published
StableStoreIdentity. It is not a new UUID or a second identity issuer. An
observed address never authorizes initialization, opening or issuer creation;
only admitted home custody does. Store initialization remains the source.

HomeIncarnation is an immutable Foundation value, durably advanced by Store
Application with checked exhaustion before service. Authority handles and
recovery claims carry it and consult the current owner before effects. Durable
receipts/history identities remain valid across reopen. Recorded publishing
incarnation is historical data, not a capability.

Move World's cross-runtime identity meaning into Foundation: owner, product
branch, branch incarnation/reference generation, composite commit, publication
attempt, bootstrap attempt and unpublished owner effects. Keep normalized
branch-name semantics: recreate keeps branch identity but gets a new branch
incarnation. No artifact/row records the process-static owner ordinal.

DurableIdentity is sealed to approved shared data values. Artifact identity
fields and encoding require it; process/session/graph instance types cannot
satisfy it. Do not open arbitrary runtime implementations. Audit existing
native IDs' home context before moving definitions; do not sweep unrelated
identity families. New shared values go through the Foundation facade and the
owning runtime facade; old definitions are removed rather than aliased.

World retains one checked issuer with its existing identity families. Recovery
supplies the maximum durable value for each family from its checkpoint,
remaining journal tails and custody. The issuer resumes at the next value.
The existing World owner identity itself is readmitted, not replaced by a new
owner on each open. Only genuinely new owner initialization issues it; reopen
restores that owner and resumes its subordinate family frontiers.
Frontier snapshots are data; only the private-construction readmitted World
state configures a production issuer. No second counter, evidence-index high
water mark, clock-derived ID or readmission-time issuance is allowed. Never
wrap or reissue a durable value. Volatile attempts outside the durable prefix
follow D14; the reference resumes at the same recovered frontiers. Memory-only
composition has an explicit ephemeral scope and identical issuance rules,
without pretending it holds Store authority.

CompositeAttemptAssociation is one immutable Foundation value: home-scoped
publication attempt, product branch identity and branch incarnation. World
creates it from the admitted attempt. Relational carries it opaquely in its
bounded caller-association field under a new canonical artifact version.
World joins effects, occurrences and dispositions by this association, never
by an evidence-store row or process instance.

## Binding contract

Query Installation derives an admitted binding from the actual installed
schema, selected validated program and closed WorthQueryProgramSupportRoster.
Its private-construction carrier preserves that validation for composition.
The descriptor records program identity/revision, rostered revisions and schema
revision alongside home identity/incarnation.

Recorded digests are descriptive data: they cannot recreate
ApplicationProgramRevision, a roster or schema admission. Reopen compares them
with freshly validated caller meaning; the owner readmission port alone turns
bytes into authority. Changed build UUID is admitted as implementation lineage,
with no eager regeneration and no new authored program identity.

Extend the descriptor's declared tagged list without changing its landed
envelope. A Relational-only home lacking application binding refuses explicitly
rather than being adopted as the caller's program. Missing/foreign binding and
D16 transition-required/pending outcomes remain distinct. Transitions progress
after readmission under their durable plan digest, never as recovery guesses.

## Frame and registry contract

Store Application owns the spec's exact versioned journal/anchor registry.
Owners define/readmit their native codecs; Store persists opaque payloads and
checks physical framing, order and membership. Unknown registry/protocol
versions or tags refuse explicitly; no section is silently skipped.

Use these new canonical protocol families, initially version 1:

| Family | Owner |
|---|---|
| worth.application.home.checkpoint | Store Application |
| worth.world.occurrence | World |
| worth.world.product.branch | World |
| worth.world.custody.disposition | World |

Declare these stable owner-section tags: 1 Relational, 2 Signal, 3 Bridge,
4 World, 5 Query, 6 Dispatch, 7 Correction. They identify sections, not recovery
order or admission policy. Phase-4 World component evidence initially admits
the Relational entry with tag 1; later phases add owner tags without changing
existing fields. Unsupported entries refuse until their owner is supported.

The home checkpoint is one versioned envelope with bounded tagged/length-delimited
opaque native sections. Each native section owns its frame/version and journal
coverage; do not create a parallel authoritative coverage field or codec.
Preserve WSRCKPT1 version 3 exactly as the Relational section. Only the outer
selected anchor is home.checkpoint. Its one root decision contains the union
of selected/referenced members. Prepared sections hold their capture/selection
fences through publication; only successful selection acknowledges capture.
Cold sources receive the selected section/membership and never consult an
independent relational.checkpoint anchor.

Shared protocol unsupported-version vocabulary carries typed owner detail.
Bound lists, lengths, members and allocations before reading. Encoding and
readmission use the original execution account. A policy value grants no
authority to create a replacement request, inflate limits or reset allowance.

World occurrence records all Phase-4 fields: composite commit/parent, home and
publishing incarnation, branch identity/incarnation/generation, attempt and
provenance, program revision, correlation and extensible component evidence.
Product-branch and custody-disposition artifacts are separate. Their retention
is Authoritative. Live obligations use owner custody plus LinearResource,
not an expiring receipt cache.

## Recovery and service contract

Store Application consumes named Proof stages in order:

1. StoreRecovered: physical recovery and continuously held lease.
2. HomeAdmitted: descriptor/binding admitted, incarnation durable.
3. RelationalReadmitted: one runtime, native section and both tails.
4. SignalReadmitted: Signal/Bridge readmission or authoritative unsupported
   family absence under the exhaustive directory support declaration.
5. WorldReconciled: branch/history/occurrence/custody/frontiers readmitted and
   every associated owner effect classified.
6. QueryReadmitted: supported binding/outcome/program/workflow state readmitted;
   unsupported persisted families refused.
7. CustodyReadmitted: dispatch/correction custody or authoritative absence.
8. TransitionSettled: D16 progress completed under the recorded digest, or
   readmitted proof of no pending transition.

Each predecessor is consumed, moving actual custody. Proof may hold one
shallow owner-sealed payload, as the publication witness does, without runtime
dependencies. Concrete runtime carriers have private construction, no
Clone/Copy, and no public conversion from descriptions. Composition demands
the concrete specialization. Boolean/unit success grants no next phase.

Only completed progression constructs service. Unsupported-owner absence is
checked against persisted state and all operation capabilities, including
failure-only obligations. It is not a dummy readmission. Relational readiness
cannot mint World/Query proofs. Adding a supported owner breaks construction
until wired in. Steps 1–7 never republish, invoke handlers, replace commits or
issue occurrences. Step 8 is explicit forward progress under the same request.

World's readmission port returns private-construction ReadmittedRuntimeWorld,
containing actual catalogs, product heads, ancestry, custody and frontiers.
The issuer consumes this state, not an independently supplied frontier map.
Store Application can then construct its concrete WorldReconciled carrier.
Every associated effect becomes a matched occurrence, durable disposition,
unresolved fate or retained ProductUnpublished(RecoveredWithoutProductMovement)
custody. Unsettled commits/unresolved forks retain their original custody.
Continue/release publishes world.custody before live disposition; released
custody never reappears. Only Committed occurrence publication mints the exact
subject-bound witness consumed by the product movement.

## Eventual World write boundary

No worker is assigned here. Before assignment, implement/review the shared
values, frontier input/output, frame constants/limits and concrete owner-port
signatures. The candidate independent responsibility is World's codecs,
owner-medium port, issuer recovery, occurrence/head/ancestry catalog and custody
reconciliation. Record exact files after tracing their current owners.

That worker must not edit Store opening/manifest/close, shared substrates,
Query installation/receipt reconstruction, or Relational's envelope version
and association field. Root owns those integrations. Local World proofs do
not certify the joint production-host lost-response/kill/reopen/retry journey.
