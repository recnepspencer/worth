//! Read-only observation at the publication-custody receiver.
use super::RelationalRequiredOutputPublication;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
impl RelationalRequiredOutputPublication {
    /// Read-only lifetime observation; the returned owner holds no basis lease.
    #[doc(hidden)]
    pub fn observe_readmitted_basis_for_test(
        &self,
    ) -> Option<impl Fn() -> bool + Send + Sync + use<>> {
        self.readmitted_at.as_ref().map(|basis| {
            let weak = basis.owner_inner().downgrade();
            move || weak.strong_count() != 0
        })
    }
}
