//! Immutable descriptive configuration identity, shared by installation and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthenticationVerifierIdentity(String);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthenticationVerifierIdentityDenial;
impl WorthQueryAuthenticationVerifierIdentity {
    pub fn new(
        value: impl Into<String>,
    ) -> Result<Self, WorthQueryAuthenticationVerifierIdentityDenial> {
        let value = value.into();
        Self::validate(&value)?;
        Ok(Self(value))
    }
    pub fn validate(value: &str) -> Result<(), WorthQueryAuthenticationVerifierIdentityDenial> {
        if value.is_empty()
            || value.len() > 512
            || value.trim() != value
            || value.chars().any(char::is_control)
        {
            return Err(WorthQueryAuthenticationVerifierIdentityDenial);
        }
        Ok(())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
