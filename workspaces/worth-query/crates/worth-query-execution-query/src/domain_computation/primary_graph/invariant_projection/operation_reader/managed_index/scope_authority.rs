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
use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionTraversalDenial;
use worth_relational::facade::identity::EntityId;

pub(super) struct DeclaredScopeAuthority {
    operation: WorthQueryOperationScopeBinding,
    own: bool,
    /// One declared peer read per governing relation.
    peers: Vec<WorthQuerySharedOwnerManagedIndexRead>,
    /// A stop that interrupted reaching a peer's owner: not a foreign scope.
    stopped: std::cell::RefCell<Option<WorthQueryInvariantProjectionTraversalDenial>>,
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
        let peers: Vec<_> = contracts
            .shared_owner_managed_index_reads(&parent)
            .cloned()
            .collect();
        let own = reader.admit_decision_target(&ApplicationOperationDecisionReadTarget::ManagedIndex {
            parent,
        });
        let own = match own {
            Ok(()) => true,
            Err(_) if !peers.is_empty() => false,
            Err(cause) => return Err(Failure::Undeclared(cause)),
        };
        let operation = reader.operation_scope().ok_or_else(unavailable)?.clone();
        Ok(Self {
            operation,
            own,
            peers,
            stopped: std::cell::RefCell::new(None),
        })
    }

    /// Whether the read's resolved `scope` and `principal` are admitted. A
    /// peer is reached only through recorded reads of a governing relation;
    /// a stopped reach is kept for `take_stop` and fails its fact.
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
        // Any declared governing relation that reaches the peer admits it.
        for peer in &self.peers {
            let relation = SharedOwnerRelation {
                relation: peer.relation(),
                owner: peer.owner(),
                member: peer.member(),
            };
            match reader.shares_relation_owner(&relation, scope) {
                Ok(true) => return true,
                Ok(false) => {}
                Err(stop) => {
                    self.stopped.replace(Some(stop));
                    return false;
                }
            }
        }
        false
    }

    /// The stop that interrupted reaching a peer, which outranks the
    /// read's refusal it caused.
    pub(super) fn take_stop(&self) -> Option<WorthQueryInvariantProjectionTraversalDenial> {
        self.stopped.take()
    }
}
