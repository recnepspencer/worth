# Portable Query Packages

## What This Feature Is

Portable packages carry validated Query meaning through a bounded,
store-neutral release format. The lifecycle is:

```text
validated package
  -> typed records
  -> canonical archive
  -> canonical signing payload
  -> external signature bytes
  -> explicitly untrusted signed envelope
  -> host trust verification
  -> reconstruction and fresh Query validation
```

Use this feature when a host must retain or publish package meaning without
serializing installed handles, runtime state, or a database layout.

## Why You Use It

- Retain the same package bytes in a release artifact, immutable repository, or
  future Worth Store integration without changing Query's semantic format.
- Sign exact canonical bytes while keeping private-key handling outside Query.
- Select a package by an independently expected semantic identity, not a
  mutable release name or `latest` label.
- Reconstruct untrusted records and make Query validate their meaning again.

## Stable Entry Points

Query meaning is available through `worth_query_host::facade::domain`:

- `WorthQueryValidatedPortableDomainPackage::export_typed_records()`
- `WorthQueryPortablePackageReconstruction`
- `WorthQueryExpectedPortablePackageIdentity`
- `WorthQueryPortablePackageReconstructionLimits`

Canonical bytes and repository contracts are available through
`worth_query_package_archive::facade`:

- `prepare_package_release_envelope(...)`
- `decode_package_release_signing_payload(...)`
- `assemble_untrusted_package_release_envelope(...)`
- `decode_package_release_envelope(...)`
- `WorthQueryPackageArchiveRepository`
- `WorthQueryPackageArchiveCompatibilityProfile::CURRENT`

The host release tool provides `worth-query-release preflight` and
`worth-query-release finalize`. Preflight performs bounded canonical decoding,
checks independently supplied release expectations, freshly readmits the
embedded archive through Query, re-derives the signed requirement description,
admits the host-expected signature size, and stages exact no-overwrite signing
bytes.

## Core Mental Model

Every boundary removes authority. Exported records are descriptive. Archive
decoding proves bounded canonical structure, not package validity. Signature
presence records bytes and a claimed signer, not signer trust. A repository
stores immutable exact bytes, but storage success does not activate them.

Only a consumer that independently selects the expected package identity,
verifies the signature under current host trust policy, reconstructs the
records, and obtains fresh Query validation may proceed toward installation.
Release names, versions, tags, checksums, and manifest identity remain
descriptive throughout that progression. Multiple same-named releases may
coexist because selection and immutable storage use exact semantic identity.

## How It Executes

1. Query validates a package and exports its complete typed record inventory.
2. The archive owner encodes the manifest and records in deterministic order
   under fixed or narrower limits.
3. The release envelope covers package identity, checksum, build metadata,
   provenance, requirements, signer description, and the archive itself.
4. The host preflights and stages the exact canonical signing payload before
   any private-key material is exposed.
5. An external signer produces opaque signature bytes. Finalization frames
   those bytes and freshly readmits the complete envelope, still as untrusted.
6. A physical repository may store those exact immutable bytes under their
   claimed identity.
7. A consuming host independently verifies trust and identity, decodes the
   archive, pushes records through reconstruction in canonical order, and asks
   Query to validate the reconstructed meaning freshly.

The current compatibility profile, `WorthQueryPackageArchiveCompatibilityProfile::CURRENT`,
accepts exactly the current protocol version of each layer: release envelope,
archive, manifest, record frame, and application program description. Bytes
framed under an earlier version are refused, never reinterpreted. Compatibility
changes must add real readers; callers cannot widen the window.

## Small Example

```rust
use worth_query_host::facade::domain::{
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryPortablePackageRecordFamily,
};

let validated = WorthQueryPortableDomainPackage::new(
    WorthQueryPortableDomainIdentity::new("acme.billing", 1, 0),
)
.requires_capability("billing-read")
.validate()
.expect("the package is valid");

let records = validated
    .export_typed_records()
    .expect("the package fits the export budgets");

assert_eq!(records.manifest().package_identity(), validated.identity());
assert_eq!(
    records
        .manifest()
        .family_count(WorthQueryPortablePackageRecordFamily::DomainIdentity),
    1,
);
```

Export is available only after package validation. The returned manifest
identity describes the exported meaning; it is not installation authority.

## Real Example

This prepares the exact bytes that an external signer may sign:

