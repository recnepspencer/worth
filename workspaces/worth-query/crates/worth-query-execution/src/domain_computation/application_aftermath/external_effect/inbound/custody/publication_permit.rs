//! A transport completion shares the finite publication lane with signed input.

use std::sync::{Arc, Mutex};

use worth_query_installation::facade::InstalledInboundOccurrenceContract;

use super::WorthQueryInboundCustody;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryTransportPublicationPermitDenial {
    ForeignBinding,
    AtCapacity,
    CounterExhausted,
}

/// One active transport World publication. Its drop releases the same
/// operation counter charged by authenticated inbound publication.
pub(in crate::domain_computation) struct WorthQueryTransportPublicationPermit {
    custody: Arc<Mutex<WorthQueryInboundCustody>>,
    operation: String,
}

pub(super) fn reserve(
    custody: Arc<Mutex<WorthQueryInboundCustody>>,
    operation: &str,
    contract: &InstalledInboundOccurrenceContract,
) -> Result<WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial> {
    {
        let mut ledger = custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let usage = ledger
            .usage_by_operation
            .entry(operation.to_owned())
            .or_default();
        if usage.publishing >= contract.limits().maximum_concurrent_publications.get() {
            return Err(WorthQueryTransportPublicationPermitDenial::AtCapacity);
        }
        usage.publishing = usage
            .publishing
            .checked_add(1)
            .ok_or(WorthQueryTransportPublicationPermitDenial::CounterExhausted)?;
    }
    Ok(WorthQueryTransportPublicationPermit {
        custody,
        operation: operation.to_owned(),
    })
}

impl Drop for WorthQueryTransportPublicationPermit {
    fn drop(&mut self) {
        let mut ledger = self
            .custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let usage = ledger
            .usage_by_operation
            .get_mut(&self.operation)
            .expect("an active publication retains its operation counter");
        usage.publishing = usage
            .publishing
            .checked_sub(1)
            .expect("an active publication keeps its counter charged");
    }
}
