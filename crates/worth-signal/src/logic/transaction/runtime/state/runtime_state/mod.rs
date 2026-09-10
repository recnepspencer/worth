mod access;
mod branch_state_capture;
mod branch_transfer;
mod construction;
mod graph_mutation;
mod owner_services;
mod resource;
mod telemetry;
mod transfer_packets;

use super::super::config::SignalRuntimeConfig;
use super::branching::BranchManager;
use super::merge::{
    FrozenAspectMergePolicyRegistry, FrozenConflictIsolationRegistry, FrozenConflictPolicyRegistry,
    FrozenDeletionPolicyRegistry, FrozenIdentityMatcherRegistry, FrozenMergeBaseStrategyRegistry,
    FrozenMergeStrategyRegistry, FrozenSourceOnlyPolicyRegistry,
};
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
    pub(in crate::logic::transaction::runtime) merge_strategy_registry: FrozenMergeStrategyRegistry,
    pub(in crate::logic::transaction::runtime) merge_base_strategy_registry:
        FrozenMergeBaseStrategyRegistry,
    pub(in crate::logic::transaction::runtime) aspect_merge_policy_registry:
        FrozenAspectMergePolicyRegistry,
    pub(in crate::logic::transaction::runtime) conflict_isolation_registry:
        FrozenConflictIsolationRegistry,
    pub(in crate::logic::transaction::runtime) conflict_policy_registry:
        FrozenConflictPolicyRegistry,
    pub(in crate::logic::transaction::runtime) identity_matcher_registry:
        FrozenIdentityMatcherRegistry,
    pub(in crate::logic::transaction::runtime) source_only_policy_registry:
        FrozenSourceOnlyPolicyRegistry,
    pub(in crate::logic::transaction::runtime) deletion_policy_registry:
        FrozenDeletionPolicyRegistry,
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
