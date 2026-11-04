//! Walk retained checkpoint owners without cloning or discarding their original data.
use super::*;
impl NativeSourceGraphRetirement {
    pub(super) fn advance_checkpoint_directory(
        &mut self,
        runtime: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), RelationalSourceCloseDenial> {
        let canonical = matches!(self.stage, Stage::RetainedCanonicalDirectory);
        let done = if !self.pass.retires() {
            if !runtime
                .durability
                .preflight_checkpoint_directory_retirement(self.checkpoint, canonical, source)?
            {
                self.stage = Stage::Complete;
                return Ok(());
            }
            true
        } else if let Some(handoff) = &mut self.directory {
            handoff.observe(request);
            runtime.durability.return_checkpoint_directory_retirement(
                self.checkpoint,
                canonical,
                &mut self.directory,
            )?
        } else {
            match runtime.durability.loan_checkpoint_directory_retirement(
                self.checkpoint,
                canonical,
                &mut self.directory,
            )? {
                Some(done) => done,
                None => {
                    self.stage = Stage::Complete;
                    return Ok(());
                }
            }
        };
        if done {
            self.stage = if canonical {
                self.checkpoint += 1;
                Stage::RetainedDirectory
            } else {
                Stage::RetainedCanonicalDirectory
            };
        }
        Ok(())
    }
}
