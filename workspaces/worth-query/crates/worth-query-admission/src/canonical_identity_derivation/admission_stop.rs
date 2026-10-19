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
    Preparation {
        cause: Stop,
        context: super::WorthQueryCanonicalIdentityPreparationContext,
    },
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
            CanonicalBasisPreparationStop::Resource { cause, quote } => {
                cause.with_canonical_quote(quote)
            }
            CanonicalBasisPreparationStop::AccountingOverflow => Self::AccountingOverflow,
            CanonicalBasisPreparationStop::UnsupportedSortImplementation => {
                Self::UnsupportedSortImplementation
            }
        }
    }
}

impl<Stop> WorthQueryCanonicalIdentityStop<Stop> {
    pub(crate) fn with_preparation_stage(
        self,
        stage: super::WorthQueryCanonicalIdentityPreparationStage,
    ) -> Self {
        self.with_context(|context| {
            context.identity.get_or_insert(stage);
        })
    }

    fn with_canonical_quote(
        self,
        quote: worth_foundational::facade::CanonicalBasisPreparationQuote,
    ) -> Self {
        self.with_context(|context| {
            context.quote.get_or_insert(quote);
        })
    }

    fn with_context(
        self,
        update: impl FnOnce(&mut super::WorthQueryCanonicalIdentityPreparationContext),
    ) -> Self {
        let (cause, mut context) = match self {
            Self::Admission(cause) => (cause, Default::default()),
            Self::Preparation { cause, context } => (cause, context),
            other => return other,
        };
        update(&mut context);
        Self::Preparation { cause, context }
    }
}
