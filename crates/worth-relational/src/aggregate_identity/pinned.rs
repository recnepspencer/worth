//! Ordinary aggregate acquisition from the existing owner-issued immutable pin.
use super::funding::{self, BuildResult, FundingFailure, PlanDenial};
use super::observations::{Acquisition, AggregateReadObservations};
use super::retention_extent as extent;
use crate::branch::RelationalBranchRootState;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::{EntityId, KindId};
use crate::runtime::RelationalPinnedRead;
use crate::validation::engine::state_view::InvariantStateView;
use worth_execution::{ExecutionRequest, MapKernelFailure};

mod contract;
mod coordination;
mod denial;
mod lineage;
mod lookup;
mod resolution;
mod result;
pub use denial::{PinnedAggregateReadDenial, PinnedAggregateReadDenialKind};
pub use lineage::{PinnedAggregateLineageRead, PinnedAggregateLineageScope};
pub use result::PinnedAggregateRead;

/// A source-qualified selection and optional narrowing constraints. Only the
/// actual pinned receiver and original request grant read/execution authority.
pub struct PinnedAggregateReadScope<'input> {
    relation: KindId,
    member: EntityId,
    limits: coordination::Limits<'input>,
}
impl<'input> PinnedAggregateReadScope<'input> {
    pub fn new(relation: KindId, member: EntityId) -> Self {
        Self {
            relation,
            member,
            limits: coordination::Limits::default(),
        }
    }
    pub fn with_work_limit(mut self, maximum: Option<u64>) -> Self {
        self.limits.maximum_work = maximum;
        self
    }
    pub fn with_control(mut self, control: &'input dyn Fn() -> Result<(), Cause>) -> Self {
        self.limits.control = Some(control);
        self
    }
}

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    /// Read one complete stored group and all its incident lineage from this
    /// exact root/schema. Work grows with its relation-kind roster, group and
    /// raw incident adjacency. No current catalog, whole-graph materialization
    /// or identity-only lookup is used.
    pub fn read_aggregate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedAggregateReadScope<'_>,
    ) -> Result<PinnedAggregateRead, PinnedAggregateReadDenial> {
        coordination::read(
            self,
            execution,
            scope.limits,
            |root, view, funded| {
                funded.retain(result::controls())?;
                let contract = contract::resolve(root, scope.relation, None, funded)?;
                let Some(member) = view.entity_metadata(scope.member) else {
                    return funded.invalid(PlanDenial::MemberUnavailable);
                };
                funded.acquired(Acquisition::Entity)?;
                if member.kind_id != contract.member_kind {
                    return funded.invalid(PlanDenial::WrongMemberKind);
                }
                let group = super::stored_read::collect(view, contract, scope.member, funded)?;
                let lineage = super::stored_lineage::read(view, &group.stored, contract, funded)?;
                funded.checkpoint(0)?;
                Ok(result::Value {
                    definition: contract.definition,
                    identity: group.stored.record.identity,
                    members: group.members,
                    lineage,
                })
            },
            PinnedAggregateRead::finish,
        )
    }
}
