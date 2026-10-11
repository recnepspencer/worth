//! Keyed snapshot identity lookups preserve phase-opening and readiness refusals.
//! The source metadata lookup does not consult or charge the execution request.
use super::super::WorthServerOperationExecutionSlot;
use super::super::WorthServerSchedulerRuntimeFailure as Failure;

pub(super) fn is_stale_for_current_basis(
    slot: &WorthServerOperationExecutionSlot,
    latest_lane_basis_digest: Option<&str>,
) -> Result<bool, Failure> {
    let Some(precondition) = slot.precondition_posture().compatibility_mutation() else {
        let Some(requested_basis_digest) = slot.slot_basis_digest() else {
            return Ok(false);
        };
        return match latest_lane_basis_digest {
            Some(latest_lane_basis_digest) => {
                Ok(latest_lane_basis_digest != requested_basis_digest)
            }
            None => Ok(current_basis_digest(slot)? != requested_basis_digest),
        };
    };
    let Some(requested_basis_digest) = precondition
        .requested_basis_digest()
        .or_else(|| slot.slot_basis_digest())
    else {
        return Ok(false);
    };
    match latest_lane_basis_digest {
        Some(latest_lane_basis_digest) => Ok(latest_lane_basis_digest != requested_basis_digest),
        None => Ok(current_basis_digest(slot)? != requested_basis_digest),
    }
}

pub(super) fn current_basis_digest(
    slot: &WorthServerOperationExecutionSlot,
) -> Result<String, Failure> {
    let workspace = slot.handoff().workspace();
    // Opening admits the phase request; snapshot identity validates its custody.
    // The source performs keyed metadata lookups without consulting that request.
    let snapshot = workspace
        .with_advancement(|phase| workspace.snapshot_identity(&phase))
        .map_err(|denial| Failure::BasisAdvancementDenied {
            denial,
            detail: format!("snapshot advancement was refused: {denial:?}"),
        })?
        .map_err(|denial| Failure::BasisSnapshotReadDenied {
            denial,
            detail: format!("snapshot read was refused: {denial:?}"),
        })?;
    Ok(snapshot.terminal_projection_for_reporting().to_string())
}
