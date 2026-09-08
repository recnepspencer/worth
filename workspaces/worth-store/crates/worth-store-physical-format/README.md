# worth-store-physical-format

Owns Roadmap 2 S.1 byte-format vocabulary: physical page ids, segment ids,
extent ids, generations, epochs, frame headers, slot directories, root
manifests, and physical references.

This crate defines physical byte containers and addressing. It must not decode
semantic artifacts or decide canonical truth.

## Integrity declarations and mechanisms

`integrity_declarations` is the stable declaration-only facade for artifact
families, supported format versions, checksum algorithms, fields, and covered
byte ranges. Writers and runtime validators use the canonical checksum
mechanism; the independent offline observer may import declarations only and
implements its own framing, field decoding, and checksums.

A new family starts with its named declaration under
`integrity_declarations/families/`: define its family/version, exact checksum
field and covered ranges, then add writer/runtime/independent-reader literal
vectors. Changing bytes or coverage requires explicit format-version and
compatibility handling; never silently widen an existing declaration.

For example, inspect an existing family declaration before defining its sibling:

```rust
use worth_store_physical_format::integrity_declarations::{
    families::PAGE_FRAME_INTEGRITY_DECLARATION,
    PhysicalIntegrityArtifactFamily,
};
let declaration = PAGE_FRAME_INTEGRITY_DECLARATION;
assert_eq!(declaration.family(), PhysicalIntegrityArtifactFamily::PageFrame);
assert_eq!(declaration.version().format_version(), 1);
assert_eq!(declaration.version().envelope_schema(), Some(2));
```

The frozen-family declaration tests exercise those exact version/coverage facts.

Metadata `project_payload` methods share the canonical payload grammar with
framed decoding but do not validate a frame checksum or grant decoder authority.
Store may consume persisted payloads only through its exact private,
source-bound admitted family views. This lets unchanged resident hits retain
their integrity evidence without rehashing. Framed `decode` remains mandatory
for untrusted validator input; the source-route guard rejects direct Store
calls, including calls inside generic admission callbacks.

`PhysicalArtifactReadTarget` and `PhysicalArtifactReadRange` describe bounded
locations, not read authority. Checkpoint identities share the mutable
`checkpoint.current` location, so overlap checks compare location as well as
expected content identity. Live acquisition stays in C.4 and Store.

The former `offline_walk` I/O/classification surface is deleted. Legacy backup
and structural-inspection duties live under their actual offline-verifier
owners, not beside immutable format meaning.
