//! Opaque source preparation keeps one admitted schema graph for one replay.
use super::super::runtime_rebuild::original_schemas::OriginalSchemaGraph;
use crate::durability::data::RecoveryPlan;
use crate::durability::initial_schema_installation_denial::RelationalInitialSchemaInstallationDenial;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::{
    HomeProgramTransitionSchemaCarrierDigest, HomeProgramTransitionSchemaDigest,
};
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistry;

pub mod identity_space;

#[derive(Debug)]
pub enum RelationalHomeRecoveryPreparationDenial {
    Request(WorkCeilingDenial),
    Recovery(RelationalRecoveryStop),
    MissingOriginalSource,
    InitialInstallationMissing,
    SchemaCarrierMissing,
    SchemaAuthorityMismatch,
    ForeignReceiver,
    ReceiverInstallation(RelationalInitialSchemaInstallationDenial),
    /// A checkpoint covers commits after the recorded planning predecessor,
    /// so the identities occupied at that predecessor cannot be separated.
    PlanningPredecessorCovered,
}
use RelationalHomeRecoveryPreparationDenial as Denial;

trait HomeRecoveryPreparationBacking: std::fmt::Debug + Send + Sync {
    fn as_state(&self) -> &HomeRecoveryPreparation;
    fn as_state_mut(&mut self) -> &mut HomeRecoveryPreparation;
    fn into_state(self: Box<Self>) -> HomeRecoveryPreparation;
}

impl HomeRecoveryPreparationBacking for HomeRecoveryPreparation {
    fn as_state(&self) -> &HomeRecoveryPreparation {
        self
    }

    fn as_state_mut(&mut self) -> &mut HomeRecoveryPreparation {
        self
    }

    fn into_state(self: Box<Self>) -> HomeRecoveryPreparation {
        *self
    }
}

/// Admitted schema DATA, never a readmitted runtime, basis or execution grant.
/// The plan cannot be mutated or replaced after its graph has been prepared.
#[derive(Debug)]
pub struct PreparedRelationalHomeRecovery {
    state: Box<dyn HomeRecoveryPreparationBacking>,
}
#[derive(Debug)]
pub struct HomeRecoveryPreparation {
    receiver: u64,
    plan: RecoveryPlan,
    graph: OriginalSchemaGraph,
    _memory: Option<ExecutionMemoryReservation>,
}

impl PreparedRelationalHomeRecovery {
    pub(super) fn state(&self) -> &HomeRecoveryPreparation {
        self.state.as_state()
    }

    fn state_mut(&mut self) -> &mut HomeRecoveryPreparation {
        self.state.as_state_mut()
    }

    pub(crate) fn prepare(
        receiver: u64,
        plan: RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        request.consult().map_err(Denial::Request)?;
        let memory = request
            .reserve_retained_memory(std::mem::size_of::<HomeRecoveryPreparation>() as u64)
            .map_err(Denial::Request)?;
        let graph = OriginalSchemaGraph::admit_unbound(&plan, request)
            .map_err(Denial::Recovery)?
            .ok_or(Denial::MissingOriginalSource)?;
        let state: Box<HomeRecoveryPreparation> = Box::new(HomeRecoveryPreparation {
            receiver,
            plan,
            graph,
            _memory: memory,
        });
        Ok(Self { state })
    }

    /// The original installation role, not the last publication or Native main.
    /// A completed transition selects its acknowledged carrier with `schema_data`.
    pub fn initial_schema_data(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<&RelationalSchemaRegistry, Denial> {
        use crate::durability::source_initial_inputs::image::InitialInputEntry;
        request.consult().map_err(Denial::Request)?;
        let Some(InitialInputEntry::Installation { after, .. }) =
            self.state().graph.original.current.entries.get(1)
        else {
            return Err(Denial::InitialInstallationMissing);
        };
        self.state()
            .graph
            .schemas
            .rows
            .get(&after.schema_carrier)
            .map(|schema| schema.registry())
            .ok_or(Denial::SchemaCarrierMissing)
    }

    /// Receiving compiler DATA comes from the actual birth or installation
    /// role. A Native-only Home can have a birth without a Query installation.
    /// Neither the current head nor the last canonical artifact selects it.
    pub fn receiver_schema_data(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<&RelationalSchemaRegistry, Denial> {
        use crate::durability::source_initial_inputs::image::InitialInputEntry as Entry;
        request.consult().map_err(Denial::Request)?;
        let carrier = match self.state().graph.original.current.entries.as_slice() {
            [Entry::Birth(birth)] => birth.schema_carrier,
            [Entry::Birth(_), Entry::Installation { after, .. }] => after.schema_carrier,
            [] | [Entry::Installation { .. }] | [_, _] | [_, _, _, ..] => {
                return Err(Denial::InitialInstallationMissing);
            }
        };
        self.state()
            .graph
            .schemas
            .rows
            .get(&carrier)
            .map(|schema| schema.registry())
            .ok_or(Denial::SchemaCarrierMissing)
    }

    /// Exact acknowledged carrier lookup. Authority and carrier identities are
    /// separate; neither catalog equality nor a fingerprint search selects it.
    pub fn schema_data(
        &self,
        request: &ExecutionRequest<'_, '_>,
        carrier: HomeProgramTransitionSchemaCarrierDigest,
        authority: HomeProgramTransitionSchemaDigest,
    ) -> Result<&RelationalSchemaRegistry, Denial> {
        request.consult().map_err(Denial::Request)?;
        let schema = self
            .state()
            .graph
            .schemas
            .rows
            .get(&carrier.bytes())
            .ok_or(Denial::SchemaCarrierMissing)?;
        if schema.authority_digest() != authority.bytes() {
            return Err(Denial::SchemaAuthorityMismatch);
        }
        Ok(schema.registry())
    }
}

#[doc(hidden)]
pub trait PreparedRelationalHomeRecoveryOwnerPort: Sized {
    fn receiver_identity(&self) -> u64;
    fn schema_graph_mut(&mut self) -> &mut OriginalSchemaGraph;
    fn bind_schema_directories(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::durability::data::RelationalRecoveryStop>;
    fn recovery_plan_mut(&mut self) -> &mut RecoveryPlan;
    fn into_prepared_recovery(
        self,
    ) -> (
        RecoveryPlan,
        OriginalSchemaGraph,
        Option<ExecutionMemoryReservation>,
    );

    fn prepare(
        receiver: u64,
        plan: RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial>;
}

impl PreparedRelationalHomeRecoveryOwnerPort for PreparedRelationalHomeRecovery {
    fn receiver_identity(&self) -> u64 {
        self.state().receiver
    }

    fn schema_graph_mut(&mut self) -> &mut OriginalSchemaGraph {
        &mut self.state_mut().graph
    }

    fn bind_schema_directories(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::durability::data::RelationalRecoveryStop> {
        let state = self.state_mut();
        state.graph.bind_directories(&mut state.plan, request)
    }

    fn recovery_plan_mut(&mut self) -> &mut RecoveryPlan {
        &mut self.state_mut().plan
    }

    fn into_prepared_recovery(
        self,
    ) -> (
        RecoveryPlan,
        OriginalSchemaGraph,
        Option<ExecutionMemoryReservation>,
    ) {
        let HomeRecoveryPreparation {
            plan,
            graph,
            _memory,
            ..
        } = self.state.into_state();
        (plan, graph, _memory)
    }

    fn prepare(
        receiver: u64,
        plan: RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        Self::prepare(receiver, plan, request)
    }
}
