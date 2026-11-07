//! An active group's path must not reach a blocked member.
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantError;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::observation::Entity;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::observation::Observation;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use worth_relational::facade::runtime::CustomInvariantLeaseBudget;
use worth_relational::facade::identity::EntityId;
use worth_foundational::facade::RelationDirection;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantContext;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantFieldBinding;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantReadView;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantRelationBinding;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantRule;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantSchemaResolver;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantVerdict;
use worth_query_declaration::facade::application_schema::*;

#[derive(Clone, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
pub struct Refusal {
    pub root: u64,
    pub blocked: u64,
    pub distance: u64,
}
worth_query_declaration::worth_query_value_binding!(pub RefusalBinding for Refusal {
    identity: "tests.invariant-walk-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(128, 128, 8, 8),
});

pub struct WalkSafety;
impl ApplicationInvariantMarkerIdentity<WalkSchema> for WalkSafety {
    const IDENTIFIER: &'static str = "graph-walk-safety";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub fn definition() -> ApplicationInvariantDefinition<WalkSchema, WalkSafety> {
    ApplicationInvariantDefinition::new(
        WalkSafety::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [
                ApplicationInvariantScopeTarget::Entity("Node".into()),
                ApplicationInvariantScopeTarget::Relation("GroupRoot".into()),
                ApplicationInvariantScopeTarget::Relation("Next".into()),
            ],
            [
                ApplicationInvariantScopeTarget::Entity("Node".into()),
                ApplicationInvariantScopeTarget::Relation("GroupRoot".into()),
                ApplicationInvariantScopeTarget::Relation("Next".into()),
            ],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}

pub struct Root {
    pub entity: Entity,
    pub key: u64,
}

pub struct Rule {
    pub observation: Observation,
    pub key: WorthQueryApplicationInvariantFieldBinding<WalkSchema, Node, u64>,
    pub active: WorthQueryApplicationInvariantFieldBinding<WalkSchema, Node, bool>,
    pub blocked: WorthQueryApplicationInvariantFieldBinding<WalkSchema, Node, bool>,
    pub membership:
        WorthQueryApplicationInvariantRelationBinding<WalkSchema, GroupRoot, Node, Node>,
    pub next: WorthQueryApplicationInvariantRelationBinding<WalkSchema, Next, Node, Node>,
}

pub fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, WalkSchema>,
    observation: Observation,
) -> Result<Rule, String> {
    Ok(Rule {
        observation,
        key: resolver
            .typed_field(NodeKey::at("NodeKey"))
            .ok_or("node key missing")?,
        active: resolver
            .typed_field(Active::at("Active"))
            .ok_or("activation missing")?,
        blocked: resolver
            .typed_field(Blocked::at("Blocked"))
            .ok_or("blocked state missing")?,
        membership: resolver
            .typed_relation(GroupRoot::reference())
            .ok_or("group membership missing")?,
        next: resolver
            .typed_relation(Next::reference())
            .ok_or("path missing")?,
    })
}

impl WorthQueryApplicationInvariantRule<WalkSchema> for Rule {
    type Scope = Vec<Root>;
    type Refusal = Refusal;
    type RefusalBinding = RefusalBinding;

    fn supports_checked_preparation(&self) -> bool {
        true
    }

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WalkSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantError<Refusal>> {
        self.prepare(planner, None)
    }

    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WalkSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantError<Refusal>>> {
        Some(self.prepare(planner, Some(budget)))
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WalkSchema>,
        scope: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantError<Refusal>,
    > {
        self.evaluate_under(context, scope, None)
    }

    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WalkSchema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<Refusal>,
            WorthQueryApplicationInvariantError<Refusal>,
        >,
    > {
        Some(self.evaluate_under(context, scope, Some(budget)))
    }
}

#[path = "evaluation/mod.rs"]
pub mod evaluation;
#[path = "preparation/mod.rs"]
pub mod preparation;
