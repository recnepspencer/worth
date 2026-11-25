//! Declarative application binding for WORTH-governed shared scope access.

use std::fmt::Debug;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
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
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::capped_writer::RemoteByteWriter;
use super::denial::SharedAccessDenial;
use super::types::{ScopeAccess, ScopeGrantId};

/// Deterministic bounded binary wire codec for remote query payloads and rows.
///
/// Implementations must stream bytes directly into [`RemoteByteWriter`], which
/// charges every byte against the active frame ceiling before allocation.
pub trait RemoteBinaryCodec: Sized {
    fn encode_remote_bounded(
        &self,
        writer: &mut RemoteByteWriter,
    ) -> Result<(), SharedAccessDenial>;

    fn decode_remote(bytes: &[u8]) -> Result<Self, SharedAccessDenial>;
}

/// Parameters for building a durable scope-access grant mutation on the owner's Home.
pub struct RemoteWatchContext<'a, ScopeId> {
    pub grant_id: &'a ScopeGrantId,
    pub scope: &'a ScopeId,
    pub collaborator: &'a WorthQueryExternalPrincipalIdentity,
    pub access: ScopeAccess,
}

/// Application-program contract that wires a WORTH schema's real capability grants,
/// governed read query, and governed live watch query into `WorthQueryApplicationHome`.
///
/// The application never commits or authorizes on its own behalf: `worth-query-host`
/// executes the declared grant and revoke mutation intents directly through WORTH's
/// program runtime (requiring a genuine published commit receipt with non-zero record
/// changes before issuing a `ScopeAccessGrant`), and executes WORTH's real
/// `execute_governed`, `admit_capability_access`, and `open_governed_application_query_live`
/// directly on every remote request.
pub trait WorthQuerySharedAccessProgram<Schema>:
    ApplicationProgramDefinition<Schema> + Sized + 'static
where
    Schema: ApplicationSchema + 'static,
{
    type ScopeId: Clone + Eq + Ord + Debug + Send + Sync + 'static;
    type QueryPayload: RemoteBinaryCodec + Clone + Debug + Send + Sync + 'static;
    type Row: RemoteBinaryCodec + Clone + Debug + Eq + Send + Sync + 'static;

    type GrantMutationIntent: ApplicationMutationIntent<Schema> + Clone + Send + Sync + 'static;
    type RevokeMutationIntent: ApplicationMutationIntent<Schema> + Clone + Send + Sync + 'static;

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

    /// Builds the WORTH mutation intent that durably records a capability grant.
    fn build_grant_mutation_intent(
        grant: &RemoteWatchContext<'_, Self::ScopeId>,
    ) -> Result<Self::GrantMutationIntent, SharedAccessDenial>;

    /// Builds the deterministic idempotency key for a durable grant mutation.
    fn grant_idempotency_key(
        grant_id: &ScopeGrantId,
        sequence: u64,
    ) -> <<Self::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey;

    /// Builds the WORTH mutation intent that durably revokes a capability grant.
    fn build_revoke_mutation_intent(
        grant_id: &ScopeGrantId,
    ) -> Result<Self::RevokeMutationIntent, SharedAccessDenial>;

    /// Builds the deterministic idempotency key for a durable revocation mutation.
    fn revoke_idempotency_key(
        grant_id: &ScopeGrantId,
        sequence: u64,
    ) -> <<Self::RevokeMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey;

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
