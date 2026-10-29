#[path = "program_example_authority/product_workflow_support/mod.rs"]
pub mod product_workflow_support;

#[path = "execution_memory.rs"]
pub mod execution_memory;
#[path = "mutation_attempt_report.rs"]
pub mod mutation_attempt_report;

pub use product_workflow_support::adapters::ClockSource;
pub use product_workflow_support::application::{example_limits, seed_graph};
pub use product_workflow_support::application_entry::AmendTemporalBinding;
pub use product_workflow_support::conditional_contribution::TemporalContributionConfiguration;
pub use product_workflow_support::program::{TemporalExampleFeature, TemporalExampleProgram};
pub use product_workflow_support::schema::{
    AmendTemporal, AmendTemporalInput, ExecuteTemporal, IntentIdentityField,
    RevokeTemporalPrincipal, TemporalHostSchema, TemporalPrincipalBinding,
};
pub use product_workflow_support::{
    principal, read_input, AmendTemporalIntent, ExampleApplication,
};
pub use worth_query_decl::facade::application_operation::ApplicationMutationIdentities;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
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
        WorthQueryApplicationIdempotencyBinding, WorthQueryInvariantExecutionFailure,
        WorthQueryPrincipalResolutionMode,
    },
    product::WorthQueryAdmittedChange,
};

pub struct MissingConditionalActionProgram;
pub struct ConflictingConditionalClientFeature;
pub struct ConflictingConditionalClientProgram;

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
                .action::<AmendTemporalBinding, product_workflow_support::program::TemporalCompletion>().finish_action()
                .operation::<RevokeTemporalPrincipal>()
                .finish(),
        ]
    }
}

pub fn amendment(input: &str, revision: u64) -> AmendTemporalIntent {
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

#[path = "temporal_integrity_lease.rs"]
pub mod temporal_integrity_lease;
#[path = "temporal_integrity_reentry.rs"]
pub mod temporal_integrity_reentry;
