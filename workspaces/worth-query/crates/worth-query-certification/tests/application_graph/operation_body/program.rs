// Authoring differs only in the literal destination, never in the host handler.
use super::binding::BodyBinding;
use super::query::{document_retention_query_definition, DocumentRetentionQueryBinding};
use super::schema::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    declaration::{application_program::*, application_schema::*},
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

pub struct BodyContribution<const DESTINATION: u64>;
#[derive(Clone, Copy)]
pub enum Installation {
    Native,
    Missing,
    Replacement,
}
impl<const DESTINATION: u64> ApplicationSchemaContribution<DocumentRetentionSchema<DESTINATION>>
    for BodyContribution<DESTINATION>
{
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.body-court.contribution.v1");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<DESTINATION>>,
    ) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<DESTINATION>> {
        let body = SetRetention::reference::<DocumentRetentionSchema<DESTINATION>>()
            .body()
            .expect(DocumentRetentionField::reference(), 7)
            .unwrap()
            .assign(DocumentRetentionField::reference(), DESTINATION)
            .unwrap()
            .finish()
            .unwrap();
        schema
            .entity(ExternalMapping::reference())
            .entity(Principal::reference())
            .entity(Document::reference())
            .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .aspect(Document::reference(), DocumentFacts::reference())
            .field(
                ExternalMapping::reference(),
                ExternalIdentityField::reference(),
            )
            .field(
                ExternalMapping::reference(),
                MappingStatusField::reference(),
            )
            .field(Principal::reference(), PrincipalIdentityField::reference())
            .field(Document::reference(), DocumentIdentityField::reference())
            .field(Document::reference(), DocumentRetentionField::reference())
            .relation(
                MappingTarget::reference(),
                ExternalMapping::reference(),
                Principal::reference(),
            )
            .principal_binding(DocumentPrincipalBinding::reference())
            .operation(
                SetRetention::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(body)
                    .unwrap(),
            )
            .operation_projection_work_budget(SetRetention::reference(), 8)
            .operation_read_field(
                SetRetention::reference(),
                DocumentIdentityField::reference(),
            )
            .application_mutation_binding::<BodyBinding<DESTINATION>>()
            .application_query(document_retention_query_definition::<DESTINATION>())
            .application_query_binding::<DocumentRetentionQueryBinding<DESTINATION>>()
    }
}

impl<const DESTINATION: u64> WorthQueryApplicationContribution<DocumentRetentionSchema<DESTINATION>>
    for BodyContribution<DESTINATION>
{
    type Configuration = Installation;
    fn configure(
        install: Installation,
        setup: &mut WorthQueryApplicationContributionSetup<
            '_,
            DocumentRetentionSchema<DESTINATION>,
        >,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        match install {
            Installation::Native => setup.operation_body::<BodyBinding<DESTINATION>>()?,
            Installation::Missing => {}
            Installation::Replacement => {
                setup.handler::<BodyBinding<DESTINATION>, _>(super::replacement::Replacement)?
            }
        }
        Ok(())
    }
}

pub struct BodyFeature;
impl<const DESTINATION: u64> ApplicationFeature<DocumentRetentionSchema<DESTINATION>>
    for BodyFeature
{
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.body-court.feature.v1";
}
pub struct BodyProgram<const DESTINATION: u64>;
impl<const DESTINATION: u64> ApplicationProgramDefinition<DocumentRetentionSchema<DESTINATION>>
    for BodyProgram<DESTINATION>
{
    type Contributions = (BodyContribution<DESTINATION>,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.body-court.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema<DESTINATION>, BodyFeature>()
                .mutation::<BodyBinding<DESTINATION>>()
                .finish(),
        ]
    }
}

pub fn schema<const DESTINATION: u64>(
) -> ApplicationSchemaDeclaration<DocumentRetentionSchema<DESTINATION>> {
    ApplicationSchemaDeclarationBuilder::<DocumentRetentionSchema<DESTINATION>>::for_schema()
        .contributions()
        .register::<BodyContribution<DESTINATION>>()
        .unwrap()
        .build()
        .unwrap()
}
pub fn validated<const DESTINATION: u64>(
    schema: &ApplicationSchemaDeclaration<DocumentRetentionSchema<DESTINATION>>,
) -> ValidatedApplicationProgram<DocumentRetentionSchema<DESTINATION>, BodyProgram<DESTINATION>> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema<DESTINATION>, BodyProgram<DESTINATION>>::begin()
        .with_schema(schema).validated_program().unwrap()
}
