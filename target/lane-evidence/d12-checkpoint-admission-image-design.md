# Native checkpoint admission image — design, not implementation

Design baseline: integration commit `838d28742767a8193391f14d0b46f5798ce0fff0`.
Author branch remains `655b27b95e`; no integration merge, Cargo invocation,
build or test belongs to this turn. Grants 11, 11a and 11b in
`orchestration/m3/prompts/p3_reopen_grants.md` governs this design, together with
milestone-3 Format stability, D12, D13 and D17. This replaces the earlier
bounded-history review's missing authority contract; it does not invalidate its
finding that today's envelope hash cannot authenticate a compact projection.

## First journey and actual boundary

Capture a coupled filesystem Home checkpoint from the real original Native and
World owners; select it through the existing one `home.checkpoint` decision;
kill, reopen and admit the retained World history using real Native roots,
schema authorities and retention leases without reading covered canonical
bodies. First feedback is the unchanged
`automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom`: at most
40 canonical bodies, with its counter and bound unchanged. No cache or counter
substitution is part of this journey.

Prerequisites, in order: establish the closed producer/proof and selected-read
contracts; capture the authoritative projected meanings; authenticate and
readmit them; replace the root's envelope-only invariant; cut all covered-image
admission consumers over together. Then qualify interruptions, refusals, a
second Complete, original-frame damage, and D13 receipt restoration. A partially
migrated root representation is not an implementation checkpoint.

Merged facts checked with `git show 838d287427:<path>`:

* `branch/root.rs::RelationalCommittedBranchRoot` owns a full envelope;
  `root_visibility.rs` hashes its whole authoritative payload.
* `associated_effects/readmission.rs`, `recovered_publication/subject.rs`,
  `recorded_component_readmission.rs` and
  `runtime_rebuild/root_inventory/checkpoint_images.rs` demand that envelope.
* `checkpoint_complete/prepare.rs` already verifies the original prefix and
  reconstructs its roots under the original request. It is the producer seam.
* Store `complete_codec.rs` is exact WSRCSRC1/v1. Native
  `source_initial_inputs/complete_source.rs` is exact WRTHRCS1/v1. Neither has
  an extension list. Home manifest2 has two opaque owner sections.
* World `history/catalog/memory_original.rs` retains committed root-free memory
  publication records independently of reclaimable live history.
* Query `provider/idempotency/snapshot_resolution/source_receipt.rs` needs
  original semantic results, created binding/outbox/causality rows, World
  coordinates, attempts and both Native component observations.

Dependency premises: Native's merged Cargo.toml already depends on proof,
foundational and execution. World already depends on Native and those crates.
Native must gain no World or Query dependency. No Cargo feature is needed.

Interim naming premise: `git grep -n WorthQueryInterimCheckpointRestore
838d287427 -- workspaces/worth-query` (also58564ccf9f) returns no matches;
`git ls-tree -r --name-only 838d287427 -- tools/boundary-check/config` has no
`interim-checkpoint-callers.txt`. These are the spec's destination door/list,
not surfaces to pretend already landed. The real pinned restore is
`primary_graph/bootstrap/preparation.rs:133-158`, fed by
application_checkpoint.rs/DecodedApplicationCheckpoint. Implement the image
there and coordinate integration's D13 door/caller-list consolidation; do not
add a parallel restore door or infer a new allowed caller from this plan.

## Authority contract and placement

The following are proposed Rust signatures, not claimed existing APIs. Names
without bodies specify interfaces; all payload fields and constructors are
private unless a signature explicitly exports an operation. Types with owned
read protection, weak runtime identity, allocation reservations or retirement
state live in the owning runtime. Pure cross-owner coordinates live in
foundational. Legal progression wrappers live in proof.

```rust
// worth-proof: one new named phase; same concrete-owner payload minting pattern
// as RelationalPublicationReadmitted. No macro or blanket implementation.
pub struct NativeCheckpointOriginalsVerified<T: AuthorityMarker> { value: T }
impl<T: AuthorityMarker> NativeCheckpointOriginalsVerified<T> {
    pub fn from_owner(value: T) -> Self;
    pub fn value(&self) -> &T;
    pub fn consume<R>(self, next: impl FnOnce(T) -> R) -> R;
}
// Reuse existing RelationalReadmitted<T> for the image-admitted phase and
// RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>
// for the actual World handoff. Concrete consumers never accept arbitrary T.

// worth-foundational: DATA, not proofs or executable capabilities.
pub enum CheckpointAdmissionScope {
    Home {
        home: HomeIdentity, native_origin: [u8; 32],
        complete_source_digest: [u8; 32],
        world_through: [u64; 3], world_image_digest: [u8; 32],
    },
    Interim {
        transport_origin: [u8; 32], registry_revision: u64,
        world_snapshot_body_digest: [u8; 32],
    },
}
pub struct CheckpointAdmissionCut {
    scope: CheckpointAdmissionScope,
    canonical_through: u64,
    transitions_through: u64,
    checkpoint_digest: [u8; 32],
}
pub struct CheckpointProductBindingIdentity([u8; 32]);
// New pure cross-runtime identity: hash of actual installed application/program/
// schema/roster meaning plus the exact product role captured by its owner.
// A copied digest remains DATA and cannot itself admit a receiver.

// worth-relational, durability/checkpoint_admission/producer.rs:
struct VerifiedCheckpointOriginals { /* actual protected originals + fold */ }
impl AuthorityMarker for VerifiedCheckpointOriginals {}
fn verify_originals(
    frozen: CapturedCompleteMetadata,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<NativeCheckpointOriginalsVerified<VerifiedCheckpointOriginals>,
            RelationalCheckpointAdmissionRefusal>;
fn capture_admission_image(
    originals: &NativeCheckpointOriginalsVerified<VerifiedCheckpointOriginals>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<CapturedRelationalCheckpointAdmissionImage,
            RelationalCheckpointAdmissionRefusal>;

pub struct CapturedRelationalCheckpointAdmissionImage {
    // Owned immutable rows, verified-original provenance and actual funding.
    // No Deserialize, Default, from_bytes, or public field constructor.
}
pub struct RelationalCheckpointAdmissionArtifact {
    // Untrusted transport DATA: immutable bytes and bounded region only.
    // Parsing this value NEVER admits a root, subject or semantic result.
}
struct CheckpointAdmissionOwner {
    // Same selected source/read-protection owner, decoded rows, funding,
    // current recovery-runtime binding and actual member custody.
}
impl AuthorityMarker for CheckpointAdmissionOwner {}
fn admit_checkpoint_image(
    source: ProtectedAdmissionSource,
    artifact: RelationalCheckpointAdmissionArtifact,
    expected: &CheckpointAdmissionCut,
    receiver: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<RelationalReadmitted<CheckpointAdmissionOwner>,
            RefusedCheckpointAdmission>;

// distinct from PositionedCanonicalCommit / full original-frame demand:
struct CheckpointAdmittedCommit { /* Arc to admitted owner + exact row key */ }
enum RelationalRootCommitEvidence {
    Original(Arc<CanonicalCommitEnvelope>),
    Checkpoint(CheckpointAdmittedCommit),
}
struct RelationalCommittedBranchRoot {
    descriptor: RelationalBranchRootDescriptor,
    evidence: RelationalRootCommitEvidence,
    axes: RelationalBranchRootAxes,
    publication_cost: RelationalBranchRootPublicationCost,
}
enum ReadmittedEffectOrigin {
    Original(PositionedCanonicalCommit),
    Checkpoint(CheckpointAdmittedCommit),
}
// The existing public ReadmittedRelationalAssociatedEffect keeps its public
// methods; its private positioned field becomes ReadmittedEffectOrigin.
// Original export/CDC still accepts ONLY PositionedCanonicalCommit.
```

