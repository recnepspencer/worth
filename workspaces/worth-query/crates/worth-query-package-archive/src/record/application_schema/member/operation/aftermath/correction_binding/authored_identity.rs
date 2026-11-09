//! Authored family/key correspondence uses the existing schema-field wire owner.
#[cfg(test)]
mod tests;
use crate::record::application_schema::{
    member::schema,
    wire_vocabulary::{decode_optional, decode_type_identity, write_optional, write_type_identity},
};
use crate::record::sequence::{decode_sequence, write_sequence};
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::{
    application_aftermath::{
        DeclaredAuthoredCorrection, DeclaredAuthoredCorrectionParts, PortablePreImageRelationLocus,
    },
    application_operation::ApplicationMutationOutputPosture as Posture,
    application_schema::ApplicationSchemaMember,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    identity: &DeclaredAuthoredCorrection,
) -> Result<(), Denial> {
    let parts = identity.parts();
    schema::require_supported_field_codec(&parts.logical_key)?;
    let ApplicationSchemaMember::Field { frame, .. } = parts.logical_key.as_ref() else {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    };
    write_type_identity(output, &parts.family)?;
    output.text(&parts.source_role)?;
    output.text(&parts.inverse_role)?;
    output.u16(
        match parts.source_posture {
            Posture::Create => 0,
            Posture::Preserve => 1,
            Posture::Retire => 2,
        } + if identity.co_anchors().is_empty() && identity.member_owner().is_none() {
            0
        } else {
            3
        },
    )?;
    write_optional(output, parts.binding_unit.as_ref(), |output, value| {
        output.text(value)
    })?;
    output.u16(if frame.is_some() { 25 } else { 3 })?;
    schema::write(output, &parts.logical_key)?;
    if !identity.co_anchors().is_empty() || identity.member_owner().is_some() {
        write_optional(output, identity.member_owner(), |output, owner| {
            output.text(owner.relation())?;
            output.text(owner.from())?;
            output.text(owner.to())
        })?;
        write_sequence(output, identity.co_anchors(), |output, anchor| {
            output.u16(u16::from(anchor.is_optional()))?;
            write(output, anchor)
        })?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut crate::record::decode_budget::RecordDecodeAttempt,
) -> Result<DeclaredAuthoredCorrection, Denial> {
    decode_at_depth(input, budget, 0)
}

fn decode_at_depth(
    input: &mut BinaryInput<'_>,
    budget: &mut crate::record::decode_budget::RecordDecodeAttempt,
    depth: u32,
) -> Result<DeclaredAuthoredCorrection, Denial> {
    budget.require_nesting_depth(depth)?;
    if depth > 1 {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    let family = decode_type_identity(input)?;
    let source_role = input.text()?.to_owned();
    let inverse_role = input.text()?.to_owned();
    let posture = input.u16()?;
    let source_posture = match posture % 3 {
        0 => Posture::Create,
        1 => Posture::Preserve,
        2 => Posture::Retire,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let binding_unit = decode_optional(input, |input| Ok(input.text()?.to_owned()))?;
    let tag = input.u16()?;
    if !matches!(tag, 3 | 25) {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let logical_key = schema::decode(tag, input, budget)?;
    if posture > 5 {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let mut identity =
        DeclaredAuthoredCorrection::from_untrusted_parts(DeclaredAuthoredCorrectionParts {
            family,
            logical_key: Box::new(logical_key),
            binding_unit,
            source_role,
            inverse_role,
            source_posture,
        });
    if posture >= 3 {
        if let Some(owner) = decode_optional(input, |input| {
            Ok(PortablePreImageRelationLocus::from_untrusted_fields(
                input.text()?.to_owned(),
                input.text()?.to_owned(),
                input.text()?.to_owned(),
            ))
        })? {
            identity = identity.with_member_owner(owner);
        }
        for (optional, anchor) in decode_sequence(input, budget, 2, |input, budget| {
            let optional = match input.u16()? {
                0 => false,
                1 => true,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            };
            Ok((optional, decode_at_depth(input, budget, depth + 1)?))
        })? {
            identity = identity.with_co_anchor(anchor, optional);
        }
    }
    Ok(identity)
}
