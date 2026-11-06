//! Checked storage of the concrete lowered plans and copied native rules.
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::identity::data::KindId;
use crate::schema::{data::*, SchemaCompilationControl};

pub(crate) struct RelationIntegrityLoweringBacking {
    pub(crate) plan: usize,
    pub(crate) rules: usize,
    pub(crate) scratch: usize,
    pub(crate) registrations: usize,
}
impl RelationIntegrityDeclarations {
    pub(crate) fn lowering_backing(
        &self,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<RelationIntegrityLoweringBacking, Denial> {
        let mut source_count = 0usize;
        let mut target_count = 0usize;
        let mut leaves = 0usize;
        let mut rules = 0usize;
        let mut registrations = 0usize;
        for contract in &self.endpoint_kind_contracts {
            control.checkpoint()?;
            add(&mut source_count, contract.allowed_source_kinds.len())?;
            add(&mut target_count, contract.allowed_target_kinds.len())?;
            let mut owned = contract.contract_id.as_str().len();
            add(
                &mut owned,
                array::<KindId>(contract.allowed_source_kinds.len())?,
            )?;
            add(
                &mut owned,
                array::<KindId>(contract.allowed_target_kinds.len())?,
            )?;
            add(&mut leaves, owned)?;
            add(&mut rules, owned)?;
            add(&mut registrations, 1)?;
        }
        let candidates = array::<KindId>(checked(source_count.checked_add(target_count))?)?;
        for contract in &self.cardinality_contracts {
            control.checkpoint()?;
            if contract.source_max.is_some()
                || contract.target_max.is_some()
                || contract.pair_max.is_some()
            {
                add(&mut leaves, contract.contract_id.as_str().len())?;
                add(&mut rules, contract.contract_id.as_str().len())?;
                add(&mut registrations, 1)?;
            }
            if contract.source_min.is_some()
                || contract.target_min.is_some()
                || contract.pair_min.is_some()
            {
                let owned = checked(contract.contract_id.as_str().len().checked_add(candidates))?;
                add(&mut leaves, owned)?;
                add(&mut rules, owned)?;
                add(&mut registrations, 1)?;
            }
        }
        for contract in &self.uniqueness_contracts {
            control.checkpoint()?;
            let owned = contract.contract_id.as_str().len();
            add(&mut leaves, owned)?;
            add(&mut rules, owned)?;
            add(&mut registrations, 1)?;
        }
        for contract in &self.symmetry_contracts {
            control.checkpoint()?;
            let owned = contract.contract_id.as_str().len();
            add(&mut leaves, owned)?;
            add(&mut rules, owned)?;
            add(&mut registrations, 1)?;
        }
        for contract in &self.endpoint_deletion_integrity_contracts {
            control.checkpoint()?;
            let owned = contract.contract_id.as_str().len();
            add(&mut leaves, owned)?;
            add(&mut rules, owned)?;
            add(&mut registrations, 1)?;
        }
        for contract in &self.acyclicity_contracts {
            control.checkpoint()?;
            let owned = contract.contract_id.as_str().len();
            add(&mut leaves, owned)?;
            add(&mut rules, owned)?;
            add(&mut registrations, 1)?;
        }
        for contract in &self.partition_isolation_contracts {
            control.checkpoint()?;
            let owned = contract.contract_id.as_str().len();
            add(&mut leaves, owned)?;
            add(&mut rules, owned)?;
            add(&mut registrations, 1)?;
        }
        for contract in &self.connectivity_minimum_contracts {
            control.checkpoint()?;
            let mut owned = contract.contract_id.as_str().len();
            add(&mut owned, array::<KindId>(contract.source_kind_ids.len())?)?;
            add(&mut owned, array::<KindId>(contract.target_kind_ids.len())?)?;
            add(&mut leaves, owned)?;
            add(&mut rules, owned)?;
            add(&mut registrations, 1)?;
        }
        if let Some(aggregate) = &self.aggregate_identity {
            control.checkpoint()?;
            add(
                &mut leaves,
                match &aggregate.source {
                    AggregateIdentitySource::Orbit(_) => 0,
                    AggregateIdentitySource::Components { relations } => {
                        array::<KindId>(relations.len())?
                    }
                    AggregateIdentitySource::DerivedOrbit { derived_orbit } => checked(
                        derived_orbit
                            .schema
                            .0
                            .len()
                            .checked_add(derived_orbit.name.len()),
                    )?,
                    AggregateIdentitySource::DerivedComponents { derived_components } => checked(
                        derived_components
                            .schema
                            .0
                            .len()
                            .checked_add(derived_components.name.len()),
                    )?,
                },
            )?;
        }
        let mut plan = leaves;
        add(
            &mut plan,
            array::<LoweredEndpointKindContract>(self.endpoint_kind_contracts.len())?,
        )?;
        add(
            &mut plan,
            array::<LoweredCardinalityMaximumContract>(self.cardinality_contracts.len())?,
        )?;
        add(
            &mut plan,
            array::<LoweredCardinalityMinimumContract>(self.cardinality_contracts.len())?,
        )?;
        add(
            &mut plan,
            array::<LoweredUniquenessContract>(self.uniqueness_contracts.len())?,
        )?;
        add(
            &mut plan,
            array::<LoweredSymmetryContract>(self.symmetry_contracts.len())?,
        )?;
        add(
            &mut plan,
            array::<LoweredEndpointDeletionIntegrityContract>(
                self.endpoint_deletion_integrity_contracts.len(),
            )?,
        )?;
        add(
            &mut plan,
            array::<LoweredAcyclicityContract>(self.acyclicity_contracts.len())?,
        )?;
        add(
            &mut plan,
            array::<LoweredPartitionIsolationContract>(self.partition_isolation_contracts.len())?,
        )?;
        add(
            &mut plan,
            array::<LoweredConnectivityMinimumContract>(self.connectivity_minimum_contracts.len())?,
        )?;
        Ok(RelationIntegrityLoweringBacking {
            plan,
            rules,
            scratch: candidates,
            registrations,
        })
    }
}
fn checked(value: Option<usize>) -> Result<usize, Denial> {
    value.ok_or(Denial::MemoryOverflow)
}
fn add(total: &mut usize, value: usize) -> Result<(), Denial> {
    *total = checked(total.checked_add(value))?;
    Ok(())
}
fn array<T>(count: usize) -> Result<usize, Denial> {
    checked(count.checked_mul(size_of::<T>()))
}
