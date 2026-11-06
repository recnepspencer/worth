//! Direct owned allocations adopted with an accepted declaration inventory.
use super::RelationIntegrityDeclarations;

impl RelationIntegrityDeclarations {
    pub(crate) fn retained_allocation_bytes(
        &self,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<Option<usize>, crate::execution::RelationalExecutionDenialCause> {
        let mut stopped = None;
        let bytes = self.measure_retained_bytes(&mut || match control.checkpoint() {
            Ok(()) => Some(()),
            Err(cause) => {
                stopped = Some(cause);
                None
            }
        });
        match stopped {
            Some(cause) => Err(cause),
            None => Ok(bytes),
        }
    }
    fn measure_retained_bytes(&self, checkpoint: &mut dyn FnMut() -> Option<()>) -> Option<usize> {
        let mut bytes = vector(&self.endpoint_kind_contracts)?
            .checked_add(vector(&self.cardinality_contracts)?)?
            .checked_add(vector(&self.uniqueness_contracts)?)?
            .checked_add(vector(&self.symmetry_contracts)?)?
            .checked_add(vector(&self.endpoint_deletion_integrity_contracts)?)?
            .checked_add(vector(&self.acyclicity_contracts)?)?
            .checked_add(vector(&self.partition_isolation_contracts)?)?
            .checked_add(vector(&self.connectivity_minimum_contracts)?)?;
        for contract in &self.endpoint_kind_contracts {
            checkpoint()?;
            bytes = bytes
                .checked_add(contract.contract_id.owned_allocation_capacity_bytes() as usize)?
                .checked_add(vector(&contract.allowed_source_kinds)?)?
                .checked_add(vector(&contract.allowed_target_kinds)?)?;
        }
        for contract in &self.cardinality_contracts {
            checkpoint()?;
            bytes = bytes
                .checked_add(contract.contract_id.owned_allocation_capacity_bytes() as usize)?;
        }
        for contract in &self.uniqueness_contracts {
            checkpoint()?;
            bytes = bytes
                .checked_add(contract.contract_id.owned_allocation_capacity_bytes() as usize)?;
        }
        for contract in &self.symmetry_contracts {
            checkpoint()?;
            bytes = bytes
                .checked_add(contract.contract_id.owned_allocation_capacity_bytes() as usize)?;
        }
        for contract in &self.endpoint_deletion_integrity_contracts {
            checkpoint()?;
            bytes = bytes
                .checked_add(contract.contract_id.owned_allocation_capacity_bytes() as usize)?;
        }
        for contract in &self.acyclicity_contracts {
            checkpoint()?;
            bytes = bytes
                .checked_add(contract.contract_id.owned_allocation_capacity_bytes() as usize)?;
        }
        for contract in &self.partition_isolation_contracts {
            checkpoint()?;
            bytes = bytes
                .checked_add(contract.contract_id.owned_allocation_capacity_bytes() as usize)?;
        }
        for contract in &self.connectivity_minimum_contracts {
            checkpoint()?;
            bytes = bytes
                .checked_add(contract.contract_id.owned_allocation_capacity_bytes() as usize)?
                .checked_add(vector(&contract.source_kind_ids)?)?
                .checked_add(vector(&contract.target_kind_ids)?)?;
        }
        if let Some(aggregate) = &self.aggregate_identity {
            bytes = bytes.checked_add(match &aggregate.source {
                super::AggregateIdentitySource::Orbit(_) => 0,
                super::AggregateIdentitySource::Components { relations } => vector(relations)?,
                super::AggregateIdentitySource::DerivedOrbit { derived_orbit } => derived_orbit
                    .schema
                    .0
                    .capacity()
                    .checked_add(derived_orbit.name.capacity())?,
                super::AggregateIdentitySource::DerivedComponents { derived_components } => {
                    derived_components
                        .schema
                        .0
                        .capacity()
                        .checked_add(derived_components.name.capacity())?
                }
            })?;
        }
        Some(bytes)
    }
}

fn vector<T>(values: &Vec<T>) -> Option<usize> {
    values.capacity().checked_mul(size_of::<T>())
}
