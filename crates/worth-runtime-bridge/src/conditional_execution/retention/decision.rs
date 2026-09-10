use super::super::{BridgeConditionalDenial, BridgeConditionalExecutionRequest};
use super::{
    arc_charge, arc_slice_charge, array_charge, sum, BridgeRetentionLedger,
    BridgeRetentionReservation,
};
use std::sync::Arc;

pub(in crate::conditional_execution) fn reserve_decision(
    ledger: &Arc<BridgeRetentionLedger>,
    request: &BridgeConditionalExecutionRequest<'_>,
) -> Result<DecisionReservations, BridgeConditionalDenial> {
    let charge = (|| {
        let strings = [request.snapshot_identity, request.execution_identity];
        let mut bytes = sum(&[arc_charge::<
            super::super::retained_decision::BridgeRetainedConditionalDecisionCore,
        >()?])?;
        for value in strings {
            bytes = sum(&[bytes, arc_slice_charge::<u8>(value.len())?])?;
        }
        Ok(bytes)
    })()
    .map_err(super::super::observation_retention::retention_denial)?;
    let core = ledger
        .reserve(0, 0, charge)
        .map_err(super::super::observation_retention::retention_denial)?;
    let evidence = reserve_reentry(ledger, request.query_binding_identity)?;
    Ok(DecisionReservations { core, evidence })
}

pub(in crate::conditional_execution) struct DecisionReservations {
    pub(in crate::conditional_execution) core: BridgeRetentionReservation,
    pub(in crate::conditional_execution) evidence: BridgeRetentionReservation,
}

pub(in crate::conditional_execution) fn reserve_reentry(
    ledger: &Arc<BridgeRetentionLedger>,
    query_binding_identity: &str,
) -> Result<BridgeRetentionReservation, BridgeConditionalDenial> {
    let charge = (|| {
        sum(&[
            array_charge::<super::super::BridgeConditionalDecisionEvidence>(1)?,
            arc_slice_charge::<u8>(query_binding_identity.len())?,
        ])
    })()
    .map_err(super::super::observation_retention::retention_denial)?;
    ledger
        .reserve(0, 0, charge)
        .map_err(super::super::observation_retention::retention_denial)
}

pub(in crate::conditional_execution) fn reserve_context(
    ledger: &Arc<BridgeRetentionLedger>,
    request: &BridgeConditionalExecutionRequest<'_>,
) -> Result<Option<Arc<BridgeRetentionReservation>>, BridgeConditionalDenial> {
    // Native conditions have no Bridge provider context. Provider contexts can
    // escape and clone; their shared guard follows that distinct allocation life.
    if request.lowering.providers.condition.is_none() && request.lowering.providers.wake.is_none() {
        return Ok(None);
    }
    let charge = (|| {
        let mut bytes = sum(&[
            arc_charge::<BridgeRetentionReservation>()?,
            arc_slice_charge::<u8>(request.snapshot_identity.len())?,
        ])?;
        if let Some(branch) = request.truth_branch_identity {
            bytes = sum(&[bytes, arc_slice_charge::<u8>(branch.len())?])?;
        }
        Ok(bytes)
    })()
    .map_err(super::super::observation_retention::retention_denial)?;
    let reservation = ledger
        .reserve(0, 0, charge)
        .map_err(super::super::observation_retention::retention_denial)?;
    Ok(Some(Arc::new(reservation)))
}
