//! Home admits its actual authored owner before reconciling World history.
use super::super::{attachment::ApplicationHomeBridgeComposition, ApplicationHomeWorldDefinition};
use super::*;
use crate::ApplicationHomeOpenRefusal as D;
use std::sync::Arc;
use worth_runtime_bridge::facade::{
    BridgeAuthoredInstallationBindings, BridgeConditionalRuntimeBuilder,
    BridgeReadmittedAuthoredAssembly, RuntimeBridge, RuntimeBridgeRelationalSource,
};
use worth_signal::facade::durability::*;

pub struct PreparedQueryHomeSource {
    pub(super) home: crate::ReadmittedApplicationHome,
    pub(super) composition: ApplicationHomeBridgeComposition,
    pub(super) resources: QueryHomeWorldResources,
}
pub(super) struct QueryHomeWorldResources {
    pub(super) budgets: worth_runtime_world::facade::RuntimeWorldBudgets,
    pub(super) clock: worth_runtime_world::facade::RuntimeWorldClock,
    pub(super) execution_policy: worth_foundational::facade::ExecutionRequestPolicy,
    pub(super) execution_authority: Option<Arc<worth_execution::ExecutionAuthority>>,
}
pub struct SignalReadmittedApplicationHome {
    pub(super) source: PreparedQueryHomeSource,
    pub(super) assembly: Arc<BridgeReadmittedAuthoredAssembly>,
}
impl worth_proof::AuthorityMarker for SignalReadmittedApplicationHome {}
impl PreparedQueryHomeSource {
    /// Dispose unserved source preparation while retaining the same physical Home.
    pub fn into_home(self) -> crate::ReadmittedApplicationHome {
        let Self {
            home,
            composition,
            resources: _,
        } = self;
        drop(composition);
        home
    }

    pub fn query_source_parts(&self) -> (RuntimeBridgeRelationalSource, RuntimeBridge) {
        (
            self.composition.source.clone(),
            self.composition.ordinary.clone(),
        )
    }
}
impl SignalReadmittedApplicationHome {
    pub fn authored_assembly(&self) -> Arc<BridgeReadmittedAuthoredAssembly> {
        Arc::clone(&self.assembly)
    }
}
pub fn prepare_query_home_source(
    native: RelationalReadmitted<crate::ReadmittedApplicationHome>,
    definition: ApplicationHomeWorldDefinition,
    request: &ExecutionRequest<'_, '_>,
) -> Result<PreparedQueryHomeSource, WorldHomeReadmissionRefused> {
    native.consume(|home| {
        let ApplicationHomeWorldDefinition {
            bridge,
            graph_role,
            budgets,
            clock,
            execution_policy,
            execution_authority,
        } = definition;
        let result = request.consult().map_err(D::Execution).and_then(|()| {
            crate::open::readmission::unclean::admit(&home, request)?;
            super::super::attachment::attach_bridge(&home.relational, bridge, graph_role)
        });
        match result {
            Ok(composition) => Ok(PreparedQueryHomeSource {
                home,
                composition,
                resources: QueryHomeWorldResources {
                    budgets,
                    clock,
                    execution_policy,
                    execution_authority,
                },
            }),
            Err(cause) => Err(WorldHomeReadmissionRefused {
                cause,
                home: Box::new(home),
                composition: None,
            }),
        }
    })
}
pub fn readmit_query_signal(
    source: PreparedQueryHomeSource,
    builder: Box<BridgeConditionalRuntimeBuilder>,
    bindings: &mut dyn BridgeAuthoredInstallationBindings,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    worth_proof::SignalReadmitted<SignalReadmittedApplicationHome>,
    WorldHomeReadmissionRefused,
> {
    match admit(&source.home, builder, bindings, request) {
        Ok(assembly) => Ok(worth_proof::SignalReadmitted::from_owner(
            SignalReadmittedApplicationHome {
                source,
                assembly: Arc::from(assembly),
            },
        )),
        Err(cause) => Err(WorldHomeReadmissionRefused {
            cause,
            home: Box::new(source.home),
            composition: Some(source.composition),
        }),
    }
}
fn admit(
    home: &crate::ReadmittedApplicationHome,
    builder: Box<BridgeConditionalRuntimeBuilder>,
    bindings: &mut dyn BridgeAuthoredInstallationBindings,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Box<BridgeReadmittedAuthoredAssembly>, D> {
    let medium = super::standalone_signal::attach_medium(home, request)?;
    let source = medium
        .capture_recovery_source(request)
        .map_err(D::SignalMedium)?;
    let contribution = source.identity_contribution();
    let fresh = source
        .frontier(
            worth_foundational::facade::SignalDurableJournal::Definitions,
            request,
        )
        .map_err(D::SignalMedium)?
        == 0
        && source
            .frontier(
                worth_foundational::facade::SignalDurableJournal::Advances,
                request,
            )
            .map_err(D::SignalMedium)?
            == 0;
    drop(source);
    let assembly = if fresh && home.relational.world_fresh_source_admission().is_some() {
        medium
            .complete_home_source_admission()
            .map_err(D::SignalMedium)?;
        let task = builder
            .prepare_initial_authored_source(medium.clone(), contribution, request)
            .map_err(|refusal| D::BridgeInitial(Box::new(refusal.into_parts().1)))?;
        let slot = home
            .context
            .world_slot
            .as_ref()
            .ok_or(D::SignalSourceRequired)?
            .signal_slot();
        let retained = slot
            .retain_initial(task, request)
            .map_err(D::SignalMedium)?;
        loop {
            match retained
                .advance(request)
                .map_err(|cause| D::SignalInitialProgress(Box::new(cause)))?
            {
                SignalInitialSourceProgress::Pending => request.consult().map_err(D::Execution)?,
                SignalInitialSourceProgress::Published => break,
            }
        }
        let published = retained
            .take_published()
            .map_err(|cause| D::SignalInitialProgress(Box::new(cause)))?
            .try_into_published()
            .unwrap_or_else(|_| unreachable!("same original reported Published"));
        published.readmit_authored_owner(bindings, request)
    } else {
        builder.readmit_authored_owner(medium.clone(), contribution, bindings, request)
    }
    .map_err(|cause| D::AuthoredBridge(Box::new(cause)))?;
    medium
        .complete_home_source_admission()
        .map_err(D::SignalMedium)?;
    Ok(assembly)
}
