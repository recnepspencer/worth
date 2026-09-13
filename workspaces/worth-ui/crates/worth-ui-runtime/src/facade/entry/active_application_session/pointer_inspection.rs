use crate::runtime::interaction::UiPresentedInteractionTargetView;
use worth_ui_inspection::{
    UiAppearanceInspectionWorld, UiPointerAffordanceInspectionDecision as Decision,
    UiPointerAffordanceInspectionExpiry as Expiry,
    UiPointerAffordanceInspectionExplanation as Explanation,
    UiPointerAffordanceInspectionFamily as Family, UiPointerAffordanceInspectionOutcome as Outcome,
    UiPointerAffordanceInspectionPresentation as Presentation,
};

impl super::WorthUiActiveApplicationSession {
    /// Explains the sealed pointer decision for an exact presented target.
    /// Inspection neither observes Intent again nor admits pending presentation.
    pub fn why_pointer_affordance(
        &self,
        world: UiAppearanceInspectionWorld,
        target: UiPresentedInteractionTargetView,
    ) -> Outcome {
        let current_world = self.appearance_inspection_world(target.surface());
        if world.session_identity() != current_world.session_identity()
            || world.surface_identity() != current_world.surface_identity()
        {
            return Outcome::WrongWorld;
        }
        if world.evidence_generation() != current_world.evidence_generation() {
            return Outcome::Expired(Expiry::GenerationChanged);
        }
        if self.mounted.validate_binding(target.binding()).is_err() {
            return Outcome::Expired(Expiry::BindingChanged);
        }
        match self
            .mounted
            .current_semantic_surface_for_presentation(target.presentation())
        {
            Ok(surface) if surface == target.surface() => {}
            Ok(_) => return Outcome::WrongWorld,
            Err(crate::mounting::UiPresentedFrameBasisDenial::PresentationTruthUnavailable) => {
                return Outcome::Unavailable;
            }
            Err(_) => return Outcome::Expired(Expiry::PresentationChanged),
        }
        if crate::runtime::interaction::targeting::admit_current_target(&self.mounted, target)
            .is_err()
        {
            return Outcome::Expired(Expiry::TargetChanged);
        }
        if self
            .host_session
            .capability_report()
            .appearance_profile()
            .is_none()
        {
            return Outcome::Unsupported;
        }
        let Some(snapshot) = self.pointer_affordance_snapshot.as_ref() else {
            return Outcome::Unavailable;
        };
        let generation = self.active_generation_identity();
        if snapshot.generation() != &generation {
            return Outcome::Expired(Expiry::GenerationChanged);
        }
        let mut examined = 0;
        let projection = snapshot.projections().iter().find(|row| {
            examined += 1;
            row.surface() == target.surface() && row.target() == Some(target.mounted_instance())
        });
        let Some(projection) = projection else {
            return Outcome::Unavailable;
        };
        if snapshot.surface_is_invalidated(projection.surface()) {
            return Outcome::Expired(Expiry::SurfaceInvalidated);
        }
        if !self
            .mounted
            .pointer_projection_is_current(snapshot, projection)
        {
            return Outcome::Expired(Expiry::ObservationNotAdmitted);
        }
        let (graph_node_digest, route, decision) = match projection.operability() {
            Some(Ok(observation)) => {
                if observation.generation() != &generation
                    || projection.presented_target() != Some(observation.target())
                {
                    return Outcome::Expired(Expiry::ObservationNotAdmitted);
                }
                (
                    Some(observation.graph_node().digest()),
                    Some(observation.route().into()),
                    observation.inspection_decision(),
                )
            }
            Some(Err(denial)) => (None, None, Decision::Unavailable(denial.inspection())),
            None => return Outcome::Unavailable,
        };
        let pending = self.mounted.pointer_presentation_pending(
            target.surface(),
            target.binding(),
            Some(snapshot),
        );
        Outcome::Found(Explanation {
            world: current_world,
            pointer_identity: projection.pointer().value(),
            mounted_instance_identity: target.mounted_instance().diagnostic_value(),
            node_receipt_identity: target.node_receipt().diagnostic_value(),
            presentation_epoch: target.presentation().epoch().diagnostic_value(),
            source_basis: snapshot.source_basis(),
            observation_turn: snapshot.observation_turn(),
            graph_node_digest,
            route,
            family: match projection.family() {
                crate::declaration::UiPointerAffordance::Default => Family::Default,
                crate::declaration::UiPointerAffordance::Activation => Family::Activation,
            },
            support: worth_ui_inspection::UiAppearanceInspectionSupport::Supported,
            decision,
            presentation: if pending {
                Presentation::Pending
            } else {
                Presentation::Current
            },
            pointer_rows_examined: examined,
        })
    }
}
