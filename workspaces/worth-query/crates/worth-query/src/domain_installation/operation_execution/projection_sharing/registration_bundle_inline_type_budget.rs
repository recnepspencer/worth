//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryCheckedSharedOwnerRegistration>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/projection_sharing/registration_bundle.rs", "WorthQueryCheckedSharedOwnerRegistration", 128 * 1024);
        owner.field::<crate::ordinary::live::WorthQueryManagedLiveHandle>("handle");
        owner.field::<super::super::super::WorthQueryLiveProjectionReceipt>("receipt");
        owner.field::<Vec<crate::domain_installation::WorthQueryConditionalProvenance>>(
            "conditional_provenance",
        );
        owner.field::<std::sync::Arc<
            crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
        >>("closure");
        owner.field::<WorthQueryAdmittedProjectionSharing>("admission");
    }
}
