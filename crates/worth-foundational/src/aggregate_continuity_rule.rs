/// Closed consumer-driven continuity rule for group split/merge keeper selection.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum AggregateContinuityRule {
    /// Follows its anchor carrier (e.g., a loop follows its host face).
    Anchor,
    /// Follows its enclosing boundary members (e.g., a space follows its walls).
    Boundary,
    /// Follows its owning producer role (e.g., a shell follows its body feature).
    ProducerRole,
}

impl AggregateContinuityRule {
    /// Stable canonical byte hashed into every v2 group identity.
    pub const fn canonical_tag(self) -> u8 {
        match self {
            Self::Anchor => 1,
            Self::Boundary => 2,
            Self::ProducerRole => 3,
        }
    }

    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Anchor => "anchor",
            Self::Boundary => "boundary",
            Self::ProducerRole => "producer-role",
        }
    }
}
