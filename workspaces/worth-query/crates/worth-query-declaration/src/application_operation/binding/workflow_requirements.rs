use crate::application_capability::ApplicationCapabilityMarkerIdentity;

/// Supplier-owned evidence required in addition to exact workflow occurrence
/// binding and ordinary operation authorization. This is meaning, not a permit.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationWorkflowExecutionRequirements {
    approval: Option<(String, String)>,
}

impl ApplicationWorkflowExecutionRequirements {
    /// Explicitly declares no additional workflow evidence. This does not make
    /// a guarded operation callable outside its current workflow occurrence.
    pub const fn new() -> Self {
        Self { approval: None }
    }

    /// Requires the existing native approval contract for this capability.
    pub fn requiring_approval<Capability>() -> Self
    where
        Capability: ApplicationCapabilityMarkerIdentity,
    {
        Self {
            approval: Some((
                Capability::IDENTIFIER.to_owned(),
                Capability::PORTABLE_TYPE_IDENTITY.as_str().to_owned(),
            )),
        }
    }

    pub fn approval(&self) -> Option<(&str, &str)> {
        self.approval
            .as_ref()
            .map(|(identifier, identity)| (identifier.as_str(), identity.as_str()))
    }

    /// Canonical portable declaration encoding, never execution authority.
    pub fn persistence_identity(&self) -> String {
        match self.approval() {
            None => "1:none".to_owned(),
            Some((name, identity)) => format!(
                "1:approval:{}:{}:{name}{identity}",
                name.len(),
                identity.len()
            ),
        }
    }

    pub fn from_persistence_identity(value: &str) -> Option<Self> {
        if value == "1:none" {
            return Some(Self::new());
        }
        let encoded = value.strip_prefix("1:approval:")?;
        let (name_len, encoded) = encoded.split_once(':')?;
        let (identity_len, encoded) = encoded.split_once(':')?;
        let name_len = name_len.parse::<usize>().ok()?;
        let identity_len = identity_len.parse::<usize>().ok()?;
        let name = encoded.get(..name_len)?;
        let identity = encoded.get(name_len..name_len.checked_add(identity_len)?)?;
        let requirements = Self {
            approval: Some((name.to_owned(), identity.to_owned())),
        };
        if name.trim().is_empty()
            || identity.trim().is_empty()
            || requirements.persistence_identity() != value
        {
            return None;
        }
        Some(requirements)
    }
}
