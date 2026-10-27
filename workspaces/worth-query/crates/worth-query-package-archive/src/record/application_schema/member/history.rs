//! Portable history meaning; decoding never captures a subject or an observation.
use worth_foundational::facade::{AspectContractRevision, AspectIdentity};
use worth_query_declaration::facade::application_query::{
    ApplicationHistoryAspectSelection, ApplicationHistoryTarget,
    ErasedApplicationHistoryQueryDefinition, WorthQueryPortableApplicationHistoryQueryParts,
};

use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::application_schema::{
    query_binding_reference,
    wire_vocabulary::{decode_type_identity, write_type_identity},
};
use crate::record::sequence::{decode_sequence, require_canonical_sequence, write_sequence};
use crate::record::{application_query, decode_budget::RecordDecodeAttempt};
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationHistoryQueryDefinition,
) -> Result<(), Denial> {
    definition
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let parts = definition.parts();
    output.text(&parts.name)?;
    write_type_identity(output, &parts.history_type)?;
    query_binding_reference::write(output, &parts.subject_read)?;
    write_target(output, &parts.target)?;
    write_sequence(output, &parts.aspects, |output, aspect| {
        output.text(&aspect.aspect)?;
        output.u64(aspect.identity.0)?;
        output.u64(aspect.revision.0)
    })?;
    application_query::disclosure::write(output, &parts.metadata_disclosure)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ErasedApplicationHistoryQueryDefinition, Denial> {
    let name = input.text()?.to_owned();
    let history_type = decode_type_identity(input)?;
    let subject_read = query_binding_reference::decode(input)?;
    let target = decode_target(input)?;
    let aspects = decode_sequence(input, budget, 20, |input, _| {
        Ok(ApplicationHistoryAspectSelection {
            aspect: input.text()?.to_owned(),
            identity: AspectIdentity(input.u64()?),
            revision: AspectContractRevision(input.u64()?),
        })
    })?;
    require_canonical_sequence(&aspects)?;
    let metadata_disclosure = application_query::disclosure::decode(input, budget)?;
    let definition = ErasedApplicationHistoryQueryDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationHistoryQueryParts {
            name,
            history_type,
            subject_read,
            target,
            aspects,
            metadata_disclosure,
        },
    );
    definition
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    Ok(definition)
}

fn write_target(
    output: &mut dyn BinaryEncodingSink,
    target: &ApplicationHistoryTarget,
) -> Result<(), Denial> {
    match target {
        ApplicationHistoryTarget::Entity { entity } => {
            output.u16(1)?;
            output.text(entity)
        }
        ApplicationHistoryTarget::Relation {
            query,
            slot,
            relation,
            from,
            to,
            direction,
            cardinality,
            output_name,
        } => {
            output.u16(2)?;
            write_type_identity(output, query)?;
            write_type_identity(output, slot)?;
            for name in [relation, from, to] {
                output.text(name)?;
            }
            application_query::result_shape::write_traversal_direction(output, *direction)?;
            application_query::controls::write_cardinality(output, *cardinality)?;
            output.text(output_name)
        }
    }
}

fn decode_target(input: &mut BinaryInput<'_>) -> Result<ApplicationHistoryTarget, Denial> {
    Ok(match input.u16()? {
        1 => ApplicationHistoryTarget::Entity {
            entity: input.text()?.to_owned(),
        },
        2 => ApplicationHistoryTarget::Relation {
            query: decode_type_identity(input)?,
            slot: decode_type_identity(input)?,
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
            direction: application_query::result_shape::decode_traversal_direction(input)?,
            cardinality: application_query::controls::decode_cardinality(input)?,
            output_name: input.text()?.to_owned(),
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}
