//! A retained patch coordinate identifies one canonical commit.
use super::PatchStreamPosition;
use crate::history::data::coordinates::CommitId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatchStreamCommitRef {
    pub position: PatchStreamPosition,
    pub commit_id: CommitId,
}
