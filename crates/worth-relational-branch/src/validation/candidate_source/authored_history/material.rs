//! Funded weak payload custody from the actual completed publication owner.
use super::super::opaque_custody::FundedSourceMaterialReceipt;
use super::ManagedSourceImpactKey;
use crate::branch::managed_source_affinity::ManagedSourceCellAffinity;
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionOrderedMap};

#[derive(Clone)]
pub enum HistoricalSourcePair {
    Conservative,
    Complete {
        before: FundedSourceMaterialReceipt,
        after: FundedSourceMaterialReceipt,
    },
}
impl ChargedBytes for HistoricalSourcePair {
    fn additional_charged_bytes(&self) -> u64 {
        0 // Each weak receipt carries its sealed allocation funding.
    }
}
pub struct RetainedSourceHistoryMaterial {
    pub owner: Option<Arc<ManagedSourceCellAffinity>>,
    pub scopes: ExecutionOrderedMap<ManagedSourceImpactKey, HistoricalSourcePair>,
}
