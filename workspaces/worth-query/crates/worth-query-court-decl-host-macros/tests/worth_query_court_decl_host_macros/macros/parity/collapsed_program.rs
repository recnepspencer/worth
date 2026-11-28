//! Collapsed program, query registration and action alias parity with a manual program.
use super::super::actions::*;
use super::super::program::{
    BoardDefinition, CompleteAction, Contribution, Program, RenameAction, RescheduleAction,
};
use super::super::reads;
use super::super::relations;
use super::super::schema::*;
use worth_query_decl::facade::application_operation::*;
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::{
    application_contribution::*, primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    worth_query_application_contribution,
};

mod manual {
    use super::*;

    pub struct ManualFeature;
    impl ApplicationFeature<Schema> for ManualFeature {
        type Inputs = ApplicationFeatureInputLeaf;
        const IDENTITY: &'static str = "macros.board.feature.v1";
    }
    worth_query_application_contribution! {
        pub feature ManualDefinition<Schema> for ManualFeature => ManualRuntime {
            schema: |schema| {
                schema
                    .application_query(reads::search())
                    .application_query_binding::<reads::SearchBinding>()
                    .application_query(reads::due())
                    .application_query_binding::<reads::DueBinding>()
                    .application_query(reads::title_search())
                    .application_query_binding::<reads::TitleSearchBinding>()
                    .application_query(relations::outline())
                    .application_query_binding::<relations::OutlineQueryBinding>()
            };
            body_action(RenameBinding, Renamed) { operation: rename_definition(); };
            body_action(RescheduleBinding, Rescheduled) { operation: reschedule_definition(); };
            body_action(RetitleFirstBinding, RetitledFirst) { operation: retitle_first_definition(); };
            body_action(CompleteBinding, Completed) { operation: complete_definition(); };
        }
    }

    pub struct ManualFeatures;
    impl Registration<Schema> for ManualFeatures {
        fn register(
            schema: ApplicationSchemaDeclarationBuilder<Schema>,
        ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
        {
            Ok(ManualDefinition::declare_members(schema))
        }
    }

    pub struct ManualContribution;
    impl ApplicationSchemaContribution<Schema> for ManualContribution {
        const IDENTITY: ApplicationSchemaContributionIdentity =
            ApplicationSchemaContributionIdentity::new("macros.board-contribution.v1");
        type Members = (Board, PrincipalBinding, ManualFeatures);
    }
    impl WorthQueryApplicationContribution<Schema> for ManualContribution {
        type Configuration =
            <ManualDefinition as WorthQueryFeatureDefinition<Schema>>::Configuration;
        fn contracts(
            contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
        ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
            ManualDefinition::contracts(contracts)
        }
        fn configure(
            configuration: Self::Configuration,
            setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
        ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
            ManualDefinition::configure(configuration, setup)
        }
    }

    pub struct ManualProgram;
    impl ApplicationProgramDefinition<Schema> for ManualProgram {
        type Contributions = (ManualContribution,);
        type Outputs = ApplicationProgramOutputs<
            <ManualDefinition as ApplicationFeatureDefinition<Schema>>::Outputs,
        >;
        type Rules = ApplicationRuleLeaf;
        const IDENTITY: ApplicationProgramIdentity =
            ApplicationProgramIdentity::new("macros.program.v1");
        fn feature_specs() -> Vec<ApplicationFeatureSpec> {
            ManualDefinition::feature_specs()
        }
    }
}

#[test]
fn collapsed_program_matches_manual_declarations_and_aliases() {
    // Collapsed program matches manual 5-trait program revision and feature specs.
    let schema = Schema::declaration().expect("schema declares");
    let derived_program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&schema)
        .validated_program()
        .expect("collapsed program validates");
    let manual_program = ApplicationProgramAuthoring::<Schema, manual::ManualProgram>::begin()
        .with_schema(&schema)
        .validated_program()
        .expect("manual program validates");
    assert_eq!(derived_program.revision(), manual_program.revision());
    let derived_specs = <BoardDefinition as ApplicationFeatureDefinition<Schema>>::feature_specs();
    let manual_specs =
        <manual::ManualDefinition as ApplicationFeatureDefinition<Schema>>::feature_specs();
    assert_eq!(derived_specs.len(), manual_specs.len());
    assert_eq!(
        derived_specs[0].feature().identity(),
        manual_specs[0].feature().identity()
    );
    assert_eq!(
        derived_specs[0].feature().composition_instance(),
        manual_specs[0].feature().composition_instance()
    );
    assert_eq!(derived_specs[0].actions(), manual_specs[0].actions());

    assert_eq!(
        RenameAction::selector().binding(),
        <RenameBinding as ApplicationMutationBinding<Schema>>::IDENTITY
    );
    assert_eq!(
        RescheduleAction::selector().binding(),
        <RescheduleBinding as ApplicationMutationBinding<Schema>>::IDENTITY
    );
    assert_eq!(
        CompleteAction::selector().binding(),
        <CompleteBinding as ApplicationMutationBinding<Schema>>::IDENTITY
    );
    assert_eq!(
        <Contribution as ApplicationSchemaContribution<Schema>>::IDENTITY,
        <manual::ManualContribution as ApplicationSchemaContribution<Schema>>::IDENTITY
    );
}
