use worth_foundational::facade::{AspectFieldLocator, AspectValue, PortableAspectContractBasis};
use worth_relational::facade::transactions::EntityReference;

pub(super) fn created_entity(key: &String, entity_name: &str) -> Option<usize> {
    key.capacity()
        .checked_add(key.len().checked_mul(2)?)?
        .checked_add(entity_name.len())
}

pub(super) fn relation(
    key: &String,
    from: &EntityReference,
    to: &EntityReference,
) -> Option<usize> {
    key.capacity()
        .checked_add(key.len())?
        .checked_add(entity_reference(from))?
        .checked_add(entity_reference(to))
}

pub(super) fn field_entry(
    locator: &AspectFieldLocator,
    field_value: &AspectValue,
    retains_locator: bool,
) -> Option<usize> {
    value(field_value).checked_add(if retains_locator {
        locator_width(locator)?
    } else {
        0
    })
}

pub(super) fn optional_field_entry(
    locator: &AspectFieldLocator,
    contract: &PortableAspectContractBasis,
    value: Option<&AspectValue>,
    retains_locator_and_contract: bool,
) -> Option<usize> {
    let value_width = value.map_or(0, self::value);
    value_width.checked_add(if retains_locator_and_contract {
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
    role.len()
        .checked_add(entity_name.len())?
        .checked_add(entity_reference(entity))
}

pub(super) fn value(value: &AspectValue) -> usize {
    value.owned_allocation_capacity_bytes()
}

pub(super) fn locator_width(locator: &AspectFieldLocator) -> Option<usize> {
    Some(locator.owned_allocation_capacity_bytes())
}

pub(super) fn contract_width(contract: &PortableAspectContractBasis) -> usize {
    contract.owned_allocation_capacity_bytes()
}

fn entity_reference(reference: &EntityReference) -> usize {
    match reference {
        EntityReference::Existing(_) => 0,
        EntityReference::Created(created) => created.client_key.as_raw_str().map_or(0, str::len),
    }
}
