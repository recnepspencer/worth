use super::{WorthQueryOperationGraphReadScope, WorthQueryOperationNativeProjectionContract};

use crate::domain_operation::{
    WorthQueryOperationGraphAccess, WorthQueryOperationGraphParticipation,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationGraphReadRole {
    role: String,
    participation: WorthQueryOperationGraphParticipation,
    access: WorthQueryOperationGraphAccess,
    read_scopes: std::sync::Arc<[WorthQueryOperationGraphReadScope]>,
    shared_identity: Option<String>,
    managed_index_reads: std::sync::Arc<
        [worth_query_declaration::facade::application_query::ApplicationQueryBindingReference],
    >,
}

impl WorthQueryOperationGraphReadRole {
    pub(crate) fn new(
        role: String,
        participation: WorthQueryOperationGraphParticipation,
        access: WorthQueryOperationGraphAccess,
        read_scopes: Vec<WorthQueryOperationGraphReadScope>,
    ) -> Self {
        Self {
            role,
            participation,
            access,
            read_scopes: read_scopes.into(),
            shared_identity: None,
            managed_index_reads: std::sync::Arc::from([]),
        }
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub const fn participation(&self) -> &WorthQueryOperationGraphParticipation {
        &self.participation
    }

    pub const fn access(&self) -> WorthQueryOperationGraphAccess {
        self.access
    }

    pub fn read_scopes(&self) -> &[WorthQueryOperationGraphReadScope] {
        &self.read_scopes
    }

    pub fn shared_scope_identity(&self) -> Option<&str> {
        self.shared_identity.as_deref()
    }

    pub(crate) fn shared(
        identity: &str,
        read_scopes: std::sync::Arc<[WorthQueryOperationGraphReadScope]>,
        managed_index_reads: std::sync::Arc<[worth_query_declaration::facade::application_query::ApplicationQueryBindingReference]>,
    ) -> Self {
        Self {
            role: format!("scope-{identity}"),
            participation: WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
            access: WorthQueryOperationGraphAccess::Project,
            read_scopes,
            shared_identity: Some(identity.to_owned()),
            managed_index_reads,
        }
    }

    pub(crate) fn managed_index_reads(
        &self,
    ) -> &[worth_query_declaration::facade::application_query::ApplicationQueryBindingReference]
    {
        &self.managed_index_reads
    }
}

/// Portable domain-operation role. It cannot impersonate a schema-bound
/// installed application read role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainOperationGraphReadRole {
    pub role: String,
    pub participation: WorthQueryOperationGraphParticipation,
    pub access: WorthQueryOperationGraphAccess,
    pub semantic_reads: Vec<WorthQueryOperationNativeProjectionContract>,
}
