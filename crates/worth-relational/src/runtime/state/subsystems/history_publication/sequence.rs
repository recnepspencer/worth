//! Publication carries identity successors checked before any cutover.
use crate::history::data::RelationalCommitReceipt;
#[derive(Debug)]
pub(crate) struct PreparedCommitSequence {
    next_commit: u64,
    next_version: u64,
}
impl PreparedCommitSequence {
    pub(crate) fn new(receipt: &RelationalCommitReceipt) -> Result<Self, String> {
        let next_commit = receipt
            .commit_id
            .0
            .checked_add(1)
            .ok_or("commit id sequence overflow")?;
        let next_version = receipt
            .version_id
            .0
            .checked_add(1)
            .ok_or("version id sequence overflow")?;
        Ok(Self {
            next_commit,
            next_version,
        })
    }
    pub(crate) fn into_successors(self) -> (u64, u64) {
        (self.next_commit, self.next_version)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn media_identity_frontiers_refuse_before_publication_sequence_construction() {
        let receipt = |commit, version| RelationalCommitReceipt {
            commit_id: crate::history::data::CommitId(commit),
            version_id: crate::identity::data::VersionId(version),
            branch_id: crate::history::data::BranchId("main".into()),
            parents: Vec::new(),
        };
        assert_eq!(
            PreparedCommitSequence::new(&receipt(u64::MAX, 1)).unwrap_err(),
            "commit id sequence overflow"
        );
        assert_eq!(
            PreparedCommitSequence::new(&receipt(1, u64::MAX)).unwrap_err(),
            "version id sequence overflow"
        );
        assert_eq!(
            PreparedCommitSequence::new(&receipt(u64::MAX - 1, u64::MAX - 1))
                .unwrap()
                .into_successors(),
            (u64::MAX, u64::MAX)
        );
    }
}
