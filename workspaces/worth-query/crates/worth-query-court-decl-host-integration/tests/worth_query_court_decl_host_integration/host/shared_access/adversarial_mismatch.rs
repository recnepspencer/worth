//! Mismatched-scope watch authority & capped-encoder watch-open rollback boundary courts.

use std::time::Duration;

use worth_query_decl::facade::{
    application_capability::ApplicationCapabilityRef,
    application_program::{
        ApplicationFeatureSpec, ApplicationNoOutputGraph, ApplicationProgramAuthoring,
        ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramOutputs,
        ApplicationRuleLeaf,
    },
    application_schema::{ApplicationOperationRef, ApplicationSchema},
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
    },
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::SignalConditionalEvaluationBudget,
    runtime::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
        WorthQueryApplicationQueryResourceProfile,
    },
    shared_access::{
        AuthorizationRefusalReason, BudgetRefusalReason, RemoteBinaryCodec, RemoteByteWriter,
        RemoteServerLimits, RemoteTokenIssueRequest, RemoteWatchContext, ScopeAccess, ScopeGrantId,
        SharedAccessDenial, WorthQueryRemoteClient, WorthQuerySharedAccessProgram,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::application_runtime_surface as identity_fixture;
use worth_query_testkit_host::facade::host::checkpoint_capture;

use super::capabilities::{
    ReadProjectCapability, ReadProjectOperation, WatchProjectCapability, WatchProjectOperation,
};
use super::dst::make_auth_config;
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{alice_identity, GrantProjectIntent, ProjectId, RevokeProjectIntent};
use super::program::{authenticate_local_owner, seed_shared_home_public};
use super::query::{ProjectQueryParams, ProjectStateRead, ProjectStateRow};
use super::schema::{SharedProjectSchema, SharedProjectSurface};

/// Adversarial program whose watch capability input always names `"Project1"` even for `"Project2"`.
pub struct MismatchedWatchScopeProgram;

impl ApplicationProgramDefinition<SharedProjectSchema> for MismatchedWatchScopeProgram {
    type Contributions = (SharedProjectSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.court.shared_access.mismatched_watch.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        SharedProjectProgram::feature_specs()
    }
}

impl WorthQuerySharedAccessProgram<SharedProjectSchema> for MismatchedWatchScopeProgram {
    type ScopeId = ProjectId;
    type QueryPayload = ProjectQueryParams;
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
        payload: &ProjectQueryParams,
    ) -> Result<ProjectStateRead, SharedAccessDenial> {
        SharedProjectProgram::build_query_intent(scope, payload)
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
    fn build_watch_capability_input(_scope: &ProjectId) -> String {
        "Project1".to_owned()
    }
    fn project_row(value: ProjectStateRow) -> ProjectStateRow {
        value
    }
}

/// Row codec that intentionally emits 512 bytes to exercise capped-encoder rollback on WatchOpen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OversizedRow(pub ProjectStateRow);

impl RemoteBinaryCodec for OversizedRow {
    fn encode_remote(&self) -> Vec<u8> {
        vec![0xAB_u8; 512]
    }
    fn encode_remote_into(&self, writer: &mut RemoteByteWriter) -> Result<(), SharedAccessDenial> {
        writer.charge(512)?;
        writer.write_fixed(&[0xAB_u8; 512])
    }
    fn decode_remote(_: &[u8]) -> Result<Self, SharedAccessDenial> {
        Ok(Self(ProjectStateRow {
            project_key: String::new(),
            version: 0,
        }))
    }
}

pub struct OversizedRowProgram;

impl ApplicationProgramDefinition<SharedProjectSchema> for OversizedRowProgram {
    type Contributions = (SharedProjectSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.court.shared_access.oversized_row.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        SharedProjectProgram::feature_specs()
    }
}

impl WorthQuerySharedAccessProgram<SharedProjectSchema> for OversizedRowProgram {
    type ScopeId = ProjectId;
    type QueryPayload = ProjectQueryParams;
    type Row = OversizedRow;
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
        payload: &ProjectQueryParams,
    ) -> Result<ProjectStateRead, SharedAccessDenial> {
        SharedProjectProgram::build_query_intent(scope, payload)
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
    fn project_row(value: ProjectStateRow) -> OversizedRow {
        OversizedRow(value)
    }
}

fn open_custom_home<P>(
    dir: &std::path::Path,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryApplicationHome<SharedProjectSchema, P>
where
    P: ApplicationProgramDefinition<
            SharedProjectSchema,
            Contributions = (SharedProjectSurface,),
            Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>,
        > + 'static,
{
    let declaration = SharedProjectSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<SharedProjectSchema, P>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let limits = WorthQueryApplicationLimits::new(
        identity_fixture::world_resources().with_execution_policy(ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
        )),
        WorthQueryApplicationQueryResourceProfile::default(),
        SignalConditionalEvaluationBudget::development(),
    );
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(dir.to_path_buf()),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        build,
        limits,
        |_, graph, schema| seed_shared_home_public(graph, schema),
    );
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap()
}

