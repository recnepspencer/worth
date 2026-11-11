//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryAdmittedProjectionSharing>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/projection_sharing/owner_admission.rs", "WorthQueryAdmittedProjectionSharing", 128 * 1024);
        owner.field::<WorthQueryProjectionSharingEvidence>("evidence");
        owner.field::<String>("subject_source_identity");
        owner.field::<crate::domain_installation::WorthQuerySemanticDependencyClosureEvidence>(
            "subject_closure_evidence",
        );
        owner.field::<crate::domain_installation::WorthQueryOperationAuthorityBasis>(
            "subject_affinity",
        );
        owner.field::<Option<WorthQueryAdmittedSharingCandidate>>("candidate");
    }
}
