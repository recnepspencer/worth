# worth-relational

`worth-relational` is a standalone truth runtime for graph-shaped state.

It is for systems that need state to be authoritative, transactional,
inspectable, replayable, and durable, rather than a mutable data structure
whose correctness rests on convention.

It is meant for more than CRUD apps. The target is systems like order and
inventory management, account ledgers, document approval workflows, planning
systems, and other runtimes where history, determinism, replay, validation,
branching, and durable recovery are first-class.

The library is built around a few obvious jobs:

- build runtime
- write truth
- read truth
- manage snapshots and branches when you need controlled views of truth
- inspect what happened
- go to history or replay when you need past truth
- use merge, validation, compiled artifacts, retention, indexes, and recovery
  when the workload gets deep

Start here:

- [`QUICKSTART.md`](./QUICKSTART.md)
- [`DAILY_WORKFLOWS.md`](./DAILY_WORKFLOWS.md)
- [`API_OVERVIEW.md`](./API_OVERVIEW.md), including [persistent aggregate identity](./API_OVERVIEW.md#persistent-aggregate-identity)
- [`BRANCH_LOCAL_MVCC.md`](./BRANCH_LOCAL_MVCC.md)
- [`OWNER_COMPONENT_PORT.md`](./OWNER_COMPONENT_PORT.md)
- [`TESTING_WORLDS.md`](./TESTING_WORLDS.md)

Examples:

- `cargo run -p worth-relational --example basic_runtime`
- `cargo run -p worth-relational --example snapshots`
- `cargo run -p worth-relational --example history_and_replay`
- `cargo run -p worth-relational --example derived_indexes`
- `cargo run -p worth-relational --example validation_and_durability`
- `cargo run -p worth-relational --example branch_local_mvcc`

## Minimal shape

```rust
use worth_relational::facade::{
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    schema::RelationalSchemaRegistry,
    transactions::WorkerIntentBatch,
};

use worth_execution::ExecutionAllocationPolicy as Allocation;

let runtime = RelationalRuntimeApi::builder()
    .schema_registry(RelationalSchemaRegistry::new())
    .build();

let main = runtime.main_branch_identity();
let (_descriptor, basis) = runtime.observe_branch(&main)?;
let mut tx = runtime.begin_branch_transaction(
    &basis,
    RelationalTransactionIntent::ordinary(),
)?;
tx.push_batch(WorkerIntentBatch::new("example"), Allocation::SystemAllocation)?;
let _outcome = tx.commit(&runtime, Allocation::SystemAllocation)?;

let _truth = runtime.read_truth();
let _snapshots = runtime.snapshots();
let _history = runtime.history();
let _inspection = runtime.inspect_what_happened();
```

Staging and commitment require an explicit allocation policy selecting System
allocation or an actual process lease. Native branch, schema, materialization,
savepoint-count and physical allocation checks remain. Transaction overlay and
footprint quotas do not add another admission gate. Query forwards the caller's
actual allocation policy and live request control through native validation.

Relation-integrity preparation retains complete affected scopes without a
profile-selected aggregate entity, relation, edge or traversal-count quota.
Transaction cancellation/deadline and the selected allocation policy are checked
through preparation; allocation stops retain their typed cause. Declared relation
rules, visited-set termination, and leased execution work, scratch and result
admission remain. Preparation's existing BTree/Vec and adjacency-copy heaps are
not admitted by these liveness checks. Prepared root byte costs remain observable
without an aggregate admission ceiling. Companion cells retain their actual Arc
ownership and publication coordination without a separate retained-cell byte
counter. Patch-count, candidate, snapshot, branch retention and companion
preflight controls remain; this does not establish a 100,000-member or
million-member lifecycle result.

Native acyclicity checks borrow the prepared visible and planned edges. Their
adjacency index, vertex index, visited bytes and frontier are admitted together
at their actual simultaneous capacities. Repeated reachability searches reuse
that scratch; freed successor buffers do not accumulate as retained memory.
Every repeated search and duplicate edge still consumes request work. Sorting,
lookup and traversal observe the original request's work and interruption stops;
refusal grants no validated candidate or publication.

Repeated equality selections can prepare an installed entity-field index with
`index_access().prepare_entity_field_lookup(&view, index, kind, &locator)`.
The opaque owner retains the exact admitted view's root and index generation;
both preparation and execution reject another runtime's authority.
`execute_prepared_entity_field_lookup` borrows that retained root for each value,
preserves complete finite selection and corruption checks, and polls the supplied
live callback. Certification still compares against authoritative storage at
that same root. This callback controls interruption, not temporary buffer
allocation admission. The separate admitted lookup retains its existing
64-candidate work/storage contract.

An installed derived index can discard its currently catalogued generations with
`index_authority().discard_generations(DerivedIndexDiscardRequest::all_bases(index))`.
This explicit scope removes every branch, basis and publication status for that
index under one catalog write lock. Definitions, authoritative roots, history and
unique-field enforcement remain. Already-held generation readers can finish;
fresh selection refuses cold access until an admitted build publishes again.
Rebuild uses the ordinary `build_for_basis` path and its exact native
basis. An old ordered continuation does not silently rebind to a new generation.

Discard is cache maintenance, not a durable tombstone or a promise of total heap
reclamation. A concurrent build may publish afterward, held readers and historical
artifacts may share backing, and later recovery replay may publish valid artifacts.
A checkpoint captures the catalog as it exists at capture time; reopening a
checkpoint captured after a completed discard does not itself reconstruct the
missing generations from source records.

## Durable index definitions

`index_authority().register` installs a definition and its original issued ID in
this owner. The next authoritative canonical commit carries the newly introduced
ordered definition prefix, including full names, kinds, scoping and duplicate
equivalent definitions at distinct IDs. Artifact v4 requires this delta in full,
authoritative and embedded checkpoint envelope projections; CHK18 carries the
catalog and issued frontier from one capture. Earlier development formats refuse.
Only an actual performed canonical cutover or acknowledged checkpoint selection
advances the accepted frontier. Preparation, refusal and uncertain fate do not.

Recovery admits checkpoint definitions and exact overlapping tail declarations
before replay, retaining the original tail payload and catalog-prefix coverage.
The catalog is global; sibling schema bases do not prune it. Embedded restoration
retains original canonical backing as data; an unselected standalone checkpoint
can re-emit checkpoint-only definitions. Actual selected cold restoration and
completed selection acknowledgement install accepted catalog coverage.
Generations remain derived: use the existing owner build/demand doors.
Catalog coverage incarnation/epochs are live observations and never enter wire.

## Mental model

- `RelationalRuntimeApi::builder()` is the setup door
- an explicit identity plus owner-admitted basis is the branch-selection door
- branch-bound transactions are the write-truth door
- `read_truth()` is the explicitly current standalone-truth door
- an exact observation plus `snapshots()` is the repeatable-view door
- `inspect_what_happened()` and `publication()` are the readback doors
- `history()` and `replay()` are the past-truth doors
- `validation()`, `compiled_artifacts()`, `retention()`, `durability()`,
  `commit_strategies()`, and indexes are the deep-system doors

If you find yourself reaching into crate internals instead of
[`facade`](./src/facade.rs), you are probably leaving the intended public
surface.

## Opaque caller association

`facade::mvcc::RelationalCallerAssociation::from_bytes` accepts nonempty opaque
bytes of at most 4096 bytes. Bind them by consuming a validated proposal with
`bind_caller_association` before the existing `prepare_validated_proposal` call;
binding twice refuses and a prepared candidate cannot be rebound. Candidate
`caller_association()` observes its retained sealed payload; the canonical
envelope exposes the same immutable bytes. Native preserves the caller's bytes
without interpreting them or granting completion authority. Canonical artifact
v4 and native checkpoint CHK18 retain the field through journal and embedded or
cold checkpoint recovery. Older artifact and checkpoint versions refuse rather
than acquiring the new grammar implicitly. Association bytes or successful
artifact decoding alone establish no durable World effect or product movement.

The canonical envelope also requires the original immutable
`facade::transactions::RelationalCommitSemanticResult`. Native produces it once
before sealing the envelope and shares that same value with the live result.
It retains canonical create names and assigned identities, ordered changed
records, structural/schema/history/change/aspect meaning, actual publication
counts, and original validation checks and decisions. It has no live snapshot,
timing, work measurements, publication status, journal position or self-digest.
Each checkpoint commit index explicitly records `RelationalAssociationDiscovery`
from the same envelope, so discovery need not demand unrelated cold bodies.
Missing semantic images or discovery fields refuse. Artifact v1/v2 and
checkpoint CHK13/CHK14/CHK15 refuse cleanly. These data fields establish neither
source-backed recovery nor recovered publication authority on their own.

## Record a current Native component

`owner_component_services().basis_port().export_recorded_current_component`
takes the original execution request and an owner-admitted current basis. It
returns a serde-transportable `RecordedRelationalComponentReference` in Native's
canonical runtime frame. Its explicit `Empty` target has no commit, position or
digest; `Canonical` retains the actual full receipt, performed position,
artifact digest and roots. The descriptor retains exact reference, truth,
lifecycle, schema and visibility axes. Foreign or stale bases refuse. This is
descriptive export: memory backing remains memory-only, and the reference grants
no retained-root, durable-source or recovered-publication authority. Native
recovery must admit the recorded subject before it can become an operational
basis; callers cannot rewrite its canonical runtime frame into a live identity.

`export_recorded_retained_component(execution, &original_basis)` uses that same
owner fact path for an already-admitted, pinned subject after a later commit
advances the branch. It preserves the original performed position and artifact
digest; it neither selects the latest head nor pairs a caller-supplied result
with a descriptor. Foreign owners refuse. Historical Journal or checkpoint
subjects demand their exact catalog-backed positioned body under the original
request. Independently readmitted bodies must match the complete authoritative
payload; resident canonical Arc identity remains the constant-time acceptance
path. A failed source demand returns a source refusal, without substituting the
current head or retaining every journal body permanently.

Recovery binds each reconstructed current root to the authenticated original
canonical envelope before recording it as a historical image. Original
validation is retained from that source, without rerunning its executor. Replay
private completeness survives only the catalog's existing full authoritative
payload equivalence; it is not transferred to a different original validation
result. Recorded-component admission checks that same complete artifact payload.

## Retire one provider member-custody share

`RelationalDurableMemberCustody::new_from_source(owner, issuing_weak_source)`
binds the original provider payload to its actual source allocation. Consuming
`into_retirement()` produces `RelationalDurableMemberRetirement`; retain it
outside disposable callbacks and advance it with the original execution request
and borrowed issuing source. Native checks issuance identity before callbacks or
share consumption, without upgrading the Weak. Unknown `new(owner)` payloads
and composite region exports return `UnsupportedProvider`; foreign sources and
foreign custody refuse without replacing the original payload.

`Complete` discharges this local share only. Another clone or independently
retained cold reader continues protecting its original placement. After Complete,
`into_retired()` returns opaque allocation custody; `reclaim()` and generic final
Drop are explicitly unbudgeted. Incomplete Drop may run the original provider
destructor and grants no close proof. Neither result establishes whole cold graph,
Native, Home or physical close.

The sized boxed-owner representation adds one acquisition allocation and one
indirection. Advance allocates/reserves nothing, never waits for locks or destroys
the original payload/last physical owner. A Store member step is O(log distinct
members); BTreeMap removal may rebalance/deallocate nodes, and Arc share release
may reclaim a small control allocation. These are not hard O(1) or wall-clock
destructor bounds.

## Readmit an associated Native effect

`durability_authority().discover_associated_effects(execution, &association)`
selects actual catalog or performed-route subjects from their required discovery
metadata. Unrelated cold bodies remain cold. Each returned subject records the
exact runtime and Journal index with weak identities for the actual source and
cell; the live Native owner retains their custody. Associated Memory refuses.
`discover_all_associated_effects(execution)` uses the same metadata scan without
a known-association filter. It returns all associated subjects with their exact
opaque bytes, so the caller can validate its own namespace and original opens
before claiming reconciliation. It skips unassociated entries without loading
them and retains the same Memory, missing-metadata and changed-subject refusals.
Native interprets no caller namespace.

For closed readmission, the actual recovered authority's
`discover_recovered_associated_effects(execution, &runtime)` unconditionally
checks its exact runtime affinity before this same full inventory scan, including
an empty inventory. This historical authority grants neither a live publication
fence nor an absence witness; the not-yet-exposed composition supplies exclusion.

`readmit_associated_effect(execution, &subject)` reads that source even when a
body is resident, validates its full receipt, position, digest, association and
schema/root metadata, and exposes the original immutable semantic image.
Refusal leaves the borrowed original subject intact. Closed or unavailable
owners refuse, and descriptive observers do not keep a closed Home serving.

Settlement descriptions distinguish matching `PerformedPending` and
`PerformedSettled` route flags from `CheckpointReadmitted` directory evidence.
Missing registration is not settlement evidence. These read-only observations
grant no recovered publication or product adoption authority.

## Admit an original recovered publication

`durability_recovery().recover_with_authority(execution, plan)` runs the existing
reconstruction once and returns `RecoveredRelationalRuntimeAuthority` alongside
the outcome. Its `admit_recorded_component(execution, &runtime, &recorded)` door
admits an exact source-produced Empty or canonical image with real observation
pins. This is distinct from its existing current-image admission. Canonical
Memory, foreign runtimes, mismatched source fields and unavailable images refuse;
no old runtime identity is rewritten and no latest root is substituted.

If a crash preceded the caller's occurrence, use
`recorded_publication_successor(execution, &runtime, &effect)` to describe the
original successor. Native selects the same effect's exact source/index and an
unambiguous original observation captured by this recovery, even after later
Native advancement. Missing or ambiguous images refuse while the effect stays
borrowed. This grants no basis, settlement or adoption authority and never uses
the current head as a substitute.

`admit_recorded_publication(execution, &runtime, effect, &before, &after)` consumes
an actual `ReadmittedRelationalAssociatedEffect`, validates its original
predecessor and complete successor subject, and acquires actual component leases
for both bases before returning
`worth_proof::RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>`.
The phase offers read-only `custody()` and one consuming transfer. Its private
Native payload exposes the original semantic data and actual bases; consuming
`into_retained_parts()` moves the effect and both basis/lease pairs to the next
owner. Native creates no live `Performed`, `CommitResult`, snapshot or timing.
Pending settlement must be completed by its existing actual owner first.
An owning refusal preserves the original source image and any acquired pins;
`into_effect()` explicitly releases partial new pins for a fresh request.

CHK18 records exact current observations and live external component obligations.
Canonical root headers require `recorded_components`; explicit
`retained_empty_components` bind Empty observations to their schema carriers.
These fields come from the same actual observations that created the external
leases, rather than an arbitrary descriptor or the previous checkpoint union.
Capture retains those selected actual bases through encoding and actual selection
acknowledgement, or until the last actual capture is abandoned, even if a caller
releases its original lease. Clones share that custody; interrupted acknowledgement
keeps it until the original progression completes. Plain observations and released external obligations do not
become additional durable component obligations.

Reopen installs current heads and retains historical component headers without
reading their artifact bodies. Exact admission demands the original source
artifact and immutable region directory, verifies all recorded commitments, and
acquires real pins before exposure. Unrelated history remains cold; missing or
mismatched selected material refuses. Checkpoint coverage alone grants no adoption.
The concrete Store/World recovered-adoption journey and coherent cross-owner Home
checkpoint cut remain separate integration obligations; this Native phase alone
grants no product movement.

## Durability

A runtime is memory-only by default: its canonical history lives in an
in-memory log and `durability().recovery_plan(..)` reconstructs from it.

A durable runtime is built with `build_durable(Box::new(medium))`, where
`medium` implements `RelationalDurableMedium` (Worth Store's Relational home
is the production implementation). The medium is moved into exactly that one
runtime: the builder never holds it, so cloning a builder cannot share it;
recovery moves it into the rebuilt runtime, after which the old attachment
refuses with `RelationalDurablePublicationRefusal::MediumMoved`; and a forked
runtime never inherits it. Every commit is durable before it is live: the
positioned commit is encoded as its `RelationalCanonicalCommitArtifact` and
published to the medium inside the patch-position reservation, and only a
`Committed` answer lets the cutover install the new head.

- `ProvenNoEffect` is a pre-effect denial:
  `TransactionCommitError::PublicationDenied` with
  `RelationalPublicationDenial::DurablePublicationRefused`. Nothing moves and
  the position is not consumed.
- `Indeterminate` returns `TransactionCommitError::DurabilityUnresolved` with
  the position, commit id and artifact digest. Nothing is installed, and the
  runtime refuses every later publication the same way until it is reopened.

To reopen, the medium's owner hands back its checkpoint and journal tail as a
`RelationalDurableRecoverySource`; a fresh runtime built with a medium over
the same storage calls `durability().recovery_plan_from_source(..)` and then
`durability_recovery().recover(..)`. Every artifact is readmitted, the tail
must continue exactly from the checkpoint's covered position, and any damage,
gap or misfiled checkpoint refuses with a typed `RecoveryFailureClass` before
service. A recovered runtime serves the schema its recovered main branch head
records: the initial schema, or the target of the last durable schema
transition. A runtime declaring any other registry, even identical contracts
under another version label, refuses with
`RecoveryFailureClass::DeclaredSchemaMismatch` before service. Recovery moves
the medium into the rebuilt runtime only after replay, so replay never
republishes. A recovered runtime publishes its next commit at the next
position. These types are in `worth_relational::facade::durability`.

The owner catalog, branch cells, roots, and retention accounting stay
memory-resident working state derived from that history.

Plan preview uses `transaction.prepare_merged_plan(execution, runtime)` and can
return a typed storage refusal before publication. Strategy `lower_execution`
and `lower_execution_with_input` likewise take the caller's execution request.
The transaction or lowered plan owns any newly reserved client-key meanings;
preview does not publish them into the runtime dictionary. Validation judges
the same applied draft later prepared for publication. Dropping it spends its
record and text identities without publishing their meanings. A successful
commit publishes only the text carried by its canonical envelope.
`plan_bulk_mutation_batch` describes the keys currently authored in the
transaction. Prepare its merged plan first when bulk naming admission requires
interned keys; the pure description itself issues no text identities.

## Validated transitions before recovered installation

`native_checkpoint(policy)` captures one immutable image and counts its actual
MessagePack wire before allocating the final fixed byte backing. The same wire
writer emits into that backing and seals it without a payload copy. The caller
explicitly chooses System allocation or an Execution lease; allocation and live
control refusals preserve the original typed cause and payload quote through
`RelationalNativeCheckpointCaptureDenial`. Captured image, partition-alias and
serializer metadata heaps are separate from this payload admission. Imported
bytes do not acquire a charge or recovery authority.

`RelationalNativeCheckpoint` retains immutable byte backing. Clones share that
backing and preserve the selected byte region and any runtime capture-section
metadata. `from_untrusted_bytes_region` accepts `ExecutionImmutableBytes` so an enclosing
checkpoint can pass the same backing without copying its payload. Wrapping a
moved external box allocates an Arc header, not another payload buffer. Imported
native bytes remain uncharged; an admitted enclosing backing retains its whole
payload reservation through region clones until the last shared owner drops. Equality compares
selected byte values. Byte custody does not confer recovery authority; each
restore still authenticates and readmits the native checkpoint.

`restore_native_checkpoint_with_authority` issues linear recovery authority
for the verified runtime's exact branch images. Applications that must change
that image before installing their recovered world can prepare a normal native
commit and pass it, with that authority, to
`durability_recovery().commit_checkpoint_transition(...)`.

The recovery owner checks the candidate's exact recovered predecessor and uses
native publication and settlement. An acknowledged result yields the commit
receipt and recovery authority for the performed successor. Ordinary commits
do not refresh recovery authority. Relational attests native lineage; the
application still owns predecessor selection and target migration validation.

Refusal returns the original authority; a durable medium's refusal or unknown
answer is such a refusal, because nothing was installed. The transition is
durable before it is live, so a performed transition settles without deferral
and acknowledges its successor. These types are available through
`worth_relational::facade::durability`.

## Aspect-Precise Publication

Committed patches are interpreted against the installed schema before they
cross a runtime boundary. The publication facade exposes
`PublishedAuthoritativeAspectChange`, which retains:

- aspect key, opaque identity, and contract revision
- the exact entity, relation, endpoint, structural, or lifecycle binding
- whole-aspect, field, endpoint, structural, lifecycle, or opaque change kind
- an optional canonical field path
- `Exact` or explicitly declared widening precision

This is the authoritative change meaning. Relational does not allocate Signal
aspects, and downstream callers must not reinterpret raw patch fields into
their own change taxonomy.

Use `worth_relational::facade::publication` and
`worth_relational::facade::schema`. Equal labels or diagnostic digests do not
replace the typed aspect identity and binding.

#### Who consumes this

The Bridge reads committed changes through
`worth_relational::facade::change_source`. It selects a commit, has Relational
mint a change receipt, which proves the publication is consistent, and lowers
that receipt into its own envelope. Relational does not depend on the Bridge.
For Query-installed conditional operations, the flow is:

```text
Relational commit
  -> aspect-precise authoritative publication and change receipt
  -> Bridge installed correspondence
  -> Signal invalidation and decision
  -> Query consequence
```


## Release an original commit-result snapshot

The existing weak basis service provides
`release_commit_result_snapshot(&CommitResult)` for an owning caller's explicit
cleanup. It checks the immutable Native-issued result, snapshot axes and the same
actual published registration. `RelationalCommitResultSnapshotRelease::Released`
reports real removal; `AlreadyReleased` requires the original registration's
private released fact, including normal snapshot release or owner closeout.
Unknown or mismatched registrations and foreign/unavailable owners refuse.
Result clones share that fact; dropping a result does not automatically release
its snapshot. Only this original registration is released, and independently
admitted reads remain retained. The observation cell holds a weak registry
reference and adds no serialized semantic field or runtime ownership cycle.

For request-bounded cleanup, call the same basis service's
`begin_commit_result_snapshot_cleanup(Arc<CommitResult>)`, then consume the task
with `advance(&original_request, &actual_native_owner)`. `Pending` and `Refused`
return the owning task; `Complete` returns the original result Arc and release
outcome. Keep the original result and pending task in the owning Home/World
progress across stops and lost responses. Native checks owner tenure, exact
runtime/registry/cell affinity and immutable result axes without constructing an
admitted runtime copy. Source readmission never reissues an old published cell.

Removal stores the actual binding in that original cell before unlocking and
releases capacity once. Cooperative retirement then relinquishes its observation,
retention-lane obligation and basis-registry lease with nonwaiting owner steps.
Normal synchronous release and closeout join the same retirement. Completion
requires actual retirement; interrupted destructive retirement refuses. Independent
readers keep their own custody. While each step borrows the real owner, live heads
or the existing retired-root inventory retain its roots; provider destruction is
outside that path. Final task/result/owner Drop remains unbudgeted Rust destruction.
This task supplies no whole-Home close, memory-close or Native-finish guarantee.
Materialized visibility caches may retain separate observations even after the
original result and independent snapshot handles release. Native close then
truthfully refuses `ReadCustodyRetained`; cache retirement remains a separate
Native terminal-cleanup obligation.

## Retain applied aspect payloads

Entity and relation mutation application admits the Foundation output and its
existing extra allocation against the original execution request before growth.
Current rows, historical metadata and private mutation views retain that same
extra owner and its reservation. Replacement and lineage copies admit independent
output; old snapshots retain their original owners. Public materialized records
remain independent copies. Checkpoint image export borrows state while retaining
the whole extra. Moving a unique extra also moves its reservation; its small fixed
allocation bill remains conservatively charged until that moved value drops.

The generic storage handle stays one word and gains no allocation or footer.
Only entity/relation extras gain a move-only reservation. On the qualified
64-bit target, their inline sizes change from 80 to 128 and 56 to 104 bytes;
entity/relation metadata changes from 64 to 56 and 80 to 72 bytes. The concrete
existing extra Arc layout is included in the admitted bill. Metadata preserves
its six original wire fields and order, and funding is excluded from extra
serialization and relation-content hashing. Ordinary extra cloning makes an
independent copy and carries no old reservation.

Successful field-authored construction also admits its borrowed ordering scratch,
selected value copies, portable buffers and readmitted patch against the same
original request. Endpoint augmentation moves the original operations rather
than cloning them again; the relation-aspect prefix copy is admitted before it
is made. The move-only private `MutationAspectPatch` retains the bill until its
actual original patch is destroyed in publication filtering, including failure.
Dead construction/admission scratch refunds before applied Extra growth. Checked
Foundation `clone_allocation_bytes()` descriptions count actual copied lengths;
retained-capacity reports measure existing output, not future copies.

On the qualified target the Foundation patch is 48 bytes, its private funded
owner is 96 bytes, and `RecordMutation` is 200 bytes. This adds no Arc or generic
storage footer. Explicit ordinary-unfunded constructors remain available only
for the existing independent portable/deletion data routes; they qualify no
funding. No wire format or scalar/struct winner and validation order changes.

Request-driven record birth, update, retirement, reuse, rematerialization and
lineage replacement also admit each historical metadata allocation before editing
its history. The same existing metadata Arc retains its own small fixed bill;
its Extra payload keeps its independent reservation. Retiring and new metadata
are prepared together before mutation, so quota refusal leaves prior history
unchanged. A unique moved metadata value conservatively retains its fixed bill
until Drop. On the qualified 64-bit target, entity/relation metadata is 104/120
bytes inline and its actual Arc allocation is 120/136 bytes. This is a localized
48-byte footer per metadata value, with no change to generic storage handles.
Metadata's six wire fields and Debug meaning exclude funding.

Ordinary image construction and maintenance cloning remain independently
unfunded. Controlled metadata replacement creates a fresh existing-type owner
even when the previous metadata was unique; no additional wrapper Arc is added.
The controlled node/page and scalar-directory guarantees below cover separate
backing allocations. This does not qualify whole-arena or Complete funding.

These are successful-path backing guarantees. Existing semantic-error locator,
path, key and formatted-detail copies, and returned-error Clone/Serde, retain
their ordinary allocation contract. Other portable-input copies, supplemental
and canonical projection copies, remaining replay construction, ordinary radix
construction, Complete capture and selected-backing handoff remain separate obligations.

## Retain controlled node and page backing

Request-driven column, AVL and scalar slot-directory writes use the original
mutation request. Each actual node owns its own reservation; a column leaf also
owns its fixed page buffer. Temporary dense-build directories release their
separate bills after destruction. Unique fixed-capacity edits need no replacement
allocation. Shared edits admit each replacement before shallow copying; partial
construction retains its admitted nodes until the unpublished workspace drops.
Published roots remain readable after refusal.

Scalars, shallow nested column headers, adjacency headers and DigestIndex map
headers are inline. Independently shared metadata, Extra and Digest child nodes
keep explicit existing allocation handles. Parent bills exclude those independent
allocations. Where a payload is funded, its escaped handles retain that owner's
independent bill. Allocation observation reports actual node and payload identities
without inventing scalar or inline header identities. Generic storage handle
layout and tree wire meaning remain unchanged.

On the qualified 64-bit target, an inline kind-bucket node is 112 bytes, replacing
a 112-byte node plus a 24-byte header Arc. An inline adjacency node is 168 bytes,
replacing a 112-byte node plus an 80-byte header Arc. Each removes 24 bytes of
backing. Child nodes retain their actual identities and reservations; no separate
header allocation or payload bill remains. This changes no selective publication
algorithm and adds no adjacency or bitset funding guarantee.

DigestIndex map nodes grow from 112 to 120 bytes while each distinct 32-byte
header Arc disappears. This trades fewer allocations for larger copied nodes:
the measured dense 64-entry owner falls from 15,360 to 13,824 bytes. Retaining
64 append roots grows usage from 51,312 to 52,344 bytes; retaining 129 hot-history
roots grows it from 160,864 to 161,376. These are measured examples, not universal
savings or bounds.
The actual Patricia and HistoryLastValue child owners stay shared and unchanged.
Their funding and general nonshallow defaults remain separate obligations.

Borrowed map walks keep their bounded path stack inline and report zero heap
scratch while preserving original work and stop polling. On the qualified
64-bit target each iterator occupies 1,040 bytes, including when empty; a pair
occupies 2,080 bytes before adapters and compiler temporaries. This exchanges
heap scratch for stack space. Owned IntoIterator and mutable iteration keep
their existing allocation contracts. No whole-operation funding is implied.

This qualifies controlled column/map nodes, fixed pages and scalar-directory
writes. Ordinary image and maintenance construction, legacy adjacency/bitset
backing, revision-map payloads, independently constructed Digest/default payloads
and Complete capture retain separate allocation obligations.

## Retire an immutable Memory runtime

Platform lifecycle owners consume their actual `RelationalRuntimeHandle` with
`into_memory_close`. Shared representations, aliased owners, admitted read copies
and actual medium/cold/transition-source affinity refuse without losing the handle.
Retain `RelationalMemoryClose` outside disposable callbacks and call its
`advance(&original_request)` through `&mut` progress. Reversible preflight refuses
retained candidates, accepted settlements and explicit snapshot/replay custody;
repair them before retrying. `into_refused_handle` is an explicitly unbudgeted
preseal restoration around the same runtime and original handle metadata.

After sealing, no serving restoration exists. Each nonwaiting step retires one
actual cached Exact/Historical observation or head obligation. Independent reads
keep their own custody and cause a truthful refusal until released. An existing
original-result cleanup task can advance through the phase's narrow forwarding
method. Completion retires business obligations, retaining the authentic private
root/cache allocations and their funding. `into_retired_memory` yields only an
opaque nonserving allocation whose `reclaim` is explicitly unbudgeted.

Incomplete phase Drop is also unbudgeted owner-loss abandonment and proves no
successful close. Concrete Home progress must retain the phase across request
stops and lost responses. This Native Memory contract grants no durable/cold Home
close or arbitrary destructor deadline. Dense cache/head tables retain their
existing allocation semantics; metadata footprint and worst-case O(1) cost are
not claimed, and close never allocates/reserves/rehashes/shrinks those tables.


## Read an original protected source cut

`RelationalDurableReadProtection::capture` binds the original source allocation
and the provider's frozen canonical/transition bounds. Its owned read methods
reject foreign sources before callbacks and never fall back to legacy `Vec`
reads. Unknown providers return `UnsupportedProvider`. The two original test
media copy/fund immutable snapshots; inline checkpoint images expose members
with zero journal bounds and do not manufacture an original journal prefix.

`RelationalDurableRecoverySource::from_protected_source` feeds exact original
owned artifacts into the existing recovery admission. Canonical admission
retains the funded original raw buffer. Transport/plan collections and decoded
transition heaps retain their ordinary allocation contract. This capability
confers no Complete checkpoint, CHK18 coverage, schema reconstruction, selected
residency transfer or acknowledgement authority.


## Retain original Memory publication frames

A new Memory publication admits and issues its original canonical artifact v4
frame under the caller's original request after reserving the position and
before publishing any head or position. The existing residency cell retains
that immutable frame. Artifact export verifies and shares the original bytes;
it does not encode a replacement. This adds count/hash and emission passes plus
frame verification to Memory publication while the position reservation is held.
The exact byte buffer is admitted before growth; failure moves no head or position.

Preparation keeps the legacy authoritative tuple's exact scalar length/digest
using its existing serializer, without retaining tuple bytes. The positioned
artifact digest remains separate. There is one retained frame allocation rather
than both a frame and tuple payload. Ordinary Journal and legacy resident export
behavior remains; strict CHK18 capture requires an actual original frame.

## Canonical checkpoint members

CHK18 requires one ordered canonical member for every indexed position, including
positions covered by an earlier checkpoint. Each member is the original complete
artifact v4 frame. Its member SHA covers the whole frame; the index retains
the separate positioned payload digest. Capture verifies original Memory backing,
protected Journal reads, or the installed prior selected read. It cannot reconstruct
a missing original from an image or resident body. Earlier CHK17 selections refuse.
Embedded checkpoints include these same original byte strings; external checkpoints
publish them as singleton members alongside the manifest.

`durability().canonical_commit_artifacts(&request)` takes the original execution
request so selected reads enforce its limits. Skipping with `nth` or
`after_position` remains lazy. Memory export shares its original byte owner.

After actual physical selection, `checkpoint.acknowledge_selected(&request, context)`
progresses the same captured cells before releasing component holds. A stop or Busy
retains the same pending capture and cursor. The new directory data pointer may be
readable while old read/member retirement remains pending; that placement does not
complete acknowledgement, advance the accepted index-definition frontier or
advance completion cadence. Historical, component, associated-effect and head
demands use the selected original member source, without
covered canonical journal fallback. Ordinary uncovered tail replay keeps its existing
contract.

Capture now visits the full indexed canonical prefix and original artifact bytes;
metadata ordering and validation cost O(C log C), plus actual artifact/image work.
New roster, mixed member list, retained per-entry owner, selected-read and ACK storage
use original-request admission and shared intrinsic custody. Existing schema/image
and other untouched manifest heaps remain outside that funding claim. Fixed
weak-retained control metadata keeps its existing ordinary allocation contract.
This ordinary checkpoint path does not issue Complete, reconstruct every historical schema/root,
or provide service-free reconstruction or a complete-checkpoint preparation port.

## Admit the original fresh Home source

The closed Store Home route uses
`durability_recovery().admit_fresh_home_source(&request, source_plan)` only
while private physical custody proves genuine newness. Native validates the
actual protected empty source against its attached medium and retains the SAME
runtime, initial root and original birth/installation schema data. The returned
`FreshRelationalSourceAdmission` is fixed-size owner data with existing schema
shares and a weak reference to that actual root; it retains no second runtime
or operational root archive.

Query validates its same-runtime initial installation against this receipt,
then registers its indexes. World validates the receipt before attaching its
fresh source. These are named, restricted owner integrations, not application
construction APIs. A foreign source/runtime, moved initial root, incompatible
installation or stopped original request refuses explicitly.

`RecoveredRelationalRuntimeAuthority` continues to mean genuine source-driven
runtime reconstruction. An existing empty Home still uses recovery, even with
no selected checkpoint or journal tail. Fresh admission never creates recovered
authority or restore-work evidence. It does not supply missing historical
original inputs on reopen, admit a selected Complete Home source, or retire
Native/provider service graphs.
