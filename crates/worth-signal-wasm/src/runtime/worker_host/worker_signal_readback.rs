use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::boundary::errors::WorthSignalJsError;
use crate::expression::model::SignalValue;

use super::{
    canonical_worker_certification_digest, committed_truth_digest_for_runtime,
    WorkerHostBoundaryPerformanceEnvelope, WorkerRuntimeShell,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerSignalReadbackRequest {
    pub signal_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerReadbackSignal {
    pub id: String,
    pub value: SignalValue,
    pub payload_byte_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerSignalReadbackPacket {
    pub envelope_family: &'static str,
    pub readback_mode: &'static str,
    pub runtime_authority: &'static str,
    pub signal_readback_packet_count: u64,
    pub signal_readback_breadth: u64,
    pub signal_payload_byte_count: u64,
    pub worker_first_truth_digest: String,
    pub signal_digest: String,
    pub boundary_performance: WorkerHostBoundaryPerformanceEnvelope,
    pub packet_digest: String,
    pub signals: Vec<WorkerReadbackSignal>,
}

impl WorkerSignalReadbackPacket {
    fn from_signals(
        signals: Vec<WorkerReadbackSignal>,
        worker_first_truth_digest: String,
    ) -> Result<Self, WorthSignalJsError> {
        let signal_readback_breadth = signals.len() as u64;
        let signal_payload_byte_count = signals.iter().fold(0_u64, |total, signal| {
            total.saturating_add(signal.payload_byte_count)
        });
        let signal_digest = canonical_worker_certification_digest(&signals)?;
        let boundary_performance = WorkerHostBoundaryPerformanceEnvelope::signal_readback(
            signal_readback_breadth,
            signal_payload_byte_count,
            signal_digest.as_str(),
        )?;
        let packet_digest = canonical_worker_certification_digest(&(
            "signalReadback",
            "CommittedSignalReadback",
            signal_readback_breadth,
            signal_payload_byte_count,
            worker_first_truth_digest.as_str(),
            signal_digest.as_str(),
            boundary_performance.performance_digest.as_str(),
        ))?;

        Ok(Self {
            envelope_family: "signalReadback",
            readback_mode: "CommittedSignalReadback",
            runtime_authority: "workerOwnedRuntime",
            signal_readback_packet_count: 1,
            signal_readback_breadth,
            signal_payload_byte_count,
            worker_first_truth_digest,
            signal_digest,
            boundary_performance,
            packet_digest,
            signals,
        })
    }
}

impl WorkerRuntimeShell {
    pub fn read_signals(
        &mut self,
        request: WorkerSignalReadbackRequest,
    ) -> Result<WorkerSignalReadbackPacket, WorthSignalJsError> {
        validate_signal_readback_request(&request)?;
        let ids = request.signal_ids;
        // Keep the worker's bulk boundary bulk inside the runtime as well:
        // scalar reads repeatedly plan overlapping dependency closures.
        let values = self.core.read_values(ids.clone())?;
        let signals = ids
            .into_iter()
            .zip(values)
            .map(|(id, value)| Self::encode_readback_signal(id, value))
            .collect::<Result<Vec<_>, _>>()?;
        WorkerSignalReadbackPacket::from_signals(
            signals,
            committed_truth_digest_for_runtime(&self.core)?,
        )
    }

    fn encode_readback_signal(
        id: String,
        value: SignalValue,
    ) -> Result<WorkerReadbackSignal, WorthSignalJsError> {
        let payload_byte_count = serde_json::to_vec(&value)
            .map_err(|error| {
                WorthSignalJsError::internal(format!(
                    "failed to measure signal readback payload `{id}`: {error}"
                ))
            })?
            .len() as u64;
        Ok(WorkerReadbackSignal {
            id,
            value,
            payload_byte_count,
        })
    }
}

fn validate_signal_readback_request(
    request: &WorkerSignalReadbackRequest,
) -> Result<(), WorthSignalJsError> {
    if request.signal_ids.is_empty() {
        return Err(WorthSignalJsError::invalid_input(
            "worker signal readback requires at least one signal id",
        ));
    }
    let mut seen = BTreeSet::new();
    for id in &request.signal_ids {
        if id.trim().is_empty() {
            return Err(WorthSignalJsError::invalid_input(
                "worker signal readback rejects blank signal ids",
            ));
        }
        if !seen.insert(id) {
            return Err(WorthSignalJsError::invalid_input(format!(
                "worker signal readback rejects duplicate signal id `{id}`"
            )));
        }
    }
    Ok(())
}
