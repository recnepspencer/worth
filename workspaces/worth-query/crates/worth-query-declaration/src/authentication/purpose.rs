//! One descriptive purpose contract shared by declared signing and event issuance.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryAuthenticationPurpose(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthenticationPurposeDenial;

impl WorthQueryAuthenticationPurpose {
    pub fn new(value: impl Into<String>) -> Result<Self, WorthQueryAuthenticationPurposeDenial> {
        let value = value.into();
        Self::validate(&value)?;
        Ok(Self(value))
    }
    pub fn validate(value: &str) -> Result<(), WorthQueryAuthenticationPurposeDenial> {
        if value.is_empty()
            || value.trim() != value
            || value.len() > 256
            || value.chars().any(char::is_control)
        {
            return Err(WorthQueryAuthenticationPurposeDenial);
        }
        Ok(())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_string(self) -> String {
        self.0
    }
}