`VerifiedCheckpointOriginals` can be constructed only after the existing
protected-read identity, canonical framing/digests, schema/mutation chronology,
folded content and current-component comparisons succeed. It retains those
actual inputs through capture. It is not a description of a proposed source.
Proof::from_owner cannot forge its concrete specialization: the Native payload
has private construction. The closed capture entry does not expose this payload
or accept generic AuthorityMarker callers. Clones of admitted commit references
share the same immutable admitted owner, not newly minted admissions.

The verifier has two exhaustive input postures: Home uses the protected durable
original prefix/fold above; Interim uses frozen actual memory-original canonical
owners, their authoritative payloads and actual captured schema/root owners.
It never invents Home birth/mutation chronology for a memory runtime. The latter
can mint only an Interim-scoped image. Reuse the same projection/authentication
code, with distinct concrete verifier inputs; do not call Home's Complete-source
constructor with missing initial-input defaults. Immutable memory originals are
genuine original input, not newly issued recovery authority. Existing Native
ordinary EmbeddedBytes capture used by interim transport must call this verifier
before its scratch/original owners leave scope.

`ProtectedAdmissionSource` is a Native-owned closed enum: the actual original
Home source plus its `RelationalDurableReadProtection`, or an owned admitted
interim transport. The Home variant is installed only by the existing Store
source construction boundary after validating `AnchoredRecordReadMembership`,
the exact section range and source allocation identity. Native never names
Store's concrete types. Existing EOF/member/source protections remain owned;
a caller-supplied digest or serialized membership cannot construct this value.
The Interim variant never produces Home/durable authority or a Home binding.
It holds ONLY Native-owned admitted slice/origin/receiver custody, never the
Host/Query transport bundle or a World type. The concrete handoff is below.

### Interim origin is a required independent trust input

Integrity of caller-authored bytes is not proof of an original World movement.
`WorthQueryApplicationCheckpoint::from_untrusted_bytes` remains DATA-only and
cannot create ProtectedAdmissionSource::Interim. A public `[u8;32]` parameter,
or a digest extracted from the same supplied wrapper, is insufficient.
The proposed restore contract requires a sealed capture-issued transport origin:

```rust
// proof: legal source progression; runtime payload is concrete at the door.
pub struct InterimCheckpointSourceAdmitted<T: AuthorityMarker> { source: T }
// Host/Query capture owner, private fields and constructor; no Deserialize.
pub struct CapturedInterimCheckpointOrigin {
    // Immutable authenticated wrapper root + application/program/source binding,
    // retained producer-origin seal and its original-request allocation owner.
}
struct AdmittedInterimTransport {
    // Host/Query-owned composition of lower-owner admitted slices, not a type
    // named by Native: SAME bytes/root/cut and actual receiver-pair bindings.
}
impl AuthorityMarker for AdmittedInterimTransport {}
fn admit_interim_transport(
    bytes: WorthQueryApplicationCheckpoint,
    origin: CapturedInterimCheckpointOrigin,
    receivers: &FreshInterimReceiverPair,
    request: &ExecutionRequest<'_, '_>,
) -> Result<InterimCheckpointSourceAdmitted<AdmittedInterimTransport>,
            InterimCheckpointSourceRefusal>;
pub enum InterimCheckpointSourceRefusal {
    MissingTrustedOrigin,
    OriginMismatch,
    ForeignReceiver,
    PredecessorFormat(BoundaryProtocolUnsupportedVersion),
    Request(WorkCeilingDenial),
}
```

Only the genuine cross-owner checkpoint producer issues
CapturedInterimCheckpointOrigin after both original-owner projections are sealed.
Its root covers the ENTIRE WQADMI01 wrapper: unchanged Query bytes (including
embedded Native), the Native admission image, World registry, binding and all
length/version tags. The origin is a separate returned owner, not serialized
inside the checkpoint. Cloning shares the same immutable producer seal and
funded origin; it does not mint a new source or preserve any live service.
Importing arbitrary bytes never creates it. A restore consumes one owning
origin reference and binds one Native/World receiver pair; further restores
need another retained owning clone and receive distinct fresh receiver fences.
Both owner admissions borrow/retain the SAME admitted transport, so exchanging
only one child or one receiver fails before either registry or basis is exposed.
Losing every origin handle yields MissingTrustedOrigin, even for self-consistent
bytes. There is no full-envelope or digest-only fallback.

Grant11b decides this contract: capture proof ONLY. Bytes restored after every
owning capture-origin reference is lost refuse MissingTrustedOrigin. There is
NO persistent custodian, signing/key API, trusted storage/transport policy or
second trust root. A file location never authenticates an original. This is a
deliberate owner-directed clean cutover of the old byte-only caller contract;
callers either retain their actual capture proof or receive the typed refusal.
Do not describe byte-only process-restart restore as supported. The proof binds
the Native image, exact source and cut and Home affinity (Interim explicitly
means NotHome); a proof from another capture/source/scope cannot be reused.

Concrete lower-owner handoff, with no dependency erasure:

