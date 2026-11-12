//! Directory callbacks return to the same original index or selection holder.
use super::*;
impl NativeSourceGraphRetirement {
    pub(super) fn advance_directory(
        &mut self,
        runtime: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), RelationalSourceCloseDenial> {
        use RelationalSourceCloseDenial::Checkpoint;
        let selected = matches!(self.stage, Stage::SelectedDirectory);
        let done = if !self.pass.retires() {
            if selected {
                runtime
                    .durability
                    .checkpoint_selection()
                    .preflight_source_retirement(source)
                    .map_err(Checkpoint)?;
            }
            true
        } else if let Some(handoff) = &mut self.directory {
            handoff.observe(request);
            if selected {
                runtime
                    .durability
                    .checkpoint_selection()
                    .return_source_retirement(&mut self.directory)
            } else {
                runtime.indexes.return_source_directory(&mut self.directory)
            }
            .map_err(Checkpoint)?
        } else {
            if selected {
                runtime
                    .durability
                    .checkpoint_selection()
                    .loan_source_retirement(&mut self.directory)
            } else {
                runtime.indexes.loan_source_directory(&mut self.directory)
            }
            .map_err(Checkpoint)?
        };
        if done {
            self.stage = if selected {
                Stage::RetainedDirectory
            } else {
                Stage::SelectedDirectory
            };
        }
        Ok(())
    }
}
