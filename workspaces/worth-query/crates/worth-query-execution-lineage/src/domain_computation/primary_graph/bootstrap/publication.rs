use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

/// Evidence that a consumed bootstrap published the primary graph and all
/// declared identity indexes into one execution runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphPublication {
    pub(crate) binding_identity: ApplicationSchemaBindingIdentity,
    pub(crate) principal_binding_count: usize,
    pub(crate) identity_index_count: usize,
    pub(crate) application_equality_index_count: usize,
    pub(crate) policy_entity_count: usize,
    pub(crate) policy_relation_count: usize,
    pub(crate) bootstrap_commit_id: worth_relational::facade::history::CommitId,
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

    pub const fn bootstrap_commit_id(&self) -> worth_relational::facade::history::CommitId {
        self.bootstrap_commit_id
    }
}

mod court_doors_for_test;

/// Crate-boundary internal port for `WorthQueryPrimaryGraphPublication`
/// construction. Never exported on `worth-query-execution::facade` or Host.
pub trait PrimaryGraphPublicationInternalPort: Sized {
    fn from_parts(
        binding_identity: ApplicationSchemaBindingIdentity,
        principal_binding_count: usize,
        identity_index_count: usize,
        application_equality_index_count: usize,
        policy_entity_count: usize,
        policy_relation_count: usize,
        bootstrap_commit_id: worth_relational::facade::history::CommitId,
    ) -> Self;
}

impl PrimaryGraphPublicationInternalPort for WorthQueryPrimaryGraphPublication {
    fn from_parts(
        binding_identity: ApplicationSchemaBindingIdentity,
        principal_binding_count: usize,
        identity_index_count: usize,
        application_equality_index_count: usize,
        policy_entity_count: usize,
        policy_relation_count: usize,
        bootstrap_commit_id: worth_relational::facade::history::CommitId,
    ) -> Self {
        Self {
            binding_identity,
            principal_binding_count,
            identity_index_count,
            application_equality_index_count,
            policy_entity_count,
            policy_relation_count,
            bootstrap_commit_id,
        }
    }
}
