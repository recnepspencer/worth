use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;

fn assert_sync<T: Sync>() {}

fn deny(candidate: PreparedRelationalCommitCandidate) {
    let _copy = candidate.clone();
    candidate.publish();
    assert_sync::<PreparedRelationalCommitCandidate>();
    let _serialized = rmp_serde::to_vec(&candidate);
}

fn main() {}
