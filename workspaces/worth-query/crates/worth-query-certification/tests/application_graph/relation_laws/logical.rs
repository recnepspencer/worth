//! One finite member domain serves rules, decisions, components and managed counts.
use super::{derived, program::GraphFeature, schema::*};
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{application_program::*, application_schema::*},
    primary_graph::*,
};
#[path = "logical/committed.rs"]
mod committed;
#[path = "logical/court.rs"]
mod court;
#[path = "logical/handler.rs"]
mod handler;
#[path = "logical/locality.rs"]
mod locality;
#[path = "logical/observation.rs"]
mod observation;
#[path = "logical/operation.rs"]
mod operation;
#[path = "logical/rule.rs"]
mod rule;
pub(super) use operation::CheckBinding;

#[derive(Clone, Debug)]
pub(super) struct Logical;
impl LawPolicy for Logical {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().orbit();
    const LOGICAL_RELATIONS: bool = true;
}
pub(super) struct Uses;
pub(super) struct Negative;
pub(super) struct Positive;
impl Uses {
    fn reference<L: LawPolicy>() -> ApplicationProjectedMemberDomainRef<GraphSchema<L>, Self, Node>
    {
        ApplicationProjectedMemberDomainRef::new("Uses", Node::reference())
    }
}
impl Negative {
    fn reference<L: LawPolicy>() -> ApplicationProjectedMemberCaseRef<GraphSchema<L>, Uses, Self> {
        ApplicationProjectedMemberCaseRef::new(Uses::reference(), "Negative")
    }
}
impl Positive {
    fn reference<L: LawPolicy>() -> ApplicationProjectedMemberCaseRef<GraphSchema<L>, Uses, Self> {
        ApplicationProjectedMemberCaseRef::new(Uses::reference(), "Positive")
    }
}
pub(super) struct RootUses;
pub(super) struct NextUses;
pub(super) struct UseSuccessor;
pub(super) struct UseComponents;
impl RootUses {
    fn reference<L: LawPolicy>() -> ApplicationDerivedRelationRef<GraphSchema<L>, Self, Node, Uses>
    {
        ApplicationDerivedRelationRef::from_member_identifiers(
            "RootUses",
            ApplicationGraphMemberSelector::Entity("Node"),
            ApplicationGraphMemberSelector::Projected("Uses"),
        )
    }
}
impl NextUses {
    fn reference<L: LawPolicy>() -> ApplicationDerivedRelationRef<GraphSchema<L>, Self, Node, Uses>
    {
        ApplicationDerivedRelationRef::from_member_identifiers(
            "NextUses",
            ApplicationGraphMemberSelector::Entity("Node"),
            ApplicationGraphMemberSelector::Projected("Uses"),
        )
    }
}
impl UseSuccessor {
    fn reference<L: LawPolicy>() -> ApplicationDerivedRelationRef<GraphSchema<L>, Self, Uses, Uses>
    {
        ApplicationDerivedRelationRef::projected("UseSuccessor", Uses::reference())
    }
}
impl UseComponents {
    fn reference<L: LawPolicy>() -> ApplicationComponentSetRef<GraphSchema<L>, Self, Uses> {
        ApplicationComponentSetRef::projected("UseComponents", Uses::reference())
    }
}
impl OperationReads<derived::CountTwoSteps> for NextUses {}
impl OperationReads<operation::CheckUses> for RootUses {}
impl OperationReads<operation::CheckUses> for UseSuccessor {}
fn negative<L: LawPolicy>() -> ApplicationDerivedRelation<GraphSchema<L>, Node, Uses> {
    ApplicationDerivedRelation::carrier_case(Uses::reference(), Negative::reference()).unwrap()
}
fn positive<L: LawPolicy>() -> ApplicationDerivedRelation<GraphSchema<L>, Node, Uses> {
    ApplicationDerivedRelation::carrier_case(Uses::reference(), Positive::reference()).unwrap()
}
fn successor<L: LawPolicy>() -> ApplicationDerivedRelation<GraphSchema<L>, Uses, Uses> {
    negative::<L>()
        .reversed()
        .then(Precedes::reference())
        .then(positive::<L>())
}
pub(super) fn declare<L: LawPolicy>(
    schema: ApplicationSchemaDeclarationBuilder<GraphSchema<L>>,
) -> ApplicationSchemaDeclarationBuilder<GraphSchema<L>> {
    schema
        .projected_member_domain(
            ApplicationProjectedMemberDomainDefinition::new(Uses::reference())
                .case(Negative::reference())
                .unwrap()
                .case(Positive::reference())
                .unwrap(),
        )
        .derived_relation(
            RootUses::reference(),
            negative::<L>().union(positive::<L>()),
            ApplicationRelationGraphLaws::unbounded(),
        )
        .derived_relation(
            NextUses::reference(),
            ApplicationDerivedRelation::relation(Precedes::reference()).then(RootUses::reference()),
            ApplicationRelationGraphLaws::unbounded(),
        )
        // compiled-example: begin
        .derived_relation(
            UseSuccessor::reference(),
            successor::<L>().union(successor::<L>().reversed()),
            ApplicationRelationGraphLaws::unbounded().orbit(),
        )
        .derived_components(ApplicationDerivedComponentSetDefinition::new(
            UseComponents::reference(),
            UseSuccessor::reference(),
        ))
        // compiled-example: end
        .operation(
            derived::CountTwoSteps::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(derived::CountTwoSteps::reference(), NodeKey::reference())
        .operation_read_derived_relation(derived::CountTwoSteps::reference(), NextUses::reference())
        .application_mutation_binding::<derived::CountBinding<L>>()
        .rule_readable_collection::<GraphFeature, derived::CountCollection, _, _, _>(
            NextUses::reference(),
        )
        .operation(
            operation::CheckUses::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(operation::CheckUses::reference(), NodeKey::reference())
        .operation_read_derived_relation(operation::CheckUses::reference(), RootUses::reference())
        .operation_read_derived_relation(
            operation::CheckUses::reference(),
            UseSuccessor::reference(),
        )
        .application_mutation_binding::<CheckBinding<L>>()
        .invariant(rule::definition::<L>())
}
pub(super) fn program_rules<L: LawPolicy>() -> Vec<ApplicationProgramRuleDeclaration> {
    vec![
        ApplicationLocalRuleRef::<GraphSchema<L>, GraphFeature, rule::Safety>::new()
            .declaration(ApplicationInvariantExecutionPoint::CommitBoundary),
    ]
}
pub(super) fn configure<L: LawPolicy>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, GraphSchema<L>>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let count = setup.derived_relation_count::<GraphFeature, derived::CountCollection,
        derived::CountTwoSteps, _, _, _>(NextUses::reference())?;
    setup.handler::<derived::CountBinding<L>, _>(derived::CountHandler(count))?;
    setup.handler::<CheckBinding<L>, _>(handler::Handler)?;
    setup.invariant(
        rule::Safety::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        rule::resolve::<L>,
    )
}
