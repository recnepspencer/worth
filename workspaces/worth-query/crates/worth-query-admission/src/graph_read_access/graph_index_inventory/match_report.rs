use super::match_selection::{
    classify_inventory_match_outcome, select_best_support_row_for_requirement,
};
use super::{
    WorthQueryGraphIndexInventory, WorthQueryGraphIndexInventoryCounters,
    WorthQueryGraphIndexInventoryMatchOutcome, WorthQueryGraphIndexPosture,
    WorthQueryGraphIndexSupportRow, WorthQueryGraphIndexSupportState,
};
use crate::admission_digest::{hash_parts, hash_parts_with_digests};
use crate::graph_read_access::{
    WorthQueryGraphIndexLifecycleClass, WorthQueryGraphIndexLifecycleOwner,
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadAccessComplexityContract,
    WorthQueryGraphReadAccessInvalidationBasis, WorthQueryGraphReadAccessRebuildBasis,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadAccessRequirementRow,
    WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadRequiredCapabilityOwner,
};
use worth_foundational::facade::CanonicalDigestId;

mod admitted;
use crate::graph_read_access::digest_text::{
    admitted_digest_text, admitted_text_clone, AdmittedDigestTextStop,
};
use std::fmt::{self, Write};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphIndexInventoryMatch {
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    requirement_row_digest: String,
    requirement_semantic_slot: String,
    support_row_digest: String,
    support_posture: WorthQueryGraphIndexPosture,
    support_state: WorthQueryGraphIndexSupportState,
    lifecycle_owner: WorthQueryGraphIndexLifecycleOwner,
    lifecycle_class: WorthQueryGraphIndexLifecycleClass,
    rebuild_basis: WorthQueryGraphReadAccessRebuildBasis,
    invalidation_basis: WorthQueryGraphReadAccessInvalidationBasis,
    complexity_contract: WorthQueryGraphReadAccessComplexityContract,
    owning_milestone: Option<String>,
    outcome: WorthQueryGraphIndexInventoryMatchOutcome,
    required_capability_owner: WorthQueryGraphReadRequiredCapabilityOwner,
    resolved_admission_posture: WorthQueryGraphReadAccessAdmissionPosture,
}

impl WorthQueryGraphIndexInventoryMatch {
    pub fn requirement_kind(&self) -> &WorthQueryGraphReadAccessRequirementKind {
        &self.requirement_kind
    }

    pub fn requirement_row_digest(&self) -> &str {
        &self.requirement_row_digest
    }

    pub fn requirement_semantic_slot(&self) -> &str {
        &self.requirement_semantic_slot
    }

    pub fn support_row_digest(&self) -> &str {
        &self.support_row_digest
    }

    pub fn support_posture(&self) -> &WorthQueryGraphIndexPosture {
        &self.support_posture
    }

    pub fn support_state(&self) -> &WorthQueryGraphIndexSupportState {
        &self.support_state
    }

    pub fn lifecycle_owner(&self) -> &WorthQueryGraphIndexLifecycleOwner {
        &self.lifecycle_owner
    }

    pub fn lifecycle_class(&self) -> &WorthQueryGraphIndexLifecycleClass {
        &self.lifecycle_class
    }

    pub fn rebuild_basis(&self) -> &WorthQueryGraphReadAccessRebuildBasis {
        &self.rebuild_basis
    }

    pub fn invalidation_basis(&self) -> &WorthQueryGraphReadAccessInvalidationBasis {
        &self.invalidation_basis
    }

    pub fn complexity_contract(&self) -> &WorthQueryGraphReadAccessComplexityContract {
        &self.complexity_contract
    }

    pub fn owning_milestone(&self) -> Option<&str> {
        self.owning_milestone.as_deref()
    }

    pub fn outcome(&self) -> &WorthQueryGraphIndexInventoryMatchOutcome {
        &self.outcome
    }

    pub fn required_capability_owner(&self) -> &WorthQueryGraphReadRequiredCapabilityOwner {
        &self.required_capability_owner
    }

