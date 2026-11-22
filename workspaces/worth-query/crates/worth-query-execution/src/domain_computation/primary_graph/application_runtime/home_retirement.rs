//! Original Query custody survives every admission or retirement refusal.

pub use super::home_retirement_denial::{
    WorthQueryHomeRetirementDenial, WorthQueryHomeRetirementOwner,
};
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::{
    handler::HandlerRetirementCursor, WorthQueryOperationHomeRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;
use worth_query_execution_lineage::facade::ports::{
    ApplicationInvariantProjectionAuthorityInternalPort as _, PrimaryGraphInternalPort as _,
};
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_query::facade::ports::QueryExecutionRuntimeInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
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
    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
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
        use worth_query_execution_runtime::facade::ports::ExecutionRuntimeManagedRunCustodyPort as _;
        self.inner
            .runtime
            .execution()
            .managed_run_custody()
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
                .output_demands_mut()
                .reclaim_home_retirement_cache(history.as_deref(), request)?;
            self.inner
                .installed_producers
                .preflight_home_retirement(request)?;
            self.inner
                .runtime
                .output_demands_mut()
                .preflight_home_retirement()?;
        }
        self.inner
            .installed_conditionals
            .preflight_home_retirement(self.inner.runtime.bridge_ref().conditional_owner())?;
        // Every recorded attempt owns its one durable observation before close.
        let dispatch_custody = self
            .inner
            .runtime
            .dispatch_journal_ref()
            .settle_for_close(request)
            .map_err(WorthQueryHomeRetirementDenial::DispatchCustody)?;
        if self.inner.runtime.dispatch_journal_ref().is_durable() {
            self.inner
                .runtime
                .primary_provider_ref()
                .protect_dispatch_originals_for_close(
                    request,
                    &self
                        .inner
                        .runtime
                        .product_runtime_ref()
                        .owner_ref()
                        .inspection_port(),
                )?;
        }
        let transport = self
            .inner
            .runtime
            .transport_completion_custody_ref()
            .try_lock()
            .map_err(|_| {
                WorthQueryHomeRetirementDenial::Busy(
                    WorthQueryHomeRetirementOwner::PublicationReceipts,
                )
            })?
            .preflight_home_retirement(request, &self.inner.runtime.dispatch_journal_ref())?;
        self.preflight_inbound_home_retirement(request)?;
        self.preflight_supplied_home_owners()?;
        {
            let (authority, provider) = self
                .inner
                .runtime
                .primary_graph_authority_option_mut_and_primary_provider_ref();
            authority
                .as_mut()
                .expect("serving participation remains owned")
                .preflight(provider)?;
        }
        self.inner.runtime.primary_provider_mut().preflight(
            request,
            dispatch_custody,
            &transport,
        )?;
        self.inner
            .mutation_projection
            .preflight_home_retirement(request)?;
        self.inner
            .runtime
            .granular_invalidation_ref()
            .preflight_home_retirement(request)?;
        if let Some(maintenance) = self
            .inner
            .runtime
            .product_runtime_mut()
            .home_maintenance_slot_mut()
            .as_mut()
        {
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
            let (execution, bridge) = self.inner.runtime.execution_ref_and_bridge_mut();
            let graph = execution
                .expect("same serving execution")
                .primary_graph()
                .expect("same serving graph");
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
            .inbound_verifiers_mut()
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

    pub(crate) fn into_home_retirement(self) -> RetiringPrimaryGraphRuntime<Schema> {
        RetiringPrimaryGraphRuntime {
            runtime: self,
            handlers: Default::default(),
            stage: QueryRetirementStage::Handlers,
            retired_product_source: None,
        }
    }
}

impl<Schema: ApplicationSchema> RetiringPrimaryGraphRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        bridge_owner: &super::super::managed_bridge::conditional_owner::ConditionalOwner,
    ) -> Result<Progress, WorthQueryHomeRetirementDenial> {
        if self
            .runtime
            .inner
            .runtime
            .product_runtime_ref()
            .home_maintenance_slot_ref()
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
                .inbound_custody_ref()
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
                .transport_completion_custody_ref()
                .try_lock()
                .map_err(|_| {
                    WorthQueryHomeRetirementDenial::Busy(
                        WorthQueryHomeRetirementOwner::PublicationReceipts,
                    )
                })?
                .advance_home_retirement(request, native)?,
            QueryRetirementStage::Participation => {
                let (authority, provider) = self
                    .runtime
                    .inner
                    .runtime
                    .primary_graph_authority_option_mut_and_primary_provider_ref();
                authority
                    .as_mut()
                    .expect("serving participation remains owned")
                    .advance(request, provider)?
            }
            QueryRetirementStage::Provider => self
                .runtime
                .inner
                .runtime
                .primary_provider_mut()
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
                .granular_invalidation_mut()
                .advance_home_retirement(request, native)?,
            QueryRetirementStage::ProductSource => {
                request
                    .consult()
                    .map_err(WorthQueryHomeRetirementDenial::Execution)?;
                let source = self
                    .runtime
                    .inner
                    .runtime
                    .product_runtime_mut()
                    .source_slot_mut()
                    .take()
                    .expect("same original product source");
                match source.try_retire_alias_for_request(request, native) {
                    Ok(retired) => self.retired_product_source = Some(retired),
                    Err((source, cause)) => {
                        *self
                            .runtime
                            .inner
                            .runtime
                            .product_runtime_mut()
                            .source_slot_mut() = Some(source);
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
                    .recovered_outputs_mut()
                    .retire_home_observations(request)?;
                Progress::Complete
            }
            QueryRetirementStage::Bridge => self
                .runtime
                .inner
                .runtime
                .bridge_mut()
                .advance_home_retirement(request, native, bridge_owner)?,
            QueryRetirementStage::Complete => return Ok(Progress::Complete),
        };
        if progress == Progress::Complete {
            self.stage = self.stage.next();
        }
        Ok(Progress::Advanced)
    }
}

#[path = "home_retirement/maintenance.rs"]
mod maintenance;
#[path = "home_retirement/stage_order.rs"]
mod stage_order;
