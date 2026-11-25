//! Same-query-identity alternate-row selector shift court for native watch entity binding.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

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
        WorthQueryAdmissionClock, WorthQueryAuthenticatedExternalPrincipal,
        WorthQueryCancellationSource, WorthQueryRequestScope,
    },
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    domain::WorthQueryInstalledApplicationSchema,
    runtime::ExecutionAllocationPolicy,
    shared_access::{
        AuthorizationRefusalReason, RemoteWatchContext, RemoteWatchPollOutcome, ScopeAccess,
        ScopeGrantId, SharedAccessDenial, WorthQueryRemoteClient, WorthQuerySharedAccessProgram,
    },
    WorthQueryApplicationHome,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

use super::alias_shift_contract::{
    SetProjectAliasInput, SetProjectAliasIntent, WatchProjectByAliasCapability,
    WatchProjectByAliasOperation,
};
use super::capabilities::{ReadProjectCapability, ReadProjectOperation};
use super::dst::make_auth_config;
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{
    alice_identity, GrantProjectIntent, ProjectId, RevokeProjectIntent, WriteProjectInput,
    WriteProjectIntent,
};
use super::program::{authenticate_local_owner, issue_alice_tok, open_custom_home};
use super::query::{ProjectQueryParams, ProjectStateRead, ProjectStateRow};
use super::schema::{SharedProjectSchema, SharedProjectSurface};

static NEXT_SHIFT_KEY: AtomicU64 = AtomicU64::new(10_000);

/// Program whose read query uses immutable `ProjectKey` while watch capability resolves via
/// mutable `ProjectAlias` — exercising native admitted resource `EntityId` comparison.
pub struct AliasWatchProgram;

impl ApplicationProgramDefinition<SharedProjectSchema> for AliasWatchProgram {
    type Contributions = (SharedProjectSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.court.shared_access.alias_watch.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        SharedProjectProgram::feature_specs()
    }
}

impl WorthQuerySharedAccessProgram<SharedProjectSchema> for AliasWatchProgram {
    type ScopeId = ProjectId;
    type QueryPayload = ProjectQueryParams;
    type Row = ProjectStateRow;
    type GrantMutationIntent = GrantProjectIntent;
    type RevokeMutationIntent = RevokeProjectIntent;
    type QueryIntent = ProjectStateRead;
    type ReadCapability = ReadProjectCapability;
    type ReadOperation = ReadProjectOperation;
    type ReadCapabilityInput = String;
    type WatchCapability = WatchProjectByAliasCapability;
    type WatchOperation = WatchProjectByAliasOperation;
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
    fn watch_capability(
    ) -> ApplicationCapabilityRef<SharedProjectSchema, WatchProjectByAliasCapability> {
        WatchProjectByAliasCapability::reference()
    }
    fn watch_operation(
    ) -> ApplicationOperationRef<SharedProjectSchema, WatchProjectByAliasOperation, String> {
        WatchProjectByAliasOperation::reference()
    }
    fn build_watch_capability_input(scope: &ProjectId) -> String {
        scope.0.clone()
    }
    fn project_row(value: ProjectStateRow) -> ProjectStateRow {
        value
    }
}

fn owner_set_alias(
    home: &WorthQueryApplicationHome<SharedProjectSchema, AliasWatchProgram>,
    project_key: &str,
    next_alias: &str,
) {
    let cancel = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(30)),
        cancel.token(),
        WorthQueryAdmissionClock::system(),
    );
    let owner =
        authenticate_local_owner(home.runtime().runtime().installed_schema(), &scope).unwrap();
    let key = NEXT_SHIFT_KEY.fetch_add(1, Ordering::Relaxed);
    let outcome = home
        .runtime()
        .request(&owner, &scope)
        .advanced()
        .mutate(SetProjectAliasIntent(SetProjectAliasInput {
            project_key: project_key.to_owned(),
            next_alias: next_alias.to_owned(),
        }))
        .without_source()
        .idempotency(&key)
        .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        .expect("alias mutation commits");
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

