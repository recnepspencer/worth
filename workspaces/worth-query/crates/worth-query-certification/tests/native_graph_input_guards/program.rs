//! Native declaration and installation are the only operation implementation.
use super::{actions::*, schema::*};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    declaration::{application_program::*, application_schema::*},
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
pub struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.path-input.contribution.v1");
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        let builder = super::path_value_targets::declare(builder);
        builder
            .entity(Mapping::reference())
            .entity(Principal::reference())
            .entity(Parent::reference())
            .entity(Row::reference())
            .entity(Audit::reference())
            .aspect(Mapping::reference(), MappingFacts::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .aspect(Parent::reference(), ParentFacts::reference())
            .aspect(Row::reference(), RowFacts::reference())
            .aspect(Audit::reference(), AuditFacts::reference())
            .field(Mapping::reference(), ExternalIdentity::reference())
            .field(Mapping::reference(), MappingStatus::reference())
            .field(Principal::reference(), PrincipalIdentity::reference())
            .field(Parent::reference(), ParentIdentity::reference())
            .field(Parent::reference(), Company::reference())
            .field(Parent::reference(), ParentRevision::reference())
            .field(Row::reference(), RowIdentity::reference())
            .field(Row::reference(), RowValue::reference())
            .field(Audit::reference(), AuditIdentity::reference())
            .field(Audit::reference(), AuditValue::reference())
            .relation(
                MappingTarget::reference(),
                Mapping::reference(),
                Principal::reference(),
            )
            .relation(
                ParentRows::reference(),
                Parent::reference(),
                Row::reference(),
            )
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
                            .set_optional_input(Company::reference(), "company")
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
                            .initialize_input(ParentIdentity::reference(), "identity")
                            .unwrap()
                            .initialize_optional_input(Company::reference(), "optional_company")
                            .unwrap()
                            .initialize(ParentRevision::reference(), 1)
                            .unwrap()
                            .link_to_scope(ParentRows::reference(), "parent-row")
                            .unwrap()
                            .finish()
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .operation_read_field(Change::reference(), ParentIdentity::reference())
            .operation_read_field(Introduce::reference(), RowIdentity::reference())
            .operation_projection_work_budget(Save::reference(), 128)
            .operation_projection_work_budget(Change::reference(), 32)
            .operation_projection_work_budget(Introduce::reference(), 32)
            .application_mutation_binding::<SaveBinding>()
            .application_mutation_binding::<ChangeBinding>()
            .application_mutation_binding::<IntroduceBinding>()
            .application_query(super::row_query::definition())
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
                RowIdentity::reference(),
            )
            .application_mutation_binding::<super::path_value_binding::BumpBinding>()
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.operation_body::<SaveBinding>()?;
        setup.operation_body::<ChangeBinding>()?;
        setup.operation_body::<IntroduceBinding>()?;
        setup.operation_body::<super::path_value_binding::BumpBinding>()?;
        setup.operation_body::<super::path_value_targets::CopyBinding>()?;
        setup.operation_body::<super::path_value_targets::AliasBinding>()
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
            .finish()]
    }
}
pub fn schema() -> ApplicationSchemaDeclaration<Schema> {
    ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .contributions()
        .register::<Contribution>()
        .unwrap()
        .build()
        .unwrap()
}
pub fn validated(
    schema: &ApplicationSchemaDeclaration<Schema>,
) -> ValidatedApplicationProgram<Schema, Program> {
    ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(schema)
        .validated_program()
        .unwrap()
}
