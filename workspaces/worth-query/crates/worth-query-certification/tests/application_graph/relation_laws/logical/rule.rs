//! Both real postures freeze and consume the same logical source interpretations.
use super::*;
use std::collections::BTreeSet;
use worth_query_host::facade::application_invariants::*;
pub(super) struct Safety;
impl<L: LawPolicy> ApplicationInvariantMarkerIdentity<GraphSchema<L>> for Safety {
    const IDENTIFIER: &'static str = "logical-use-safety";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(super) fn definition<L: LawPolicy>() -> ApplicationInvariantDefinition<GraphSchema<L>, Safety> {
    let closure = [
        ApplicationInvariantScopeTarget::Entity("Node".into()),
        ApplicationInvariantScopeTarget::Relation("Precedes".into()),
    ];
    ApplicationInvariantDefinition::new(
        Safety::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            closure.clone(),
            closure,
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
type Entity<L> = WorthQueryApplicationInvariantEntity<GraphSchema<L>, Node>;
type Member<L> = WorthQueryApplicationInvariantGraphMember<GraphSchema<L>, Uses>;
type Coordinate = (EntityId, String);
#[derive(Debug, PartialEq)]
pub(super) struct Answer {
    members: BTreeSet<Coordinate>,
    identity: [u8; 32],
}
pub(super) struct Root<L: LawPolicy> {
    proposed: Entity<L>,
    proposed_answer: Answer,
    committed: Option<(Entity<L>, Answer)>,
}
pub(super) struct Rule<L: LawPolicy> {
    observer: Option<observation::Observer>,
    node: WorthQueryApplicationInvariantEntityBinding<GraphSchema<L>, Node>,
    physical: WorthQueryApplicationInvariantRelationBinding<GraphSchema<L>, Precedes, Node, Node>,
    negative:
        WorthQueryApplicationInvariantProjectedCaseBinding<GraphSchema<L>, Uses, Node, Negative>,
    roots:
        WorthQueryApplicationInvariantDerivedRelationBinding<GraphSchema<L>, RootUses, Node, Uses>,
    source: WorthQueryApplicationInvariantDerivedRelationBinding<
        GraphSchema<L>,
        UseSuccessor,
        Uses,
        Uses,
    >,
    set: WorthQueryApplicationInvariantComponentSetBinding<GraphSchema<L>, UseComponents, Uses>,
}
pub(super) fn resolve<L: LawPolicy>(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, GraphSchema<L>>,
) -> Result<Rule<L>, String> {
    Ok(Rule {
        observer: L::logical_read_observer(),
        node: resolver.typed_entity(Node::reference()).ok_or("Node")?,
        physical: resolver
            .typed_relation(Precedes::reference())
            .ok_or("Precedes")?,
        negative: resolver
            .typed_projected_case(Uses::reference(), Negative::reference())
            .ok_or("Negative")?,
        roots: resolver
            .typed_derived_relation(RootUses::reference())
            .ok_or("RootUses")?,
        source: resolver
            .typed_derived_relation(UseSuccessor::reference())
            .ok_or("UseSuccessor")?,
        set: resolver
            .typed_component_set(UseComponents::reference())
            .ok_or("UseComponents")?,
    })
}
fn coordinate<L: LawPolicy>(member: &Member<L>) -> Coordinate {
    assert!(member.as_entity().is_none());
    (member.carrier_id(), member.case().unwrap().to_owned())
}
impl<L: LawPolicy> Rule<L> {
    fn inspect(
        &self,
        view: &WorthQueryApplicationInvariantReadView<'_, GraphSchema<L>>,
        root: &Entity<L>,
    ) -> Result<Answer, WorthQueryInvariantAccessDenial> {
        let entity = view.entity_member(root)?;
        let cases = view.derived_member_relations_from(&self.roots, &entity)?;
        assert_eq!(
            cases
                .iter()
                .map(|edge| coordinate(edge.to()))
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                (root.entity_id(), "Negative".into()),
                (root.entity_id(), "Positive".into())
            ])
        );
        let negative = view.projected_member(&self.negative, root)?;
        let physical = view.relations_from(&self.physical, root)?;
        assert_eq!(physical.len(), 1);
        let expected = BTreeSet::from([
            (root.entity_id(), "Negative".into()),
            (physical[0].to().entity_id(), "Positive".into()),
        ]);
        let adjacency = view.derived_member_relations_from(&self.source, &negative)?;
        assert_eq!(adjacency.len(), 1);
        assert_eq!(
            coordinate(adjacency[0].to()),
            (physical[0].to().entity_id(), "Positive".into())
        );
        // compiled-example: begin
        let walk = view.derived_walk(&self.source, RelationDirection::Forward, &negative)?;
        // compiled-example: end
        assert_eq!(
            walk.steps()
                .iter()
                .map(|step| coordinate(step.member()))
                .collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            walk.steps()
                .iter()
                .map(|step| step.distance())
                .collect::<Vec<_>>(),
            [0, 1]
        );
        let reverse = view.derived_walk(&self.source, RelationDirection::Reverse, &negative)?;
        assert_eq!(reverse.len(), 2);
        let component = view.graph_component_of(&self.set, &negative)?;
        let members = component
            .members()
            .iter()
            .map(coordinate)
            .collect::<BTreeSet<_>>();
        assert_eq!(members, expected);
        let orbit = view.derived_orbit(&self.source, &negative)?;
        assert_eq!(orbit.len(), 2);
        assert_eq!(
            orbit
                .members()
                .iter()
                .map(coordinate)
                .collect::<BTreeSet<_>>(),
            expected
        );
        // Orbit and explicit set have separate installed persistent definitions.
        Ok(Answer {
            members,
            identity: *component.identity().bytes(),
        })
    }
    fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, GraphSchema<L>>,
    ) -> Result<Vec<Root<L>>, WorthQueryApplicationInvariantPreparationError> {
        let proposed = planner.proposed();
        let committed = planner.committed();
        // The default Logical policy exercises census; LocalLogical measures scoped reads.
        if L::LOGICAL_CENSUS {
            proposed.graph_components(&self.set).map_err(preparation)?;
            committed.graph_components(&self.set).map_err(preparation)?;
        }
        let mut roots = Vec::new();
        for root in proposed
            .touched_entities_of(&self.node)
            .map_err(preparation)?
        {
            let proposed_answer = self.inspect(&proposed, &root).map_err(preparation)?;
            let old = committed
                .reobserve_entity(&self.node, &root)
                .map_err(preparation)?;
            let old = old
                .map(|old| self.inspect(&committed, &old).map(|answer| (old, answer)))
                .transpose()
                .map_err(preparation)?;
            roots.push(Root {
                proposed: root,
                proposed_answer,
                committed: old,
            });
        }
        if let Some(observer) = &self.observer {
            observer(false, planner.reads());
        }
        Ok(roots)
    }
    fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, GraphSchema<L>>,
        roots: &[Root<L>],
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        let proposed = context.proposed();
        let committed = context.committed();
        if L::LOGICAL_CENSUS {
            proposed.graph_components(&self.set).map_err(execution)?;
            committed.graph_components(&self.set).map_err(execution)?;
        }
        for root in roots {
            context
                .check_live()
                .map_err(|cause| execution(format!("{cause:?}")))?;
            assert_eq!(
                self.inspect(&proposed, &root.proposed).map_err(execution)?,
                root.proposed_answer
            );
            if let Some((old, answer)) = &root.committed {
                assert_eq!(&self.inspect(&committed, old).map_err(execution)?, answer);
            }
        }
        if let Some(observer) = &self.observer {
            observer(true, context.reads());
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
impl<L: LawPolicy> WorthQueryApplicationInvariantRule<GraphSchema<L>> for Rule<L> {
    type Scope = Vec<Root<L>>;
    type Refusal = ();
    type RefusalBinding = WorthQueryEmptyInvariantRefusal;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, GraphSchema<L>>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.prepare(planner)
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, GraphSchema<L>>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        if !budget.checkpoint(1) {
            return Some(Err(preparation("original preparation stopped")));
        }
        Some(self.prepare(planner))
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, GraphSchema<L>>,
        scope: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        self.judge(context, scope)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, GraphSchema<L>>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<()>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        if !budget.checkpoint(1) {
            return Some(Err(execution("original evaluation stopped")));
        }
        Some(self.judge(context, scope))
    }
}
fn preparation(cause: impl std::fmt::Display) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(cause.to_string())
}
fn execution(cause: impl std::fmt::Display) -> WorthQueryApplicationInvariantExecutionError {
    WorthQueryApplicationInvariantExecutionError::new(cause.to_string())
}
