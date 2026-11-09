//! Phase 3 reconstruction and fresh expected-identity validation.

use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableDomainPackageIdentity,
    WorthQueryPortablePackageReconstruction, WorthQueryPortablePackageReconstructionLimits,
};
use worth_query_package_archive::facade::{
    decode_package_archive, prepare_package_release_envelope, WorthQueryPackageArchiveLimits,
    WorthQueryPackageEnvelopeLimits, WorthQueryUnsignedPackageReleaseEnvelope,
};

use crate::denial::WorthQueryReleaseCeremonyError as Error;

pub(crate) fn readmit_exact_release(
    unsigned: &WorthQueryUnsignedPackageReleaseEnvelope,
    expected_identity: &WorthQueryPortableDomainPackageIdentity,
) -> Result<WorthQueryPortableDomainPackageIdentity, Error> {
    let archive =
        decode_package_archive(unsigned.archive(), WorthQueryPackageArchiveLimits::DEFAULT)
            .map_err(|denial| Error::Archive {
                stage: "decode",
                denial,
            })?;
    let (manifest, frames) = archive.into_parts();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .map_err(|denial| Error::Reconstruction { denial })?;
    for frame in frames {
        let (index, record) = frame.into_parts();
        reconstruction = reconstruction
            .push_record(index, record)
            .map_err(|denial| Error::Reconstruction { denial })?;
    }
    let validated = reconstruction
        .close()
        .and_then(|candidate| candidate.materialize())
        .and_then(|candidate| {
            candidate.validate_freshly(
                WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                    expected_identity.clone(),
                ),
            )
        })
        .map_err(|denial| Error::Reconstruction { denial })?;
    let records = validated
        .export_typed_records()
        .map_err(|denial| Error::Export { denial })?;
    let reproduced = prepare_package_release_envelope(
        &records,
        unsigned.descriptor().clone(),
        WorthQueryPackageArchiveLimits::DEFAULT,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )
    .map_err(|denial| Error::Archive {
        stage: "release re-derivation",
        denial,
    })?;
    if reproduced.signing_payload() != unsigned.signing_payload() {
        return Err(Error::ReleaseDescriptionMismatch);
    }
    Ok(validated.identity().clone())
}

#[cfg(test)]
mod tests {
    use worth_query_installation::facade::{
        WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    };
    use worth_query_package_archive::facade::{
        decode_package_release_envelope, decode_package_release_signing_payload,
        prepare_package_release_envelope, WorthQueryPackageArchiveLimits,
        WorthQueryPackageEnvelopeLimits,
    };

    use super::readmit_exact_release;

    const GOLDEN_ENVELOPE_HEX: &str = include_str!(
        "../../../workspaces/worth-query/crates/worth-query-testkit-package-archive/src/package_archive/archive_protocol/release_envelope/release_envelope_v1_record_v18.hex"
    );

    #[test]
    fn fresh_release_readmission_rejects_requirements_not_derived_from_the_archive() {
        let descriptor = decode_package_release_envelope(
            &decode_hex(GOLDEN_ENVELOPE_HEX.trim()),
            WorthQueryPackageEnvelopeLimits::DEFAULT,
        )
        .unwrap()
        .envelope()
        .unsigned()
        .descriptor()
        .clone();
        let validated = WorthQueryPortableDomainPackage::new(
            WorthQueryPortableDomainIdentity::new("qa.release", 1, 0),
        )
        .requires_capability("query-read")
        .validate()
        .unwrap();
        let records = validated.export_typed_records().unwrap();
        let produced = prepare_package_release_envelope(
            &records,
            descriptor,
            WorthQueryPackageArchiveLimits::DEFAULT,
            WorthQueryPackageEnvelopeLimits::DEFAULT,
        )
        .unwrap();
        let mut forged = produced.signing_payload().to_vec();
        replace_first(&mut forged, b"query-read", b"query-dead");
        let decoded = decode_package_release_signing_payload(
            &forged,
            WorthQueryPackageEnvelopeLimits::DEFAULT,
        )
        .unwrap();

        let denial = readmit_exact_release(decoded.unsigned(), validated.identity()).unwrap_err();

        assert!(matches!(
            denial,
            crate::denial::WorthQueryReleaseCeremonyError::ReleaseDescriptionMismatch
        ));
    }

    fn replace_first(bytes: &mut [u8], original: &[u8], replacement: &[u8]) {
        assert_eq!(original.len(), replacement.len());
        let offset = bytes
            .windows(original.len())
            .position(|window| window == original)
            .expect("fixture contains the release requirement");
        bytes[offset..offset + original.len()].copy_from_slice(replacement);
    }

    fn decode_hex(encoded: &str) -> Vec<u8> {
        encoded
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                let text = std::str::from_utf8(pair).unwrap();
                u8::from_str_radix(text, 16).unwrap()
            })
            .collect()
    }
}
