//! Policy geometry must reject before constructing ordinary Store owners.

#[allow(unused_imports)] use crate::physical_record_journeys::format_readmission::residency_geometry::*;
use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalPageSizeClass, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::RecordBootstrapDenial};

#[test]
fn initialization_rejects_foreign_policy_format_before_any_media_effect() {
    let parent = tempfile::tempdir().unwrap();
    let target = media(&parent.path().join("initialize-policy-geometry"));
    let before = target.media_counters();
    let (format, placement, access) = configuration();
    let policy = foreign_format_policy();
    let outcome = initialize_record_store!(target, |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
            .with_residency_policy(policy)
    })
    .into_raw();
    let worth_proof::TransitionOutcome::Denied(denial) = outcome else {
        panic!("a policy admitted against another format must deny initialization");
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::ResidencyPolicyFormatMismatch {
            configured: format.declaration(),
            admitted: policy.record_format(),
        }
    );
    let target = denial.into_runtime();
    assert_eq!(target.media_counters(), before);
    target.close();
}

#[test]
fn open_rejects_foreign_policy_format_before_root_or_media_admission() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("open-policy-geometry");
    serving_from_initialization(&root).close();
    let target = media(&root);
    let before = target.media_counters();
    let (format, _, access) = configuration();
    let policy = foreign_format_policy();
    let outcome = open_record_store!(target, |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability).with_residency_policy(policy)
    })
    .into_raw();
    let worth_proof::TransitionOutcome::Denied(denial) = outcome else {
        panic!("a policy admitted against another format must deny ordinary open");
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::ResidencyPolicyFormatMismatch {
            configured: format.declaration(),
            admitted: policy.record_format(),
        }
    );
    let target = denial.into_runtime();
    assert_eq!(target.media_counters(), before);
    target.close();
}
