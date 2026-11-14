//! A nonempty operation-output record survives real Home close and cold workflow reconstruction.
use super::*;
use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request;
use worth_query_host::facade::application_discovery::{
    WorthQueryWorkflowInstanceRead, WorthQueryWorkflowObservedState,
};
use worth_query_host::facade::application_installation::{
    ApplicationBuildIdentity, WorthQueryApplicationProgramRoster,
};
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
    WorthQueryWorkflowApplicationHome,
};
type Home = WorthQueryWorkflowApplicationHome<Schema, Spec, Program>;
fn input() -> SetRetentionInput {
    SetRetentionInput {
        identity: document::retention_entry::DOCUMENT_IDENTITY.into(),
        retention_days: 7,
    }
}
fn open(path: &std::path::Path) -> Home {
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(path.to_owned()),
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .with_schema(&Schema::declaration().unwrap())
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        document::host::host_limits(WorthQueryAdmissionClock::system()),
        seed::seed,
    );
    let lease = super::super::super::checkpoint_capture::lease(256 << 20);
    let home = with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| request.open_in_advancement(&phase).unwrap(),
    )
    .unwrap();
    let spec = worth_query_host::facade::domain::WorthQueryApplicationWorkflowSpecInstallation::<
        Schema,
        Spec,
        Program,
    >::begin(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
        document::workflow::workflow_resources(),
    )
    .operation::<ControlBinding>()
    .unwrap()
    .operation::<Binding>()
    .unwrap()
    .authoring_capability::<ControlCapability, ControlOperation, SetRetentionInput>()
    .unwrap()
    .instance_start_capability::<StartCapability, ControlOperation, SetRetentionInput>()
    .unwrap()
    .advance_capability::<AdvanceCapability, ControlOperation, SetRetentionInput>()
    .unwrap()
    .finish()
    .unwrap();
    let authentication = document::workflow::install_certification_authentication_with_clock(
        home.runtime().installed_schema(),
    )
    .0;
    home.retain_workflow_spec(spec, authentication.signing_owner())
        .unwrap()
}
fn authenticate(
    home: &Home,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<Schema> {
    let adapter = admit_authentication_adapter(
        home.runtime().installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("document-retention-host").unwrap(),
            WorthQueryAuthenticationMethod::new("certification").unwrap(),
            scope.clock().clone(),
        ),
        document::operator_identity::OperatorAdapter,
    )
    .unwrap();
    document::operator_identity::block_on(adapter.authenticate((), scope)).unwrap()
}
fn close(home: Home) {
    let lease = super::super::super::checkpoint_capture::lease(256 << 20);
    home.close(&worth_execution::ExecutionRequest::leased(&lease))
        .unwrap()
        .reclaim();
}
fn advance(
    home: &Home,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> WorkflowProgressOutcome {
    let scope = document::operator_identity::request_scope();
    let principal = authenticate(home, &scope);
    home.runtime()
        .request(&principal, &scope)
        .mutate(Advance(input()))
        .without_source()
        .idempotency(&key)
        .prepare_workflow_advance(home.vocabulary(), instance)
        .unwrap()
        .execute()
}
fn read(
    home: &Home,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
) -> WorthQueryWorkflowInstanceRead {
    let scope = document::operator_identity::request_scope();
    let principal = authenticate(home, &scope);
    let request = home.runtime().request(&principal, &scope);
    let basis = request.retain_current_read().unwrap();
    request
        .at(&basis)
        .read_original_workflow::<Spec, Read, Read>(home.vocabulary(), original, input(), input())
        .unwrap()
}
#[test]
fn directory_workflow_retains_a_nonempty_operation_output_inventory_across_close() {
    let path = super::super::super::fixture::location();
    let home = open(&path);
    let mut graph = ApplicationWorkflowDefinitionBuilder::<Spec>::new(
        "output-records",
        document::workflow::definition_limits(),
    )
    .unwrap();
    let propose = graph
        .operation_binding::<ControlBinding>("propose")
        .unwrap();
    let effect = graph.operation_binding::<Binding>("preserve").unwrap();
    let done = graph.terminal("done").unwrap();
    graph
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &effect,
        )
        .control(&effect, ApplicationWorkflowControlOutcome::Completed, &done)
        .operation_input(&propose, &effect);
    let (instance, original) = {
        let scope = document::operator_identity::request_scope();
        let principal = authenticate(&home, &scope);
        let request = home.runtime().request(&principal, &scope);
        let contract = home
            .workflow_spec()
            .bind_definition(graph.finish().unwrap().validate().unwrap())
            .unwrap();
        let definition = match request
            .mutate(Control(input()))
            .without_source()
            .idempotency(&401u64)
            .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
            .unwrap()
            .execute()
        {
            WorkflowDefinitionPublicationOutcome::Published(done) => done.definition().clone(),
            other => panic!("definition: {other:?}"),
        };
        let start = request
            .mutate(Start(input()))
            .without_source()
            .idempotency(&402u64)
            .prepare_workflow_instance_start(home.vocabulary(), definition)
            .unwrap();
        let original = start.discovery_intent();
        let instance = match start.execute() {
            WorkflowInstanceStartOutcome::Started(done) => done.instance().clone(),
            other => panic!("start: {other:?}"),
        };
        assert!(matches!(
            request
                .mutate(Control(input()))
                .without_source()
                .idempotency(&403u64)
                .prepare_workflow_proposal(home.vocabulary(), instance.clone())
                .unwrap()
                .execute(),
            WorkflowProposalOutcome::Published(_)
        ));
        (instance, original)
    };
    let identity = instance.entity_id();
    let required = match advance(&home, instance.clone(), 404) {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("output need: {other:?}"),
    };
    {
        let scope = document::operator_identity::request_scope();
        let principal = authenticate(&home, &scope);
        let result = home
            .runtime()
            .request(&principal, &scope)
            .mutate(Intent(input()))
            .without_source()
            .idempotency(&405u64)
            .for_workflow_operation(home.vocabulary(), &required)
            .unwrap()
            .advanced()
            .execute_in_program(
                home.runtime(),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = result else {
            panic!("native preservation: {result:?}")
        };
        assert!(receipt
            .outputs_of::<Outputs>()
            .unwrap()
            .entity::<DocumentOutput>()
            .is_ok());
    }
    close(home);
    let home = open(&path);
    let instance = match read(&home, &original) {
        WorthQueryWorkflowInstanceRead::Live { instance, .. } => instance,
        _ => panic!("original workflow remains live"),
    };
    assert_eq!(instance.entity_id(), identity);
    assert!(
        matches!(advance(&home, instance, 406), WorkflowProgressOutcome::Completed(done) if done.node_path() == "done")
    );
    close(home);
    let home = open(&path);
    match read(&home, &original) {
        WorthQueryWorkflowInstanceRead::Historical { observation, .. } => {
            assert_eq!(
                observation.state(),
                WorthQueryWorkflowObservedState::Completed
            );
            assert_eq!(
                observation.recorded_paths(),
                &["propose", "preserve", "done"]
            );
        }
        _ => panic!("nonempty output survived final reopen"),
    }
    close(home);
}
