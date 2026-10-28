use super::super::probes::{RecoveryProbe, ReplayProbe};

pub(crate) fn compare_replay_probe(left: &ReplayProbe, right: &ReplayProbe) -> Vec<String> {
    let mut mismatches = Vec::new();
    if left.branch_name != right.branch_name {
        mismatches.push("branch_name".to_string());
    }
    if left.commit_id != right.commit_id {
        mismatches.push("commit_id".to_string());
    }
    if left.mismatch_count != right.mismatch_count {
        mismatches.push("mismatch_count".to_string());
    }
    if left.failure != right.failure {
        mismatches.push("failure".to_string());
    }
    mismatches
}

pub(crate) fn compare_recovery_probe(left: &RecoveryProbe, right: &RecoveryProbe) -> Vec<String> {
    let mut mismatches = Vec::new();
    if left.latest_commit_id != right.latest_commit_id {
        mismatches.push("latest_commit_id".to_string());
    }
    if left.branch_heads != right.branch_heads {
        mismatches.push("branch_heads".to_string());
    }
    if left.checkpoint_covered_position != right.checkpoint_covered_position {
        mismatches.push("checkpoint_covered_position".to_string());
    }
    mismatches
}