```rust
// Native capture owner; constructor private to verified-original capture.
pub struct CapturedNativeInterimOrigin { /* own image + full wrapper seal */ }
pub struct AdmittedNativeInterimSource {
    // SAME immutable wrapper backing, native child span, full wrapper root/cut,
    // native capture origin and Weak native receiver; request-owned funding.
}
fn admit_native_interim_source(
    bytes: ExecutionImmutableBytes,
    origin: CapturedNativeInterimOrigin,
    receiver: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> Result<AdmittedNativeInterimSource, InterimCheckpointSourceRefusal>;
// World capture owner; private proof of original movement/registry/snapshot.
pub struct CapturedWorldInterimOrigin { /* own projection + wrapper seal */ }
pub struct AdmittedWorldInterimSource {
    // SAME wrapper root/cut, World child span, World capture origin, actual
    // World receiver binding/fence and funding; no Native source implementation.
}
```

At capture each genuine lower owner seals the full immutable wrapper root only
after checking its own child span, scope and cut against its ACTUAL captured
output. Query's capture owner also checks its unchanged Query checkpoint span.
Host/Query CapturedInterimCheckpointOrigin contains these concrete opaque
lower-owner origins; it cannot mint them from hashes. Restore splits that bundle
through the closed orchestration into Native's and World's exact facade doors.
Each lower owner consumes its own origin, verifies full-wrapper equality and
binds its actual local receiver. Native's ProtectedAdmissionSource::Interim
stores AdmittedNativeInterimSource only. World's source stores its World owner
only. Common wrapper root/scope/cut and receiver-pair correspondence are DATA;
the actual local receiver ownership is checked by each owning runtime. The
orchestrator compares their admitted bindings before exposing a composed
receiver. No Native import of World/Query, generic authority or Any-erased
transport owner is needed. No public digest-only origin constructor exists.

For a DIRECTORY Home, the new Relational package carries NO projected World
memory movement: its World companion is the explicit DurableOwnerJournal
posture, and unchanged World section/original journals continue to supply
durable World evidence through their existing owner admission. Native owns only
its Native projection, not a claim that a World movement occurred. A memory-
registry projection in Home scope refuses ScopeMismatch; a Home with such live
memory-only records refuses capture rather than silently dropping them. Thus
Home kill/reopen does not depend on an in-process interim origin handle or
invent recovery of a lost capture proof from plain bytes. Grant11b governs every
World memory movement projection actually carried by the new image: only its
capture-issued proof can authenticate it. Existing protected Home selection
remains Native's admission source, not a trusted-location policy or second root.

Wire/authentication preflight runs before a Native receiver exists and retains
the protected source plus immutable verified DATA. It cannot issue a commit
reference or operational basis. The signature above is the later receiver-bound
step: the freshly constructed Native runtime has no exposed service while its
image is bound. Preflight failure therefore precedes incarnation/domain
publication; runtime binding failure retains the actual receiver and source.
The receiver's runtime identity is freshly bound, never read from wire.

```rust
pub enum RelationalCheckpointAdmissionRefusal {
    MissingArtifact,
    PredecessorFormat {
        protocol: BoundaryProtocolIdentity,
        reason: BoundaryProtocolUnsupportedVersion,
    },
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    BindingMismatch(CheckpointAdmissionBindingMismatch),
    AuthenticationMismatch(CheckpointAdmissionAuthenticationMismatch),
    MissingRootImage,
    InvalidProjectedMeaning,
    Request(RelationalExecutionDenialCause),
    Source(RelationalRecoveryStop),
    OwnerUnavailable,
    CapacityOverflow,
}
pub enum CheckpointAdmissionBindingMismatch {
    Scope, Home, NativeOrigin, Cut, Source, Checkpoint,
    WorldImage, Registry, Runtime,
}
pub enum CheckpointAdmissionAuthenticationMismatch {
    Frame, Directory, DuplicateKey, Projection, RegionBinding, SchemaBinding,
}
struct RefusedCheckpointAdmission {
    cause: RelationalCheckpointAdmissionRefusal,
    // SAME partial owner, allocations, source and any acquired custody.
    retained: PartialCheckpointAdmission,
}
```

Place the shared mismatch classifications in foundational only where crossing
the Store/Native/World boundary; keep source/refusal owners in Native. Export
every new public Native type through `facade/durability.rs`, every new public
World type through its facade, proof through lib/prelude, meaning through the
foundational facade. No operational constructor takes a DTO instead of an
admitted owner. Exhaustive enums have no wildcard matching.

## New declared artifact and protected format

Use a new admission package INSIDE manifest2's existing opaque Relational
section. Its three declared children are unchanged CompleteSource bytes,
Native admission-image bytes, and a World-owned receipt-registry companion.
This is a new artifact, not extra fields appended to WSRCSRC1 or WRTHRCS1.
Store Application assembles it from the two genuine owner capture results;
Store Relational reads its Native children and forwards the opaque World child
to World. Native never interprets World records. Cross-owner assembly stays
above both owners; there is no Native-to-World dependency.

Proposed exact protocols:

| Artifact | Magic / version | Meaning |
|---|---|---|
| Relational admission section | `WSRCAI01` / u32 LE 1 | lengths, common cut, child digests, children |
| Native admission image | `WRTHCAI1` / u32 LE 1 | Native original-derived root/publication projections |
| World receipt registry | `WRWMREG1` / u32 LE 1 | committed originals plus explicit receiver-correspondence snapshot |
| Interim transport wrapper | `WQADMI01` / u32 LE 1 | unchanged WQAPCP01/v13 plus admission package |

Protocol identities are respectively `worth.store.relational.admission-section`,
`worth.relational.checkpoint-admission`, `worth.world.receipt-registry` and
`worth.query.interim-admission`. Each accepts exactly v1. They have bounded
length-prefixed, explicitly ordered named records; unknown fields/tags,
duplicates, noncanonical ordering, truncated data and trailing bytes refuse.
No extensibility is implied. Future additions require a declared new artifact
unless a later explicitly defined extension is introduced by its own format.

An image has the common binding above, root/component directory, canonical
projection directory, schema/region references, and typed empty-or-present
registry companion. A canonical row has receipt, original position/digest,
branch context, original predecessor branch-cell observation, successor
observations, association discovery, complete original semantic result,
original created-record patches, schema/descriptor semantics, recorded original
visibility commitment and its root/correctness axes, required-output origin,
materialization, publication positions and codec-retirement facts. Use existing
meaning DTOs with explicit admitted/untrusted wrappers; do not create Query-
specific binding/outbox field parsers in Native. No merged plan, whole update
patch, diagnostic/index payload or canonical envelope is smuggled into the
image to disguise historical-body reads.

Authentication is newly authored by the actual checkpoint owners:

