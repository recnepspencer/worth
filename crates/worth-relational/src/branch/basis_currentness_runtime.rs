use crate::branch::{
    basis::AdmittedRelationalBranchBasis, basis_denial::RelationalBranchBasisDenial,
};
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub(crate) fn compare_current_exact(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<(), RelationalBranchBasisDenial> {
        super::basis_currentness::compare_current_exact(
            &self.history,
            &self.config.schema.registry,
            self.runtime_instance_id(),
            basis,
        )
    }
}
