//! Original Query custody survives every admission or retirement refusal.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::{
    handler::HandlerRetirementCursor, WorthQueryOperationHomeRetirementDenial,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;
mod inbound;
mod participation;
mod stages;
pub(in crate::domain_computation::primary_graph) use participation::PrimaryGraphParticipationCustody;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryHomeRetirementOwner {
    ProgramRuntime,
    ConditionalRuntime,
    InstalledResources,
    InstalledProducers,
    InstalledConditionals,
    InboundVerifiers,
    SigningOwners,
    RelationalSource,
    GraphProvider,
    ReceiptBases,
    PublicationReceipts,
    HomeMaintenance,
}

#[derive(Debug)]
pub enum WorthQueryHomeRetirementDenial {
    RecoveryCustody(crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial),
    CorrectionRetention(super::super::WorthQueryCorrectionRetentionDenial),
    ManagedCleanup(super::super::query_obligations::QueryObligationDenial),
    Resource(super::super::application_resource::WorthQueryResourceActionDenial),
    ConditionalClock(worth_runtime_bridge::facade::BridgeManagedTemporalDenialKind),
    ConditionalBinding(worth_runtime_bridge::facade::BridgeConditionalDenialKind),
    CorrectionMaterial(crate::facade::correction_material::QueryCorrectionMaterialDenial),
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
    Publication(worth_runtime_world::facade::ConsumedPublicationRetirementDenial),
    /// The dispatch journal still holds an executing attempt or an unsettled record.
    DispatchCustody(
        crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial,
    ),
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

    fn preflight_supplied_home_owners(&mut self) -> Result<(), WorthQueryHomeRetirementDenial> {
        use WorthQueryHomeRetirementDenial::Busy;
        use WorthQueryHomeRetirementOwner as Owner;
        self.runtime
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
        self.signing_owners
            .get_mut()
            .map_err(|_| Busy(Owner::SigningOwners))?;
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
        self.runtime
            .runtime
            .product_runtime
            .home_maintenance
            .as_mut()
    }
    pub(in crate::domain_computation::primary_graph) fn take_home_maintenance(&mut self)
    -> Option<std::sync::Arc<dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance>>{
        self.runtime.runtime.product_runtime.home_maintenance.take()
    }
    #[inline(never)]
    pub(in crate::domain_computation::primary_graph) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        bridge_owner: &super::super::managed_bridge::conditional_owner::ConditionalOwner,
    ) -> Result<Progress, WorthQueryHomeRetirementDenial> {
        if self
            .runtime
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
            QueryRetirementStage::Handlers
            | QueryRetirementStage::InboundCustody
            | QueryRetirementStage::TransportCustody
            | QueryRetirementStage::Participation => self.advance_custody_stages(request, native)?,
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
            self.stage = match self.stage {
                QueryRetirementStage::Handlers => QueryRetirementStage::InboundCustody,
                QueryRetirementStage::InboundCustody => QueryRetirementStage::TransportCustody,
                QueryRetirementStage::TransportCustody => QueryRetirementStage::Participation,
                QueryRetirementStage::Participation => QueryRetirementStage::InstalledConditionals,
                QueryRetirementStage::InstalledConditionals => {
                    QueryRetirementStage::RecoveredOutputs
                }
                QueryRetirementStage::RecoveredOutputs => QueryRetirementStage::Provider,
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
