use worth_relational::facade::branch::{RelationalBranchRootDescriptor, RelationalBranchTarget};
use worth_relational::facade::history::{BranchId, CommitId, RelationalCommitReceipt};
use worth_relational::facade::identity::VersionId;

fn main() {
    let _ = RelationalBranchTarget::from_commit_receipt(
        7,
        &RelationalCommitReceipt {
            commit_id: CommitId(1),
            version_id: VersionId(0),
            branch_id: BranchId("main".to_owned()),
            parents: Vec::new(),
        },
        RelationalBranchRootDescriptor::new([0; 32], [0; 32]),
    );
}
