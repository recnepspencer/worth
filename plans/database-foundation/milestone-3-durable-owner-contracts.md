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
use that SAME snapshot for delta, descriptive index-touch projection and
authored-index-write preparation. Touch counting, building, extent and revision
merging must not independently reread a newer live catalog. Capture before
these projections so replay of an older prefix preserves every authoritative
payload field, including touched later-definition cases.
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

## Coupled World checkpoint and automatic maintenance (D12)

This is the governing destination for the next checkpoint slice, not an
implementation or completion claim. It replaces the Native-only refusal when
World journals are nonempty with one actual multi-owner Home checkpoint.
Full original-result retry (D9) and bounded cold teardown (D11) remain separate
obligations. Neither may be inferred from a selected checkpoint.

The decisive court crosses a real process restart with a selected normalized
World image AND nonempty tails. Include Native ahead of the World business
basis, durably opened unpublished custody, and a business error after accepted
work. Interrupt selection and the interval between owner acknowledgements.
The original business result or denial must survive maintenance failure; a
later usable request must rejoin the same accepted checkpoint attempt. Missing
members, corrupt owner meaning and a mismatched Native association must refuse,
not disappear behind the selected image. Hold cold readers and the retired
allocation through actual close; checkpoint selection alone proves no close.

### World image and admitted historical meaning

World owns a normalized image, its versioned codec, validation and coverage.
Preserve the exact Home/owner/program binding; three journal positions and
digests; branch origins and bootstrap/component descriptions; every durable
open's complete association, predecessor, commit, custody and program revision;
all occurrence/history/head facts, generations, Native commit/position/digest,
publishing incarnation and original World occurrence coordinates; all terminal
release/continuation dispositions and actual Native release evidence; and all
identity maxima. Enumerate these fields against the actual associated-effect
reconciler before source implementation. Live catalog reclamation does not
authorize omission of historical opens, associations or dispositions.
Preserve an original Owner record only if one exists: production bootstrap
currently publishes Branch directly. Do not synthesize an Owner row or infer
journal coverage from the number of normalized branch origins. Preserve current
multi-branch/Continued readmission refusals until their owning support lands;
capturing their meaning does not admit unsupported service.

Capture folds the SAME selected World image and only its uncovered tails into
this state. Without a selected World image, initial capture validates the
original streams once. This is not concatenated journal frames, replay
instructions or a new live mirror/registry. Readmission validates normalized
tables and their complete graph, then uses actual Native association admission
and existing World materialization. No business handler or canonical mutation
is replayed. Image processing is O(all retained authoritative state plus the
uncovered tails); it is not an O(tail) or constant-heap claim. Authoritative
history remains retained until Milestone 4 permits reclamation.

Replace the independently paired occurrence artifact and raw source map with
owner-private `AdmittedWorldOccurrence`: complete typed occurrence meaning and
an inline admitted location correlating owner, attempt, commit, position and
digest. Journal admission constructs it after actual byte/SHA/frame/contiguity
validation. Checkpoint admission constructs it only after validating the actual
selected World section, Home/source affinity, protected membership, image
integrity, coverage bounds and graph. Retain checkpoint read protection through
validation and materialization. `prepare_readmitted_occurrence` consumes that
correlated fact; the installed record retains its location and SAME genuine
Native publication custody, immutable identity and history protections.

Remove `occurrence_sources` and the unused original-byte backing in materialized
readmitted records. Do not reencode parsed meaning into purported original
journal bytes, reread covered history, or construct `DurablyCommitted`. Image
coordinates are checkpoint-admitted historical facts, not a claim that reopen
read the original payload or that the selection receipt created the occurrence.
`WorldReadmittedPublication` keeps its existing historical accessors and cannot
become `PublicationEvidence::Live` or a performed delivery.

World checkpoint codec version 1, Home manifest version 2 and Native checkpoint
CHK18 are the clean development-format cutover. Selected unsupported versions
refuse explicitly; missing World coverage cannot mean empty state. Native
canonical artifact v4 and the Relational WSRCKPT1 version-3 section remain
unchanged. Store transports
opaque owner sections; it does not parse World or Native semantic payloads.
Keep the existing 1 GiB combined Home-manifest ceiling and 64 KiB individual
World journal-frame ceiling. Check lengths/counts/arithmetic against remaining
input and the original request before allocation. Fund retained maps/buffers
and poll during encoding/decoding. Limit refusal preserves prior selection and
all journals; it never prunes history to make a checkpoint fit.

