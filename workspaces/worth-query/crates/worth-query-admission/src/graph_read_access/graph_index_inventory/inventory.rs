use super::WorthQueryGraphIndexSupportRow;
use crate::graph_read_access::WorthQueryGraphReadAccessRequirementKind;

mod admitted;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphIndexInventory {
    digest: String,
    rows: Vec<WorthQueryGraphIndexSupportRow>,
}

impl WorthQueryGraphIndexInventory {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn rows(&self) -> &[WorthQueryGraphIndexSupportRow] {
        &self.rows
    }

    pub fn row_for_requirement_kind(
        &self,
        requirement_kind: &WorthQueryGraphReadAccessRequirementKind,
    ) -> Option<&WorthQueryGraphIndexSupportRow> {
        self.rows
            .iter()
            .find(|row| row.requirement_kind() == requirement_kind)
    }

    pub fn from_current_runtime_support() -> Self {
        Self::from_rows(
            WorthQueryGraphReadAccessRequirementKind::all()
                .iter()
                .cloned()
                .map(WorthQueryGraphIndexSupportRow::for_requirement_kind)
                .collect(),
        )
    }

    pub fn from_rows(mut rows: Vec<WorthQueryGraphIndexSupportRow>) -> Self {
        rows.sort_by(|a, b| {
            a.requirement_kind()
                .as_str()
                .cmp(b.requirement_kind().as_str())
                .then_with(|| a.digest().cmp(b.digest()))
        });
        rows.dedup_by(|a, b| a.digest() == b.digest());
        let mut parts = Vec::with_capacity(rows.len() + 1);
        parts.push("worth_query_graph_index_inventory_v1".to_owned());
        for row in &rows {
            let mut part = String::new();
            row.write_digest_part(&mut part)
                .expect("String formatting cannot fail");
            parts.push(part);
        }
        let digest = crate::admission_digest::hash_parts(&parts);
        Self { digest, rows }
    }
}

pub fn worth_query_graph_index_inventory() -> WorthQueryGraphIndexInventory {
    WorthQueryGraphIndexInventory::from_current_runtime_support()
}
