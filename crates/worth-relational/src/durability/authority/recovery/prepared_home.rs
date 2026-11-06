//! Opaque source preparation keeps one admitted schema graph for one replay.
use super::super::runtime_rebuild::OriginalSchemaGraph;
use crate::durability::data::{RecoveryPlan, RelationalRecoveryStop};
use crate::runtime::{RelationalInitialSchemaInstallationDenial, RelationalRuntime};
use crate::schema::data::RelationalSchemaRegistry;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::{
    HomeProgramTransitionSchemaCarrierDigest, HomeProgramTransitionSchemaDigest,
};

mod recovery;

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
}
use RelationalHomeRecoveryPreparationDenial as Denial;

/// Admitted schema DATA, never a readmitted runtime, basis or execution grant.
/// The plan cannot be mutated or replaced after its graph has been prepared.
#[derive(Debug)]
pub struct PreparedRelationalHomeRecovery {
    state: Box<HomeRecoveryPreparation>,
}
#[derive(Debug)]
struct HomeRecoveryPreparation {
    receiver: u64,
    plan: RecoveryPlan,
    graph: OriginalSchemaGraph,
    _memory: Option<ExecutionMemoryReservation>,
}

impl PreparedRelationalHomeRecovery {
    pub(in crate::durability) fn prepare(
        runtime: &RelationalRuntime,
        mut plan: RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        request.consult().map_err(Denial::Request)?;
        let memory = request
            .reserve_retained_memory(std::mem::size_of::<HomeRecoveryPreparation>() as u64)
            .map_err(Denial::Request)?;
        let graph = OriginalSchemaGraph::admit(&mut plan, request)
            .map_err(Denial::Recovery)?
            .ok_or(Denial::MissingOriginalSource)?;
        Ok(Self {
            state: Box::new(HomeRecoveryPreparation {
                receiver: runtime.runtime_instance_id(),
                plan,
                graph,
                _memory: memory,
            }),
        })
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
            self.state.graph.original.current.entries.get(1)
        else {
            return Err(Denial::InitialInstallationMissing);
        };
        self.state
            .graph
            .schemas
            .rows
            .get(&after.schema_carrier)
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
            .state
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

/// Failures before reconstruction return this exact opaque preparation.
/// Reconstruction failures consume that attempt, just like ordinary recovery;
/// the Store source owner retains its independent original source for retry.
#[derive(Debug)]
pub enum RefusedPreparedRelationalHomeRecovery {
    Preparation {
        prepared: PreparedRelationalHomeRecovery,
        cause: RelationalHomeRecoveryPreparationDenial,
    },
    Recovery(RelationalRecoveryStop),
    Transition(super::RelationalHomeProgramTransitionSourceRefusal),
}