### Native historical-root reconstruction prerequisite

The current Native capture selects current cells and actual external component
holds. Its commit catalog stores root descriptions, basis indexes are Weak, and
recovered roots become Weak after readmission. Legal process-local reclamation
can therefore leave a World historical component with neither a live Native
root nor a root image in the selected Native checkpoint. The retained-body
export repair alone does not preserve that root. Copying all World descriptions
into a checkpoint would still lose executable history.

The coupled Home path requires a distinct Native complete-source-history
capture operation. Existing standalone Native capture keeps its contract and
ordinary cost. Do not introduce a history-policy mode enum, persistent pin
registry, second serving runtime, or a second recovery/selection authority.

Under the existing Native capture gate, first freeze the ORIGINAL
`CapturedCheckpointBasis`, canonical/transition coverage, source affinities and
expected source artifact metadata. Retain a protected immutable source prefix
for that same cut, then release guards before reconstruction. Use actual
canonical and transition records from position one through that cut when prior
checkpoint roots are insufficient. Admit source framing, continuity, schemas
and complete authoritative meaning through existing owner recovery admission.
Legacy `encode_authoritative_payload` tuple parity omits association, semantic
result, symbol texts and codec retirements; it cannot substitute for the full
Artifact projection agreement required here.

Use existing Native recovery/rebuild/root-inventory mechanics in an owner-local
unshared checkpoint reconstruction workspace with NO durable medium and NO
service/publication exposure. Never call `recover_with_authority`, detach or
move the actual Home medium, install reconstructed live branch cells, or mint
external observations from caller descriptions. Preserve the source-produced
canonical/transition order. Extract actual roots, schema ownership and canonical
observations while `RecoveredRootInventory` still owns Reconstructing roots,
before its Weak downgrade. Merge those images into the ORIGINAL frozen captured
basis after validating full source/receipt/position/root/schema agreement.
Process-local Arc pointer equality cannot decide equivalence between two
authentic reconstructions. Later live commits cannot move the captured cut.

Retain ALL actual source-produced root observations through that Native cut,
including Empty origins. Only preserving references found in the earlier World
image is insufficient: World can publish between its cut and Native capture,
and those legitimate occurrences arrive through uncovered World tails. No
prior-checkpoint-only fallback may silently omit a requested historical root.
Actual reconstructed roots and region/member protection, rather than copied
descriptors, survive transfer into the prepared Native section and shared Home
attempt until selection/acknowledgement completes.

The invoked reconstruction must NOT take a detached whole-history rollback
snapshot for each commit. Current `apply_authoritative_commit_artifacts` does
that, rebuilding accumulated routes and cloning catalog/graph/envelope/index
maps; merely funding those clones leaves quadratic work for constant-size
metadata commits. Validate/prepare each replacement before its bounded owner
installation. A later failure discards the private reconstruction workspace;
it never rolls back or alters the serving runtime. Reuse owner validation and
installation mechanics without importing their unnecessary whole-prefix
rollback into this isolated path. Preserve any distinct ordinary recovery
contract that genuinely requires it.

Reconstruction cost is O(n log n plus admitted source decoding/replay work and
retained root/image construction), where n is the prefix's metadata entries.
Growth in actual row history or encoded images remains explicit, not hidden in
that metadata bound. Repeated whole-history copies for constant-size entries
must be absent. This grants no ordinary-serving replay permission or constant-
work/whole-heap claim. The original journals remain available until M4 retention
authorizes reclamation. Image-size refusal preserves history; no incomplete-
history checkpoint may be selected to hide it.

