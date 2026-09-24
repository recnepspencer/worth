use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

use crate::facade::durability::RecoveryVerificationMode;
use crate::facade::history::BranchId;
use crate::tests::support::*;

#[test]
fn checkpoint_restore_reports_unavailable_instead_of_equivalent_field_revision() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "same");
    let entity = changed_entities(&created)[0];
    let name = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key("name"),
        CanonicalFieldPath::single(field_key("name")),
    );
    assert!(runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .entity_field_revision(entity, &name)
        .is_some());
    runtime.durability_authority().checkpoint().unwrap();
    let recovery = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let mut recovered = runtime_with_test_schema();
    recovered.durability_recovery().recover(recovery).unwrap();
    let snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".to_owned()));
    assert_eq!(
        recovered
            .read_truth()
            .project_snapshot(&snapshot)
            .unwrap()
            .entity_field_revision(entity, &name),
        None
    );
    recovered.snapshots().release_snapshot(&snapshot).unwrap();
    release_test_commit_snapshot(&runtime, &created);
}
