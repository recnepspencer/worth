use crate::admission_digest::AdmittedHashStop;
use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;
use crate::graph_read_access::AdmittedDigestTextStop;

#[derive(Debug)]
pub enum WorthQueryGraphIndexInventoryAdmissionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
    AllocationUnavailable,
    UnsupportedPreparation,
}

impl<Stop> From<AdmittedHashStop<Stop>> for WorthQueryGraphIndexInventoryAdmissionStop<Stop> {
    fn from(stop: AdmittedHashStop<Stop>) -> Self {
        match stop {
            AdmittedHashStop::Admission(stop) => Self::Admission(stop),
            AdmittedHashStop::AccountingOverflow => Self::AccountingOverflow,
        }
    }
}

impl<Stop> From<AdmittedDigestTextStop<Stop>> for WorthQueryGraphIndexInventoryAdmissionStop<Stop> {
    fn from(stop: AdmittedDigestTextStop<Stop>) -> Self {
        match stop {
            AdmittedDigestTextStop::Admission(stop) => Self::Admission(stop),
            AdmittedDigestTextStop::AccountingOverflow => Self::AccountingOverflow,
        }
    }
}

impl<Stop> From<WorthQueryCanonicalIdentityStop<Stop>>
    for WorthQueryGraphIndexInventoryAdmissionStop<Stop>
{
    fn from(stop: WorthQueryCanonicalIdentityStop<Stop>) -> Self {
        match stop {
            WorthQueryCanonicalIdentityStop::Admission(stop) => Self::Admission(stop),
            WorthQueryCanonicalIdentityStop::AccountingOverflow => Self::AccountingOverflow,
            WorthQueryCanonicalIdentityStop::AllocationUnavailable => Self::AllocationUnavailable,
            WorthQueryCanonicalIdentityStop::Derivation(_)
            | WorthQueryCanonicalIdentityStop::Construction(_)
            | WorthQueryCanonicalIdentityStop::UnsupportedSortImplementation => {
                Self::UnsupportedPreparation
            }
        }
    }
}
