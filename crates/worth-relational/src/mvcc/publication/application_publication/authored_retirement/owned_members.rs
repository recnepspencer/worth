//! Exact authored membership comes from live owner adjacency at the admitted basis.
use crate::{
    branch::AdmittedRelationalBranchBasis,
    identity::data::{EntityId, KindId},
    mvcc::{RequiredOutputRolePosture, RequiredOutputRoleTarget},
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    runtime::RelationalRuntime,
    transactions::data::{EntityReference, RecordRef, TransactionCommitError},
    validation::engine::state_view::InvariantStateView,
};
use worth_execution::ExecutionRequest;

/// Installed structural meaning, never a stored member roster or mutation permit.
#[derive(Clone, Copy, Debug)]
pub struct AuthoredOwnedMemberKind {
    pub member_kind: KindId,
    pub owner_relation: KindId,
}

impl RelationalRuntime {
    /// O(A + R log R) over this anchor's incoming owner adjacency A and the
    /// submitted member roles R. Membership has no second authoritative list.
    pub fn validate_authored_owned_members(
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
        basis
            .inner
            .root
            .demand_head_partition(execution, anchor.partition_id)
            .map_err(super::super::custody::execution_stop)?;
        let axes = basis
            .inner
            .root
            .axes()
            .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, RecordRef::Entity(anchor)))?;
        let state = InvariantStateView::new(
            basis.inner.root.as_ref(),
            crate::identity::data::VersionId(axes.storage_version),
        );
        execution
            .run_native_with_checkpoint(|context| {
                let mut expected = std::collections::BTreeSet::new();
                let mut supplied = std::collections::BTreeSet::new();
                let edges = state.relation_candidates(anchor, false);
                for &id in edges.iter() {
                    context
                        .checkpoint(kinds.len() as u64 + 1)
                        .map_err(super::super::custody::execution_stop)?;
                    basis
                        .inner
                        .root
                        .demand_head_partition(execution, id.partition_id)
                        .map_err(super::super::custody::execution_stop)?;
                    let Some(edge) = state.relation_metadata(id) else {
                        continue;
                    };
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
                        .inner
                        .root
                        .demand_head_partition(execution, edge.source.partition_id)
                        .map_err(super::super::custody::execution_stop)?;
                    if !state
                        .entity_metadata(edge.source)
                        .is_some_and(|member| member.kind_id == kind.member_kind)
                    {
                        return Err(reject(
                            Cause::DanglingOwnedMember,
                            RecordRef::Entity(edge.source),
                        ));
                    }
                    if !expected.insert(edge.source) {
                        return Err(reject(
                            Cause::RepeatedOwnedMember,
                            RecordRef::Entity(edge.source),
                        ));
                    }
                }
                for role in roles.iter().filter(|role| {
                    role.posture == RequiredOutputRolePosture::Retire
                        && kinds.iter().any(|kind| kind.member_kind == role.kind)
                }) {
                    context
                        .checkpoint(1)
                        .map_err(super::super::custody::execution_stop)?;
                    let EntityReference::Existing(id) = role.target else {
                        return Err(reject(Cause::ExtraOwnedMember, RecordRef::Entity(anchor)));
                    };
                    if !supplied.insert(id) {
                        return Err(reject(Cause::RepeatedOwnedMember, RecordRef::Entity(id)));
                    }
                    if !expected.contains(&id) {
                        return Err(reject(Cause::ForeignOwnedMember, RecordRef::Entity(id)));
                    }
                }
                if let Some(id) = expected.difference(&supplied).next() {
                    return Err(reject(Cause::MissingOwnedMember, RecordRef::Entity(*id)));
                }
                Ok(())
            })
            .map_err(super::super::custody::execution_stop)?
            .0
    }
}
