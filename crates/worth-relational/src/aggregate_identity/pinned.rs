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

mod denial;
mod result;
pub use denial::{PinnedAggregateReadDenial, PinnedAggregateReadDenialKind};
pub use result::PinnedAggregateRead;

/// A source-qualified selection and optional narrowing constraints. Only the
/// actual pinned receiver and original request grant read/execution authority.
pub struct PinnedAggregateReadScope<'input> {
    relation: KindId,
    member: EntityId,
    maximum_work: Option<u64>,
    control: Option<&'input dyn Fn() -> Result<(), Cause>>,
}
impl<'input> PinnedAggregateReadScope<'input> {
    pub fn new(relation: KindId, member: EntityId) -> Self {
        Self {
            relation,
            member,
            maximum_work: None,
            control: None,
        }
    }
    pub fn with_work_limit(mut self, maximum: Option<u64>) -> Self {
        self.maximum_work = maximum;
        self
    }
    pub fn with_control(mut self, control: &'input dyn Fn() -> Result<(), Cause>) -> Self {
        self.control = Some(control);
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
        let mut reads = AggregateReadObservations::default();
        let mut work = 0u64;
        let mut settled = None;
        let handoff = execution.run_native_with_checkpoint(|context| {
            settled = Some((|| {
                context
                    .poll()
                    .map_err(|stop| FundingFailure::Execution(stop.into()))?;
                let root = RelationalBranchRootState::new(std::sync::Arc::clone(
                    self.pinned_basis().root(),
                ));
                let view = InvariantStateView::new(&root, self.pinned_basis().version_id());
                let mut checkpoint = |units| read_checkpoint(&scope, &mut work, units);
                let (value, retention) =
                    funding::run_read(execution, &mut checkpoint, &mut reads, |funded| {
                        funded.retain(result::controls())?;
                        let mut contract = None;
                        for kind in self.schema().relation_kinds.values() {
                            funded.checkpoint(1)?;
                            if let Some(found) = kind
                                .relation_integrity
                                .aggregate_identity
                                .filter(|c| c.source_relation == scope.relation)
                            {
                                if contract.replace(found).is_some() {
                                    return funded.invalid(PlanDenial::AmbiguousAggregateRelation);
                                }
                            }
                        }
                        let Some(contract) = contract else {
                            return funded.invalid(PlanDenial::NotAggregateRelation);
                        };
                        let Some(member) = view.entity_metadata(scope.member) else {
                            return funded.invalid(PlanDenial::MemberUnavailable);
                        };
                        funded.acquired(Acquisition::Entity)?;
                        if member.kind_id != contract.member_kind {
                            return funded.invalid(PlanDenial::WrongMemberKind);
                        }
                        let group =
                            super::stored_read::collect(&view, contract, scope.member, funded)?;
                        let lineage =
                            super::stored_lineage::read(&view, &group.stored, contract, funded)?;
                        funded.checkpoint(0)?;
                        Ok(result::Value {
                            definition: contract.definition,
                            identity: group.stored.record.identity,
                            members: group.members,
                            lineage,
                        })
                    })?;
                let result = PinnedAggregateRead::finish(value, reads, work, retention);
                if let Some(control) = scope.control {
                    control().map_err(FundingFailure::Execution)?;
                }
                context
                    .poll()
                    .map_err(|stop| FundingFailure::Execution(stop.into()))?;
                Ok(result)
            })());
        });
        let result = match (handoff, settled) {
            (Ok(((), _)), Some(result)) => result,
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(failure)),
            ) => Err(failure),
            (Err(stop), _) => Err(FundingFailure::Execution(stop.into())),
            (Ok(_), None) => unreachable!("successful Native entry invokes its read"),
        };
        result.map_err(|failure| PinnedAggregateReadDenial {
            kind: denial::kind(failure),
            reads,
            work,
        })
    }
}

fn read_checkpoint(
    scope: &PinnedAggregateReadScope<'_>,
    work: &mut u64,
    units: u64,
) -> BuildResult<()> {
    *work = work
        .checked_add(units)
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    if let Some(control) = scope.control {
        control().map_err(|cause| MapKernelFailure::Domain(PlanDenial::Execution(cause)))?;
    }
    if scope.maximum_work.is_some_and(|maximum| *work > maximum) {
        return Err(MapKernelFailure::Domain(PlanDenial::ReadWorkLimitExceeded));
    }
    Ok(())
}
