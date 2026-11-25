use std::collections::BTreeSet;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;

use worth_foundational::facade::{AspectBinding, AspectKey, FieldKey};
use worth_query_installation::facade::{
    WorthQueryPortableNativeAspectContractParts, WorthQueryPortableNativeAspectContractRecord,
    WorthQueryPortablePackageRecord,
};

use crate::binary_encoding::{BinaryEncodingMeasure, BinaryEncodingSink};
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::limits::WorthQueryPackageArchiveLimits;

use super::decode_budget::RecordDecodeAttempt;
use super::encoding_budget::RecordPayloadEncodingWork;
use super::sequence::{decode_sequence, require_canonical_sequence};

pub(super) fn payload_encoding_work(
    record: &WorthQueryPortableNativeAspectContractRecord,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<RecordPayloadEncodingWork, Denial> {
    let limits = limits.narrowed();
    let mut measure = BinaryEncodingMeasure::default();
    write_record(&mut measure, record)?;
    RecordPayloadEncodingWork::from_measure(&measure, limits)
}

pub(super) fn write_payload(
    record: &WorthQueryPortableNativeAspectContractRecord,
    output: &mut BinaryOutput,
) -> Result<(), Denial> {
    write_record(output, record)
}

pub(super) fn decode_payload(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortablePackageRecord, Denial> {
    let schema = input.text()?.to_owned();
    let entity = input.text()?.to_owned();
    let aspect = AspectKey::new(input.text()?.to_owned())
        .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?;
    let contract = super::foundational_aspect::decode_aspect_contract(input, budget)?;
    let fields = decode_fields(input, budget)?;
    let binding = decode_binding(input)?;
    Ok(WorthQueryPortablePackageRecord::NativeAspectContract(
        WorthQueryPortableNativeAspectContractRecord::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableNativeAspectContractParts {
                schema,
                entity,
                aspect,
                contract,
                fields,
                binding,
            },
        ),
    ))
}

fn write_record(
    output: &mut dyn BinaryEncodingSink,
    record: &WorthQueryPortableNativeAspectContractRecord,
) -> Result<(), Denial> {
    output.text(record.schema())?;
    output.text(record.entity())?;
    output.text(record.aspect().as_str())?;
    super::foundational_aspect::write_aspect_contract(output, record.contract())?;
    write_fields(output, record)?;
    write_binding(output, record.binding())
}

fn write_fields(
    output: &mut dyn BinaryEncodingSink,
    record: &WorthQueryPortableNativeAspectContractRecord,
) -> Result<(), Denial> {
    let fields = record.fields();
    let count = u32::try_from(fields.len()).map_err(|_| Denial::new(Kind::NumericWidthExceeded))?;
    output.claim_nested_entries(count)?;
    output.u32(count)?;
    for field in fields {
        output.text(field.as_str())?;
    }
    Ok(())
}

fn decode_fields(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<BTreeSet<FieldKey>, Denial> {
    let fields = decode_sequence(input, budget, 4, |input, _| {
        FieldKey::new(input.text()?.to_owned()).ok_or_else(|| Denial::new(Kind::InvalidRecordShape))
    })?;
    require_canonical_sequence(&fields)?;
    Ok(fields.into_iter().collect())
}

fn write_binding(
    output: &mut dyn BinaryEncodingSink,
    binding: &AspectBinding,
) -> Result<(), Denial> {
    match binding {
        AspectBinding::EntityField { field } => tagged_field(output, 1, field),
        AspectBinding::RelationField { field } => tagged_field(output, 2, field),
        AspectBinding::RelationSourceEndpoint => output.u16(3),
        AspectBinding::RelationTargetEndpoint => output.u16(4),
        AspectBinding::StructuralRegion => output.u16(5),
        AspectBinding::StructuralPartition => output.u16(6),
        AspectBinding::StructuralFacet => output.u16(7),
        AspectBinding::LifecycleTransition => output.u16(8),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

fn decode_binding(input: &mut BinaryInput<'_>) -> Result<AspectBinding, Denial> {
    Ok(match input.u16()? {
        1 => AspectBinding::EntityField {
            field: decode_field_key(input)?,
        },
        2 => AspectBinding::RelationField {
            field: decode_field_key(input)?,
        },
        3 => AspectBinding::RelationSourceEndpoint,
        4 => AspectBinding::RelationTargetEndpoint,
        5 => AspectBinding::StructuralRegion,
        6 => AspectBinding::StructuralPartition,
        7 => AspectBinding::StructuralFacet,
        8 => AspectBinding::LifecycleTransition,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

fn tagged_field(
    output: &mut dyn BinaryEncodingSink,
    tag: u16,
    field: &FieldKey,
) -> Result<(), Denial> {
    output.u16(tag)?;
    output.text(field.as_str())
}

fn decode_field_key(input: &mut BinaryInput<'_>) -> Result<FieldKey, Denial> {
    FieldKey::new(input.text()?.to_owned()).ok_or_else(|| Denial::new(Kind::InvalidRecordShape))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_version_one_aspect_binding_tag_round_trips() {
        let field = || FieldKey::new("binding-field").unwrap();
        let bindings = [
            AspectBinding::EntityField { field: field() },
            AspectBinding::RelationField { field: field() },
            AspectBinding::RelationSourceEndpoint,
            AspectBinding::RelationTargetEndpoint,
            AspectBinding::StructuralRegion,
            AspectBinding::StructuralPartition,
            AspectBinding::StructuralFacet,
            AspectBinding::LifecycleTransition,
        ];
        for (expected_tag, binding) in (1_u16..).zip(bindings) {
            let mut output = BinaryOutput::with_capacity(32);
            write_binding(&mut output, &binding).unwrap();
            let bytes = output.into_bytes();
            assert_eq!(
                u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
                expected_tag
            );
            let mut input = BinaryInput::new(&bytes);
            assert_eq!(decode_binding(&mut input), Ok(binding));
            assert!(input.is_finished());
        }
    }

    #[test]
    fn unknown_aspect_binding_tags_fail_closed() {
        for tag in [0_u16, 9, u16::MAX] {
            let bytes = tag.to_be_bytes();
            assert_eq!(
                decode_binding(&mut BinaryInput::new(&bytes))
                    .unwrap_err()
                    .kind(),
                Kind::UnsupportedRecordVariant
            );
        }
    }
}
