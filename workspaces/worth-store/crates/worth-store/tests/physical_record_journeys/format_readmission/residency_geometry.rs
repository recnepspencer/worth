//! Policy geometry must reject before constructing ordinary Store owners.

use super::super::{configuration, media, serving_from_initialization};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy, PhysicalPageSizeClass,
    PhysicalRecordFormatDeclaration, PhysicalRecordInitialization, PhysicalRecordOpen,
    RecordBootstrapDenial,
};

#[test]
fn initialization_rejects_foreign_policy_format_before_any_media_effect() {
    let parent = tempfile::tempdir().unwrap();
    let target = media(&parent.path().join("initialize-policy-geometry"));
    let before = target.media_counters();
    let (format, placement, access) = configuration();
    let policy = foreign_format_policy();
    let outcome = initialize_record_store!(target, |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
            .with_residency_policy(policy)
    })
    .into_raw();
    let TransitionOutcome::Denied(denial) = outcome else {
        panic!("a policy admitted against another format must deny initialization");
    };
    assert_eq!(
        denial.reason(),
        RecordBootstrapDenial::ResidencyPolicyFormatMismatch {
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
        PhysicalRecordOpen::new(format, access, durability).with_residency_policy(policy)
    })
    .into_raw();
    let TransitionOutcome::Denied(denial) = outcome else {
        panic!("a policy admitted against another format must deny ordinary open");
    };
    assert_eq!(
        denial.reason(),
        RecordBootstrapDenial::ResidencyPolicyFormatMismatch {
            configured: format.declaration(),
            admitted: policy.record_format(),
        }
    );
    let target = denial.into_runtime();
    assert_eq!(target.media_counters(), before);
    target.close();
}

fn foreign_format_policy() -> AdmittedPhysicalRecordResidencyPolicy {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    );
    AdmittedPhysicalRecordResidencyPolicy::canonical(format)
}
