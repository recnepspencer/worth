//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeAsyncRequestTruthViewBasis>("crates/worth-runtime-bridge/src/source/async_declaration/request_identity/truth_basis.rs", "BridgeAsyncRequestTruthViewBasis", 128 * 1024);
        owner.field::<BridgeAsyncRequestTruthViewBasisIdentity>("basis_identity");
        owner.field::<BridgeAsyncRequestTruthViewBasisKind>("kind");
        owner.field::<Option<TruthBranchIdentity>>("truth_branch_identity");
        owner.field::<Option<TruthCommitIdentity>>("truth_commit_identity");
        owner.field::<Option<TruthSnapshotIdentity>>("truth_snapshot_identity");
        owner.field::<Option<crate::subscription::BridgePreviewActiveSubscriptionIdentity>>(
            "preview_active_subscription_identity",
        );
        owner.field::<Option<Arc<str>>>("preview_parent_truth_view_basis_digest");
        owner.field::<Arc<str>>("canonical_basis");
        owner.field::<Arc<str>>("digest");
    }
}
