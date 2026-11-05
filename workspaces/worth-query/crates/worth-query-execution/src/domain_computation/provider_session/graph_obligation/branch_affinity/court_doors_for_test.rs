//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryGraphWorkBranchAffinity {
    #[doc(hidden)]
    pub fn admits_snapshot_for_test(&self, snapshot: &SnapshotHandle) -> bool {
        Self::admits_snapshot(self, snapshot)
    }
}

impl WorthQueryGraphWorkBranchAffinity {
    #[doc(hidden)]
    pub fn from_snapshot_for_test(snapshot: &SnapshotHandle) -> Self {
        Self::from_snapshot(snapshot)
    }
}

impl WorthQueryGraphWorkBranchAffinity {
    #[doc(hidden)]
    pub fn relational_for_test(&self) -> &BranchId {
        Self::relational(self)
    }
}

impl WorthQueryGraphWorkBranchAffinity {
    #[doc(hidden)]
    pub fn truth_for_test(&self) -> &TruthBranchIdentity {
        Self::truth(self)
    }
}
