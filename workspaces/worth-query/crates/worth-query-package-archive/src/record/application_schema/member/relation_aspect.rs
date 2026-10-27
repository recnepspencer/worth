//! One explicit relation-owned aspect, never an entity-field wire alias.
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::application_schema::wire_vocabulary::{
    decode_optional, decode_presence, write_optional, write_presence,
};
use crate::record::sequence::{decode_sequence, write_sequence};
use crate::record::{decode_budget::RecordDecodeAttempt, foundational_aspect, foundational_value};
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};
use worth_foundational::facade::{AspectContractRevision, AspectIdentity};
use worth_query_declaration::facade::application_schema::{
    ApplicationRelationFieldDefinition, ErasedApplicationRelationAspectDefinition,
    WorthQueryPortableRelationAspectParts,
};

pub(super) fn require_supported_field_codec(
    definition: &ErasedApplicationRelationAspectDefinition,
) -> Result<(), Denial> {
    if definition
        .parts()
        .fields
        .iter()
        .any(|field| field.canonical.is_some())
    {
        return Err(Denial::new(Kind::UnsupportedCanonicalValueContract));
    }
    definition
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationRelationAspectDefinition,
) -> Result<(), Denial> {
    require_supported_field_codec(definition)?;
    let parts = definition.parts();
    for name in [&parts.relation, &parts.from, &parts.to, &parts.aspect] {
        output.text(name)?;
    }
    output.u64(parts.identity.0)?;
    output.u64(parts.revision.0)?;
    foundational_aspect::write_absence(output, parts.absence)?;
    write_sequence(output, &parts.fields, |output, field| {
        output.text(&field.field)?;
        write_presence(output, field.presence)?;
        foundational_aspect::write_scalar_type(output, field.scalar_family)?;
        output.text(&field.value_type)?;
        write_optional(output, field.unit.as_ref(), |output, value| {
            output.text(value)
        })?;
        write_optional(output, field.frame.as_ref(), |output, value| {
            output.text(value)
        })?;
        foundational_value::write_bool(output, field.writable)
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ErasedApplicationRelationAspectDefinition, Denial> {
    let relation = input.text()?.to_owned();
    let from = input.text()?.to_owned();
    let to = input.text()?.to_owned();
    let aspect = input.text()?.to_owned();
    let identity = AspectIdentity(input.u64()?);
    let revision = AspectContractRevision(input.u64()?);
    let absence = foundational_aspect::decode_absence(input)?;
    let fields = decode_sequence(input, budget, 14, |input, _| {
        Ok(ApplicationRelationFieldDefinition {
            field: input.text()?.to_owned(),
            presence: decode_presence(input)?,
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
            value_type: input.text()?.to_owned(),
            canonical: None,
            unit: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
            frame: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
            writable: foundational_value::decode_bool(input)?,
        })
    })?;
    let definition = ErasedApplicationRelationAspectDefinition::from_untrusted_parts(
        WorthQueryPortableRelationAspectParts {
            relation,
            from,
            to,
            aspect,
            identity,
            revision,
            absence,
            fields,
        },
    );
    definition
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    Ok(definition)
}
