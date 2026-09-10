use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationProjectionDenialKind {
    Authorization(WorthQueryOperationAuthorizationDenialKind),
    InvariantAdmission(super::WorthQueryInvariantProjectionDenialKind),
    WorkBudgetExceeded,
}

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
