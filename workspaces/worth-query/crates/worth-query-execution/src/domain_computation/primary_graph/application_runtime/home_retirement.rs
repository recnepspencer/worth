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
mod stages;
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
    #[inline(never)]
    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        self.preflight_cleanup_and_handlers(request)?;
        let (dispatch_custody, transport) = self.preflight_producers_and_dispatch(request)?;
        self.preflight_provider_and_bridge(request, dispatch_custody, &transport)
    }

    pub(in crate::domain_computation::primary_graph) fn restore_after_refused_home_retirement(
        &mut self,
    ) {
        self.inner
            .runtime
            .primary_provider_mut()
            .restore_after_refused_home_retirement();
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
    #[inline(never)]
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
            QueryRetirementStage::Handlers
            | QueryRetirementStage::InboundCustody
            | QueryRetirementStage::TransportCustody
            | QueryRetirementStage::Participation => {
                self.advance_custody_stages(request, native)?
            }
            QueryRetirementStage::InstalledConditionals
            | QueryRetirementStage::RecoveredOutputs
            | QueryRetirementStage::Provider
            | QueryRetirementStage::Projection
            | QueryRetirementStage::Granular => {
                self.advance_provider_and_output_stages(request, native)?
            }
            QueryRetirementStage::ProductSource
            | QueryRetirementStage::ServingGraph
            | QueryRetirementStage::Bridge => {
                self.advance_graph_and_bridge_stages(request, native, bridge_owner)?
            }
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