fn owner_write_version(
    home: &WorthQueryApplicationHome<SharedProjectSchema, AliasWatchProgram>,
    project_key: &str,
    next_version: u64,
) {
    let cancel = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(30)),
        cancel.token(),
        WorthQueryAdmissionClock::system(),
    );
    let owner =
        authenticate_local_owner(home.runtime().runtime().installed_schema(), &scope).unwrap();
    let key = NEXT_SHIFT_KEY.fetch_add(1, Ordering::Relaxed);
    let outcome = home
        .runtime()
        .request(&owner, &scope)
        .advanced()
        .mutate(WriteProjectIntent(WriteProjectInput {
            project_key: project_key.to_owned(),
            next_version,
        }))
        .without_source()
        .idempotency(&key)
        .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        .expect("version write commits");
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

#[test]
fn adversarial_same_query_identity_alternate_row_selector_shift() {
    let dir = worth_query_testkit_host::facade::host::application_home::location()
        .with_extension("shared-r3-alias-shift");
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let home = open_custom_home::<AliasWatchProgram>(&dir, execution);
    home.grant_scope_access(
        ProjectId::project1(),
        alice_identity(),
        ScopeAccess::read_and_watch(),
    )
    .unwrap();
    let p2_grant = home
        .grant_scope_access(
            ProjectId::project2(),
            alice_identity(),
            ScopeAccess::read_and_watch(),
        )
        .unwrap();

    let cfg = make_auth_config();
    let tok = issue_alice_tok(&cfg, home.home_identity().unwrap(), 0x71);
    let mut server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), cfg)
        .unwrap();
    let mut alice = WorthQueryRemoteClient::connect(server.local_addr(), tok).unwrap();

    assert_poll_rejects_shifted_alias(&home, &mut server, &mut alice, p2_grant.id());
    assert_open_rejects_shifted_alias(&home, &mut server, &mut alice);

    server.close();
    home.close(&execution).unwrap().reclaim();
    let _ = std::fs::remove_dir_all(&dir);
}

fn assert_poll_rejects_shifted_alias(
    home: &WorthQueryApplicationHome<SharedProjectSchema, AliasWatchProgram>,
    server: &mut worth_query_host::facade::shared_access::WorthQueryRemoteServer<
        '_,
        SharedProjectSchema,
        AliasWatchProgram,
    >,
    alice: &mut WorthQueryRemoteClient,
    p2_grant_id: &ScopeGrantId,
) {
    alice
        .send_watch_open::<SharedProjectSchema, AliasWatchProgram>(
            &ProjectId::project2(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    let mut watch = alice
        .recv_watch_open::<SharedProjectSchema, AliasWatchProgram>()
        .expect("initial aligned alias watch opens");
    assert_eq!(server.active_watch_count(), 1);
    watch.send_poll().unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    assert_eq!(watch.recv_poll(), Ok(RemoteWatchPollOutcome::Pending));

    // Shift ProjectAlias="Project2" onto Project1 while Project2 keeps ProjectKey="Project2".
    owner_set_alias(home, "Project2", "Detached2");
    owner_set_alias(home, "Project1", "Project2");
    home.revoke_scope_access(p2_grant_id).unwrap();
    owner_write_version(home, "Project2", 5);

    // WatchProjectByAliasCapability admits Entity(Project1) != bound Entity(Project2) -> StaleScope.
    watch.send_poll().unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    assert_eq!(
        watch.recv_poll(),
        Err(SharedAccessDenial::AuthorizationRefused(
            AuthorizationRefusalReason::StaleScope
        ))
    );
    assert_eq!(server.active_watch_count(), 0);
}

fn assert_open_rejects_shifted_alias(
    home: &WorthQueryApplicationHome<SharedProjectSchema, AliasWatchProgram>,
    server: &mut worth_query_host::facade::shared_access::WorthQueryRemoteServer<
        '_,
        SharedProjectSchema,
        AliasWatchProgram,
    >,
    alice: &mut WorthQueryRemoteClient,
) {
    home.grant_scope_access(
        ProjectId::project2(),
        alice_identity(),
        ScopeAccess::read_only(),
    )
    .unwrap();
    alice
        .send_watch_open::<SharedProjectSchema, AliasWatchProgram>(
            &ProjectId::project2(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    assert_eq!(
        alice
            .recv_watch_open::<SharedProjectSchema, AliasWatchProgram>()
            .err(),
        Some(SharedAccessDenial::AuthorizationRefused(
            AuthorizationRefusalReason::CapabilityDenied
        ))
    );
    assert_eq!(server.active_watch_count(), 0);
}
