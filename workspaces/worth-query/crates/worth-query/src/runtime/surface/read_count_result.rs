use super::{WorthQueryReadExecutionProduct, WorthQueryReadReceipt};

/// A count aggregate produced from one admitted collection read.
///
/// The scalar is deliberately not represented as an entity row. Its receipt
/// retains the source query, basis, plan, graph-access, and bounded-work proof.
#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryCountResult {
    count: u64,
    receipt: WorthQueryReadReceipt,
}

impl WorthQueryCountResult {
    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn receipt(&self) -> &WorthQueryReadReceipt {
        &self.receipt
    }

    pub(in crate::runtime) fn new(count: u64, receipt: WorthQueryReadReceipt) -> Self {
        Self { count, receipt }
    }
}

impl WorthQueryReadExecutionProduct for WorthQueryCountResult {
    fn receipt(&self) -> &WorthQueryReadReceipt {
        &self.receipt
    }

    fn receipt_mut(&mut self) -> &mut WorthQueryReadReceipt {
        &mut self.receipt
    }
}
