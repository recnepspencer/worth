//! One authored derived recipe is shared by ordinary rules and a managed collection.
pub use super::{program::GraphFeature, schema::*};
pub use worth_query_host::facade::{
    application_contribution::*, declaration::application_program::*, primary_graph::*,
};
#[path = "derived/collection.rs"]
pub mod collection;
#[path = "derived/court.rs"]
pub mod court;
#[path = "derived/custom_owner.rs"]
pub mod custom_owner;
#[path = "derived/handler.rs"]
pub mod handler;
#[path = "derived/operation.rs"]
pub mod operation;
#[path = "derived/rule.rs"]
pub mod rule;
pub use collection::{Artifact, CountCollection, CountComputation};
pub use handler::Handler as CountHandler;
pub use operation::{CountBinding, CountTwoSteps};
pub use operation::{Input as CountInput, Intent as CountIntent};

#[derive(Clone, Debug)]
pub struct Derived;
impl LawPolicy for Derived {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    const DERIVED_RELATIONS: bool = true;
}
#[derive(Clone, Debug)]
pub struct CustomDerivedOwner;
impl LawPolicy for CustomDerivedOwner {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    const DERIVED_RELATIONS: bool = true;
    const CUSTOM_DERIVED_OWNER: bool = true;
}
pub struct TwoSteps;
impl TwoSteps {
    pub fn reference<L: LawPolicy>(
    ) -> ApplicationDerivedRelationRef<GraphSchema<L>, Self, Node, Node> {
        ApplicationDerivedRelationRef::from_schema_identifiers("TwoSteps", "Node", "Node")
    }
}
impl OperationReads<CountTwoSteps> for TwoSteps {}

pub fn program_rules<L: LawPolicy>() -> Vec<ApplicationProgramRuleDeclaration> {
    vec![
        ApplicationLocalRuleRef::<GraphSchema<L>, GraphFeature, rule::Safety>::new()
            .declaration(ApplicationInvariantExecutionPoint::CommitBoundary),
    ]
}

pub fn declare<L: LawPolicy>(
    schema: ApplicationSchemaDeclarationBuilder<GraphSchema<L>>,
) -> ApplicationSchemaDeclarationBuilder<GraphSchema<L>> {
    schema
        .derived_relation(
            TwoSteps::reference(),
            ApplicationDerivedRelation::compose(Precedes::reference(), Precedes::reference())
                .without_self_edges(),
            ApplicationRelationGraphLaws::unbounded(),
        )
        .operation(
            CountTwoSteps::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(CountTwoSteps::reference(), NodeKey::reference())
        .operation_read_derived_relation(CountTwoSteps::reference(), TwoSteps::reference())
        .application_mutation_binding::<CountBinding<L>>()
        .rule_readable_collection::<GraphFeature, CountCollection, _, _, _>(TwoSteps::reference())
        .invariant(rule::definition::<L>())
}
pub fn configure<L: LawPolicy>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, GraphSchema<L>>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    if L::CUSTOM_DERIVED_OWNER {
        let computation = setup.derived_collection::<GraphFeature, CountCollection, _>(
            custom_owner::Owner::<L>::new(),
        )?;
        setup.handler::<CountBinding<L>, _>(handler::Handler(computation))?;
    } else {
        let computation = setup
            .derived_relation_count::<GraphFeature, CountCollection, CountTwoSteps, _, _, _>(
                TwoSteps::reference(),
            )?;
        setup.handler::<CountBinding<L>, _>(handler::Handler(computation))?;
    }
    setup.invariant(
        rule::Safety::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        rule::resolve::<L>,
    )
}
