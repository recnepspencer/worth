use crate::data::error::SignalError;
use worth_execution::MapKernelContext;

pub(crate) fn checkpoint(
    work: Option<&mut MapKernelContext<'_, '_>>,
    units: usize,
) -> Result<(), SignalError> {
    if let Some(work) = work {
        let units = u64::try_from(units)
            .map_err(|_| SignalError::invalid_input("graph execution work overflow"))?;
        work.checkpoint(units)
            .map_err(SignalError::execution_checkpoint_stopped)?;
    }
    Ok(())
}
