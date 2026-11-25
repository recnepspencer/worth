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

/// Query payload that streams bytes into the single capped request writer and records
/// observed length, remaining bytes, and backing `Vec` capacity.
#[derive(Clone, Debug)]
pub struct BoundedQueryPayload {
    pub body_len: usize,
    pub fill_exact_remaining: bool,
    pub extra_byte_after_fill: bool,
    pub observed_remaining: Arc<AtomicUsize>,
    pub observed_len: Arc<AtomicUsize>,
    pub observed_capacity: Arc<AtomicUsize>,
}

impl BoundedQueryPayload {
    fn fixed(body_len: usize, observed_remaining: Arc<AtomicUsize>) -> Self {
        Self {
            body_len,
            fill_exact_remaining: false,
            extra_byte_after_fill: false,
            observed_remaining,
            observed_len: Arc::new(AtomicUsize::new(0)),
            observed_capacity: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn fill_remaining(extra_byte_after_fill: bool) -> Self {
        Self {
            body_len: 0,
            fill_exact_remaining: true,
            extra_byte_after_fill,
            observed_remaining: Arc::new(AtomicUsize::new(0)),
            observed_len: Arc::new(AtomicUsize::new(0)),
            observed_capacity: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl RemoteBinaryCodec for BoundedQueryPayload {
    fn encode_remote_bounded(
        &self,
        writer: &mut RemoteByteWriter,
    ) -> Result<(), SharedAccessDenial> {
        self.observed_remaining
            .store(writer.remaining(), Ordering::SeqCst);
        let mut left = if self.fill_exact_remaining {
            writer.remaining()
        } else {
            self.body_len
        };
        let chunk = [0xCD_u8; 16];
        while left > 0 {
            let take = left.min(chunk.len());
            writer.write_fixed(&chunk[..take])?;
            left -= take;
        }
        self.observed_len.store(writer.len(), Ordering::SeqCst);
        self.observed_capacity
            .store(writer.capacity(), Ordering::SeqCst);
        if self.extra_byte_after_fill {
            let err = writer.write_u8(0xEE);
            self.observed_capacity
                .store(writer.capacity(), Ordering::SeqCst);
            err?;
        }
        Ok(())
    }

    fn decode_remote(bytes: &[u8]) -> Result<Self, SharedAccessDenial> {
        Ok(Self::fixed(bytes.len(), Arc::new(AtomicUsize::new(0))))
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
    let tok_clone = tok.clone();
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
    assert_exact_257_byte_capacity_ceiling(&mut server, &tok_clone);

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
    let oversized_with_envelope = BoundedQueryPayload::fixed(200, Arc::clone(&observed));
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

    let fits = BoundedQueryPayload::fixed(32, Arc::new(AtomicUsize::new(0)));
    alice
        .send_query::<SharedProjectSchema, EnvelopeBudgetProgram>(&ProjectId::project1(), &fits)
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    let rows = alice
        .recv_query::<SharedProjectSchema, EnvelopeBudgetProgram>()
        .unwrap();
    assert_eq!(rows[0].project_key, "Project1");
}

/// Verifies `max_bytes = 257` (just above initial 256 capacity): writing 257 bytes succeeds
/// with `capacity() <= 257` (never doubling to 512) and the 258th byte fails closed.
fn assert_exact_257_byte_capacity_ceiling(
    server: &mut worth_query_host::facade::shared_access::WorthQueryRemoteServer<
        '_,
        SharedProjectSchema,
        EnvelopeBudgetProgram,
    >,
    tok: &worth_query_host::facade::shared_access::RemoteAuthToken,
) {
    let limits = RemoteClientLimits::default()
        .with_max_frame_bytes(257)
        .unwrap();
    let mut client =
        WorthQueryRemoteClient::connect_with_limits(server.local_addr(), tok.clone(), limits)
            .unwrap();

    let exact_fill = BoundedQueryPayload::fill_remaining(false);
    client
        .send_query::<SharedProjectSchema, EnvelopeBudgetProgram>(
            &ProjectId::project1(),
            &exact_fill,
        )
        .unwrap();
    assert_eq!(exact_fill.observed_len.load(Ordering::SeqCst), 257);
    assert!(
        exact_fill.observed_capacity.load(Ordering::SeqCst) <= 257,
        "RemoteByteWriter must never double capacity past 257 (got {})",
        exact_fill.observed_capacity.load(Ordering::SeqCst)
    );
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    assert_eq!(
        client
            .recv_query::<SharedProjectSchema, EnvelopeBudgetProgram>()
            .unwrap()[0]
            .project_key,
        "Project1"
    );

    let overflow_258 = BoundedQueryPayload::fill_remaining(true);
    assert_eq!(
        client.send_query::<SharedProjectSchema, EnvelopeBudgetProgram>(
            &ProjectId::project1(),
            &overflow_258,
        ),
        Err(SharedAccessDenial::BudgetRefused(
            BudgetRefusalReason::RequestLimitExceeded
        ))
    );
    assert!(
        overflow_258.observed_capacity.load(Ordering::SeqCst) <= 257,
        "258th-byte refusal must not grow capacity past 257 (got {})",
        overflow_258.observed_capacity.load(Ordering::SeqCst)
    );
}
