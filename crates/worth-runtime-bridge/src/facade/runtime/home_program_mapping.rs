//! Configuration-only mappings over the incumbent InitialEmpty Home owners.
use super::*;

#[derive(Debug)]
pub enum BridgeHomeProgramMappingDenial {
    SourceNotAttached,
    CorrespondenceBusy,
    CorrespondenceInstalled,
    Mappings(crate::error::BridgeBuildError),
}

impl RuntimeBridge {
    /// Prepare a mapping snapshot without replacing the source, Signal owner,
    /// custody or execution reservations. Authored correspondence is unsupported
    /// by the Relational-only Home lane and prevents this preparation.
    pub fn with_uninstalled_home_program_mappings(
        &self,
        mappings: Vec<crate::mapping::BridgeMappingRegistration>,
        aspects: Vec<crate::mapping::BridgeAspectRegistration>,
    ) -> Result<Self, BridgeHomeProgramMappingDenial> {
        use BridgeHomeProgramMappingDenial as Denial;
        if self.source_adapter.is_none() || self.authoritative_source_profile.is_none() {
            return Err(Denial::SourceNotAttached);
        }
        let allocations = self
            .correspondence_allocations
            .try_read()
            .map_err(|_| Denial::CorrespondenceBusy)?;
        if !allocations.is_empty() || !self.semantic_dependency_registry.is_empty_for_home_stop() {
            return Err(Denial::CorrespondenceInstalled);
        }
        let mapping_registry = FrozenMappingRegistry::freeze(mappings).map_err(Denial::Mappings)?;
        let aspect_registry =
            FrozenAspectMappingRegistry::freeze(aspects).map_err(Denial::Mappings)?;
        let mut prepared = self.clone();
        prepared.mapping_registry = mapping_registry;
        prepared.aspect_registry = aspect_registry;
        Ok(prepared)
    }
}
