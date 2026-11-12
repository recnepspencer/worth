//! Numeric predicate parameters belong to authored meaning; clocks remain derived.
use super::{domain, Predicate, ReadyNode};
impl domain::WorthQueryAuthoredHostConditionalPredicateProvider<ReadyNode> for Predicate {
    fn authored_provider_description(&self) -> domain::BridgeAuthoredProviderDescription {
        domain::BridgeAuthoredProviderDescription::new(
            domain::BoundaryProtocolIdentity::new("numeric-definition.predicate"),
            domain::BoundaryProtocolVersion::new(1),
            self.0.to_le_bytes().to_vec(),
        )
    }
    fn readmit_authored_provider(
        &self,
        source: &domain::BridgeAuthoredProviderDescription,
    ) -> Result<Self, domain::AuthoredProviderReconstructionDenial> {
        self.authored_provider_description()
            .admit_source_version(source)?;
        let value: [u8; 8] = source
            .payload()
            .try_into()
            .map_err(|_| domain::AuthoredProviderReconstructionDenial::MalformedPayload)?;
        Ok(Self(u64::from_le_bytes(value)))
    }
}
