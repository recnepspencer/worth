//! Positive presence and comparison of an exact native workflow occurrence.
//! These declarations carry no principal identity or execution authority.

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// The original admitted actor of one canonical native workflow occurrence.
pub enum ApplicationWorkflowRecordedRole {
    /// Principal admitted when this proposal was published.
    Proposer,
    /// Principal admitted for the command that recorded the transition.
    Advancer,
    /// Principal that actually performed the guarded effect, when performed.
    Executor,
    /// Principal admitted for the original approval decision.
    Approver,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Exact source selected from the original typed command, never by subject lookup.
pub enum ApplicationWorkflowRecordedActorSource {
    /// The proposal consumed by the bound command.
    BoundProposal,
    /// The original input transition consumed by the bound operation.
    BoundTransition,
    /// The approval occurrence consumed by the bound operation.
    BoundApproval,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Comparison against the freshly admitted acting principal, using the full entity identity.
pub enum ApplicationRecordedActorComparison {
    /// Requires positive presence and equality of partition, slot and generation.
    SamePrincipal,
    /// Requires positive presence and inequality of the complete identity.
    DifferentPrincipal,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// A positive native occurrence requirement in the existing capability decision.
/// Missing or inapplicable roles never satisfy either comparison.
pub struct ApplicationCapabilityRecordedActorRequirement {
    source: ApplicationWorkflowRecordedActorSource,
    role: ApplicationWorkflowRecordedRole,
    comparison: ApplicationRecordedActorComparison,
}
impl ApplicationCapabilityRecordedActorRequirement {
    /// Declares a role comparison. Installation refuses invalid source/role pairs.
    pub const fn new(
        source: ApplicationWorkflowRecordedActorSource,
        role: ApplicationWorkflowRecordedRole,
        comparison: ApplicationRecordedActorComparison,
    ) -> Self {
        Self {
            source,
            role,
            comparison,
        }
    }
    /// The exact original occurrence source.
    pub const fn source(self) -> ApplicationWorkflowRecordedActorSource {
        self.source
    }
    /// The recorded role to read from that source.
    pub const fn role(self) -> ApplicationWorkflowRecordedRole {
        self.role
    }
    /// The required full-identity comparison.
    pub const fn comparison(self) -> ApplicationRecordedActorComparison {
        self.comparison
    }
    /// Whether the role exists on the declared source kind.
    pub const fn is_valid(self) -> bool {
        use ApplicationWorkflowRecordedActorSource::*;
        use ApplicationWorkflowRecordedRole::*;
        matches!(
            (self.source, self.role),
            (BoundProposal, Proposer)
                | (BoundApproval, Approver)
                | (BoundTransition, Advancer | Executor)
        )
    }
    /// Current canonical declaration identity; invalid pairs cannot install.
    pub const fn persistence_identity(self) -> &'static str {
        use ApplicationRecordedActorComparison::*;
        use ApplicationWorkflowRecordedActorSource::*;
        use ApplicationWorkflowRecordedRole::*;
        match (self.source, self.role, self.comparison) {
            (BoundProposal, Proposer, SamePrincipal) => "1:proposal:proposer:same",
            (BoundProposal, Proposer, DifferentPrincipal) => "1:proposal:proposer:different",
            (BoundApproval, Approver, SamePrincipal) => "1:approval:approver:same",
            (BoundApproval, Approver, DifferentPrincipal) => "1:approval:approver:different",
            (BoundTransition, Advancer, SamePrincipal) => "1:transition:advancer:same",
            (BoundTransition, Advancer, DifferentPrincipal) => "1:transition:advancer:different",
            (BoundTransition, Executor, SamePrincipal) => "1:transition:executor:same",
            (BoundTransition, Executor, DifferentPrincipal) => "1:transition:executor:different",
            _ => "invalid",
        }
    }
    /// Parses only a current, valid declaration identity.
    pub fn from_persistence_identity(value: &str) -> Option<Self> {
        use ApplicationRecordedActorComparison::*;
        use ApplicationWorkflowRecordedActorSource::*;
        use ApplicationWorkflowRecordedRole::*;
        for (source, role) in [
            (BoundProposal, Proposer),
            (BoundApproval, Approver),
            (BoundTransition, Advancer),
            (BoundTransition, Executor),
        ] {
            for comparison in [SamePrincipal, DifferentPrincipal] {
                let requirement = Self::new(source, role, comparison);
                if requirement.persistence_identity() == value {
                    return Some(requirement);
                }
            }
        }
        None
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Explicit applicability of native recorded-actor evidence.
pub enum ApplicationCapabilityRecordedActorRule {
    /// This capability has no recorded-actor requirement.
    NotApplicable,
    /// Positive presence and comparison participate in the native Required clause.
    Required(ApplicationCapabilityRecordedActorRequirement),
}
impl ApplicationCapabilityRecordedActorRule {
    /// Explicitly declares no recorded-actor rule.
    pub const fn not_applicable() -> Self {
        Self::NotApplicable
    }
    /// Requires the exact native source and comparison before access is minted.
    pub const fn required(value: ApplicationCapabilityRecordedActorRequirement) -> Self {
        Self::Required(value)
    }
    /// The declared positive rule, when applicable.
    pub const fn requirement(&self) -> Option<ApplicationCapabilityRecordedActorRequirement> {
        match self {
            Self::NotApplicable => None,
            Self::Required(value) => Some(*value),
        }
    }
}
