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

## Native index catalog durability prerequisite

Existing-Home Query readmission requires actual recovered Native index
definitions. The Native catalog is owner-issued persistent configuration and
ID provenance: registration order matters, duplicate equivalent definitions
are lawful, and canonical authored writes reference these identities. It is
not derivable from Native schema authority or from Query's expected shapes.
Generation images remain derived data. Tail-only recovery currently loses the
catalog, while close may lawfully have no checkpoint. Close-time checkpointing
alone cannot close this boundary.

The decisive courtroom uses a real filesystem Store Home with no selected
checkpoint. Register actual field, ordering/join and untouched non-Query
definitions, including equivalent definitions at distinct IDs; commit ordinary
Native records, release/drop the original owner and reopen from the actual
canonical and transition tails. Require exact original definitions/IDs,
subsequent allocator continuity, genuine record reads and owner generation
build/demand. A same-live-runtime lookup is not recovery evidence. Removing
the catalog from the authoritative projection must make this journey fail.

Put one required, privately constructed `CanonicalIndexDefinitionDelta` in
the existing Native canonical envelope BEFORE its one seal/Arc/artifact/route
construction. Its wire meaning is `base_next_index_id: u64`,
`next_index_id: u64`, and the ordered actual
`definitions: Vec<DerivedIndexDefinition>`. These frontiers copy the existing
issued-ID range; they allocate no new identities. Preserve every name, kind,
scoping value and ID, including distinct duplicate definitions. There is no
separate catalog journal, artifact, Query registration fallback, caller vector
or second preparation algorithm. Live IndexCatalogCoverage incarnation/epoch
must remain nonserialized and be freshly constructed after recovery.

Read accepted coverage BEFORE freezing the live catalog. Capture the actual
immutable catalog Arc and next issued ID under the SAME indexing-state read;
use that SAME snapshot for both delta and authored-index-write preparation.
The first actual canonical commit carries the full uncovered prefix. Later
commits traverse only its uncovered issued-ID range. Registration remains
process-local until covered by a real committed canonical publication or an
actually selected checkpoint. Concurrent later registration never changes an
already prepared delta.

Retain accepted coverage on the EXISTING PerformedPublicationNode as
`max(previous effective catalog frontier, this captured delta's next ID)`.
Assign that fact only inside the existing actual committed cutover, and apply
the same monotone rule in `link_recovered`. Preparation reads the actual head
node directly in O(1), not refresh_index, history scans or envelope body demand.
Its baseline is the maximum of this accepted node fact and the required
catalog frontier in the existing genuinely acknowledged checkpoint selection.
Do not add an allocator, shadow acknowledgement counter, registry, scheduler,
worker, live ownership mode or source-independent progress authority.

Prepared, refused, no-effect and indeterminate outcomes do not advance accepted
coverage. Preserve the original frozen payload and existing unresolved
publication custody. Actual committed cutover acknowledges only its captured
prefix, never a newer live next ID. Older concurrent candidates may repeat
equal overlapping definitions; exact equal overlap is idempotent and progress
cannot regress. Gaps and conflicting definitions at an existing ID refuse.
Admit the actual source payload before recovery replay consumes its IDs.
Carry its ORIGINAL delta privately through the existing recovery context into
the one preparation path; do not regenerate a reply from the restored catalog.
Preserve original bytes, digest, position and receipt.

The catalog is global across branches. Do not prune sibling-only definitions
or validate the entire inventory against every individual replay branch's
schema. Keep existing ordered replay of both tails. Branch transitions alone
do not persist pending registrations; a covering canonical commit does. Forks
preserve catalog/history meaning without acquiring another runtime's durable
publication authority. Definition admission precedes genuine owner-derived
generation build/demand; no generation image is invented as authoritative data.

