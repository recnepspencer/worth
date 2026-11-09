use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramError {
    message: String,
    identity_derivation_denial: Option<IdentityDerivationDenial>,
}

impl WorthQueryProgramError {
    pub fn identity_derivation_denial(&self) -> Option<IdentityDerivationDenial> {
        self.identity_derivation_denial
    }
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            identity_derivation_denial: None,
        }
    }
}

impl std::fmt::Display for WorthQueryProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WorthQueryProgramError {}

impl From<IdentityDerivationDenial> for WorthQueryProgramError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self {
            message: format!("program evidence identity refused: {cause:?}"),
            identity_derivation_denial: Some(cause),
        }
    }
}
