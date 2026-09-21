use std::borrow::Cow;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowSpecIdentity(Cow<'static, str>);

impl ApplicationWorkflowSpecIdentity {
    pub const fn new(identity: &'static str) -> Self {
        Self(Cow::Borrowed(identity))
    }

    pub const fn as_str(&self) -> &str {
        match &self.0 {
            Cow::Borrowed(value) => value,
            Cow::Owned(value) => value.as_str(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowDefinitionIdentity(String);

impl ApplicationWorkflowDefinitionIdentity {
    pub fn new(identity: impl Into<String>) -> Result<Self, String> {
        let identity = identity.into();
        require_identity(&identity)?;
        Ok(Self(identity))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowNodeIdentity(String);

impl ApplicationWorkflowNodeIdentity {
    pub fn new(identity: impl Into<String>) -> Result<Self, String> {
        let identity = identity.into();
        require_identity(&identity)?;
        Ok(Self(identity))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(super) fn prefixed(&self, prefix: &str) -> Result<Self, String> {
        Self::new(format!("{prefix}/{}", self.as_str()))
    }
}

/// Canonical content identity of validated workflow meaning.
///
/// This value is descriptive and grants no publication or execution authority.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowDefinitionContentIdentity(pub(super) [u8; 32]);

impl ApplicationWorkflowDefinitionContentIdentity {
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for ApplicationWorkflowDefinitionContentIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

pub(super) fn require_identity(identity: &str) -> Result<(), String> {
    if identity.is_empty()
        || identity.trim() != identity
        || identity
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        Err(identity.to_owned())
    } else {
        Ok(())
    }
}
