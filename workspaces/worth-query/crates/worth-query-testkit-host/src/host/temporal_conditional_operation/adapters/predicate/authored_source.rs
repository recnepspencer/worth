//! Authored algorithm selection excludes the live panic control and contact counters.
use super::*;
impl domain::WorthQueryAuthoredHostConditionalPredicateProvider<TemporalReadyNode> for Predicate {
    fn authored_provider_description(&self) -> domain::BridgeAuthoredProviderDescription {
        domain::BridgeAuthoredProviderDescription::new(
            domain::BoundaryProtocolIdentity::new("worth.court.temporal-predicate"),
            domain::BoundaryProtocolVersion::new(1),
            Vec::<u8>::new(),
        )
    }
    fn readmit_authored_provider(
        &self,
        source: &domain::BridgeAuthoredProviderDescription,
    ) -> Result<Self, domain::AuthoredProviderReconstructionDenial> {
        self.authored_provider_description()
            .admit_source_version(source)?;
        if !source.payload().is_empty() {
            return Err(domain::AuthoredProviderReconstructionDenial::MalformedPayload);
        }
        Ok(Self {
            panic: self.panic.clone(),
            contacts: self.contacts.clone(),
        })
    }
}
impl domain::WorthQueryAuthoredHostConditionalPredicateProvider<TemporalReadyNode>
    for ReplacementPredicate
{
    fn authored_provider_description(&self) -> domain::BridgeAuthoredProviderDescription {
        domain::BridgeAuthoredProviderDescription::new(
            domain::BoundaryProtocolIdentity::new("worth.court.replacement-predicate"),
            domain::BoundaryProtocolVersion::new(1),
            Vec::<u8>::new(),
        )
    }
    fn readmit_authored_provider(
        &self,
        source: &domain::BridgeAuthoredProviderDescription,
    ) -> Result<Self, domain::AuthoredProviderReconstructionDenial> {
        self.authored_provider_description()
            .admit_source_version(source)?;
        if !source.payload().is_empty() {
            return Err(domain::AuthoredProviderReconstructionDenial::MalformedPayload);
        }
        Ok(Self(Predicate {
            panic: self.0.panic.clone(),
            contacts: self.0.contacts.clone(),
        }))
    }
}
