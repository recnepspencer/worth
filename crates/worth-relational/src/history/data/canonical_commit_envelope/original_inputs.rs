//! Original artifact attribution is installed once before positioned emission.
use super::CanonicalCommitEnvelope;
use crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs;

impl CanonicalCommitEnvelope {
    pub(crate) fn recorded_original_inputs(&self) -> Option<&RecordedCanonicalInputs> {
        self.original_inputs.get()
    }

    pub(crate) fn retain_recorded_original_inputs(
        &self,
        inputs: RecordedCanonicalInputs,
    ) -> Result<(), &'static str> {
        if let Some(original) = self.original_inputs.get() {
            return (original == &inputs)
                .then_some(())
                .ok_or("canonical original input attribution changed");
        }
        if let Err(inputs) = self.original_inputs.set(inputs) {
            return (self.original_inputs.get() == Some(&inputs))
                .then_some(())
                .ok_or("canonical original input attribution changed concurrently");
        }
        Ok(())
    }
}
