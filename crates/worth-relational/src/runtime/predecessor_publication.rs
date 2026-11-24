//! Bind predecessor reads to this installed owner.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::CommitId;
use crate::mvcc::publication::application_publication::predecessor_basis;
use crate::mvcc::{
    RelationalPredecessorBasisDenial, RelationalPredecessorInput,
    RelationalRequiredOutputPublication,
};

impl super::RelationalRuntime {
    /// Requires genuine publication custody and the exact predecessor commit
    /// selected by the candidate's succession fence. This performs no current
    /// observation and adds no dependency or publication authority.
    ///
    /// A first publication or a cold restored root has no retained decision
    /// basis. Its canonical coordinate is reserved by origin.basis, but that
    /// descriptor never reconstructs versions or approximates with current state.
    /// Cost: logarithmic route-index lookup plus descriptor metadata, no truth copy.
    pub fn predecessor_publication_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: &AdmittedRelationalBranchBasis,
        predecessor: Option<&RelationalRequiredOutputPublication>,
        expected_predecessor: Option<CommitId>,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalPredecessorBasisDenial> {
        predecessor_basis::predecessor_publication_basis(
            self.runtime_instance_id(),
            &self.history,
            execution,
            selected,
            predecessor,
            expected_predecessor,
        )
    }

    /// Select genuine committed or pre-writer applied inputs from the same
    /// authenticated route retention. No input is reconstructed from C.
    pub fn predecessor_publication_input(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: &AdmittedRelationalBranchBasis,
        predecessor: Option<&RelationalRequiredOutputPublication>,
        expected_predecessor: Option<CommitId>,
    ) -> Result<RelationalPredecessorInput, RelationalPredecessorBasisDenial> {
        predecessor_basis::predecessor_publication_input(
            self.runtime_instance_id(),
            &self.history,
            execution,
            selected,
            predecessor,
            expected_predecessor,
        )
    }
}
