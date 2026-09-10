use super::WorthQueryGraphReadRow;

#[derive(Debug, PartialEq)]
pub struct WorthQueryGraphReadMaterial {
    rows: Vec<WorthQueryGraphReadRow>,
}

impl WorthQueryGraphReadMaterial {
    pub fn new(rows: impl IntoIterator<Item = WorthQueryGraphReadRow>) -> Self {
        Self {
            rows: rows.into_iter().collect(),
        }
    }

    pub fn rows(&self) -> &[WorthQueryGraphReadRow] {
        &self.rows
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.rows
            .capacity()
            .saturating_mul(std::mem::size_of::<WorthQueryGraphReadRow>())
            .saturating_add(
                self.rows
                    .iter()
                    .map(WorthQueryGraphReadRow::owned_allocation_capacity_bytes)
                    .sum(),
            )
    }
}
