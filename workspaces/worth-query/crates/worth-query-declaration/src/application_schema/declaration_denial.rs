#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    InvalidRelationIntegrity,
    MissingPrincipalBindingDependency,
    DuplicateApplicationQuery,
    MissingApplicationQueryDependency,
    InvalidApplicationQuery,
    DuplicateApplicationCapability,
    MissingApplicationCapabilityDependency,
    InvalidApplicationCapability,
    InvalidApplicationCapabilityDelegationActivationProgram,
    DuplicateApplicationCapabilityContext,
    DuplicateApplicationCapabilityContextSlot,
    DuplicateApplicationCapabilityProvenance,
    MissingApplicationCapabilityContextDependency,
    MissingApplicationCapabilityProvenanceDependency,
    MissingOperationProgramDependency,
    MissingOperationDecisionReadDependency,
    MissingOperationMutationPreconditionDependency,
    InvalidOperationDecisionFactBudget,
    InvalidOperationProjectionWorkBudget,
    DuplicateOperationExternalEffect,
    DuplicateOperationAftermath,
    MissingAbilityDependency,
    MissingAbilityPolicyDependency,
    InvalidAbilityPolicy,
    PortableCanonicalSourceBytesBudgetExceeded { observed: u64, maximum: u64 },
    PortableCanonicalEntryBudgetExceeded { observed: u64, maximum: u64 },
}

impl std::fmt::Display for ApplicationSchemaDeclarationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "application schema declaration denied: {self:?}")
    }
}

impl std::error::Error for ApplicationSchemaDeclarationDenial {}
