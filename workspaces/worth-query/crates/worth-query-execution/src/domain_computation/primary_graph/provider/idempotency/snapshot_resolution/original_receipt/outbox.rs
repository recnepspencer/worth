//! Original outbox DATA and separately readmitted installed meaning.
use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryDispatchOutboxRestoredFields,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxReadDenial as Denial;
use std::sync::Arc;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_installation::facade::{
    InstalledExternalEffectContract, InstalledInboundOccurrenceContract, InstalledOperationCatalog,
    InstalledOperationSource,
};
use worth_relational::facade::transactions::RecordRef;

pub struct WorthQueryRecoveredDispatchOutbox {
    record_ref: RecordRef,
    fields: WorthQueryDispatchOutboxRestoredFields,
    installed: Result<Arc<dyn InstalledOperationSource>, Denial>,
}

impl WorthQueryRecoveredDispatchOutbox {
    pub(in crate::domain_computation::primary_graph::provider) fn from_source_row(
        record_ref: RecordRef,
        fields: WorthQueryDispatchOutboxRestoredFields,
        recorded_intent: &str,
        catalog: &InstalledOperationCatalog,
    ) -> Self {
        let installed = readmit_contract(&fields, recorded_intent, catalog);
        Self {
            record_ref,
            fields,
            installed,
        }
    }
    pub fn record_ref(&self) -> &RecordRef {
        &self.record_ref
    }
    pub fn correlation(&self) -> &ExternalEffectCorrelationIdentity {
        &self.fields.correlation
    }
    pub fn correlation_family(&self) -> &str {
        self.fields.correlation_family.as_str()
    }
    pub fn effect(&self) -> &str {
        &self.fields.effect
    }
    pub fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        &self.fields.protocol_identity
    }
    pub fn protocol_version(&self) -> BoundaryProtocolVersion {
        self.fields.protocol_version
    }
    pub fn maximum_payload_bytes(&self) -> u64 {
        self.fields.maximum_payload_bytes
    }
    pub fn payload(&self) -> &[u8] {
        &self.fields.payload
    }
    pub fn outcome_identity(&self) -> u64 {
        self.fields.outcome_identity
    }
    pub fn operation_slot(&self) -> Option<&str> {
        self.fields.operation_slot.as_deref()
    }

    /// Read the original operation's installed inbound contract, without
    /// granting verifier, dispatch, publication or handler authority. A missing
    /// or mismatched installation is an error, never an undeclared inbound lane.
    pub fn inbound(&self) -> Result<Option<&InstalledInboundOccurrenceContract>, Denial> {
        let installed = self.installed.as_ref().map_err(|denial| *denial)?;
        Ok(installed
            .installed_view()
            .contracts()
            .external_effect()
            .inbound())
    }
}

fn readmit_contract(
    fields: &WorthQueryDispatchOutboxRestoredFields,
    recorded_intent: &str,
    catalog: &InstalledOperationCatalog,
) -> Result<Arc<dyn InstalledOperationSource>, Denial> {
    let slot = fields
        .operation_slot
        .as_deref()
        .ok_or(Denial::NotAuthoritative)?;
    let definition =
        WorthQueryApplicationIdempotencyBinding::recorded_operation_definition(recorded_intent)
            .map_err(|_| Denial::Malformed)?
            .ok_or(Denial::NotAuthoritative)?;
    let installed = catalog.operation(slot).ok_or(Denial::Missing)?;
    let view = installed.installed_view();
    if &view.definition_identity_bytes() != definition.bytes() {
        return Err(Denial::RecordMismatch);
    }
    let InstalledExternalEffectContract::Declared {
        correlation_family,
        effect,
        protocol,
        maximum_payload_bytes,
        ..
    } = view.contracts().external_effect()
    else {
        return Err(Denial::RecordMismatch);
    };
    if &fields.correlation_family != correlation_family
        || &fields.effect != effect
        || &fields.protocol_identity != protocol.identity()
        || fields.protocol_version != protocol.version()
        || fields.maximum_payload_bytes != *maximum_payload_bytes
        || u64::try_from(fields.payload.len()).map_err(|_| Denial::Malformed)?
            > *maximum_payload_bytes
    {
        return Err(Denial::RecordMismatch);
    }
    Ok(installed)
}

impl std::fmt::Debug for WorthQueryRecoveredDispatchOutbox {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryRecoveredDispatchOutbox")
            .field("record_ref", &self.record_ref)
            .field("fields", &self.fields)
            .field("inbound", &self.inbound())
            .finish()
    }
}

impl PartialEq for WorthQueryRecoveredDispatchOutbox {
    fn eq(&self, other: &Self) -> bool {
        self.record_ref == other.record_ref && self.fields == other.fields
    }
}
impl Eq for WorthQueryRecoveredDispatchOutbox {}

#[cfg(test)]
mod tests;