The current ordinary schema-transition admission requires identical executable
kind contracts. Its real courtroom may exercise sibling schema versions and
globally registered untouched/nonapplicable definitions, preserving their
actual IDs. Do not invent differing executable contracts or another transition
authority to stage this case. Absence of per-branch catalog filtering is also
reviewed at its actual owner; no distinct-executable-schema runtime proof is
claimed here. Full future transition support remains D16 work.

Make the delta required in Full, Authoritative and Checkpoint envelope
projections, including embedded checkpoint-envelope serialization. Cut over Native
artifact v3 to v4 and Native checkpoint CHK16 to CHK17; older development
formats refuse before effects. Missing required catalog information cannot
mean an empty catalog. Preserve the Store Relational section's WSRCKPT1
version-3 outer format, registry and owner directions: Store carries opaque
Native bytes and never gains a Native schema/header parser or raw getter.

Checkpoint definitions AND their required next-ID frontier come from the SAME
captured index state. The current later definitions_snapshot reread is cut
over, since registration does not take the canonical checkpoint gate. Validate
monotonicity in canonical position, transition position AND catalog frontier
through existing source-bound publication validation before Home acceptance
and through actual acknowledgement. Otherwise an older catalog-only capture
at the same canonical/transition positions could erase newly selected meaning.

The Store cold-directory restore seeds its actual existing checkpoint
selection. Expanded checkpoint restoration has no RelationalCheckpointDirectory;
do not fabricate one or treat an unacknowledged live checkpoint as selected.
Safe one-time equal-overlap re-emission after expanded restore is allowed as
declared reconstructive work. A failed checkpoint cannot acknowledge its
captured definitions. A canonical publication overlapping a selected checkpoint
remains admissible only when exact shared definitions agree.

Destination topology under `crates/worth-relational/src/`:

```text
history/data/canonical_commit_envelope/index_catalog.rs [create: private wire meaning]
runtime/state/subsystems/indexing/catalog_publication.rs [create: actual snapshot/range admission]
authority/commit/pipeline/artifact_execution/index_catalog.rs [create: request-bound preparation]
durability/authority/runtime_rebuild/index_catalog.rs [create: source admission/original carry]
existing performed stream/preparation binding [revise: accepted carrier projection]
existing artifact/checkpoint codecs/capture/selection [revise: required grammar and one capture]
```

Keep catalog meaning, preparation, accepted publication and recovery at their
existing authority owners; add no empty successor modules. No new public
authority API is required. Query continues using existing recovered-definition
lookup. Update the Native README's registration/recovery semantics and format
compatibility and the affected provider documentation against real exports.

Unchanged ordinary preparation compares scalars and emits an empty delta.
Changed extraction costs O(log catalog size + introduced/overlap definitions
and their bytes), without copying/serializing the full inventory on every
ordinary commit. New retained allocations use the original request and
existing reservation mechanisms. Checkpoint/recovery may traverse the catalog;
registration's existing immutable-catalog replacement cost is not an ordinary
commit cost claim. No hard wall-clock, whole-close or D11 claim follows.

Before integration prove reverse preparation/commit ordering on different
branches, registration after capture, checkpoint-only definitions overlapping
an older frozen tail, same-position checkpoint regression refusal, failed
checkpoint plus tail recovery, expanded-checkpoint overlap, missing/gapped/
conflicting deltas, old-format refusal and actual no-effect/uncertain posture.
Use genuine producer/custody boundaries and existing controls; no fabricated
positive catalog authority or frontier assignment substitutes. Run focused
owner/real filesystem courts, format, scoped caps, boundary/context and fresh
independent source QA. Existing Query readmission remains unqualified until
this prerequisite is integrated and its real recovery evidence passes.

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

### No pending owner admission before service

The first existing-Home service slice supports zero retained recovery custody.
It must not derive TransitionSettled from program activation equality, a
support table, journal registration, an empty process-local Query store or
successful Native recovery alone. Full D16 transition execution remains open.

