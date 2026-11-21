//! Original Query custody survives every admission or retirement refusal.

pub use super::home_retirement_denial::{
    WorthQueryHomeRetirementDenial, WorthQueryHomeRetirementOwner,
};
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::{
    handler::HandlerRetirementCursor, WorthQueryOperationHomeRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;
mod inbound;
pub(in crate::domain_computation::primary_graph) use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::home_retirement::participation;

/// Keeps the original runtime and handler cursor, never a replacement service.
pub struct RetiringPrimaryGraphRuntime<Schema> {
    runtime: WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    handlers: HandlerRetirementCursor,
    stage: QueryRetirementStage,
    retired_product_source:
        Option<worth_runtime_bridge::facade::RetiredRuntimeBridgeRelationalSource>,
}
enum QueryRetirementStage {
    Handlers,
    InboundCustody,
    TransportCustody,
    Participation,
    InstalledConditionals,
    RecoveredOutputs,
    Provider,
    Projection,
    Granular,
    ProductSource,
    ServingGraph,
    Bridge,
    Complete,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        request
            .consult()
            .map_err(WorthQueryHomeRetirementDenial::Execution)?;
        self.settle_recovery_custody(request)
            .map_err(WorthQueryHomeRetirementDenial::RecoveryCustody)?;
        let operations = self
            .conditional_cleanup
            .routes
            .operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot();
        operations.handoff_to_home(request, self)?;
        self.conditional_cleanup
            .resources
            .handoff_home(self, request)?;
        self.inner
            .runtime
            .execution()
            .managed_run_custody
            .flush(request)
            .map_err(WorthQueryHomeRetirementDenial::ManagedCleanup)?;
        if let Some(support) = self.installed_program_support() {
            support
                .lifecycle()
                .preflight_resource_close(request)
                .map_err(WorthQueryHomeRetirementDenial::ManagedCleanup)?;
            support
                .lifecycle()
                .preflight_managed_run_close(request)
                .map_err(WorthQueryHomeRetirementDenial::ManagedCleanup)?;
            support
                .lifecycle()
                .preflight_managed_cleanup_close(request)
                .map_err(WorthQueryHomeRetirementDenial::ManagedCleanup)?;
        }
        self.inner
            .mutation_handlers
            .preflight_home_retirement(request)?;
        self.conditional_cleanup
            .preflight_home_retirement(request)?;
        if !self.inner.installed_producers.is_empty() {
            let history = self
                .installed_program_support()
                .map(|support| support.obligation_history());
            self.inner
                .runtime
                .output_demands
                .reclaim_home_retirement_cache(history.as_deref(), request)?;
            self.inner
                .installed_producers
                .preflight_home_retirement(request)?;
            self.inner
                .runtime
                .output_demands
                .preflight_home_retirement()?;
        }
        self.inner
            .installed_conditionals
            .preflight_home_retirement(self.inner.runtime.bridge.conditional_owner())?;
        // Every recorded attempt owns its one durable observation before close.
        let dispatch_custody = self
            .inner
            .runtime
            .dispatch_journal
            .settle_for_close(request)
            .map_err(WorthQueryHomeRetirementDenial::DispatchCustody)?;
        if self.inner.runtime.dispatch_journal.is_durable() {
            self.inner
                .runtime
                .primary_provider
                .protect_dispatch_originals_for_close(
                    request,
                    &self.inner.runtime.product_runtime.owner.inspection_port(),
                )?;
        }
        let transport = self
            .inner
            .runtime
            .transport_completion_custody
            .try_lock()
            .map_err(|_| {
                WorthQueryHomeRetirementDenial::Busy(
                    WorthQueryHomeRetirementOwner::PublicationReceipts,
                )
            })?
            .preflight_home_retirement(request, &self.inner.runtime.dispatch_journal)?;
        self.preflight_inbound_home_retirement(request)?;
        self.preflight_supplied_home_owners()?;
        self.inner
            .runtime
            .primary_graph_authority
            .as_mut()
            .expect("serving participation remains owned")
            .preflight(&self.inner.runtime.primary_provider)?;
        self.inner
            .runtime
            .primary_provider
            .preflight(request, dispatch_custody, &transport)?;
        self.inner
            .mutation_projection
            .preflight_home_retirement(request)?;
        self.inner
            .runtime
            .granular_invalidation
            .preflight_home_retirement(request)?;
        if let Some(maintenance) = self.inner.runtime.product_runtime.home_maintenance.as_mut() {
            let adapter = std::sync::Arc::get_mut(maintenance)
                .ok_or(WorthQueryHomeRetirementDenial::Busy(
                    WorthQueryHomeRetirementOwner::HomeMaintenance,
                ))?
                .as_any_mut()
                .downcast_mut::<crate::domain_computation::execution_runtime::product_world::HomeMaintenanceAdapter>()
                .ok_or(WorthQueryHomeRetirementDenial::Unsupported(
                    WorthQueryHomeRetirementOwner::HomeMaintenance,
                ))?;
            std::sync::Arc::get_mut(adapter.inner_mut())
                .ok_or(WorthQueryHomeRetirementDenial::Busy(
                    WorthQueryHomeRetirementOwner::HomeMaintenance,
                ))?
                .preflight_home_retirement(request)?;
        }
        {
            let core = &mut self.inner.runtime;
            let graph = core
                .execution
                .as_ref()
                .expect("same serving execution")
                .primary_graph()
                .expect("same serving graph");
            let bridge = &mut core.bridge;
            graph.with_runtime(request, |native| {
                bridge.preflight_home_retirement(request, native)
            })?;
        }
        // The actual source owner, not the durable roster, decides whether
        // its Native handle/subscription/head custody has a consuming path.
        self.inner
            .runtime
            .primary_graph()
            .expect("an installed application owns its original primary graph")
            .preflight_home_retirement(request)
            .map_err(Into::into)
    }

    fn preflight_supplied_home_owners(&mut self) -> Result<(), WorthQueryHomeRetirementDenial> {
        use WorthQueryHomeRetirementDenial::Busy;
        use WorthQueryHomeRetirementOwner as Owner;
        self.inner
            .runtime
            .inbound_verifiers
            .get_mut()
            .map_err(|_| Busy(Owner::InboundVerifiers))?;
        // Verifiers borrow the original journal weakly. Their Home-level source
        // facts are settled above; supplied mechanisms remain inert with this runtime.
        // The supplied transport holds no Home custody. Exclusive retirement
        // access means no contact is executing, and the dispatch journal has
        // just settled: every recorded attempt owns its durable observation.
        // The transport therefore follows this original runtime into inert
        // retirement custody; a reopened Home is given its transport afresh.
        // The intact signer registry follows this original runtime into inert
        // retirement custody. Its final supplier destruction belongs to reclaim,
        // not budgeted close; external authentication owners are not revoked.
        self.inner
            .signing_owners
            .get_mut()
            .map_err(|_| Busy(Owner::SigningOwners))?;
        Ok(())
    }

    pub fn into_home_retirement(self) -> RetiringPrimaryGraphRuntime<Schema> {
        RetiringPrimaryGraphRuntime {
            runtime: self,
            handlers: Default::default(),
            stage: QueryRetirementStage::Handlers,
            retired_product_source: None,
        }
    }
}

