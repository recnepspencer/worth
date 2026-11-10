use crate::canonicalization::CanonicalQueryBundle;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::identity::{hash_parts, CanonicalQueryDigest, CanonicalResultShapeDigest};
use crate::planning::ExecutionPlanBundle;
use crate::validation::ValidatedQueryBundle;

use super::admission::AdmittedViewShape;
use super::delivery::{
    ViewShapeDeliveryMetadata, ViewShapeInvalidationPosture, ViewShapePatchPosture,
};
use super::digest::ViewShapeDigest;
use super::error::{ViewShapeError, ViewShapeFailureClass};
use super::family::ViewShapeFamily;
use super::grouped_maintenance::ViewShapeMaintenanceContract;
use super::performance::ViewShapeComplexityReport;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ViewShapePlanDigest(String);

impl ViewShapePlanDigest {
    pub(crate) fn from_parts(parts: &[String]) -> Self {
        Self(hash_parts(parts))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn evidence_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        worth_query_evidence_identity(WorthQueryEvidenceScope::ViewShapePlanDigest)
            .field_value(WorthQueryEvidenceTag::new("plan_digest"), self.as_str())
            .seal()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewShapeValidatedBundle {
    canonical: CanonicalQueryBundle,
    admitted: AdmittedViewShape,
    validated: ValidatedQueryBundle,
}

impl ViewShapeValidatedBundle {
    pub(crate) fn new(
        canonical: CanonicalQueryBundle,
        admitted: AdmittedViewShape,
        validated: ValidatedQueryBundle,
    ) -> Self {
        Self {
            canonical,
            admitted,
            validated,
        }
    }

    pub fn canonical(&self) -> &CanonicalQueryBundle {
        &self.canonical
    }

    pub fn admitted(&self) -> &AdmittedViewShape {
        &self.admitted
    }

    pub fn validated(&self) -> &ValidatedQueryBundle {
        &self.validated
    }

    pub fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        self.canonical.query().digest()
    }

    pub fn canonical_result_shape_digest(&self) -> &CanonicalResultShapeDigest {
        self.canonical.result_shape().digest()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewShapePlanArtifact {
    validated: ViewShapeValidatedBundle,
    execution_plan: ExecutionPlanBundle,
    view_plan_digest: ViewShapePlanDigest,
    delivery_metadata: ViewShapeDeliveryMetadata,
    invalidation_posture: ViewShapeInvalidationPosture,
    patch_posture: ViewShapePatchPosture,
    complexity: ViewShapeComplexityReport,
    maintenance_contract: ViewShapeMaintenanceContract,
}

impl ViewShapePlanArtifact {
    pub(crate) fn new(
        validated: ViewShapeValidatedBundle,
        execution_plan: ExecutionPlanBundle,
        delivery_metadata: ViewShapeDeliveryMetadata,
        invalidation_posture: ViewShapeInvalidationPosture,
        patch_posture: ViewShapePatchPosture,
        complexity: ViewShapeComplexityReport,
        maintenance_contract: ViewShapeMaintenanceContract,
    ) -> Result<Self, ViewShapeError> {
        match (validated.admitted().family(), &maintenance_contract) {
            (
                ViewShapeFamily::KanbanGrouped,
                ViewShapeMaintenanceContract::KanbanGrouped { .. },
            ) if delivery_metadata.grouped_delivery() => {}
            (ViewShapeFamily::KanbanGrouped, _) => {
                return Err(ViewShapeError::new(
                    ViewShapeFailureClass::PlanningInvariantRejected,
                    "kanban grouped plans must carry a grouped maintenance contract",
                ));
            }
            (_, ViewShapeMaintenanceContract::Ungrouped)
                if !delivery_metadata.grouped_delivery() => {}
            (_, _) => {
                return Err(ViewShapeError::new(
                    ViewShapeFailureClass::PlanningInvariantRejected,
                    "non-grouped plans may not carry grouped maintenance contracts",
                ));
            }
        }

        let view_plan_digest = ViewShapePlanDigest::from_parts(&[
            format!("view_shape:{}", validated.admitted().digest().as_str()),
            format!(
                "canonical_query:{}",
                validated.canonical_query_digest().as_str()
            ),
            format!(
                "canonical_result_shape:{}",
                validated.canonical_result_shape_digest().as_str()
            ),
            format!(
                "validated_query:{}",
                validated.validated().query().digest().as_str()
            ),
            format!(
                "validated_result_shape:{}",
                validated.validated().result_shape().digest().as_str()
            ),
            format!(
                "execution_plan:{}",
                execution_plan.query().plan_digest().as_str()
            ),
            format!("invalidation:{}", invalidation_posture.as_str()),
            format!("patch:{}", patch_posture.as_str()),
            format!(
                "focus:{}",
                delivery_metadata
                    .native_focus_aspect_key()
                    .map(|key| key.as_str())
                    .unwrap_or("none")
            ),
            format!(
                "grouping:{}",
                delivery_metadata
                    .native_grouping_aspect_key()
                    .map(|key| key.as_str())
                    .unwrap_or("none")
            ),
            format!(
                "identity_consumption:{}",
                delivery_metadata.identity_consumption().digest().as_str()
            ),
            format!(
                "projection_matches_detail:{}",
                delivery_metadata.projection_legality_matches_detail()
            ),
            format!(
                "delivery_narrowed:{}",
                delivery_metadata.delivery_width_narrowed()
            ),
            format!("grouped_delivery:{}", delivery_metadata.grouped_delivery()),
            format!(
                "grouped_contract:{}",
                maintenance_contract
                    .grouped_delta_policy()
                    .map(|policy| policy.contract().as_str())
                    .unwrap_or("none")
            ),
            format!(
                "grouped_identity_binding_index:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning.identity_binding_index())
                    .unwrap_or(usize::MAX)
            ),
            format!(
                "grouped_identity_binding_field:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning
                        .identity_binding()
                        .native_binding_aspect_key()
                        .as_str())
                    .unwrap_or("none")
            ),
            format!(
                "grouped_grouping_binding_index:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning.grouping_binding_index())
                    .unwrap_or(usize::MAX)
            ),
            format!(
                "grouped_grouping_binding_field:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning
                        .grouping_binding()
                        .native_binding_aspect_key()
                        .as_str())
                    .unwrap_or("none")
            ),
            format!(
                "grouped_member_transition_bound:{}",
                maintenance_contract
                    .grouped_delta_policy()
                    .map(|policy| policy.max_member_transitions())
                    .unwrap_or(0)
            ),
            format!(
                "grouped_lane_reassignment_bound:{}",
                maintenance_contract
                    .grouped_delta_policy()
                    .map(|policy| policy.max_lane_reassignments())
                    .unwrap_or(0)
            ),
            format!(
                "grouped_binding_count:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning.grouped_binding_width())
                    .unwrap_or(0)
            ),
            format!(
                "grouped_projection_count:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning.grouped_projection_width())
                    .unwrap_or(0)
            ),
            format!(
                "grouped_traversal_count:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning.traversal_count())
                    .unwrap_or(0)
            ),
            format!(
                "grouped_ordering_count:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning.ordering_count())
                    .unwrap_or(0)
            ),
            format!(
                "grouped_predicate_count:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning.predicate_count())
                    .unwrap_or(0)
            ),
            format!(
                "grouped_fallback:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning.fallback().as_str())
                    .unwrap_or("none")
            ),
            format!(
                "grouped_materialization_width:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning.baseline_materialization().grouped_binding_width())
                    .unwrap_or(0)
            ),
            format!(
                "grouped_replay_commits_contract:{}",
                maintenance_contract
                    .grouped_planning()
                    .map(|planning| planning
                        .replay_delivery_posture()
                        .replay_commits_grouping_contract())
                    .unwrap_or(false)
            ),
            format!("cost:{}", complexity.cost_class().as_str()),
            format!("complexity:{}", complexity.status().as_str()),
            format!("fallback:{:?}", complexity.fallback()),
        ]);

        Ok(Self {
            validated,
            execution_plan,
            view_plan_digest,
            delivery_metadata,
            invalidation_posture,
            patch_posture,
            complexity,
            maintenance_contract,
        })
    }

    pub fn family(&self) -> ViewShapeFamily {
        self.validated.admitted().family()
    }

    pub fn admitted(&self) -> &AdmittedViewShape {
        self.validated.admitted()
    }

    pub fn view_shape_digest(&self) -> &ViewShapeDigest {
        self.validated.admitted().digest()
    }

    pub fn view_plan_digest(&self) -> &ViewShapePlanDigest {
        &self.view_plan_digest
    }

    pub fn validated_view(&self) -> &ViewShapeValidatedBundle {
        &self.validated
    }

    pub fn validated(&self) -> &ValidatedQueryBundle {
        self.validated.validated()
    }

    pub fn canonical(&self) -> &CanonicalQueryBundle {
        self.validated.canonical()
    }

    pub fn execution_plan(&self) -> &ExecutionPlanBundle {
        &self.execution_plan
    }

    pub fn delivery_metadata(&self) -> &ViewShapeDeliveryMetadata {
        &self.delivery_metadata
    }

    pub fn invalidation_posture(&self) -> &ViewShapeInvalidationPosture {
        &self.invalidation_posture
    }

    pub fn patch_posture(&self) -> &ViewShapePatchPosture {
        &self.patch_posture
    }

    pub fn complexity(&self) -> &ViewShapeComplexityReport {
        &self.complexity
    }

    pub fn maintenance_contract(&self) -> &ViewShapeMaintenanceContract {
        &self.maintenance_contract
    }

    pub fn grouped_delta_policy(
        &self,
    ) -> Option<&super::grouped_policy::GroupedDeltaAdmissionPolicy> {
        self.maintenance_contract.grouped_delta_policy()
    }

    pub fn grouped_planning_artifact(
        &self,
    ) -> Option<&super::grouped_planning::GroupedViewPlanningArtifact> {
        self.maintenance_contract.grouped_planning()
    }
}
