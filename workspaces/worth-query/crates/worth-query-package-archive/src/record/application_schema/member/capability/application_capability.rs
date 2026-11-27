use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
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
mod extended_record;
mod resource_path;
mod scope;

use extended_record::requires_extended_record;

/// The capability member's record shape, one per member tag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CapabilityRecordShape {
    /// Tag 7: a direct grant resource on a workflow-tenure grant.
    Direct,
    /// Tag 33: a context-rooted resource path on a workflow-tenure grant.
    Rooted,
    /// Tag 59: standing tenure, extent status, UTC or local-date validity,
    /// or a self-grant rule.
    Extended,
}

impl CapabilityRecordShape {
    pub(super) fn of(contract: &ErasedApplicationCapabilityContract) -> Self {
        if requires_extended_record(contract) {
            Self::Extended
        } else if contract.target().resource_path().is_direct() {
            Self::Direct
        } else {
            Self::Rooted
        }
    }

    pub(super) const fn member_tag(self) -> u16 {
        match self {
            Self::Direct => 7,
            Self::Rooted => 33,
            Self::Extended => 59,
        }
    }
}

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
    match CapabilityRecordShape::of(contract) {
        CapabilityRecordShape::Direct | CapabilityRecordShape::Rooted => {
            scope::write_target(output, &parts.target)?;
            scope::write_constraints(output, &parts.constraints)?;
            scope::write_delegation(output, &parts.delegation)?;
        }
        CapabilityRecordShape::Extended => {
            extended_record::write_target(output, &parts.target)?;
            extended_record::write_constraints(output, &parts.constraints)?;
            extended_record::write_delegation(output, &parts.delegation)?;
        }
    }
    composition::write(output, &parts.composition)?;
    elevation::write(output, &parts.elevation)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    shape: CapabilityRecordShape,
) -> Result<ErasedApplicationCapabilityContract, Denial> {
    let name = input.text()?.to_owned();
    let capability_type = decode_type_identity(input)?;
    let operation = input.text()?.to_owned();
    let operation_type = decode_type_identity(input)?;
    let input_type = decode_type_identity(input)?;
    let grant_entity = input.text()?.to_owned();
    let (target, constraints, delegation) = match shape {
        CapabilityRecordShape::Direct | CapabilityRecordShape::Rooted => (
            scope::decode_target(input, budget, shape == CapabilityRecordShape::Rooted)?,
            scope::decode_constraints(input)?,
            scope::decode_delegation(input, budget)?,
        ),
        CapabilityRecordShape::Extended => (
            extended_record::decode_target(input, budget)?,
            extended_record::decode_constraints(input, budget)?,
            extended_record::decode_delegation(input, budget)?,
        ),
    };
    let composition = composition::decode(input, budget)?;
    let elevation = elevation::decode(input, budget)?;
    let contract = ErasedApplicationCapabilityContract::from_untrusted_parts(
        ARCHIVE_DECODE,
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
    );
    // One encoding per contract: a shape other than the writer's is refused.
    if CapabilityRecordShape::of(&contract) != shape {
        return Err(Denial::new(
            crate::denial::WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant,
        ));
    }
    Ok(contract)
}