Fund the checkpoint reconstruction workspace from the SAME original request.
Existing reconstruction deep-clones pending tails, creates unfunded ID sets,
root/descriptor/route collections, and stages replay intents with
`SystemAllocation`. Its current allocation-observation inventory serializes
some fields and saturates sums; it is not a preallocation quote. Repair the
invoked path: avoid unnecessary clones, fund source buffers and collection
growth before allocation, use checked owner quotes for retained
intent/descriptor/schema backing, stage with the original allocation policy,
and carry genuine backing grants with the actual captured roots. Reuse existing
Execution ordered/array backing where it fits; add no general allocator or byte
ledger. Raw `rmp_serde::from_slice` does not establish nested allocation funding:
the invoked decode admission must have a checked funded path too. The 1 GiB
wire/read limit is not a reconstruction heap budget. Refusal or stop before Home
publication preserves prior selection and the actual live runtime.

Decoded body custody resides INSIDE the existing Arc-owned
`CanonicalCommitEnvelope`; semantic custody resides INSIDE the existing
`RelationalCommitSemanticResult`, because `semantic_result_arc()` escapes
independently. Remove their infallible deep Clone implementations: admitted
owners share/move the actual Arcs, and any necessary draft copying has explicit
original-policy admission. Do not add a second body Arc to every ordinary
commit or share one ticket across separate deep copies. Grants do not enter
wire meaning or semantic equality. Moving `ReadmittedCanonicalCommit` into a
positioned value must retain these grants; its discarded sidecar is not their
owner. Document the added inline ownership footprint.

### Selected canonical backing without covered-journal replay

Actual root images alone are insufficient. Directory CHK17 omits complete
canonical envelopes; its index has receipt/position/root/digest/schema and
association-discovery metadata, but not full semantic results. Current cold head,
historical component and associated-effect admission reread covered canonical
journals. Do not describe that as bounded-tail recovery. Nor may the index be
expanded into fabricated original canonical meaning.

CHK18 adds content-addressed canonical-history members and their Native-owned
position/page directory to the existing external-member checkpoint format.
Preserve the exact authenticated canonical artifact bytes admitted from the
captured source, including all full authoritative fields and their original
positions/digests. These members join the same semantic-member union, actual
read protection, placement, selection and acknowledgement as region/history/
symbol/lineage/index members. Store remains an opaque member transporter.
Do not put historical payloads into the eager manifest, select fully Embedded
placement for Directory Homes, or trigger the old decoder's inline-expansion
branch by adding inline envelopes alongside external members.

The first format uses one original artifact per member and one directory entry
`{ position: PatchStreamPosition, member: RelationalDurableMemberLocator }`.
Entries are strictly ordered and correspond one-to-one with the commit index.
Retain the existing bounded individual artifact size; add no packing policy or
smaller limit that rejects a lawful large artifact. Do not reuse symbol-page
sentinel assumptions for canonical positions. The member locator hashes the
ORIGINAL FULL artifact bytes; it is distinct from the index's artifact-payload
digest. Validate both at their existing owners.

The Native directory validates the new position/page inventory, complete
coverage, uniqueness and agreement with its actual commit index. Construct
selected-checkpoint canonical backing only from that validated directory and
its genuine source/member custody. Existing standalone capture may retain its
declared partial history contract; the coupled operation must issue and retain
complete canonical backing for every covered commit, and coupled Home opening
must require it. A missing covered member or incomplete inventory refuses; it
never falls back to a canonical journal read or to index-only meaning.

Extend the actual envelope-residency source placement with selected-checkpoint
backing. Preserve its original source affinity and genuine member obligations;
do not wrap a physical source in another issuer identity or treat the bytes as
authority-free Memory backing. One existing full artifact admission validates
both journal-backed tail bytes and selected-member bytes against the original
receipt, position, digest, schema, roots, association and complete authoritative
projection. Covered head admission, historical image materialization, component
admission and associated-effect admission all consume the actual selected
backing retained by their catalog residency. Only uncovered tails use journal
backing. Strengthen component admission's legacy tuple equality to the same full
Artifact predicate. No live performed capability is reconstructed.

New Directory canonical members remain demand-read native data. World may
require all historical semantic results during reconciliation, so that work is
proportional to the selected owner members/image plus uncovered tails, not
constant-size metadata alone. Reuse actual selected backing across repeated
admission rather than rereading journals or duplicating payloads. Explicitly
fund page/index decode, temporary/retained bytes and escaped backing grants with
the original request. Fully Embedded checkpoint support remains distinct and
keeps its own cost contract; no new ownership-policy mode enum is introduced.

