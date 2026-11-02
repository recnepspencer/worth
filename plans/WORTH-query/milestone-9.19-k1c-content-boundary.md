# K1c: Native content boundary review

Status: accepted implementation boundary, not implemented or qualified. Initial
reviewed checkout: `567b4f8abbb38a96e651fb8bdadeaf2a1d2c9e66`. Before implementation,
master carrying K1b, K1e, and the issue 36 repairs was merged at
`09e6b497484b14a57e77dc9173fd5c9f117583d8`. The user authorized the five slices
after the corrections recorded here land in their own documentation commit.

The user clarified that establishing Native content authority is in scope.
K1c in [milestone 9.19](milestone-9.19.md#phase-6-proprietary-cutover-and-integrated-cad-acceptance)
requires both admission before publication and declared Source dependency. The
separately supplied governing plan remains read-only. No application-specific
schema, inline-bytes scalar, new AspectShape form, or SDK pin change is proposed.

## Adversarial constraint and actual starting point

A field can contain a plausible reference while its bytes are absent, damaged,
incompatible, or supplied by another owner. A digest can agree while the supplied
body disagrees. A cancelled attempt can leave retained bytes charged to nobody.
A managed candidate can remain stale if only the scalar reference is tracked.
K1c must refuse all of these before geometry evaluation or graph/index selection;
it must also preserve content reachable only through undo or retained history.

These are observations of current code, not capabilities inferred from names:

| Current source | Actual authority and missing handoff |
|---|---|
| [values/references.rs](../../crates/worth-foundational/src/values/references.rs) and [content_lowering.rs](../../crates/worth-foundational/src/compatibility/content_lowering.rs) | ContentRefId is a public u64 wrapper; JSON lowering checks only u64 shape. It supplies no availability, format, or admission evidence. |
| [aspect value codec](../../crates/worth-relational/src/aspect_wire/aspect_value_canonical_codec/writing.rs) | Bytes and ContentRef both encode that u64. No byte resolver exists behind either scalar. |
| [branch root](../../crates/worth-relational/src/branch/root.rs), [transaction admission](../../crates/worth-relational/src/mvcc/transaction/admission.rs), and [savepoints](../../crates/worth-relational/src/mvcc/transaction/savepoint.rs) | Native owns immutable roots, genuine branch bases, staged intents, rollback, and retention obligations. A content table must participate in these objects rather than live beside them in Query. |
| [canonical envelope](../../crates/worth-relational/src/history/data/canonical_commit_envelope.rs) and [publication execution](../../crates/worth-relational/src/authority/commit/pipeline/publication_execution.rs) | Native owns the authoritative commit artifact and preparation/publication sequence. No content payload section exists today. |
| [durable medium](../../crates/worth-relational/src/durability/medium.rs) and [cold source](../../crates/worth-relational/src/durability/medium/cold_source.rs) | Existing downward ports publish opaque canonical commits and retain/read checkpoint members or commits. They are not currently content-reference read/write doors. |
| [Store journal adapter](../../workspaces/worth-store/crates/worth-store-relational/src/journal_medium.rs), [checkpoint record](../../workspaces/worth-store/crates/worth-store-relational/src/checkpoint_record.rs), and [cold adapter](../../workspaces/worth-store/crates/worth-store-relational/src/cold_source.rs) | worth-store-relational already implements those Native ports over Store. Store persists opaque Native bytes; Native checks their meaning on readmission. |
| [projected content](../../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/projection/projected_tree/content.rs) and [exact comparison](../../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/projection/projected_tree/content_comparison.rs) | Phase 4 retained content means complete projected bodies, occurrences, slots, and dependency coordinates. Hashes shortlist comparison; actual bodies decide equality. It is not a content-byte store. |
| [Source history](../../crates/worth-relational/src/history/data/canonical_commit_envelope/live_managed_source_history.rs) and [history selection](../../crates/worth-relational/src/history/access/relevant_writes/managed_source.rs) | Live managed Source packets bind actual envelope custody. Deep cloning and recovery do not reconstruct their issuance; missing packets refuse as IncompleteManagedSourceHistory. |
| [CandidateWriter](../../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/handler/candidate.rs), [DecisionReader field read](../../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/handler/decision_reads.rs), and [managed Source row](../../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/managed_index/source_row.rs) | Installed operations already carry field authority and original execution. Source rows already lend controlled invocation-confined text. These are the entry points to extend. |

## Decision: Native owns bytes and their lifecycle

Add one content responsibility under `worth-relational/src/content/`. It owns
immutable canonical frames, their identity-to-frame association, format admission,
candidate attachments, retention, and readmission. Its persistent directory is
part of each Native branch root. A derived lookup accelerator is rebuildable from
that directory and must never become the only route to bytes.

New content and the fields naming it are one transaction. Staging may create an
attempt-owned admitted frame but cannot insert it into committed global state.
Savepoints include content staging; rollback releases removed attachments.
Preparation seals the complete content closure of the proposed fields, including
the before images needed by retained history. Candidate Source and invariant
reads consume that same staged closure. No ordinary mutation, merge, correction,
bulk write, or low-level Native field path may bypass closure validation.

The canonical envelope carries newly introduced frames and exact content
associations needed to interpret its patch; checkpoint capture carries the
complete retained closure. Native's existing medium publishes this artifact
before live root installation. `Committed`, proven-no-effect, and indeterminate
publication retain their current meanings. Indeterminate fate keeps recovery
custody and fences further publication; cancellation cannot claim rollback of a
durable or possibly durable commit.

Use the existing checkpoint-member and canonical-commit ports for physical bytes.
Add a Native content-member kind and manifest directory, covered by the Native
frame/version/digest checks and existing selected-member retention. Commit tails
carry new frames until checkpoint membership makes them independently reachable.
Store's blob runtime is unnecessary for K1c. Store may choose physical placement
inside its existing adapter; it gains no format or reference-admission authority.

Dependency shape stays `Query -> Relational -> Foundational/Proof/Execution` and
`worth-store-relational -> Relational + Store`. The [boundary rules](../../tools/boundary-check/config/road1.toml)
explicitly forbid Query -> worth-store*. Relational has no Store dependency;
adding one would also cycle through worth-store-relational if that adapter were
used as the implementation. The smallest legal shape is to extend Native-owned
artifact/member data through the existing ports, with no reverse import or new
Store service. This is source inspection, not a claim that a new dependency graph
has passed boundary-check; no graph changes or builds occur in this slice.

## Decision: content-derived identity, descriptive until admitted

Replace ContentRefId's u64 representation with the full 32-byte SHA-256 of a
canonical Native content frame. Do not truncate a digest into the old scalar or
retain a second u64-to-digest authority. The portable identity and format
descriptor belong in worth-foundational; constructing a descriptive digest grants
no access or publication authority.

The sole new production frame-hashing owner is
`crates/worth-relational/src/content/frame/integrity.rs`. In the slice that adds
it, extend the exact-owner guard in
`worth-query-installation/src/application_query/tests/canonical_basis_residue.rs`
to inspect Native's content tree and permit SHA tokens only in that exact file,
alongside its existing exact projected-content and retained-capability owners.
Require the new owner's original-request chunk admission; do not exempt the
directory or permit a second Query-side hasher. The court's raw SHA implementation
checks the written frame independently and is not used by the production writer.

The frame is `WRTHCONT` (8 ASCII bytes), envelope version (u16), format-name length
(u16), that many ASCII format-name bytes, format version (u32), payload length
(u64), then exactly that many canonical payload bytes. All integers are
little-endian. Envelope version 1 is the sole initial version. Format names match
`[a-z][a-z0-9._/-]{0,255}`; no case conversion or Unicode normalization occurs.
Lengths are checked before slicing or allocation, and trailing bytes refuse.
Hash the fixed ASCII domain `worth.native.content.v1` followed by one zero byte
and this complete frame. Format and version are inside the hash, so equal payload
bytes in different formats or versions have different identities.

Equal complete canonical frames have the same identity across writes, branches,
and recovery. A new write may share an existing admitted immutable backing after
full-frame equality. Hash agreement alone never admits a replacement backing:
same identity with unequal frames is ContentIdentityConflict, an internal
inconsistency refusal. A forced-collision court must exercise this decision.
Source support equality likewise uses shared admitted backing or paid exact
frame comparison, retaining Phase 4's rule that commitments are only shortlists.

A field value alone still does not prove the expected format. The installed
field contract binds its ContentRef scalar to one named format/version. Native
resolves that reference at the genuine selected/candidate root, verifies the
frame and identity, applies that installed format's canonical admission, and
issues the completed read. Unknown format, wrong version, or absent content
returns a typed refusal. Equality with a caller-supplied digest never supplies
the proof. Raw scalar writes undergo the same Native check.

The legacy `AspectValue::Bytes(ContentRefId)` arm has no byte owner or production
consumer; the application/example searches find no use beyond scalar plumbing
and certification fixtures. K1e's landed InlineBytes supplies inline values.
Retire the legacy arm and its scalar/declaration/codecs in the identity slice;
do not leave an unchecked reference arm or replace it with another resolver.
If that slice discovers a genuine remaining consumer, it must instead route
the arm through the identical Native field-format and closure admission, with
the same refusal courts, before it can land. Nothing is deployed, so retirement
requires no legacy Bytes decoder. Explicitly inventory foundational identity
material, Native scalar codec, Query portable schema/archive, and contract
revisions before that slice.
Write new current artifact versions; refuse obsolete runtime content-reference
versions before effects. Preserve declared historical archive bytes as historical
data, never reinterpret their old u64 as a new digest. If a supported archive
contract requires conversion into current live content that its artifact cannot
supply, expose a typed unsupported-reference refusal and bring any required
change to that supported contract back for a ruling. Do not fabricate content or
rewrite frozen historical bytes to make compatibility tests pass.

## Format meaning and the exact numeric fixture

Native owns envelope integrity and runs the installed admission. A format owner
owns payload meaning, exact numerical encoding, dimensions, order, and any domain
certificate. Query installs the declaration but is not the numerical owner.
Define a Query-free Native format-admission port using foundational descriptors
and original Execution controls; the entry-band declaration lowers into this
port. No substrate imports a Query binding trait.

Payloads are identity-free, reference-free immutable values. A format must not
hide independently mutable objects or graph links inside bytes. Its binding
guard rejects such meaning; numerical bit patterns that happen to equal an id
are not references. Codec callbacks receive only their input/bytes and original
controls, with no ambient graph resolver, storage service, or fresh request.

The generic court format is `sample-grid`, version 1: u32 rows, u32 columns, then
exactly rows*columns signed i64 samples in row-major order, all little-endian.
Rows and columns must be nonzero; checked multiplication and exact payload
length reject overflow, truncation, surplus samples, and trailing bytes. The
signed integer encoding is fixed-width two's complement; no float, JSON, textual
number, implicit transpose, or approximation enters this format. Its header
occupies 8 bytes and each sample occupies 8 bytes.

This is a generic exact numeric format used to prove the door, not a platform
claim to admit every numerical family. Other formats must declare their own
versioned canonical representation and controlled validator. Rational formats
must preserve numerator/denominator exactly through the existing mathematical
owner; algebraic or certified formats retain their representation and certificate.
The current foundational CanonicalRational constructor alone is not a reduced
rational or geometry certificate. No new rational-normalization algorithm belongs
in this content owner.

Every write has three distinct steps: controlled encoding, canonical validation,
and an independent input-to-encoding correspondence check supplied by the format
owner. Native requires all three before issuing a staged attachment. A format
cannot omit the correspondence check or implement it by invoking its encoder
again. For the grid, the verifier reads the fixed dimensions and sample offsets
and compares them directly with the original immutable input, without using the
production encoder or decoder. Canonical but altered samples return the typed
internal-inconsistency outcome EncodingInconsistency. Raw bytes supplied without
a typed input still require canonical admission; they cannot claim certification
of a different supplied value. Reads/recovery validate the canonical frame and
identity; they do not reconstruct an original input or rerun write correspondence.

The court constructs fixture frames with literal byte layout and derives grid
consequences with integer indexing, without the production encoder, validator,
or verifier. It installs both a noncanonical encoder and an encoder that alters
one sample while emitting valid dimensions, length, and hash. Native must refuse
both before attachment/publication and release the original-funded scratch.
Complex numerical formats require a checkable domain certificate or independent
correspondence algorithm; hashing an opaque numerical answer cannot certify it.

## Declared Source dependency and retained evidence

Extend the installed Source field contract with its content format binding. The
compiler derives a content read from that projected field; providers cannot fetch
by arbitrary digest. This is an extension to scalar-field interpretation, not a
new nested/list/variant AspectShape or a new provider-maintained Source roster.

Acquisition records the existing full-generation occurrence, actual field locator,
ordinary field/presence revision, installed Source program/cell/scope/audience,
and Native-admitted immutable content identity and custody. Add the content fact
to the existing ViewDependency/observed-fact and complete projected-content owners.
Retain complete support for exact rejects and negative coverage as Phases 4/5 do.
A Source projection that requests content must admit it before provider or exact
geometry callbacks run; unsupported profiles refuse rather than omit support.

Changing reference A to reference B is an ordinary authored field write. Native
pulls it back through the installed Source program, publishes the occurrence
change to the existing managed Source, and reaches the consuming candidate even
when all other row scalars and coarse candidate keys are unchanged. There is no
mutable byte-update operation behind A, hence no second content-change event bus.
Content GC or cold placement changes do not change semantic Source identity.
Missing or damaged backing is a read failure, never an empty value or an unchanged
candidate. Actual field revisions remain independent obligations.

Correction compares genuine original and target support using the existing
authored-history, performed-cancellation, and ancestry requirements. A->B->A
does not become an authored no-op because bytes or digests agree. Both-image
Source material must retain the associated admitted frames while it exists.
Undo and reapplication restore the actual earlier reference and its retained
content, through the same transaction and admission, not by re-encoding an old
rendered value. Merge closes over content reachable from both actual parents and
refuses any same-identity/body disagreement before selecting fields.

Recovery restores frames, field associations, historical reachability, and format
descriptors from Native checkpoint plus journal tail, validates them before root
selection, then rebuilds disposable lookup and managed index state. The installed
current application supplies the matching codec; an unavailable codec refuses
before live use. Historical raw bytes stay interpretable by their recorded
descriptor and are not silently decoded using the latest version.
Restoring bytes does not recreate LiveManagedSourceHistory issuance. Existing
IncompleteManagedSourceHistory refusal remains when that live correction proof
is unavailable; equal retained content grants neither historical READ nor fresh
publication authority. No new replay door enters Host or Decl.

## Retention, funding, and cancellation

Bytes are collectable only when absent from every retained Native root/branch,
transaction/savepoint, snapshot, candidate, before image, history/undo/correction
obligation, and active completed read that can name them. A content reference in
a retained artifact is part of its reachability closure; deleting the current
field is insufficient. Persisted collection follows existing journal/checkpoint
retirement and selected-member custody, not an independent content sweep which
can strand a retained patch. Historical expiration retains its typed gap posture.

Use original Execution reservations, following [reserve_retained_memory](../../crates/worth-execution/src/authority/request/retained_memory.rs).
The staging writer funds frame, encoding scratch, directory paths, and admission
metadata before allocation. Move reservations with the actual immutable backing;
aliases share that reservation until final release. Per-reader decode scratch
and newly retained evidence are funded by the reader's original request. Where
an existing owner conservatively funds each complete view, preserve that rule;
do not claim global deduplication savings without changing its actual accounting.
Committed retention keeps its original backing reservation; checkpoint/recovery
allocations use that operation's admitted request and never resurrect a spent
reservation. Physical durable capacity remains Store's accounting responsibility.

Poll original cancellation/deadline before reserve, during each bounded codec,
hash, copy, and comparison chunk, and at final handoff. The control preserves the
first actual stop; a callback swallowing it cannot turn final handoff into success.
A failed/cancelled write
returns no attachment or publishable program, drops its staged closure/scratch,
and releases its original reservations when the actual last staged owner drops.
A failed read returns no usable lease and releases its acquisition scratch;
retained committed bytes and independent readers keep their lawful custody.
Cancellation cannot collect content still reachable from committed history.
Drop performs bounded local release; funded maintenance settles deferred directory
or physical retirement work. No fresh request, hidden worker, or unbounded GC scan
is permitted on cancellation.

## Bounds, charged work, and refusal ordering

Proposed K1c ceilings are 16 MiB per complete canonical content frame and 64 MiB
of newly staged frame bytes per transaction, including duplicates before reuse
is established. Each installed format may lower the frame ceiling. Identifiers
are at most 256 ASCII bytes; versions are nonzero u32. These are new proposed
bounds, not measurements or existing constants. The grid limit is derived from
the complete frame ceiling, not merely its sample payload.

ContentRef is the ordinary representation for ordered identity-free numerical
data at every size. There is no small-value inline or bounded-canonical fallback.
The consuming spline contract permits up to 1,048,576 controls of four exact
rationals each; that count does not imply the encoded frame fits 16 MiB. Above
the frame ceiling, admission returns typed SizeLimit before attachment, geometry,
index publication, or durable effects. The caller must reduce the input or seek
an explicitly revised content bound; no silent truncation, segmentation scheme,
lossy numeric conversion, or alternate storage representation is selected.

The complete `RelationalCanonicalCommitArtifact::as_bytes()` also has a 256 MiB
ceiling, including its 66-byte outer frame and all ordinary field, content, and
nested encoding material, matching the current Store adapter's
[journal recovery limit](../../workspaces/worth-store/crates/worth-store-relational/src/recovered_history.rs).
Quote the actual enclosing encoding before allocation or publication; exceeding
any enclosing artifact/checkpoint/member limit refuses before effects. A small
per-item limit never proves an aggregate artifact fits. Checkpoint construction
must obey the existing selected-manifest/member limits; split content across
bounded members rather than embedding an unbounded table in the manifest.

All traversals use the existing request work account, including
[Native preparation](../../crates/worth-relational/src/execution/request_work.rs).
Charge actual visited directory nodes/key bytes, inspected/encoded/hashed/copied
bytes, codec steps, exact equality bytes, and retained-evidence work before each
bounded step. Enclosing artifact serialization and hashing remain separately
charged actual passes. Neither cache hits nor cancellation erase work already
performed. A format validator must consume the supplied original control and
admit every variable-width numerical step; it cannot hide an uncharged decoder
or create an independent budget.

Cold read/write cost is O(frame bytes + codec work + directory lookup/update);
same admitted backing can skip repeat integrity work inside that trust boundary.
Nonshared identity matches pay full equality. Warm reads still pay field/authority
checks and any newly performed decode/copy. No full content-table traversal is
on an ordinary field read or write. Cost observations stay internal or descriptive
test telemetry, never a Work token or cost parameter in the public content door.
Record leased and serial-accounted measurements separately.

Cheap limits, field authority, format/version eligibility, memory, and request
admission precede expensive codec work. Validation of incoming bytes and closure
precedes geometry/provider invocation, index construction from that material,
canonical publication, and root selection. Staged derived work is disposable;
no bad content may publish a geometry result or index. Failures remain typed:
MissingContent, FormatMismatch, UnsupportedVersion, NonCanonicalContent,
IntegrityMismatch, ContentIdentityConflict, EncodingInconsistency, SizeLimit, and original execution
stops (cancellation, deadline, work, memory, overflow) with original cause.
The public denial must preserve which boundary failed, not stringify it into an
ordinary provider rejection.

Query content field writes extend the one landed pre-publication owner at
`application_attempt/provider_execution/phase/prepared/domain_value_write.rs`.
Its current domain-value predicate must also reach content field contracts;
missing/wrong-format/wrong-version content refuses there and preserves the
selected commit. Do not add a parallel content publication validator. Read
corruption courts flip a stored byte, swap to another format's reference, and
remove retained backing; each must yield its typed refusal and no plausible
value. Their digest oracle hashes literal frame bytes independently of the
production writer.

## Smallest Decl and Host door

All signatures below are proposed API contracts, not compiling examples today.
Use the existing facade modules; add no runtime or store constructor.

Decl adds `application_schema::ApplicationContentBinding`: named format identity,
version, maximum frame bytes, and controlled encode, validate, and verify_encoding
functions. `encode(&Value, sink, control)` emits bounded bytes;
`validate(bytes, control)` checks canonical payload meaning;
`verify_encoding(&Value, bytes, control)` independently checks input correspondence.
The encoder writes only through a framework-owned bounded sink; Native invokes
both checks before attachment. All callbacks receive original controlled admission
and return typed outcomes, without returning a work certificate. Installation
lowers them into the Query-free Native format port. A second registration of an
identity/version refuses as DuplicateContentFormat, even if its function pointers
match. Multiple fields reuse the same already-admitted declaration/installed
handle. No callback equality or semantic-equivalence decision is inferred.

Decl adds `application_schema::ApplicationContentFieldRef<..., Format>` and
`content_field(field, format_binding)`, accepting only an existing ContentRef
scalar field reference. The wrapper is a typed declaration of the installed
field-format contract, not an alternate field identity. Its recipe is part of
the schema, portable archive, and Source contract. Required/Optional posture,
existing OperationReads/OperationWrites and field authority remain in force.
The application registers this recipe once in its schema contribution; runtime
callers cannot supply a different format to reinterpret the same field.

Host exposes three field operations through existing owners:

```text
DecisionReader::content(identity, content_field)
    -> Result<Option<AdmittedContent<Format>>, ContentReadDenial>
CandidateWriter::initialize_content(target, content_field, &Format::Value)
    -> Result<(), ContentWriteDenial>
CandidateWriter::write_content(target, content_field, &Format::Value)
    -> Result<(), ContentWriteDenial>
```

The read's None means genuine optional field absence only. A present reference
whose content is missing is always Err. Required field absence retains the
existing required-field refusal. Creation/write bounds mirror initialize_field
and write_field. The writes stage admission plus the reference field atomically;
there is no freestanding put returning a reusable mutation permit. Removal and
clearing use existing declared field/aspect lifecycle and Native closure checks.

For the existing managed provider receiver add
`WorthQueryManagedIndexSourceRow::with_content(content_result_selector, callback)`.
The selector is the Source projection of that same content field contract. It
lends an `AdmittedContentView<Format>` only for the controlled invocation, using
the acquired Source row's request and support. Ordinary projected/invariant reads
delegate to the same Native acquisition, including staged candidate reads.
Completed-Source and Native-maintained postures both retain that admitted support.

AdmittedContent is a read-only Native custody handle, not a digest constructor.
It exposes the checked descriptor and immutable canonical payload bytes; decoding
or geometry beyond format admission still uses its consumer's original controls.
It carries no publication, historical READ, or mutation permission. Proof-bearing
admission legality belongs in worth-proof with an owner-bound concrete issuer;
Native owns runtime leases/reservations and the exact owner association. A foreign
issuer cannot satisfy this runtime's field read. Public facades only re-export
these roles. Add positive/negative compile courts for field/format mismatch,
undeclared operation access, forged admission, and callback escape.

Example intended use is `writer.write_content(target, Samples::content(), &grid)`
followed, in a consuming operation, by
`reader.content(identity, Samples::content())?`. The managed provider obtains
the same grid bytes with `row.with_content(Source::samples(), callback)?`.
No caller handles a Store address, supplies a second content cache, or obtains
bytes from an arbitrary ContentRefId. A digest is descriptive data, not a secret
or an access capability.

## Destination topology and ordered implementation slices

The following locations are proposed, populated only as their slice lands:

| Owner | Responsibility and stable location |
|---|---|
| Foundational | `values/content/identity.rs` and `format.rs`: portable digest/frame descriptor, no lifecycle or Query dependency. Existing scalar/identity codecs consume this vocabulary. |
| Proof | `content/`: concrete owner-bound admission proof and issuer association, no byte store or generic AuthorityMarker bound. |
| Native | `content/{admission,staging,read,retention,reconstruction}/`: format eligibility, candidate attachments, immutable borrowing, reachable directory, and readmission respectively. Facade: `facade::content`. Branch, transaction, canonical envelope, and checkpoint owners carry these parts explicitly. |
| Decl/Installation | `application_schema/content/` for portable field/format declarations; corresponding installed catalog lowers into Native's port. No Query import in Foundational or Native. |
| Query execution | `handler/candidate/content.rs`, `handler/decision_reads/content.rs`, and managed Source content acquisition under its existing source/projection owner. Existing retained-content comparison and history consumers gain the native content support; there is no Query byte-store module. |
| Store adapter | Existing physical adapter only; integration and its courts are outside K1c. Preserve only the two Bytes-deletion compile fixes. No Native -> Store dependency or application codec inside Store. |

Never place these responsibilities in helpers/common, a second cache service, a
test-only runtime, or the public facade implementation. All new code/test files
stay at or below 400 lines; any touched oversized owner must be split within its
responsibility under the repository rule. Do not hand-edit AGENT_CONTEXT files.

Each row is a separately useful, building slice, independently reviewed and
committed. Fetch/merge both required sync branches before each implementation
slice. No partial public byte door is exposed before its closure is enforced.

| Slice | Change and court that lands with it |
|---|---|
| 1. Native transactional authority and identity | Cut over the descriptive identity and current codecs; add format port, independent encoding-correspondence check, staged frames, field contract, Native read/attach, root/envelope closure, savepoint/refusal cleanup. Extend the existing `worth-court-relational-certification` binary with a `content::transaction` family: valid grid, equal-frame identity, format/version mismatch, missing/corrupt bytes, a canonical altered-sample encoder refused as EncodingInconsistency with no attachment/publication and released original scratch, forced identity collision, foreign owner, raw write bypass, concurrent same-frame admission, rollback, and original-work/cancellation release. Literal frame bytes and a separate integer-grid interpreter are independent oracles. No Host byte API yet. |
| 2. Native durable closure and retention | Include frames/association in canonical journal and bounded checkpoint members, recover before root selection, and preserve funded branch/history/undo reachability. Native certification compares literal frames and selected fields after checkpoint plus journal-tail reopen, replacement checkpoint selection and actual admitted historical-reader retention across runtime destruction. Existing Native corruption courts refuse altered/omitted closure before service. Store content courts are withdrawn: Store home publication, filesystem retention and physical fate integration are outside this lane. |
| 3. Public field-bound door | Add the exact Decl/Host surface and portable field/format contract; install the grid binding through ordinary contribution/configuration, and document it in the capability index. Use `worth-query-court-decl` for binding admission/portable mismatch and duplicate registration versus lawful shared-handle reuse; use `worth-query-court-decl-host` for a facade-only generic operation fixture exercising create, read, replace, clear, typed refusals, original budget/cancellation and final custody release. Compile-contract cases include their valid counterparts and share the existing compiler session. |
| 4. Source and correction integration | Extend complete retained Source support, both read postures, history material and expectation comparison. In the same `worth-query-court-decl-host` generic grid fixture, change only A->B behind a field and observe the actual consuming candidate change through Source; hold coarse keys constant, include an exact reject and an absent entrant, and compare results to a literal full scan. Exercise candidate rejection before geometry/index publication for missing/wrong-version/corrupt content, A->B->A correction cancellation, undo, retained history, recovery/rebuild and explicit recovered-live-history refusal. Corrupt encoder/summary/identity evidence must be caught by independent checks or the full-scan oracle, never yield a plausible result. |
| 5. Bounded-cost qualification and public closeout | Put scale tests in the owning court's `cost` module. Measure original read/write invoices, retained and peak memory, codec/hash/comparison byte visits and timing for 1/2/4 MiB frames and fixed local edits with 100/1000/10000 unrelated content entries. Require generous linear frame growth and logarithmic directory-local work, plus warm shared-backing versus cold reconstruction separation; record timing as telemetry, not a machine-speed merge gate. At each bound inject cancellation/work/memory exhaustion and prove selected field/geometry/index state and reservation fate. The independent integer full scan remains the result oracle. Map both K1c table rows to these actual courts and record measured counts only after running them. |

Slice 1 must not enable durable content writes until slice 2 supplies complete
reconstruction: the Native durable posture returns a named ContentDurabilityNotReady
refusal, never a successful artifact missing bytes. This is an explicit temporary
capability refusal on the existing publication owner, removed in slice 2; it is
not a second ordinary storage path. Likewise Source content recipes remain
unsupported with a typed installation refusal until slice 4 installs their full
support. Earlier slice successes do not claim K1c completion.

Future implementation gates remain the user's package-only tests and exact
footer checks, with Query's manifest path and affected test binary filters;
long cargo commands run in the background to logs. No workspace build/test is
authorized. This documentation slice runs only `git diff --check`.

## Readings chosen and remaining decision boundaries

- Establishing Native authority is authorized; the earlier brief's assumption
  that the owner already existed is superseded. Existing durable ports are the
  backing seam, not an existing content resolver.
- Immutable identity means complete framed content, including format/version;
  admission and access still require the genuine field/root/owner. No digest
  equality is promoted into exact retained-content or correction evidence.
- Exact encoding belongs to the declared format owner. The generic integer
  grid proves the public mechanism without exporting an application numerical
  model or silently converting nonrational numbers.
- Current-format-only Native evolution writes no migration. Historical archives
  keep their declared data contracts; an unsupported live conversion refuses
  explicitly and any incompatible public promise requires a ruling.
- Existing Store journal publication uses its own physical deadline and a copied
  byte input; that port does not promise hard preemption by Query cancellation.
  K1c must check the original request before the irreversible handoff, retain any
  accepted publication fate, and never claim cancellation undid accepted media
  effects. Stronger asynchronous Store interruption would require a separately
  reviewed extension of the existing port, not a direct blob-runtime dependency.

No implementation, runtime test, measured performance result, or K1c acceptance
is established by this document. The next action after user review is slice 1's
focused implementation plan and identity/compatibility inventory, not the public
door in isolation.
