//! A resolved read scope is admitted only by the operation's declared
//! authority: its own scope through `operation_managed_index_read`, a peer
//! scope through `operation_shared_owner_managed_index_read` and an owner it
//! shares with the operation's scope. Every other scope or principal is
//! refused as `ForeignAssociation` before any Parent or provider work.
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationScopeBinding, WorthQueryOperationScopeEntityBinding as ScopeEntity,
};
use worth_query_execution_lineage::facade::primary_graph::invariant_projection::operation_reader::SharedOwnerRelation;
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, WorthQueryCompiledApplicationOperationContracts,
    WorthQuerySharedOwnerManagedIndexRead,
};
use worth_relational::facade::identity::EntityId;

pub(super) struct DeclaredScopeAuthority {
    operation: WorthQueryOperationScopeBinding,
    own: bool,
    peer: Option<WorthQuerySharedOwnerManagedIndexRead>,
}

impl DeclaredScopeAuthority {
    /// The declared authority for Parent `B`; neither declaration refuses as
    /// the undeclared own-scope target, exactly as before.
    pub(super) fn admit<Schema: ApplicationSchema, Operation, B>(
        reader: &WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Operation,
        >,
        contracts: &WorthQueryCompiledApplicationOperationContracts,
    ) -> Result<Self, Failure>
    where
        B: ApplicationQueryBinding<Schema>,
    {
        let parent =
            worth_query_declaration::facade::application_query::ApplicationQueryBindingReference::of::<Schema, B>();
        let peer = contracts.shared_owner_managed_index_read(&parent).cloned();
        let own = reader.admit_decision_target(&ApplicationOperationDecisionReadTarget::ManagedIndex {
            parent,
        });
        let own = match (own, &peer) {
            (Ok(()), _) => true,
            (Err(_), Some(_)) => false,
            (Err(cause), None) => return Err(Failure::Undeclared(cause)),
        };
        let operation = reader.operation_scope().ok_or_else(unavailable)?.clone();
        Ok(Self {
            operation,
            own,
            peer,
        })
    }

    /// Whether the read's resolved `scope` and `principal` are admitted. A
    /// peer is reached only through recorded reads of the governing relation;
    /// a failed reach also fails its fact, so the decision seals nothing.
    pub(super) fn admit_scope<Schema: ApplicationSchema, Operation>(
        &self,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Operation,
        >,
        scope: EntityId,
        principal: EntityId,
    ) -> bool {
        if ScopeEntity::from_entity(principal) != self.operation.principal() {
            return false;
        }
        if ScopeEntity::from_entity(scope) == self.operation.scope() {
            return self.own;
        }
        let Some(peer) = &self.peer else {
            return false;
        };
        reader
            .shares_relation_owner(
                &SharedOwnerRelation {
                    relation: peer.relation(),
                    owner: peer.owner(),
                    member: peer.member(),
                },
                scope,
            )
            .unwrap_or(false)
    }
}
