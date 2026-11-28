use super::*;
use crate::source::{SourceFailureRecord, SourceMaterializationRecord};

impl BridgeDiagnosticsFacade {
    pub(crate) fn record_route(&self, record: BridgeRouteRecord) {
        if !self.config.records_enabled {
            return;
        }

        self.state
            .write()
            .expect("bridge diagnostics lock poisoned")
            .record_route(record, self.config.route_record_limit);
    }

    pub(crate) fn record_failure(&self, record: BridgeFailureRecord) {
        self.state
            .write()
            .expect("bridge diagnostics lock poisoned")
            .record_failure(record, self.config.failure_record_limit);
    }

    pub(crate) fn record_continuity(&self, record: BridgeCanonicalContinuityRecord) {
        if !self.config.records_enabled {
            return;
        }

        self.state
            .write()
            .expect("bridge diagnostics lock poisoned")
            .record_continuity(record, self.config.route_record_limit);
    }

    pub(crate) fn record_bulk(&self, record: BridgeCanonicalBulkPlanRecord) {
        if !self.config.records_enabled {
            return;
        }

        self.state
            .write()
            .expect("bridge diagnostics lock poisoned")
            .record_bulk(record, self.config.route_record_limit);
    }

    pub(crate) fn record_historical_evaluation(
        &self,
        record: BridgeCanonicalHistoricalEvaluationRecord,
    ) {
        if !self.config.records_enabled {
            return;
        }

        self.state
            .write()
            .expect("bridge diagnostics lock poisoned")
            .record_historical(record, self.config.route_record_limit);
    }

    pub(crate) fn record_historical_evaluation_failure(
        &self,
        record: BridgeHistoricalEvaluationFailureRecord,
    ) {
        if !self.config.records_enabled {
            return;
        }

        self.state
            .write()
            .expect("bridge diagnostics lock poisoned")
            .record_historical_failure(record, self.config.route_record_limit);
    }

    pub(crate) fn record_source_materialization(&self, record: SourceMaterializationRecord) {
        if !self.config.records_enabled {
            return;
        }

        self.state
            .write()
            .expect("bridge diagnostics lock poisoned")
            .record_source_materialization(record, self.config.route_record_limit);
    }

    pub(crate) fn record_source_failure(&self, record: SourceFailureRecord) {
        if !self.config.records_enabled {
            return;
        }

        self.state
            .write()
            .expect("bridge diagnostics lock poisoned")
            .record_source_failure(record, self.config.route_record_limit);
    }

    pub(crate) fn record_delivery_failure(
        &self,
        source: BridgeFailureSource,
        error: &BridgeDeliveryError,
    ) {
        self.emit_operator_report(error);
        self.record_failure(BridgeFailureRecord::from_failure(
            source,
            BridgeFailureClass::Delivery(error.kind()),
            error.to_string(),
            error.context().clone(),
        ));
    }

    pub(crate) fn record_replay_failure(
        &self,
        source: BridgeFailureSource,
        error: &BridgeReplayError,
    ) {
        self.emit_operator_report(error);
        self.record_failure(BridgeFailureRecord::from_failure(
            source,
            BridgeFailureClass::Replay(error.kind()),
            error.to_string(),
            error.context().clone(),
        ));
    }
}
