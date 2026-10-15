use worth_query_declaration::facade::application_capability::{
    ErasedApplicationCapabilityContract, WorthQueryPortableApplicationCapabilityContractParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;

use super::super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::wire_vocabulary::{decode_type_identity, write_type_identity};

mod bindings;
mod composition;
mod elevation;
mod resource_path;
mod scope;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    contract: &ErasedApplicationCapabilityContract,
) -> Result<(), Denial> {
    let parts = contract.parts();
    output.text(&parts.name)?;
    write_type_identity(output, &parts.capability_type)?;
    output.text(&parts.operation)?;
    write_type_identity(output, &parts.operation_type)?;
    write_type_identity(output, &parts.input_type)?;
    output.text(&parts.grant_entity)?;
    scope::write_target(output, &parts.target)?;
    scope::write_constraints(output, &parts.constraints)?;
    scope::write_delegation(output, &parts.delegation)?;
    composition::write(output, &parts.composition)?;
    elevation::write(output, &parts.elevation)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    scoped_resource: bool,
) -> Result<ErasedApplicationCapabilityContract, Denial> {
    let name = input.text()?.to_owned();
    let capability_type = decode_type_identity(input)?;
    let operation = input.text()?.to_owned();
    let operation_type = decode_type_identity(input)?;
    let input_type = decode_type_identity(input)?;
    let grant_entity = input.text()?.to_owned();
    let target = scope::decode_target(input, budget, scoped_resource)?;
    let constraints = scope::decode_constraints(input)?;
    let delegation = scope::decode_delegation(input, budget)?;
    let composition = composition::decode(input, budget)?;
    let elevation = elevation::decode(input, budget)?;
    Ok(ErasedApplicationCapabilityContract::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityContractParts {
            name,
            capability_type,
            operation,
            operation_type,
            input_type,
            grant_entity,
            target,
            constraints,
            delegation,
            composition,
            elevation,
        },
    ))
}
