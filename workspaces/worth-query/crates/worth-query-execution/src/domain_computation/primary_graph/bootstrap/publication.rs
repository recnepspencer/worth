use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

/// Evidence that a consumed bootstrap published the primary graph and all
/// declared identity indexes into one execution runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphPublication {
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) principal_binding_count: usize,
    pub(super) identity_index_count: usize,
    pub(super) application_equality_index_count: usize,
    pub(super) policy_entity_count: usize,
    pub(super) policy_relation_count: usize,
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
}