1. At capture the original reader verifies each original frame and its subject;
   the fold verifies root content/schema/visibility and exact before/after axes.
   Only then may the typed producer copy each projected meaning.
2. Each field group gets a separate SHA-256 leaf:
   `H(domain || protocol/version || binding || row_key || group_tag || length ||
   canonical_group_bytes)`. Tags distinguish semantic result, axes, association,
   canonical coordinates, schema, visibility, creation rows and output facts.
   Root/empty-component rows and World registry rows have separate domains.
3. Sorted, duplicate-free directory entries bind row key, all group hashes,
   lengths and group membership. A directory root and artifact header checksum
   cover exact counts, byte ranges and the common cut. Check references form a
   closed graph. One domain cannot substitute for another, even with equal bytes.
4. Store's SINGLE protected Home root decision covers the entire new package,
   unchanged World section and content-addressed members. The authenticated
   Native header binds exact CHK18 and CompleteSource digests; the outer package
   binds the Native and World registry digests plus the World section digest.
   Home common cut uses the unchanged base Native/World artifacts. Interim
   first hashes the binding-snapshot BODY (without cut/authentication wrapper),
   then binds that digest and the unchanged Native checkpoint into the common
   cut; each child then hashes its fields with that cut. Finally hash the whole
   outer package. No child requires the final package root to compute itself;
   capture-origin owners seal that final root after assembly, avoiding a cycle.
5. Admission hashes actual selected bytes and checks every group against its
   directory, then compares common cut and exact schema/region/component keys
   against the selected CHK18/World sections. A self-checksum alone grants NO
   authority: selected membership/source protection or explicit interim owner
   transport is mandatory. Copying Canonical5's flat leaf is never a group proof.

The old visibility commitment is an authenticated original fact, verified from
the full original at capture. Admission checks it against the image's separately
authenticated storage/schema/correctness/commit tuple and the exact descriptor;
it does not pretend to recompute the old full-envelope hash from a projection.
Local root completeness gains an exhaustive evidence-specific validator.

Canonical5, transition2, CHK18, WRTHRCS1/v1, WSRCSRC1/v1, World checkpoint and
WQAPCP01/v13 grammars remain byte-for-byte unchanged. Old WSRCSRC1/WSRCKPT1
selected Home sections and old bare interim transports are recognized ONLY to
return PredecessorFormat; no payload reconstruction or migration reader runs.
Use foundational BoundaryProtocolCompatibilityWindow/UnsupportedVersion,
including Retired posture for explicitly recognized retired protocols. Missing
new children are MissingArtifact; unsupported new versions are UnsupportedVersion.
Old development Homes with a selected checkpoint without this package, including
the previously current Complete source, stop opening. A new selected image may
not omit a covered row and fall back to reading its original frame.

## Grant 11 coverage

| Required meaning | Field/proof | Admission checker |
|---|---|---|
| Original semantic result | semantic group; exact existing semantic DTO, all created identities/counts and receipt | Native image semantic validator; effect owner |
| Predecessor axes | original branch-cell descriptor, observation/generation/truth version; Empty or exact canonical target | Native subject validator and root inventory |
| Successor axes | exact descriptor/reference/posture plus receipt/parents/version/root target | Native successor and recorded-component admission |
| Association | required Unassociated/Associated discovery and full original caller association | Native index/subject/effect validation; World occurrence join |
| Receipt, position, digest | canonical key and group, tied to actual CHK18 index/member locator | Native selected directory admission |
| Schema | exact schema carrier, registry/contracts, schema/version and descriptor semantics digest | Existing schema authority readmission, then root bind |
| Visibility | original commitment plus storage/schema/correctness/commit tuple, separately authenticated | Native root evidence completeness validator |
| Source affinity | actual source Weak/allocation identity and original protected read owner | Native protected source admission |
| Home affinity | HomeIdentity + native initial-input origin digest; no process address in wire | Store descriptor/source admission and Native cut validator |
| Cut affinity | canonical/transition/World positions and exact selected child digests | Store package admission, Native and World |
| Protected membership | same anchored read membership/section span, selected member custody; never serialized as authority | Store source owner, Native read protection |
| Retention custody | actual reconstructed root/schema owners and original member/source retention | Native component admission, unchanged basis registration and retain_component_basis |
| Fencing | local recovered runtime identity, owner-binding operation guard, branch cell/publication binding | Native operational admission; existing Home incarnation gate |
| Funding | request-owned byte backing, decoded inventory, root allocation and retained custody grants | Native/World/Store allocation owners |
| Refusal custody | RefusedCheckpointAdmission and existing partial publication/component owners | Failing owner; no basis/phase escapes refusal |
| Original-frame distinction | CheckpointAdmittedCommit versus PositionedCanonicalCommit | Compiler and exhaustive root/effect origin matches |
| Missing artifact/Homes | mandatory selected package/children, typed MissingArtifact/PredecessorFormat | Store preflight before Native/incarnation/owner publication |
| D13 registry | committed World rows + Native row keys + original creation patches, all independently hashed | World registry owner plus Native origin correspondence |

## D13: original receipt registry, not receipt fabrication

The integration note `m3-host-home/target/lane-evidence/d13-interim-registry.md`
appeared during this turn and was read in full. It records integration
58564ccf9f and says these registry paths are unchanged from838d287427. Its
requirement for a World-authored movement projection cross-bound to the Native
image and coherent cut is included here. Source at838d287427 establishes the
same boundary. Capture committed
`MemoryPublicationRecord`s from `MemoryOriginalRegistry`, including records
whose live history entry was reclaimed; exclude Vacant and uncommitted slots.
Freeze under the actual publication/registry owner, copy immutable references,
release locks, then do request-funded encoding and any Native original reads.
Validate revision/currentness before sealing; conflict refuses, not best effort.

Each World registry row preserves exact original before/after
MemoryProductBranchReference values, attempt owner/ordinal, before/after Native
recorded component coordinates, corresponding Native semantic row key, committed
movement evidence and source-owner transport identity. Native must authenticate
the complete original created-record patches needed by binding, dispatch-outbox
and aftermath-causality reads, even after current rows change. World authenticates
its own row domains and verifies successful movement, association/native receipt,
reference successor/lineage and a unique Native-origin correspondence. Duplicates,
missing Native rows, conflicting attempts or a claimed committed pending slot
refuse. Retained/unpublished Native effects must never become committed registry
entries merely because their Native image exists.

