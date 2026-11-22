//! Crate-boundary internal ports for decision read-set capture/comparison.
//! Never exported on `worth-query-execution::facade` or Host.

use std::sync::Arc;

use super::capture::{WorthQueryCompleteDecisionReadSetReceipt, WorthQueryFreshDecisionReadSet};
use crate::ports::PrimaryReadCustody;

pub trait DecisionReadSetReceiptInternalPort {
    fn attach_primary_full(
        &mut self,
        custody: Arc<dyn PrimaryReadCustody>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    );

    #[allow(clippy::result_unit_err)]
    fn attach_primary_prefix(
        &mut self,
        grouped_count: usize,
        shared: Arc<dyn PrimaryReadCustody>,
    ) -> Result<(), ()>;

    fn primary_custody(&self) -> Option<&Arc<dyn PrimaryReadCustody>>;

    fn take_primary(&mut self) -> Option<Arc<dyn PrimaryReadCustody>>;
}

impl DecisionReadSetReceiptInternalPort for WorthQueryCompleteDecisionReadSetReceipt {
    fn attach_primary_full(
        &mut self,
        custody: Arc<dyn PrimaryReadCustody>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) {
        self._primary_custody = Some(custody);
        self._primary_memory = memory;
    }

    #[allow(clippy::result_unit_err)]
    fn attach_primary_prefix(
        &mut self,
        grouped_count: usize,
        shared: Arc<dyn PrimaryReadCustody>,
    ) -> Result<(), ()> {
        self.counters.requested_facts = self
            .counters
            .requested_facts
            .checked_add(grouped_count)
            .ok_or(())?;
        self._primary_custody = Some(Arc::clone(&shared));
        self.primary = Some(shared);
        Ok(())
    }

    fn primary_custody(&self) -> Option<&Arc<dyn PrimaryReadCustody>> {
        self._primary_custody.as_ref()
    }

    fn take_primary(&mut self) -> Option<Arc<dyn PrimaryReadCustody>> {
        self.primary.take()
    }
}

pub trait FreshDecisionReadSetInternalPort: Sized {
    fn has_primary(&self) -> bool;
    fn primary_custody(&self) -> Option<&Arc<dyn PrimaryReadCustody>>;
    fn set_primary(&mut self, primary: Option<Arc<dyn PrimaryReadCustody>>);
    fn into_receipt(self) -> WorthQueryCompleteDecisionReadSetReceipt;
}

impl FreshDecisionReadSetInternalPort for WorthQueryFreshDecisionReadSet {
    fn has_primary(&self) -> bool {
        self.receipt.primary.is_some()
    }

    fn primary_custody(&self) -> Option<&Arc<dyn PrimaryReadCustody>> {
        self.receipt._primary_custody.as_ref()
    }

    fn set_primary(&mut self, primary: Option<Arc<dyn PrimaryReadCustody>>) {
        self.receipt.primary = primary;
    }

    fn into_receipt(self) -> WorthQueryCompleteDecisionReadSetReceipt {
        self.receipt
    }
}
