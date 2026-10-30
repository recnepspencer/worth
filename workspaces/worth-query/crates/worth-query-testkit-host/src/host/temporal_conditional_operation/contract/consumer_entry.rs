//! Temporal consumer entry implementations live with their Host marker types.
use super::{TemporalDomain, TemporalDomainFamily, TemporalDomainOperation};
use worth_query::facade::{domain, read};

impl domain::WorthQueryDomainEntryMarker for TemporalDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.certification.temporal-consumer"
    }

    fn display_name(&self) -> &'static str {
        "Temporal consumer"
    }

    fn required_capability_families(&self) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[]
    }
}

impl domain::WorthQueryExecutableDomainOperation<TemporalDomain, TemporalDomainFamily>
    for TemporalDomainOperation
{
    type Input = ();
    type Output = read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryDirectOperation;
}