No Weak, Arc address, live result, settlement ticket or old runtime capability is
serialized. Restored World registry records are a private exhaustive origin
family: actual live original versus owner-admitted interim original. The latter
retains the authenticated original coordinates and a new current-runtime-bound
Native observation; it does not forge the former's weak-envelope identity.
`ObservedRelationalMemoryCommit` and `ObservedRelationalMemoryComponent` receive
corresponding private origin families. Their restored-interim variants carry
CheckpointAdmittedCommit, never PositionedCanonicalCommit. Matching joins that
same admitted image owner/row and current runtime binding; live variants retain
the current pointer-identity checks. No image-to-live-Performed conversion.

Existing World observation signatures and receipt minting stay unchanged:
`observe_memory_commit` admits the explicit restored-INTERIM origin only in an
interim-bound runtime, still refuses Home-source-backed artifacts; World's
`observe_memory_original_publication` matches the privately admitted registry
origin. Original-frame export always separately demands the full frame.
Preserve original coordinate values; carry an explicit original-owner-to-current-
receiver binding for affinity checks. Do not rewrite archived owner ids to look
like newly issued originals. Existing output/outcome frontiers still resume from
their actual owners; registry rows grant no future identity issuance.

One exact Query observation-comparison change IS necessary. In merged
`provider/original_publication.rs::OriginalPublication::matches_affinity`, the
Memory arm currently compares archived `after().owner()` to the fresh World
owner. Replace that arm's direct comparison with a World-owned readonly
`WorldObservedMemoryPublication::matches_receiver(&ProductBranchObservation)`.
The live origin arm performs today's exact owner/name/incarnation check. The
restored origin arm additionally requires its admitted transport and actual
current World receiver identity, then checks the authenticated old branch/
incarnation to current branch/incarnation correspondence established by restore.
No name-only match or unchecked caller-supplied owner map. The explicit
WorldReceiptBindingSnapshot below supplies its source anchor; the mapping is
installed only by that snapshot's receiver admission and retained with the
origin owner. It covers the one declared checkpoint product receiver; all other
archived branch/incarnation records are explicitly ArchivedOnly.
`owner_coordinate()`, `before()`, `after()`, attempt and causality rows continue
reporting ARCHIVED original coordinates. Do not replace the owner used by
source_receipt.rs's original causality consistency checks. This additive readonly
World observation and exact Query comparison file are explicit D13 coordinator
handoffs; they do not alter held basis/publication/close signatures.

Similarly, export_memory_component matches its private Live/Restored origin
exhaustively: Live keeps original Arc identity; Restored exports the exact
checkpoint-admitted original component description from the same receiver/source.
It cannot manufacture a PositionedCanonicalCommit or executable basis. A
receipt-only historical component whose root has been legitimately reclaimed
may remain root-free, as today's memory registry is; it cannot satisfy
operational recorded-component admission. Distinguish receipt-only rows from
operational root rows, and retain MissingRecoveredImage for the latter.

### Explicit World receiver-correspondence snapshot

The pinned interim path does NOT already carry a World branch image:
`execution_runtime/product_world/bootstrap.rs::install` constructs a fresh owner
and fresh `primary`, and DecodedApplicationCheckpoint has no World field.
Therefore WRWMREG1 must include the following World-owned snapshot, under its
own `world-receiver-anchor` authentication domain as well as the whole-wrapper
origin proof. Do not infer it from registry rows or an equal-looking fresh head.

```rust
// World-owned untrusted wire DATA, then checked owner payload; Native component
// DTOs remain Native's vocabulary, not a new World interpretation of roots.
struct WorldReceiptReceiverAnchor {
    source_owner: u64,
    source_branch: String,
    source_incarnation: u64,
    source_head: MemoryProductBranchReference,
    native_at_capture: RecordedRelationalComponentReference,
    native_image_row: CheckpointComponentRowKey,
    product_binding: CheckpointProductBindingIdentity,
}
struct WorldReceiptBindingSnapshot {
    anchor: WorldReceiptReceiverAnchor,
    records: Vec<WorldOriginalPublicationProjection>,
}
enum RestoredPublicationReceiver {
    Bound(Arc<WorldReceiptReceiverBinding>),
    ArchivedOnly,
}
struct WorldReceiptReceiverBinding {
    // Actual current owner/branch/lifecycle receiver witness + source anchor,
    // same admitted transport owner, native admission and owned funding.
}
```

Capture rule: the actual Query product checkpoint composition passes its actual
World ProductBranchObservation to a World-owned capture method. The World owner
checks that observation against its real branch cell and lifecycle incarnation,
captures its head/component under the same publication revision as the committed
registry snapshot, and seals that exact observation as the single product
receiver anchor. The new pure `product_binding` digest is derived by the actual
Query checkpoint producer from its installed application/program/schema/roster
meaning and exact product role/branch selector; it is not an assumed existing
wire field or an id invented by a decoder. The receiver computes that same
meaning from its actual installed configuration and requires exact agreement.
The outer independent origin binds this digest along with the whole package.
Native sealing checks that native_at_capture is the exact captured
Native component/image row and that the selected Query Native image has that
same product component. A caller cannot relabel an arbitrary registry branch as
the checkpoint's product role. A concurrent change refuses the capture.

Restore rule: after the new Native receiver admits its image and before any
new Query-authored mutation/service, Query's restore composition gives the
same admitted transport and anchor to World's product bootstrap owner. That
owner first admits the anchor's actual Native basis through the recovered
Native authority and checks the configured product binding against the sealed
anchor. It then constructs the actual fresh World/product receiver using that
basis, records its actual newly issued owner/branch/incarnation in a private
WorldReceiptReceiverBinding, and installs the registry in the same private
readmission phase. The old World head/attempt coordinates remain archived
DATA; they are not installed as fresh live CAS movements or Performed results.
If baseline bootstrap would already have published new Query changes, move this
binding step earlier; never accept its later head as a proxy for the anchor.

Scope is explicit: rows for the anchor's exact original owner/branch/incarnation
receive Bound to THIS receiver; records from another branch or earlier
incarnation remain preserved as ArchivedOnly and cannot satisfy that receiver's
receipt-affinity check. This preserves today's memory receipt branch/incarnation
scope. It does not silently drop other originals, restore unmentioned active
World branches, or claim to migrate the whole World branch topology. Each
transport targets one declared product receiver; restoring another product
requires its separately captured binding and original anchor. Receipt/history
metadata grants no executable root to an ArchivedOnly row.