    pub fn resolved_admission_posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        &self.resolved_admission_posture
    }

    fn from_support_row(
        requirement: &WorthQueryGraphReadAccessRequirementRow,
        row: &WorthQueryGraphIndexSupportRow,
    ) -> Self {
        Self::from_support_row_with_text(
            requirement,
            row,
            requirement.digest_part(),
            requirement.semantic_slot_key(),
            row.digest().to_string(),
            row.owning_milestone().map(str::to_string),
        )
    }

    fn from_support_row_with_text(
        requirement: &WorthQueryGraphReadAccessRequirementRow,
        row: &WorthQueryGraphIndexSupportRow,
        requirement_row_digest: String,
        requirement_semantic_slot: String,
        support_row_digest: String,
        owning_milestone: Option<String>,
    ) -> Self {
        let outcome = classify_inventory_match_outcome(requirement, row);
        let (required_capability_owner, resolved_admission_posture) =
            if outcome == WorthQueryGraphIndexInventoryMatchOutcome::ExactMatch {
                resolved_exact_match_posture(row)
            } else {
                (
                    WorthQueryGraphReadRequiredCapabilityOwner::LowerRuntime,
                    WorthQueryGraphReadAccessAdmissionPosture::Denied,
                )
            };
        Self {
            requirement_kind: requirement.kind().clone(),
            requirement_row_digest,
            requirement_semantic_slot,
            support_row_digest,
            support_posture: row.posture().clone(),
            support_state: row.support_state().clone(),
            lifecycle_owner: row.lifecycle_owner().clone(),
            lifecycle_class: row.lifecycle_class().clone(),
            rebuild_basis: row.rebuild_basis().clone(),
            invalidation_basis: row.invalidation_basis().clone(),
            complexity_contract: row.complexity_contract().clone(),
            owning_milestone,
            outcome,
            required_capability_owner,
            resolved_admission_posture,
        }
    }

    fn missing_support_row(requirement: &WorthQueryGraphReadAccessRequirementRow) -> Self {
        let requirement_row_digest = requirement.digest_part();
        let support_row_digest = hash_parts(&[
            "worth_query_graph_index_missing_support_row_v1".to_string(),
            requirement_row_digest.clone(),
        ]);
        Self::missing_support_row_with_text(
            requirement,
            requirement_row_digest,
            requirement.semantic_slot_key(),
            support_row_digest,
        )
    }

    fn missing_support_row_with_text(
        requirement: &WorthQueryGraphReadAccessRequirementRow,
        requirement_row_digest: String,
        requirement_semantic_slot: String,
        support_row_digest: String,
    ) -> Self {
        Self {
            requirement_kind: requirement.kind().clone(),
            requirement_row_digest,
            requirement_semantic_slot,
            support_row_digest,
            support_posture: WorthQueryGraphIndexPosture::Denied,
            support_state: WorthQueryGraphIndexSupportState::Unsupported,
            lifecycle_owner: WorthQueryGraphIndexLifecycleOwner::LowerRuntime,
            lifecycle_class: WorthQueryGraphIndexLifecycleClass::Unsupported,
            rebuild_basis: requirement.rebuild_basis().clone(),
            invalidation_basis: requirement.invalidation_basis().clone(),
            complexity_contract: requirement.complexity_contract().clone(),
            owning_milestone: None,
            outcome: WorthQueryGraphIndexInventoryMatchOutcome::MissingSupportRow,
            required_capability_owner: WorthQueryGraphReadRequiredCapabilityOwner::LowerRuntime,
            resolved_admission_posture: WorthQueryGraphReadAccessAdmissionPosture::Denied,
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(
            output,
            "match:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.requirement_kind.as_str(),
            self.requirement_row_digest,
            self.requirement_semantic_slot,
            self.support_row_digest,
            self.support_posture.as_str(),
            self.support_state.as_str(),
            self.lifecycle_owner.as_str(),
            self.lifecycle_class.as_str(),
            self.rebuild_basis.as_str(),
            self.invalidation_basis.as_str(),
            self.complexity_contract.as_str(),
            self.owning_milestone.as_deref().unwrap_or("none"),
            self.outcome.as_str(),
            self.required_capability_owner.as_str(),
            self.resolved_admission_posture.as_str()
        )
    }
}

