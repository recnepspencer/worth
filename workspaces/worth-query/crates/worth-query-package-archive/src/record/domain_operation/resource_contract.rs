use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily as SafePoint, WorthQueryExecutionBoundary as Boundary,
    WorthQueryExecutionDegradation as Degradation, WorthQueryExecutionMode as Mode,
    WorthQueryPartialEffectPosture as PartialEffect,
    WorthQueryResourceDimension as ResourceDimension,
    WorthQueryResourceLimitRequest as ResourceLimits,
    WorthQueryRetainedProgressPosture as RetainedProgress,
    WorthQuerySemanticScaleAxis as ScaleAxis, WorthQuerySemanticScaleRequest as ScaleLimits,
    WorthQueryYieldedStatePosture as YieldedState,
};
use worth_query_installation::facade::{
    WorthQueryExecutionAccessProductFamily as AccessProduct,
    WorthQueryExecutionAllocatorFamily as Allocator, WorthQueryExecutionProviderFamily as Provider,
    WorthQueryExecutionProviderRequirements as ProviderRequirements,
    WorthQueryExecutionResourceContract as ResourceContract,
    WorthQueryExecutionResourceEnvelope as Envelope,
    WorthQueryExecutionStrategyContract as Strategy,
    WorthQueryExecutionStrategyName as StrategyName,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::sequence::{decode_sequence, write_sequence};

mod postures;
mod sparse;

pub(super) fn write_resource_contract(
    output: &mut dyn BinaryEncodingSink,
    contract: &ResourceContract,
) -> Result<(), Denial> {
    match contract {
        ResourceContract::Undeclared => output.u16(1),
        ResourceContract::Declared { strategies } => {
            if strategies
                .iter()
                .any(|strategy| strategy.envelope().boundary() == Boundary::Atomic)
            {
                output.u16(4)?;
                return write_sequence(output, strategies, sparse::write_strategy);
            }
            let optional_work = strategies.iter().any(|strategy| {
                strategy
                    .envelope()
                    .optional_scale_ceiling(ScaleAxis::WorkItems)
                    .is_none()
            });
            output.u16(if optional_work { 3 } else { 2 })?;
            write_sequence(output, strategies, |output, strategy| {
                write_strategy_variant(output, strategy, optional_work)
            })
        }
    }
}

pub(super) fn decode_resource_contract(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ResourceContract, Denial> {
    match input.u16()? {
        1 => Ok(ResourceContract::Undeclared),
        4 => sparse::decode_contract(input, budget),
        tag @ (2 | 3) => {
            let optional_work = tag == 3;
            let strategies = decode_sequence(
                input,
                budget,
                if optional_work { 84 } else { 90 },
                |input, _| decode_strategy_variant(input, optional_work),
            )?;
            if optional_work
                && strategies.iter().all(|strategy| {
                    strategy
                        .envelope()
                        .optional_scale_ceiling(ScaleAxis::WorkItems)
                        .is_some()
                })
            {
                return Err(Denial::new(Kind::NonCanonicalRecordSequence));
            }
            if strategies
                .windows(2)
                .any(|pair| pair[0].name() >= pair[1].name())
            {
                return Err(Denial::new(Kind::NonCanonicalRecordSequence));
            }
            ResourceContract::declared(strategies)
                .map_err(|_| Denial::new(Kind::InvalidRecordShape))
        }
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

fn write_strategy_variant(
    output: &mut dyn BinaryEncodingSink,
    strategy: &Strategy,
    optional_work: bool,
) -> Result<(), Denial> {
    output.text(strategy.name().as_str())?;
    write_envelope(output, strategy.envelope(), optional_work)?;
    let providers = strategy.provider_requirements();
    output.text(providers.provider().as_str())?;
    output.text(providers.access_product().as_str())?;
    output.text(providers.allocator().as_str())
}

fn decode_strategy_variant(
    input: &mut BinaryInput<'_>,
    optional_work: bool,
) -> Result<Strategy, Denial> {
    let name = StrategyName::new(input.text()?.to_owned())
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let envelope = decode_envelope(input, optional_work)?;
    let providers = ProviderRequirements::new(
        Provider::new(input.text()?.to_owned())
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        AccessProduct::new(input.text()?.to_owned())
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        Allocator::new(input.text()?.to_owned())
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
    );
    Ok(Strategy::new(name, envelope, providers))
}

fn write_envelope(
    output: &mut dyn BinaryEncodingSink,
    envelope: &Envelope,
    optional_work: bool,
) -> Result<(), Denial> {
    let work_present = envelope
        .optional_scale_ceiling(ScaleAxis::WorkItems)
        .is_some();
    if optional_work {
        output.u16(if work_present { 2 } else { 1 })?;
    }
    for axis in ScaleAxis::ALL {
        if axis != ScaleAxis::WorkItems || work_present {
            output.u64(envelope.scale_ceiling(axis))?;
        }
    }
    for dimension in ResourceDimension::ALL {
        output.u64(envelope.resource_ceiling(dimension))?;
    }
    postures::write(output, envelope)
}

fn decode_envelope(input: &mut BinaryInput<'_>, optional_work: bool) -> Result<Envelope, Denial> {
    let work_present = if optional_work {
        match input.u16()? {
            1 => false,
            2 => true,
            _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
        }
    } else {
        true
    };
    let mut scale = ScaleLimits::bounded(input.u64()?);
    for axis in ScaleAxis::ALL.into_iter().skip(1) {
        if axis != ScaleAxis::WorkItems || work_present {
            scale = scale.with(axis, input.u64()?);
        }
    }
    if !work_present {
        scale = scale.without_work_budget();
    }
    let mut resources = ResourceLimits::bounded(input.u64()?);
    for dimension in ResourceDimension::ALL.into_iter().skip(1) {
        resources = resources.with(dimension, input.u64()?);
    }
    postures::decode(input, scale, resources, Boundary::BoundedStep)
}

#[cfg(test)]
mod tests;
