//! The same generation-bearing root has independent proposed and committed proofs.
pub use super::*;
pub use std::collections::BTreeMap;
pub use worth_query_host::facade::application_invariants::*;
pub struct Safety;
impl<L: LawPolicy> ApplicationInvariantMarkerIdentity<GraphSchema<L>> for Safety {
    const IDENTIFIER: &'static str = "derived-two-step-safety";
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
pub type Entity<L> = WorthQueryApplicationInvariantEntity<GraphSchema<L>, Node>;
pub struct Root<L: LawPolicy> {
    pub proposed: Entity<L>,
    pub committed: Option<Entity<L>>,
}
pub struct Rule<L: LawPolicy> {
    pub node: WorthQueryApplicationInvariantEntityBinding<GraphSchema<L>, Node>,
    pub key: WorthQueryApplicationInvariantFieldBinding<GraphSchema<L>, Node, u64>,
    pub physical:
        WorthQueryApplicationInvariantRelationBinding<GraphSchema<L>, Precedes, Node, Node>,
    pub derived:
        WorthQueryApplicationInvariantDerivedRelationBinding<GraphSchema<L>, TwoSteps, Node, Node>,
}
pub fn resolve<L: LawPolicy>(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, GraphSchema<L>>,
) -> Result<Rule<L>, String> {
    Ok(Rule {
        node: resolver
            .typed_entity(Node::reference())
            .ok_or("Node absent")?,
        key: resolver
            .typed_field(NodeKey::reference())
            .ok_or("NodeKey absent")?,
        physical: resolver
            .typed_relation(Precedes::reference())
            .ok_or("Precedes absent")?,
        derived: resolver
            .typed_derived_relation(TwoSteps::reference())
            .ok_or("TwoSteps closure absent")?,
    })
}
impl<L: LawPolicy> Rule<L> {
    pub fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, GraphSchema<L>>,
    ) -> Result<Vec<Root<L>>, WorthQueryApplicationInvariantPreparationError> {
        let proposed = planner.proposed();
        let committed = planner.committed();
        let mut roots = BTreeMap::new();
        // Changing U→V can affect U and each predecessor of U in this two-leg
        // expression. Explore both actual postures so removals retain old parents.
        for touched in proposed
            .touched_entities_of(&self.node)
            .map_err(preparation)?
        {
            roots.insert(touched.entity_id(), touched.clone());
            for edge in proposed
                .relations_to(&self.physical, &touched)
                .map_err(preparation)?
            {
                roots.insert(edge.from().entity_id(), edge.from().clone());
            }
            if let Some(old) = committed
                .reobserve_entity(&self.node, &touched)
                .map_err(preparation)?
            {
                for edge in committed
                    .relations_to(&self.physical, &old)
                    .map_err(preparation)?
                {
                    if let Some(now) = proposed
                        .reobserve_entity(&self.node, edge.from())
                        .map_err(preparation)?
                    {
                        roots.insert(now.entity_id(), now);
                    }
                }
            }
        }
        let mut result = Vec::new();
        for root in roots.into_values() {
            proposed
                .derived_relations_from(&self.derived, &root)
                .map_err(preparation)?;
            let old = committed
                .reobserve_entity(&self.node, &root)
                .map_err(preparation)?;
            if let Some(old) = &old {
                committed
                    .derived_relations_from(&self.derived, old)
                    .map_err(preparation)?;
            }
            result.push(Root {
                proposed: root,
                committed: old,
            });
        }
        Ok(result)
    }
    pub fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, GraphSchema<L>>,
        roots: &[Root<L>],
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        let proposed = context.proposed();
        let committed = context.committed();
        for root in roots {
            context
                .check_live()
                .map_err(|cause| execution(format!("{cause:?}")))?;
            if let Some(old) = &root.committed {
                // The real committed read must remain available alongside the
                // proposed proof; a shared (definition,root)-only key is false.
                committed
                    .derived_relations_from(&self.derived, old)
                    .map_err(|cause| execution(cause.to_string()))?;
            }
            let from = proposed
                .field(&self.key, &root.proposed)
                .map_err(|cause| execution(cause.to_string()))?;
            for edge in proposed
                .derived_relations_from(&self.derived, &root.proposed)
                .map_err(|cause| execution(cause.to_string()))?
            {
                let to = proposed
                    .field(&self.key, edge.to())
                    .map_err(|cause| execution(cause.to_string()))?;
                if from == Some(0) && to == Some(2) {
                    return Ok(WorthQueryApplicationInvariantVerdict::Violation(()));
                }
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
pub fn preparation(
    cause: impl std::fmt::Display,
) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(cause.to_string())
}
pub fn execution(cause: impl Into<String>) -> WorthQueryApplicationInvariantExecutionError {
    WorthQueryApplicationInvariantExecutionError::new(cause.into())
}
