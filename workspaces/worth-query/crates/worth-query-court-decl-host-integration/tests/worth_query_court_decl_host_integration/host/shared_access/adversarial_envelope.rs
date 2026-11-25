//! Full-envelope request framing budget court verifying single-writer pre-allocation caps.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use worth_query_decl::facade::{
    application_capability::ApplicationCapabilityRef,
    application_program::{
        ApplicationFeatureSpec, ApplicationNoOutputGraph, ApplicationProgramDefinition,
        ApplicationProgramIdentity, ApplicationProgramOutputs, ApplicationRuleLeaf,
    },
    application_schema::ApplicationOperationRef,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    domain::WorthQueryInstalledApplicationSchema,
    shared_access::{
        BudgetRefusalReason, RemoteBinaryCodec, RemoteByteWriter, RemoteClientLimits,
        RemoteWatchContext, ScopeAccess, ScopeGrantId, SharedAccessDenial, WorthQueryRemoteClient,
        WorthQuerySharedAccessProgram,
    },
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

use super::capabilities::{
    ReadProjectCapability, ReadProjectOperation, WatchProjectCapability, WatchProjectOperation,
};
use super::dst::make_auth_config;
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{alice_identity, GrantProjectIntent, ProjectId, RevokeProjectIntent};
use super::program::{authenticate_local_owner, issue_alice_tok, open_custom_home};
use super::query::{ProjectStateRead, ProjectStateRow};
use super::schema::{SharedProjectSchema, SharedProjectSurface};

/// Query payload that streams `body_len` bytes into the single capped request writer
/// and records the remaining envelope capacity visible to `encode_remote_bounded`.
#[derive(Clone, Debug)]
pub struct BoundedQueryPayload {
    pub body_len: usize,
    pub observed_remaining: Arc<AtomicUsize>,
}

impl RemoteBinaryCodec for BoundedQueryPayload {
    fn encode_remote_bounded(
        &self,
        writer: &mut RemoteByteWriter,
    ) -> Result<(), SharedAccessDenial> {
        self.observed_remaining
            .store(writer.remaining(), Ordering::SeqCst);
        let chunk = [0xCD_u8; 16];
        let mut left = self.body_len;
        while left > 0 {
            let take = left.min(chunk.len());
            writer.write_fixed(&chunk[..take])?;
            left -= take;
        }
        Ok(())
    }

    fn decode_remote(bytes: &[u8]) -> Result<Self, SharedAccessDenial> {
        Ok(Self {
            body_len: bytes.len(),
            observed_remaining: Arc::new(AtomicUsize::new(0)),
        })
    }
}

pub struct EnvelopeBudgetProgram;

impl ApplicationProgramDefinition<SharedProjectSchema> for EnvelopeBudgetProgram {
    type Contributions = (SharedProjectSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.court.shared_access.envelope_budget.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        SharedProjectProgram::feature_specs()
    }
}

impl WorthQuerySharedAccessProgram<SharedProjectSchema> for EnvelopeBudgetProgram {
    type ScopeId = ProjectId;
    type QueryPayload = BoundedQueryPayload;
    type Row = ProjectStateRow;
    type GrantMutationIntent = GrantProjectIntent;
    type RevokeMutationIntent = RevokeProjectIntent;
    type QueryIntent = ProjectStateRead;
    type ReadCapability = ReadProjectCapability;
    type ReadOperation = ReadProjectOperation;
    type ReadCapabilityInput = String;
    type WatchCapability = WatchProjectCapability;
    type WatchOperation = WatchProjectOperation;
    type WatchCapabilityInput = String;

