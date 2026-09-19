use super::super::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationFamily,
    UiHostObservationLoss, UiHostObservationSequenceRange,
};

pub(super) fn merge(
    previous: &UiHostObservationBatch,
    next: &UiHostObservationBatch,
) -> Option<UiHostObservationBatch> {
    for batch in [previous, next] {
        batch.validate_shape().ok()?;
        if batch.integrity()
            != super::super::UiHostObservationIntegrity::derive(
                batch.canonical_core(),
                batch.reports(),
            )
        {
            return None;
        }
    }
    let [before] = previous.reports() else {
        return None;
    };
    let [after] = next.reports() else {
        return None;
    };
    let old = previous.canonical_core();
    let new = next.canonical_core();
    let survivor = after.coalescing_identity()?;
    if after.family() != UiHostObservationFamily::PointerMotion
        || before.coalescing_identity() != Some(survivor)
        || before.mounted_basis() != after.mounted_basis()
        || before.input_affinity() != after.input_affinity()
        || old.protocol() != new.protocol()
        || old.host_session() != new.host_session()
        || old.presentation() != new.presentation()
        || old.sequences().last().value().checked_add(1) != Some(new.sequences().first().value())
        || new.sequences().first() != after.sequence()
        || new.sequences().last() != after.sequence()
        || new.loss() != UiHostObservationLoss::Complete
        || before.sequence() != old.sequences().last()
        || before.encoded_len() != after.encoded_len()
    {
        return None;
    }
    match old.loss() {
        UiHostObservationLoss::Complete if old.sequences().first() == before.sequence() => {}
        UiHostObservationLoss::Coalesced {
            family,
            replaced,
            survivor: identity,
        } if family == UiHostObservationFamily::PointerMotion
            && identity == survivor
            && replaced.first() == old.sequences().first()
            && replaced.last().value().checked_add(1) == Some(before.sequence().value()) => {}
        _ => return None,
    }
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol: new.protocol(),
        host_session: new.host_session(),
        presentation: new.presentation(),
        sequences: UiHostObservationSequenceRange::new(old.sequences().first(), after.sequence()),
        loss: UiHostObservationLoss::Coalesced {
            family: UiHostObservationFamily::PointerMotion,
            replaced: old.sequences(),
            survivor,
        },
        reports: vec![after.clone()],
    })
    .ok()
}
