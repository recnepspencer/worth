#[path = "../examples/product_workflow_support/mod.rs"]
pub mod product_workflow_support;

use std::sync::Arc;

use product_workflow_support::adapters::ClockSource;
use product_workflow_support::application::{example_limits, seed_graph};
use product_workflow_support::conditional_contribution::TemporalContributionConfiguration;
use product_workflow_support::program::{TemporalExampleFeature, TemporalExampleProgram};
use product_workflow_support::schema::{
    AmendTemporal, AmendTemporalAndPublishDefinition, ExecuteTemporal, RevokeTemporalPrincipal,
    TemporalHostSchema, TemporalPrincipalBinding,
};
use product_workflow_support::{ExampleApplication, ReplacementPredicate};
use worth_query_host::facade::{
    application_installation::{self, WorthQueryInMemoryApplicationDenial},
    declaration::application_program::{
        ApplicationActionLeaf, ApplicationActionList, ApplicationFeature,
        ApplicationFeatureInputLeaf, ApplicationFeatureList, ApplicationFeatureRef,
        ApplicationOperationActionRef, ApplicationProgramAuthoring, ApplicationProgramDefinition,
        ApplicationProgramIdentity,
    },
    primary_graph::{WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome},
    product::WorthQueryProductTransactionCommitError,
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
    type Actions = ApplicationActionList<
        ApplicationOperationActionRef<
            TemporalHostSchema,
            ConflictingConditionalClientFeature,
            ExecuteTemporal,
        >,
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Actions,
    >;
    type Features = ApplicationFeatureList<
        ApplicationFeatureRef<TemporalHostSchema, ConflictingConditionalClientFeature>,
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Features,
    >;
    type Outputs =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Outputs;
    type Rules =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Rules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.example.conflicting-conditional-client.v1");
}

impl ApplicationProgramDefinition<TemporalHostSchema> for MissingConditionalActionProgram {
    type Contributions =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Contributions;
    type Actions = ApplicationActionList<
        ApplicationOperationActionRef<TemporalHostSchema, TemporalExampleFeature, AmendTemporal>,
        ApplicationActionList<
            ApplicationOperationActionRef<
                TemporalHostSchema,
                TemporalExampleFeature,
                AmendTemporalAndPublishDefinition,
            >,
            ApplicationActionList<
                ApplicationOperationActionRef<
                    TemporalHostSchema,
                    TemporalExampleFeature,
                    RevokeTemporalPrincipal,
                >,
                ApplicationActionLeaf,
            >,
        >,
    >;
    type Features =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Features;
    type Outputs =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Outputs;
    type Rules =
        <TemporalExampleProgram as ApplicationProgramDefinition<TemporalHostSchema>>::Rules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.example.missing-conditional-action.v1");
}

#[test]
fn program_example_denies_plain_commit_and_conditional_client_admission() {
    let mut application = ExampleApplication::publish("blocked");
    assert!(application
        .runtime
        .admit_program_operation::<ExecuteTemporal>()
        .is_err());

    let branch = application.runtime.current_world();
    let selected = application
        .runtime
        .on_branch(branch)
        .select()
        .expect("the published branch is selectable");
    let predecessor = selected.product().selected_commit().clone();
    let change = application.admit_combined_change(
        selected,
        "must-not-publish",
        Arc::new(ReplacementPredicate),
    );
    let outcome = application
        .runtime
        .on_branch(branch)
        .transaction()
        .apply(change)
        .commit()
        .expect("the change belongs to this branch");
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("the plain entry must deny a program-owned action");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    );
    let current = application
        .runtime
        .on_branch(branch)
        .select()
        .expect("the denied branch remains selectable");
    assert_eq!(current.product().selected_commit(), &predecessor);

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
    let selected = first
        .runtime
        .on_branch(branch)
        .select()
        .expect("the first product is selectable");
    let predecessor = selected.product().selected_commit().clone();
    let change =
        first.admit_combined_change(selected, "must-not-publish", Arc::new(ReplacementPredicate));
    let foreign = second
        .runtime
        .admit_program_operation::<AmendTemporalAndPublishDefinition>()
        .expect("the second program declares the same action");
    let result = first
        .runtime
        .on_branch(branch)
        .transaction()
        .apply(change)
        .commit_for_program(foreign);
    assert!(matches!(
        result,
        Err(WorthQueryProductTransactionCommitError::ApplicationMismatch)
    ));
    let current = first
        .runtime
        .on_branch(branch)
        .select()
        .expect("the first branch remains selectable");
    assert_eq!(current.product().selected_commit(), &predecessor);
    first
        .runtime
        .close_conditional_runtime()
        .expect("the first conditional runtime closes");
    second
        .runtime
        .close_conditional_runtime()
        .expect("the second conditional runtime closes");
}
