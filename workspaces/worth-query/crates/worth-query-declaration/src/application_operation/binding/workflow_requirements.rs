use crate::application_capability::ApplicationCapabilityMarkerIdentity;

/// Explicit applicability and evidence for one mutation binding. Descriptive
/// meaning only: native admission still supplies every execution right.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationWorkflowExecutionRequirements {
    /// This binding does not execute under a workflow occurrence.
    NotApplicable,
    /// Requires the exact native occurrence, with no additional approval minimum.
    OccurrenceOnly,
    /// Requires the named installed native approval capability.
    Approval {
        /// Capability declaration name.
        capability: String,
        /// Capability portable type identity.
        identity: String,
    },
}
impl ApplicationWorkflowExecutionRequirements {
    /// Explicit applicability for a non-workflow binding.
    pub const fn not_applicable() -> Self {
        Self::NotApplicable
    }
    /// Exact occurrence execution without an additional approval minimum.
    pub const fn occurrence_only() -> Self {
        Self::OccurrenceOnly
    }
    /// Exact occurrence execution with this mandatory installed approval.
    pub fn requiring_approval<C: ApplicationCapabilityMarkerIdentity>() -> Self {
        Self::Approval {
            capability: C::IDENTIFIER.to_owned(),
            identity: C::PORTABLE_TYPE_IDENTITY.as_str().to_owned(),
        }
    }
    /// Whether this binding requires workflow occurrence authority.
    pub const fn is_applicable(&self) -> bool {
        !matches!(self, Self::NotApplicable)
    }
    /// The mandatory approval name and portable type identity, when declared.
    pub fn approval(&self) -> Option<(&str, &str)> {
        match self {
            Self::Approval {
                capability,
                identity,
            } => Some((capability, identity)),
            _ => None,
        }
    }
    /// Current explicit applicability identity.
    pub fn persistence_identity(&self) -> String {
        match self {
            Self::NotApplicable => "2:not-applicable".to_owned(),
            Self::OccurrenceOnly => "2:occurrence".to_owned(),
            Self::Approval {
                capability,
                identity,
            } => format!(
                "2:approval:{}:{}:{capability}{identity}",
                capability.len(),
                identity.len()
            ),
        }
    }
    /// Parses only current explicit applicability identities.
    pub fn from_persistence_identity(value: &str) -> Option<Self> {
        match value {
            "2:not-applicable" => return Some(Self::NotApplicable),
            "2:occurrence" => return Some(Self::OccurrenceOnly),
            _ => {}
        }
        let encoded = value.strip_prefix("2:approval:")?;
        let (name_len, encoded) = encoded.split_once(':')?;
        let (identity_len, encoded) = encoded.split_once(':')?;
        let name_len = name_len.parse::<usize>().ok()?;
        let identity_len = identity_len.parse::<usize>().ok()?;
        let name = encoded.get(..name_len)?;
        let identity = encoded.get(name_len..name_len.checked_add(identity_len)?)?;
        let result = Self::Approval {
            capability: name.to_owned(),
            identity: identity.to_owned(),
        };
        (!name.trim().is_empty()
            && !identity.trim().is_empty()
            && result.persistence_identity() == value)
            .then_some(result)
    }
}
