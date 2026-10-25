use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::runtime::RelationalRuntime;

pub(crate) fn authoritative_branch_for_version(
    runtime: &RelationalRuntime,
    version_id: VersionId,
) -> BranchId {
    branch_for_version(runtime, version_id)
        .expect("every visible Relational version has authoritative branch identity")
}

pub(crate) fn branch_for_version(
    runtime: &RelationalRuntime,
    version_id: VersionId,
) -> Option<BranchId> {
    runtime
        .history()
        .immutable_commit_receipt_for_version(version_id)
        .map(|receipt| receipt.branch_id)
        .or_else(|| {
            (version_id == VersionId(0)).then(|| runtime.config().history.main_branch.clone())
        })
}