    fn scope_key(scope: &ProjectId) -> String {
        SharedProjectProgram::scope_key(scope)
    }
    fn parse_scope_key(key: &str) -> Result<ProjectId, SharedAccessDenial> {
        SharedProjectProgram::parse_scope_key(key)
    }
    fn authenticate_owner(
        schema: &WorthQueryInstalledApplicationSchema<SharedProjectSchema>,
        request_scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryAuthenticatedExternalPrincipal<SharedProjectSchema>, SharedAccessDenial>
    {
        authenticate_local_owner(schema, request_scope)
    }
    fn build_grant_mutation_intent(
        grant: &RemoteWatchContext<'_, ProjectId>,
    ) -> Result<GrantProjectIntent, SharedAccessDenial> {
        SharedProjectProgram::build_grant_mutation_intent(grant)
    }
    fn grant_idempotency_key(grant_id: &ScopeGrantId, sequence: u64) -> u64 {
        SharedProjectProgram::grant_idempotency_key(grant_id, sequence)
    }
    fn build_revoke_mutation_intent(
        grant_id: &ScopeGrantId,
    ) -> Result<RevokeProjectIntent, SharedAccessDenial> {
        SharedProjectProgram::build_revoke_mutation_intent(grant_id)
    }
    fn revoke_idempotency_key(grant_id: &ScopeGrantId, sequence: u64) -> u64 {
        SharedProjectProgram::revoke_idempotency_key(grant_id, sequence)
    }
    fn build_query_intent(
        scope: &ProjectId,
        _: &BoundedQueryPayload,
    ) -> Result<ProjectStateRead, SharedAccessDenial> {
        Ok(ProjectStateRead(scope.0.clone()))
    }
    fn read_capability() -> ApplicationCapabilityRef<SharedProjectSchema, ReadProjectCapability> {
        ReadProjectCapability::reference()
    }
    fn read_operation() -> ApplicationOperationRef<SharedProjectSchema, ReadProjectOperation, String>
    {
        ReadProjectOperation::reference()
    }
    fn build_read_capability_input(scope: &ProjectId) -> String {
        scope.0.clone()
    }
    fn watch_capability() -> ApplicationCapabilityRef<SharedProjectSchema, WatchProjectCapability> {
        WatchProjectCapability::reference()
    }
    fn watch_operation(
    ) -> ApplicationOperationRef<SharedProjectSchema, WatchProjectOperation, String> {
        WatchProjectOperation::reference()
    }
    fn build_watch_capability_input(scope: &ProjectId) -> String {
        scope.0.clone()
    }
    fn project_row(value: ProjectStateRow) -> ProjectStateRow {
        value
    }
}

/// Verifies `query.len() <= max_frame_bytes` with `envelope_overhead + query.len() > max_frame_bytes`
/// refuses cleanly with `BudgetRefused(RequestLimitExceeded)` through ONE capped writer.
#[test]
fn adversarial_request_envelope_enforces_single_writer_frame_cap() {
    let dir = worth_query_testkit_host::facade::host::application_home::location()
        .with_extension("shared-r4-envelope");
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let home = open_custom_home::<EnvelopeBudgetProgram>(&dir, execution);
    home.grant_scope_access(
        ProjectId::project1(),
        alice_identity(),
        ScopeAccess::read_and_watch(),
    )
    .unwrap();
    let cfg = make_auth_config();
    let tok = issue_alice_tok(&cfg, home.home_identity().unwrap(), 0x79);
    let mut server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), cfg)
        .unwrap();

    let max_frame_bytes: u32 = 256;
    let client_limits = RemoteClientLimits::default()
        .with_max_frame_bytes(max_frame_bytes)
        .unwrap();
    let mut alice =
        WorthQueryRemoteClient::connect_with_limits(server.local_addr(), tok, client_limits)
            .unwrap();

    assert_envelope_budget_refuses_and_allows(&mut server, &mut alice, max_frame_bytes);

    server.close();
    home.close(&execution).unwrap().reclaim();
    let _ = std::fs::remove_dir_all(&dir);
}

fn assert_envelope_budget_refuses_and_allows(
    server: &mut worth_query_host::facade::shared_access::WorthQueryRemoteServer<
        '_,
        SharedProjectSchema,
        EnvelopeBudgetProgram,
    >,
    alice: &mut WorthQueryRemoteClient,
    max_frame_bytes: u32,
) {
    // 200 <= 256 (fits by itself), but 12B frame header + token + scope key + 4B len + 200B > 256B.
    let observed = Arc::new(AtomicUsize::new(usize::MAX));
    let oversized_with_envelope = BoundedQueryPayload {
        body_len: 200,
        observed_remaining: Arc::clone(&observed),
    };
    assert!(oversized_with_envelope.body_len <= max_frame_bytes as usize);
    assert_eq!(
        alice.send_query::<SharedProjectSchema, EnvelopeBudgetProgram>(
            &ProjectId::project1(),
            &oversized_with_envelope,
        ),
        Err(SharedAccessDenial::BudgetRefused(
            BudgetRefusalReason::RequestLimitExceeded
        ))
    );
    let remaining = observed.load(Ordering::SeqCst);
    assert!(
        remaining > 0 && remaining < oversized_with_envelope.body_len,
        "single request writer must deduct envelope overhead before query encoding (remaining={remaining})"
    );
    assert_eq!(
        alice.send_watch_open::<SharedProjectSchema, EnvelopeBudgetProgram>(
            &ProjectId::project1(),
            &oversized_with_envelope,
        ),
        Err(SharedAccessDenial::BudgetRefused(
            BudgetRefusalReason::RequestLimitExceeded
        ))
    );
    assert_eq!(server.active_watch_count(), 0);

    let fits = BoundedQueryPayload {
        body_len: 32,
        observed_remaining: Arc::new(AtomicUsize::new(0)),
    };
    alice
        .send_query::<SharedProjectSchema, EnvelopeBudgetProgram>(&ProjectId::project1(), &fits)
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    let rows = alice
        .recv_query::<SharedProjectSchema, EnvelopeBudgetProgram>()
        .unwrap();
    assert_eq!(rows[0].project_key, "Project1");
}
