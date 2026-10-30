pub use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
pub use worth_query_package_archive::facade::*;

pub use super::archive_stream::fixture as package_fixture;

pub fn fixture_descriptor() -> WorthQueryPackageReleaseEnvelopeDescriptor {
    WorthQueryPackageReleaseEnvelopeDescriptor::new(
        WorthQueryPackageBuildMetadata::new(
            "rustc",
            "1.99.0",
            "stable",
            "1.99.0",
            "x86_64-pc-windows-msvc",
        )
        .unwrap(),
        WorthQueryPackageReleaseMetadata::new("workflow-editor", "2026.08.26").unwrap(),
        WorthQueryPackageReleaseProvenance::new(
            "https://github.com/worth/core",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "refs/tags/query-9.16.2",
        )
        .unwrap(),
        WorthQueryPackageReleaseSignerDescriptor::new(
            "release-key-01",
            BoundaryProtocolIdentity::new("worth.release.ed25519"),
            BoundaryProtocolVersion::new(1),
        )
        .unwrap(),
    )
}

pub fn descriptor_with_build(
    compiler: &str,
    compiler_version: &str,
    toolchain: &str,
    toolchain_version: &str,
    target: &str,
) -> WorthQueryPackageReleaseEnvelopeDescriptor {
    let baseline = fixture_descriptor();
    WorthQueryPackageReleaseEnvelopeDescriptor::new(
        WorthQueryPackageBuildMetadata::new(
            compiler,
            compiler_version,
            toolchain,
            toolchain_version,
            target,
        )
        .unwrap(),
        baseline.release_metadata().clone(),
        baseline.provenance().clone(),
        baseline.signer().clone(),
    )
}

pub fn descriptor_with_release(
    name: &str,
    version: &str,
) -> WorthQueryPackageReleaseEnvelopeDescriptor {
    let baseline = fixture_descriptor();
    WorthQueryPackageReleaseEnvelopeDescriptor::new(
        baseline.build_metadata().clone(),
        WorthQueryPackageReleaseMetadata::new(name, version).unwrap(),
        baseline.provenance().clone(),
        baseline.signer().clone(),
    )
}

pub fn descriptor_with_provenance(
    repository: &str,
    revision: &str,
    reference: &str,
) -> WorthQueryPackageReleaseEnvelopeDescriptor {
    let baseline = fixture_descriptor();
    WorthQueryPackageReleaseEnvelopeDescriptor::new(
        baseline.build_metadata().clone(),
        baseline.release_metadata().clone(),
        WorthQueryPackageReleaseProvenance::new(repository, revision, reference).unwrap(),
        baseline.signer().clone(),
    )
}

pub fn descriptor_with_signer(identity: &str) -> WorthQueryPackageReleaseEnvelopeDescriptor {
    let baseline = fixture_descriptor();
    WorthQueryPackageReleaseEnvelopeDescriptor::new(
        baseline.build_metadata().clone(),
        baseline.release_metadata().clone(),
        baseline.provenance().clone(),
        WorthQueryPackageReleaseSignerDescriptor::new(
            identity,
            BoundaryProtocolIdentity::new("worth.release.ed25519"),
            BoundaryProtocolVersion::new(1),
        )
        .unwrap(),
    )
}

pub fn unsigned_envelope(
    records: &worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
    descriptor: WorthQueryPackageReleaseEnvelopeDescriptor,
) -> WorthQueryUnsignedPackageReleaseEnvelope {
    prepare_package_release_envelope(
        records,
        descriptor,
        WorthQueryPackageArchiveLimits::DEFAULT,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )
    .unwrap()
}

pub fn signed_envelope_bytes(
    records: &worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
    descriptor: WorthQueryPackageReleaseEnvelopeDescriptor,
) -> Vec<u8> {
    let signed = unsigned_envelope(records, descriptor)
        .attach_signature(signature(0xa5), WorthQueryPackageEnvelopeLimits::DEFAULT)
        .unwrap();
    encode_package_release_envelope(&signed, WorthQueryPackageEnvelopeLimits::DEFAULT).unwrap()
}

pub fn ordered_envelope_bytes() -> Vec<u8> {
    let records = package_fixture::ordered_requirement_package()
        .export_typed_records()
        .unwrap();
    signed_envelope_bytes(&records, fixture_descriptor())
}

pub fn signature(byte: u8) -> WorthQueryPackageReleaseEnvelopeSignature {
    WorthQueryPackageReleaseEnvelopeSignature::new(vec![byte; 64]).unwrap()
}
