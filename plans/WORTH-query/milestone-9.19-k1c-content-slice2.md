# K1c slice 2: durable closure and retention

Current qualification is recorded in the [resumed repair ledger](milestone-9.19-k1c-certification-evidence.md).
This document preserves historical slice receipts. Its Store content courts and
Store qualification claims are withdrawn; K1c has no integrated Store content
court. Combined checkpoint/tail/replacement/retained-reader proof now belongs to
Native certification. Earlier filtered runs do not qualify the full binaries.

This is the implementation-batch boundary review and plan for the second accepted
slice. Slice 1 and the subsequent sync merge are committed. The governing content
boundary and the CAD integration plan remain unchanged.

## Boundary review from the current code

Native's transaction attachments contain the actual funded frame backing. The
current root directory retains every admitted frame strongly, including frames
with no remaining record reference. Its canonical envelope carries only the
transaction's new records. These are adequate for the explicitly limited slice 1;
they are insufficient for retention, checkpoint replacement and cold recovery.

`ContentPublication` reaches both ordinary and retained-selection application
before Source/invariants and the final publication owner after consequences.
Retained selection also retains its actual historical Native basis. A content
resolver must use that basis when restoring its fields; a digest alone cannot
substitute for historical custody. Existing raw retained-selection tail replay
refuses without that custody, and this slice preserves that explicit refusal.

The current canonical artifact decoder derives `CanonicalEnvelopeWire` before it
has admitted content allocations. Its content records allocate owned byte arrays.
Production content decoding therefore needs an original-request seed at that
existing wire owner. The existing canonical re-emission comparison independently
checks the decoded envelope. It must remain authoritative and reject disagreement.
The framed-content digest is rechecked from actual incoming bytes independently
of the writer; configured codecs subsequently validate canonical payload meaning.

Current checkpoints select distinct branch roots, schema carriers, a journal
index and physical members through Native's existing cold-source boundary. Store
already publishes those members atomically with the manifest and retains selected
member placement for readers across replacement. No Native-to-Store dependency is
needed. Both Native and the Store adapter already depend on worth-execution.
`RelationalCheckpointMember::New` currently carries `Arc<[u8]>`, which cannot carry
an original allocation reservation. Its existing byte field will become
`ExecutionImmutableBytes`; this is the existing SDK custody type, not a new byte
wrapper, cache, member variant or physical store.

Schema carriers currently omit content format declarations. Portable descriptor,
ceiling and field-alias data must enter the same carrier digest and optional tail.
Recorded declarations are descriptive only. They cannot produce an installed
format handle. Exact recovery binds them to the configured frozen Native codecs
before any content-bearing root becomes usable. Missing or different installed
format/version/ceiling data refuses; current branch meaning cannot silently change.

The temporary slice 1 durable/readmission/schema-replacement fences are removed
only after the complete replacement paths below exist. Untrusted roots must check
actual visible field references against actual admitted content before root or
index installation. Cold recovery may perform an explicitly reconstructive scan
of content-bearing roots under the original request. Ordinary edits keep local,
paid persistent-map paths; they do not scan unrelated content or branch roots.

## Implementation plan

1. Replace strong ever-growing identity retention with Native record/content
   associations. The ordered association key is `(ContentRefId, Option<RecordRef>)`;
   inserted rows always have `Some(record)`, and `None` supplies the lower bound for
   an identity lookup. A reverse record-to-identity index supports local replacement
   and deletion. Both use the existing original-funded persistent ordered map.
   Actual association rows retain admitted frames; no hand-maintained reference
   count, weak byte cache or independent retention ledger decides collection.
   Old roots, Source supports and retained history keep their own shared custody.
   Unreferenced and duplicate staged records are excluded from final canonical
   publication. Freed backing releases the original reservation when its last
   genuine owner drops.
2. Keep a preview closure before Source and rebuild the final associations from
   actual post-consequence record state through the same owner. Use current Native
   schema declarations, not a stale stored contract, to select field formats.
   Ordinary edits update changed records. Schema replacement and reconstruction
   enumerate the affected complete root under original controls. Retained selection
   may resolve through its genuine source basis; ordinary writes acquire no such
   historical permission.
