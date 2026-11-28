mod access;
mod authored_owner;
mod branch_state_capture;
mod branch_transfer;
mod construction;
mod graph_mutation;
mod initial_empty_stop;
mod initial_source;
mod owner_services;
mod resource;
mod telemetry;
mod transfer_packets;

use super::super::config::SignalRuntimeConfig;
use super::branching::BranchManager;

use super::resource::ResourceRuntimeState;
use super::runtime_observation::RuntimeObservationRegistry;
use super::temporal::TemporalRuntimeState;
use crate::data::graph::SignalGraph;
use crate::data::telemetry::RuntimeTelemetry;
use crate::logic::checkpoint::CheckpointRuntime;
use crate::logic::events::EventBus;
use crate::schema::data::SignalSchemaRegistry;
/// Full runtime surface for transactional evaluation, diagnostics, replay, and
/// keyed or tier-aware execution.
pub struct SignalRuntime<D, I, E, Ctx, T = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime) config: SignalRuntimeConfig<T>,
    pub(in crate::logic::transaction::runtime) graph: Box<SignalGraph>,
    pub(in crate::logic::transaction::runtime) schema_registry: SignalSchemaRegistry,

    pub(in crate::logic::transaction::runtime) checkpoint: CheckpointRuntime<D, I>,
    pub(in crate::logic::transaction::runtime) event_bus: EventBus<E, D, Ctx>,
    pub(in crate::logic::transaction::runtime) observations:
        RuntimeObservationRegistry<D, I, E, Ctx, T>,
    pub(in crate::logic::transaction::runtime) resource: ResourceRuntimeState,
    pub(in crate::logic::transaction::runtime) temporal: TemporalRuntimeState,
    pub(in crate::logic::transaction::runtime) telemetry: RuntimeTelemetry,
    pub(in crate::logic::transaction::runtime) branches: BranchManager<D, I, T>,
    pub(in crate::logic::transaction::runtime) basis_registry:
        crate::branch::SignalBranchBasisRegistry,
    pub(in crate::logic::transaction::runtime) owner_services:
        crate::branch::owner_services::SignalOwnerRoot<D, I, T>,
}
pub use graph_mutation::SignalGraphMut;
pub(in crate::logic::transaction::runtime) use transfer_packets::{
    AuthorityTransferPacket, BranchLifecycleTransfer, ExplicitBranchForkPacket,
    RestoreTransferPacket,
};

#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
mod operation_control;

#[cfg(test)]
fn collect_owned_inline_type_sizes(census: &mut crate::inline_type_sizes::Census) {
    branch_state_capture::inline_type_budget::collect(census);
    branch_transfer::inline_type_budget::collect(census);
}
