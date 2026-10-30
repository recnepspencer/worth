//! Each independent policy binds to the same real Inspection session and target.
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::WorthQueryApplicationGovernanceBinding,
    primary_graph::admitted_capture::{checkpoint, run_retained_stage, RetainedStorageDenial},
};
use worth_execution::MapKernelFailure;
use worth_foundational::facade::CanonicalDigestId;
use worth_relational::facade::identity::EntityId;

pub(super) fn capture(
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    session: &WorthQueryManagedGraphWorkSession,
    target: &Arc<WorthQueryApplicationInspectionTarget>,
    parameters: CanonicalDigestId,
    principal: EntityId,
    scope: EntityId,
) -> Result<WorthQueryApplicationGovernanceBinding, InspectionAdmissionDenial> {
    run_retained_stage(execution, memory, |context, memory| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        let bytes = WorthQueryApplicationGovernanceBinding::inspection_backing_bytes(session)
            .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
        checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
        if let Some(memory) = memory {
            memory.resize(bytes).map_err(MapKernelFailure::Domain)?;
        }
        Ok(
            WorthQueryApplicationGovernanceBinding::from_inspection_session(
                session, target, parameters, principal, scope,
            ),
        )
    })
    .map_err(InspectionAdmissionDenial::Storage)?
    .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)
}
