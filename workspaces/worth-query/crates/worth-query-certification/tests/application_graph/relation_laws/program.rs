//! Schema and native action meaning derive from one relation declaration.
use super::{handler::Handler, operation::*, schema::*};
use std::marker::PhantomData;
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{application_program::*, application_schema::*},
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
pub struct GraphContribution<L: LawPolicy>(PhantomData<fn() -> L>);
impl<L: LawPolicy> ApplicationSchemaContribution<GraphSchema<L>> for GraphContribution<L> {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.relation-laws.contribution.v1");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<GraphSchema<L>>,
    ) -> ApplicationSchemaDeclarationBuilder<GraphSchema<L>> {
        schema
            .entity(ExternalMapping::reference())
            .entity(Principal::reference())
            .entity(Node::reference())
            .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .aspect(Node::reference(), NodeFacts::reference())
            .field(
                ExternalMapping::reference(),
                ExternalIdentityField::reference(),
            )
            .field(
                ExternalMapping::reference(),
                MappingStatusField::reference(),
            )
            .field(Principal::reference(), PrincipalKey::reference())
            .field(Node::reference(), NodeKey::reference())
            .relation(
                MappingTarget::reference(),
                ExternalMapping::reference(),
                Principal::reference(),
            )
            .principal_binding(GraphPrincipal::reference())
            .relation(Precedes::reference(), Node::reference(), Node::reference())
            .operation(
                EditEdges::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(EditEdges::reference(), NodeKey::reference())
            .operation_link(EditEdges::reference(), Precedes::reference())
            .application_mutation_binding::<EditBinding<L>>()
    }
}
impl<L: LawPolicy> WorthQueryApplicationContribution<GraphSchema<L>> for GraphContribution<L> {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, GraphSchema<L>>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<EditBinding<L>, _>(Handler)
    }
}
pub struct GraphFeature;
impl<L: LawPolicy> ApplicationFeature<GraphSchema<L>> for GraphFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.relation-laws.feature.v1";
}
pub struct GraphProgram<L: LawPolicy>(PhantomData<fn() -> L>);
impl<L: LawPolicy> ApplicationProgramDefinition<GraphSchema<L>> for GraphProgram<L> {
    type Contributions = (GraphContribution<L>,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.relation-laws.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<GraphSchema<L>, GraphFeature>()
                .native_action::<EditBinding<L>>()
                .finish_action()
                .finish(),
        ]
    }
}
