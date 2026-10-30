use worth_runtime_bridge::facade::TruthCommitIdentity;

fn main() {
    let projection = TruthCommitIdentity::from_relational_commit_id(7);
    let _ = projection.bridge_trust_boundary();
}
