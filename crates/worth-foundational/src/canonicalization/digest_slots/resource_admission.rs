use super::CanonicalDigestDerivationDenial;

/// Resource refusal preserves the caller's original reason independently of
/// canonical grammar and byte-budget denial. No ready artifact is emitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalDigestAdmissionStop<Stop> {
    Derivation(CanonicalDigestDerivationDenial),
    Resource(Stop),
    AccountingOverflow,
    AllocationUnavailable,
}

pub(super) type CanonicalResourceAdmission<'a> = dyn FnMut(usize, usize) -> Result<(), ()> + 'a;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CanonicalDigestPreparationStop {
    Derivation(CanonicalDigestDerivationDenial),
    ResourceRefused,
    AccountingOverflow,
    AllocationUnavailable,
}

impl CanonicalDigestPreparationStop {
    pub(super) fn preserve<Stop>(
        self,
        refusal: Option<Stop>,
    ) -> CanonicalDigestAdmissionStop<Stop> {
        match self {
            Self::Derivation(denial) => CanonicalDigestAdmissionStop::Derivation(denial),
            Self::ResourceRefused => CanonicalDigestAdmissionStop::Resource(
                refusal.expect("resource callback retains its original refusal"),
            ),
            Self::AccountingOverflow => CanonicalDigestAdmissionStop::AccountingOverflow,
            Self::AllocationUnavailable => CanonicalDigestAdmissionStop::AllocationUnavailable,
        }
    }
}
