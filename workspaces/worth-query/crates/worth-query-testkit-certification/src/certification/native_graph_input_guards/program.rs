//! Native declaration and installation are the only operation implementation.
pub use super::actions::*;
pub use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    declaration::application_program::*,
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
pub struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.path-input.contribution.v1");
    type Members = (super::schema::InputModel, Contracts);
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.operation_body::<super::peer_audit::Binding>()?;
        setup.operation_body::<SaveBinding>()?;
        setup.operation_body::<ChangeBinding>()?;
        setup.operation_body::<IntroduceBinding>()?;
        setup.operation_body::<super::path_value_binding::BumpBinding>()?;
        setup.operation_body::<super::path_value_targets::CopyBinding>()?;
        setup.operation_body::<super::path_value_targets::AliasBinding>()?;
        setup.operation_body::<super::retiring_actions::KeepBinding>()?;
        setup.operation_body::<super::retiring_actions::RetireBinding>()?;
        setup.operation_body::<super::reuse_actions::Binding>()?;
        setup.operation_body::<super::reuse_actions::NoteBinding>()?;
        setup.operation_body::<super::path_signing::GraphBinding>()?;
        setup.operation_body::<super::path_signing::GraphAliasBinding>()?;
        setup.operation_body::<super::path_signing::ScopeAliasBinding>()?;
        setup.operation_body::<super::reference_input::Binding>()?;
        setup.operation_body::<super::reference_input::SignBinding>()?;
        setup.operation_body::<super::child_actions::Binding>()
    }
}
pub struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.path-input.feature.v1";
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.path-input.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .native_action::<super::peer_audit::Binding>()
            .finish_action()
            .native_action::<SaveBinding>()
            .finish_action()
            .native_action::<ChangeBinding>()
            .finish_action()
            .native_action::<IntroduceBinding>()
            .finish_action()
            .native_action::<super::path_value_binding::BumpBinding>()
            .finish_action()
            .native_action::<super::path_value_targets::CopyBinding>()
            .finish_action()
            .native_action::<super::path_value_targets::AliasBinding>()
            .finish_action()
            .native_action::<super::retiring_actions::KeepBinding>()
            .finish_action()
            .native_action::<super::retiring_actions::RetireBinding>()
            .finish_action()
            .native_action::<super::reuse_actions::Binding>()
            .finish_action()
            .native_action::<super::reuse_actions::NoteBinding>()
            .finish_action()
            .native_action::<super::child_actions::Binding>()
            .finish_action()
            .native_action::<super::path_signing::GraphBinding>()
            .finish_action()
            .native_action::<super::path_signing::GraphAliasBinding>()
            .finish_action()
            .native_action::<super::path_signing::ScopeAliasBinding>()
            .finish_action()
            .native_action::<super::reference_input::Binding>()
            .finish_action()
            .native_action::<super::reference_input::SignBinding>()
            .finish_action()
            .finish()]
    }
}
pub fn schema() -> ApplicationSchemaDeclaration<Schema> {
    Schema::declaration().unwrap()
}
pub fn validated(
    schema: &ApplicationSchemaDeclaration<Schema>,
) -> ValidatedApplicationProgram<Schema, Program> {
    ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(schema)
        .validated_program()
        .unwrap()
}

pub struct Contracts;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for Contracts {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok({
            let builder = super::path_value_targets::declare(builder);
            let builder =
                super::retiring_actions::declare(super::selection_schema::declare(builder));
            let builder = super::reuse_actions::declare(super::peer_audit::declare(builder));
            let builder = super::child_actions::declare(builder);
            let builder = super::path_signing::declare(builder);
            let builder = super::reference_input::declare(builder);
            builder
                .principal_binding(PrincipalBinding::reference())
                .operation(
                    Save::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish()
                        .with_body(save_body())
                        .unwrap(),
                )
                .operation(
                    Change::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish()
                        .with_body(
                            Change::reference()
                                .body()
                                .set_optional_input(Company::at("Company"), "company")
                                .unwrap()
                                .finish()
                                .unwrap(),
                        )
                        .unwrap(),
                )
                .operation(
                    Introduce::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish()
                        .with_body(
                            Introduce::reference()
                                .body()
                                .in_scope(Row::reference())
                                .unwrap()
                                .create_in_scope(Parent::reference(), "parent")
                                .unwrap()
                                .initialize_input(ParentIdentity::at("ParentIdentity"), "identity")
                                .unwrap()
                                .initialize_optional_input(
                                    Company::at("Company"),
                                    "optional_company",
                                )
                                .unwrap()
                                .initialize(ParentRevision::at("ParentRevision"), 1)
                                .unwrap()
                                .link_to_scope(ParentRows::reference(), "parent-row")
                                .unwrap()
                                .finish()
                                .unwrap(),
                        )
                        .unwrap(),
                )
                .operation_read_field(Change::reference(), ParentIdentity::at("ParentIdentity"))
                .operation_read_field(Introduce::reference(), RowIdentity::at("RowIdentity"))
                .operation_projection_work_budget(Save::reference(), 128)
                .operation_projection_work_budget(Change::reference(), 32)
                .operation_projection_work_budget(Introduce::reference(), 32)
                .application_mutation_binding::<SaveBinding>()
                .application_mutation_binding::<ChangeBinding>()
                .application_mutation_binding::<IntroduceBinding>()
                .application_query(super::row_query::definition())
                .application_query(super::child_query::definition())
                .application_query(super::child_query::archive_definition())
                .application_query_binding::<super::child_query::ReadArchiveBinding>()
                .application_query_binding::<super::child_query::ReadBinding>()
                .application_query_binding::<super::row_query::ReadBinding>()
                .application_query(super::audit_query::definition())
                .application_query_binding::<super::audit_query::ReadBinding>()
                .application_query(super::revision_query::definition())
                .application_query_binding::<super::revision_query::ReadBinding>()
                .operation(
                    super::path_value::Bump::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish()
                        .with_body(super::path_value::body())
                        .unwrap(),
                )
                .operation_projection_work_budget(super::path_value::Bump::reference(), 256)
                .operation_read_field(
                    super::path_value::Bump::reference(),
                    RowIdentity::at("RowIdentity"),
                )
                .application_mutation_binding::<super::path_value_binding::BumpBinding>()
                .application_query(super::snapshot_query::definition())
                .application_query_binding::<super::snapshot_query::ReadBinding>()
                .application_query(super::current_snapshot_query::definition())
                .application_query_binding::<super::current_snapshot_query::ReadBinding>()
        })
    }
}
