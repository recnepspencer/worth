use worth_foundational::facade::{AspectFieldLocator, AspectValue, PortableAspectContractBasis};
use worth_relational::facade::transactions::EntityReference;

pub(super) fn created_entity(key: &String, entity_name: &str) -> Option<usize> {
    created_entity_bytes(key.capacity(), key.len(), entity_name)
}

pub(in crate::domain_computation::primary_graph) fn created_entity_bytes(
    key_capacity: usize,
    key_len: usize,
    entity_name: &str,
) -> Option<usize> {
    key_capacity
        .checked_add(key_len.checked_mul(2)?)?
        .checked_add(entity_name.len())
}

pub(super) fn relation(
    key: &String,
    from: &EntityReference,
    to: &EntityReference,
) -> Option<usize> {
    relation_bytes(key, entity_reference(from), entity_reference(to))
}

pub(in crate::domain_computation::primary_graph) fn relation_bytes(
    key: &String,
    from_bytes: usize,
    to_bytes: usize,
) -> Option<usize> {
    key.capacity()
        .checked_add(key.len())?
        .checked_add(from_bytes)?
        .checked_add(to_bytes)
}

pub(super) fn field_entry(
    locator: &AspectFieldLocator,
    field_value: &AspectValue,
    retains_locator: bool,
) -> Option<usize> {
    field_entry_bytes(locator, value(field_value), retains_locator)
}

pub(in crate::domain_computation::primary_graph) fn field_entry_bytes(
    locator: &AspectFieldLocator,
    value_bytes: usize,
    retains_locator: bool,
) -> Option<usize> {
    value_bytes.checked_add(if retains_locator {
        locator_width(locator)?
    } else {
        0
    })
}

/// An entry in an optional patch, including required fields stored there.
/// A cleared value still retains the entry's locator and contract.
pub(super) fn optional_field_entry(
    locator: &AspectFieldLocator,
    contract: &PortableAspectContractBasis,
    value: Option<&AspectValue>,
    retains_locator_and_contract: bool,
) -> Option<usize> {
    optional_field_entry_bytes(
        locator,
        contract,
        value.map_or(0, self::value),
        retains_locator_and_contract,
    )
}

pub(in crate::domain_computation::primary_graph) fn optional_field_entry_bytes(
    locator: &AspectFieldLocator,
    contract: &PortableAspectContractBasis,
    value_bytes: usize,
    retains_locator_and_contract: bool,
) -> Option<usize> {
    value_bytes.checked_add(if retains_locator_and_contract {
        locator_width(locator)?.checked_add(contract_width(contract))?
    } else {
        0
    })
}

pub(super) fn output_binding(
    role: &str,
    entity_name: &str,
    entity: &EntityReference,
) -> Option<usize> {
    output_binding_bytes(role, entity_name, entity_reference(entity))
}

pub(in crate::domain_computation::primary_graph) fn output_binding_bytes(
    role: &str,
    entity_name: &str,
    reference_bytes: usize,
) -> Option<usize> {
    role.len()
        .checked_add(entity_name.len())?
        .checked_add(reference_bytes)
}

pub(in crate::domain_computation::primary_graph) fn output_contract_bytes<Schema, Contract>(
) -> Option<usize>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Contract:
        worth_query_declaration::facade::application_operation::ApplicationMutationOutputContract<
            Schema,
        >,
{
    Contract::ROLES
        .iter()
        .map(|role| role.name().len())
        .chain(
            Contract::ROLE_FAMILIES
                .iter()
                .map(|family| family.prefix().len()),
        )
        .try_fold(0_usize, usize::checked_add)
}

pub(super) fn value(value: &AspectValue) -> usize {
    value.owned_allocation_capacity_bytes()
}

pub(super) fn locator_width(locator: &AspectFieldLocator) -> Option<usize> {
    Some(locator.owned_allocation_capacity_bytes())
}

pub(in crate::domain_computation::primary_graph) fn contract_width(
    contract: &PortableAspectContractBasis,
) -> usize {
    contract.owned_allocation_capacity_bytes()
}

fn entity_reference(reference: &EntityReference) -> usize {
    match reference {
        EntityReference::Existing(_) => 0,
        EntityReference::Created(created) => created.client_key.as_raw_str().map_or(0, str::len),
    }
}
