//! Both real postures freeze and consume the same logical source interpretations.
pub use super::*;
pub use std::collections::BTreeSet;
pub use worth_query_host::facade::application_invariants::*;
pub struct Safety;
impl<L: LawPolicy> ApplicationInvariantMarkerIdentity<GraphSchema<L>> for Safety {
    const IDENTIFIER: &'static str = "logical-use-safety";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub fn definition<L: LawPolicy>() -> ApplicationInvariantDefinition<GraphSchema<L>, Safety> {
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
pub type Entity<L> =
    WorthQueryApplicationInvariantEntity<GraphSchema<L>, GraphNode<GraphSchema<L>>>;
pub type Member<L> = WorthQueryApplicationInvariantGraphMember<GraphSchema<L>, Uses>;
pub type Coordinate = (EntityId, String);
#[derive(Debug, PartialEq)]
pub struct Answer {
    pub members: BTreeSet<Coordinate>,
    pub identity: [u8; 32],
}
pub struct Root<L: LawPolicy> {
    pub proposed: Entity<L>,
    pub proposed_answer: Answer,
    pub committed: Option<(Entity<L>, Answer)>,
}
pub struct Rule<L: LawPolicy> {
    pub node:
        WorthQueryApplicationInvariantEntityBinding<GraphSchema<L>, GraphNode<GraphSchema<L>>>,
    pub physical: WorthQueryApplicationInvariantRelationBinding<
        GraphSchema<L>,
        GraphPrecedes<GraphSchema<L>>,
        GraphNode<GraphSchema<L>>,
        GraphNode<GraphSchema<L>>,
    >,
    pub negative: WorthQueryApplicationInvariantProjectedCaseBinding<
        GraphSchema<L>,
        Uses,
        GraphNode<GraphSchema<L>>,
        Negative,
    >,
    pub roots: WorthQueryApplicationInvariantDerivedRelationBinding<
        GraphSchema<L>,
        RootUses,
        GraphNode<GraphSchema<L>>,
        Uses,
    >,
    pub source: WorthQueryApplicationInvariantDerivedRelationBinding<
        GraphSchema<L>,
        UseSuccessor,
        Uses,
        Uses,
    >,
    pub set: WorthQueryApplicationInvariantComponentSetBinding<GraphSchema<L>, UseComponents, Uses>,
}
pub fn resolve<L: LawPolicy>(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, GraphSchema<L>>,
) -> Result<Rule<L>, String> {
    Ok(Rule {
        node: resolver
            .typed_entity(node::<GraphSchema<L>>())
            .ok_or("Node")?,
        physical: resolver
            .typed_relation(precedes::<GraphSchema<L>>())
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
pub fn coordinate<L: LawPolicy>(member: &Member<L>) -> Coordinate {
    assert!(member.as_entity().is_none());
    (member.carrier_id(), member.case().unwrap().to_owned())
}
impl<L: LawPolicy> Rule<L> {
    pub fn inspect(
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
    pub fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, GraphSchema<L>>,
    ) -> Result<Vec<Root<L>>, WorthQueryApplicationInvariantError> {
        let proposed = planner.proposed();
        let committed = planner.committed();
        proposed.graph_components(&self.set).map_err(preparation)?;
        committed.graph_components(&self.set).map_err(preparation)?;
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
        Ok(roots)
    }
    pub fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, GraphSchema<L>>,
        roots: &[Root<L>],
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        let proposed = context.proposed();
        let committed = context.committed();
        proposed.graph_components(&self.set).map_err(execution)?;
        committed.graph_components(&self.set).map_err(execution)?;
        for root in roots {
            context.check_live().map_err(execution)?;
            assert_eq!(
                self.inspect(&proposed, &root.proposed).map_err(execution)?,
                root.proposed_answer
            );
            if let Some((old, answer)) = &root.committed {
                assert_eq!(&self.inspect(&committed, old).map_err(execution)?, answer);
            }
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
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantError> {
        self.prepare(planner)
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, GraphSchema<L>>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantError>> {
        if !budget.checkpoint(1) {
            return Some(Err(WorthQueryApplicationInvariantError::Stopped));
        }
        Some(self.prepare(planner))
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, GraphSchema<L>>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        self.judge(context, scope)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, GraphSchema<L>>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>,
    > {
        if !budget.checkpoint(1) {
            return Some(Err(WorthQueryApplicationInvariantError::Stopped));
        }
        Some(self.judge(context, scope))
    }
}
pub fn preparation(
    cause: impl Into<WorthQueryApplicationInvariantError>,
) -> WorthQueryApplicationInvariantError {
    cause.into()
}
pub fn execution(
    cause: impl Into<WorthQueryApplicationInvariantError>,
) -> WorthQueryApplicationInvariantError {
    cause.into()
}
