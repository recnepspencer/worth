# worth-foundational

`worth-foundational` owns portable meaning shared across WORTH runtimes. Use it
for vocabulary that must remain identical when Query, Relational, Runtime
Bridge, Signal, Proof, or a downstream domain crosses a boundary.

It does not own runtime execution or stronger owner-specific authority.

## What It Owns

- aspect contracts, keys, opaque identities, revisions, masks, and bindings
- scalar and struct aspect values plus validation
- canonicalization, comparison, and digest preparation
- locators and boundary mismatch vocabulary
- evidence, provenance, lineage, support, and performance vocabulary shared by
  multiple owners

Performance work keeps authoritative observation distinct from authoritative
mutation. A read receipt must not claim that source truth advanced merely
because it crossed an execution boundary.

Use the facade:

```rust
use worth_foundational::facade::*;
```

## Aspect Meaning

`AspectContract` describes stable semantic shape. `AspectBinding` describes
where that meaning appears in authoritative graph-shaped truth.
`AspectMask<ProjectionMask>` selects the exact slice a consumer needs.
`AuthoritativeAspectChangeKind` describes the meaning of a committed change.

These are portable semantics. They must not contain a physical storage address,
a Query runtime identity, a Runtime Bridge mapping label, or a numeric Signal
aspect slot.

```rust
let binding = AspectBinding::EntityField {
    field: FieldKey::new("distance")?,
};

let relevant = [AuthoritativeAspectChangeKind::FieldSet];
```

Query uses these values to author semantic dependencies. Relational uses them
to publish authoritative change meaning. Runtime Bridge uses them to admit an
installed correspondence. Signal keeps its local aspect allocation separate.

## Canonicalization

When identity or equality crosses crates, use Foundational canonicalization and
comparison artifacts. Do not format debug text or join strings to create a
semantic digest.

When a ready canonical sequence needs a compact SHA-256 key, use the admitted
digest front door:

```rust
let ready = canonicalization()
    .digest()
    .for_sequence(sequence, CanonicalDigestAlgorithmId::sha256())
    .into_result()?;
let digest = canonicalization().digest().derive(ready);
```

Do not call a hashing library directly for cross-crate semantic identity. The
digest front door binds the algorithm, input shape, domain, and rule version to
the ready basis. Its output remains derived evidence rather than authority.

Canonical artifacts describe and compare shared meaning. They do not become
Query operation authority, a Relational patch, a Bridge correspondence witness,
or a Signal decision.

## Strongest-Owner Rule

The crate that owns an operational transition must mint the stronger artifact:

- Query mints bound operations, publications, consumer contracts, and workflow
  traces
- Relational mints authoritative patch publication
- Runtime Bridge mints installed correspondence and lowering
- Signal mints evaluation decisions
- Proof supplies reusable progression law beneath those owner-specific types

Foundational supplies the shared semantic material those artifacts retain. A
caller cannot assemble a stronger owner artifact from Foundational parts.

## Anti-Patterns

- Duplicating aspect keys, masks, bindings, or change kinds in another crate.
- Using strings or debug output as cross-runtime identity.
- Treating a canonical digest as operational admission authority.
- Putting runtime-local Signal allocation into an aspect contract.
- Weakening an owner-specific proof to a generic Foundational report and then
  trying to promote it again.

## Related Docs

### Physical integrity observations

`physical_integrity_observation` exports portable artifact/family identities,
byte ranges, generations, five integrity postures, quarantine descriptions,
adapter evidence, and disagreement. Constructors bound identifier and range
shape; protocol consumers must also validate untrusted wire schemas and limits.
These DTOs cannot open media, runtime decoding, recovery choice, or repair.
Keep owner disposition separate from observed integrity; projection is one-way,
never a route back into a sealed runtime proof.

See the [Store integrity guide](../../_docs/worth-store/physical-integrity-and-offline-verification.md)
and the module's compiling descriptive-projection example.

- [Query Aspects And Authority Lanes](../../workspaces/worth-query/crates/worth-query/docs/modeling/aspects-and-authority-lanes.md)
- [Conditional Installed Operations](../../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md)
- [worth-proof](../worth-proof/README.md)
