# Record-frame 16 vector provenance

Milestone 9.18.1 Phase 4 adds an explicit orbit flag to relation integrity.
The exact-current record grammar advances from 15 to 16. Operation-body
profiles are separate and retain their existing versions.

The version-16 vectors introduced here are independent derivations of existing
payload-unchanged vectors, not historical captures or output blessed from the
current encoder. Existing version-14 integration positives already lagged the
version-15 record owner. All retained historical bytes remain unchanged and
now participate in unsupported-version checks.

For operation-contract, domain-operation, artifact, native-aspect, conditional
binding, entity-only schema and package-root records, the derivation replaces
only the first big-endian u16 with 16. Family, canonical index, payload length
and payload bytes remain identical. These selected schema fixtures contain
only the entity declaration, so they contain no changed relation-integrity
payload. The other selected record families do not encode relation integrity.

For `archive_v2_record_v16.hex`, keep the manifest and archive2 framing. Starting
after its 118-byte manifest frame, read each 12-byte record header, replace its
version with 16 and advance by its explicit payload length. The minimal archive
contains one record. For `release_envelope_v1_record_v16.hex`, replace that sole
embedded archive and recompute its SHA256 checksum. Preserve envelope1 framing,
package identity, metadata, lengths and the existing dummy signature bytes.
This vector does not claim a real signature over the changed payload.

The ordinary encoder must match every new frozen literal exactly. The same
tests decode the current vectors, reconstruct real exported records and
re-enter fresh package validation. Old vectors must refuse at the exact-current
record boundary; changing a header does not grant a compatibility path.
