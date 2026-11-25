//! Exact authored membership comes from live owner adjacency at the admitted basis.
use super::AuthoredPublicationReads;
use contracts::identity::data::primitives::KindId;
use contracts::identity::data::record_ids::EntityId;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::publication::data::publication_error::PublicationError;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
    AdmittedRelationalBranchBasisOwnerPort as _,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::validation::engine::state_view::InvariantStateView;
pub mod roster;
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
    pub fn validate_authored_owned_members(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        anchor: EntityId,
        kinds: &[AuthoredOwnedMemberKind],
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<(), TransactionCommitError> {
        use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
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
        basis
            .demand_head_partition(execution, anchor.partition_id)
            .map_err(super::super::custody::execution_stop)?;
        let axes = basis
            .owner_inner()
            .root
            .axes()
            .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, RecordRef::Entity(anchor)))?;
        let state = InvariantStateView::new(
            basis,
            contracts::identity::data::primitives::VersionId(axes.storage_version),
        );
        use contracts::execution::request_stage::checkpoint;
        execution
            .run_native_with_checkpoint(|context| {
                let mut roster = OwnedMemberRoster::new(anchor);
                let edges = state.relation_candidates(anchor, false);
                for &id in edges.iter() {
                    checkpoint(context, kinds.len() as u64 + 1)
                        .map_err(super::super::custody::execution_stop)?;
                    basis
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
                    basis
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