Binding admission returns distinct typed refusals for MissingReceiverAnchor,
DuplicateReceiverAnchor, ProductBindingMismatch, SourceBranchMismatch,
NativeAnchorMismatch, ForeignTransport and ReceiverAlreadyBound. A refusal keeps
any actual admitted Native basis/pin and partial World receiver in its existing
readmission custody; it does not return a serving receiver. Receiver closure or
incarnation change invalidates the binding. Subsequent ordinary publications
may advance its head without invalidating historical receipts on that same
branch/incarnation. Query matches_receiver checks the real binding, not source
and receiver numeric-owner equality.

Required proofs include a registry with no anchor; an anchor from another
capture; an unrelated branch with equal name/head; wrong Native anchor and
product selector; archived-only/old-incarnation receipt rejection; successful
restoration after allocating an unrelated World first; preservation of original
before/after/attempt/causality coordinates while only affinity uses the current
binding; and late refusal with an acquired Native pin. Exact scheduling seam:
World receipt-registry snapshot/receiver-binding children, Query
`execution_runtime/product_world/bootstrap.rs`, and
`primary_graph/bootstrap/preparation.rs` plus checkpoint decode staging. No
existing durable World basis/commit/publication/close signature is replaced.

The new interim wrapper carries unchanged Query checkpoint bytes, Native image
and authenticated World companion as one transport. Its whole digest is
integrity for EXPLICIT NON-DURABLE transport, not Home durability. Query's same
frozen interim door captures/restores the companion before receipt lookup and
before service. No new interim caller is added. A durable Home encodes an
explicit durable-registry posture joined to its existing durable World
occurrences; it never treats an absent memory companion as an empty registry.
These narrow registry/observation-origin changes are required D13 owner seams,
not permission to redesign World history, close or publication interfaces.

Proposed cross-owner capture and admission signatures make the second author
explicit (new operations; existing World observation/basis signatures unchanged):

```rust
// worth-runtime-world, private construction, public opaque facade result.
pub struct CapturedWorldReceiptRegistry {
    // Actual immutable committed registry records, owner revision/fence,
    // byte backing and original-request funding; no public DTO constructor.
}
impl WorldCheckpointPort {
    pub fn capture_receipt_registry(
        &self, request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedWorldReceiptRegistry, WorldReceiptRegistryRefusal>;
}
// Memory-only interim capture uses the actual World owner checkpoint receiver,
// rather than constructing a durable WorldCheckpointPort without a Home.
impl CapturedWorldReceiptRegistry {
    pub fn seal_native_correspondence(
        self,
        native: &CapturedRelationalCheckpointAdmissionImage,
        cut: &CheckpointAdmissionCut,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedWorldReceiptRegistry, WorldReceiptRegistryRefusal>;
}
// Store Application or existing Query interim composition, above both owners:
fn assemble_admission_package(
    native: CompleteRelationalNativeCheckpoint,
    world: PreparedWorldCheckpoint,
    registry: PreparedWorldReceiptRegistry,
    request: &ExecutionRequest<'_, '_>,
) -> Result<PreparedCheckpointAdmissionPackage, PackageRefusal>;
// Analogous interim assembler takes actual Query/Native/World capture owners;
// it cannot manufacture PreparedWorldCheckpoint for a memory-only World.
// World private restoration, before service, returns its concrete phase:
fn readmit_receipt_registry(
    receiver: &RuntimeWorldOwnerRoot,
    source: ProtectedWorldReceiptRegistry,
    native: &RecoveredRelationalRuntimeAuthority,
    request: &ExecutionRequest<'_, '_>,
) -> Result<WorldReconciled<WorldReceiptRegistryReadmission>,
            RefusedWorldReceiptRegistry>;
```

`PreparedWorldReceiptRegistry`, `ProtectedWorldReceiptRegistry`, partial refusal
and `WorldReceiptRegistryReadmission` are funded World runtime owners, not new
foundational capability descriptions. ProtectedWorldReceiptRegistry comes only
from the same authenticated Home package or explicit verified interim wrapper.
The Native captured image exposes readonly canonical coordinates and projected
meaning needed for correspondence, but no root/proof constructors to World.
The Native image's own emitter does not pretend to certify World movement:
the package constructor requires the genuine World result. No public bytes-only
assembler can produce the strong publication subject. Exact closed producer
boundaries cover both Home and interim assemblers.

## Constructor cutover and actual operational custody

| Merged constructor/consumer | Required replacement | Why World receives actual owners |
|---|---|---|
| RelationalCommittedBranchRoot in root.rs | exhaustive Original/Checkpoint evidence; receipt/schema/visibility accessors independent of envelope presence | owns real region set and schema authority, same runtime-issued root identity |
| root_readmission.rs and request child | add private readmit_checkpoint_region_directory accepting only CheckpointAdmittedCommit and admitted CHK18 directory; share actual region allocation/binding | descriptors checked against real region hashes and executable schema; no synthetic envelope |
| associated_effects/readmission.rs | covered source placement selects admitted image row; uncovered tail selects verified positioned original; missing covered row refuses | same subject/catalog/source/runtime checks, original semantics and actual settlement registry |
| recovered_publication/subject.rs | compare admitted predecessor/successor/association facts through exhaustive origin accessors | validates the same original Native-to-World occurrence relationship |
| recorded_component_readmission.rs | compare image receipt/position/digest/root against target and admitted source, then existing issue/register basis | existing cell, branch identity, retention binding and registration really run; MissingRecoveredImage stays |
| checkpoint_images.rs | materialize retained canonical/Empty image from exact admitted row + existing schema/region directory, without demand_positioned | materializes real root, then actual retain_component_basis leases; missing root remains denial |

Also cut over successor.rs, created_row.rs, basis::commit_identity, fork target,
delete head-version and root schema/completeness readers to typed fact accessors.
Original-frame byte export and live memory-pointer operations match Original
explicitly; they cannot receive None and silently act Empty for a checkpoint
root. Recorded-component DESCRIPTION export is different: in
`branch/recorded_basis/export.rs` an exhaustive Checkpoint arm validates the
same admitted owner/row, receipt/version, root, source and descriptor, then
copies the authenticated recorded coordinates without demanding a full frame.
Its Original arm retains P-D's request-funded original-payload agreement/digest
checks. `export_memory_component` has the analogous restored-interim arm noted
above. These metadata description paths must not defeat the bounded reopen by
silently invoking original-frame demand. New live
commits use Original; unchanged forks may share Checkpoint evidence. On-demand
artifact/CDC reads keep original source verification, sticky damage and request
precedence. A later Complete re-verifies originals through its original read
operation; admission images are not authority to invent original frame bytes.

