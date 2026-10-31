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

### Live abandonment and recovered disposition

Phases 3–4 use recovered-only disposition for a live directory preparation
that cannot complete. Its next action and attempted live continuation/release
answer `ReopenRequired`. The original Query entry and World custody remain
owned. Explicit `try_into_recovery` abandons the World phase; it does not
discard the Native proposal/candidate retained separately in that entry,
release the business key, or establish absence of a Native effect.

The distinguishing production court starts an ordinary Host mutation, pauses
after its one genuine Native preparation and Query fences, then cancels the
original request. A duplicate must observe the same recovery identity without
invoking the handler or taking the original checkout. After explicit handoff,
inspection and live release must retain the same custody and advertise
`ReopenRequired`. A post-Native-effect panic must retain the actual effect and
leave the World head unchanged. Neither case may manufacture a no-effect
classification from a count, a stage, or an empty live discovery result.

The court then ends the current Home/process, reopens the same directory in a
fresh process under Store's mutation lease, and reconciles the complete
original association without invoking the handler. Recovered release must
publish the original `Released` subject before removing custody. Kill before
and after that publication, and interrupt recovery: released custody must
never reappear. A fresh same-key operation is admitted after actual recovered
no-effect disposition. Effect-bearing custody must preserve its original
receipt, position, digest and retained result; it never becomes `OpenOnly`.

`ReopenRequired` is a caller action, not permission to erase custody. Retain
stable request/correlation data, end ownership of the current Home, and reopen
through the Host. Old recovery handles are incarnation-fenced; obtain claims
from the newly readmitted Home. Close must retain/refuse accepted custody or
return its actual owned recovery posture. Generic disposal and the explicit
preparation handoff remain unbudgeted; neither reports successful disposition.
The Host README and `docs/build-an-application.md` must explain this procedure
and lease contention while the old Home remains owned.

World derives a live record's action from its actual retained durable-open
custody. Such a record offers inspection and `ReopenRequired`, rather than
live release, settlement or a fresh publication. Memory records preserve
their existing actions. Recovered custody keeps its distinct disposition
operations. Replace `DurableDispositionRequired` with
`RuntimeWorldRecoveryDenial::ReopenRequired` and add
`ProductUnpublishedNextAction::ReopenRequired`; re-export both actual diagnostic
data types through Query's declaration/Host facades, without a second wrapper
or a Runtime World service port.

The caller-facing diagnostic surface is:

```rust
use worth_query_decl::facade::{
    ProductUnpublishedNextAction, RuntimeWorldRecoveryDenial,
    WorthQueryProductUnpublishedRecovery,
};

fn needs_reopen(
    claim: &WorthQueryProductUnpublishedRecovery,
) -> Result<bool, RuntimeWorldRecoveryDenial> {
    Ok(claim.inspect()?.next_actions()
        .contains(&ProductUnpublishedNextAction::ReopenRequired))
}
```

### Native inventory admission inside D8

Use the existing `RecoveredRelationalRuntimeAuthority`. It proves genuine
reconstruction of one exact Native runtime and captured roots. It does not
prove continued quiescence or authorize live no-effect release. A discard
report, association bytes, a handle and ordinary discovery remain descriptions.
Another independently validated proposal can carry the same association bytes;
discarding one candidate cannot fence publication by all such proposals.

The not-yet-exposed production D8 composition supplies the exclusion boundary:
Store reads the selected Native checkpoint and both complete uncovered tails;
Host then immediately readmits World before initialization or service. Keep
the exact caller seals on native recovery extraction and World readmission.
The public partial-Home type alone is insufficient, since it exposes ordinary
Native methods when consumed.

Add `RecoveredRelationalRuntimeAuthority::discover_recovered_associated_effects`
in `worth-relational/src/durability/authority/recovered_associated_effects.rs`.
It accepts the original execution request and actual borrowed Native runtime,
unconditionally checks exact recovery/runtime affinity, then obtains the full
existing Native associated inventory. Reuse `RelationalAssociatedEffectRefusal`
and its `ForeignRuntime` cause. It accepts no caller-supplied vector and returns
no public absence witness. Its only outside-owner caller is World's existing
`lifecycle/builder/home_readmission/associated_effects.rs`, enforced by the
construction caller seal.

Pass the actual recovery authority through that private World inventory
consumer before any reconciliation or empty-World return. Preserve discovery
of both catalog and performed-publication residency, source validation,
duplicate/orphan/foreign association refusal and unsettled-effect refusal.
Only private complete reconciliation may derive an absent recovered effect
and construct the existing `RecoveredNativeCustody::OpenOnly`.
`NoNativeEffect` names that semantic condition here; it is not a new public
proof type or an implementation claim about a symbol of that name.

The destination remains the existing owner topology:

```text
worth-relational/src/durability/authority/
  associated_effects.rs                 existing descriptive discovery
  recovered_associated_effects.rs       new recovery/runtime inventory admission
worth-runtime-world/src/lifecycle/builder/home_readmission/
  associated_effects.rs                 consumes the actual recovery authority
  reconcile.rs                         private complete fate classification
worth-runtime-world/src/recovery/
  product_unpublished/actions.rs        actual live-custody next actions
  recovered_publication/release.rs      existing durable-before-live disposition
worth-query-execution/src/domain_computation/product_publication/
  preparation_pending.rs               explicit handoff and retained ownership
  recovery.rs                          caller-facing reopen/recovery diagnostics
```

Implement in authority order: Native affinity admission, World inventory
cutover, exact construction seal, then explicit World/Query reopen diagnostics
and Host documentation. The negative admission court uses an actual live
Native runtime, genuine recovery authority belonging to another runtime, and
an empty World source. It must refuse before World construction or absence
classification; the exact same-runtime recovered counterpart must succeed.
A source-seal control must reject an ordinary outside caller of the inventory
admission. These local controls complement the fresh-process production court;
neither substitutes for the other.

Pure terminal preparation refusal removes its exact business entry. Its opaque
owner may retain actual partial reservations until unbudgeted disposal, so
physical resource availability can still refuse another operation. Only an
early refusal before those reservations supports a fresh-commit-while-held
claim. Do not generalize that local proof to later resource-owning refusals.

## Eventual World write boundary

No worker is assigned here. Before assignment, implement/review the shared
values, frontier input/output, frame constants/limits and concrete owner-port
signatures. The candidate independent responsibility is World's codecs,
owner-medium port, issuer recovery, occurrence/head/ancestry catalog and custody
reconciliation. Record exact files after tracing their current owners.

That worker must not edit Store opening/manifest/close, shared substrates,
Query installation/receipt reconstruction, or Relational's envelope version
and association field. The Phase 3 implementation agent owns those integration
edits; the main chat's Root coordinates assignment and final landing. Local World proofs do
not certify the joint production-host lost-response/kill/reopen/retry journey.
