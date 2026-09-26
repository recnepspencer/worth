use super::UiConstraintPropagationEdgeFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiConstraintPropagationDenialReason {
    FamilyNotAllowed,
    DuplicateEdgeAuthority,
    MissingRequiredDownwardConstraint,
    MissingRequiredIntrinsicContribution,
    MissingRequiredViewportPlanningInput,
    MissingRequiredScrollOwnerPlanningInput,
    MissingRequiredPortalAnchorPlanningInput,
    MissingRequiredSpecialInput,
    IncompatibleMeasurementPosture,
    UnsupportedCycleConvergence,
    UnsupportedSiblingFixedPoint,
    ContradictorySiblingRequirements,
    ContradictoryEqualShareRequirements,
    /// More peers share an extent than remainder ranks can order.
    EqualShareRankExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiConstraintPropagationDenial {
    reason: UiConstraintPropagationDenialReason,
    neighborhood_identity_digest: u64,
    contract_identity_digest: u64,
    family: Option<UiConstraintPropagationEdgeFamily>,
    witness_digest: u64,
}

impl UiConstraintPropagationDenial {
    pub(crate) fn new(
        reason: UiConstraintPropagationDenialReason,
        neighborhood_identity_digest: u64,
        contract_identity_digest: u64,
        family: Option<UiConstraintPropagationEdgeFamily>,
        witness_digest: u64,
    ) -> Self {
        Self {
            reason,
            neighborhood_identity_digest,
            contract_identity_digest,
            family,
            witness_digest,
        }
    }

    pub fn reason(&self) -> UiConstraintPropagationDenialReason {
        self.reason
    }

    pub fn neighborhood_identity_digest(&self) -> u64 {
        self.neighborhood_identity_digest
    }

    pub fn contract_identity_digest(&self) -> u64 {
        self.contract_identity_digest
    }

    pub fn family(&self) -> Option<UiConstraintPropagationEdgeFamily> {
        self.family
    }

    pub fn witness_digest(&self) -> u64 {
        self.witness_digest
    }
}
