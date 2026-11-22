//! Phase-isolated helpers keeping home-retirement frames bounded.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[inline(never)]
    pub(super) fn preflight_cleanup_and_handlers(
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
            .operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot();
        operations.handoff_to_home(request, self)?;
        self.conditional_cleanup
            .resources
            .handoff_home(self, request)?;
        self.runtime
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
        self.mutation_handlers.preflight_home_retirement(request)?;
        self.conditional_cleanup
            .preflight_home_retirement(request)
    }

    #[inline(never)]
    pub(super) fn preflight_producers_and_dispatch(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchCloseCustody,
            crate::domain_computation::primary_graph::application_runtime::inbound_publication::InstalledTransportCloseCustody,
        ),
        WorthQueryHomeRetirementDenial,
    > {
        if !self.installed_producers.is_empty() {
            let history = self
                .installed_program_support()
                .map(|support| support.obligation_history());
            self.runtime
                .output_demands
                .reclaim_home_retirement_cache(history.as_deref(), request)?;
            self.installed_producers
                .preflight_home_retirement(request)?;
            self.runtime.output_demands.preflight_home_retirement()?;
        }
        self.installed_conditionals
            .preflight_home_retirement(self.runtime.bridge.conditional_owner())?;
        // Every recorded attempt owns its one durable observation before close.
        let dispatch_custody = self
            .runtime
            .dispatch_journal
            .settle_for_close(request)
            .map_err(WorthQueryHomeRetirementDenial::DispatchCustody)?;
        if self.runtime.dispatch_journal.is_durable() {
            self.runtime
                .primary_provider
                .protect_dispatch_originals_for_close(
                    request,
                    &self.runtime.product_runtime.owner.inspection_port(),
                )?;
        }
        let transport = self
            .runtime
            .transport_completion_custody
            .try_lock()
            .map_err(|_| {
                WorthQueryHomeRetirementDenial::Busy(
                    WorthQueryHomeRetirementOwner::PublicationReceipts,
                )
            })?
            .preflight_home_retirement(request, &self.runtime.dispatch_journal)?;
        self.preflight_inbound_home_retirement(request)?;
        self.preflight_supplied_home_owners()?;
        Ok((dispatch_custody, transport))
    }

    #[inline(never)]
    pub(super) fn preflight_provider_and_bridge(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        dispatch_custody: crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchCloseCustody,
        transport: &crate::domain_computation::primary_graph::application_runtime::inbound_publication::InstalledTransportCloseCustody,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        self.runtime
            .primary_graph_authority
            .as_mut()
            .expect("serving participation remains owned")
            .preflight(&self.runtime.primary_provider)?;
        self.runtime
            .primary_provider
            .preflight(request, dispatch_custody, transport)?;
        self.mutation_projection
            .preflight_home_retirement(request)?;
        self.runtime
            .granular_invalidation
            .preflight_home_retirement(request)?;
        if let Some(maintenance) = self.runtime.product_runtime.home_maintenance.as_mut() {
            std::sync::Arc::get_mut(maintenance)
                .ok_or(WorthQueryHomeRetirementDenial::Busy(
                    WorthQueryHomeRetirementOwner::HomeMaintenance,
                ))?
                .preflight_home_retirement(request)?;
        }
        {
            let core = &mut self.runtime;
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
        self.runtime
            .primary_graph()
            .expect("an installed application owns its original primary graph")
            .preflight_home_retirement(request)
    }
}

impl<Schema: ApplicationSchema> RetiringPrimaryGraphRuntime<Schema> {
    #[inline(never)]
    pub(super) fn advance_custody_stages(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<Progress, WorthQueryHomeRetirementDenial> {
        match self.stage {
            QueryRetirementStage::Handlers => self
                .runtime
                .mutation_handlers
                .advance_home_retirement(&mut self.handlers, request),
            QueryRetirementStage::InboundCustody => self
                .runtime
                .runtime
                .inbound_custody
                .try_lock()
                .map_err(|_| {
                    WorthQueryHomeRetirementDenial::Busy(
                        WorthQueryHomeRetirementOwner::PublicationReceipts,
                    )
                })?
                .advance_home_retirement(request, native),
            QueryRetirementStage::TransportCustody => self
                .runtime
                .runtime
                .transport_completion_custody
                .try_lock()
                .map_err(|_| {
                    WorthQueryHomeRetirementDenial::Busy(
                        WorthQueryHomeRetirementOwner::PublicationReceipts,
                    )
                })?
                .advance_home_retirement(request, native),
            QueryRetirementStage::Participation => self
                .runtime
                .runtime
                .primary_graph_authority
                .as_mut()
                .expect("serving participation remains owned")
                .advance(request, &self.runtime.runtime.primary_provider),
            _ => unreachable!("custody stages only"),
        }
    }

    #[inline(never)]
    pub(super) fn advance_provider_and_output_stages(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<Progress, WorthQueryHomeRetirementDenial> {
        match self.stage {
            QueryRetirementStage::InstalledConditionals => {
                request
                    .consult()
                    .map_err(WorthQueryHomeRetirementDenial::Execution)?;
                self.runtime.installed_conditionals = Default::default();
                self.runtime.release_query_conditional_bindings();
                Ok(Progress::Complete)
            }
            QueryRetirementStage::RecoveredOutputs => {
                self.runtime
                    .runtime
                    .recovered_outputs
                    .retire_home_observations(request)?;
                Ok(Progress::Complete)
            }
            QueryRetirementStage::Provider => self
                .runtime
                .runtime
                .primary_provider
                .advance(request, native),
            QueryRetirementStage::Projection => self
                .runtime
                .mutation_projection
                .advance_home_retirement(request, native),
            QueryRetirementStage::Granular => self
                .runtime
                .runtime
                .granular_invalidation
                .advance_home_retirement(request, native),
            _ => unreachable!("provider and output stages only"),
        }
    }

    #[inline(never)]
    pub(super) fn advance_graph_and_bridge_stages(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        bridge_owner: &crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
    ) -> Result<Progress, WorthQueryHomeRetirementDenial> {
        match self.stage {
            QueryRetirementStage::ProductSource => {
                request
                    .consult()
                    .map_err(WorthQueryHomeRetirementDenial::Execution)?;
                let source = self
                    .runtime
                    .runtime
                    .product_runtime
                    .source
                    .take()
                    .expect("same original product source");
                match source.try_retire_alias_for_request(request, native) {
                    Ok(retired) => self.retired_product_source = Some(retired),
                    Err((source, cause)) => {
                        self.runtime.runtime.product_runtime.source = Some(source);
                        return Err(WorthQueryHomeRetirementDenial::NativeAlias(cause));
                    }
                }
                Ok(Progress::Complete)
            }
            QueryRetirementStage::ServingGraph => self
                .runtime
                .runtime
                .execution_mut()
                .primary_graph_mut()
                .expect("same original serving graph")
                .advance_home_retirement(request, native),
            QueryRetirementStage::Bridge => self.runtime.runtime.bridge.advance_home_retirement(
                request,
                native,
                bridge_owner,
            ),
            _ => unreachable!("graph and bridge stages only"),
        }
    }
}
