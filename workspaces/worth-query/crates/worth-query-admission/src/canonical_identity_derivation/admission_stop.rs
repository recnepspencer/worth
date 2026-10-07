use worth_foundational::facade::{
    CanonicalBasisConstructionDenial, CanonicalDigestDerivationDenial,
};

/// Canonical requirement preparation preserves semantic denial independently
/// of the caller's resource refusal. No prepared requirement set escapes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorthQueryCanonicalIdentityStop<Stop> {
    Derivation(CanonicalDigestDerivationDenial),
    Construction(CanonicalBasisConstructionDenial),
    Admission(Stop),
    AccountingOverflow,
    AllocationUnavailable,
    UnsupportedSortImplementation,
}

impl<Stop> From<crate::graph_read_access::AdmittedDigestTextStop<Stop>>
    for WorthQueryCanonicalIdentityStop<Stop>
{
    fn from(stop: crate::graph_read_access::AdmittedDigestTextStop<Stop>) -> Self {
        match stop {
            crate::graph_read_access::AdmittedDigestTextStop::Admission(stop) => {
                Self::Admission(stop)
            }
            crate::graph_read_access::AdmittedDigestTextStop::AccountingOverflow => {
                Self::AccountingOverflow
            }
        }
    }
}

impl<Stop> From<worth_foundational::facade::CanonicalDigestAdmissionStop<Self>>
    for WorthQueryCanonicalIdentityStop<Stop>
{
    fn from(stop: worth_foundational::facade::CanonicalDigestAdmissionStop<Self>) -> Self {
        use worth_foundational::facade::CanonicalDigestAdmissionStop;
        match stop {
            CanonicalDigestAdmissionStop::Derivation(denial) => Self::Derivation(denial),
            CanonicalDigestAdmissionStop::Resource(stop) => stop,
            CanonicalDigestAdmissionStop::AccountingOverflow => Self::AccountingOverflow,
            CanonicalDigestAdmissionStop::AllocationUnavailable => Self::AllocationUnavailable,
        }
    }
}

impl<Stop> From<worth_foundational::facade::CanonicalBasisPreparationStop<Self>>
    for WorthQueryCanonicalIdentityStop<Stop>
{
    fn from(stop: worth_foundational::facade::CanonicalBasisPreparationStop<Self>) -> Self {
        use worth_foundational::facade::CanonicalBasisPreparationStop;
        match stop {
            CanonicalBasisPreparationStop::Construction(denial) => Self::Construction(denial),
            CanonicalBasisPreparationStop::Resource(stop) => stop,
            CanonicalBasisPreparationStop::AccountingOverflow => Self::AccountingOverflow,
            CanonicalBasisPreparationStop::UnsupportedSortImplementation => {
                Self::UnsupportedSortImplementation
            }
        }
    }
}
