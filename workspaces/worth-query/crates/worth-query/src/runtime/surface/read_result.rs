use crate::memory_workspace::WorthQueryEntity;

use super::{WorthQueryReadExecutionProduct, WorthQueryReadReceipt};

#[derive(Clone)]
pub struct WorthQueryReadResult {
    rows: Vec<WorthQueryEntity>,
    maintenance_source_rows: Option<Vec<WorthQueryEntity>>,
    receipt: WorthQueryReadReceipt,
}

impl std::fmt::Debug for WorthQueryReadResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryReadResult")
            .field("rows", &self.rows)
            .field("receipt", &self.receipt)
            .finish()
    }
}

impl PartialEq for WorthQueryReadResult {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.receipt == other.receipt
    }
}

impl WorthQueryReadResult {
    pub fn rows(&self) -> &[WorthQueryEntity] {
        &self.rows
    }

    pub fn receipt(&self) -> &WorthQueryReadReceipt {
        &self.receipt
    }

    pub(crate) fn maintenance_source_rows(&self) -> &[WorthQueryEntity] {
        self.maintenance_source_rows
            .as_deref()
            .unwrap_or(&self.rows)
    }

    pub(in crate::runtime) fn new(
        rows: Vec<WorthQueryEntity>,
        receipt: WorthQueryReadReceipt,
    ) -> Self {
        Self {
            rows,
            maintenance_source_rows: None,
            receipt,
        }
    }

    pub(in crate::runtime) fn new_with_source_rows(
        rows: Vec<WorthQueryEntity>,
        maintenance_source_rows: Vec<WorthQueryEntity>,
        receipt: WorthQueryReadReceipt,
    ) -> Self {
        Self {
            rows,
            maintenance_source_rows: Some(maintenance_source_rows),
            receipt,
        }
    }

    #[cfg(test)]
    pub(crate) fn test_only(rows: Vec<WorthQueryEntity>, receipt: WorthQueryReadReceipt) -> Self {
        Self {
            rows,
            maintenance_source_rows: None,
            receipt,
        }
    }
}

impl WorthQueryReadExecutionProduct for WorthQueryReadResult {
    fn receipt(&self) -> &WorthQueryReadReceipt {
        &self.receipt
    }

    fn receipt_mut(&mut self) -> &mut WorthQueryReadReceipt {
        &mut self.receipt
    }
}
