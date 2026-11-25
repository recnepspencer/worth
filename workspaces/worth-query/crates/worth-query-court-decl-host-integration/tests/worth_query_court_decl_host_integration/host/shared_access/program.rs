//! `WorthQuerySharedAccessProgram` binding and Directory Home fixture helpers.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use worth_query_decl::facade::{
    application_program::{
        ApplicationNoOutputGraph, ApplicationProgramAuthoring, ApplicationProgramDefinition,
        ApplicationProgramOutputs,
    },
    application_schema::ApplicationSchema,
    authentication::WorthQueryPrincipalMappingStatus,
};
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
    },
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        SignalConditionalEvaluationBudget, WorthQueryApplicationEntityKey,
        WorthQueryApplicationEntitySeed, WorthQueryApplicationPrincipalKey,
        WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
        WorthQueryPrimaryGraphInstallationDenial,
    },
    runtime::{
        DeterminismContract, ExecutionAllocationPolicy, ExecutionBudget, ExecutionPosture,
        ExecutionRequestPolicy, WorthQueryApplicationQueryResourceProfile,
    },
    shared_access::{ScopeAccess, SharedAccessDenial},
    worth_query_shared_access, WorthQueryApplicationHome, WorthQueryApplicationHomeLocation,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::application_runtime_surface as identity_fixture;

use super::capabilities::{
    ReadProjectCapability, ReadProjectOperation, SharedPrincipalBinding, WatchProjectCapability,
    WatchProjectOperation,
};
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{
    alice_identity, bob_identity, owner_identity, GrantProjectInput, GrantProjectIntent, ProjectId,
    RevokeProjectInput, RevokeProjectIntent, WriteProjectInput, WriteProjectIntent,
    ALICE_PRINCIPAL_ID, BOB_PRINCIPAL_ID, OWNER_PRINCIPAL_ID,
};
use super::query::{ProjectQueryParams, ProjectStateRead, ProjectStateRow};
use super::schema::{
    Project, ProjectAliasField, ProjectKeyField, ProjectLiveSelf, ProjectVersionField,
    SharedProjectSchema, SharedProjectSurface,
};

static NEXT_IDEMPOTENCY_KEY: AtomicU64 = AtomicU64::new(1);

worth_query_shared_access! {
    program SharedProjectProgram for SharedProjectSchema {
        scope: ProjectId,
        payload: ProjectQueryParams,
        query: ProjectStateRead = |scope, _| ProjectStateRead(scope.0.clone()),
        row: ProjectStateRow = |value| value,
        read: ReadProjectCapability => ReadProjectOperation,
        watch: WatchProjectCapability => WatchProjectOperation,
        grant: GrantProjectIntent = |grant| GrantProjectIntent(GrantProjectInput {
            grant_id: grant.grant_id.as_str().to_owned(),
            project_key: grant.scope.0.clone(),
            grantee_issuer: grant.collaborator.issuer().to_owned(),
            grantee_subject: grant.collaborator.subject().to_owned(),
            mode: match grant.access {
                ScopeAccess::ReadOnly => "read_only",
                ScopeAccess::ReadAndWatch => "read_and_watch",
            }
            .to_owned(),
        }),
        revoke: RevokeProjectIntent = |grant_id| RevokeProjectIntent(RevokeProjectInput {
            grant_id: grant_id.as_str().to_owned(),
        }),
        owner: |schema, request_scope| authenticate_local_owner(schema, request_scope),
    }
}

pub fn authenticate_local_owner(
    schema: &WorthQueryInstalledApplicationSchema<SharedProjectSchema>,
    request_scope: &WorthQueryRequestScope,
) -> Result<WorthQueryAuthenticatedExternalPrincipal<SharedProjectSchema>, SharedAccessDenial> {
    let audience = WorthQueryAuthenticationAudience::new("daymark.local").unwrap();
    let method = WorthQueryAuthenticationMethod::new("local-process").unwrap();
    let local = WorthQueryLocalProcessPrincipal::new(
        "worth.court.shared_access.local_owner.v1",
        owner_identity(),
        audience.clone(),
        method.clone(),
        Duration::from_secs(3600),
    )
    .unwrap();
    admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            audience,
            method,
            WorthQueryAdmissionClock::system(),
        ),
        local,
    )
    .map_err(|_| SharedAccessDenial::HostUnavailable)?
    .authenticate_local(request_scope)
    .map_err(|_| SharedAccessDenial::HostUnavailable)
}

