use super::{
    match_graph_index_inventory_for_requirements, WorthQueryGraphIndexInventory,
    WorthQueryGraphIndexInventoryMatchReport, WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadPlanReviewDenial,
    WorthQueryGraphReadPlanReviewDenialKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadPlanReview {
    requirements: WorthQueryGraphReadAccessRequirementSet,

    inventory: WorthQueryGraphIndexInventory,
    inventory_match: WorthQueryGraphIndexInventoryMatchReport,
    posture: WorthQueryGraphReadAccessAdmissionPosture,
    denial: Option<WorthQueryGraphReadPlanReviewDenial>,
}

impl WorthQueryGraphReadPlanReview {
    pub fn requirements(&self) -> &WorthQueryGraphReadAccessRequirementSet {
        &self.requirements
    }

    pub fn inventory(&self) -> &WorthQueryGraphIndexInventory {
        &self.inventory
    }

    pub fn inventory_match(&self) -> &WorthQueryGraphIndexInventoryMatchReport {
        &self.inventory_match
    }

    pub fn posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        &self.posture
    }

    pub fn denial(&self) -> Option<&WorthQueryGraphReadPlanReviewDenial> {
        self.denial.as_ref()
    }

    pub const fn is_admitted(&self) -> bool {
        self.denial.is_none()
    }
}

#[doc(hidden)]
pub fn review_graph_read_access(
    requirements: WorthQueryGraphReadAccessRequirementSet,
    inventory: WorthQueryGraphIndexInventory,
) -> WorthQueryGraphReadPlanReview {
    let inventory_match = match_graph_index_inventory_for_requirements(&requirements, &inventory);
    let (posture, denial) = review_posture(&inventory_match);
    WorthQueryGraphReadPlanReview {
        requirements,

        inventory,
        inventory_match,
        posture,
        denial,
    }
}

fn review_posture(
    inventory: &WorthQueryGraphIndexInventoryMatchReport,
) -> (
    WorthQueryGraphReadAccessAdmissionPosture,
    Option<WorthQueryGraphReadPlanReviewDenial>,
) {
    for (posture, denial_kind) in [
        (
            WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired,
            WorthQueryGraphReadPlanReviewDenialKind::RequiredAsyncMaterialization,
        ),
        (
            WorthQueryGraphReadAccessAdmissionPosture::AccessCapabilityRegistrationRequired,
            WorthQueryGraphReadPlanReviewDenialKind::RequiredAccessCapabilityRegistration,
        ),
        (
            WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired,
            WorthQueryGraphReadPlanReviewDenialKind::RequiredPersistentIndex,
        ),
        (
            WorthQueryGraphReadAccessAdmissionPosture::Denied,
            WorthQueryGraphReadPlanReviewDenialKind::UnsupportedGraphIndexSupport,
        ),
    ] {
        if includes(inventory, posture) {
            return denied(denial_kind);
        }
    }
    if includes(
        inventory,
        WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex,
    ) {
        return (
            WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex,
            None,
        );
    }
    (
        WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed,
        None,
    )
}

fn denied(
    kind: WorthQueryGraphReadPlanReviewDenialKind,
) -> (
    WorthQueryGraphReadAccessAdmissionPosture,
    Option<WorthQueryGraphReadPlanReviewDenial>,
) {
    (
        WorthQueryGraphReadAccessAdmissionPosture::Denied,
        Some(WorthQueryGraphReadPlanReviewDenial::new(kind)),
    )
}

fn includes(
    report: &WorthQueryGraphIndexInventoryMatchReport,
    posture: WorthQueryGraphReadAccessAdmissionPosture,
) -> bool {
    report.includes_admission_posture(&posture)
}
