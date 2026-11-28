use worth_runtime_bridge::facade::{
    TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity, TruthSnapshotIdentity,
};

fn received_commit_from_bridge(commit_identity: TruthCommitIdentity) {
    let _formatted = format!("{}", commit_identity);
    let _text = commit_identity.as_str();
    let _compared = commit_identity == "commit-1";
}

fn received_snapshot_from_bridge(snapshot_identity: TruthSnapshotIdentity) {
    let _formatted = format!("{}", snapshot_identity);
    let _text = snapshot_identity.as_str();
    let _compared = snapshot_identity == "snapshot-1";
}

fn received_patch_from_bridge(patch_identity: TruthPatchIdentity) {
    let _formatted = format!("{}", patch_identity);
    let _text = patch_identity.as_str();
    let _compared = patch_identity == "patch-1";
}

fn received_branch_from_bridge(branch_identity: TruthBranchIdentity) {
    let _formatted = format!("{}", branch_identity);
    let _text = branch_identity.as_str();
    let _compared = branch_identity == "branch-main";
}

fn main() {
    let _ = TruthCommitIdentity::new("commit-1");
    let _ = TruthSnapshotIdentity::new("snapshot-1");
    let _ = TruthPatchIdentity::new("patch-1");
    let _ = TruthBranchIdentity::new("branch-main");
}
