# Canonical Basis And Reproducible Identity

This folder documents the Milestone 2 canonicalization surface in
`worth-foundational`.

Use these docs when you need to answer questions like:

- How do I turn a foundational surface into a canonical basis that other
  runtimes can reproduce?
- How do I compare two canonical forms without collapsing everything to a bool?
- How do I publish canonical fixtures or manifests for other producers and
  consumers?
- How do I derive a digest without treating the digest itself as semantic
  authority?
- Which public lane should I use first if I just want the supported path?

Read the docs in this order if you are new to the surface:

1. [Canonical Basis Sequences And Entry Grammar](./canonical-basis-sequences-and-entry-grammar.md)
2. [Equivalence And Mismatch Classification](./equivalence-and-mismatch-classification.md)
3. [Export Bundles And Producer Shape](./export-bundles-and-producer-shape.md)
4. [Digest Derivation And Slot Semantics](./digest-derivation-and-slot-semantics.md)
5. [Authority Identity Boundaries](./authority-identity-boundaries.md)
6. [Grouped Public Lanes And Front-Door Usage](./grouped-public-lanes-and-front-door-usage.md)
7. [Canonical Production Readiness](./canonical-production-readiness.md)

The order matters.

- Start with basis, because basis is the stable meaning surface.
- Compare basis before you publish bundles or manifests.
- Export only after you already hold ready canonical basis artifacts.
- Derive digests after readiness, not before it.
- Treat authority identity as witness-admitted, not as a string, digest, or
  projection label.
- Prefer the authority identity helper functions for the normal lifecycle.
  They reduce call-site ceremony without hiding the authority witness.
- Use the grouped public lanes when you want the teachable common path or the
  supported lower or stronger lanes.
- Use readiness when you need the exact machine-checkable closure contract for
  the milestone.

The grouped public surface for this milestone is part of what shipped:

- `worth_foundational::canonicalization_api::common_path`
- `worth_foundational::canonicalization_api::lower_lane::basis`
- `worth_foundational::canonicalization_api::lower_lane::comparison`
- `worth_foundational::canonicalization_api::lower_lane::export`
- `worth_foundational::canonicalization_api::lower_lane::digest`
- `worth_foundational::canonicalization_api::stronger_lane`
- `worth_foundational::canonicalization_api::stronger_lane::readiness`

These docs are capability-first on purpose. They are not milestone notes,
closeout notes, or test tours. If a canonicalization capability shipped, it
should have one stable home in this folder.

`CanonicalDigestId` is the descriptive 32-byte digest value. Its binary Serde
form is an exact byte string (MessagePack `bin`); its human-readable form is
64 hexadecimal digits. Both readers reject the retired integer-array form.
Wire length is independent of digest contents. These bytes grant no execution,
publication, currentness or recovery authority.
