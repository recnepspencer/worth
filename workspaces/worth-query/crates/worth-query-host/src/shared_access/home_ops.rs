//! Owner-side capability grant, revocation, and remote hosting on `WorthQueryApplicationHome`.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationSchemaComposition, ApplicationStructuredValueBinding},
    authentication::WorthQueryExternalPrincipalIdentity,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;

use crate::application_home::WorthQueryApplicationHome;

use super::auth::RemoteAuthConfig;
use super::binding::{RemoteQueryContext, RemoteWatchContext, WorthQuerySharedAccessProgram};
use super::denial::SharedAccessDenial;
use super::server::WorthQueryRemoteServer;
use super::types::{CommittedGrantReceipt, ScopeAccess, ScopeAccessGrant, ScopeGrantId};

const OWNER_OPERATION_DEADLINE_SECS: u64 = 30;
static NEXT_GRANT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchemaComposition + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    /// Grants a collaborator governed read (or read+watch) capability strictly for `scope`.
    ///
    /// Commits a durable grant into this Home's authoritative WORTH primary graph.
    pub fn grant_scope_access(
        &self,
        scope: Program::ScopeId,
        principal: WorthQueryExternalPrincipalIdentity,
        access: ScopeAccess,
    ) -> Result<ScopeAccessGrant, SharedAccessDenial> {
        let scope_key = Program::scope_key(&scope);
        let home_identity = self.home_identity();
        let home_incarnation = self.home_incarnation();
        let grant_id = derive_grant_id(home_incarnation.map(|i| i.get()), &scope_key, &principal)?;
        let cancellation = WorthQueryCancellationSource::new();
        let request_scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(OWNER_OPERATION_DEADLINE_SECS)),
            cancellation.token(),
            WorthQueryAdmissionClock::system(),
        );
        let owner = Program::authenticate_owner(
            self.runtime().runtime().installed_schema(),
            &request_scope,
        )?;
        let ctx = RemoteQueryContext {
            runtime: self.runtime(),
            owner: &owner,
            request_scope: &request_scope,
        };
        let watch_ctx = RemoteWatchContext {
            grant_id: &grant_id,
            scope: &scope,
            collaborator: &principal,
            access,
        };
        Program::commit_grant_scope_access(&ctx, &watch_ctx)?;
        Ok(ScopeAccessGrant::from_committed(CommittedGrantReceipt {
            id: grant_id,
            scope_key,
            principal,
            access,
            home_identity,
            home_incarnation,
        }))
    }

    /// Revokes a previously issued scope-access grant immediately and durably.
    pub fn revoke_scope_access(&self, grant_id: &ScopeGrantId) -> Result<(), SharedAccessDenial> {
        let cancellation = WorthQueryCancellationSource::new();
        let request_scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(OWNER_OPERATION_DEADLINE_SECS)),
            cancellation.token(),
            WorthQueryAdmissionClock::system(),
        );
        let owner = Program::authenticate_owner(
            self.runtime().runtime().installed_schema(),
            &request_scope,
        )?;
        let ctx = RemoteQueryContext {
            runtime: self.runtime(),
            owner: &owner,
            request_scope: &request_scope,
        };
        Program::commit_revoke_scope_access(&ctx, grant_id)
    }

    /// Binds a single-owner authenticated remote server backed exclusively by this Directory Home.
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
        <Program::ReadOperation as worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            ApplicationStructuredValueBinding<Value = Program::ReadCapabilityInput>,
        <Program::WatchOperation as worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            ApplicationStructuredValueBinding<Value = Program::WatchCapabilityInput>,
{
        WorthQueryRemoteServer::bind(self, bind_addr, auth_config)
    }
}

fn derive_grant_id(
    incarnation: Option<u64>,
    scope_key: &str,
    principal: &WorthQueryExternalPrincipalIdentity,
) -> Result<ScopeGrantId, SharedAccessDenial> {
    let seq = NEXT_GRANT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
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
