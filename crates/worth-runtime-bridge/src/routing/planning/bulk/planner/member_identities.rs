pub(super) fn bulk_continuity_member_identity(
    lineage_context: &crate::continuity::BridgeLineageContext,
) -> BulkContinuityMemberIdentity {
    let authority_basis = lineage_context.authority_basis();
    BulkContinuityMemberIdentity::admit_bridge_owned(digest_string(
        "bulk-continuity-member",
        &format!(
            "bulk-continuity-member|authority={}|branch={}|snapshot={}",
            authority_basis.digest(),
            authority_basis.branch_identity().canonical_label(),
            authority_basis.snapshot_identity().canonical_label()
        ),
    ))
}

pub(super) fn bulk_truth_view_member_identity(
    route: &BridgePlannedRoute,
) -> BulkTruthViewMemberIdentity {
    BulkTruthViewMemberIdentity::admit_bridge_owned(digest_string(
        "bulk-truth-view-member",
        &format!(
            "bulk-truth-view-member|branch={}|snapshot={}|commit={}",
            route.source_branch().canonical_label(),
            route.source_snapshot().canonical_label(),
            route.source_commit().canonical_label()
        ),
    ))
}

use super::*;
