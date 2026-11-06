use crate::application_query::WorthQueryApplicationQueryInstallationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationResourceInstallationDenialKind {
    ResourceNotInstalled,
    MissingTypedBinding,
    BindingMeaningChanged,
    AccessBindingMismatch,
    UnsupportedValueProfile,
    InvalidLimits,
    MissingActionAuthorization,
    AccessContractInvalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationResourceInstallationDenial {
    kind: WorthQueryApplicationResourceInstallationDenialKind,
    subject: String,
    access: Option<WorthQueryApplicationQueryInstallationDenial>,
    canonical_denial: Option<worth_foundational::facade::CanonicalDigestDerivationDenial>,
}

impl WorthQueryApplicationResourceInstallationDenial {
    pub(super) fn new(
        kind: WorthQueryApplicationResourceInstallationDenialKind,
        subject: &str,
    ) -> Self {
        Self {
            kind,
            subject: subject.to_owned(),
            access: None,
            canonical_denial: None,
        }
    }
    pub(super) fn access(
        subject: &str,
        cause: WorthQueryApplicationQueryInstallationDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationResourceInstallationDenialKind::AccessBindingMismatch,
            subject: subject.to_owned(),
            access: Some(cause),
            canonical_denial: None,
        }
    }
    pub const fn kind(&self) -> WorthQueryApplicationResourceInstallationDenialKind {
        self.kind
    }
    pub fn subject(&self) -> &str {
        &self.subject
    }
    pub fn access_binding_denial(&self) -> Option<&WorthQueryApplicationQueryInstallationDenial> {
        self.access.as_ref()
    }

    pub const fn canonical_denial(
        &self,
    ) -> Option<&worth_foundational::facade::CanonicalDigestDerivationDenial> {
        self.canonical_denial.as_ref()
    }

    pub(super) fn graph_obligation(
        subject: &str,
        cause: crate::graph_obligation::WorthQueryGraphObligationInstallationDenial,
    ) -> Self {
        let mut denial = Self::new(
            WorthQueryApplicationResourceInstallationDenialKind::AccessContractInvalid,
            subject,
        );
        if let crate::graph_obligation::WorthQueryGraphObligationInstallationDenial::Canonical(
            cause,
        ) = cause
        {
            denial.canonical_denial = Some(cause);
        }
        denial
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_obligation::WorthQueryGraphObligationInstallationDenial;
    use worth_foundational::facade::CanonicalDigestDerivationDenial;

    #[test]
    fn resource_obligation_budget_refusal_retains_its_canonical_cause() {
        let cause = CanonicalDigestDerivationDenial::EntryLimitExceeded {
            actual: 17,
            maximum: 16,
        };
        let denial = WorthQueryApplicationResourceInstallationDenial::graph_obligation(
            "resource",
            WorthQueryGraphObligationInstallationDenial::Canonical(cause),
        );
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationResourceInstallationDenialKind::AccessContractInvalid
        );
        assert_eq!(denial.canonical_denial(), Some(&cause));
        assert_eq!(denial.subject(), "resource");
    }
}
