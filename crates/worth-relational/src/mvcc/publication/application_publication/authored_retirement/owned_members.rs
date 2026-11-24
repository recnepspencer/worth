//! Exact authored membership comes from live owner adjacency at the admitted basis.
use super::AuthoredPublicationReads;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::EntityId;
use crate::identity::data::KindId;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::publication::data::PublicationError;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use crate::validation::engine::state_view::InvariantStateView;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;
mod roster;
use roster::OwnedMemberRoster;

/// Installed structural meaning, never a stored member roster or mutation permit.
#[derive(Clone, Copy, Debug)]
pub struct AuthoredOwnedMemberKind {
    pub member_kind: KindId,
    pub owner_relation: KindId,
}

impl AuthoredPublicationReads<'_> {
    /// O((A + R) K + A log A + R log R) over incoming owner adjacency A,
    /// submitted member roles R and declared member kinds K. Membership has no second authoritative list.
    /// A request with no work account is polled, not charged.
    pub(crate) fn validate_authored_owned_members(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        anchor: EntityId,
        kinds: &[AuthoredOwnedMemberKind],
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<(), TransactionCommitError> {
        use crate::storage::overlay::CurrentPartitionAccess;
        let reject = |cause, record| {
            TransactionCommitError::publication(PublicationError::authored_retirement(
                cause, record,
            ))
        };
        if basis.identity().runtime_instance_id() != self.runtime_instance_id()
            || !basis.is_current()
        {
            return Err(reject(
                Cause::InvalidAuthoredAnchor,
                RecordRef::Entity(anchor),
            ));
        }
        (*basis.owner_inner())
            .root
            .demand_head_partition(execution, anchor.partition_id)
            .map_err(super::super::custody::execution_stop)?;
        let axes = (*basis.owner_inner())
            .root
            .axes()
            .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, RecordRef::Entity(anchor)))?;
        let state = InvariantStateView::new(
            (*basis.owner_inner()).root.as_ref(),
            crate::identity::data::VersionId(axes.storage_version),
        );
        use crate::execution::request_stage::checkpoint;
        execution
            .run_native_with_checkpoint(|context| {
                let mut roster = OwnedMemberRoster::new(anchor);
                let edges = state.relation_candidates(anchor, false);
                for &id in edges.iter() {
                    checkpoint(context, kinds.len() as u64 + 1)
                        .map_err(super::super::custody::execution_stop)?;
                    (*basis.owner_inner())
                        .root
                        .demand_head_partition(execution, id.partition_id)
                        .map_err(super::super::custody::execution_stop)?;
                    let edge = roster::owner_metadata(id, state.relation_metadata(id))?;
                    if edge.target != anchor {
                        continue;
                    }
                    let Some(kind) = kinds
                        .iter()
                        .find(|kind| kind.owner_relation == edge.kind_id)
                    else {
                        continue;
                    };
                    (*basis.owner_inner())
                        .root
                        .demand_head_partition(execution, edge.source.partition_id)
                        .map_err(super::super::custody::execution_stop)?;
                    roster.observe_member(
                        edge.source,
                        state
                            .entity_metadata(edge.source)
                            .map(|member| member.kind_id),
                        kind.member_kind,
                    )?;
                }
                for role in roles.iter().filter(|role| {
                    role.posture == RequiredOutputRolePosture::Retire
                        && kinds.iter().any(|kind| kind.member_kind == role.kind)
                }) {
                    checkpoint(context, 1).map_err(super::super::custody::execution_stop)?;
                    roster.supply(role)?;
                }
                roster.finish()
            })
            .map_err(super::super::custody::execution_stop)?
            .0
    }
}
