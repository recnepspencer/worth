//! Genuine installed workflow vocabulary at the definition-publication admission gate.
use crate::process::domain_computation::primary_graph::tests::application_attempt::erased_core_forgery::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::ComposedCapabilityTouchOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::ComposedTouchAccountCapability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityTouchOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ElevatedCapabilityTouchOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ElevatedTouchAccountCapability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
use worth_query_execution::publication_boundary::workflow_definition_publication::WorkflowDefinitionBindingDenial;
use worth_query_execution::publication_boundary::workflow_definition_publication::WorkflowDefinitionExpectedPredecessor;
use worth_query_execution::publication_boundary::workflow_definition_publication::WorkflowDefinitionPreparationDenial;
use worth_query_declaration::facade::application_program::*;
use worth_query_installation::facade::install_application_program;
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;
use worth_query_installation::facade::WorthQueryApplicationWorkflowSpecInstallation;
use worth_query_installation::facade::WorthQueryInstalledWorkflowDefinitionContract;

pub struct Feature;
impl ApplicationFeature<IdentityExecutionSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.admission-workflow.feature";
}
pub struct Program;
impl ApplicationProgramDefinition<IdentityExecutionSchema> for Program {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.admission-workflow.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<IdentityExecutionSchema, Feature>()
                .native_action::<ProgramRequiredMutationBinding>()
                .finish_action()
                .operation::<CapabilityTouchOperation>()
                .operation::<ComposedCapabilityTouchOperation>()
                .operation::<ElevatedCapabilityTouchOperation>()
                .finish(),
        ]
    }
}
pub struct Spec;
impl ApplicationWorkflowSpec for Spec {
    type Schema = IdentityExecutionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.query.test.admission-workflow.spec");
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::not_applicable()
    }
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::disabled()
    }
}

pub fn contract(
    world: &AuthorizationWorld,
) -> WorthQueryInstalledWorkflowDefinitionContract<IdentityExecutionSchema, Spec> {
    let program = ApplicationProgramAuthoring::<IdentityExecutionSchema, Program>::begin()
        .validated_program()
        .unwrap();
    let schema = world.application.installed_schema();
    let program = install_application_program(program, schema).unwrap();
    let components = ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1).unwrap();
    let resources =
        WorthQueryApplicationWorkflowResourceCeiling::new(2, 2, 16, components, 4096, 1, 1, 4096)
            .unwrap();
    let installed=WorthQueryApplicationWorkflowSpecInstallation::<IdentityExecutionSchema,Spec,Program>::begin(schema,&program,resources)
        .operation::<ProgramRequiredMutationBinding>().unwrap()
        .authoring_capability::<TouchAccountCapability,CapabilityTouchOperation,_>().unwrap()
        .instance_start_capability::<ComposedTouchAccountCapability,ComposedCapabilityTouchOperation,_>().unwrap()
        .advance_capability::<ElevatedTouchAccountCapability,ElevatedCapabilityTouchOperation,_>().unwrap()
        .finish().unwrap();
    let limits = ApplicationWorkflowDefinitionLimits::new(2, 2, 16, components, 4096).unwrap();
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<Spec>::new("terminal-only", limits).unwrap();
    let terminal = builder.terminal("complete").unwrap();
    builder.start(&terminal);
    installed
        .bind_definition(builder.finish().unwrap().validate().unwrap())
        .unwrap()
}

#[test]
fn foreign_installed_seal_is_stale_at_workflow_definition_before_work() {
    for forged in [false, true] {
        let world = installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let contract = contract(&world);
        let request = live_scope();
        let (principal, account) = party(&world, &request);
        let mut admission = admit!(
            world,
            &principal,
            &account,
            ProgramRequiredOperation,
            &request
        );
        if forged {
            let donor = world
                .application
                .installed_schema()
                .installed_operation(TouchAccountOperation::reference())
                .unwrap();
            admission.forge_installed_seal_for_test(donor.installed_view());
        }
        world.application.with_host_advancement(|phase| {
            let selected=world.selected_product(&phase.test_host_request());
            let execution=phase.execution_request_for(&world.application.runtime.product_runtime).unwrap();
            let _observation=worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
            fact_work();
            let before=reads();
            let consults=observe_consults_for_test(|_|{});
            let result=selected.prepare_workflow_publication::<TouchAccountCapability,_,_,_,Spec>(
                &execution,contract,WorkflowDefinitionExpectedPredecessor::Absent,admission);
            if forged {
                let Err(WorkflowDefinitionPreparationDenial::Projection(denial))=result else { panic!("workflow publication must check its own installed seal"); };
                assert_eq!(denial.kind(),WorthQueryOperationProjectionDenialKind::Authorization(refused(AuthorizationRefusal::StaleInstalledOperation)));
                assert_eq!(reads(),before);
                assert_eq!(fact_work().source_admission_visits,0);
                assert_eq!(consult_count_for_test(&consults),0);
                assert!(charged_prefixes_for_test(&consults).is_empty());
            } else {
                assert!(matches!(result,Err(WorkflowDefinitionPreparationDenial::Binding(WorkflowDefinitionBindingDenial::SelectedProgramUnavailable))),
                    "genuine admission reaches the separate selected-program gate");
            }
        }).unwrap();
    }
}
