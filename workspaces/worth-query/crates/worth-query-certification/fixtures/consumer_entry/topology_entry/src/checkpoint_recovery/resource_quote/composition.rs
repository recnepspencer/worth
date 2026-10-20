//! The counted fixture owns exactly its source, output and conditional action.
use super::*;
use worth_query_decl::facade::{worth_query_application, worth_query_application_contribution};

worth_query_application! {
    pub QuoteSchema {
        owner: "worth.query.certification.quote-count",
        version: (1, 0),
        contributions: [QuoteContribution],
    }
}
impl TopologySchemaBinding for QuoteSchema {}

worth_query_application_contribution! {
    pub contribution QuoteContribution for S: TopologySchemaBinding {
        identity: "worth.query.certification.quote-count-contribution.v1",
        members: |schema| {
            let schema = schema
                .entity(Body::reference::<S>())
                .aspect(Body::reference::<S>(), PlanarPosition::reference::<S>())
                .field(Body::reference::<S>(), BodyKey::reference::<S>())
                .entity(ExternalPrincipalMapping::reference::<S>())
                .entity(Principal::reference::<S>())
                .aspect(ExternalPrincipalMapping::reference::<S>(), ExternalIdentity::reference::<S>())
                .aspect(Principal::reference::<S>(), PrincipalFacts::reference::<S>())
                .field(ExternalPrincipalMapping::reference::<S>(), ExternalIdentityField::reference::<S>())
                .field(ExternalPrincipalMapping::reference::<S>(), MappingStatusField::reference::<S>())
                .field(Principal::reference::<S>(), PrincipalIdentity::reference::<S>())
                .relation(MappingTarget::reference::<S>(), ExternalPrincipalMapping::reference::<S>(), Principal::reference::<S>())
                .principal_binding(ConsumerPrincipalBinding::reference::<S>());
            super::declare(schema)
        }
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
