//! Original Query custody survives every admission or retirement refusal.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::{
    handler::HandlerRetirementCursor, WorthQueryOperationHomeRetirementDenial,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;
mod participation;
pub(in crate::domain_computation::primary_graph) use participation::PrimaryGraphParticipationCustody;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryHomeRetirementOwner {
    ProgramRuntime,
    ConditionalRuntime,
    InstalledResources,
    InstalledProducers,
    InstalledConditionals,
    InboundVerifiers,
    ExternalTransport,
    SigningOwners,
    RelationalSource,
    GraphProvider,
    ReceiptBases,
    HomeMaintenance,
}

#[derive(Debug)]
pub enum WorthQueryHomeRetirementDenial {
    Busy(WorthQueryHomeRetirementOwner),
    Unsupported(WorthQueryHomeRetirementOwner),
    Handler {
        binding: &'static str,
        cause: WorthQueryOperationHomeRetirementDenial,
    },
    NativeAlias(
        worth_relational::facade::change_source::RelationalRuntimeHandleAliasRetirementDenial,
    ),
    NativeObservation(
        worth_relational::facade::change_source::RelationalRetainedObservationRetirementDenial,
    ),
    BridgeBasis(worth_runtime_bridge::facade::RelationalBridgeBasisRetirementDenial),
    CanonicalSubscription(worth_relational::facade::mvcc::PublicationCompanionRetirementDenial),
    Bridge(worth_runtime_bridge::facade::BridgeHomeRetirementDenial),
    Execution(worth_execution::WorkCeilingDenial),
}

