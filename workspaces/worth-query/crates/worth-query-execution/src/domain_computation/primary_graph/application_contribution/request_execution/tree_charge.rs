//! An incremental tree keeps the full canonical build's contractual bill.
use super::*;

impl QueryDispatch<'_> {
    pub(in super::super) fn tree_stage<R>(
        &self,
        ceiling: Option<u64>,
        stage: impl FnOnce(&mut MapKernelContext<'_, '_>) -> R,
    ) -> Result<(R, ExecutionReport), WorkCeilingDenial> {
        match &self.form {
            DispatchForm::Leased(lease) if !lease.has_accounting() && ceiling.is_none() => {
                worth_execution::ExecutionRequest::leased(lease)
                    .run_native_with_checkpoint(stage)
                    .map(|(result, physical)| {
                        (
                            result,
                            ExecutionReport::native(
                                worth_foundational::ExecutionPosture::Serial,
                                physical,
                            ),
                        )
                    })
            }
            DispatchForm::Leased(lease) => {
                ExecutionWorkCeiling::new(ceiling).run_with_checkpoint(Some(lease), None, stage)
            }
        }
    }
}
