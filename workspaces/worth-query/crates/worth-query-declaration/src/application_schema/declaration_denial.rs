use super::ApplicationOperationDependencyLocus;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationSchemaDeclarationDenial {
    InvalidIdentifier,
    InvalidCanonicalOrdering,
    DuplicateMember,
    ConflictingFieldBinding,
    InvalidContributionIdentity,
    DuplicateContributionIdentity,
    EmptyContributionClosure,
    InvalidContributionOrdering,
    InvalidContributionMemberOrdinal,
    OverlappingContributionClosure,
    IncompleteContributionClosure,
    InvalidApplicationMutationDescription,
    MissingApplicationMutationDependency,
    MissingEntity,
    MissingAspect,
    MissingUnit,
    UniqueFieldWithoutEqualityIndex,
    InvalidRelationIntegrity,
    MissingPrincipalBindingDependency,
    DuplicateApplicationQuery,
    MissingApplicationQueryDependency,
    InvalidApplicationQuery,
    DuplicateApplicationCapability,
    MissingApplicationCapabilityDependency,
    InvalidApplicationCapability,
    InvalidApplicationCapabilityDelegationActivationProgram,
    DelegationActivationIdentityNotUnique,
    DuplicateApplicationCapabilityContext,
    DuplicateApplicationCapabilityContextSlot,
    DuplicateApplicationCapabilityProvenance,
    MissingApplicationCapabilityContextDependency,
    MissingApplicationCapabilityProvenanceDependency,
    MissingOperationProgramDependency {
        operation: String,
        dependency: ApplicationOperationDependencyLocus,
    },
    MissingOperationDecisionReadDependency {
        operation: String,
        dependency: ApplicationOperationDependencyLocus,
    },
    MissingOperationMutationPreconditionDependency {
        operation: String,
        dependency: ApplicationOperationDependencyLocus,
    },
    InvalidOperationProjectionWorkBudget,
    DuplicateOperationExternalEffect,
    DuplicateOperationAftermath,
    MissingAbilityDependency,
    MissingAbilityPolicyDependency,
    InvalidAbilityPolicy,
    PortableCanonicalSourceBytesBudgetExceeded {
        observed: u64,
        maximum: u64,
    },
    PortableCanonicalEntryBudgetExceeded {
        observed: u64,
        maximum: u64,
    },
}

impl std::fmt::Display for ApplicationSchemaDeclarationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "application schema declaration denied: {self:?}")
    }
}

impl std::error::Error for ApplicationSchemaDeclarationDenial {}

impl ApplicationSchemaDeclarationDenial {
    pub(crate) fn program_body_dependency(operation: &str) -> Self {
        Self::MissingOperationProgramDependency {
            operation: operation.to_owned(),
            dependency: ApplicationOperationDependencyLocus::Body,
        }
    }

    pub(crate) fn decision_read_body_dependency(operation: &str) -> Self {
        Self::MissingOperationDecisionReadDependency {
            operation: operation.to_owned(),
            dependency: ApplicationOperationDependencyLocus::Body,
        }
    }
}
