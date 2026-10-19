//! Current record protocol carries whole contract laws without Required reconstruction.
use super::*;
use worth_foundational::facade::{
    aspects, AspectEquivalenceBasis as Equivalence, AspectMaskContract, OpaqueAspectType,
    ReferenceAspectType,
};

pub(in super::super) fn write_aspect_contract(
    output: &mut dyn BinaryEncodingSink,
    contract: &AspectContract,
) -> Result<(), Denial> {
    output.text(contract.key().as_str())?;
    output.u64(contract.identity().0)?;
    output.u64(contract.revision().0)?;
    write_absence(output, contract.absence())?;
    output.u16(evolution_tag(contract.evolution()))?;
    output.u16(match contract.equivalence() {
        Equivalence::ExactCanonicalValue => 1,
        Equivalence::DeclaredStructFields => 2,
        Equivalence::OpaqueIdentity => 3,
        Equivalence::ReferenceIdentity => 4,
        Equivalence::ContentIdentity => 5,
    })?;
    let masks = contract.masks();
    output.u16(if masks == &AspectMaskContract::scalar() {
        1
    } else if masks == &AspectMaskContract::opaque_diagnostic_only() {
        2
    } else {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    })?;
    match contract.shape() {
        AspectShape::Scalar(value) => {
            output.u16(1)?;
            write_scalar_type(output, *value)
        }
        AspectShape::Struct(shape) => {
            output.u16(2)?;
            write_sequence(output, shape.fields(), write_field_declaration)
        }
        AspectShape::Opaque(OpaqueAspectType::Token) => output.u16(3),
        AspectShape::Reference(ReferenceAspectType::Entity) => output.u16(4),
        AspectShape::Content => output.u16(5),
    }
}

pub(in super::super) fn decode_aspect_contract(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<AspectContract, Denial> {
    let key = AspectKey::new(input.text()?.to_owned())
        .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?;
    let identity = AspectIdentity(input.u64()?);
    let revision = AspectContractRevision(input.u64()?);
    let absence = decode_absence(input)?;
    let evolution = evolution_from_tag(input.u16()?)?;
    let equivalence = match input.u16()? {
        1 => Equivalence::ExactCanonicalValue,
        2 => Equivalence::DeclaredStructFields,
        3 => Equivalence::OpaqueIdentity,
        4 => Equivalence::ReferenceIdentity,
        5 => Equivalence::ContentIdentity,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let masks = match input.u16()? {
        1 => AspectMaskContract::scalar(),
        2 => AspectMaskContract::opaque_diagnostic_only(),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let tag = input.u16()?;
    let builder = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(identity)
        .at_revision(revision);
    Ok(match tag {
        1 => builder.scalar_with(
            decode_scalar_type(input)?,
            masks,
            absence,
            equivalence,
            evolution,
        ),
        2 => {
            let fields = decode_sequence(input, budget, 12, |input, _| {
                decode_field_declaration(input)
            })?;
            require_canonical_sequence_by(&fields, |field| field.key())?;
            let shape = StructAspectShape::new(fields)
                .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?;
            builder.struct_with(shape, masks, absence, equivalence, evolution)
        }
        3 => builder
            .opaque_with(
                OpaqueAspectType::Token,
                masks,
                absence,
                equivalence,
                evolution,
            )
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        4 => builder.reference_with(
            ReferenceAspectType::Entity,
            masks,
            absence,
            equivalence,
            evolution,
        ),
        5 => {
            let contract = builder.content_ref();
            if contract.absence() != absence
                || contract.evolution() != evolution
                || contract.equivalence() != equivalence
                || contract.masks() != &masks
            {
                return Err(Denial::new(Kind::UnsupportedRecordVariant));
            }
            contract
        }
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}
