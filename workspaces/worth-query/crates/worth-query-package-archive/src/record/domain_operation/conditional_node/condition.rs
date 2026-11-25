use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_installation::facade::*;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::foundational_value::{decode_aspect_value, write_aspect_value};
use crate::record::sequence::{decode_sequence, write_sequence};

use super::dependency::{decode_dependency, write_dependency};

pub(super) fn write_condition(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryConditionalEvaluationCondition,
) -> Result<(), Denial> {
    match value.class() {
        WorthQueryConditionalConditionClass::AlwaysEligible => output.u16(1),
        WorthQueryConditionalConditionClass::AspectFiltered => {
            output.u16(2)?;
            write_sequence(output, value.dependencies(), write_dependency)
        }
        WorthQueryConditionalConditionClass::DeltaThreshold => {
            output.u16(3)?;
            let (dependency, threshold) = value.delta_threshold_contract().ok_or_else(invalid)?;
            write_dependency(output, dependency)?;
            write_aspect_value(output, threshold.value())?;
            output.text(threshold.unit().as_str())?;
            output.u16(quantity_family_tag(threshold.value_family()))?;
            output.u16(match threshold.comparison_domain() {
                WorthQueryDeltaComparisonDomain::AbsoluteDifference => 1,
                WorthQueryDeltaComparisonDomain::RelativeRatio => 2,
            })?;
            output.u16(match threshold.boundary() {
                WorthQueryThresholdBoundary::Inclusive => 1,
                WorthQueryThresholdBoundary::Exclusive => 2,
            })
        }
        WorthQueryConditionalConditionClass::OnDemand => output.u16(4),
        WorthQueryConditionalConditionClass::Temporal => {
            output.u16(5)?;
            write_temporal(output, value.temporal_condition().ok_or_else(invalid)?)
        }
        WorthQueryConditionalConditionClass::DomainSpecific => {
            output.u16(6)?;
            output.text(
                value
                    .portable_family_identity()
                    .ok_or_else(invalid)?
                    .as_str(),
            )?;
            write_sequence(output, value.domain_specific_parameters(), write_parameter)
        }
    }
}

