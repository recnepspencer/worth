use serde::{Deserialize, Serialize};

use crate::history::data::CommitId;

/// Ordered immutable parentage for a committed history fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalCommitParentage {
    parents: Vec<CommitId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalCommitParentageDenial {
    DuplicateParent,
}

impl RelationalCommitParentage {
    pub(crate) fn from_ordered(
        parents: Vec<CommitId>,
    ) -> Result<Self, RelationalCommitParentageDenial> {
        for (index, parent) in parents.iter().enumerate() {
            if parents[..index].contains(parent) {
                return Err(RelationalCommitParentageDenial::DuplicateParent);
            }
        }
        Ok(Self { parents })
    }

    pub fn as_slice(&self) -> &[CommitId] {
        &self.parents
    }

    pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{RelationalCommitParentage, RelationalCommitParentageDenial};
    use crate::history::data::CommitId;

    #[test]
    fn parent_order_is_preserved_and_duplicates_are_denied() {
        let parentage = RelationalCommitParentage::from_ordered(vec![CommitId(7), CommitId(2)])
            .expect("ordered parents are retained");
        assert_eq!(parentage.as_slice(), &[CommitId(7), CommitId(2)]);
        assert!(matches!(
            RelationalCommitParentage::from_ordered(vec![CommitId(7), CommitId(7)]),
            Err(RelationalCommitParentageDenial::DuplicateParent)
        ));
    }
}
