use super::case_inventory::inventory_match_digest;
use super::{
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadAccessCaseRegistry,
    WorthQueryGraphReadAccessDenial, WorthQueryGraphReadAccessInventoryMatch,
};
use crate::identity::hash_parts;
use crate::runtime::{
    WorthQueryGraphIndexInventory, WorthQueryGraphIndexInventoryMatchReport,
    WorthQueryGraphReadAccessAuthorityReceipt, WorthQueryGraphReadAccessRequirementSet,
    WorthQueryGraphReadFamilyIndexContract, WorthQueryPersistentGraphIndexRequirementDeclaration,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessAdmission {
    digest: String,
    requirement_set: WorthQueryGraphReadAccessRequirementSet,
    case_registry: WorthQueryGraphReadAccessCaseRegistry,
    graph_index_inventory: WorthQueryGraphIndexInventory,
    graph_index_inventory_match_report: WorthQueryGraphIndexInventoryMatchReport,
    graph_read_family_index_contract: WorthQueryGraphReadFamilyIndexContract,
    authority_receipt: WorthQueryGraphReadAccessAuthorityReceipt,
    persistent_index_requirement: Option<WorthQueryPersistentGraphIndexRequirementDeclaration>,
    inventory_matches: Vec<WorthQueryGraphReadAccessInventoryMatch>,
    posture: WorthQueryGraphReadAccessAdmissionPosture,
    denial: Option<WorthQueryGraphReadAccessDenial>,
}

impl WorthQueryGraphReadAccessAdmission {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn requirement_set(&self) -> &WorthQueryGraphReadAccessRequirementSet {
        &self.requirement_set
    }

    pub fn case_registry(&self) -> &WorthQueryGraphReadAccessCaseRegistry {
        &self.case_registry
    }

    pub fn graph_index_inventory(&self) -> &WorthQueryGraphIndexInventory {
        &self.graph_index_inventory
    }

    pub fn graph_index_inventory_match_report(&self) -> &WorthQueryGraphIndexInventoryMatchReport {
        &self.graph_index_inventory_match_report
    }

    pub fn persistent_index_requirement(
        &self,
    ) -> Option<&WorthQueryPersistentGraphIndexRequirementDeclaration> {
        self.persistent_index_requirement.as_ref()
    }

    pub fn graph_read_family_index_contract(&self) -> &WorthQueryGraphReadFamilyIndexContract {
        &self.graph_read_family_index_contract
    }

    pub fn authority_receipt(&self) -> &WorthQueryGraphReadAccessAuthorityReceipt {
        &self.authority_receipt
    }

    pub fn inventory_matches(&self) -> &[WorthQueryGraphReadAccessInventoryMatch] {
        &self.inventory_matches
    }

    pub fn posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        &self.posture
    }

    pub fn denial(&self) -> Option<&WorthQueryGraphReadAccessDenial> {
        self.denial.as_ref()
    }

    pub fn is_admitted(&self) -> bool {
        self.denial.is_none()
    }

    pub(crate) fn admitted_in_authority(
        requirement_set: WorthQueryGraphReadAccessRequirementSet,
        case_registry: WorthQueryGraphReadAccessCaseRegistry,
        graph_index_inventory: WorthQueryGraphIndexInventory,
        graph_index_inventory_match_report: WorthQueryGraphIndexInventoryMatchReport,
        authority_receipt: WorthQueryGraphReadAccessAuthorityReceipt,
        posture: WorthQueryGraphReadAccessAdmissionPosture,
    ) -> Self {
        Self::new(
            requirement_set,
            case_registry,
            graph_index_inventory,
            graph_index_inventory_match_report,
            authority_receipt,
            posture,
            None,
        )
    }

    pub(crate) fn denied_in_authority(
        requirement_set: WorthQueryGraphReadAccessRequirementSet,
        case_registry: WorthQueryGraphReadAccessCaseRegistry,
        graph_index_inventory: WorthQueryGraphIndexInventory,
        graph_index_inventory_match_report: WorthQueryGraphIndexInventoryMatchReport,
        authority_receipt: WorthQueryGraphReadAccessAuthorityReceipt,
        denial: WorthQueryGraphReadAccessDenial,
    ) -> Self {
        Self::new(
            requirement_set,
            case_registry,
            graph_index_inventory,
            graph_index_inventory_match_report,
            authority_receipt,
            WorthQueryGraphReadAccessAdmissionPosture::Denied,
            Some(denial),
        )
    }

    fn new(
        requirement_set: WorthQueryGraphReadAccessRequirementSet,
        case_registry: WorthQueryGraphReadAccessCaseRegistry,
        graph_index_inventory: WorthQueryGraphIndexInventory,
        graph_index_inventory_match_report: WorthQueryGraphIndexInventoryMatchReport,
        authority_receipt: WorthQueryGraphReadAccessAuthorityReceipt,
        posture: WorthQueryGraphReadAccessAdmissionPosture,
        denial: Option<WorthQueryGraphReadAccessDenial>,
    ) -> Self {
        let inventory_matches =
            WorthQueryGraphReadAccessInventoryMatch::from_graph_index_match_report(
                &graph_index_inventory_match_report,
            );
        let persistent_index_requirement_is_required = posture
            == WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired
            || denial.as_ref().is_some_and(|denial| {
                denial.suggested_posture()
                    == &WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired
            });
        let persistent_index_requirement = persistent_index_requirement_is_required
            .then(|| {
                WorthQueryPersistentGraphIndexRequirementDeclaration::from_admission_parts(
                    &requirement_set,
                    &graph_index_inventory_match_report,
                )
            })
            .flatten();
        let graph_read_family_index_contract =
            WorthQueryGraphReadFamilyIndexContract::from_admission_parts(
                &requirement_set,
                persistent_index_requirement.as_ref(),
            );
        let inventory_match_digest = inventory_match_digest(&inventory_matches);
        let digest = hash_parts(&[
            "worth_query_graph_read_access_admission_v2".to_string(),
            format!(
                "requirements:{}",
                requirement_set.digest().render_support_hex()
            ),
            format!("case_registry:{}", case_registry.digest()),
            format!("graph_index_inventory:{}", graph_index_inventory.digest()),
            format!(
                "graph_index_inventory_match_report:{}",
                graph_index_inventory_match_report.digest()
            ),
            format!("inventory_match:{inventory_match_digest}"),
            format!(
                "family_index_contract:{}",
                graph_read_family_index_contract.digest()
            ),
            format!("authority_receipt:{}", authority_receipt.digest()),
            format!(
                "persistent_index_requirement:{}",
                persistent_index_requirement
                    .as_ref()
                    .map(WorthQueryPersistentGraphIndexRequirementDeclaration::digest)
                    .unwrap_or("none")
            ),
            posture.digest_part(),
            denial
                .as_ref()
                .map(WorthQueryGraphReadAccessDenial::digest_part)
                .unwrap_or_else(|| "denial:none".to_string()),
        ]);
        Self {
            digest,
            requirement_set,
            case_registry,
            graph_index_inventory,
            graph_index_inventory_match_report,
            graph_read_family_index_contract,
            authority_receipt,
            persistent_index_requirement,
            inventory_matches,
            posture,
            denial,
        }
    }
}
