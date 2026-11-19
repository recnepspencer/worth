# Current package-archive known-answer vectors

The reader is current-only, so this testkit freezes current bytes only. There
are no predecessor fixtures. A non-current version is refused with a typed
compatibility denial, checked once per layer: the shared-decision-reads court
for the previous record version, the oracle manifest court for the previous
manifest version, and the protocol-compatibility court for zero and future
versions of every layer.

## Record protocol 21, derived from 19

Protocols 20 and 21 changed only the relation grammar of application-query
result shapes (`record/application_query/result_shape.rs`). No frozen family
carries a result shape: the package roots, entity-only schema, conditional
binding, native aspect, artifact contract, domain operation and complete
operation contract. Their payloads are unchanged.

The protocol-21 vectors were derived from the committed protocol-19 vectors
without calling the production encoder. A Python script walked each record
frame by its declared payload length and replaced only the two-byte frame
version. Each derived vector differs from its predecessor as follows.

- Single record frames: byte 1 of each frame, 0x13 to 0x15.
- Minimal archive (164 bytes, one record at byte 118): byte 119.
- Release envelope (586 bytes): byte 473, which is byte 119 of the archive
  nested at byte 354, and the archive SHA-256 at bytes 50–81. `hashlib.sha256`
  over the patched archive gives
  `ddd37a84910b2b0214919448d955fda37042d2b622eff71d37bfa7c64277bd26`. The
  dummy signature bytes are unchanged and claim no real signature.

The courts require the production encoder to reproduce every vector exactly,
then decode each vector back to its complete typed record.

## Manifest 2

The manifest frame vector differs from its manifest-1 form only at byte 15,
the low byte of the manifest version. The frame layout did not change.

## Frames without a frozen version

The content-schema and document-history courts freeze a hand-written family,
index, length and payload, and prepend the current record version. They test
payload meaning, and the vectors above already pin the version bytes.