World's existing `admit_recorded_component`, `admit_recorded_publication` and
RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody> contracts
stay unchanged. Recovered publication still acquires predecessor and successor
bases and both real leases, validates twice, and retains partial acquisitions
on failure. CheckpointReadmitted remains distinct from PerformedSettled. No
image marks a pending publication settled or supplies a live CommitResult.

## Capture, limits, selection and kill semantics

Native captures within checkpoint_complete/prepare.rs after verified originals,
folded roots, schemas, component correspondence and canonical index exist, before
scratch originals are dropped. Factor the already verified fold into the concrete
proof owner rather than reading each original twice. Image rows cover every
covered canonical subject needed for discovery, root materialization or receipt
lookup; retained Empty and historical images remain separately required. Keep
actual obligations at origin. Do not reconstruct a missing root from an image
descriptor or union old headers.

Store preparation is World-first in the merged source. Keep that ordering.
Capture immutable World companion input at the same owner fence, then Native;
establish the common cut after Native coverage is known, seal both child bindings,
and validate all cross-references before physical publication. A World occurrence
may not reference Native beyond its capture cut. Concurrent advancement either
falls in an actual later tail or returns a typed cut conflict. Never hold World
or Native logical locks across source I/O or the other owner's callbacks.
Store Application's private prepared-package constructor requires the actual
Native strong capture and actual World strong capture; raw child bytes cannot
construct that producer result. Existing selection subject binds the NEW entire
section hash, member roster and exact initial-input coordinate, not its old
WSRCSRC1 child alone. ACK and selected-read verification compare those same
subjects. Do not introduce an independent anchor or durable write side channel.

Reserve from the invoking ExecutionRequest before each vector/Arc/byte allocation
or growth. Charge actual encoded/copied/hashed bytes and rows using the existing
request context, poll in bounded chunks (at most64KiB), checked lengths/counts,
and preserve nested stops. No fresh SerialRequest, System fallback, profile
override or uncharged reallocation. Funding includes pointer/vector capacity,
schema/created-row text and values, directory indexes, encoded children and
temporary overlap with original decoded frames. Destroy payloads before their
reservations. Keep actual grants through asynchronous selection and ACK.

Let N be covered original rows, B their verified bytes, P projected bytes, R
root/region references and W registry bytes. Capture adds O(P+R+W) work to the
existing O(B) verified fold, O(P+R+W) output plus explicitly funded overlap.
No algorithm repeatedly scans every original for each World occurrence. Reopen
authenticates O(P+R+W), builds indexed row lookup once, and does O(log N) lookups
per subject. It reads ZERO covered canonical bodies for image admission and only
the actual bounded canonical tail; demand of region members is separate real
storage work, not relabeled canonical body work. This meets manifest-plus-tail
cost, not a false O(1)-sized checkpoint claim. Existing1GiB section/package limit
and owner limits bound input bytes; capacity/work failure is typed before
selection. Do not add ordinary caller-facing quote APIs.

Before selection, kill leaves the prior complete package selected and orphan
members unselected. After selection, the one new package includes every child;
kill before Native/World ACK resumes that original attempt, never recaptures it.
Source protection and member custody protect the exact selected package across
replacement. Refused admission retains its same actual source and partial pins;
normal release/disposal follows the existing explicit owner retirement path.
No durable effect or serving publication precedes format/authentication preflight
at the Native/domain boundary; physical Store recovery itself is not claimed
effect-free. No new stack-size setting or large by-value phase carrier.

## Proof plan and scheduling inventory

No lane below ran this turn. Select by exact test name, never ignored tests or
exhaustive oracles; retain full required evidence headers when implementing.

| Obligation | Named proof to add/retain |
|---|---|
| D12 bounded work | unchanged automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom <=40; paired tiny/large covered history with equal tail reads; observe actual source seam |
| Original-only construction | compile-fail consumer cannot construct VerifiedCheckpointOriginals, admit DTO as root, convert CheckpointAdmittedCommit to PositionedCanonicalCommit or fake a concrete proof payload |
| Missing child | selected_home_without_admission_image_refuses_before_native_publication; same directory bytes/initializer count unchanged |
| Predecessors/future | selected_predecessor_admission_package_refuses_with_retired_protocol; unsupported_admission_version_refuses_before_allocation_of_roots |
| Binding | foreign_home_admission_image_refuses; admission_image_from_another_cut_or_source_refuses; each mismatch enum field independently mutated |
| Authentication | semantic_result_projection_damage_refuses; predecessor_successor_projection_damage_refuses; association_projection_damage_refuses; schema_visibility_projection_damage_refuses; changed_creation_row_projection_refuses |
| Membership | self_consistent_unselected_image_cannot_admit_a_root; selected_image_cannot_splice_another_registry_or_world_section; duplicate_or_missing_projection_row_refuses |
| Real custody | image_admission_acquires_actual_basis_and_schema_retention; late_image_admission_refusal_retains_acquired_pins; owner_close_fences_image_observations |
| Original separation | image_admission_leaves_original_bodies_cold_but_cdc_demand_verifies_damage; original_frame_demand_preserves_request_stop_precedence |
| D13 | interim_checkpoint_restores_original_receipts_after_live_history_retirement_and_row_mutation; original_before_after_and_attempt_are_preserved; uncommitted_registry_slot_never_restores_as_published; foreign_or_damaged_registry_refuses; receipt_clone_shares_authenticated_owner |
| Interim trust | rehashed_forged_interim_bytes_refuse_without_captured_origin; unrelated_captured_origin_cannot_authenticate_rehashed_wrapper; missing_origin_refuses_even_when_all_checksums_match; Native_and_World_cannot_admit_different_transport_receivers |
| Fresh World affinity | restored_receipt_resolves_after_another_world_consumes_owner_identity; original_owner_causality_is_preserved_while_receiver_affinity_is_checked; another_receiver_cannot_use_restored_registry |
| Capture funding | original_request_exhaustion_prevents_admission_image_selection; retained_image_funding_survives_capture_request_unwind_and_partial_ack; swallowed_stop_never_returns_capture_success |

