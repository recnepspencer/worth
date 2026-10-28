//! An active group's path must not reach a blocked member.
use super::{
    observation::{Entity, Observation},
    schema::*,
};
use crate::facade::application_invariants::*;
use worth_query_declaration::facade::application_schema::*;

mod evaluation;
mod preparation;

#[derive(Clone, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
pub(super) struct Refusal {
    pub(super) root: u64,
    pub(super) blocked: u64,
    pub(super) distance: u64,
}
worth_query_declaration::worth_query_value_binding!(pub(super) RefusalBinding for Refusal {
    identity: "tests.invariant-walk-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(128, 128, 8, 8),
});

pub(super) struct WalkSafety;
impl ApplicationInvariantMarkerIdentity<WalkSchema> for WalkSafety {
    const IDENTIFIER: &'static str = "graph-walk-safety";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub(super) fn definition() -> ApplicationInvariantDefinition<WalkSchema, WalkSafety> {
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

pub(super) struct Root {
    entity: Entity,
    key: u64,
}

pub(super) struct Rule {
    observation: Observation,
    key: WorthQueryApplicationInvariantFieldBinding<WalkSchema, Node, u64>,
    active: WorthQueryApplicationInvariantFieldBinding<WalkSchema, Node, bool>,
    blocked: WorthQueryApplicationInvariantFieldBinding<WalkSchema, Node, bool>,
    membership: WorthQueryApplicationInvariantRelationBinding<WalkSchema, GroupRoot, Node, Node>,
    next: WorthQueryApplicationInvariantRelationBinding<WalkSchema, Next, Node, Node>,
}

pub(super) fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, WalkSchema>,
    observation: Observation,
) -> Result<Rule, String> {
    Ok(Rule {
        observation,
        key: resolver
            .typed_field(NodeKey::reference())
            .ok_or("node key missing")?,
        active: resolver
            .typed_field(Active::reference())
            .ok_or("activation missing")?,
        blocked: resolver
            .typed_field(Blocked::reference())
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
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.prepare(planner, None)
    }

    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WalkSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        Some(self.prepare(planner, Some(budget)))
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WalkSchema>,
        scope: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantExecutionError,
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
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        Some(self.evaluate_under(context, scope, Some(budget)))
    }
}
