//! Prepare a coherent receiver pair before its executable meaning is shared.
use super::*;
use crate::runtime::builder::RelationalRuntimeDefinition;
use crate::runtime::RelationalRuntimeConfig;

#[derive(Debug)]
pub(in crate::runtime) struct HomeConfigurationBacking {
    _memory: Option<ExecutionMemoryReservation>,
}

impl PreparedRelationalHomeProgramContracts {
    pub fn receiver_configuration(&self) -> Option<&RelationalRuntimeConfig> {
        self.configuration.as_deref()
    }

    /// Retain B's receiving configuration without installing it. The complete
    /// executable owner must still be private; no Target root can be rebound.
    /// Native services, symbols, history and medium remain the original owner.
    pub fn with_receiver_configuration(
        mut self,
        request: &ExecutionRequest<'_, '_>,
        definition: RelationalRuntimeDefinition,
    ) -> Result<Self, RelationalHomeProgramContractDenial> {
        use RelationalHomeProgramContractDenial as Denial;
        request.consult().map_err(Denial::Request)?;
        let mut configuration = definition.into_home_configuration()?;
        if !same_owner_policies(&self.receiver_epoch.config, &configuration) {
            return Err(Denial::UnsupportedConfigurationChange);
        }
        let meaning = Arc::get_mut(&mut self.meaning).ok_or(Denial::TargetAlreadyShared)?;
        let contracts = Arc::get_mut(&mut meaning.contracts).ok_or(Denial::TargetAlreadyShared)?;
        if contracts.home_configuration_backing.is_some() || self.configuration.is_some() {
            return Err(Denial::TargetAlreadyShared);
        }
        let fixed = arc_bytes::<RelationalRuntimeConfig>()?
            .checked_add(arc_bytes::<HomeConfigurationBacking>()?)
            .ok_or(Denial::CapacityOverflow)?;
        let bytes = meaning
            .schema
            .registry_copy_bytes(request)
            .map_err(super::schema_compilation_denial::translate)?
            .checked_add(fixed)
            .ok_or(Denial::CapacityOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        request.consult().map_err(Denial::Request)?;
        configuration.schema.registry = meaning.schema.registry().clone();
        contracts.home_configuration_backing =
            Some(Arc::new(HomeConfigurationBacking { _memory: memory }));
        self.configuration = Some(Arc::new(configuration));
        Ok(self)
    }
}

fn arc_bytes<T>() -> Result<u64, RelationalHomeProgramContractDenial> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
        .ok_or(RelationalHomeProgramContractDenial::CapacityOverflow)
}

// The per-commit record limit is the only Native construction policy that
// Query's Home program definition can override. Other owner policies describe
// live subsystems and cannot be changed by replacing a schema pair.
fn same_owner_policies(before: &RelationalRuntimeConfig, after: &RelationalRuntimeConfig) -> bool {
    let mut publication = before.publication.policy.clone();
    publication.max_patch_records_per_commit =
        after.publication.policy.max_patch_records_per_commit;
    before.profile == after.profile
        && before.execution == after.execution
        && before.diagnostics == after.diagnostics
        && before.history == after.history
        && before.schema.invariant_catalog == after.schema.invariant_catalog
        && before.schema.descriptor_semantics_policy == after.schema.descriptor_semantics_policy
        && before.schema.descriptor_canonical_basis_policy
            == after.schema.descriptor_canonical_basis_policy
        && before.commit_strategies == after.commit_strategies
        && before.identity == after.identity
        && before.storage == after.storage
        && before.visibility == after.visibility
        && publication == after.publication.policy
        && before.durability == after.durability
}