Under the continuously held Home mutation lease and closed pre-service
composition, Store inspects these exact unsupported journals:
`home.transitions`, `signal.definitions`, `signal.advances`,
`query.obligations`, `query.dispatch`, and `query.corrections`. For each actual
journal require `head() == None`, then accept only
`resolve(1, [0; 32]) == NotCommitted { head: 0 }`. The digest is a
nonauthoritative probe: the existing resolver does not compare it in the
empty-journal branch. Existing sequential publication admits only head+1, so
position 1 is the only lawful unresolved position of an empty journal.
Entries, Unresolved, Conflict, inspection-required posture, failed lookup and
read failure all refuse; no fallback converts them to absence. Consult the
original execution request through this admission. Also require actual Native
recovery with `codec_transition_pending() == false`. The selected checkpoint
must retain its existing exact owner-section admission; unknown owner sections
remain refused rather than silently ignored.

This inspection is not a concurrent absence lock. No publisher for these
unsupported families is exposed before service. Keep the closed Host and
Store construction seals; do not expose the raw Serving runtime or an open
journal publisher through this operation. Adding a supported owner requires
its actual readmission and an intentional support-contract change.

The consuming Store operation returns privately constructed, move-only
`NoPendingApplicationHomeTransition`, retaining the SAME actual
`WorldReadmittedApplicationHome<(), (), (), (), ()>` and therefore its original
Home, incarnation and lease. It needs no new HomeContext fields, identity
copies or Arc retention. Failure owns the same Home and exact typed cause.
It offers no public conversion from descriptions and no Clone/Copy. Its
consuming Home extraction is restricted to the closed Host completion.

The concrete Host custody payload retains Query and the actual Home. After
real custody admission, consume that payload, obtain the Store carrier and
wrap the private Query-plus-carrier payload in TransitionSettled. Only consume
that concrete specialization into the public Home service. Wire this gate
into fresh service as well as the subsequent existing-Home path; remove the
fresh-only comment as authority for transition settlement. A failed gate
returns owned Query and Home custody before any public service escapes.

The destination is `worth-store-application/src/open/readmission/transition.rs`
for the persisted no-pending inspection, with the concrete carrier, refusal
and source-local consuming method in
`world_section/readmission/transition.rs`. That method borrows the private
Home/Native context and moves the actual Home. Preserve private fields; add
no public raw access to solve module visibility. The Host's
existing `application_home/opening.rs` completion consumes the concrete result.
The known D16 transition successor belongs under the same open/readmission
axis; it may not reinterpret this absence carrier as completed migration.
Keep the existing canonical version-3 journal declaration and its exhaustive
required-empty classification together in `registry.rs`; add no second
namespace list or general registry framework. Native and World journals keep
their actual existing owner readmission and are not required empty.

```rust,ignore
impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    pub fn readmit_no_pending_owners(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<NoPendingApplicationHomeTransition, ApplicationHomeOwnerAdmissionRefused>;
}
```

The private carrier field is the same concrete WorldReadmittedApplicationHome.
The owned refusal exposes its typed reason and consuming Home return for the
existing failure-close route. Admit only the exact source-local transition
file to Native recovery extraction, and only the exact Host completion to
admission/carrier extraction. A public generic Proof wrapper cannot substitute
for either private payload. No new proof type, ownership mode or publisher is
introduced.

The decisive negative proof uses the existing external Application court and
its exact existing physical_serving_for_test control. It starts with an actual
filesystem Home and genuine Native/World composition, publishes one real entry to each unsupported
journal in turn, and verifies owned refusal rather than service. It must
convict deletion of any one of the six checks. A genuine empty counterpart
succeeds and moves the same Home/lease through the carrier. A stopped original
request preserves the same owner; compile-time/caller-seal evidence denies
forged construction and outside extraction. Prove unresolved-position refusal
with actual admitted ambiguous publication while the durable head remains
empty, requiring the exact new Store gate's typed refusal rather than an
upstream open error. Current Store sections use normal
`recovery_plan_from_source`/`recover_with_authority`, whose recovered authority
always has no pending codec transition. The sole pending producer is the
separate Native codec-transition restoration path, which cannot populate this
Home carrier. Preserve and review that construction boundary, and retain the
gate's pending-codec check as defensive future admission. Existing genuine
Native transition controls and early changed-codec refusal do not claim to
exercise this unreachable Store guard. Add no private authority replacement,
synthetic transition flag or extraction door to make such a test pass.
Do not invent an unresolved map in the fixture. Use exact
caller additions to existing controls, with no new getter or private-test
exception. Existing real Host Directory
mutation and ownership-close controls must remain passing.