3. Add controlled record/attachment deserialize seeds to the existing canonical
   envelope wire owner. Borrow binary frame bytes until their size, metadata and
   original allocation have been admitted; copy/hash in bounded chunks. Preserve
   typed content denials outside serde's error string, and keep original execution
   stops authoritative. Both journal and embedded checkpoint envelopes use this
   ingress. General descriptive deserialization grants no content authority.
4. Carry portable content declarations in schema carriers, with compact-tail
   roundtrip courts. Bind actual configured codecs before runtime root admission.
   Add content member locators to the existing root checkpoint directory; members
   carry immutable SDK byte custody. Native rechecks member/frame integrity, expected
   identity, format/version and visible field closure before root/index installation.
   Journal replay stages its controlled content records through the same transaction
   owner. Relinking an already admitted root preserves genuine custody and checks
   the actual canonical records. Complete canonical artifacts refuse above 256 MiB,
   including the 66-byte outer header, before publication.
5. Extend the generic Native court with durable tail/checkpoint reconstruction,
   retained-reference restoration, deletion/last-owner release, cancellation and
   corruption. Extend the existing Store relational court with real home reopen,
   checkpoint replacement while a reader retains custody, missing/modified members,
   and publication fate. Physical corruption and reference swaps must yield typed
   refusals before usable values or selected-state changes. The literal 75-byte
   frame, independent pinned digest and integer-grid interpreter remain the result
   oracles. No disposable index participates in reconstructing content truth.

The public Decl/Host and Source byte doors remain the next accepted slices. This
slice adds no AspectShape form and no generic Query execution engine. Verification
uses only affected Native/Store owner and court packages, with focused binaries and
filters, followed by the user's formatting, clippy, boundary, context, dirty line-cap
and diff checks. Long cargo commands run in background logs. The same reviewer gates
the completed slice before its commit; no implementation claim is made by this plan.

## Additional boundary evidence during implementation

`ExecutionRequest::allocation_policy` intentionally returns system allocation for
serial requests; using it for new retained member bytes would lose original serial
memory custody. Execution already provides the complete legal path:
`ExecutionRequestArrayBuilder<u8>::allocate`, `extend_from_slice`, `seal`, and
`ExecutionArray<u8>::into_immutable_bytes`. Native uses that path for new content
members. No new SDK constructor, reservation-adoption API or Native byte wrapper
is required. Installed codecs are rebound before actual content-bearing roots are
selected; descriptive carrier validation alone cannot resolve a format handle.

## Readings chosen while closing original controls

The original request funds the complete durable artifact and embedded checkpoint,
not only the canonical Native frame and member. The existing encoding owners now
retain SDK byte backing through count, emit, hashing, whole-frame construction and
final handoff. Captured envelopes use the same backing. Section accounting counts
without materializing payload copies. Explicit legacy policy-only construction is
compiled only for the existing Native unit codec profile; production selects the
genuine original-request case. No codec, field or checkpoint grammar is forked.

The existing cold journal port certifies no encoded length before reading. A cold head or carried
journal envelope therefore reserves the complete 256 MiB artifact read bound on the
original request before invoking that port, then funds the actual validated immutable
copy independently, then drops the physical temporary before decoding. A request that cannot fund this conservative temporary refuses
with its precise memory cause before reading; the implementation never guesses a
smaller length from current field values. Physical members do have certified locator
lengths and use that exact funded limit. This introduces no dependency or physical
backing API. Serial memory remains original-funded even when the legacy allocation
policy spells SystemAllocation.

Reusing bytes across a schema change requires the selected installed codec handle,
including its frozen ceiling and canonical validator. The complete root readmission
shortcut proves absence from both current declarations and every carried historical
contract. Retained older contracts cannot hide an actual unbound ContentRef value.
The raw retained-selection journal profile keeps its existing typed unsupported
refusal, while actual Native checkpoint reconstruction retains the positive path.
