use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, PhysicalRecordFormatDeclaration, ROOT_SELECTOR_BYTES,
};
use worth_store_physical_integrity::{
    validate_current_root_selector, CurrentRootSelectorIntegrityValidation, PhysicalArtifactScope,
    PhysicalByteRange, UntrustedPhysicalArtifact,
};

use super::families::root::IntegrityAdmittedStagedCurrentSelector;
use super::RecoveryIntegrityIngressRejection;

pub(crate) fn admit_staged_current_selector(
    bytes: &[u8],
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<IntegrityAdmittedStagedCurrentSelector<'_>, RecoveryIntegrityIngressRejection> {
    let scope = PhysicalArtifactScope::current_root_selector(
        store,
        format,
        PhysicalByteRange::new(0, ROOT_SELECTOR_BYTES as u64)
            .expect("the selector declaration has a nonzero fixed width"),
    );
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(bytes);
    let (validation, _) = validate_current_root_selector(input, scope);
    let validated = match validation {
        CurrentRootSelectorIntegrityValidation::Intact(validated) => validated,
        CurrentRootSelectorIntegrityValidation::Rejected(rejection) => {
            return Err(RecoveryIntegrityIngressRejection::Integrity(rejection));
        }
    };
    IntegrityAdmittedStagedCurrentSelector::bind(bytes, validated)
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };
    use worth_store_physical_format::{
        DurableRootSelector, RootSelectorIdentity, RootSelectorRole,
    };

    use super::*;

    #[test]
    fn crc_valid_noncanonical_staged_selector_is_rejected_before_projection() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([23; 16]).unwrap(),
        )
        .published_identity();
        let selector = DurableRootSelector::new(
            store,
            format,
            RootSelectorIdentity::new(1).unwrap(),
            RootSelectorRole::Current,
            1,
            None,
            None,
        )
        .unwrap();
        let mut bytes = selector.encode();
        bytes[22] = 1;
        let checksum = crc32c_parts(&[&bytes[..44], &bytes[48..]]);
        bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
        assert!(admit_staged_current_selector(&bytes, store, format).is_err());
    }

    fn crc32c_parts(parts: &[&[u8]]) -> u32 {
        let mut value = !0_u32;
        for byte in parts.iter().flat_map(|part| part.iter()) {
            value ^= u32::from(*byte);
            for _ in 0..8 {
                let mask = 0_u32.wrapping_sub(value & 1);
                value = (value >> 1) ^ (0x82f6_3b78 & mask);
            }
        }
        !value
    }
}
