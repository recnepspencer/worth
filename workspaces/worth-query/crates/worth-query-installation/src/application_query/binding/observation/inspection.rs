//! Inspection meaning can be retained only from its actual installed binding.
use super::WorthQueryInstalledApplicationQueryBindingObservation;
use std::sync::Arc;
use worth_query_declaration::facade::application_query::ErasedApplicationQueryInspectionDefinition;

/// The exact installed binding and its immutable inspection interpretation.
/// This descriptive contract grants no permission to inspect an observation.
#[derive(Clone)]
pub struct WorthQueryInstalledApplicationInspectionContract {
    binding: WorthQueryInstalledApplicationQueryBindingObservation,
    meaning: Arc<crate::application_query::inspection::CompiledInspectionMeaning>,
}

impl WorthQueryInstalledApplicationInspectionContract {
    pub(super) fn retain(
        binding: &WorthQueryInstalledApplicationQueryBindingObservation,
    ) -> Option<Self> {
        Some(Self {
            meaning: binding.binding.inspection_meaning()?,
            binding: binding.clone(),
        })
    }

    /// The actual immutable binding that issued this interpretation.
    pub fn binding_observation(&self) -> &WorthQueryInstalledApplicationQueryBindingObservation {
        &self.binding
    }

    /// Pure metadata-disclosure meaning retained from that same binding.
    pub fn definition(&self) -> &ErasedApplicationQueryInspectionDefinition {
        &self.meaning.definition
    }

    /// The exact metadata-only obligation, without payload-query permission.
    pub fn graph_obligations(
        &self,
    ) -> crate::graph_obligation::WorthQueryInstalledGraphObligationInspection<'_> {
        self.meaning.obligations.inspect()
    }

    /// The six exact metadata slots prepared once by this binding's compiler.
    pub fn metadata_slots(
        &self,
    ) -> &[worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey; 6]
    {
        &self.meaning.slots
    }
}
