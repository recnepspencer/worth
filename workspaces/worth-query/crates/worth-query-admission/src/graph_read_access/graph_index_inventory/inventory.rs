use super::WorthQueryGraphIndexSupportRow;
use crate::graph_read_access::WorthQueryGraphReadAccessRequirementKind;
use std::convert::Infallible;

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

    pub fn from_rows(rows: Vec<WorthQueryGraphIndexSupportRow>) -> Self {
        Self::from_rows_admitted(rows, &mut |_, _| Ok::<(), Infallible>(()))
            .expect("ordinary inventory construction has no resource refusal")
    }
}

pub fn worth_query_graph_index_inventory() -> WorthQueryGraphIndexInventory {
    WorthQueryGraphIndexInventory::from_current_runtime_support()
}
