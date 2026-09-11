#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryApplicationValueBindingInstallationDenialKind {
    MissingBinding,
    UnexpectedBinding,
    ContractMismatch,
    NativeContractMissing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryApplicationValueBindingInstallationDenial {
    kind: WorthQueryApplicationValueBindingInstallationDenialKind,
    subject: String,
}

impl WorthQueryApplicationValueBindingInstallationDenial {
    pub(crate) fn new(
        kind: WorthQueryApplicationValueBindingInstallationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub(crate) const fn kind(&self) -> WorthQueryApplicationValueBindingInstallationDenialKind {
        self.kind
    }

    pub(crate) fn subject(&self) -> &str {
        &self.subject
    }
}
