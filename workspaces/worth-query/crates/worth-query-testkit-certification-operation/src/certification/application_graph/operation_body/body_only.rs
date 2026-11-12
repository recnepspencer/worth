//! A body without an executable binding must not install as descriptive metadata.
pub use super::{host_limits, schema::*};
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    application_installation::{in_memory, WorthQueryApplicationPreparationDenial},
    declaration::application_schema::*,
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    },
};

pub struct BodyOnlySchema;
impl BodySchemaBinding for BodyOnlySchema {
    type InputBinding = BodyInputBinding<0>;
}
impl ApplicationSchema for BodyOnlySchema {
    const OWNER: &'static str = "worth.query.body-only-court";
    const NAME: &'static str = "BodyOnlySchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for BodyOnlySchema {
    type Contributions =
        <Self as worth_query_decl::facade::application_schema::fragment::Fragment>::Members;
}
pub struct BodyOnlyContribution;
impl ApplicationSchemaContribution<BodyOnlySchema> for BodyOnlyContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.body-only-court.contribution.v1");
    type Members = (BodyModel, BodyOnlyContracts);
}
impl WorthQueryApplicationContribution<BodyOnlySchema> for BodyOnlyContribution {
    type Configuration = ();
    fn configure(
        _: (),
        _: &mut WorthQueryApplicationContributionSetup<'_, BodyOnlySchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        Ok(())
    }
}

impl worth_query_decl::facade::application_schema::fragment::Fragment for BodyOnlySchema {
    const NAME: &'static str = "BodyOnlySchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (BodyOnlyContribution,);
}
pub struct BodyOnlyContracts;
impl worth_query_decl::facade::application_schema::fragment::Registration<BodyOnlySchema>
    for BodyOnlyContracts
{
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<BodyOnlySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<BodyOnlySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let body = SetRetention::reference::<BodyOnlySchema>()
                .body()
                .assign(DocumentRetentionField::at("DocumentRetentionField"), 8)
                .unwrap()
                .finish()
                .unwrap();
            schema.operation(
                SetRetention::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(body)
                    .unwrap(),
            )
        })
    }
}
