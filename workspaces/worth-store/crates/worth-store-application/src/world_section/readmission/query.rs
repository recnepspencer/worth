//! World reconciles only after the same Home's actual authored Signal admission.
use super::*;
use std::sync::Arc;
use worth_runtime_bridge::facade::{
    BridgeReadmittedAuthoredAssembly, RuntimeBridge, RuntimeBridgeRelationalSource,
};
#[derive(Debug)]
pub enum ApplicationHomeQueryCompositionDenial {
    QueryCompositionNotAttached,
}
pub fn readmit_query_world(
    signal: worth_proof::SignalReadmitted<super::signal::SignalReadmittedApplicationHome>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorldReconciled<WorldReadmittedApplicationHome<(), (), (), (), ()>>,
    WorldHomeReadmissionRefused,
> {
    signal.consume(|admitted| {
        let super::signal::SignalReadmittedApplicationHome { source, assembly } = admitted;
        let super::signal::PreparedQueryHomeSource {
            home,
            mut composition,
            resources,
        } = source;
        composition.conditional = Some(Arc::clone(&assembly));
        let result = (|| {
            let attachment =
                worth_runtime_world::facade::RuntimeWorldAuthoredSignalAttachment::install_bridge(
                    &assembly, request,
                )
                .map_err(|cause| {
                    crate::ApplicationHomeOpenRefusal::SignalAttachment(Box::new(cause))
                })?;
            let mut builder = RuntimeWorldOwner::builder()
                .with_budgets(resources.budgets)
                .with_clock(resources.clock)
                .with_execution_policy(resources.execution_policy);
            if let Some(authority) = resources.execution_authority {
                builder = builder.with_execution_authority(authority);
            }
            readmit_attached_world(
                &home,
                builder.with_authored_signal(attachment),
                assembly.runtime_world_correspondence_port(),
                request,
            )
        })();
        match result {
            Ok(world) => Ok(finish_attachment(
                home,
                world,
                Some(composition),
                None,
                assembly.signal_owner().checkpoint_port(),
            )),
            Err(cause) => Err(WorldHomeReadmissionRefused {
                cause,
                home: Box::new(home),
                composition: Some(composition),
            }),
        }
    })
}
impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    pub fn query_composition_parts(
        &self,
    ) -> Result<
        (
            RuntimeBridgeRelationalSource,
            RuntimeBridge,
            Arc<BridgeReadmittedAuthoredAssembly>,
        ),
        ApplicationHomeQueryCompositionDenial,
    > {
        let composition = self
            .composition
            .as_ref()
            .ok_or(ApplicationHomeQueryCompositionDenial::QueryCompositionNotAttached)?;
        Ok((
            composition.source.clone(),
            composition.ordinary.clone(),
            Arc::clone(
                composition
                    .conditional
                    .as_ref()
                    .ok_or(ApplicationHomeQueryCompositionDenial::QueryCompositionNotAttached)?,
            ),
        ))
    }
}
