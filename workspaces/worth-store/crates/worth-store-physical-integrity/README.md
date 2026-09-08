# worth-store-physical-integrity

Owns C.9's pure runtime-side physical validation vocabulary: bounded untrusted
artifact inputs, exact scopes, sealed family-validation results, rejection and
localization descriptions, scrub windows, quarantine observations, and
counters. Store and recovery own live admission and lifecycle; artifact owners
own authority, rebuildability, repair, and mutation decisions.

Checksums prove physical integrity, not authenticity. A validation result is
descriptive and cannot open a semantic decoder without an owner-private binding
to the exact live source incarnation.

## Caller surfaces

Family `validate_*` functions accept `UntrustedPhysicalArtifact` and a concrete
`PhysicalArtifactScope`, and return sealed family results plus exact counters.
They preserve damaged, unsupported, unknown, and indeterminate distinctions.
Parent-bound validators require the matching admitted descriptive parent; an
envelope checksum alone never substitutes for membership or aggregate checks.

`PhysicalIntegrityScrubValidator` interprets bounded windows with constant-space
extent/checkpoint context. It owns no files, scheduler, live allocation, clock,
worker, cancellation, or close operation. Those belong to Store's
`ManagedPhysicalIntegrityScrubHandle`. Quarantine is descriptive only.

There are no generic evidence bundles, repair/recovery authorities, operational
repair exports, or offline classifier in this facade. The independent observer
imports format declarations, never this implementation. Runtime/recovery owners
consume sealed validation through their own source-bound admission; serialized
foundational projections cannot be promoted back into that authority.

See the [caller/operator guide](../../../../_docs/worth-store/physical-integrity-and-offline-verification.md)
for the actual managed API, source-change behavior, protocol, and boundaries.
