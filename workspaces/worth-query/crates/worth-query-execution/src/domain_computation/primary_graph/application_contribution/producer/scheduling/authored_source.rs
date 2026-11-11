//! Static installed role source is complete and checked against the same fresh declaration.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_runtime_bridge::facade::{
    BridgeAuthoredConditionalProvider, BridgeAuthoredProviderDenial,
    BridgeAuthoredProviderDescription,
};
impl BridgeAuthoredConditionalProvider for ProducerDemandTrigger {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.query.producer-demand"),
            BoundaryProtocolVersion::new(1),
            self.semantics.0.as_bytes().to_vec(),
        )
    }
    fn readmit_authored_provider(
        &self,
        description: &BridgeAuthoredProviderDescription,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        execution
            .consult()
            .map_err(BridgeAuthoredProviderDenial::Request)?;
        self.admit_authored_source_version(description)?;
        if description != &self.authored_description() {
            return Err(BridgeAuthoredProviderDenial::InvalidPayload);
        }
        Ok(Self {
            semantics: self.semantics.clone(),
        })
    }
}
impl BridgeAuthoredConditionalProvider for ProducerSignalCompute {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.query.producer-version"),
            BoundaryProtocolVersion::new(1),
            self.semantics.0.as_bytes().to_vec(),
        )
    }
    fn readmit_authored_provider(
        &self,
        description: &BridgeAuthoredProviderDescription,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        execution
            .consult()
            .map_err(BridgeAuthoredProviderDenial::Request)?;
        self.admit_authored_source_version(description)?;
        if description != &self.authored_description() {
            return Err(BridgeAuthoredProviderDenial::InvalidPayload);
        }
        Ok(Self {
            semantics: self.semantics.clone(),
        })
    }
}
