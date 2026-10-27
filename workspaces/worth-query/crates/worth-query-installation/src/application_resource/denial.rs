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
}