fn resolved_exact_match_posture(
    row: &WorthQueryGraphIndexSupportRow,
) -> (
    WorthQueryGraphReadRequiredCapabilityOwner,
    WorthQueryGraphReadAccessAdmissionPosture,
) {
    match row.posture() {
        WorthQueryGraphIndexPosture::Verified
        | WorthQueryGraphIndexPosture::RuntimeMaintained
        | WorthQueryGraphIndexPosture::LowerRuntimeOwned => (
            WorthQueryGraphReadRequiredCapabilityOwner::QueryRuntime,
            WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed,
        ),
        WorthQueryGraphIndexPosture::EphemeralAvailable => (
            WorthQueryGraphReadRequiredCapabilityOwner::QueryRuntime,
            WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex,
        ),
        WorthQueryGraphIndexPosture::RequiresAccessCapabilityRegistration => (
            WorthQueryGraphReadRequiredCapabilityOwner::DomainRegistration,
            WorthQueryGraphReadAccessAdmissionPosture::AccessCapabilityRegistrationRequired,
        ),
        WorthQueryGraphIndexPosture::RequiresStoreBackedPersistentIndex => (
            WorthQueryGraphReadRequiredCapabilityOwner::PersistentStore,
            WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired,
        ),
        WorthQueryGraphIndexPosture::TemporarilyUnavailable => (
            WorthQueryGraphReadRequiredCapabilityOwner::LowerRuntime,
            WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired,
        ),
        WorthQueryGraphIndexPosture::Denied => (
            WorthQueryGraphReadRequiredCapabilityOwner::LowerRuntime,
            WorthQueryGraphReadAccessAdmissionPosture::Denied,
        ),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphIndexInventoryMatchReport {
    digest: String,
    inventory_digest: String,
    requirement_set_digest: CanonicalDigestId,
    matches: Vec<WorthQueryGraphIndexInventoryMatch>,
    counters: WorthQueryGraphIndexInventoryCounters,
}

impl WorthQueryGraphIndexInventoryMatchReport {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn inventory_digest(&self) -> &str {
        &self.inventory_digest
    }

    pub const fn requirement_set_digest(&self) -> &CanonicalDigestId {
        &self.requirement_set_digest
    }

    pub fn matches(&self) -> &[WorthQueryGraphIndexInventoryMatch] {
        &self.matches
    }

    pub fn counters(&self) -> &WorthQueryGraphIndexInventoryCounters {
        &self.counters
    }

    pub fn includes_admission_posture(
        &self,
        posture: &WorthQueryGraphReadAccessAdmissionPosture,
    ) -> bool {
        self.matches
            .iter()
            .any(|row| row.resolved_admission_posture() == posture)
    }

    pub(crate) fn match_requirements(
        requirements: &WorthQueryGraphReadAccessRequirementSet,
        inventory: &WorthQueryGraphIndexInventory,
    ) -> Self {
        let matches = requirements
            .rows()
            .iter()
            .map(|requirement| {
                select_best_support_row_for_requirement(requirement, inventory).map_or_else(
                    || WorthQueryGraphIndexInventoryMatch::missing_support_row(requirement),
                    |row| WorthQueryGraphIndexInventoryMatch::from_support_row(requirement, row),
                )
            })
            .collect::<Vec<_>>();
        let unsupported_requirement_count = matches
            .iter()
            .filter(|row| {
                row.outcome() != &WorthQueryGraphIndexInventoryMatchOutcome::ExactMatch
                    || !row.support_posture().is_supported()
            })
            .count();
        let counters = WorthQueryGraphIndexInventoryCounters::new(
            inventory.rows().len(),
            requirements.rows().len(),
            matches.len(),
            unsupported_requirement_count,
            0,
        );
        let digest_parts = &[
            "worth_query_graph_index_inventory_match_report_v1".to_string(),
            format!("inventory:{}", inventory.digest()),
            counters.digest_part(),
        ]
        .into_iter()
        .chain(
            matches
                .iter()
                .map(WorthQueryGraphIndexInventoryMatch::digest_part),
        )
        .collect::<Vec<_>>();
        let digest = hash_parts_with_digests(digest_parts, &[requirements.digest().as_digest()]);
        Self {
            digest,
            inventory_digest: inventory.digest().to_string(),
            requirement_set_digest: *requirements.digest().as_digest(),
            matches,
            counters,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_read_access::{
        WorthQueryGraphReadAccessComplexityContract, WorthQueryGraphReadAccessInvalidationBasis,
        WorthQueryGraphReadAccessMemoryEstimateBasis, WorthQueryGraphReadAccessRebuildBasis,
        WorthQueryGraphReadAccessRequirementRow,
    };

    #[test]
    fn missing_inventory_row_remains_localized_in_match_report() {
        let requirement = WorthQueryGraphReadAccessRequirementRow::new(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
            WorthQueryGraphReadAccessRebuildBasis::AuthoritativeRelationTruth,
            WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeRelationDelta,
            WorthQueryGraphReadAccessComplexityContract::DirectionalRelationLookup,
            WorthQueryGraphReadAccessMemoryEstimateBasis::RelationDegreeBound,
        );
        let requirements = WorthQueryGraphReadAccessRequirementSet::new(
            CanonicalDigestId::new([1; 32]),
            CanonicalDigestId::new([2; 32]),
            CanonicalDigestId::new([3; 32]),
            vec![requirement],
            worth_foundational::facade::CanonicalDigestWorkBudget::new(16, 4096)
                .expect("the test canonical budget is nonzero"),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
        )
        .expect("the test requirement set fits its canonical budget");
        let inventory = WorthQueryGraphIndexInventory::from_rows(Vec::new());
        let report =
            WorthQueryGraphIndexInventoryMatchReport::match_requirements(&requirements, &inventory);

        assert_eq!(report.matches().len(), 1);
        assert_eq!(report.counters().matched_requirement_count(), 1);
        assert_eq!(report.counters().unsupported_requirement_count(), 1);
        assert_eq!(
            report.matches()[0].outcome(),
            &WorthQueryGraphIndexInventoryMatchOutcome::MissingSupportRow
        );
        assert_eq!(
            report.matches()[0].resolved_admission_posture(),
            &WorthQueryGraphReadAccessAdmissionPosture::Denied
        );
    }
}
