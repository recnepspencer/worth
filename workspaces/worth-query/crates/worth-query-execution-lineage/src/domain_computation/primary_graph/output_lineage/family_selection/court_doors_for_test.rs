//! Sealed test doors over this module's private members.
use super::*;

impl ObservedFamilySelection {
    /// This selection as if its read's own commit had applied `applied` and
    /// masked `replaced`: the shape a site 2 rebase certifies.
    #[doc(hidden)]
    pub fn with_overlay_for_test(
        &self,
        applied: impl IntoIterator<Item = worth_relational::facade::identity::EntityId>,
        replaced: &[(Option<TypeId>, [u8; 32])],
    ) -> Self {
        let port = self.live_port().expect("the selection's owners are live");
        Self::new(
            self.producer.clone(),
            self.sources.to_vec(),
            self.outcome.iter().copied(),
            applied,
            replaced,
            &port.lineage,
            &port.owner,
            &port.layout,
        )
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn resolve_current_family_overlaid_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        family: &str,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        replaced: &[(Option<TypeId>, [u8; 32])],
    ) -> Result<WorthQueryCurrentOutputFamilyResolution, CompanionPreflightStop> {
        self.resolve_current_family_overlaid(
            execution,
            runtime_authority,
            schema,
            scope,
            family,
            occurrence,
            generation,
            replaced,
        )
    }

    #[doc(hidden)]
    pub fn resolve_current_family_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        family: &str,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
    ) -> Result<WorthQueryCurrentOutputFamilyResolution, CompanionPreflightStop> {
        Self::resolve_current_family(
            self,
            execution,
            runtime_authority,
            schema,
            scope,
            family,
            occurrence,
            generation,
        )
    }
}
