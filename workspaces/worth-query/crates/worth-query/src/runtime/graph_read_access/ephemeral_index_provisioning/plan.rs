use crate::identity::hash_parts;
use crate::runtime::{
    WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadAccessRequirementKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEphemeralGraphIndexAllocationRow {
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    requirement_row_digest: String,
}

impl WorthQueryEphemeralGraphIndexAllocationRow {
    pub fn requirement_kind(&self) -> &WorthQueryGraphReadAccessRequirementKind {
        &self.requirement_kind
    }

    pub fn requirement_row_digest(&self) -> &str {
        &self.requirement_row_digest
    }

    fn new(
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        requirement_row_digest: impl Into<String>,
    ) -> Self {
        Self {
            requirement_kind,
            requirement_row_digest: requirement_row_digest.into(),
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        format!(
            "allocation:{}:{}",
            self.requirement_kind.as_str(),
            self.requirement_row_digest
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEphemeralGraphIndexPlan {
    digest: String,
    admission_digest: String,
    requirement_set_digest: String,
    allocation_rows: Vec<WorthQueryEphemeralGraphIndexAllocationRow>,
}

impl WorthQueryEphemeralGraphIndexPlan {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn admission_digest(&self) -> &str {
        &self.admission_digest
    }

    pub fn requirement_set_digest(&self) -> &str {
        &self.requirement_set_digest
    }

    pub fn required_allocations(&self) -> &[WorthQueryEphemeralGraphIndexAllocationRow] {
        &self.allocation_rows
    }

    pub(crate) fn from_admission(admission: &WorthQueryGraphReadAccessAdmission) -> Option<Self> {
        if admission.posture() != &WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex
        {
            return None;
        }
        let allocation_source_rows = admission
            .graph_index_inventory_match_report()
            .matches()
            .iter()
            .filter(|row| {
                row.resolved_admission_posture()
                    == &WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex
            })
            .collect::<Vec<_>>();
        let allocation_rows = allocation_source_rows
            .into_iter()
            .map(|row| {
                WorthQueryEphemeralGraphIndexAllocationRow::new(
                    row.requirement_kind().clone(),
                    row.requirement_row_digest(),
                )
            })
            .collect::<Vec<_>>();
        let requirement_set_digest = admission.requirement_set().digest().render_support_hex();
        Some(Self::new(
            admission.digest(),
            requirement_set_digest,
            allocation_rows,
        ))
    }

    fn new(
        admission_digest: impl Into<String>,
        requirement_set_digest: String,
        allocation_rows: Vec<WorthQueryEphemeralGraphIndexAllocationRow>,
    ) -> Self {
        let admission_digest = admission_digest.into();
        let digest = hash_parts(
            &[
                "worth_query_ephemeral_graph_index_plan_v3".to_string(),
                format!("admission:{admission_digest}"),
                format!("requirements:{requirement_set_digest}"),
            ]
            .into_iter()
            .chain(
                allocation_rows
                    .iter()
                    .map(WorthQueryEphemeralGraphIndexAllocationRow::digest_part),
            )
            .collect::<Vec<_>>(),
        );
        Self {
            digest,
            admission_digest,
            requirement_set_digest,
            allocation_rows,
        }
    }
}
