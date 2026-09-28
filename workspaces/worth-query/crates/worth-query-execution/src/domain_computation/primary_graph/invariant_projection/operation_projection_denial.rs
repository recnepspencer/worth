use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

/// Why an invariant projection for an admitted operation was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationProjectionDenialKind {
    /// The admission does not authorize this projection; the full denial is
    /// attached.
    Authorization(WorthQueryOperationAuthorizationDenialKind),
    /// The projection snapshot could not be taken.
    InvariantAdmission(super::WorthQueryInvariantProjectionDenialKind),
    /// The projection exceeded the operation's projection work budget.
    WorkBudgetExceeded,
}

/// Refusal to run an invariant projection for an admitted operation. No output
/// or snapshot is returned.
///
/// [`Self::kind`] says why; for authorization causes,
/// [`Self::authorization_denial`] carries the full denial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationProjectionDenial {
    kind: WorthQueryOperationProjectionDenialKind,
    authorization_denial: Option<Box<WorthQueryOperationAuthorizationDenial>>,
    subject: String,
}

impl WorthQueryOperationProjectionDenial {
    pub(super) fn from_invariant(
        denial: super::WorthQueryInvariantProjectionDenial,
        subject: impl Into<String>,
    ) -> Self {
        let kind = match denial.kind() {
            super::WorthQueryInvariantProjectionDenialKind::WorkBudgetExceeded => {
                WorthQueryOperationProjectionDenialKind::WorkBudgetExceeded
            }
            kind => WorthQueryOperationProjectionDenialKind::InvariantAdmission(kind),
        };
        Self {
            kind,
            authorization_denial: None,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryOperationProjectionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn authorization_denial(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        self.authorization_denial.as_deref()
    }

    pub fn into_authorization_denial(self) -> Option<WorthQueryOperationAuthorizationDenial> {
        self.authorization_denial.map(|denial| *denial)
    }
}

impl From<WorthQueryOperationAuthorizationDenial> for WorthQueryOperationProjectionDenial {
    fn from(denial: WorthQueryOperationAuthorizationDenial) -> Self {
        Self {
            kind: WorthQueryOperationProjectionDenialKind::Authorization(denial.kind()),
            subject: denial.subject().to_string(),
            authorization_denial: Some(Box::new(denial)),
        }
    }
}

impl std::fmt::Display for WorthQueryOperationProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application operation projection denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryOperationProjectionDenial {}
