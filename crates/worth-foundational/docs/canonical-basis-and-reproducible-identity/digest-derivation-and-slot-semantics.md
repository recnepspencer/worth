# Digest Derivation And Slot Semantics

## What This Feature Is

This feature derives digest output from ready canonical inputs. It makes the
input shape, algorithm slot, and derivation readiness explicit so digest output
stays downstream of admitted canonical meaning.

## Why You Use It

- Use this when you need a compact digest for a ready sequence, bundle, or
  export artifact.
- Use this when algorithm choice and input shape should be explicit instead of
  ambient.
- Use this when you want the digest lane to stay honest about what it is
  compressing.

## Stable Entry Points

Common path:

- `canonicalization().digest().for_sequence(...)`
- `canonicalization().digest().for_bundle(...)`
- `canonicalization().digest().for_export(...)`
- `canonicalization().digest().derive(...)`

Lower lane:

- `admit_canonical_sequence_digest_derivation(...)`
- `admit_canonical_bundle_digest_derivation(...)`
- `admit_canonical_export_digest_derivation(...)`
- `derive_canonical_digest(...)`
- `CanonicalDigestDerivationReadyArtifact`
- `CanonicalDerivedDigest`
- `CanonicalDigestAlgorithmId`
- `CanonicalDigestAlgorithmSlot`
- `CanonicalDigestInputShape`
- `CanonicalDigestDerivationDenial`

Good to know:

- `canonicalization_api::common_path` is the recommended grouped public lane.
- `canonicalization_api::lower_lane::digest` is the inspectable lower lane.
- digest is derived compression, not semantic authority.

## Core Mental Model

The digest lane answers one question: given a ready canonical artifact, what
derived digest do we compute from it under this algorithm slot?

It does not answer:

- whether the artifact was semantically valid to begin with
- whether two digests are enough to replace canonical comparison
- whether the digest itself becomes the source of truth

That is why the common path starts from already-ready canonical artifacts.

## How It Executes

The normal flow is:

1. start from a ready basis sequence, ready bundle, or ready export artifact
2. choose the digest algorithm id explicitly
3. admit digest derivation for that exact input shape
4. derive the digest from the ready derivation artifact

The front door builds the slot metadata for the common path so the first
contact API stays authority-first instead of slot-first.

The hostile proof bar here is intentionally strict. Wrong domain, wrong input
shape, wrong version, or raw-byte substitution must fail closed before a digest
can be derived.

## Small Example

```rust
use worth_foundational::{canonicalization, CanonicalDigestAlgorithmId};
use worth_proof::TransitionOutcome;

let ready = match canonicalization()
    .digest()
    .for_sequence(sequence_ready, CanonicalDigestAlgorithmId::sha256())
{
    TransitionOutcome::Success(ready) => ready,
    other => return Err(format!("digest admission failed: {other:?}").into()),
};

let digest = canonicalization().digest().derive(ready);
```

This is the smallest honest example because digest derivation begins only after
canonical readiness already exists.

## Real Example

```rust
use worth_foundational::{canonicalization, CanonicalDigestAlgorithmId};
use worth_proof::TransitionOutcome;

let ready = match canonicalization()
    .digest()
    .for_export(export_ready, CanonicalDigestAlgorithmId::sha256())
{
    TransitionOutcome::Success(ready) => ready,
    other => return Err(format!("export digest admission failed: {other:?}").into()),
};

let derived = canonicalization().digest().derive(ready);

println!("algorithm: {:?}", derived.algorithm_id());
println!("value: {:?}", derived.value());
```

What is authoritative here is still the ready export artifact. The digest is a
derived summary of that artifact, not a replacement for it.

## How It Relates To Other Features

- [Canonical Basis Sequences And Entry Grammar](./canonical-basis-sequences-and-entry-grammar.md)
  explains where ready canonical inputs come from.
- [Export Bundles And Producer Shape](./export-bundles-and-producer-shape.md)
  explains how export artifacts become valid digest inputs.
- [Canonical Production Readiness](./canonical-production-readiness.md)
  freezes digest-slot hostility, readiness gating, and grouped surface
  inventory as shipped behavior.

## Inspection And Debugging

Inspect these first:

- `canonicalization_api::lower_lane::digest` when you need exact slot and input
  vocabulary
- `CanonicalDigestDerivationDenial` when derivation admission fails
- the input shape and algorithm id when two digests differ unexpectedly
- the upstream ready artifact when a digest feels semantically wrong

If digest output surprises you, check the input artifact and algorithm slot
before questioning canonical basis law.

## Anti-Patterns

- Do not derive digests from raw, not-ready canonical inputs.
- Do not compare digests as a substitute for structured canonical comparison.
- Do not treat digest output as the semantic authority lane.

## Current Limits

- `CanonicalDigestWorkBudget::for_encoded_byte_ceiling(bytes)` derives a finite
  entry ceiling from the encoder's guaranteed minimum of 30 bytes per entry.
  Owners can reject impossible entry breadth before sorting or retaining a
  basis without rejecting a sequence that fits that byte ceiling. Digest
  admission still checks exact encoded bytes. This bounds entry cardinality;
  it does not claim that basis heap allocation equals encoded material size.
- `CanonicalDigestAlgorithmId::sha256()` is the only admitted digest algorithm.
  Unsupported algorithm identifiers fail before derivation.
- This layer standardizes derivation from ready canonical inputs. Owning
  runtimes still decide which semantic families require a compact key and keep
  their stronger authority artifacts.

## Related Docs

- [Export Bundles And Producer Shape](./export-bundles-and-producer-shape.md)
- [Canonical Production Readiness](./canonical-production-readiness.md)
