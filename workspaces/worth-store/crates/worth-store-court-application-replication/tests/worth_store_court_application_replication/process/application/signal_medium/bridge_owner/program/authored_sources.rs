//! These fixed algorithms have no authored instance parameters; no values are retained.
use super::*;
impl BridgeAuthoredConditionalProvider for Requested {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.store.court.explicit-demand"),
            BoundaryProtocolVersion::new(1),
            Vec::<u8>::new(),
        )
    }
    fn readmit_authored_provider(
        &self,
        source: &BridgeAuthoredProviderDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        self.admit_authored_source_version(source)?;
        if !source.payload().is_empty() {
            return Err(BridgeAuthoredProviderDenial::InvalidPayload);
        }
        Ok(Self)
    }
}
impl BridgeAuthoredConditionalProvider for Compute {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.store.court.cold-compute"),
            BoundaryProtocolVersion::new(1),
            Vec::<u8>::new(),
        )
    }
    fn readmit_authored_provider(
        &self,
        source: &BridgeAuthoredProviderDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        self.admit_authored_source_version(source)?;
        if !source.payload().is_empty() {
            return Err(BridgeAuthoredProviderDenial::InvalidPayload);
        }
        Ok(Self)
    }
}
