

pub(crate) use {worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery, worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked, worth_store_testkit_recovery_runtime::phase_three_support::initialize_store, worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis};
pub(crate) use {worth_store_physical_format::RootSelectorRole};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial, worth_store_recovery_runtime::PhysicalRecoverySourceDenial};

pub(crate) fn is_selector_denial(denial: &PhysicalRecoverySourceDenial) -> bool {
    matches!(
        denial,
        worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
            artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector
                | worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
            denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(_),
        }
    )
}

pub(crate) fn mutate_frame_metadata(bytes: &mut [u8]) {
    bytes[22] = 1;
    let checksum = crc32c_parts(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

pub(crate) fn crc32c_parts(parts: &[&[u8]]) -> u32 {
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