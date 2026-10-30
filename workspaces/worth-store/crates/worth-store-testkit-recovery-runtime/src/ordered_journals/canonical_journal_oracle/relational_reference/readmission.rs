use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::RelationalCanonicalCommitArtifact;

/// Readmit real reference bytes using the execution request owned by this case.
pub fn assert_readmits(position: u64, bytes: &[u8], execution: &ExecutionRequest<'_, '_>) {
    let readmitted = RelationalCanonicalCommitArtifact::readmit(bytes, execution)
        .unwrap_or_else(|refusal| panic!("journal entry {position} refused: {refusal:?}"));
    assert_eq!(
        readmitted.position().0,
        position,
        "journal position {position}"
    );
}
