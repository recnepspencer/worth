//! Moved whole from `domain_computation::primary_graph::bootstrap::publication`: the items below and every impl they own.
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

/// Evidence that a consumed bootstrap published the primary graph and all
/// declared identity indexes into one execution runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphPublication {
    pub(in crate::domain_computation::primary_graph) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) principal_binding_count: usize,
    pub(in crate::domain_computation::primary_graph) identity_index_count: usize,
    pub(in crate::domain_computation::primary_graph) application_equality_index_count: usize,
    pub(in crate::domain_computation::primary_graph) policy_entity_count: usize,
    pub(in crate::domain_computation::primary_graph) policy_relation_count: usize,
    pub(in crate::domain_computation::primary_graph) bootstrap_commit_id:
        worth_relational::facade::history::CommitId,
}

impl WorthQueryPrimaryGraphPublication {
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub const fn principal_binding_count(&self) -> usize {
        self.principal_binding_count
    }

    pub const fn identity_index_count(&self) -> usize {
        self.identity_index_count
    }

    pub const fn application_equality_index_count(&self) -> usize {
        self.application_equality_index_count
    }

    pub const fn policy_entity_count(&self) -> usize {
        self.policy_entity_count
    }

    pub const fn policy_relation_count(&self) -> usize {
        self.policy_relation_count
    }

    pub(in crate::domain_computation::primary_graph) const fn bootstrap_commit_id(
        &self,
    ) -> worth_relational::facade::history::CommitId {
        self.bootstrap_commit_id
    }
}

impl WorthQueryPrimaryGraphPublication {
    #[doc(hidden)]
    pub fn bootstrap_commit_id_for_test(&self) -> worth_relational::facade::history::CommitId {
        Self::bootstrap_commit_id(self)
    }
}
