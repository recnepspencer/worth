//! Moved whole from `domain_computation::primary_graph::application_output_demand::registry::required_work`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::application_output_demand::native_hints::NativeHint;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;
use std::sync::Arc;

pub struct ReplacedRequiredWorkHint {
    _native: Option<Box<NativeHint>>,
    _settlement: Option<Arc<RecordedSettlementIdentity>>,
}