### Existing Query publication readmission scope

The next dependent slice reopens a genuinely initialized Directory Home,
reads its latest acknowledged product state and closes it. ExistingArtifacts
must consume the same Store refusal owner into that path; initialization,
seed suppliers and business handlers never replay on reopen. Native index
admission uses RequireRecovered, and an activation seed exists only on fresh
initialization.

A single Query-owned bootstrap publication record is required in the existing
Native Query schema. Multi-batch seeding produces aggregate Query publication
facts not present in World genesis. Complete all actual seeds and indexes,
then commit the record before World bootstrap. World genesis binds that
record-containing Native basis; the record's bootstrap locator still names
the actual terminal bootstrap commit. The existing seed producer consumes
activation before the first real row commit, so its actual last-seed receipt
is terminal even when all rows were flushed before finalization. Preserve that
producer invariant and build indexes at the same admitted basis; never invent
an activation tail or guess the current head. Persist portable admitted schema/program meaning
and descriptive aggregate counts. Portable meaning here is the actual admitted
ApplicationHomeBinding: package/schema CanonicalDigestIds, selected program
identity/revision and complete closed-roster identities/revisions. Compare it
exactly and perform actual Native schema/receipt and RequireRecovered index
admission before rebinding the fresh installed identity. Do not duplicate the
full member graph or facts-only archive representation. Never persist process identities, runtime snapshots,
an extra receipt registry or a guessed current head. Rebind only after actual
portable meaning and immutable Native commit admission.

Both Query attachment preflight and product-source preparation must use the
selected World root's relational basis. Native's latest head is insufficient:
a genuine interrupted publication may leave Native at 9 and World at 8.
That effect-bearing state retains actual World recovery custody and must
remain owned and refused before service in this first slice. Public service
at World 8 in that adversary requires real recovered Query binding/custody
admission in the D9 successor, fencing the original key even though its Native
9 binding is absent from World 8. Do not create an empty Query store to erase
that obligation. Safe CommittedReceiptNotRetained is permitted where the actual
durable binding exists but the original process receipt is not retained; it
does not certify original-result retry. Full D8/D9/D11 closure remains open.

Before new service can mint an outcome identity, D17 also requires scanning
the actual latest-Native Query idempotency rows under the original request.
Validate actual row shape, uniqueness and immutable Native creation receipts,
then use the existing outcome identity restoration operation for every actual
row. A future retry resolver alone is too late: a new process could otherwise
mint an identity already present in durable state. This reconstructive scan
does not change the selected-World basis for ordinary product reads, create
another ledger or claim restoration of a missing process-local receipt.

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
`ProductUnpublishedNextAction::ReopenRequired`. Both actual diagnostic data
types already reach the Host runtime facade; preserve that route without a
second wrapper or a Runtime World service port. The declaration-only crate
gains no execution dependency or runtime recovery audience.

The caller-facing diagnostic surface is:

```rust
use worth_query_host::facade::primary_graph::{
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
no public absence witness. Its only production outside-owner caller is World's existing
`lifecycle/builder/home_readmission/associated_effects.rs`, enforced by the
construction caller seal. The existing Native certification court's exact
`relational/durable_publication/recovered_associated_effects.rs` file is also
admitted solely for the genuine foreign/matching runtime admission cases;
ordinary callers gain no inventory-admission audience.

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
