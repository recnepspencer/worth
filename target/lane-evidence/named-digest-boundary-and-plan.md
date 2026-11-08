# Named checkpoint admission digests

D13 LANDED(track H):14c4040669, accepted by coordinator. Retirement census is
CLEANUP/C2; leave it unchanged and red. Current clean source base15c56da531.

Boundary: Foundation CanonicalDigestId already defines the32-byte digest meaning
and strict binary Serde. Its canonical_digest_wire.rs owns encoding. Native's
BinaryFixedArrays forwards serializer mode and already emits fixed arrays as bin,
so replacing a stored field with CanonicalDigestId preserves the existing bytes.
No authority is minted by this change; image/source/refusal checks remain owned.

Convert all actual projected-coordinate/schema/visibility/reference/root-directory
producer and consumer digest fields, capture image's checkpoint/source/origin
fields, prepared canonical commitment leaves, and the owned schema/carrier and
region-capture maps. The latter keep the same32-byte keys/values, ordering,
reservation lifetimes and charge order. Use explicit bytes at existing raw API
boundaries. No new type, feature, macro, blanket impl, codec, migration or reader.

Reviewed retained seam: source.rs::OriginalCanonicalCommit.original_payload_digest
is original-frame source DATA assigned directly by P-C-owned original_image.rs:79
to CanonicalOriginalRecord.payload_digest. Leave this carrier and its protected
consumer untouched; do not claim every digest under checkpoint_complete is named.
P-C also owns original_image.rs,branches.rs,branches/. Existing public Foundation
CheckpointAdmissionCut/Scope arrays and raw parameters remain their owners'.

Files: checkpoint_admission/admitted_commit.rs; checkpoint_complete/admission_projection.rs
and its row/reference/root_directory leaves; schemas.rs,verified_originals.rs,
covered_prefix.rs,regions.rs,regions/{grouping,covered,emission}.rs,
interim/verified/projection.rs; durability/data/checkpoint_schema_images.rs;
canonical_commit_artifact/prepared_payload.rs; pure binary-encoding tests.
The old cited canonical_commit_artifact/readmission.rs:99 has no digest field on
this integrated tree (source read and rg inventory); do not invent another one.

Implement the complete producer/consumer family before compilation. Verify same
binary width and bytes with pure value tests against the preexisting fixed-array
encoding; run real image capture, authentication/binding refusals,selected source
recovery and recapture courts together. Use one Root scoped check/Clippy and one
owned selected build/run packet. No public signature changes; Query/Store/D13
inputs keep their exact wire and constructor contracts. Phase gates at completion,
fresh independent review and coherent commit. Never run ignored,scale,Home oracle
or scenario suites. The original-frame protected seam is explicit in the report.

Independent boundary review named_digest_boundary_review: proceed with this
bounded scope, disclose the protected source carrier. No builds/tests by reviewer.