Retain current selected_complete_home_reopens_and_selects_another_complete_checkpoint,
journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing,
killed_home_reopens_normalized_image_plus_nonempty_tail,
interrupted_owner_ack_resumes_original_attempt_and_reopens_unacknowledged_selection,
same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key,
selected_completion_and_activation_failures_keep_owned_home_without_initializing,
changed_host_build_keeps_initializer_closed_and_reaches_native_readmission and
competing_host_processes_admit_exactly_one_writer_until_process_death. Run the
new Host controls once directly with RUST_MIN_STACK unset. Preserve the current
damage/foreign/schema/incomplete/predecessor refusals and second-Complete retry.
Keep CDC cdc_property_persisted_random_operation_matrix_recovers with unchanged
seed; seeded_full_key_differential_fork_recovery_and_replay_keep_actual_names_and_lineage;
existing original-input, recorded-component, cold-index, generation, cancellation,
pool-transfer and checkpoint ACK courts. Test fixture wire construction changes
must preserve their original behavior claims; no weakened assertion/bound.

Expected Native files, split by responsibility at <=400 code lines:

* New `durability/checkpoint_admission/{mod,artifact,wire,producer,projection,
  authentication,admission,source,refusal,registry_companion}.rs`; split any
  producer/group/validation child further by its meaning, never helpers/common.
* Existing `authority/checkpoint_complete/{prepare,capture,source,canonical,
  roots,regions/grouping,branches/axes}.rs`; new projection capture child.
* `branch/{root,root_readmission,root_readmission/request,root_visibility,
  root_schema,root_schema_meaning,basis,fork_target_preparation,lifecycle_delete}.rs`;
  new root checkpoint evidence child, not a generic bag.
* `authority/{associated_effects,associated_effects/readmission,
  associated_effects/created_row,associated_effects/memory_original,
  recovered_publication/subject,recovered_publication/successor,
  recorded_component_readmission,runtime_rebuild/root_inventory/checkpoint_images}.rs`.
* `history/commit/envelope_residency` selected owner attachment and root facts;
  checkpoint directory/recovery_source carry admitted image owner; original
  demand/export must stay strict. `branch/recorded_basis/{export,memory_component}.rs`
  are exact coordination seams with P-D. No source-retirement redesign.
* Native `medium/{checkpoint_selection,recovery_source,read_protection}.rs`,
  actual checkpoint-directory image ownership and canonical publication subject
  binding: retain the admitted image and seal the new whole-section subject.
* Native public durability facade and public recovery/capture refusals, their
  exact exhaustive callers; existing ordinary source fixtures use real owners.

Cross-owner scheduling seams (design only; coordinator must schedule ownership):

* Proof named phase file + lib/prelude; foundational cut/scope meaning + facade.
* Store Relational new admission-section codec, existing complete codec/readmission,
  checkpoint publication/Complete source types; Store Application checkpoint
  preparation/framing and selected-source preflight. Shared section selection
  identity must bind the entire new package. Avoid touching physical recovery
  admission/layout work being repaired by integrate2.
* World new receipt-registry checkpoint capture/admission children of
  `history/catalog/memory_original`, private observation-origin matching in
  `history/publication/memory_observation`, and its facade/owner checkpoint handoff.
  Existing durable history/catalog/commit/basis and close interfaces are unchanged.
  Coordinate these exact registry files with integration before implementation.
* Query application_checkpoint new outer transport codec and capture/restore
  composition + existing frozen interim door and independent origin input.
  Exact Query `provider/original_publication.rs::matches_affinity` handoff above.
  Provider/idempotency source_receipt is read-only design evidence; preserve its
  receipt minting API and original intent/causality checks. Schedule the exact
  comparison seam with its owner rather than bypassing the receipt path.
* Existing relational certification, World and Query/Host court crates; Store
  Section/selected-source courts. No new court feature or extra target directory.
* Narrow exact factory/caller entries only for newly introduced closed producer
  and selected-source admissions; list exact entries when implementation paths
  are finalized. No pattern broadening, baseline row or new interim caller.

Implementation phases: Native proof/producer/image and constructor cutover as
one coherent unit; Store protected selection transport + first bounded Home
journey; D13 registry/transport journey; adversarial refusals/funding/ACK and
original-frame courts. At actual phase completion run the required formatting,
line-cap, boundary and generated-context checks, then fresh independent review.
No build/test phase is scheduled until coordinator resolves disk/build ownership.

## Owner decisions

Q1 RESOLVED by grant11a, read during this design turn: an existing SELECTED
CHECKPOINT requires the image. Absence there refuses, with no eager fallback.
A Home that has never selected a checkpoint keeps genuine original-input-plus-
journal recovery; the journal-only kill/reopen court stays unchanged. This is
not a fallback or checkpoint compatibility path. There is no zero-cut birth
image and no new birth side channel.

Q2 RESOLVED by grant11b, read during this design turn: only capture-issued proof
authenticates the projected World movement/registry; bytes without a valid proof
or with a mismatching proof refuse. No persistent trust policy or second root.
The supported interim path retains the capture origin; proof-losing byte-only
callers now intentionally refuse and must not have a compatibility fallback.
No owner semantic question remains. Scheduling the named cross-owner files
against integration is still required before implementation.

The D13 integration note is now incorporated; recheck its owner seams against
the eventual integration landing before implementation.
## Independent design review and resolutions

1. `/root/d12_admission_image_design_review`: REJECT initial draft. High findings:
   untrusted interim bytes had no independent original-provenance authority;
   preserved archived World owners would fail Query's direct current-owner
   comparison. Resolved with capture-issued origin/loss/refusal contracts and
   the exact World matches_receiver / Query matches_affinity handoff, preserving
   original owner_coordinate and causality. Grants11a/11b resolved both owner
   policy questions; no persistent trust policy remains.
2. `/root/d12_admission_image_design_review_final`: REJECT the next draft. High
   finding: receiver affinity assumed an undeclared World branch image.
   Resolved with WorldReceiptBindingSnapshot/ReceiverAnchor, its real capture
   authority, exact Native correspondence, restore-before-effects ordering,
   Bound versus ArchivedOnly scope, typed refusals and named bootstrap seams.
3. Fresh `/root/d12_admission_design_third_review`: **APPROVE — design completeness
   only**, after reading grants11/11a/11b, the D13 integration note, the final
   draft and pinned merged source. Its high findings about stale persistent-
   custodian assumptions, a Native reference to an upper-owner transport type,
   and lost interim proof across Home process death are resolved: capture proof
   only; concrete lower-owner origin/source handoffs; explicit durable-journal
   Home posture with no projected memory movement. No additional material
   authority assumption or unresolved semantic choice found.

The approval covers implementation planning, not implementation correctness,
the <=40 body-read result or a green lane. Every proposed proof/test above is
UNRUN. No Cargo invocation, build, test or integration worktree edit occurred.
Cross-owner scheduling and rereading eventual integration seams are required
before implementation. Grant12's separate test-topology cutover remains later
implementation work and was not executed in this design-only turn.
