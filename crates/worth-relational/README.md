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
v3 and native checkpoint CHK15 retain the field through journal and embedded or
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
checkpoint CHK13/CHK14 refuse cleanly. These data fields establish neither
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
with a descriptor. Foreign owners refuse.

## Readmit an associated Native effect

`durability_authority().discover_associated_effects(execution, &association)`
selects actual catalog or performed-route subjects from their required discovery
metadata. Unrelated cold bodies remain cold. Each returned subject records the
exact runtime and Journal index with weak identities for the actual source and
cell; the live Native owner retains their custody. Associated Memory refuses.
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

Current exact-image admission covers source-produced Empty, checkpoint-current
and bounded-tail images still available from the Native owner. Missing historical
checkpoint root/reference images refuse. Full cold historical predecessor
materialization and the concrete Store/World recovered-adoption journey remain
separate unfinished work; this Native phase alone grants no product movement.

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

