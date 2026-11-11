//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryLiveView<()>>(
            "workspaces/worth-query/crates/worth-query/src/runtime/surface/live.rs",
            "WorthQueryLiveView",
            128 * 1024,
        );
        owner.field::<WorthQueryLiveViewHandle>("handle");
        owner.field::<WorthQueryAuthorityLane>("authority_lane");
        owner.field::<WorthQueryRuntimeLiveSubscriptionInstallation>("subscription_installation");
        owner.field::<PhantomData<()>>("marker");
    }
}
