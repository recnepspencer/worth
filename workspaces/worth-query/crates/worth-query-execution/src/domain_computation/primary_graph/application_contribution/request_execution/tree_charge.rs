//! An incremental tree keeps the full canonical build's contractual bill.
use super::*;

impl QueryDispatch<'_> {
    pub(in super::super) fn tree_stage<R>(
        &self,
        ceiling: u64,
        stage: impl FnOnce(&mut MapKernelContext<'_, '_>) -> R,
    ) -> Result<(R, ExecutionReport), WorkCeilingDenial> {
        let ceiling = ExecutionWorkCeiling::new(ceiling);
        match &self.form {
            DispatchForm::Leased(lease) => ceiling.run_with_checkpoint(Some(lease), None, stage),
        }
    }
}
