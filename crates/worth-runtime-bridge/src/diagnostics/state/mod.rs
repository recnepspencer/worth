use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use crate::input::envelope::TruthCommitIdentity;
use crate::routing::{
    BridgeCanonicalBulkPlanRecord, BridgeInvalidationIdentity, BridgeRouteIdentity,
    BridgeWorkloadIdentity,
};
use crate::source::{SourceFailureRecord, SourceMaterializationRecord};

use super::continuity::BridgeCanonicalContinuityRecord;
use super::history::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeHistoricalEvaluationDecisionLogIdentity,
    BridgeHistoricalEvaluationFailureIdentity, BridgeHistoricalEvaluationFailureRecord,
    BridgeHistoricalEvaluationRecordIdentity,
};
use super::records::{BridgeFailureRecord, BridgeRouteRecord};

mod config;
mod evict;
mod query;
mod record;

pub(crate) use config::BridgeDiagnosticsConfig;

#[derive(Debug, Default)]
pub(crate) struct BridgeDiagnosticsState {
    pub(super) conditional_history: super::conditional_history::ConditionalExecutionHistory,
    route_records: VecDeque<Arc<BridgeRouteRecord>>,
    bulk_records: VecDeque<Arc<BridgeCanonicalBulkPlanRecord>>,
    continuity_records: VecDeque<Arc<BridgeCanonicalContinuityRecord>>,
    historical_records: VecDeque<Arc<BridgeCanonicalHistoricalEvaluationRecord>>,
    historical_failures: VecDeque<Arc<BridgeHistoricalEvaluationFailureRecord>>,
    source_materialization_records: VecDeque<Arc<SourceMaterializationRecord>>,
    source_failure_records: VecDeque<Arc<SourceFailureRecord>>,
    failure_records: VecDeque<Arc<BridgeFailureRecord>>,
    latest_route_by_route_identity: BTreeMap<BridgeRouteIdentity, Arc<BridgeRouteRecord>>,
    latest_bulk_by_workload_identity: BTreeMap<String, Arc<BridgeCanonicalBulkPlanRecord>>,
    latest_continuity_by_route_identity:
        BTreeMap<BridgeRouteIdentity, Arc<BridgeCanonicalContinuityRecord>>,
    latest_historical_by_record_identity:
        BTreeMap<String, Arc<BridgeCanonicalHistoricalEvaluationRecord>>,
    latest_historical_by_decision_log_identity:
        BTreeMap<String, Arc<BridgeCanonicalHistoricalEvaluationRecord>>,
    latest_historical_failure_by_declaration_identity:
        BTreeMap<String, Arc<BridgeHistoricalEvaluationFailureRecord>>,
    latest_historical_failure_by_failure_identity:
        BTreeMap<String, Arc<BridgeHistoricalEvaluationFailureRecord>>,
    latest_source_materialization_by_record_identity:
        BTreeMap<String, Arc<SourceMaterializationRecord>>,
    latest_source_failure_by_declaration_identity: BTreeMap<String, Arc<SourceFailureRecord>>,
    latest_source_failure_by_failure_identity: BTreeMap<String, Arc<SourceFailureRecord>>,
    latest_route_by_invalidation_identity:
        BTreeMap<BridgeInvalidationIdentity, Arc<BridgeRouteRecord>>,
    latest_route_by_source_commit: BTreeMap<TruthCommitIdentity, Arc<BridgeRouteRecord>>,
}
