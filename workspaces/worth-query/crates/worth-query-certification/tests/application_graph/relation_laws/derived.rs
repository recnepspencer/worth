//! One authored derived recipe is shared by ordinary rules and a managed collection.
use super::{program::GraphFeature, schema::*};
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{application_program::*, application_schema::*},
    primary_graph::*,
};
#[path = "derived/collection.rs"]
mod collection;
#[path = "derived/court.rs"]
mod court;
#[path = "derived/custom_owner.rs"]
mod custom_owner;
#[path = "derived/handler.rs"]
mod handler;
#[path = "derived/operation.rs"]
mod operation;
#[path = "derived/rule.rs"]
mod rule;
pub(super) use collection::{Artifact, CountCollection, CountComputation};
pub(super) use handler::Handler as CountHandler;
pub(super) use operation::{CountBinding, CountTwoSteps};
pub(super) use operation::{Input as CountInput, Intent as CountIntent};

#[derive(Clone, Debug)]
pub(super) struct Derived;
impl LawPolicy for Derived {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    const DERIVED_RELATIONS: bool = true;
}
#[derive(Clone, Debug)]
pub(super) struct CustomDerivedOwner;
impl LawPolicy for CustomDerivedOwner {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    const DERIVED_RELATIONS: bool = true;
    const CUSTOM_DERIVED_OWNER: bool = true;
}
pub(super) struct TwoSteps;
impl TwoSteps {
    fn reference<L: LawPolicy>() -> ApplicationDerivedRelationRef<GraphSchema<L>, Self, Node, Node>
    {
        ApplicationDerivedRelationRef::from_schema_identifiers("TwoSteps", "Node", "Node")
    }
}
impl OperationReads<CountTwoSteps> for TwoSteps {}

pub(super) fn program_rules<L: LawPolicy>() -> Vec<ApplicationProgramRuleDeclaration> {
    vec![
        ApplicationLocalRuleRef::<GraphSchema<L>, GraphFeature, rule::Safety>::new()
            .declaration(ApplicationInvariantExecutionPoint::CommitBoundary),
    ]
}

pub(super) fn declare<L: LawPolicy>(
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
pub(super) fn configure<L: LawPolicy>(
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
