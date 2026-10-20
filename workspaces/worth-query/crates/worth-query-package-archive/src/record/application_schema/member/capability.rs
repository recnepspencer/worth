use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::wire_vocabulary::{decode_type_identity, write_type_identity};

mod application_capability;
mod application_query;

pub(super) fn require_nesting_depth(
    member: &ApplicationSchemaMember,
    maximum_depth: u32,
) -> Result<(), Denial> {
    if let ApplicationSchemaMember::ApplicationQuery { definition } = member {
        application_query::require_nesting_depth(definition, maximum_depth)?;
    }
    Ok(())
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    match member {
        ApplicationSchemaMember::ApplicationQuery { definition } => {
            application_query::write(output, definition)
        }
        ApplicationSchemaMember::ApplicationCapability { contract } => {
            application_capability::write(output, contract)
        }
        ApplicationSchemaMember::ApplicationCapabilityContext {
            context,
            context_type,
        } => {
            output.text(context)?;
            write_type_identity(output, context_type)
        }
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
            context,
            context_type,
            slot,
            slot_type,
            entity,
        } => {
            output.text(context)?;
            write_type_identity(output, context_type)?;
            output.text(slot)?;
            write_type_identity(output, slot_type)?;
            output.text(entity)
        }
        ApplicationSchemaMember::ApplicationCapabilityProvenance {
            provenance,
            provenance_type,
        } => {
            output.text(provenance)?;
            write_type_identity(output, provenance_type)
        }
        _ => unreachable!("capability member dispatch is exhaustive"),
    }
}

pub(super) fn decode(
    tag: u16,
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    Ok(match tag {
        6 => ApplicationSchemaMember::ApplicationQuery {
            definition: application_query::decode(input, budget)?,
        },
        7 | 33 => ApplicationSchemaMember::ApplicationCapability {
            contract: application_capability::decode(input, budget, tag == 33)?,
        },
        8 => ApplicationSchemaMember::ApplicationCapabilityContext {
            context: input.text()?.to_owned(),
            context_type: decode_type_identity(input)?,
        },
        9 => ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
            context: input.text()?.to_owned(),
            context_type: decode_type_identity(input)?,
            slot: input.text()?.to_owned(),
            slot_type: decode_type_identity(input)?,
            entity: input.text()?.to_owned(),
        },
        10 => ApplicationSchemaMember::ApplicationCapabilityProvenance {
            provenance: input.text()?.to_owned(),
            provenance_type: decode_type_identity(input)?,
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}
