//! Ordinary aggregate acquisition from the existing owner-issued immutable pin.
use super::funding::{self, BuildResult, FundingFailure, PlanDenial};
use super::observations::{Acquisition, AggregateReadObservations};
use super::retention_extent as extent;
use crate::contracts::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::contracts::graph_member::key::{NativeMemberKey, ProjectedMemberCoordinate};
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::validation::engine::state_view::InvariantStateView;
use worth_execution::{ExecutionRequest, MapKernelFailure};
use worth_relational_branch::internal_port::branch::root::RelationalBranchRootState;
use worth_relational_branch::internal_port::history::retention::selected_snapshot::VisibilitySnapshotBasis;

use super::selection::AggregateSelection;
use worth_foundational::facade::AggregateDefinitionIdentity;

pub mod contract;
pub mod coordination;
pub mod denial;
pub mod lineage;
pub mod lookup;
pub mod member;
pub mod proposed;
pub mod resolution;
pub mod result;
pub use denial::{PinnedAggregateReadDenial, PinnedAggregateReadDenialKind};
pub use lineage::read_aggregate_lineage;
pub use lineage::{PinnedAggregateLineageRead, PinnedAggregateLineageScope};
pub use result::PinnedAggregateRead;

/// A source-qualified selection and optional narrowing constraints. Only the
/// actual pinned receiver and original request grant read/execution authority.
pub struct PinnedAggregateReadScope<'input, Projection = EntityId> {
    selection: AggregateSelection,
    member: NativeMemberKey,
    _projection: std::marker::PhantomData<fn() -> Projection>,
    limits: coordination::Limits<'input>,
}
impl<'input> PinnedAggregateReadScope<'input> {
    pub fn new(relation: KindId, member: EntityId) -> Self {
        Self {
            selection: AggregateSelection::Orbit {
                relation,
                definition: None,
            },
            member: member.into(),
            _projection: std::marker::PhantomData,
            limits: coordination::Limits::default(),
        }
    }
    /// Select the actual installed component/orbit definition without relation ambiguity.
    pub fn for_definition(definition: AggregateDefinitionIdentity, member: EntityId) -> Self {
        Self {
            selection: AggregateSelection::Definition(definition),
            member: member.into(),
            _projection: std::marker::PhantomData,
            limits: coordination::Limits::default(),
        }
    }
}
impl<'input> PinnedAggregateReadScope<'input, NativeMemberKey> {
    /// A descriptor selects within this genuine broad pin; it grants no
    /// standalone source or mutation authority. The read verifies the actual
    /// installed domain/case and full carrier generation at that exact root.
    pub fn for_projected_definition(
        definition: AggregateDefinitionIdentity,
        member: ProjectedMemberCoordinate,
    ) -> Self {
        Self::for_member_definition(
            definition,
            NativeMemberKey::Projected(std::sync::Arc::new(member).into()),
        )
    }
    /// Descriptive tagged member selection. The actual pinned read validates
    /// the installed domain, case and carrier generation before returning data.
    pub fn for_member_definition(
        definition: AggregateDefinitionIdentity,
        member: NativeMemberKey,
    ) -> Self {
        Self {
            selection: AggregateSelection::Definition(definition),
            member,
            _projection: std::marker::PhantomData,
            limits: coordination::Limits::default(),
        }
    }
}
impl<'input, Projection> PinnedAggregateReadScope<'input, Projection> {
    pub fn with_work_limit(mut self, maximum: Option<u64>) -> Self {
        self.limits.maximum_work = maximum;
        self
    }
    pub fn with_control(mut self, control: &'input dyn Fn() -> Result<(), Cause>) -> Self {
        self.limits.control = Some(control);
        self
    }
}

pub fn read_aggregate_scope<Projection>(
    basis: &VisibilitySnapshotBasis,
    candidate: Option<&super::AggregateCandidateRead>,
    execution: &ExecutionRequest<'_, '_>,
    scope: PinnedAggregateReadScope<'_, Projection>,
) -> Result<PinnedAggregateRead<Projection>, PinnedAggregateReadDenial> {
    coordination::read(
        basis,
        execution,
        scope.limits,
        |root, view, funded| {
            funded.retain(result::controls())?;
            let contract = contract::resolve(root, scope.selection, funded)?;
            let registry = root.schema_authority().registry();
            member::validate(view, registry, contract, &scope.member, funded)?;
            if let Some(candidate) = candidate
                && let Some(value) = proposed::read(candidate, contract, &scope.member, funded)?
            {
                return Ok(value);
            }
            let group =
                super::stored_read::collect(view, contract, registry, scope.member, funded)?;
            let lineage =
                super::stored_lineage::read(view, &group.stored, contract, registry, funded)?;
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