fn issue_alice_tok(
    cfg: &worth_query_host::facade::shared_access::RemoteAuthConfig,
    home_id: worth_foundational::facade::HomeIdentity,
    nonce_byte: u8,
) -> worth_query_host::facade::shared_access::RemoteAuthToken {
    cfg.issue_token(RemoteTokenIssueRequest {
        key_id: "k1".to_owned(),
        home_identity: home_id,
        pinned_incarnation: None,
        principal: alice_identity(),
        ttl: Duration::from_secs(600),
        nonce: [nonce_byte; 16],
    })
    .unwrap()
}

#[test]
fn adversarial_mismatched_watch_scope_and_capped_response_rollback() {
    let base = worth_query_testkit_host::facade::host::application_home::location();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    assert_mismatched_watch_scope_denied(&base.with_extension("shared-r2-mismatch"), execution);
    assert_oversized_watch_open_rolls_back(&base.with_extension("shared-r2-oversized"), execution);
}

fn assert_mismatched_watch_scope_denied(
    dir: &std::path::Path,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) {
    let home = open_custom_home::<MismatchedWatchScopeProgram>(dir, execution);
    home.grant_scope_access(
        ProjectId::project1(),
        alice_identity(),
        ScopeAccess::read_and_watch(),
    )
    .unwrap();
    home.grant_scope_access(
        ProjectId::project2(),
        alice_identity(),
        ScopeAccess::read_only(),
    )
    .unwrap();
    let cfg = make_auth_config();
    let tok = issue_alice_tok(&cfg, home.home_identity().unwrap(), 0x66);
    let mut server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), cfg)
        .unwrap();
    let mut alice = WorthQueryRemoteClient::connect(server.local_addr(), tok).unwrap();
    alice
        .send_watch_open::<SharedProjectSchema, MismatchedWatchScopeProgram>(
            &ProjectId::project2(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    assert_eq!(
        alice
            .recv_watch_open::<SharedProjectSchema, MismatchedWatchScopeProgram>()
            .err(),
        Some(SharedAccessDenial::AuthorizationRefused(
            AuthorizationRefusalReason::CapabilityDenied
        )),
    );
    assert_eq!(server.active_watch_count(), 0);
    server.close();
    home.close(&execution).unwrap().reclaim();
    let _ = std::fs::remove_dir_all(dir);
}

fn assert_oversized_watch_open_rolls_back(
    dir: &std::path::Path,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) {
    let home = open_custom_home::<OversizedRowProgram>(dir, execution);
    home.grant_scope_access(
        ProjectId::project1(),
        alice_identity(),
        ScopeAccess::read_and_watch(),
    )
    .unwrap();
    let limits = RemoteServerLimits::default()
        .with_max_frame_bytes(300)
        .unwrap();
    let cfg = make_auth_config().with_limits(limits);
    let tok = issue_alice_tok(&cfg, home.home_identity().unwrap(), 0x67);
    let mut server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), cfg)
        .unwrap();
    let mut alice = WorthQueryRemoteClient::connect(server.local_addr(), tok).unwrap();
    alice
        .send_watch_open::<SharedProjectSchema, OversizedRowProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    let res = alice.recv_watch_open::<SharedProjectSchema, OversizedRowProgram>();
    assert!(
        matches!(
            res,
            Err(SharedAccessDenial::BudgetRefused(
                BudgetRefusalReason::ResponseTooLarge { maximum: 300, .. }
            ))
        ),
        "oversized watch-open MUST fail closed with ResponseTooLarge: {:?}",
        res.err()
    );
    assert_eq!(server.active_watch_count(), 0);
    server.close();
    home.close(&execution).unwrap().reclaim();
    let _ = std::fs::remove_dir_all(dir);
}
