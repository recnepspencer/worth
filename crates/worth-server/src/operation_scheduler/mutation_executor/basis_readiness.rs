//! Keyed snapshot identity lookups preserve phase-opening and readiness refusals.
//! The source metadata lookup does not consult or charge the execution request.
use super::super::WorthServerOperationExecutionSlot;
use crate::{
    WorthServerProductOperationSurfaceDenial, WorthServerProductOperationSurfaceDenialCode,
};

pub(super) fn is_stale_for_current_basis(
    slot: &WorthServerOperationExecutionSlot,
    latest_lane_basis_digest: Option<&str>,
) -> Result<bool, WorthServerProductOperationSurfaceDenial> {
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
) -> Result<String, WorthServerProductOperationSurfaceDenial> {
    let workspace = slot.handoff().workspace();
    // Opening admits the phase request; snapshot identity validates its custody.
    // The source performs keyed metadata lookups without consulting that request.
    let snapshot = workspace
        .with_advancement(|phase| workspace.snapshot_identity(&phase))
        .map_err(snapshot_readiness_denial)?
        .map_err(snapshot_readiness_denial)?;
    Ok(snapshot.terminal_projection_for_reporting().to_string())
}

fn snapshot_readiness_denial(
    denial: impl std::fmt::Debug,
) -> WorthServerProductOperationSurfaceDenial {
    WorthServerProductOperationSurfaceDenial::new(
        WorthServerProductOperationSurfaceDenialCode::ReadinessDenied,
        format!("snapshot readiness was refused: {denial:?}"),
    )
}
