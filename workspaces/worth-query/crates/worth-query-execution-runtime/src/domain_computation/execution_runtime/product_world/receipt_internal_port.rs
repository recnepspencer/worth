//! Crate-boundary internal port for `WorthQueryProductPublicationReceipt`
//! one-shot custody drains. Never exported on `worth-query-execution::facade` or Host.

use super::receipt::{WorthQueryCommitHistoryHold, WorthQueryProductPublicationReceipt};
use super::relational_publication::WorthQueryPerformedRelationalProductChange;

pub trait ProductPublicationReceiptInternalPort {
    fn take_history(&mut self) -> Option<WorthQueryCommitHistoryHold>;
    fn take_fresh_delivery(&self) -> Option<WorthQueryPerformedRelationalProductChange>;
    fn discharge_fresh_delivery(&self) -> bool;
    fn take_live_successor_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation>;
    fn take_output_demand_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation>;
    fn take_client_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation>;
}

impl ProductPublicationReceiptInternalPort for WorthQueryProductPublicationReceipt {
    fn take_history(&mut self) -> Option<WorthQueryCommitHistoryHold> {
        Self::take_history(self)
    }

    fn take_fresh_delivery(&self) -> Option<WorthQueryPerformedRelationalProductChange> {
        Self::take_fresh_delivery(self)
    }

    fn discharge_fresh_delivery(&self) -> bool {
        Self::discharge_fresh_delivery(self)
    }

    fn take_live_successor_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        Self::take_live_successor_observation(self)
    }

    fn take_output_demand_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        Self::take_output_demand_observation(self)
    }

    fn take_client_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        Self::take_client_observation(self)
    }
}