Complete-operation acknowledgement also installs validated selected-member
backing into its ORIGINAL frozen covered residency cells. Preserve the actual
entry, original source Arc and unchanged canonical meaning/private completeness;
this is placement transfer, not replay or a canonical-envelope replacement.
Later entries remain untouched. The existing prepared operation retains a
request-polled per-entry acknowledgement cursor and actual selected directory.
Busy/stop retains its remaining handoff in the SAME Home attempt. Coverage is
not fully acknowledged for cadence until that required handoff completes.
Partial transfer never recaptures, republishes, moves a head or acknowledges
uncaptured definitions. A completed operation can then reuse selected backing
for later capture without covered-journal demand. Test interruption mid-transfer
and source/member protection through final acknowledgement.

The decisive Native prerequisite court legally releases observations and
reclaims old live roots/history metadata BEFORE coupled capture. A fresh
process must then recover historical values and World publication meaning from
the selected actual root images plus nonempty tails, without reading covered
journal history, replaying handlers, publishing canonical work, moving a current
head, or manufacturing receipt authority. Include a World publication between
the World and Native cuts. Exact root/image/member observations must convict
descriptor-only or earlier-World-reference-only implementations. Independent
Native boundary review must close reconstruction/source/funding/transfer
contracts before this prerequisite receives Source GO.
Use an actual owner read observer/refusal to forbid covered canonical journal
reads throughout checkpoint-based World reconciliation, rather than merely
asserting that a checkpoint exists. Cover missing/corrupt canonical members,
incomplete/duplicate page coverage and CHK17 refusal. A metadata-only prefix
growth case must expose whole-history-per-commit copies even when rows and
retained image sizes remain small. Preserve differing historical schemas,
transition order, Empty origins and exact funding/acknowledgement custody.

### Source cut, one selection and acknowledgement

Use a private RAII capture reservation in the existing
`WorldDurableMediumBinding.gates`. Nonwaiting admission requires all three
flights to have no active caller, retained subject or barrier, plus settled
actual adapter task slots. Publication checks that SAME reservation under the
same gates. While reserved, acquire an immutable protected cut of the actual
three heads/digests and their source obligations. Release the reservation after
that cut; normalize, encode and perform I/O outside gate guards. Unwind releases
the reservation. Do not require unrelated active operations to be zero: the
image covers durable meaning, including opened unpublished attempts, not a
speculative live CAS.

Capture World first, retaining actual component/source observations, then
Native. Native may be ahead; current Native head is never a substitute for
World coverage. Extend the existing `HomeCheckpointAttempt` with both prepared
sections, their original funding/affinity and union of physical memberships.
Publish one manifest under `home.checkpoint`. Both acknowledgements consume the
SAME actual physical selection receipt and enforce owner coverage monotonicity;
Native also preserves transition/catalog monotonicity. An uncertain selection
or interrupted acknowledgement retains the exact attempt and remaining work.
Do not recapture, republish, move a head or change disposition to retry it.

Opening selects once, passes each opaque section to its owner, and reads every
tail from its actual coverage plus one. The complete image-plus-tail meaning
feeds existing Native association reconciliation before service. Missing opens,
orphan effects, conflicting lineage, duplicate association and missing Native
evidence retain their refusals and genuine acquired custody.

Close invokes this SAME checkpoint after stopping admission/settling actual
publications while World is still alive, before World Drop. A settled failure
is reported and does not veto otherwise lawful close. Unknown selection or
unfinished acknowledgement retains actual recovery custody. No checkpoint
result upgrades incomplete Native/physical teardown to Closed. The capture port
must admit this stopped-but-owned close phase without reopening ordinary
service or requiring an Open-only service gate.

### Finite cadence and unavoidable advancement handoff

Host checkpoint policy defaults to 4,096 uncovered entries OR 64 MiB of owner
payload in ANY participating journal: Native canonical/transitions and World
branches/custody/occurrences. Expose this policy in Host Home request limits;
count and byte overrides are finite, nonzero, checked values. Memory-only has
no Directory checkpoint port. Use only owner-issued acknowledged coverage;
unpublished/unacknowledged captures never advance the cadence baseline.

