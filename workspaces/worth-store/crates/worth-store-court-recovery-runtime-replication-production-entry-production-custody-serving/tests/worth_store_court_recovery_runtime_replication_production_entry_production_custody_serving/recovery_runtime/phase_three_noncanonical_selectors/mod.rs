#[allow(unused_imports)]
use crate::phase_three_noncanonical_selectors::*;
use worth_store_physical_format::RootSelectorRole;
use {
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial,
    worth_store_recovery_runtime::PhysicalRecoverySourceDenial,
};
use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
};

#[test]
fn crc_valid_noncanonical_current_and_previous_selectors_never_project() {
    for role in [
        worth_store_physical_format::RootSelectorRole::Current,
        worth_store_physical_format::RootSelectorRole::Previous,
    ] {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join(format!("noncanonical-{role:?}"));
        let store = initialize_store(&root);
        publish_synthetic_genesis(&root, store);
        let records = root.join("families").join("records");
        let path = records.join(match role {
            worth_store_physical_format::RootSelectorRole::Current => "root-current.selector",
            worth_store_physical_format::RootSelectorRole::Previous => "root-previous.selector",
        });
        let mut bytes = std::fs::read(records.join("root-current.selector")).unwrap();
        mutate_frame_metadata(&mut bytes);
        std::fs::write(path, bytes).unwrap();

        let discovered = admitted_recovery(&root).discover().unwrap();
        let counters = discovered.counters();
        match role {
            worth_store_physical_format::RootSelectorRole::Current => {
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
            worth_store_physical_format::RootSelectorRole::Previous => {
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
