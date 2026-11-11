//! Before-growth admission for the shared owned root-schema compiler.
pub(super) use super::quote_denial::SchemaCompilationQuoteDenial as Denial;
pub(super) use super::standard_tree_quote::retained_tree_bytes as tree;
use super::{compiled_retention::SchemaCompilationFunding, RelationalBranchRootSchemaAuthority};
use crate::schema::data::*;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey,
};

type ContractBasis = (AspectKey, AspectIdentity, AspectContractRevision);

pub(super) struct SchemaCompilationQuote {
    pub(super) retained: u64,
    pub(super) temporary: u64,
}

impl SchemaCompilationQuote {
    pub(super) fn for_target(
        registry: &RelationalSchemaRegistry,
        previous: &RelationalBranchRootSchemaAuthority,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, Denial> {
        control.checkpoint().map_err(Denial::Request)?;
        let mut retained = retained_base(registry, control)?;
        let mut contracts = previous.retained_aspect_contracts().len();
        let mut copied = 0u64;
        let mut keys = 0u64;
        for contract in previous.retained_aspect_contracts() {
            control.checkpoint().map_err(Denial::Request)?;
            add(
                &mut copied,
                size(contract.owned_allocation_capacity_bytes())?,
            )?;
            add(&mut keys, size(contract.key().as_str().len())?)?;
        }
        for declarations in registry
            .entity_kinds
            .values()
            .map(|kind| &kind.aspect_contract_declarations)
            .chain(
                registry
                    .relation_kinds
                    .values()
                    .map(|kind| &kind.aspect_contract_declarations),
            )
        {
            control.checkpoint().map_err(Denial::Request)?;
            contracts = checked(contracts.checked_add(declarations.aspects.len()))?;
            for aspect in &declarations.aspects {
                control.checkpoint().map_err(Denial::Request)?;
                let clone = size(aspect.contract.owned_allocation_capacity_bytes())?;
                add(&mut copied, clone)?; // retained portable contract
                add(&mut keys, size(aspect.contract.key().as_str().len())?)?;
            }
        }
        add(&mut retained, array::<AspectContract>(contracts)?)?;
        add(
            &mut retained,
            array::<usize>(checked(contracts.checked_mul(2))?)?,
        )?;
        add(&mut retained, copied)?;
        let mut temporary = 0;
        add(
            &mut temporary,
            tree::<ContractBasis, AspectContract>(contracts)?,
        )?;
        add(&mut temporary, keys)?;
        // Ordinal shrink may overlap old/new backing for one directory.
        add(&mut temporary, array::<usize>(contracts)?)?;
        // Arc collection materializes its fixed descriptor roster before the final Arc.
        add(
            &mut temporary,
            array::<RelationIntegrityPlanRevision>(registry.relation_kinds.len())?,
        )?;
        Ok(Self {
            retained,
            temporary,
        })
    }

    pub(super) fn for_exact(
        registry: &RelationalSchemaRegistry,
        contracts: &Vec<AspectContract>,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, Denial> {
        control.checkpoint().map_err(Denial::Request)?;
        let mut retained = retained_base(registry, control)?;
        add(
            &mut retained,
            array::<AspectContract>(contracts.capacity())?,
        )?;
        for contract in contracts {
            control.checkpoint().map_err(Denial::Request)?;
            add(
                &mut retained,
                size(contract.owned_allocation_capacity_bytes())?,
            )?;
        }
        add(
            &mut retained,
            array::<usize>(checked(contracts.len().checked_mul(2))?)?,
        )?;
        let mut temporary = array::<usize>(contracts.len())?;
        add(
            &mut temporary,
            array::<RelationIntegrityPlanRevision>(registry.relation_kinds.len())?,
        )?;
        Ok(Self {
            retained,
            temporary,
        })
    }

    pub(super) fn reserve(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Option<ExecutionMemoryReservation>,
            Option<ExecutionMemoryReservation>,
        ),
        Denial,
    > {
        let retained = request
            .reserve_retained_memory(self.retained)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let temporary = request
            .reserve_retained_memory(self.temporary)
            .map_err(|cause| Denial::Request(cause.into()))?;
        Ok((retained, temporary))
    }
}

fn retained_base(
    registry: &RelationalSchemaRegistry,
    control: crate::schema::SchemaCompilationControl<'_, '_>,
) -> Result<u64, Denial> {
    let mut retained = super::registry_quote::retained(registry, control)?;
    add(&mut retained, arc::<RelationalBranchRootSchemaAuthority>()?)?;
    add(&mut retained, arc::<RelationalSchemaRegistry>()?)?;
    add(
        &mut retained,
        size(
            super::aspect_plan_quote::retained_bytes(registry, control).map_err(Denial::Request)?,
        )?,
    )?;
    add(&mut retained, arc::<super::RetainedContracts>()?)?;
    add(&mut retained, arc::<SchemaCompilationFunding>()?)?;
    // Arc<[Revision]> includes its counts before the fixed element payload.
    add(
        &mut retained,
        array_arc::<RelationIntegrityPlanRevision>(registry.relation_kinds.len())?,
    )?;
    Ok(retained)
}

pub(super) fn checked<T>(value: Option<T>) -> Result<T, Denial> {
    value.ok_or(Denial::AllocationLayoutOverflow)
}
pub(super) fn size(bytes: usize) -> Result<u64, Denial> {
    u64::try_from(bytes).map_err(|_| Denial::AllocationLayoutOverflow)
}
pub(super) fn add(total: &mut u64, bytes: u64) -> Result<(), Denial> {
    *total = checked(total.checked_add(bytes))?;
    Ok(())
}
pub(super) fn array<T>(count: usize) -> Result<u64, Denial> {
    size(
        std::alloc::Layout::array::<T>(count)
            .map_err(|_| Denial::AllocationLayoutOverflow)?
            .size(),
    )
}
pub(super) fn arc<T>() -> Result<u64, Denial> {
    array_arc::<T>(1)
}
fn array_arc<T>(count: usize) -> Result<u64, Denial> {
    let layout = std::alloc::Layout::new::<[usize; 2]>()
        .extend(
            std::alloc::Layout::array::<T>(count).map_err(|_| Denial::AllocationLayoutOverflow)?,
        )
        .map_err(|_| Denial::AllocationLayoutOverflow)?
        .0
        .pad_to_align();
    size(layout.size())
}
