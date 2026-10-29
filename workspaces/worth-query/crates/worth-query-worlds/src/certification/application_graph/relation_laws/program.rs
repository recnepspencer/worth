//! Schema and native action meaning derive from one relation declaration.
pub use super::{creation::*, feeding::*, handler::Handler, operation::*, schema::*};
pub use std::marker::PhantomData;
pub use worth_query_host::facade::{
    application_contribution::*,
    declaration::{application_program::*, application_schema::*},
    domain::AbsenceLaw,
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
pub struct GraphContribution<L: LawPolicy>(pub PhantomData<fn() -> L>);
impl<L: LawPolicy> ApplicationSchemaContribution<GraphSchema<L>> for GraphContribution<L> {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.relation-laws.contribution.v1");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<GraphSchema<L>>,
    ) -> ApplicationSchemaDeclarationBuilder<GraphSchema<L>> {
        let schema = schema
            .entity(ExternalMapping::reference())
            .entity(Principal::reference())
            .entity(Node::reference())
            .entity(Leaf::reference())
            .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .aspect(Node::reference(), NodeFacts::reference())
            .aspect(Leaf::reference(), LeafFacts::reference())
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
            .field(Leaf::reference(), LeafKey::reference())
            .relation(
                MappingTarget::reference(),
                ExternalMapping::reference(),
                Principal::reference(),
            )
            .principal_binding(GraphPrincipal::reference())
            .relation(Precedes::reference(), Node::reference(), Node::reference())
            .relation(Feeds::reference(), Node::reference(), Leaf::reference())
            .operation(
                EditEdges::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(EditEdges::reference(), NodeKey::reference())
            .operation_read_relation(EditEdges::reference(), Precedes::reference())
            .operation_link(EditEdges::reference(), Precedes::reference())
            .operation_unlink(EditEdges::reference(), Precedes::reference())
            .application_mutation_binding::<EditBinding<L>>()
            .operation(
                CreateNode::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(CreateNode::reference(), NodeKey::reference())
            .operation_link(CreateNode::reference(), Precedes::reference())
            .operation_create(CreateNode::reference(), Node::reference())
            .operation_write(CreateNode::reference(), NodeKey::reference())
            .application_mutation_binding::<CreateBinding<L>>()
            .operation(
                EditFeeds::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(EditFeeds::reference(), NodeKey::reference())
            .operation_read_field(EditFeeds::reference(), LeafKey::reference())
            .operation_read_relation(EditFeeds::reference(), Feeds::reference())
            .operation_read_entity(EditFeeds::reference(), Node::reference())
            .operation_read_entity(EditFeeds::reference(), Leaf::reference())
            .operation_create(EditFeeds::reference(), Leaf::reference())
            .operation_write(EditFeeds::reference(), LeafKey::reference())
            .operation_link(EditFeeds::reference(), Feeds::reference())
            .operation_unlink(EditFeeds::reference(), Feeds::reference())
            .operation_delete(EditFeeds::reference(), Node::reference())
            .operation_delete(EditFeeds::reference(), Leaf::reference())
            .application_mutation_binding::<FeedsBinding<L>>();
        let schema = if L::RELATION_PAYLOAD {
            schema
                .relation_aspect(
                    ApplicationRelationAspectDefinition::new(
                        Precedes::reference(),
                        EdgeFacts::reference(),
                        AbsenceLaw::Optional,
                    )
                    .field(EdgeWeight::reference())
                    .finish()
                    .unwrap(),
                )
                .operation_initialize_relation_field(
                    EditEdges::reference(),
                    EdgeWeight::reference(),
                )
        } else {
            schema
        };
        let schema = if L::DERIVED_RELATIONS {
            super::derived::declare(schema)
        } else {
            schema
        };
        let schema = if L::LOGICAL_RELATIONS {
            super::logical::declare(schema)
        } else {
            schema
        };
        L::observation_members(schema)
    }
}
impl<L: LawPolicy> WorthQueryApplicationContribution<GraphSchema<L>> for GraphContribution<L> {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, GraphSchema<L>>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if L::RELATION_PAYLOAD {
            setup.handler::<EditBinding<L>, _>(super::payload::PayloadHandler::<L>::default())?;
        } else {
            setup.handler::<EditBinding<L>, _>(Handler)?;
        }
        setup.handler::<CreateBinding<L>, _>(CreateHandler)?;
        setup.handler::<FeedsBinding<L>, _>(FeedsHandler)?;
        if L::DERIVED_RELATIONS {
            super::derived::configure(setup)?;
        }
        if L::LOGICAL_RELATIONS {
            super::logical::configure(setup)?;
        }
        Ok(())
    }
}
pub struct GraphFeature;
impl<L: LawPolicy> ApplicationFeature<GraphSchema<L>> for GraphFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.relation-laws.feature.v1";
}
pub struct GraphRules<L: LawPolicy>(pub PhantomData<fn() -> L>);
impl<L: LawPolicy> ApplicationProgramRulesShape<GraphSchema<L>> for GraphRules<L> {
    fn rules() -> Vec<ApplicationProgramRuleDeclaration> {
        if L::DERIVED_RELATIONS {
            super::derived::program_rules::<L>()
        } else if L::LOGICAL_RELATIONS {
            super::logical::program_rules::<L>()
        } else {
            Vec::new()
        }
    }
}
pub struct GraphProgram<L: LawPolicy>(pub PhantomData<fn() -> L>);
impl<L: LawPolicy> ApplicationProgramDefinition<GraphSchema<L>> for GraphProgram<L> {
    type Contributions = (GraphContribution<L>,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = GraphRules<L>;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.relation-laws.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let feature = ApplicationFeatureSpec::root::<GraphSchema<L>, GraphFeature>()
            .native_action::<EditBinding<L>>()
            .finish_action()
            .native_action::<CreateBinding<L>>()
            .finish_action()
            .native_action::<FeedsBinding<L>>()
            .finish_action();
        let feature = if L::DERIVED_RELATIONS || L::LOGICAL_RELATIONS {
            feature
                .native_action::<super::derived::CountBinding<L>>()
                .finish_action()
                .derived_artifact::<super::derived::Artifact>()
                .managed_computation::<super::derived::CountComputation>()
                .derived_collection::<super::derived::CountCollection>()
        } else {
            feature
        };
        let feature = if L::LOGICAL_RELATIONS {
            feature
                .native_action::<super::logical::CheckBinding<L>>()
                .finish_action()
        } else {
            feature
        };
        vec![feature.finish()]
    }
}
