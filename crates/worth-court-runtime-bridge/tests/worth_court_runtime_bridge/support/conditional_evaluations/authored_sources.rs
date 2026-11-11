//! Owner-defined sources for the court's stateless algorithms; counters stay live.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_runtime_bridge::facade::*;
impl BridgeAuthoredConditionalProvider for Requested {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.court.explicit-demand"),
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
        Ok(Self)
    }
}
impl BridgeAuthoredConditionalProvider for VersionCompute {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.court.output-version"),
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
        Ok(Self)
    }
}
