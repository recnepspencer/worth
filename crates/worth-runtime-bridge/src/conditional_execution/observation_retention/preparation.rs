use super::super::retention::semantic_payload;
use super::super::retention::{
    arc_charge, array_charge, sum, BridgeRetentionDenial as D, BridgeRetentionLedger,
};
use super::super::{BridgeConditionalDenial, BridgeConditionalSemanticObservation};
use super::{baseline::Backing, BridgeRetainedObservations};
use std::sync::Arc;
use worth_foundational::facade::ContractValidatedAspectArtifact;

pub(in crate::conditional_execution) fn prepare_observations<'a>(
    plan: &super::super::semantic_observation_plan::BridgeConditionalSemanticObservationPlan,
    previous: &BridgeRetainedObservations,
    current: impl Iterator<Item = Option<&'a ContractValidatedAspectArtifact>> + Clone,
    ledger: &Arc<BridgeRetentionLedger>,
) -> Result<BridgeRetainedObservations, BridgeConditionalDenial> {
    let mut work = ledger.maximum_preparation_visits();
    let mut count = 0usize;
    let mut bytes = arc_charge::<Backing>().map_err(super::retention_denial)?;
    for (ordinal, current) in plan.ordinals().zip(current.clone()) {
        semantic_payload::visit(&mut work).map_err(super::retention_denial)?;
        count = count
            .checked_add(1)
            .ok_or_else(|| super::retention_denial(D::BytesExhausted))?;
        for value in [previous.current(ordinal), current].into_iter().flatten() {
            bytes = sum(&[
                bytes,
                semantic_payload::artifact(value, &mut work).map_err(super::retention_denial)?,
            ])
            .map_err(super::retention_denial)?;
        }
        let mask = plan
            .projection_mask(ordinal)
            .expect("compiled observation retains projection");
        bytes = sum(&[
            bytes,
            semantic_payload::mask(mask, &mut work).map_err(super::retention_denial)?,
        ])
        .map_err(super::retention_denial)?;
    }
    if count == 0 {
        return Ok(Default::default());
    }
    // Keep the exact-capacity Vec as immutable backing. There is no conversion
    // allocation or temporary second array hidden in the retained charge.
    bytes = sum(&[
        bytes,
        array_charge::<BridgeConditionalSemanticObservation>(count)
            .map_err(super::retention_denial)?,
    ])
    .map_err(super::retention_denial)?;
    let reservation = ledger
        .reserve(0, 0, bytes)
        .map_err(super::retention_denial)?;
    let mut observations = Vec::with_capacity(count);
    assert_eq!(
        observations.capacity(),
        count,
        "non-zero-sized observation capacity is exact"
    );
    for (ordinal, current) in plan.ordinals().zip(current) {
        observations.push(BridgeConditionalSemanticObservation::new(
            ordinal,
            previous.current(ordinal).cloned(),
            current.cloned(),
            plan.projection_mask(ordinal)
                .expect("compiled projection")
                .clone(),
        ));
    }
    observations.sort_unstable_by_key(BridgeConditionalSemanticObservation::dependency_ordinal);
    Ok(BridgeRetainedObservations::new(observations, reservation))
}
