//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryManagedTruthReadRequest {
    #[doc(hidden)]
    pub fn from_relational_basis_for_test( relational_basis: RelationalBranchBasisDescriptor, packet: SnapshotReadPacket) -> Self {
        Self::from_relational_basis(relational_basis, packet)
    }
}
