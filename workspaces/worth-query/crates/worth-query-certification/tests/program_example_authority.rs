#[path = "../examples/product_workflow_support/mod.rs"]
pub mod product_workflow_support;

use product_workflow_support::adapters::ClockSource;
use product_workflow_support::application::{example_limits, seed_graph};
use product_workflow_support::application_entry::AmendTemporalBinding;
use product_workflow_support::conditional_contribution::TemporalContributionConfiguration;
use product_workflow_support::program::{TemporalExampleFeature, TemporalExampleProgram};
use product_workflow_support::schema::{
    AmendTemporal, AmendTemporalInput, ExecuteTemporal, IntentIdentityField,
    RevokeTemporalPrincipal, TemporalHostSchema, TemporalPrincipalBinding,
};
use product_workflow_support::{principal, read_input, AmendTemporalIntent, ExampleApplication};
use worth_query_host::facade::{
    application_discovery::WorthQueryApplicationCallablePosture,
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial,
    },
    application_installation::{self, WorthQueryInMemoryApplicationDenial},
    declaration::application_operation::ApplicationMutationBinding,
    declaration::application_program::{
        ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
        ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramIdentity,
    },
    primary_graph::{
        HandlerResult, WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationIdempotencyBinding, WorthQueryPrincipalResolutionMode,
    },
    product::WorthQueryAdmittedChange,
};

struct MissingConditionalActionProgram;
struct ConflictingConditionalClientFeature;
struct ConflictingConditionalClientProgram;

impl ApplicationFeature<TemporalHostSchema> for ConflictingConditionalClientFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.query.example.conflicting-conditional-client.v1";
}

impl ApplicationProgramDefinition<TemporalHostSchema> for ConflictingConditionalClientProgram {
    type Contributions =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Contributions;
    type Outputs =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Outputs;
    type Rules =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Rules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.example.conflicting-conditional-client.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = TemporalExampleProgram::feature_specs();
        specs.push(
            ApplicationFeatureSpec::root::<TemporalHostSchema, ConflictingConditionalClientFeature>()
                .operation::<ExecuteTemporal>()
                .finish(),
        );
        specs
    }
}

impl ApplicationProgramDefinition<TemporalHostSchema> for MissingConditionalActionProgram {
    type Contributions =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Contributions;
    type Outputs =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Outputs;
    type Rules =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Rules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.example.missing-conditional-action.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<TemporalHostSchema, TemporalExampleFeature>()
                .mutation::<AmendTemporalBinding>()
                .operation::<RevokeTemporalPrincipal>()
                .finish(),
        ]
    }
}

