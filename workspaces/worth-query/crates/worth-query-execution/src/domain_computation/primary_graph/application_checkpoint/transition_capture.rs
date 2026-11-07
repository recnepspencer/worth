//! Unpublished repair captures keep actual source and successor custody together.
use super::*;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{NativeInterimBootstrapContinuation, RecoveredRelationalRuntimeAuthority},
    runtime::RelationalRuntime,
};

impl WorthQueryApplicationCheckpoint {
    pub(in crate::domain_computation::primary_graph) fn capture_native_transition(
        runtime: &mut RelationalRuntime,
        recovered: &mut RecoveredRelationalRuntimeAuthority,
        original: &WorthQueryInterimCheckpointOrigin,
        publication: &super::super::WorthQueryPrimaryGraphPublication,
        outputs: &[super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity],
        request: &ExecutionRequest<'_, '_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<(Self, NativeInterimBootstrapContinuation), WorthQueryCheckpointCaptureDenial> {
        policy.check_live()?;
        let request = match policy {
            WorthQueryCheckpointCapturePolicy::SystemAllocation => *request,
            WorthQueryCheckpointCapturePolicy::Execution(lease) => ExecutionRequest::leased(lease),
        };
        let mut answer = None;
        let stage = request.run_native_with_checkpoint(|_| {
            answer = Some((|| {
                let continuation = runtime
                    .durability_recovery()
                    .prepare_interim_bootstrap_continuation(&request, &original.native, recovered)
                    .map_err(|cause| {
                        WorthQueryCheckpointCaptureDenial::AdmissionImage(Box::new(cause))
                    })?;
                let (native, capture) = runtime
                    .durability_authority()
                    .capture_interim_checkpoint(&request, policy)?;
                let port = runtime.owner_component_services().basis_port();
                let world = original
                    .world
                    .continue_after_native_transition(
                        &request,
                        &port,
                        &continuation,
                        &capture,
                        interim_transport::application_binding(publication),
                    )
                    .map_err(|cause| {
                        WorthQueryCheckpointCaptureDenial::WorldOriginals(Box::new(cause))
                    })?;
                let (checkpoint, _) = Self::encode_captured(
                    native,
                    capture,
                    world,
                    publication,
                    outputs,
                    &request,
                    policy,
                )?;
                Ok((checkpoint, continuation))
            })());
        });
        match (stage, answer) {
            (Ok(_), Some(answer)) => answer,
            (Ok(_), None) => unreachable!("the transition capture stage produced its answer"),
            (Err(cause), Some(Err(inner))) if worth_relational::facade::transactions::RelationalExecutionDenialCause::from(cause)
                == worth_relational::facade::transactions::RelationalExecutionDenialCause::NestedStopped => Err(inner),
            (Err(cause), _) => Err(WorthQueryCheckpointCaptureDenial::from(
                worth_relational::facade::durability::RelationalNativeCheckpointCaptureDenial::Request(cause.into()))),
        }
    }
}
