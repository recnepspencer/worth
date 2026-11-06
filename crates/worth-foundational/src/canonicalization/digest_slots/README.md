# Canonical digest policy v2

The supported policy identifier is `sha256-canonical-v2`. SHA-256 itself is
unchanged. Canonical sequence material now carries material grammar version 2
and commits only to the source rule version, domain, loci, entry kinds and values.
Preparation costs remain evidence and never enter identity material.

The former `sha256` policy is unsupported. Admission returns the typed
`UnsupportedAlgorithm` denial for it. Never relabel a v1 digest as v2 or compare
its bytes as though it were current. At an upgrade trust boundary, re-derive the
identity from its original canonical basis and record the new policy identity;
if that basis is unavailable, refuse the upgrade. Descriptive Query and Bridge
identity tokens include the policy identifier, so retained old tokens remain
distinguishable. The source rule version is independent of this material policy.

Digest admission moves owned evidence and streams the same versioned material
grammar used by the buffered oracle into SHA-256. Its entry and encoded-byte
limits return typed refusals. No partially admitted digest is emitted.
