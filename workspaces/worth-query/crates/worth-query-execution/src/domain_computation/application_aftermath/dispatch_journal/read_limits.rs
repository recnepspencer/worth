//! Readmission bounds come from immutable installed inbound contracts.
use super::*;
use worth_query_installation::facade::{
    InstalledInboundOccurrenceContract, InstalledOperationCatalog,
};

#[derive(Default)]
pub(super) struct QueryDispatchReadLimits {
    inbound: BTreeMap<String, InstalledInboundOccurrenceContract>,
    maximum_frame_bytes: u64,
}
impl QueryDispatchReadLimits {
    pub(super) fn from_catalog(catalog: &InstalledOperationCatalog) -> Self {
        let inbound: BTreeMap<String, InstalledInboundOccurrenceContract> = catalog
            .readmission_snapshot()
            .into_iter()
            .filter_map(|source| {
                let view = source.installed_view();
                view.contracts()
                    .external_effect()
                    .inbound()
                    .map(|contract| (view.operation().to_owned(), contract.clone()))
            })
            .collect();
        let maximum_frame_bytes = inbound
            .values()
            .map(|contract| contract.limits().maximum_accepted_bytes.get())
            .max()
            .unwrap_or(0)
            .max(QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES);
        Self {
            inbound,
            maximum_frame_bytes,
        }
    }
    pub(super) fn maximum_frame_bytes(&self) -> u64 {
        self.maximum_frame_bytes
            .max(QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES)
    }
    pub(super) fn validate(
        &self,
        frame: &QueryDispatchFrame,
    ) -> Result<(), QueryDispatchJournalDenial> {
        if frame.bytes.len() as u64 > self.maximum_frame_bytes() {
            return Err(QueryDispatchJournalDenial::MalformedFrame);
        }
        match frame.record() {
            QueryDispatchRecord::Inbound(QueryInboundRecord::TransportBinding(binding)) => {
                let contract = self
                    .inbound
                    .get(&binding.operation)
                    .ok_or(QueryDispatchJournalDenial::MalformedFrame)?;
                if frame.bytes.len() as u64 > contract.limits().maximum_accepted_bytes.get() {
                    return Err(QueryDispatchJournalDenial::MalformedFrame);
                }
                Ok(())
            }
            QueryDispatchRecord::Inbound(QueryInboundRecord::Accepted(accepted)) => {
                let contract = self
                    .inbound
                    .get(&accepted.operation)
                    .ok_or(QueryDispatchJournalDenial::MalformedFrame)?;
                let limits = contract.limits();
                let validity_ms = accepted
                    .claims
                    .expires_at_unix_seconds
                    .checked_sub(accepted.claims.issued_at_unix_seconds)
                    .and_then(|seconds| seconds.checked_mul(1_000));
                if accepted.claims.key_epoch == 0
                    || validity_ms
                        .is_none_or(|validity| validity > limits.replay_window_milliseconds.get())
                    || accepted.claims.source_identity != contract.source_identity()
                    || &accepted.claims.protocol_identity != contract.protocol().identity()
                    || accepted.claims.protocol_version != contract.protocol().version()
                    || accepted.claims.payload.len() as u64 > limits.maximum_payload_bytes.get()
                    || accepted.envelope_bytes > limits.maximum_envelope_bytes.get()
                    || (frame.bytes.len() as u64)
                        .checked_add(accepted.envelope_bytes)
                        .is_none_or(|bytes| bytes > limits.maximum_accepted_bytes.get())
                {
                    return Err(QueryDispatchJournalDenial::MalformedFrame);
                }
                Ok(())
            }
            QueryDispatchRecord::Inbound(
                QueryInboundRecord::Acknowledged { .. } | QueryInboundRecord::Expired { .. },
            )
            | QueryDispatchRecord::SourceControl { .. }
            | QueryDispatchRecord::Attempt(_)
            | QueryDispatchRecord::Observation { .. } => Ok(()),
        }
    }
}
