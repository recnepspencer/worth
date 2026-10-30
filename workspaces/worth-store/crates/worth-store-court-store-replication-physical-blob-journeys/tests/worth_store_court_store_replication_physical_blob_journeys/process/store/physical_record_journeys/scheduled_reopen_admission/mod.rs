#[allow(unused_imports)]
use crate::physical_record_journeys::scheduled_reopen_admission::*;

#[test]
fn scheduled_reopen_enters_selector_and_root_only_after_admission() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("scheduled-reopen");
    let (expected_root, expected_selector) = published_root_protocol(&root);
    let (coordination, media) = recovery_coordination(&root);
    let outcome =
        coordination.execute_fresh_reopen(&media, command(expected_root, expected_selector));
    assert!(matches!(
        outcome,
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenOutcome::Completed(_)
    ));
    let counters = coordination.root_protocol_counters();
    assert_eq!(
        counters.selector_entries(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        1,
    );
    assert_eq!(
        counters.root_entries(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        1,
    );
    assert!(coordination.shutdown_is_quiescent());
}

#[test]
fn previous_role_substitution_cannot_enter_current_selector_interpretation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("duplicate-current-slot");
    let (expected_root, expected_selector) = published_root_protocol(&root);
    let duplicate = worth_store_physical_format::DurableRootSelector::new(
        expected_selector.store_identity(),
        expected_selector.format(),
        expected_selector.identity(),
        worth_store_physical_format::RootSelectorRole::Previous,
        expected_selector.root_generation(),
        expected_selector.linked_selector(),
        expected_selector.linked_root_generation(),
    )
    .unwrap();
    std::fs::write(selector_path(&root), duplicate.encode()).unwrap();
    let (coordination, media) = recovery_coordination(&root);
    let outcome =
        coordination.execute_fresh_reopen(&media, command(expected_root, expected_selector));
    let worth_store::physical_runtime::PhysicalRecoveryFreshReopenOutcome::Denied(denial) = outcome
    else {
        panic!("a previous-role substitution in the current slot must be denied")
    };
    let Some(worth_store::physical_runtime::RootProtocolAdmissionDenial::Validation(
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization),
    )) = denial.integrity()
    else {
        panic!("role substitution must retain the validator's narrow damage cause")
    };
    assert_eq!(
        localization.scope().artifact_family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CurrentRootSelector,
    );
    assert_eq!(
        localization.cause(),
        worth_store_physical_integrity::PhysicalDamageCause::SelectorRoleMismatch,
    );
    assert_eq!(
        localization.damaged_range(),
        worth_store_physical_integrity::PhysicalByteRange::new(64, 1).unwrap(),
    );
    assert_eq!(
        localization.field(),
        Some(worth_store_physical_integrity::PhysicalFormatField::SelectorRole),
    );
    assert_eq!(
        localization.blast_radius(),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
    let counters = coordination.root_protocol_counters();
    assert_eq!(
        counters.selector_entries(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        0,
    );
    assert_eq!(
        counters.root_entries(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        0,
    );
    assert_eq!(
        counters.publications(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        0,
    );
    assert!(coordination.shutdown_is_quiescent());
}

#[test]
fn admitted_unexpected_selector_cannot_address_a_root() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("unexpected-current-selector");
    let (expected_root, expected_selector) = published_root_protocol(&root);
    let unexpected = worth_store_physical_format::DurableRootSelector::new(
        expected_selector.store_identity(),
        expected_selector.format(),
        expected_selector.identity(),
        expected_selector.role(),
        expected_selector.root_generation() + 1,
        expected_selector.linked_selector(),
        expected_selector.linked_root_generation(),
    )
    .unwrap();
    std::fs::write(selector_path(&root), unexpected.encode()).unwrap();
    let (coordination, media) = recovery_coordination(&root);
    let outcome =
        coordination.execute_fresh_reopen(&media, command(expected_root, expected_selector));
    let worth_store::physical_runtime::PhysicalRecoveryFreshReopenOutcome::Denied(denial) = outcome
    else {
        panic!("an admitted selector outside the planned binding must be denied")
    };
    assert_eq!(
        denial.kind(),
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenDenialKind::BindingMismatch,
    );
    assert!(denial.integrity().is_none());
    assert!(denial.root().is_none());
    let counters = coordination.root_protocol_counters();
    assert_eq!(
        counters.selector_entries(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        1,
    );
    assert_eq!(
        counters.root_entries(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        0,
    );
    assert_eq!(
        counters.publications(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        0,
    );
    assert!(coordination.shutdown_is_quiescent());
}

#[test]
fn wrong_generation_root_cannot_enter_reopen_root_interpretation_or_publication() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("wrong-generation-root");
    let (expected_root, expected_selector) = published_root_protocol(&root);
    std::fs::copy(
        root_path(&root, 1),
        root_path(&root, expected_root.generation()),
    )
    .unwrap();
    let (coordination, media) = recovery_coordination(&root);
    let outcome =
        coordination.execute_fresh_reopen(&media, command(expected_root, expected_selector));
    let worth_store::physical_runtime::PhysicalRecoveryFreshReopenOutcome::Denied(denial) = outcome
    else {
        panic!("a checksum-valid root for another generation must be denied")
    };
    assert!(matches!(
        denial.integrity(),
        Some(worth_store::physical_runtime::RootProtocolAdmissionDenial::Validation(_))
    ));
    let counters = coordination.root_protocol_counters();
    assert_eq!(
        counters.selector_entries(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        1,
    );
    assert_eq!(
        counters.root_entries(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        0,
    );
    assert_eq!(
        counters.publications(
            worth_store::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen
        ),
        0,
    );
    assert!(coordination.shutdown_is_quiescent());
}