```rust
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_host::facade::domain::{
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_query_package_archive::facade::{
    prepare_package_release_envelope, WorthQueryPackageArchiveLimits,
    WorthQueryPackageBuildMetadata, WorthQueryPackageEnvelopeLimits,
    WorthQueryPackageReleaseEnvelopeDescriptor, WorthQueryPackageReleaseMetadata,
    WorthQueryPackageReleaseProvenance, WorthQueryPackageReleaseSignerDescriptor,
};

let validated = WorthQueryPortableDomainPackage::new(
    WorthQueryPortableDomainIdentity::new("acme.orders", 3, 1),
)
.validate()
.expect("package validation");
let records = validated.export_typed_records().expect("bounded export");
let descriptor = WorthQueryPackageReleaseEnvelopeDescriptor::new(
    WorthQueryPackageBuildMetadata::new(
        "rustc", "1.99.0", "stable", "1.99.0", "x86_64-unknown-linux-gnu",
    )
    .expect("canonical build metadata"),
    WorthQueryPackageReleaseMetadata::new("orders", "2026.08.26")
        .expect("canonical release metadata"),
    WorthQueryPackageReleaseProvenance::new(
        "https://github.com/acme/platform",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "refs/tags/query-9.16.2",
    )
    .expect("canonical provenance"),
    WorthQueryPackageReleaseSignerDescriptor::new(
        "release-key-01",
        BoundaryProtocolIdentity::new("worth.release.ed25519"),
        BoundaryProtocolVersion::new(1),
    )
    .expect("canonical signer description"),
);
let unsigned = prepare_package_release_envelope(
    &records,
    descriptor,
    WorthQueryPackageArchiveLimits::DEFAULT,
    WorthQueryPackageEnvelopeLimits::DEFAULT,
)
.expect("bounded canonical release payload");

std::fs::write("release.signing-payload", unsigned.signing_payload())
    .expect("write the exact signing payload");
```

Production metadata must come from the build and protected Git context, not
from copied literals. The private key must remain in the host signing boundary.

The reusable workflow at `.github/workflows/publish-worth-query-release.yml`
expects a same-run artifact named `worth-query-release-signing-payload` that
contains exactly one regular file, `release.signing-payload`. A caller invokes
it as a job after the package-producing job:

```yaml
publish-query-package:
  needs: build-query-package
  permissions:
    actions: read
    contents: write
  uses: ./.github/workflows/publish-worth-query-release.yml
  with:
    expected_package_identity: ${{ needs.build-query-package.outputs.package_identity }}
    release_name: workflow-editor
    release_version: 2026.08.26
```

Configure the protected `worth-query-release` GitHub environment with required
review/tag protection, variable `WORTH_QUERY_RELEASE_SIGNER_IDENTITY`, and the
secrets `WORTH_QUERY_RELEASE_ED25519_PRIVATE_KEY_PEM` and
`WORTH_QUERY_RELEASE_ED25519_PUBLIC_KEY_PEM`. The workflow admits only a
protected tag from the same repository and revision; it never selects or marks
a release as latest.

## How It Relates To Other Features

- Package validation owns semantic truth before export and after fresh
  reconstruction.
- The archive crate owns deterministic bounded bytes and compatibility readers.
- The release tool owns host expectation checks and no-overwrite staging.
- OpenSSL owns Ed25519 mechanics; its success does not grant Query authority.
- A host repository or future Worth Store binding may retain exact archive
  bytes. It does not decode or validate package meaning, and package retention
  does not persist application state.
- Installation consumes only a freshly validated package and then adds live
  bindings and runtime authority.

## Inspection And Debugging

Inspect `records.manifest()` for semantic identity, family counts, canonical
source bytes, and logical export bytes. Inspect a decoded envelope for its
claimed provenance, requirements, signer description, checksum, and embedded
archive. Treat every decoded field as untrusted input until the current host
policy and fresh Query validation accept it.

`worth-query-release finalize` emits a JSON report whose
`artifact_posture` is `untrusted-signed-envelope`. Preserve that wording in
operator surfaces. A denial identifies whether failure occurred during bounded
input, canonical archive decoding, expectation matching, reconstruction, or
no-overwrite output.

## Anti-Patterns

- Do not use a release name, version, tag, database row ID, or `latest` label as
  package identity.
- Do not treat a checksum or signature-shaped byte string as signer trust.
- Do not let a storage adapter reconstruct, validate, install, or activate a
  package.
- Do not overwrite different envelope bytes under the same claimed identity.
- Do not deserialize installed handles, runtime generations, providers,
  callbacks, or adapter rows as portable meaning.
- Do not bypass canonical decoding or fresh validation because an artifact was
  produced by CI.

## Current Limits

- The default export ceiling is 65,536 records and 64 MiB of logical material;
  custom limits may narrow but not widen constitutional ceilings.
- The current reader window is exactly the current version at every archive
  layer; artifacts framed under an earlier protocol must be re-exported.
- The archive surface frames opaque external signature bytes; consuming hosts
  remain responsible for current trust policy and cryptographic verification.
- GitHub publication is a human distribution surface, not package discovery or
  activation authority.
- Export, archive, signing, and reconstruction are cold package-management
  operations, not warm query, mutation, or backpressure lanes.

## Related Docs

- [Query orientation for AI agents](./AI_README.md)
- [Query documentation index](./README.md)
