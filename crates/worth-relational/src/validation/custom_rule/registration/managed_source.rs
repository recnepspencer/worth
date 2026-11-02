//! An installed rule's physical Source requirements, not a scope callback.
use super::CustomInvariantRegistration;
use crate::derived_relation::DerivedRelationDenial;
use crate::validation::candidate_source::ManagedIndexSourceProgram;
mod execution;

impl CustomInvariantRegistration {
    /// Each program came from the selected schema compiler. The full declared
    /// read AND affected audience must cover it before registration can succeed.
    pub fn with_managed_index_sources(
        mut self,
        sources: Vec<ManagedIndexSourceProgram>,
    ) -> Result<Self, DerivedRelationDenial> {
        if !self.managed_sources.is_empty() {
            return Err(DerivedRelationDenial::ForeignDefinition);
        }
        if sources.is_empty() {
            return Ok(self);
        }
        let mut scope = None;
        for source in &sources {
            source.require_access(self.access_contract())?;
            if scope
                .replace(source.scope_kind())
                .is_some_and(|previous| previous != source.scope_kind())
            {
                return Err(DerivedRelationDenial::DiscontinuousEndpoints);
            }
        }
        self.managed_sources = sources.into();
        self.executable = std::sync::Arc::new(execution::SourceScopedRule::new(
            self.executable,
            self.managed_sources.clone(),
            self.descriptor.identity.clone(),
        ));
        Ok(self)
    }
}