For each journal, inspect its actual metadata head and refuse a baseline ahead
of it. A count reaching the selected threshold is immediately due. Otherwise
page/fund at most that uncovered suffix's headers, summing checked payload bytes
and stopping at the byte threshold. Default worst-case admission examines five
heads and at most 4,095 suffix headers per journal, independent of older
history and owner payload size. Charge/poll the SAME original request.

Add a narrowly owned request-funded scheduling-metadata operation to Store's
ordered journal. Existing `head()` reads/hashes the whole latest payload and
`range_descending()` has no request/funding parameter; they do not establish this
cost. Header-only metadata checks framing, journal/position/spine/length under
actual read protection. It is scheduling information, NOT full-payload SHA,
checkpoint coverage or recovery authority. The selected record digest covers
the entire record, and the header has no independent equivalent digest. Actual
capture/recovery still validates payloads; no cadence-only wire version change
or cumulative counter is introduced.

Every installed-opening advancement route with a Directory port must use one
private completion orchestrator around
`QueryRequestExecution::run_admitted_advancement`:

```text
original_accounted_or_native_completion_scope(|| {
    business_result = run_admitted_advancement(body)
    maintenance_attempt_using_the_same_original_request()
    return business_result
})
```

Run maintenance after EVERY returned business Result, including Err, while the
SAME execution owner, lease, placement and opening custody remain alive. The
outer original completion scope MUST begin before the inner business run and
retain its actual meter until maintenance ends. A lease alone does not retain
remaining work after a scope closes: restarting the full policy is forbidden.
Existing nested meter charging carries consumed work, remaining ceiling and
latched stops; do not replace it with a copied remainder or counter.

For accounted placement, use the existing value-preserving
`ExecutionRequest::run`/`ExecutionWorkCeiling::run` contract. Its callback value
is the complete original business Result, returned whenever the callback ran,
even if later maintenance causes a final scope stop. Maintenance lies after the
INNER business result handoff, but inside the outer accounted completion scope.
Return the exact business R/denial; an outer refusal before callback entry still
returns its normal typed advancement denial.

For worker-only placement, add the narrow Execution-owned
`ExecutionRequest::run_native_completion` counterpart to `run_native`, with the
same original request/backing, native scope, callback lease and physical-only
report. Its explicit effectful custody contract returns the callback value
whenever the callback returned, alongside the actual physical report; admission
or stop before invocation returns the existing typed denial. Keep the value
inside the Execution owner, using its existing scope/result-carriage mechanics.
Do not drain a Query-side result slot or force an accounted scope on a native
request. Existing `run_native` remains unchanged: its final poll may discard a
value, so it is insufficient for this completion boundary. `prepare_native` is
pre-effect-only and must not publish checkpoints. Nested stop ownership remains
latched; preserving an effectful callback result does not clear stops or grant
fresh allowance. Test both native posture and accounted-parent inheritance.
An original request already stopped before entry must not run an effectful
callback, including when its invoking meter has a latched nested stop but could
admit another zero-work child. Preserve that original control fact through
actual scope admission. Foreign/sibling lineage refusal keeps its existing
precedence over ambient interruption; do not return a pre-entry ambient consult
error before checking actual lineage admission. A stop arising after the
callback returned still preserves its value as specified above.

Use Query's existing catch/resume-unwind pattern around the whole completion;
Execution containment must not convert business or maintenance panic into a
different ordinary result. Preserve ordinary Memory/bootstrap paths without
the extra Directory completion scope. Do not mint another allowance, clock,
deadline or cancellation token. A stopped original request produces an honest
stopped maintenance attempt with unchanged coverage. An unwind preserves
existing semantics/barriers and performs no acknowledgement.

