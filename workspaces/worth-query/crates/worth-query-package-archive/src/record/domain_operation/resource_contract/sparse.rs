use super::*;

// Five nonempty text fields, their u32 lengths, boundary, 32 presence tags,
// and the five posture tags. Present values add eight bytes each.
const MINIMUM_STRATEGY_BYTES: usize = 101;

pub(super) fn write_strategy(
    output: &mut dyn BinaryEncodingSink,
    strategy: &Strategy,
) -> Result<(), Denial> {
    output.text(strategy.name().as_str())?;
    let envelope = strategy.envelope();
    output.u16(match envelope.boundary() {
        Boundary::Atomic => 1,
        Boundary::BoundedStep => 2,
    })?;
    for axis in ScaleAxis::ALL {
        write_value(output, envelope.optional_scale_ceiling(axis))?;
    }
    for dimension in ResourceDimension::ALL {
        write_value(output, envelope.optional_resource_ceiling(dimension))?;
    }
    postures::write(output, envelope)?;
    let providers = strategy.provider_requirements();
    output.text(providers.provider().as_str())?;
    output.text(providers.access_product().as_str())?;
    output.text(providers.allocator().as_str())
}
fn write_value(output: &mut dyn BinaryEncodingSink, value: Option<u64>) -> Result<(), Denial> {
    output.u16(if value.is_some() { 2 } else { 1 })?;
    if let Some(value) = value {
        output.u64(value)?;
    }
    Ok(())
}
fn decode_value(input: &mut BinaryInput<'_>) -> Result<Option<u64>, Denial> {
    match input.u16()? {
        1 => Ok(None),
        2 => Ok(Some(input.u64()?)),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
fn decode_strategy(input: &mut BinaryInput<'_>) -> Result<Strategy, Denial> {
    let name = StrategyName::new(input.text()?.to_owned())
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let boundary = match input.u16()? {
        1 => Boundary::Atomic,
        2 => Boundary::BoundedStep,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let mut scale = ScaleLimits::selective();
    for axis in ScaleAxis::ALL {
        if let Some(value) = decode_value(input)? {
            scale = scale.with(axis, value);
        }
    }
    let mut resources = ResourceLimits::selective();
    for dimension in ResourceDimension::ALL {
        if let Some(value) = decode_value(input)? {
            resources = resources.with(dimension, value);
        }
    }
    let envelope = postures::decode(input, scale, resources, boundary)?;
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
pub(super) fn decode_contract(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ResourceContract, Denial> {
    let strategies = decode_sequence(input, budget, MINIMUM_STRATEGY_BYTES, |input, _| {
        decode_strategy(input)
    })?;
    if strategies
        .iter()
        .all(|strategy| strategy.envelope().boundary() == Boundary::BoundedStep)
        || strategies
            .windows(2)
            .any(|pair| pair[0].name() >= pair[1].name())
    {
        return Err(Denial::new(Kind::NonCanonicalRecordSequence));
    }
    ResourceContract::declared(strategies).map_err(|_| Denial::new(Kind::InvalidRecordShape))
}
