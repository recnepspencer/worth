//! Static installed role source is complete and checked against the same fresh declaration.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_runtime_bridge::facade::{
    BridgeAuthoredConditionalProvider, BridgeAuthoredProviderDenial,
    BridgeAuthoredProviderDescription,
};
impl BridgeAuthoredConditionalProvider for QueryOutputReadinessPredicate {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.query.output-readiness"),
            BoundaryProtocolVersion::new(1),
            {
                let mut bytes = (self.semantics.dependency_count as u64)
                    .to_le_bytes()
                    .to_vec();
                bytes.extend_from_slice(self.authored_node.as_bytes());
                bytes
            },
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
            authored_node: self.authored_node.clone(),
        })
    }
}
