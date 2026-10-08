use super::*;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    envelope: &Envelope,
) -> Result<(), Denial> {
    output.u16(match envelope.mode() {
        Mode::Synchronous => 1,
        Mode::Asynchronous => 2,
    })?;
    output.u16(match envelope.degradation() {
        None => 1,
        Some(Degradation::PartialResult) => 2,
    })?;
    output.u16(match envelope.partial_effect_posture() {
        PartialEffect::EffectFree => 1,
        PartialEffect::PartialEffectsMayRemain => 2,
    })?;
    output.u16(match envelope.yielded_state_posture() {
        YieldedState::NotYieldable => 1,
        YieldedState::ProviderCheckpoint => 2,
    })?;
    output.u16(match envelope.retained_progress_posture() {
        RetainedProgress::ReleaseAfterAttempt => 1,
        RetainedProgress::RetainAttemptCapacity => 2,
    })?;
    output.text(envelope.cancellation_safe_point().as_str())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    scale: ScaleLimits,
    resources: ResourceLimits,
    boundary: Boundary,
) -> Result<Envelope, Denial> {
    let mode = match input.u16()? {
        1 => Mode::Synchronous,
        2 => Mode::Asynchronous,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let degradation = match input.u16()? {
        1 => None,
        2 => Some(Degradation::PartialResult),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let partial_effect = match input.u16()? {
        1 => PartialEffect::EffectFree,
        2 => PartialEffect::PartialEffectsMayRemain,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let yielded_state = match input.u16()? {
        1 => YieldedState::NotYieldable,
        2 => YieldedState::ProviderCheckpoint,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let retained_progress = match input.u16()? {
        1 => RetainedProgress::ReleaseAfterAttempt,
        2 => RetainedProgress::RetainAttemptCapacity,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let safe_point = SafePoint::new(input.text()?.to_owned())
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let envelope = match boundary {
        Boundary::BoundedStep => Envelope::new(scale, resources, mode, degradation, safe_point),
        Boundary::Atomic => {
            if mode != Mode::Synchronous || degradation.is_some() {
                return Err(Denial::new(Kind::InvalidRecordShape));
            }
            Envelope::atomic(scale, resources, safe_point)
        }
    };
    Ok(envelope
        .with_partial_effect_posture(partial_effect)
        .with_yielded_state_posture(yielded_state)
        .with_retained_progress_posture(retained_progress))
}
