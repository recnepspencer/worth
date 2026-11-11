//! Source contracts retain full conditions and literal parameters, not installed identities.
use super::{threshold_value, Denial, Read, Write};
use crate::data::conditional_execution::{
    SignalConditionalArtifactReuse as Reuse, SignalConditionalCondition as Condition,
    SignalConditionalContractDefinition as Definition,
    SignalConditionalVersionComparator as Comparator, SignalDeltaThresholdContract,
    SignalThresholdBoundary, SignalThresholdComparisonDomain, SignalThresholdValueFamily,
};

pub(super) fn write(out: &mut Write<'_>, definition: &Definition) -> Result<(), Denial> {
    match &definition.condition {
        Condition::Always => out.byte(0)?,
        Condition::AspectFilter(mask) => {
            out.byte(1)?;
            out.mask(*mask)?;
        }
        Condition::DeltaThreshold(value) => {
            out.byte(2)?;
            threshold_value::write(out, value.threshold())?;
            out.text(value.unit_identity())?;
            out.byte(match value.value_family() {
                SignalThresholdValueFamily::Integer => 0,
                SignalThresholdValueFamily::Float32 => 1,
                SignalThresholdValueFamily::Float64 => 2,
            })?;
            out.byte(match value.comparison_domain() {
                SignalThresholdComparisonDomain::AbsoluteDifference => 0,
                SignalThresholdComparisonDomain::RelativeRatio => 1,
            })?;
            out.byte(match value.boundary() {
                SignalThresholdBoundary::Inclusive => 0,
                SignalThresholdBoundary::Exclusive => 1,
            })?;
        }
        Condition::OnDemand => out.byte(3)?,
        Condition::RuntimePredicate => out.byte(4)?,
        Condition::TemporalWake => out.byte(5)?,
    }
    out.mask(definition.dependency_aspects)?;
    out.mask(definition.trigger_aspects)?;
    write_comparator(out, definition.dependency_comparator)?;
    write_comparator(out, definition.output_comparator)?;
    out.byte(match definition.artifact_reuse {
        Reuse::NotReusable => 0,
        Reuse::DependencyAndOutputEquivalent => 1,
        Reuse::OutputEquivalent => 2,
        Reuse::RuntimeResolved => 3,
    })
}
pub(super) fn read(input: &mut Read<'_>) -> Result<Definition, Denial> {
    let condition = match input.byte()? {
        0 => Condition::Always,
        1 => Condition::AspectFilter(input.mask()?),
        2 => {
            let threshold = threshold_value::read(input)?;
            let unit = input.text()?;
            let family = match input.byte()? {
                0 => SignalThresholdValueFamily::Integer,
                1 => SignalThresholdValueFamily::Float32,
                2 => SignalThresholdValueFamily::Float64,
                _ => return Err(Denial::Malformed),
            };
            let domain = match input.byte()? {
                0 => SignalThresholdComparisonDomain::AbsoluteDifference,
                1 => SignalThresholdComparisonDomain::RelativeRatio,
                _ => return Err(Denial::Malformed),
            };
            let boundary = match input.byte()? {
                0 => SignalThresholdBoundary::Inclusive,
                1 => SignalThresholdBoundary::Exclusive,
                _ => return Err(Denial::Malformed),
            };
            Condition::DeltaThreshold(SignalDeltaThresholdContract::new(
                threshold, unit, family, domain, boundary,
            ))
        }
        3 => Condition::OnDemand,
        4 => Condition::RuntimePredicate,
        5 => Condition::TemporalWake,
        _ => return Err(Denial::Malformed),
    };
    Ok(Definition {
        condition,
        dependency_aspects: input.mask()?,
        trigger_aspects: input.mask()?,
        dependency_comparator: read_comparator(input)?,
        output_comparator: read_comparator(input)?,
        artifact_reuse: match input.byte()? {
            0 => Reuse::NotReusable,
            1 => Reuse::DependencyAndOutputEquivalent,
            2 => Reuse::OutputEquivalent,
            3 => Reuse::RuntimeResolved,
            _ => return Err(Denial::Malformed),
        },
    })
}
fn write_comparator(out: &mut Write<'_>, comparator: Comparator) -> Result<(), Denial> {
    match comparator {
        Comparator::Exact => out.byte(0),
        Comparator::Tolerance(value) => {
            out.byte(1)?;
            out.number(value)
        }
        Comparator::OutputIdentity => out.byte(2),
        Comparator::RuntimeResolved => out.byte(3),
    }
}
fn read_comparator(input: &mut Read<'_>) -> Result<Comparator, Denial> {
    Ok(match input.byte()? {
        0 => Comparator::Exact,
        1 => Comparator::Tolerance(input.number()?),
        2 => Comparator::OutputIdentity,
        3 => Comparator::RuntimeResolved,
        _ => return Err(Denial::Malformed),
    })
}
