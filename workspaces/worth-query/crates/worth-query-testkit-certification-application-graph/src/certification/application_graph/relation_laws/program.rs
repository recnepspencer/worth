//! Schema and native action meaning derive from one relation declaration.
pub use super::{creation::*, feeding::*, handler::Handler, operation::*};
pub use std::marker::PhantomData;
pub use worth_query_host::facade::{
    application_contribution::*, declaration::application_program::*, domain::AbsenceLaw,
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
pub struct GraphContribution<L: LawPolicy>(pub PhantomData<fn() -> L>);
impl<L: LawPolicy> ApplicationSchemaContribution<GraphSchema<L>> for GraphContribution<L> {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.relation-laws.contribution.v1");
    type Members = (L::Model, GraphContracts<L>);
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

pub struct GraphContracts<L: LawPolicy>(PhantomData<fn() -> L>);
impl<L: LawPolicy> Registration<GraphSchema<L>> for GraphContracts<L> {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<GraphSchema<L>>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<GraphSchema<L>>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = schema
                .principal_binding(GraphPrincipal::reference())
                .operation(
                    EditEdges::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_read_field(EditEdges::reference(), node_key::<GraphSchema<L>>())
                .operation_read_relation(EditEdges::reference(), precedes::<GraphSchema<L>>())
                .operation_link(EditEdges::reference(), precedes::<GraphSchema<L>>())
                .operation_unlink(EditEdges::reference(), precedes::<GraphSchema<L>>())
                .application_mutation_binding::<EditBinding<L>>()
                .operation(
                    CreateNode::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_read_field(CreateNode::reference(), node_key::<GraphSchema<L>>())
                .operation_link(CreateNode::reference(), precedes::<GraphSchema<L>>())
                .operation_create(CreateNode::reference(), node::<GraphSchema<L>>())
                .operation_write(CreateNode::reference(), node_key::<GraphSchema<L>>())
                .application_mutation_binding::<CreateBinding<L>>()
                .operation(
                    EditFeeds::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_read_field(EditFeeds::reference(), node_key::<GraphSchema<L>>())
                .operation_read_field(EditFeeds::reference(), LeafKey::at("LeafKey"))
                .operation_read_relation(EditFeeds::reference(), feeds::<GraphSchema<L>>())
                .operation_read_entity(EditFeeds::reference(), node::<GraphSchema<L>>())
                .operation_read_entity(EditFeeds::reference(), Leaf::reference())
                .operation_create(EditFeeds::reference(), Leaf::reference())
                .operation_write(EditFeeds::reference(), LeafKey::at("LeafKey"))
                .operation_link(EditFeeds::reference(), feeds::<GraphSchema<L>>())
                .operation_unlink(EditFeeds::reference(), feeds::<GraphSchema<L>>())
                .operation_delete(EditFeeds::reference(), node::<GraphSchema<L>>())
                .operation_delete(EditFeeds::reference(), Leaf::reference())
                .application_mutation_binding::<FeedsBinding<L>>();
            let schema = if L::RELATION_PAYLOAD {
                schema.operation_initialize_relation_field(
                    EditEdges::reference(),
                    edge_weight::<GraphSchema<L>>().expect("payload record field"),
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
        })
    }
}
