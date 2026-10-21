//! A body without an executable binding must not install as descriptive metadata.
use super::{host_limits, schema::*};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    application_installation::{in_memory, WorthQueryInMemoryApplicationDenial},
    declaration::application_schema::*,
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    },
};

struct BodyOnlySchema;
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
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .contributions()
            .register::<BodyOnlyContribution>()?
            .build()
    }
}
impl ApplicationSchemaComposition for BodyOnlySchema {
    type Contributions = (BodyOnlyContribution,);
}
struct BodyOnlyContribution;
impl ApplicationSchemaContribution<BodyOnlySchema> for BodyOnlyContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.body-only-court.contribution.v1");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<BodyOnlySchema>,
    ) -> ApplicationSchemaDeclarationBuilder<BodyOnlySchema> {
        let body = SetRetention::reference::<BodyOnlySchema>()
            .body()
            .assign(DocumentRetentionField::reference(), 8)
            .unwrap()
            .finish()
            .unwrap();
        schema
            .entity(Document::reference())
            .aspect(Document::reference(), DocumentFacts::reference())
            .field(Document::reference(), DocumentRetentionField::reference())
            .operation(
                SetRetention::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(body)
                    .unwrap(),
            )
    }
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

#[test]
fn authored_body_without_a_mutation_binding_is_denied_before_initialization() {
    let result = in_memory(
        BodyOnlySchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        |_phase, _graph, _schema| panic!("missing body execution must deny before initial state"),
    );
    match result {
        Err(WorthQueryInMemoryApplicationDenial::Contributions(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::OperationBodyUnavailable,
        ),
        Err(other) => panic!("expected missing body coverage: {other:?}"),
        Ok(_) => panic!("a metadata-only operation body cannot be installed"),
    }
}
