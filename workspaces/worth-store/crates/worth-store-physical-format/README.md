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

`PhysicalArtifactReadTarget` and `PhysicalArtifactReadRange` describe bounded
locations, not read authority. Checkpoint identities share the mutable
`checkpoint.current` location, so overlap checks compare location as well as
expected content identity. Live acquisition stays in C.4 and Store.

The former `offline_walk` I/O/classification surface is deleted. Legacy backup
and structural-inspection duties live under their actual offline-verifier
owners, not beside immutable format meaning.
