//! The counted fixture owns exactly its source, output and conditional action.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuoteSchema;
impl worth_query_decl::facade::application_schema::fragment::Fragment for QuoteSchema {
    const NAME: &'static str = "QuoteSchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (QuoteContribution,);
}
impl worth_query_decl::facade::application_schema::ApplicationSchema for QuoteSchema {
    const OWNER: &'static str = "worth.query.certification.quote-count";
    const NAME: &'static str = "QuoteSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<Self>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        worth_query_decl::facade::application_schema::fragment::visit_correction(
            Self, source, visitor,
        )
    }
}
impl worth_query_decl::facade::application_schema::ApplicationSchemaComposition for QuoteSchema {
    type Contributions = (QuoteContribution,);
}

impl TopologySchemaBinding for QuoteSchema {
    type PrincipalFragment = crate::PrincipalRecords;
}

pub struct QuoteContribution;
impl<S: TopologySchemaBinding>
    worth_query_decl::facade::application_schema::ApplicationSchemaContribution<S>
    for QuoteContribution
{
    const IDENTITY:
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionIdentity =
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionIdentity::new(
            "worth.query.certification.quote-count-contribution.v1",
        );
    type Members = (
        QuoteRecords,
        crate::MinimalPrincipalRecords,
        QuoteContributionContracts,
    );
}
pub struct QuoteContributionContracts;
impl<S: TopologySchemaBinding>
    worth_query_decl::facade::application_schema::fragment::Registration<S>
    for QuoteContributionContracts
{
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
            S,
        >,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<S>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = schema.principal_binding(ConsumerPrincipalBinding::reference::<S>());
            super::declare(schema)
        })
    }
}

impl<S: TopologySchemaBinding> WorthQueryApplicationContribution<S> for QuoteContribution {
    type Configuration = ();

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<S>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        super::contracts(contracts)
    }

    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        super::configure(setup)
    }
}
