use super::ApplicationOperationDependencyLocus;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationSchemaDeclarationDenial {
    InvalidCorrectionMechanism {
        correction: String,
        cause: crate::application_aftermath::DeclaredCorrectionMechanismDenial,
    },

    InvalidQueryDefinition {
        owner: String,
        query: String,
        cause: crate::application_query::ApplicationQueryDefinitionDenial,
    },
    InvalidCorrectionDemand {
        correction: String,
        cause: crate::application_aftermath::DeclaredPreImageDemandDenial,
    },

    MissingReferencedMutation {
        operation: String,
        binding: String,
    },
    InvalidPrincipalBindingField {
        binding: String,
        field: super::ApplicationFieldBindingLocus,
    },
    InvalidCorrectionField {
        correction: String,
        field: super::ApplicationFieldBindingLocus,
    },
    ExtensionRelationMinimum {
        path: String,
    },
    UndeclaredExtensionEntity {
        path: String,
    },
    DuplicateExtensionMember {
        first: String,
        path: String,
    },
    DuplicateFragmentMember {
        path: String,
    },
    FragmentIdentityCollision {
        identity: u64,
        first: String,
        second: String,
    },
    InvalidRecord {
        path: String,
    },
    InvalidRecordRevision,
    DuplicateOneOfRow {
        entity: String,
        group: &'static str,
        row: &'static str,
    },
    InvalidRecordMember {
        path: String,
    },
    InvalidRecordField {
        field: super::ApplicationFieldBindingLocus,
    },
    InvalidIdentifier,
    InvalidCanonicalOrdering,
    DuplicateMember,
    ConflictingFieldBinding,
    InvalidContentFormat,
    DuplicateContentFormat,
    MissingContentFormat,
    InvalidContentField,
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
    InvalidComponentSet,
    InvalidManagedIndex {
        index: String,
        cause: crate::application_query::ApplicationManagedIndexDefinitionDenial,
    },
    UnsupportedInvariantSource {
        invariant: String,
        index: String,
    },
    InvalidProjectedMemberDomain {
        domain: String,
        cause: super::ApplicationProjectedMemberDomainDefinitionDenial,
    },
    InvalidOperationComponentRead,
    MissingPrincipalBindingDependency,
    DuplicateApplicationQuery,
    MissingApplicationQueryDependency,
    InvalidApplicationQuery,
    InvalidApplicationHistoryQuery,
    InvalidApplicationHistory,
    MissingApplicationDocumentHistoryDependency,
    InvalidApplicationResource,
    MissingApplicationResourceDependency,
    DuplicateApplicationResource,
    MissingApplicationHistoryDependency,
    DuplicateApplicationHistoryQuery,
    InvalidApplicationQueryInspection,
    MissingApplicationQueryInspectionDependency,
    DuplicateApplicationQueryInspection,
    InvalidApplicationRelationAspect,
    InvalidApplicationDerivedRelation {
        relation: String,
        cause: super::ApplicationDerivedRelationDefinitionDenial,
    },
    MissingApplicationRelationAspectDependency,
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
