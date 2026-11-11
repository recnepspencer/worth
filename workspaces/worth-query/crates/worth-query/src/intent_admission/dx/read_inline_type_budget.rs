//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryWorkspaceReadIntentAuthoring<'static>>(
            "workspaces/worth-query/crates/worth-query/src/intent_admission/dx/read.rs",
            "WorthQueryWorkspaceReadIntentAuthoring",
            128 * 1024,
        );
        owner.field::<&'static mut WorthQueryWorkspace>("workspace");
        owner.field::<WorthQueryReadFamily>("read_family");
        owner.field::<Option<ScopedQueryBasisContext>>("basis_context");
        owner.field::<Option<WorthQueryGraphReadAccessAuthorityContext>>("graph_read_authority");
    }
    {
        let mut owner = census.owner::<WorthQueryWorkspaceReadIntentAdmissionReview<'static>>(
            "workspaces/worth-query/crates/worth-query/src/intent_admission/dx/read.rs",
            "WorthQueryWorkspaceReadIntentAdmissionReview",
            128 * 1024,
        );
        owner.field::<&'static mut WorthQueryWorkspace>("workspace");
        owner.field::<WorthQueryRuntimeIntentAdmissionReviewData>("review");
        owner.field::<Option<WorthQueryGraphReadAccessAuthorityContext>>("graph_read_authority");
    }
    {
        let mut owner = census.owner::<WorthQueryAdmittedWorkspaceReadIntent<'static>>(
            "workspaces/worth-query/crates/worth-query/src/intent_admission/dx/read.rs",
            "WorthQueryAdmittedWorkspaceReadIntent",
            128 * 1024,
        );
        owner.field::<&'static mut WorthQueryWorkspace>("workspace");
        owner.field::<WorthQueryRuntimeIntentAdmissionReviewData>("review");
        owner.field::<WorthQueryReadExecutionHandoff>("handoff");
        owner.field::<WorthQueryReadExecutionBinding>("execution_binding");
    }
}
