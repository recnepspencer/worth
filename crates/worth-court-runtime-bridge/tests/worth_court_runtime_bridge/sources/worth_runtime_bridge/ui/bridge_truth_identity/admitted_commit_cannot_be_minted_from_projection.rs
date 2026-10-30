use worth_runtime_bridge::facade::{
    BridgeAdmittedTruthCommitIdentity, TruthCommitIdentity,
};

fn main() {
    let projection = TruthCommitIdentity::from_relational_commit_id(7);
    let _ = BridgeAdmittedTruthCommitIdentity::admit(projection);
}