pub(super) fn decode_condition(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryConditionalEvaluationCondition, Denial> {
    let parts = match input.u16()? {
        1 => WorthQueryPortableConditionalConditionParts::AlwaysEligible,
        2 => WorthQueryPortableConditionalConditionParts::AspectFiltered(decode_sequence(
            input,
            budget,
            20,
            decode_dependency,
        )?),
        3 => {
            let dependency = decode_dependency(input, budget)?;
            let value = decode_aspect_value(input)?;
            let unit = decode_family(input)?;
            let value_family = quantity_family(input.u16()?)?;
            let comparison_domain = match input.u16()? {
                1 => WorthQueryDeltaComparisonDomain::AbsoluteDifference,
                2 => WorthQueryDeltaComparisonDomain::RelativeRatio,
                _ => return unsupported(),
            };
            let boundary = match input.u16()? {
                1 => WorthQueryThresholdBoundary::Inclusive,
                2 => WorthQueryThresholdBoundary::Exclusive,
                _ => return unsupported(),
            };
            let threshold = WorthQueryDeltaThreshold::from_untrusted_parts(
                ARCHIVE_DECODE,
                WorthQueryPortableDeltaThresholdParts {
                    value,
                    unit,
                    value_family,
                    comparison_domain,
                    boundary,
                },
            )
            .map_err(|_| invalid())?;
            WorthQueryPortableConditionalConditionParts::DeltaThreshold {
                dependency,
                threshold,
            }
        }
        4 => WorthQueryPortableConditionalConditionParts::OnDemand,
        5 => WorthQueryPortableConditionalConditionParts::Temporal(decode_temporal(input)?),
        6 => WorthQueryPortableConditionalConditionParts::DomainSpecific {
            family: decode_family(input)?,
            parameters: decode_sequence(input, budget, 6, decode_parameter)?,
        },
        _ => return unsupported(),
    };
    Ok(WorthQueryConditionalEvaluationCondition::from_untrusted_parts(ARCHIVE_DECODE, parts))
}

fn write_parameter(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryPortableConditionParameter,
) -> Result<(), Denial> {
    output.text(value.name())?;
    match value.value() {
        WorthQueryPortableConditionParameterValue::Bool(value) => {
            output.u16(1)?;
            output.u16(u16::from(*value))
        }
        WorthQueryPortableConditionParameterValue::U64(value) => {
            output.u16(2)?;
            output.u64(*value)
        }
        WorthQueryPortableConditionParameterValue::I64(value) => {
            output.u16(3)?;
            output.i64(*value)
        }
        WorthQueryPortableConditionParameterValue::Text(value) => {
            output.u16(4)?;
            output.text(value)
        }
        WorthQueryPortableConditionParameterValue::NativeValue(value) => {
            output.u16(5)?;
            write_aspect_value(output, value)
        }
    }
}

fn decode_parameter(
    input: &mut BinaryInput<'_>,
    _budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortableConditionParameter, Denial> {
    let name = input.text()?.to_owned();
    match input.u16()? {
        1 => WorthQueryPortableConditionParameter::bool(name, decode_bool(input)?),
        2 => WorthQueryPortableConditionParameter::u64(name, input.u64()?),
        3 => WorthQueryPortableConditionParameter::i64(name, input.i64()?),
        4 => WorthQueryPortableConditionParameter::text(name, input.text()?.to_owned()),
        5 => WorthQueryPortableConditionParameter::native_value(name, decode_aspect_value(input)?),
        _ => return unsupported(),
    }
    .map_err(|_| invalid())
}

fn write_temporal(
    output: &mut dyn BinaryEncodingSink,
    value: WorthQueryTemporalCondition,
) -> Result<(), Denial> {
    let (tag, number) = match value {
        WorthQueryTemporalCondition::AfterNanoseconds(value) => (1, Some(value)),
        WorthQueryTemporalCondition::AtOrAfterUnixNanoseconds(value) => (2, Some(value)),
        WorthQueryTemporalCondition::DebounceNanoseconds(value) => (3, Some(value)),
        WorthQueryTemporalCondition::ThrottleNanoseconds(value) => (4, Some(value)),
        WorthQueryTemporalCondition::StaleAfterNanoseconds(value) => (5, Some(value)),
        WorthQueryTemporalCondition::IntervalNanoseconds(value) => (6, Some(value)),
        WorthQueryTemporalCondition::SnapshotAdvance => (7, None),
    };
    output.u16(tag)?;
    if let Some(number) = number {
        output.u64(number)?;
    }
    Ok(())
}

fn decode_temporal(input: &mut BinaryInput<'_>) -> Result<WorthQueryTemporalCondition, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryTemporalCondition::AfterNanoseconds(input.u64()?)),
        2 => Ok(WorthQueryTemporalCondition::AtOrAfterUnixNanoseconds(
            input.u64()?,
        )),
        3 => Ok(WorthQueryTemporalCondition::DebounceNanoseconds(
            input.u64()?,
        )),
        4 => Ok(WorthQueryTemporalCondition::ThrottleNanoseconds(
            input.u64()?,
        )),
        5 => Ok(WorthQueryTemporalCondition::StaleAfterNanoseconds(
            input.u64()?,
        )),
        6 => Ok(WorthQueryTemporalCondition::IntervalNanoseconds(
            input.u64()?,
        )),
        7 => Ok(WorthQueryTemporalCondition::SnapshotAdvance),
        _ => unsupported(),
    }
}

fn decode_family(input: &mut BinaryInput<'_>) -> Result<WorthQueryTypedFamilyIdentity, Denial> {
    WorthQueryTypedFamilyIdentity::from_untrusted_portable_identity(
        ARCHIVE_DECODE,
        input.text()?.to_owned(),
    )
    .map_err(|_| invalid())
}

fn quantity_family_tag(value: WorthQueryQuantityValueFamily) -> u16 {
    match value {
        WorthQueryQuantityValueFamily::Integer => 1,
        WorthQueryQuantityValueFamily::Float32 => 2,
        WorthQueryQuantityValueFamily::Float64 => 3,
    }
}

fn quantity_family(tag: u16) -> Result<WorthQueryQuantityValueFamily, Denial> {
    match tag {
        1 => Ok(WorthQueryQuantityValueFamily::Integer),
        2 => Ok(WorthQueryQuantityValueFamily::Float32),
        3 => Ok(WorthQueryQuantityValueFamily::Float64),
        _ => unsupported(),
    }
}

fn decode_bool(input: &mut BinaryInput<'_>) -> Result<bool, Denial> {
    match input.u16()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}

fn unsupported<T>() -> Result<T, Denial> {
    Err(Denial::new(Kind::UnsupportedRecordVariant))
}

fn invalid() -> Denial {
    Denial::new(Kind::InvalidRecordShape)
}
