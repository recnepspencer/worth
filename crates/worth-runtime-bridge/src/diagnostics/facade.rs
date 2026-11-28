use std::sync::{Arc, RwLock};

use crate::error::{BridgeDeliveryError, BridgeReplayError};
use crate::policy::{BridgeDiagnosticsTier, BridgeRuntimePolicy};
use crate::routing::BridgeCanonicalBulkPlanRecord;

use super::bulk::BridgeBulkPlanExplanation;
use super::continuity::{BridgeCanonicalContinuityRecord, BridgeContinuityExplanation};
use super::failure_source::BridgeFailureSource;
use super::handle::BridgeDiagnosticsHandle;
use super::history::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeHistoricalEvaluationExplanation,
    BridgeHistoricalEvaluationFailureRecord,
};
use super::records::{BridgeFailureClass, BridgeFailureRecord, BridgeRouteRecord};
use super::replay::{BridgeCanonicalRouteRecord, BridgeReplayRecord};
use super::state::{BridgeDiagnosticsConfig, BridgeDiagnosticsState};
use super::BridgeRouteExplanation;

mod conditional_history;
mod explain;
mod operator_reports;
mod policy;
mod query;
mod record;

#[derive(Debug, Clone)]
pub struct BridgeDiagnosticsFacade {
    config: Arc<BridgeDiagnosticsConfig>,
    operator_reports: operator_reports::BridgeOperatorReports,
    state: Arc<RwLock<BridgeDiagnosticsState>>,
}

impl BridgeDiagnosticsFacade {
    pub(crate) fn for_authored_installation(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            operator_reports: self.operator_reports.clone(),
            state: Arc::new(RwLock::new(Default::default())),
        }
    }
}
