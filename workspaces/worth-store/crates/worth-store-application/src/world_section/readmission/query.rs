//! The Query/Home recipe composes one actual source, Bridge and Signal assembly.
use super::*;
use std::sync::{Arc, RwLock};
use worth_runtime_bridge::facade::{
    BridgeSealedRuntimeAssembly, RuntimeBridge, RuntimeBridgeRelationalSource,
};

#[derive(Debug)]
pub enum ApplicationHomeQueryCompositionDenial {
    QueryCompositionNotAttached,
}

pub fn readmit_query_world(
    native: RelationalReadmitted<crate::ReadmittedApplicationHome>,
    definition: super::super::ApplicationHomeWorldDefinition,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorldReconciled<WorldReadmittedApplicationHome<(), (), (), (), ()>>,
    WorldHomeReadmissionRefused,
> {
    native.consume(|home| {
        if let Err(cause) = request.consult() {
            return Err(WorldHomeReadmissionRefused {
                cause: crate::ApplicationHomeOpenRefusal::Execution(cause),
                home: Box::new(home),
                composition: None,
            });
        }
        let super::super::ApplicationHomeWorldDefinition {
            bridge,
            graph_role,
            budgets,
            clock,
            execution_policy,
            execution_authority,
            conditional_budget,
        } = definition;
        let composition = match super::super::attachment::attach_query_bridge(
            &home.relational,
            bridge,
            graph_role,
            conditional_budget,
        ) {
            Ok(composition) => composition,
            Err(cause) => {
                return Err(WorldHomeReadmissionRefused {
                    cause,
                    home: Box::new(home),
                    composition: None,
                })
            }
        };
        let result = (|| {
            let (signal, publication, correspondence) = {
                let mut conditional = composition
                    .conditional
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                (
                    conditional.signal_owner_services(),
                    conditional
                        .take_runtime_world_signal_definition_publication()
                        .map_err(crate::ApplicationHomeOpenRefusal::ConditionalBridge)?,
                    conditional.runtime_world_correspondence_port(),
                )
            };
            let mut builder = RuntimeWorldOwner::builder()
                .with_signal_services(signal)
                .with_signal_definition_publication(publication)
                .with_budgets(budgets)
                .with_clock(clock)
                .with_execution_policy(execution_policy);
            if let Some(authority) = execution_authority {
                builder = builder.with_execution_authority(authority);
            }
            readmit_attached_world(&home, builder, correspondence, request)
        })();
        match result {
            Ok(world) => Ok(finish_attachment(home, world, Some(composition))),
            Err(cause) => Err(WorldHomeReadmissionRefused {
                cause,
                home: Box::new(home),
                composition: Some(composition),
            }),
        }
    })
}
impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    /// Closed Host attachment clones only the three actual composition owners.
    pub fn query_composition_parts(
        &self,
    ) -> Result<
        (
            RuntimeBridgeRelationalSource,
            RuntimeBridge,
            Arc<RwLock<BridgeSealedRuntimeAssembly>>,
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
            composition.conditional.clone(),
        ))
    }
}
