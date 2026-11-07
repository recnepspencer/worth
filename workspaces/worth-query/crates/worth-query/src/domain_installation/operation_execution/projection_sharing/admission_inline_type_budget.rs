//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryProjectionSharingDenialKind>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/projection_sharing/admission.rs", "WorthQueryProjectionSharingDenialKind", 128 * 1024);
    }
    {
        let mut owner = census.owner::<WorthQueryProjectionSharingCounters>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/projection_sharing/admission.rs", "WorthQueryProjectionSharingCounters", 128 * 1024);
        owner.field::<usize>("support_posture_checks");
        owner.field::<usize>("candidate_preflight_checks");
        owner.field::<super::super::super::WorthQueryProjectionPromotionCounters>(
            "subject_preflight",
        );
        owner.field::<super::super::super::WorthQueryProjectionPromotionCounters>(
            "candidate_preflight",
        );
        owner.field::<crate::domain_installation::WorthQueryCompatibilityCounters>("compatibility");
        owner.field::<usize>("compatibility_checks");
        owner.field::<usize>("closure_comparisons");
        owner.field::<usize>("closure_readmissions");
        owner.field::<usize>("dependency_edges_compared");
        owner.field::<usize>("dependency_edges_readmitted");
        owner.field::<usize>("projection_contract_checks");
        owner.field::<usize>("native_layout_checks");
        owner.field::<usize>("installed_read_checks");
        owner.field::<usize>("owner_registrations");
        owner.field::<usize>("lease_issues");
        owner.field::<usize>("unrelated_registry_scans");
    }
    let _ = census.owner::<WorthQuerySharedLiveProjectionPair<(), (), (), crate::basis_lifecycle::ObservationLaneWitness>>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/projection_sharing/admission.rs", "WorthQuerySharedLiveProjectionPair", 128 * 1024);
    let _ = census.owner::<WorthQueryProjectionSharingOutcome<(), (), (), crate::basis_lifecycle::ObservationLaneWitness>>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/projection_sharing/admission.rs", "WorthQueryProjectionSharingOutcome", 128 * 1024);
    let _ = census.owner::<WorthQueryProjectionSharingStop<(), (), (), crate::basis_lifecycle::ObservationLaneWitness>>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/projection_sharing/admission.rs", "WorthQueryProjectionSharingStop", 128 * 1024);
}
