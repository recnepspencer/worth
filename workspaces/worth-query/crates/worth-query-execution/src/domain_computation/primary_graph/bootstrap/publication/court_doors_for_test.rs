//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphPublication {
    #[doc(hidden)]
    pub fn bootstrap_commit_id_for_test(&self) -> worth_relational::facade::history::CommitId {
        Self::bootstrap_commit_id(self)
    }
}
