//! Native contour eligibility; no box or coordinate proxy can certify closure.
use super::*;
use std::collections::BTreeMap;
use worth_query_host::facade::application_invariants::*;
mod preparation;

pub struct TriangleFaceContour;
impl<S: SceneFaceSchemaBinding> ApplicationInvariantMarkerIdentity<S> for TriangleFaceContour {
    const IDENTIFIER: &'static str = "TriangleFaceContour";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(super) fn definition<S: SceneFaceSchemaBinding>(
) -> ApplicationInvariantDefinition<S, TriangleFaceContour> {
    let closure = [
        ApplicationInvariantScopeTarget::Entity("Body".into()),
        ApplicationInvariantScopeTarget::Entity("Face".into()),
        ApplicationInvariantScopeTarget::Entity("Scene".into()),
        ApplicationInvariantScopeTarget::Relation("PlanarSuccessor".into()),
        ApplicationInvariantScopeTarget::Relation("FaceAnchor".into()),
        ApplicationInvariantScopeTarget::Relation("SceneFace".into()),
    ];
    ApplicationInvariantDefinition::new(
        TriangleFaceContour::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        // Original request and Native read controls own work refusal.
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [
                ApplicationInvariantGroup::SchemaCompliance,
                ApplicationInvariantGroup::RelationIntegrity,
            ],
            closure.clone(),
            closure,
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
type FaceEntity<S> = WorthQueryApplicationInvariantEntity<S, Face>;
pub struct TriangleFaceRule<S: SceneFaceSchemaBinding> {
    faces: WorthQueryApplicationInvariantEntityBinding<S, Face>,
    bodies: WorthQueryApplicationInvariantEntityBinding<S, Body>,
    anchor: WorthQueryApplicationInvariantRelationBinding<S, FaceAnchor, Face, Body>,
    successor: WorthQueryApplicationInvariantRelationBinding<S, PlanarSuccessor, Body, Body>,
}
pub(super) fn resolve<S: SceneFaceSchemaBinding>(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, S>,
) -> Result<TriangleFaceRule<S>, String> {
    Ok(TriangleFaceRule {
        faces: resolver
            .typed_entity(Face::reference())
            .ok_or("missing Face")?,
        bodies: resolver
            .typed_entity(Body::reference())
            .ok_or("missing Body")?,
        anchor: resolver
            .typed_relation(FaceAnchor::reference())
            .ok_or("missing FaceAnchor")?,
        successor: resolver
            .typed_relation(PlanarSuccessor::reference())
            .ok_or("missing PlanarSuccessor")?,
    })
}
impl<S: SceneFaceSchemaBinding> TriangleFaceRule<S> {
    fn contour_is_triangle(
        &self,
        view: &WorthQueryApplicationInvariantReadView<'_, S>,
        face: &FaceEntity<S>,
    ) -> Result<bool, WorthQueryInvariantAccessDenial> {
        let anchors = view.relations_from(&self.anchor, face)?;
        let [anchor] = anchors.as_slice() else {
            return Ok(false);
        };
        let a = anchor.to();
        let first = view.relations_from(&self.successor, a)?;
        let [first] = first.as_slice() else {
            return Ok(false);
        };
        let b = first.to();
        let second = view.relations_from(&self.successor, b)?;
        let [second] = second.as_slice() else {
            return Ok(false);
        };
        let c = second.to();
        let third = view.relations_from(&self.successor, c)?;
        let [third] = third.as_slice() else {
            return Ok(false);
        };
        Ok(a.entity_id() != b.entity_id()
            && b.entity_id() != c.entity_id()
            && c.entity_id() != a.entity_id()
            && third.to().entity_id() == a.entity_id())
    }
    fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, S>,
        scope: &[FaceEntity<S>],
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        if budget.is_some_and(|b| !b.checkpoint(0)) {
            return Err(execution("face evaluation stopped"));
        }
        let proposed = context.proposed();
        for face in scope {
            if budget.is_some_and(|b| !b.checkpoint(1)) {
                return Err(execution("face evaluation stopped"));
            }
            if !self
                .contour_is_triangle(&proposed, face)
                .map_err(execution)?
            {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation(()));
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
fn execution(cause: impl std::fmt::Display) -> WorthQueryApplicationInvariantExecutionError {
    WorthQueryApplicationInvariantExecutionError::new(cause.to_string())
}
impl<S: SceneFaceSchemaBinding> WorthQueryApplicationInvariantRule<S> for TriangleFaceRule<S> {
    type Scope = Vec<FaceEntity<S>>;
    type Refusal = ();
    type RefusalBinding = WorthQueryEmptyInvariantRefusal;
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, S>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.prepare(planner, None)
    }
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, S>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        Some(self.prepare(planner, Some(budget)))
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, S>,
        scope: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        self.judge(context, scope, None)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, S>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<()>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        Some(self.judge(context, scope, Some(budget)))
    }
}
