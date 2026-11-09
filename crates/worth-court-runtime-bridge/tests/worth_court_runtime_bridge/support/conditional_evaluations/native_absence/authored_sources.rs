//! The absence algorithm is authored; contact telemetry is supplied freshly.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
pub(crate) fn authored_providers(contacts: Arc<Contacts>) -> BridgeConditionalProviderSet {
    BridgeConditionalProviderSet::new()
        .authored_condition(Providers(contacts.clone()))
        .authored_compute(Providers(contacts))
}
impl BridgeAuthoredConditionalProvider for Providers {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.court.authoritative-absence"),
            BoundaryProtocolVersion::new(1),
            Vec::<u8>::new(),
        )
    }
    fn readmit_authored_provider(
        &self,
        description: &BridgeAuthoredProviderDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        self.admit_authored_source_version(description)?;
        if !description.payload().is_empty() {
            return Err(BridgeAuthoredProviderDenial::InvalidPayload);
        }
        Ok(Self(self.0.clone()))
    }
}
