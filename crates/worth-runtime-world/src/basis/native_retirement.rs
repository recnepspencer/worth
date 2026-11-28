//! Original Native basis phase and its actual retired allocation.
use worth_execution::{CooperativeRetirementState, CooperativeStep, ExecutionRequest};
use worth_relational::facade::{
    branch::{
        AdmittedRelationalBranchBasis, RelationalBranchBasisRetirement,
        RelationalBranchBasisRetirementDenial as Denial, RelationalBranchBasisRetirementProgress,
        RetiredRelationalBranchBasis,
    },
    runtime::RelationalRuntime,
};

#[derive(Debug)]
pub(crate) struct NativeBasisRetirement(
    CooperativeRetirementState<RelationalBranchBasisRetirement, RetiredRelationalBranchBasis>,
);

impl NativeBasisRetirement {
    pub(crate) fn new(basis: AdmittedRelationalBranchBasis) -> Self {
        Self(CooperativeRetirementState::Retiring(Some(
            basis.into_retirement(),
        )))
    }

    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, Denial> {
        self.0
            .advance_with(
                request,
                |task, request| match task.advance(request, native) {
                    Ok(RelationalBranchBasisRetirementProgress::Complete) => {
                        Ok(CooperativeStep::Complete(()))
                    }
                    Ok(_) => Ok(CooperativeStep::Continue),
                    Err(Denial::Busy) => Ok(CooperativeStep::Busy),
                    Err(Denial::Execution(stop)) => Ok(CooperativeStep::Stopped(stop)),
                    Err(error) => Err(error),
                },
                |task| {
                    Ok(task
                        .into_retired()
                        .unwrap_or_else(|_| unreachable!("Native reported Complete")))
                },
            )?
            .into_progress(Denial::Busy, Denial::Execution)
            .map(|complete| complete.is_some())
    }
}
