use crate::application_query::WorthQueryApplicationQueryInstallationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDocumentHistoryInstallationDenialKind {
    NotInstalled,
    MissingTypedBinding,
    BindingMeaningChanged,
    AccessBindingMismatch,
    OperationBindingMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDocumentHistoryInstallationDenial {
    kind: WorthQueryDocumentHistoryInstallationDenialKind,
    subject: String,
    access: Option<WorthQueryApplicationQueryInstallationDenial>,
}

impl WorthQueryDocumentHistoryInstallationDenial {
    pub(super) fn new(
        kind: WorthQueryDocumentHistoryInstallationDenialKind,
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
            kind: WorthQueryDocumentHistoryInstallationDenialKind::AccessBindingMismatch,
            subject: subject.to_owned(),
            access: Some(cause),
        }
    }

    pub const fn kind(&self) -> WorthQueryDocumentHistoryInstallationDenialKind {
        self.kind
    }
    pub fn subject(&self) -> &str {
        &self.subject
    }
    pub fn access_binding_denial(&self) -> Option<&WorthQueryApplicationQueryInstallationDenial> {
        self.access.as_ref()
    }
}
