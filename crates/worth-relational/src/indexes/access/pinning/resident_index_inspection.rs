//! Unit courts inspect resident index state without altering its admission.
use super::*;
use crate::indexes::data::DerivedIndexGeneration;
impl IndexAccess<'_> {
    pub(crate) fn generations_snapshot(&self) -> Vec<DerivedIndexGeneration> {
        self.runtime
            .indexes
            .all_generations()
            .into_iter()
            .map(|generation| generation.as_ref().clone())
            .collect()
    }
}
