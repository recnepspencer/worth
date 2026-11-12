//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryWorkspaceLiveReadIntentAuthoring<'static, ()>>(
            "workspaces/worth-query/crates/worth-query/src/intent_admission/dx/live_read.rs",
            "WorthQueryWorkspaceLiveReadIntentAuthoring",
            128 * 1024,
        );
        owner.field::<&'static mut WorthQueryWorkspace>("workspace");
        owner.field::<WorthQueryLiveView<()>>("live_view");
    }
    {
        let mut owner = census.owner::<WorthQueryWorkspaceLiveReadIntentAdmissionReview<'static>>(
            "workspaces/worth-query/crates/worth-query/src/intent_admission/dx/live_read.rs",
            "WorthQueryWorkspaceLiveReadIntentAdmissionReview",
            128 * 1024,
        );
        owner.field::<&'static mut WorthQueryWorkspace>("workspace");
        owner.field::<WorthQueryRuntimeIntentAdmissionReviewData>("review");
    }
    {
        let mut owner = census.owner::<WorthQueryAdmittedWorkspaceLiveReadIntent<'static>>(
            "workspaces/worth-query/crates/worth-query/src/intent_admission/dx/live_read.rs",
            "WorthQueryAdmittedWorkspaceLiveReadIntent",
            128 * 1024,
        );
        owner.field::<&'static mut WorthQueryWorkspace>("workspace");
        owner.field::<WorthQueryRuntimeIntentAdmissionReviewData>("review");
        owner.field::<WorthQueryLiveReadExecutionHandoff>("handoff");
        owner.field::<WorthQueryLiveReadExecutionBinding>("execution_binding");
    }
}
