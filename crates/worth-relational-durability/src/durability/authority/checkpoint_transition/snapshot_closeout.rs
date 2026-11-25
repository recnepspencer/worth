//! Post-performance snapshot closeout preserves both commit identity and refusal.
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::visibility::snapshot_release::RelationalSnapshotReleaseDenial;

pub struct SnapshotCloseoutFailure {
    pub commit_id: CommitId,
    pub denial: RelationalSnapshotReleaseDenial,
}

pub fn closeout(
    commit_id: CommitId,
    release: Result<(), RelationalSnapshotReleaseDenial>,
) -> Result<(), SnapshotCloseoutFailure> {
    release.map_err(|denial| SnapshotCloseoutFailure { commit_id, denial })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn an_already_closed_performed_snapshot_preserves_its_commit_and_typed_denial() {
        let committed = CommitId(37);
        let release = Err(RelationalSnapshotReleaseDenial::UnknownSnapshot);
        let Err(SnapshotCloseoutFailure { commit_id, denial }) = closeout(committed, release)
        else {
            panic!("snapshot closeout cannot be relabelled as a pre-effect refusal");
        };
        assert_eq!(commit_id, committed);
        assert_eq!(denial, RelationalSnapshotReleaseDenial::UnknownSnapshot);
    }
}
