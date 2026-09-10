mod baseline;
mod preparation;

pub(super) use baseline::{BridgeObservationBaselines, BridgeRetainedObservations};
pub(super) use preparation::prepare_observations;

pub(super) fn retention_denial(
    denial: super::retention::BridgeRetentionDenial,
) -> super::BridgeConditionalDenial {
    use super::retention::BridgeRetentionDenial as D;
    use super::BridgeConditionalDenialKind as K;
    let kind = match denial {
        D::Closed => K::ConditionalRetentionClosed,
        D::Quarantined => K::ConditionalRetentionQuarantined,
        _ => K::ConditionalRetentionCapacity,
    };
    super::BridgeConditionalDenial::new(
        kind,
        format!("Bridge conditional retention denied admission: {denial:?}"),
    )
}

#[cfg(test)]
mod tests;