Store issues an opaque automatic-checkpoint port from the actual Directory
Home. It retains the same checkpoint owner, Serving and Native checkpoint
capability plus World's capture port. Host implements the narrow Query adapter;
both fresh and existing attachment REQUIRE it before QueryReadmitted. The
clone-carried product runtime retains it, so `home.runtime()` and existing
advancement entry points cannot bypass cadence. The six public installed entry
doors currently share four actual execution sites in `installed_opening.rs`:
direct Host, funded Host/application, direct application, and product opening.
All four must enter the single completion orchestrator. Enumerate those routes
in the implementation plan. No optional wrapper, global registry, daemon or
scheduler is permitted. Document retained handle/allocation costs and release
them in existing `drop(query)` before Native teardown; external runtime clones
retain real custody and may defer close.

Inspection is request-funded typed observation derived from actual selected
coverage, current scheduling metadata/due decision and the existing pending
attempt. Add no last-error history, report ledger or sequence counter. A failed
pre-effect attempt remains due; a later usable request re-evaluates the backlog.
Permanently stopped/insufficient requests promise no eventual success. Explicit
checkpoint and close return/report their actual detailed outcomes.

### Destination, proof and continuing documentation

Extend existing owners; do not create placeholder bodies:

```text
worth-runtime-world/src/durability/checkpoint/ [create owner boundary]
  image.rs, codec/, capture.rs, readmission.rs [distinct owner responsibilities]
worth-runtime-world/src/lifecycle/ports/checkpoint.rs [create actual capture port]
worth-runtime-world/src/history/publication/readmission.rs [revise admitted location]
worth-store/src/physical_runtime/ordered_journal/metadata.rs [create scheduling read]
worth-store-application/src/checkpoint/ [extend existing single attempt]
  cadence.rs, maintenance.rs, inspection.rs [create bounded policy/port/observation]
worth-store-application/src/world_section/checkpoint/ [create opaque owner adapter]
worth-store-relational/src/section/checkpoint_port.rs [create actual section capability]
worth-query-host/src/application_home/checkpoint.rs [create closed composition adapter]
worth-query-execution/.../request_execution/advancement/installed_completion.rs
  [create mandatory installed-operation completion]
worth-execution/src/authority/request/native_completion.rs
  [create native effectful completion; preserve existing scope/report owner]
worth-relational/src/durability/authority/checkpoint_history/
  source.rs, reconstruction.rs, funding.rs, transfer.rs
  [create complete-source-history operation at existing Native capture owner]
worth-relational/src/durability/authority/checkpoint_capture.rs
  [revise original frozen capture handoff; standalone contract preserved]
worth-relational/src/durability/log/native_checkpoint_codec/canonical_history/
  capture.rs, validation.rs, readmission.rs
  [create selected canonical members; encoding/decoding remain at codec owner]
worth-relational/src/history/commit/envelope_residency/
  [extend actual source placement and reuse full artifact admission]
```

World alone owns image meaning/admission. Store alone owns journal metadata,
lease/membership and physical selection. Query owns advancement completion;
Host composes the owners without exporting raw source/runtime getters. Keep
exact caller seals and the D1 dependency direction. No competing selector,
raw-byte reconstructed authority, shadow catalog or generic maintenance engine.

Real filesystem proofs cover manual, count, byte and close triggers of the same
operation; normalized image plus tails across process restart; continued work;
Native-ahead-World and unpublished custody; missing/corrupt members; failed and
unknown selection; interrupted second acknowledgement; original-request stop;
and a business failure after accepted effects. Include a request whose business
work leaves insufficient work for maintenance: no second full allowance may
reach checkpoint selection, and the original result survives. Exercise native
worker-only completion under cancellation after the business result and a
genuine accounted ancestor; no fabricated accounting or swallowed parent stop.
Assert preserved exact business outcomes, unchanged baseline on failure,
same-attempt rejoin, no handler replay,
and bounded suffix metadata work through ordinary `home.runtime()`. Owner tests
validate every image field/graph/limit and genuine admitted-source correlation.
Fresh source QA and boundary/context/format/cap checks precede integration.

Revise the existing Host README and `docs/build-an-application.md` open/close/
checkpoint guidance with the actual limits, explicit checkpoint/inspection,
independent maintenance failures, retained custody, format refusals and restart
cost. Revise the World and Store owner READMEs for their exact source/coverage
contracts and `docs/capability-index.md` for the real public capability. Examples
must use Decl/Host and the actual mandatory path; selected Native-only evidence
must not be described as multi-owner checkpoint completion.

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