#[test]
fn program_example_denies_plain_commit_and_conditional_client_admission() {
    let mut application = ExampleApplication::publish("blocked");
    assert!(application
        .runtime
        .admit_program_operation::<ExecuteTemporal>()
        .is_err());

    let scope = product_workflow_support::adapters::request_scope();
    let principal = principal(&application, &scope);
    let branch = application.runtime.current_world();
    let predecessor = read_input(&application, branch, &principal, &scope);
    let intent = amendment("must-not-publish", 2);
    let idempotency_key = 0x71_u64;
    let callable = application
        .runtime
        .discovery()
        .mutations()
        .find(|description| {
            description.declaration().binding_identity().as_str() == AmendTemporalBinding::IDENTITY
        })
        .expect("the installed mutation is discoverable");
    assert_eq!(
        callable.availability(),
        WorthQueryApplicationCallablePosture::InstalledRequestBinding
    );
    application
        .runtime
        .request(&principal, &scope)
        .mutate(intent.clone())
        .assess_current_authorization()
        .expect("fresh current authorization is observable without execution authority");
    assert_eq!(
        read_input(&application, branch, &principal, &scope),
        predecessor,
        "callability and authorization observations cannot execute the action"
    );
    let denial = application
        .runtime
        .request(&principal, &scope)
        .mutate(intent.clone())
        .without_source()
        .idempotency(&idempotency_key)
        .execute()
        .expect_err("the plain entry must deny a program-owned action");
    assert!(matches!(
        denial,
        WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired
    ));
    assert_eq!(
        read_input(&application, branch, &principal, &scope),
        predecessor
    );

    let selected = application
        .runtime
        .on_branch(branch)
        .select()
        .expect("the published branch remains selectable");
    let principal_binding = application
        .runtime
        .installed_schema()
        .principal_binding(TemporalPrincipalBinding::reference())
        .expect("the temporal principal binding must be installed");
    let resolved_principal = selected
        .resolve_authenticated_principal(
            &principal_binding,
            &principal,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the external principal must resolve through the installed binding");
    let entity = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            intent.identity.clone(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the temporal intent must resolve");
    let operation = application
        .runtime
        .installed_schema()
        .installed_operation(AmendTemporal::reference())
        .expect("the temporal amendment must be installed");
    let admission = selected
        .authorize_operation(
            &resolved_principal,
            &entity,
            &operation,
            Default::default(),
            &scope,
        )
        .expect("the low-level public chain may prepare the installed operation");
    let HandlerResult::Completed(completed) = application
        .runtime
        .execute_mutation_handler::<AmendTemporalBinding>(
            &intent.amendment,
            &idempotency_key,
            resolved_principal.principal_identity(),
            admission,
        )
        .expect("the admitted mutation handler must complete")
    else {
        panic!("the admitted mutation handler must produce a candidate");
    };
    let (program, _) = completed.into_parts();
    let change = WorthQueryAdmittedChange::new(
        program,
        WorthQueryApplicationIdempotencyBinding::new(
            AmendTemporalBinding::idempotency_key_identity(&idempotency_key),
            AmendTemporalBinding::input_identity(&intent.amendment),
        ),
    );
    let outcome = application
        .runtime
        .on_branch(branch)
        .transaction()
        .apply(change)
        .commit()
        .expect("the prepared change belongs to this branch");
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("the low-level commit door must deny a program-owned action");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    );
    assert_eq!(
        read_input(&application, branch, &principal, &scope),
        predecessor,
        "both denied doors must leave the branch unchanged"
    );

    application
        .runtime
        .close_conditional_runtime()
        .expect("the denied program closes its conditional resources");
}

#[test]
fn installed_conditional_requires_its_declared_program_action() {
    let (clock_source, _) = ClockSource::due();
    let program =
        ApplicationProgramAuthoring::<TemporalHostSchema, MissingConditionalActionProgram>::begin()
            .validated_program()
            .expect("the deliberately incomplete program is syntactically valid");
    let denial = application_installation::in_memory_program(
        program,
        TemporalHostSchema::declaration().expect("the temporal schema is valid"),
        (TemporalContributionConfiguration { clock_source },),
        example_limits(),
        |graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .expect("the temporal principal binding is installed");
            seed_graph(graph, &principal, "blocked");
            Ok(())
        },
    )
    .err()
    .expect("installation must reject the missing conditional action");
    assert!(matches!(
        denial,
        WorthQueryInMemoryApplicationDenial::ConditionalProgramMismatch
    ));
}

#[test]
fn installed_conditional_rejects_a_client_action_for_its_operation() {
    let (clock_source, _) = ClockSource::due();
    let program = ApplicationProgramAuthoring::<
        TemporalHostSchema,
        ConflictingConditionalClientProgram,
    >::begin()
    .validated_program()
    .expect("distinct features make both action postures syntactically valid");
    let denial = application_installation::in_memory_program(
        program,
        TemporalHostSchema::declaration().expect("the temporal schema is valid"),
        (TemporalContributionConfiguration { clock_source },),
        example_limits(),
        |graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .expect("the temporal principal binding is installed");
            seed_graph(graph, &principal, "blocked");
            Ok(())
        },
    )
    .err()
    .expect("the client action must not share the conditional operation");
    assert!(matches!(
        denial,
        WorthQueryInMemoryApplicationDenial::ConditionalProgramMismatch
    ));
}

#[test]
fn program_action_from_another_runtime_cannot_commit_this_product() {
    let mut first = ExampleApplication::publish("blocked");
    let mut second = ExampleApplication::publish("blocked");
    let branch = first.runtime.current_world();
    let scope = product_workflow_support::adapters::request_scope();
    let principal = principal(&first, &scope);
    let predecessor = read_input(&first, branch, &principal, &scope);
    let result = first
        .runtime
        .request(&principal, &scope)
        .mutate(amendment("must-not-publish", 2))
        .without_source()
        .idempotency(&0x72_u64)
        .execute_in_program(&second.runtime);
    assert!(matches!(
        result,
        Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch)
    ));
    assert_eq!(read_input(&first, branch, &principal, &scope), predecessor);
    first
        .runtime
        .close_conditional_runtime()
        .expect("the first conditional runtime closes");
    second
        .runtime
        .close_conditional_runtime()
        .expect("the second conditional runtime closes");
}

fn amendment(input: &str, revision: u64) -> AmendTemporalIntent {
    AmendTemporalIntent {
        identity: "intent-1".to_owned(),
        amendment: AmendTemporalInput {
            revision,
            due: 11,
            lifecycle: "active".to_owned(),
            input: input.to_owned(),
            gate: "ready".to_owned(),
        },
    }
}
