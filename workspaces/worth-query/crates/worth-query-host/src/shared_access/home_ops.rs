//! Owner-side capability grant, revocation, and remote hosting on `WorthQueryApplicationHome`.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
    },
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchemaComposition,
        ApplicationStructuredValueBinding,
    },
    authentication::WorthQueryExternalPrincipalIdentity,
};
use worth_query_execution::facade::{
    primary_graph::WorthQueryApplicationProjection, runtime::ExecutionAllocationPolicy,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
};

use crate::application_home::WorthQueryApplicationHome;

use super::auth::RemoteAuthConfig;
use super::binding::{RemoteWatchContext, WorthQuerySharedAccessProgram};
use super::denial::SharedAccessDenial;
use super::server::WorthQueryRemoteServer;
use super::types::{CommittedGrantReceipt, ScopeAccess, ScopeAccessGrant, ScopeGrantId};

const OWNER_OPERATION_DEADLINE_SECS: u64 = 30;
static NEXT_GRANT_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static NEXT_REVOKE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchemaComposition + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    /// Grants a collaborator governed read (or read+watch) capability strictly for `scope`.
    ///
    /// Executes the program's declared grant mutation directly through WORTH's own
    /// mutation runtime and refuses unless a genuine published commit receipt with
    /// non-zero durable record changes is produced.
    pub fn grant_scope_access(
        &self,
        scope: Program::ScopeId,
        principal: WorthQueryExternalPrincipalIdentity,
        access: ScopeAccess,
    ) -> Result<ScopeAccessGrant, SharedAccessDenial>
    where
        <<Program::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::Input:
            Clone + Send + Sync,
        <<Program::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <<Program::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
        <Program::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding:
            ApplicationMutationBinding<
                Schema,
                SourceExpectation = worth_query_declaration::facade::application_operation::NoApplicationMutationSource,
            >,
{
        let scope_key = Program::scope_key(&scope);
        let home_identity = self.home_identity();
        let home_incarnation = self.home_incarnation();
        let seq = NEXT_GRANT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let grant_id = derive_grant_id(
            home_incarnation.map(|i| i.get()),
            seq,
            &scope_key,
            &principal,
        )?;
        let (_cancel, request_scope) = owner_request_scope();
        let owner = Program::authenticate_owner(
            self.runtime().runtime().installed_schema(),
            &request_scope,
        )?;
        let watch_ctx = RemoteWatchContext {
            grant_id: &grant_id,
            scope: &scope,
            collaborator: &principal,
            access,
        };
        let intent = Program::build_grant_mutation_intent(&watch_ctx)?;
        let key = Program::grant_idempotency_key(&grant_id, seq);
        let committed_record_changes =
            self.execute_grant_mutation(&owner, &request_scope, intent, &key)?;
        Ok(ScopeAccessGrant::from_committed(CommittedGrantReceipt {
            id: grant_id,
            scope_key,
            principal,
            access,
            home_identity,
            home_incarnation,
            committed_record_changes,
        }))
    }

    fn execute_grant_mutation(
        &self,
        owner: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        intent: Program::GrantMutationIntent,
        key: &<<Program::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    ) -> Result<usize, SharedAccessDenial>
    where
        <<Program::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::Input:
            Clone + Send + Sync,
        <<Program::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <<Program::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
        <Program::GrantMutationIntent as ApplicationMutationIntent<Schema>>::Binding:
            ApplicationMutationBinding<
                Schema,
                SourceExpectation = worth_query_declaration::facade::application_operation::NoApplicationMutationSource,
            >,
{
        let outcome = self
            .runtime()
            .request(owner, request_scope)
            .advanced()
            .mutate(intent)
            .without_source()
            .idempotency(key)
            .execute_in_program(self.runtime(), ExecutionAllocationPolicy::SystemAllocation)
            .map_err(|err| SharedAccessDenial::GrantMutationRefused(format!("{err:?}")))?;
        require_committed_changes(outcome)
    }

    /// Revokes a previously issued scope-access grant immediately and durably.
    ///
    /// Executes the program's declared revocation mutation directly through WORTH's
    /// mutation runtime and refuses unless a genuine published commit receipt with
    /// non-zero durable record changes is produced.
    pub fn revoke_scope_access(&self, grant_id: &ScopeGrantId) -> Result<(), SharedAccessDenial>
    where
        <<Program::RevokeMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::Input:
            Clone + Send + Sync,
        <<Program::RevokeMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <<Program::RevokeMutationIntent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
        <Program::RevokeMutationIntent as ApplicationMutationIntent<Schema>>::Binding:
            ApplicationMutationBinding<
                Schema,
                SourceExpectation = worth_query_declaration::facade::application_operation::NoApplicationMutationSource,
            >,
{
        let seq = NEXT_REVOKE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let (_cancel, request_scope) = owner_request_scope();
        let owner = Program::authenticate_owner(
            self.runtime().runtime().installed_schema(),
            &request_scope,
        )?;
        let intent = Program::build_revoke_mutation_intent(grant_id)?;
        let key = Program::revoke_idempotency_key(grant_id, seq);
        let outcome = self
            .runtime()
            .request(&owner, &request_scope)
            .advanced()
            .mutate(intent)
            .without_source()
            .idempotency(&key)
            .execute_in_program(self.runtime(), ExecutionAllocationPolicy::SystemAllocation)
            .map_err(|err| SharedAccessDenial::GrantMutationRefused(format!("{err:?}")))?;
        let _ = require_committed_changes(outcome)?;
        Ok(())
    }

    /// Binds a single-owner authenticated remote server backed exclusively by this Directory Home.
    ///
    /// Direct TCP binding is strictly loopback-only (`127.0.0.1` / `::1`) because frames carry
    /// reusable bearer tokens and row payloads without transport encryption. For cross-host
    /// deployment, tunnel loopback over an authenticated encrypted channel (TLS/mTLS, Tailscale,
    /// or SSH) or drive `WorthQueryRemoteServer::handle_frame` over an encrypted transport.
    pub fn serve_remote(
        &self,
        bind_addr: SocketAddr,
        auth_config: RemoteAuthConfig,
    ) -> Result<WorthQueryRemoteServer<'_, Schema, Program>, SharedAccessDenial>
    where
        <<Program::QueryIntent as ApplicationQueryIntent<Schema>>::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <<Program::QueryIntent as ApplicationQueryIntent<Schema>>::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        <<<Program::QueryIntent as ApplicationQueryIntent<Schema>>::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<
                Schema,
                <<Program::QueryIntent as ApplicationQueryIntent<Schema>>::Binding as ApplicationQueryBinding<Schema>>::Query,
            > + Clone,
        <Program::ReadOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            ApplicationStructuredValueBinding<Value = Program::ReadCapabilityInput>,
        <Program::WatchOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            ApplicationStructuredValueBinding<Value = Program::WatchCapabilityInput>,
{
        WorthQueryRemoteServer::bind(self, bind_addr, auth_config)
    }
}

