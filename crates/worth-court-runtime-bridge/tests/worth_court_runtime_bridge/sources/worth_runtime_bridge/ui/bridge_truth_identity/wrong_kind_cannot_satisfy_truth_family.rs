use worth_runtime_bridge::facade::{BridgeTruthViewSelector, TruthCommitIdentity};

fn main() {
    let commit = TruthCommitIdentity::from_relational_commit_id(1);

    let _selector = BridgeTruthViewSelector::branch_head(commit);
}
