//! Declarative application binding for WORTH-governed shared scope access.

use std::fmt::Debug;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_program::ApplicationProgramDefinition,
    application_query::{
        ApplicationLiveQueryIntent, ApplicationQueryBinding, ApplicationQueryScopeBinding,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
    authentication::WorthQueryExternalPrincipalIdentity,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::denial::SharedAccessDenial;
use super::types::{ScopeAccess, ScopeGrantId};

/// Deterministic binary wire codec for remote scope identifiers, queries, and rows.
pub trait RemoteBinaryCodec: Sized {
    fn encode_remote(&self) -> Vec<u8>;
    fn decode_remote(bytes: &[u8]) -> Result<Self, SharedAccessDenial>;
}

/// Parameters for committing a durable scope-access grant on the owner's Home.
pub struct RemoteQueryContext<
    'a,
    Schema: ApplicationSchema,
    Program: WorthQuerySharedAccessProgram<Schema>,
> {
    pub runtime: &'a WorthQueryProgramApplicationRuntime<Schema, Program>,
    pub owner: &'a WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub request_scope: &'a WorthQueryRequestScope,
}

/// Parameters for opening or revoking one scope grant on the owner's Home.
pub struct RemoteWatchContext<'a, ScopeId> {
    pub grant_id: &'a ScopeGrantId,
    pub scope: &'a ScopeId,
    pub collaborator: &'a WorthQueryExternalPrincipalIdentity,
    pub access: ScopeAccess,
}

/// Application-program contract that wires a WORTH schema's real capability grants,
/// governed read query, and governed live watch query into `WorthQueryApplicationHome`.
///
/// The application never performs ad-hoc authorization checks here: `worth-query-host`
/// executes WORTH's real `execute_governed`, `admit_capability_access`, and
/// `open_governed_application_query_live` directly using the declared capabilities.
pub trait WorthQuerySharedAccessProgram<Schema>:
    ApplicationProgramDefinition<Schema> + Sized + 'static
where
    Schema: ApplicationSchema + 'static,
{
    type ScopeId: Clone + Eq + Ord + Debug + Send + Sync + 'static;
    type QueryPayload: RemoteBinaryCodec + Clone + Debug + Send + Sync + 'static;
    type Row: RemoteBinaryCodec + Clone + Debug + Eq + Send + Sync + 'static;

    type QueryIntent: ApplicationLiveQueryIntent<Schema>;
    type ReadCapability: 'static;
    type ReadOperation: ApplicationOperationMarkerIdentity<Schema> + 'static;
    type ReadCapabilityInput: ApplicationCapabilityRequest<
            Schema,
            Self::ReadCapability,
            Scope = <<<Self::QueryIntent as worth_query_declaration::facade::application_query::ApplicationQueryIntent<Schema>>::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        > + Clone
        + 'static;

    type WatchCapability: 'static;
    type WatchOperation: ApplicationOperationMarkerIdentity<Schema> + 'static;
    type WatchCapabilityInput: ApplicationCapabilityRequest<
            Schema,
            Self::WatchCapability,
            Scope = <<<Self::QueryIntent as worth_query_declaration::facade::application_query::ApplicationQueryIntent<Schema>>::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        > + Clone
        + 'static;

    fn scope_key(scope: &Self::ScopeId) -> String;
    fn parse_scope_key(key: &str) -> Result<Self::ScopeId, SharedAccessDenial>;

    /// Authenticates the local Home owner for grant/revoke mutations.
    fn authenticate_owner(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        request_scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryAuthenticatedExternalPrincipal<Schema>, SharedAccessDenial>;

    /// Commits a durable WORTH capability grant record in the owner's primary graph.
    fn commit_grant_scope_access(
        ctx: &RemoteQueryContext<'_, Schema, Self>,
        grant: &RemoteWatchContext<'_, Self::ScopeId>,
    ) -> Result<(), SharedAccessDenial>;

    /// Commits a durable WORTH capability revocation in the owner's primary graph.
    fn commit_revoke_scope_access(
        ctx: &RemoteQueryContext<'_, Schema, Self>,
        grant_id: &ScopeGrantId,
    ) -> Result<(), SharedAccessDenial>;

    fn build_query_intent(
        scope: &Self::ScopeId,
        payload: &Self::QueryPayload,
    ) -> Result<Self::QueryIntent, SharedAccessDenial>;

    fn read_capability() -> ApplicationCapabilityRef<Schema, Self::ReadCapability>;
    fn read_operation(
    ) -> ApplicationOperationRef<Schema, Self::ReadOperation, Self::ReadCapabilityInput>;
    fn build_read_capability_input(scope: &Self::ScopeId) -> Self::ReadCapabilityInput;

    fn watch_capability() -> ApplicationCapabilityRef<Schema, Self::WatchCapability>;
    fn watch_operation(
    ) -> ApplicationOperationRef<Schema, Self::WatchOperation, Self::WatchCapabilityInput>;
    fn build_watch_capability_input(scope: &Self::ScopeId) -> Self::WatchCapabilityInput;

    fn project_row(
        value: <<<Self::QueryIntent as worth_query_declaration::facade::application_query::ApplicationQueryIntent<Schema>>::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> Self::Row;
}