fn owner_request_scope() -> (WorthQueryCancellationSource, WorthQueryRequestScope) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(OWNER_OPERATION_DEADLINE_SECS)),
        cancellation.token(),
        WorthQueryAdmissionClock::system(),
    );
    (cancellation, scope)
}

fn require_committed_changes<Denial, ResultVal>(
    outcome: WorthQueryApplicationMutationOutcome<Denial, ResultVal>,
) -> Result<usize, SharedAccessDenial> {
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => {
            let changed = receipt.changed_record_count();
            if changed == 0 {
                return Err(SharedAccessDenial::GrantMutationRefused(
                    "grant mutation committed without changing any durable graph record".to_owned(),
                ));
            }
            Ok(changed)
        }
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(_) => Err(
            SharedAccessDenial::GrantMutationRefused("mutation already committed".to_owned()),
        ),
        WorthQueryApplicationMutationOutcome::RecoveredCommitted(_) => {
            Err(SharedAccessDenial::GrantMutationRefused(
                "mutation recovered without fresh commit".to_owned(),
            ))
        }
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift => Err(
            SharedAccessDenial::GrantMutationRefused("idempotency intent drift".to_owned()),
        ),
        WorthQueryApplicationMutationOutcome::DomainDenied(_) => Err(
            SharedAccessDenial::GrantMutationRefused("domain denied grant mutation".to_owned()),
        ),
        WorthQueryApplicationMutationOutcome::Cancelled => Err(
            SharedAccessDenial::GrantMutationRefused("grant mutation cancelled".to_owned()),
        ),
        WorthQueryApplicationMutationOutcome::DeadlineExceeded => Err(
            SharedAccessDenial::GrantMutationRefused("grant mutation deadline exceeded".to_owned()),
        ),
        WorthQueryApplicationMutationOutcome::Commit(_) => {
            Err(SharedAccessDenial::GrantMutationRefused(
                "grant mutation did not publish commit".to_owned(),
            ))
        }
    }
}

fn derive_grant_id(
    incarnation: Option<u64>,
    seq: u64,
    scope_key: &str,
    principal: &WorthQueryExternalPrincipalIdentity,
) -> Result<ScopeGrantId, SharedAccessDenial> {
    let inc = incarnation.unwrap_or(0);
    let mut hasher = Sha256::new();
    hasher.update(b"worth.shared_access.grant_id.v1\0");
    hasher.update(inc.to_be_bytes());
    hasher.update(seq.to_be_bytes());
    hasher.update(scope_key.as_bytes());
    hasher.update(b"\0");
    hasher.update(principal.issuer().as_bytes());
    hasher.update(b"\0");
    hasher.update(principal.subject().as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    let short_hex = encode_hex_8(&digest[..8]);
    ScopeGrantId::parse(format!("grant-{inc}-{seq}-{short_hex}"))
}

fn encode_hex_8(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}