pub fn owner_write_project(
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
    project_key: &str,
    next_version: u64,
) -> u64 {
    let cancel = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + Duration::from_secs(30)),
        cancel.token(),
        WorthQueryAdmissionClock::system(),
    );
    let owner =
        authenticate_local_owner(home.runtime().runtime().installed_schema(), &scope).unwrap();
    let key = NEXT_IDEMPOTENCY_KEY.fetch_add(1, Ordering::Relaxed);
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
        .expect("owner write commits cleanly");
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { result, .. } => result,
        other => panic!("expected committed owner write, got {other:?}"),
    }
}

pub fn open_shared_home(
    location: WorthQueryApplicationHomeLocation,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    allow_initialize: bool,
) -> Result<
    WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
    worth_query_host::facade::WorthQueryApplicationHomeOpenRefused<
        SharedProjectSchema,
        SharedProjectProgram,
    >,
> {
    let declaration = SharedProjectSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<SharedProjectSchema, SharedProjectProgram>::begin()
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
        location,
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        build,
        limits,
        move |_, graph, schema| {
            assert!(
                allow_initialize,
                "reopened Directory Home must never re-run initializer"
            );
            seed_shared_home(graph, schema)
        },
    );
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .expect("bootstrap advancement executes")
}

pub fn open_custom_home<P>(
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
        |_, graph, schema| seed_shared_home(graph, schema),
    );
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap()
}

pub fn issue_alice_tok(
    cfg: &worth_query_host::facade::shared_access::RemoteAuthConfig,
    home_id: worth_foundational::facade::HomeIdentity,
    nonce_byte: u8,
) -> worth_query_host::facade::shared_access::RemoteAuthToken {
    cfg.issue_token(
        worth_query_host::facade::shared_access::RemoteTokenIssueRequest {
            key_id: "k1".to_owned(),
            home_identity: home_id,
            pinned_incarnation: None,
            principal: alice_identity(),
            ttl: Duration::from_secs(600),
            nonce: [nonce_byte; 16],
        },
    )
    .unwrap()
}

fn seed_shared_home(
    graph: &mut WorthQueryPrimaryGraphBootstrap<SharedProjectSchema>,
    schema: &WorthQueryInstalledApplicationSchema<SharedProjectSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = schema
        .principal_binding(SharedPrincipalBinding::reference())
        .unwrap();
    for (key, id, ext) in [
        ("owner-principal", OWNER_PRINCIPAL_ID, owner_identity()),
        ("alice-principal", ALICE_PRINCIPAL_ID, alice_identity()),
        ("bob-principal", BOB_PRINCIPAL_ID, bob_identity()),
    ] {
        graph
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new(key).unwrap(),
                id,
                ext,
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap();
    }
    for (entity_key, project_key) in [("project-1", "Project1"), ("project-2", "Project2")] {
        let mk_key = || WorthQueryApplicationEntityKey::new(entity_key).unwrap();
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(Project::reference(), mk_key())
                    .field(ProjectKeyField::at("ProjectKey"), project_key.to_owned())
                    .field(ProjectVersionField::at("ProjectVersion"), 0_u64)
                    .field(
                        ProjectAliasField::at("ProjectAlias"),
                        project_key.to_owned(),
                    ),
            )
            .unwrap();
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                ProjectLiveSelf::reference(),
                format!("live-self-{entity_key}"),
                mk_key(),
                mk_key(),
            ))
            .unwrap();
    }
    Ok(())
}
