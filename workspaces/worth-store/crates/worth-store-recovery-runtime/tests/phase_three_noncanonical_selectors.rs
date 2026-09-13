#[allow(dead_code)]
mod phase_three_support;

use phase_three_support::*;
use worth_store_physical_format::RootSelectorRole;
use worth_store_recovery_runtime::{
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoveryRootProtocolDenial,
    PhysicalRecoverySourceDenial,
};

#[test]
fn crc_valid_noncanonical_current_and_previous_selectors_never_project() {
    for role in [RootSelectorRole::Current, RootSelectorRole::Previous] {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join(format!("noncanonical-{role:?}"));
        let store = initialize_store(&root);
        publish_synthetic_genesis(&root, store);
        let records = root.join("families").join("records");
        let path = records.join(match role {
            RootSelectorRole::Current => "root-current.selector",
            RootSelectorRole::Previous => "root-previous.selector",
        });
        let mut bytes = std::fs::read(records.join("root-current.selector")).unwrap();
        mutate_frame_metadata(&mut bytes);
        std::fs::write(path, bytes).unwrap();

        let discovered = admitted_recovery(&root).discover().unwrap();
        let counters = discovered.counters();
        match role {
            RootSelectorRole::Current => {
                assert_eq!(counters.current_selector_integrity_admissions, 0);
                assert_eq!(counters.current_selector_interpretations, 0);
                assert_eq!(counters.current_root_integrity_admissions, 0);
                let blocked = match discovered.select() {
                    Ok(_) => panic!("noncanonical current selector must block"),
                    Err(outcome) => expect_blocked(outcome),
                };
                assert!(blocked
                    .evidence()
                    .source_denials
                    .iter()
                    .any(is_selector_denial));
            }
            RootSelectorRole::Previous => {
                assert_eq!(counters.previous_selector_integrity_admissions, 0);
                assert_eq!(counters.previous_selector_interpretations, 0);
                let selected = discovered.select().unwrap();
                assert!(selected
                    .root_protocol_denials()
                    .iter()
                    .any(is_selector_denial));
                let _ = selected.cancel_before_reconstruction();
            }
        }
    }
}

fn is_selector_denial(denial: &PhysicalRecoverySourceDenial) -> bool {
    matches!(
        denial,
        PhysicalRecoverySourceDenial::RootProtocol {
            artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector
                | PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
            denial: PhysicalRecoveryRootProtocolDenial::Integrity(_),
        }
    )
}

fn mutate_frame_metadata(bytes: &mut [u8]) {
    bytes[22] = 1;
    let checksum = crc32c_parts(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
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
