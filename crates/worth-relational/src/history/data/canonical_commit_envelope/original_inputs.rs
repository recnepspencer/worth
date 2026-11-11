//! Original artifact attribution is installed once before positioned emission.
use super::CanonicalCommitEnvelope;
use crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs;

impl CanonicalCommitEnvelope {
    pub(crate) fn recorded_original_inputs(&self) -> Option<&RecordedCanonicalInputs> {
        self.original_inputs.get().and_then(Option::as_ref)
    }

    /// Explicit standalone None is sealed too: a prepared DATA commitment
    /// cannot subsequently acquire an input attribution through another Arc.
    pub(crate) fn seal_recorded_original_inputs(&self) {
        self.original_inputs.get_or_init(|| None);
    }

    pub(crate) fn retain_recorded_original_inputs(
        &self,
        inputs: RecordedCanonicalInputs,
    ) -> Result<(), &'static str> {
        if let Some(original) = self.original_inputs.get() {
            return (original.as_ref() == Some(&inputs))
                .then_some(())
                .ok_or("canonical original input attribution changed");
        }
        if let Err(inputs) = self.original_inputs.set(Some(inputs)) {
            return (self.original_inputs.get() == Some(&inputs))
                .then_some(())
                .ok_or("canonical original input attribution changed concurrently");
        }
        Ok(())
    }
}

mod observation;
pub use observation::{RelationalIntroducedSchemaObservation, RelationalOriginalInputObservation};