/// Keeps the original runtime and handler cursor, never a replacement service.
pub(in crate::domain_computation::primary_graph) struct RetiringPrimaryGraphRuntime<Schema> {
    runtime: WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    handlers: HandlerRetirementCursor,
    stage: QueryRetirementStage,
    retired_product_source:
        Option<worth_runtime_bridge::facade::RetiredRuntimeBridgeRelationalSource>,
}
enum QueryRetirementStage {
    Handlers,
    Participation,
    Provider,
    Projection,
    Granular,
    ProductSource,
    ServingGraph,
    Bridge,
    Complete,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        request
            .consult()
            .map_err(WorthQueryHomeRetirementDenial::Execution)?;
        self.mutation_handlers.preflight_home_retirement(request)?;
        self.conditional_cleanup
            .preflight_home_retirement(request)?;
        self.installed_producers.preflight_home_retirement()?;
        self.installed_conditionals.preflight_home_retirement()?;
        self.preflight_supplied_home_owners()?;
        self.primary_graph_authority
            .preflight(&self.primary_provider)?;
        self.primary_provider.preflight(request)?;
        if let Some(maintenance) = self.product_runtime.home_maintenance.as_mut() {
            std::sync::Arc::get_mut(maintenance)
                .ok_or(WorthQueryHomeRetirementDenial::Busy(
                    WorthQueryHomeRetirementOwner::HomeMaintenance,
                ))?
                .preflight_home_retirement(request)?;
        }
        self.runtime
            .primary_graph()
            .expect("same serving graph")
            .with_runtime(request, |native| {
                self.bridge.preflight_home_retirement(request, native)
            })?;
        // The actual source owner, not the durable roster, decides whether
        // its Native handle/subscription/head custody has a consuming path.
        self.runtime
            .primary_graph()
            .expect("an installed application owns its original primary graph")
            .preflight_home_retirement(request)
    }

    fn preflight_supplied_home_owners(&mut self) -> Result<(), WorthQueryHomeRetirementDenial> {
        use WorthQueryHomeRetirementDenial::{Busy, Unsupported};
        use WorthQueryHomeRetirementOwner as Owner;
        let verifiers = self
            .inbound_verifiers
            .get_mut()
            .map_err(|_| Busy(Owner::InboundVerifiers))?;
        if !verifiers.is_empty() {
            return Err(Unsupported(Owner::InboundVerifiers));
        }
        if self.external_effect_transport.get().is_some() {
            return Err(Unsupported(Owner::ExternalTransport));
        }
        if !self
            .signing_owners
            .get_mut()
            .map_err(|_| Busy(Owner::SigningOwners))?
            .is_empty()
        {
            return Err(Unsupported(Owner::SigningOwners));
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn into_home_retirement(
        self,
    ) -> RetiringPrimaryGraphRuntime<Schema> {
        RetiringPrimaryGraphRuntime {
            runtime: self,
            handlers: Default::default(),
            stage: QueryRetirementStage::Handlers,
            retired_product_source: None,
        }
    }
}

impl<Schema: ApplicationSchema> RetiringPrimaryGraphRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn home_maintenance_mut(&mut self)
    -> Option<&mut std::sync::Arc<dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance>>{
        self.runtime.product_runtime.home_maintenance.as_mut()
    }
    pub(in crate::domain_computation::primary_graph) fn take_home_maintenance(&mut self)
    -> Option<std::sync::Arc<dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance>>{
        self.runtime.product_runtime.home_maintenance.take()
    }
    pub(in crate::domain_computation::primary_graph) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        bridge_owner: &std::sync::Arc<
            std::sync::RwLock<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>,
        >,
    ) -> Result<Progress, WorthQueryHomeRetirementDenial> {
        if self.runtime.product_runtime.home_maintenance.is_some() {
            return Err(WorthQueryHomeRetirementDenial::Unsupported(
                WorthQueryHomeRetirementOwner::HomeMaintenance,
            ));
        }
        let progress = match self.stage {
            QueryRetirementStage::Handlers => self
                .runtime
                .mutation_handlers
                .advance_home_retirement(&mut self.handlers, request)?,
            QueryRetirementStage::Participation => self
                .runtime
                .primary_graph_authority
                .advance(request, &self.runtime.primary_provider)?,
            QueryRetirementStage::Provider => {
                self.runtime.primary_provider.advance(request, native)?
            }
            QueryRetirementStage::Projection => self
                .runtime
                .mutation_projection
                .advance_home_retirement(request, native)?,
            QueryRetirementStage::Granular => self
                .runtime
                .granular_invalidation
                .advance_home_retirement(request, native)?,
            QueryRetirementStage::ProductSource => {
                request
                    .consult()
                    .map_err(WorthQueryHomeRetirementDenial::Execution)?;
                let source = self
                    .runtime
                    .product_runtime
                    .source
                    .take()
                    .expect("same original product source");
                match source.try_retire_alias_for_request(request, native) {
                    Ok(retired) => self.retired_product_source = Some(retired),
                    Err((source, cause)) => {
                        self.runtime.product_runtime.source = Some(source);
                        return Err(WorthQueryHomeRetirementDenial::NativeAlias(cause));
                    }
                }
                Progress::Complete
            }
            QueryRetirementStage::ServingGraph => self
                .runtime
                .runtime
                .primary_graph_mut()
                .expect("same original serving graph")
                .advance_home_retirement(request, native)?,
            QueryRetirementStage::Bridge => {
                self.runtime
                    .bridge
                    .advance_home_retirement(request, native, bridge_owner)?
            }
            QueryRetirementStage::Complete => return Ok(Progress::Complete),
        };
        if progress == Progress::Complete {
            self.stage = match self.stage {
                QueryRetirementStage::Handlers => QueryRetirementStage::Participation,
                QueryRetirementStage::Participation => QueryRetirementStage::Provider,
                QueryRetirementStage::Provider => QueryRetirementStage::Projection,
                QueryRetirementStage::Projection => QueryRetirementStage::Granular,
                QueryRetirementStage::Granular => QueryRetirementStage::ProductSource,
                QueryRetirementStage::ProductSource => QueryRetirementStage::ServingGraph,
                QueryRetirementStage::ServingGraph => QueryRetirementStage::Bridge,
                QueryRetirementStage::Bridge => QueryRetirementStage::Complete,
                QueryRetirementStage::Complete => QueryRetirementStage::Complete,
            };
        }
        Ok(Progress::Advanced)
    }
}
