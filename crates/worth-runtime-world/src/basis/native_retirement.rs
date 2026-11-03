//! Original Native basis phase and its actual retired allocation.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::{
        AdmittedRelationalBranchBasis, RelationalBranchBasisRetirement,
        RelationalBranchBasisRetirementDenial, RelationalBranchBasisRetirementProgress,
        RetiredRelationalBranchBasis,
    },
    runtime::RelationalRuntime,
};
pub(crate) enum NativeBasisRetirement {
    Retiring(Option<RelationalBranchBasisRetirement>),
    Retired {
        _original: RetiredRelationalBranchBasis,
    },
}
impl NativeBasisRetirement {
    pub(crate) fn new(basis: AdmittedRelationalBranchBasis) -> Self {
        Self::Retiring(Some(basis.into_retirement()))
    }
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, RelationalBranchBasisRetirementDenial> {
        match self {
            Self::Retired { .. } => Ok(true),
            Self::Retiring(original) => {
                if original
                    .as_mut()
                    .expect("original task remains owned")
                    .advance(request, native)?
                    != RelationalBranchBasisRetirementProgress::Complete
                {
                    return Ok(false);
                }
                let original = original
                    .take()
                    .expect("complete original task")
                    .into_retired()
                    .unwrap_or_else(|_| unreachable!("Native reported Complete"));
                *self = Self::Retired {
                    _original: original,
                };
                Ok(true)
            }
        }
    }
}
