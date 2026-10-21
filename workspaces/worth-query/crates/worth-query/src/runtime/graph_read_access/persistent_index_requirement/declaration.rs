use super::{
    WorthQueryPersistentGraphIndexRequirementCounters,
    WorthQueryPersistentGraphIndexRequirementReceipt, WorthQueryPersistentGraphIndexRequirementRow,
};
use crate::identity::hash_parts;
use crate::runtime::{
    WorthQueryGraphIndexInventoryMatchReport, WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadRequiredCapabilityOwner,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPersistentGraphIndexRequirementDeclaration {
    digest: String,
    read_graph_digest: String,
    access_shape_digest: String,
    selectivity_shape_digest: String,
    requirement_set_digest: String,
    inventory_match_report_digest: String,
    required_owner: WorthQueryGraphReadRequiredCapabilityOwner,
    requirement_rows: Vec<WorthQueryPersistentGraphIndexRequirementRow>,
    counters: WorthQueryPersistentGraphIndexRequirementCounters,
}

impl WorthQueryPersistentGraphIndexRequirementDeclaration {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn read_graph_digest(&self) -> &str {
        &self.read_graph_digest
    }

    pub fn access_shape_digest(&self) -> &str {
        &self.access_shape_digest
    }

    pub fn selectivity_shape_digest(&self) -> &str {
        &self.selectivity_shape_digest
    }

    pub fn requirement_set_digest(&self) -> &str {
        &self.requirement_set_digest
    }

    pub fn inventory_match_report_digest(&self) -> &str {
        &self.inventory_match_report_digest
    }

    pub fn required_owner(&self) -> &WorthQueryGraphReadRequiredCapabilityOwner {
        &self.required_owner
    }

    pub fn requirement_rows(&self) -> &[WorthQueryPersistentGraphIndexRequirementRow] {
        &self.requirement_rows
    }

    pub fn counters(&self) -> &WorthQueryPersistentGraphIndexRequirementCounters {
        &self.counters
    }

    pub fn receipt(&self) -> WorthQueryPersistentGraphIndexRequirementReceipt {
        WorthQueryPersistentGraphIndexRequirementReceipt::new(
            self.digest.clone(),
            self.counters.clone(),
        )
    }

    pub(crate) fn from_admission_parts(
        requirements: &WorthQueryGraphReadAccessRequirementSet,
        report: &WorthQueryGraphIndexInventoryMatchReport,
    ) -> Option<Self> {
        let requirement_rows = report
            .matches()
            .iter()
            .filter(|row| {
                row.resolved_admission_posture()
                    == &WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired
            })
            .map(WorthQueryPersistentGraphIndexRequirementRow::from_inventory_match)
            .collect::<Vec<_>>();
        if requirement_rows.is_empty() {
            return None;
        }
        let read_graph_digest = requirements.read_graph_digest().render_hex();
        let access_shape_digest = requirements.access_shape_digest().render_hex();
        let selectivity_shape_digest = requirements.selectivity_shape_digest().render_hex();
        let requirement_set_digest = requirements.digest().render_support_hex();
        let inventory_match_report_digest = report.digest().to_string();
        let required_owner = WorthQueryGraphReadRequiredCapabilityOwner::PersistentStore;
        let counters =
            WorthQueryPersistentGraphIndexRequirementCounters::new(requirement_rows.len());
        let digest = hash_parts(
            &[
                "worth_query_persistent_graph_index_requirement_declaration_v2".to_string(),
                format!("read_graph:{read_graph_digest}"),
                format!("access_shape:{access_shape_digest}"),
                format!("selectivity_shape:{selectivity_shape_digest}"),
                format!("requirements:{requirement_set_digest}"),
                format!("inventory_match_report:{inventory_match_report_digest}"),
                format!("required_owner:{}", required_owner.as_str()),
                counters.digest_part(),
            ]
            .into_iter()
            .chain(
                requirement_rows
                    .iter()
                    .map(WorthQueryPersistentGraphIndexRequirementRow::digest_part),
            )
            .collect::<Vec<_>>(),
        );
        Some(Self {
            digest,
            read_graph_digest,
            access_shape_digest,
            selectivity_shape_digest,
            requirement_set_digest,
            inventory_match_report_digest,
            required_owner,
            requirement_rows,
            counters,
        })
    }
}