impl<Schema: ApplicationSchema> RetiringPrimaryGraphRuntime<Schema> {
    pub fn home_maintenance_mut(&mut self)
    -> Option<&mut std::sync::Arc<dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance>>{
        self.runtime
            .inner.runtime
            .product_runtime
            .home_maintenance
            .as_mut()
            .and_then(std::sync::Arc::get_mut)
            .and_then(|adapter| {
                adapter
                    .as_any_mut()
                    .downcast_mut::<crate::domain_computation::execution_runtime::product_world::HomeMaintenanceAdapter>()
            })
            .map(crate::domain_computation::execution_runtime::product_world::HomeMaintenanceAdapter::inner_mut)
    }
    pub fn take_home_maintenance(&mut self)
    -> Option<std::sync::Arc<dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance>>{
        let taken = self
            .runtime
            .inner
            .runtime
            .product_runtime
            .home_maintenance
            .take()?;
        let inner = taken
            .as_any()
            .downcast_ref::<crate::domain_computation::execution_runtime::product_world::HomeMaintenanceAdapter>()
            .map(|adapter| std::sync::Arc::clone(&adapter.0));
        drop(taken);
        inner
    }
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        bridge_owner: &super::super::managed_bridge::conditional_owner::ConditionalOwner,
    ) -> Result<Progress, WorthQueryHomeRetirementDenial> {
        if self
            .runtime
            .inner
            .runtime
            .product_runtime
            .home_maintenance
            .is_some()
        {
            return Err(WorthQueryHomeRetirementDenial::Unsupported(
                WorthQueryHomeRetirementOwner::HomeMaintenance,
            ));
        }
        let progress = match self.stage {
            QueryRetirementStage::Handlers => self
                .runtime
                .inner
                .mutation_handlers
                .advance_home_retirement(&mut self.handlers, request)?,
            QueryRetirementStage::InboundCustody => self
                .runtime
                .inner
                .runtime
                .inbound_custody
                .try_lock()
                .map_err(|_| {
                    WorthQueryHomeRetirementDenial::Busy(
                        WorthQueryHomeRetirementOwner::PublicationReceipts,
                    )
                })?
                .advance_home_retirement(request, native)?,
            QueryRetirementStage::TransportCustody => self
                .runtime
                .inner
                .runtime
                .transport_completion_custody
                .try_lock()
                .map_err(|_| {
                    WorthQueryHomeRetirementDenial::Busy(
                        WorthQueryHomeRetirementOwner::PublicationReceipts,
                    )
                })?
                .advance_home_retirement(request, native)?,
            QueryRetirementStage::Participation => self
                .runtime
                .inner
                .runtime
                .primary_graph_authority
                .as_mut()
                .expect("serving participation remains owned")
                .advance(request, &self.runtime.inner.runtime.primary_provider)?,
            QueryRetirementStage::Provider => self
                .runtime
                .inner
                .runtime
                .primary_provider
                .advance(request, native)?,
            QueryRetirementStage::Projection => self
                .runtime
                .inner
                .mutation_projection
                .advance_home_retirement(request, native)?,
            QueryRetirementStage::Granular => self
                .runtime
                .inner
                .runtime
                .granular_invalidation
                .advance_home_retirement(request, native)?,
            QueryRetirementStage::ProductSource => {
                request
                    .consult()
                    .map_err(WorthQueryHomeRetirementDenial::Execution)?;
                let source = self
                    .runtime
                    .inner
                    .runtime
                    .product_runtime
                    .source
                    .take()
                    .expect("same original product source");
                match source.try_retire_alias_for_request(request, native) {
                    Ok(retired) => self.retired_product_source = Some(retired),
                    Err((source, cause)) => {
                        self.runtime.inner.runtime.product_runtime.source = Some(source);
                        return Err(WorthQueryHomeRetirementDenial::NativeAlias(cause));
                    }
                }
                Progress::Complete
            }
            QueryRetirementStage::ServingGraph => self
                .runtime
                .inner
                .runtime
                .execution_mut()
                .primary_graph_mut()
                .expect("same original serving graph")
                .advance_home_retirement(request, native)?,
            // The quiesced checkpoint already captured authored definitions.
            // Installed projection authorities share the original graph source;
            // release them before consuming source shares so the final source
            // retirement owns the Native subscription detach.
            QueryRetirementStage::InstalledConditionals => {
                request
                    .consult()
                    .map_err(WorthQueryHomeRetirementDenial::Execution)?;
                self.runtime.inner.installed_conditionals = Default::default();
                self.runtime.release_query_conditional_bindings();
                Progress::Complete
            }
            QueryRetirementStage::RecoveredOutputs => {
                self.runtime
                    .inner
                    .runtime
                    .recovered_outputs
                    .retire_home_observations(request)?;
                Progress::Complete
            }
            QueryRetirementStage::Bridge => self
                .runtime
                .inner
                .runtime
                .bridge
                .advance_home_retirement(request, native, bridge_owner)?,
            QueryRetirementStage::Complete => return Ok(Progress::Complete),
        };
        if progress == Progress::Complete {
            self.stage = self.stage.next();
        }
        Ok(Progress::Advanced)
    }
}

#[path = "home_retirement/stage_order.rs"]
mod stage_order;
