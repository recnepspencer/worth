use super::{
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadRequiredCapabilityOwner,
};
use crate::identity::hash_parts;
use crate::runtime::{
    WorthQueryGraphIndexInventoryMatchReport, WorthQueryGraphReadAccessRequirementKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessCase {
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    inline_posture: WorthQueryGraphReadAccessAdmissionPosture,
    required_capability_owner: WorthQueryGraphReadRequiredCapabilityOwner,
}

impl WorthQueryGraphReadAccessCase {
    pub fn requirement_kind(&self) -> &WorthQueryGraphReadAccessRequirementKind {
        &self.requirement_kind
    }

    pub fn inline_posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        &self.inline_posture
    }

    pub fn required_capability_owner(&self) -> &WorthQueryGraphReadRequiredCapabilityOwner {
        &self.required_capability_owner
    }

    pub(crate) fn for_requirement_kind(
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    ) -> Self {
        let (inline_posture, required_capability_owner) =
            access_case_posture_and_owner(&requirement_kind);
        Self {
            requirement_kind,
            inline_posture,
            required_capability_owner,
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        format!(
            "case:{}:{}:{}",
            self.requirement_kind.as_str(),
            self.inline_posture.as_str(),
            self.required_capability_owner.as_str()
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessCaseRegistry {
    digest: String,
    cases: Vec<WorthQueryGraphReadAccessCase>,
}

impl WorthQueryGraphReadAccessCaseRegistry {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn cases(&self) -> &[WorthQueryGraphReadAccessCase] {
        &self.cases
    }

    pub fn requirement_kinds(&self) -> Vec<WorthQueryGraphReadAccessRequirementKind> {
        self.cases
            .iter()
            .map(|case| case.requirement_kind().clone())
            .collect()
    }

    pub fn case_for_requirement_kind(
        &self,
        requirement_kind: &WorthQueryGraphReadAccessRequirementKind,
    ) -> Option<&WorthQueryGraphReadAccessCase> {
        self.cases
            .iter()
            .find(|case| case.requirement_kind() == requirement_kind)
    }

    pub(crate) fn exhaustive() -> Self {
        Self::from_cases(
            WorthQueryGraphReadAccessRequirementKind::all()
                .iter()
                .cloned()
                .map(WorthQueryGraphReadAccessCase::for_requirement_kind)
                .collect(),
        )
    }

    pub(crate) fn from_cases(cases: Vec<WorthQueryGraphReadAccessCase>) -> Self {
        let digest = hash_parts(
            &std::iter::once("worth_query_graph_read_access_case_registry_v1".to_string())
                .chain(cases.iter().map(WorthQueryGraphReadAccessCase::digest_part))
                .collect::<Vec<_>>(),
        );
        Self { digest, cases }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessInventoryMatch {
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    required_capability_owner: WorthQueryGraphReadRequiredCapabilityOwner,
    resolved_posture: WorthQueryGraphReadAccessAdmissionPosture,
}

impl WorthQueryGraphReadAccessInventoryMatch {
    pub fn requirement_kind(&self) -> &WorthQueryGraphReadAccessRequirementKind {
        &self.requirement_kind
    }

    pub fn required_capability_owner(&self) -> &WorthQueryGraphReadRequiredCapabilityOwner {
        &self.required_capability_owner
    }

    pub fn resolved_posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        &self.resolved_posture
    }

    pub(crate) fn from_graph_index_match_report(
        report: &WorthQueryGraphIndexInventoryMatchReport,
    ) -> Vec<Self> {
        report
            .matches()
            .iter()
            .map(|row| Self {
                requirement_kind: row.requirement_kind().clone(),
                required_capability_owner: row.required_capability_owner().clone(),
                resolved_posture: row.resolved_admission_posture().clone(),
            })
            .collect()
    }

    pub(crate) fn digest_part(&self) -> String {
        format!(
            "inventory_match:{}:{}:{}",
            self.requirement_kind.as_str(),
            self.required_capability_owner.as_str(),
            self.resolved_posture.as_str()
        )
    }
}

pub(crate) fn inventory_match_digest(
    inventory_matches: &[WorthQueryGraphReadAccessInventoryMatch],
) -> String {
    hash_parts(
        &std::iter::once("worth_query_graph_read_access_inventory_match_v1".to_string())
            .chain(
                inventory_matches
                    .iter()
                    .map(WorthQueryGraphReadAccessInventoryMatch::digest_part),
            )
            .collect::<Vec<_>>(),
    )
}

fn access_case_posture_and_owner(
    requirement_kind: &WorthQueryGraphReadAccessRequirementKind,
) -> (
    WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadRequiredCapabilityOwner,
) {
    match requirement_kind {
        WorthQueryGraphReadAccessRequirementKind::AggregateSelection
        | WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency
        | WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency
        | WorthQueryGraphReadAccessRequirementKind::TraversalWorkset
        | WorthQueryGraphReadAccessRequirementKind::VisitedSet
        | WorthQueryGraphReadAccessRequirementKind::DedupSet
        | WorthQueryGraphReadAccessRequirementKind::PredicateSupport
        | WorthQueryGraphReadAccessRequirementKind::OrderingSupport
        | WorthQueryGraphReadAccessRequirementKind::ProofSupport
        | WorthQueryGraphReadAccessRequirementKind::ResultBuffer
        | WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle => (
            WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed,
            WorthQueryGraphReadRequiredCapabilityOwner::QueryRuntime,
        ),
        WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport
        | WorthQueryGraphReadAccessRequirementKind::DomainOperationCapabilityRegistration => (
            WorthQueryGraphReadAccessAdmissionPosture::AccessCapabilityRegistrationRequired,
            WorthQueryGraphReadRequiredCapabilityOwner::DomainRegistration,
        ),
    }
}
