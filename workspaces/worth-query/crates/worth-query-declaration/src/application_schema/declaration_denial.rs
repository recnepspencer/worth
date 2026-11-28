use super::ApplicationOperationDependencyLocus;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationSchemaDeclarationDenial {
    ConflictingDecisionReadScope {
        scope: String,
    },
    UndeclaredDecisionReadScope {
        scope: String,
    },
    MissingDecisionReadScope {
        operation: String,
        scope: String,
    },
    /// A shared decision-read scope names a peer-scope Parent read; only the
    /// reading operation's own declaration can grant one.
    SharedScopePeerRead,
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
    /// A recorded inverse would leave a field its source operation writes
    /// unreversed (a half-undo). Retain and restore the field, or grant the
    /// omission explicitly with `allow_omitted_reversal(Field)`.
    IncompleteCorrectionCoverage {
        operation: String,
        entity: String,
        aspect: String,
        field: String,
        gap: super::ApplicationCorrectionCoverageGap,
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
    /// A query scoped on a field of its own root selects that root by the
    /// field, so the field must be a key: declare it
    /// `#[worth_query(equality, unique)]` (or make it an immutable
    /// `read_only` identity). A mutable, non-unique field such as a status
    /// cannot select one root; to list the roots sharing a value, scope the
    /// query on another entity or filter by a parameter.
    RootScopedQueryRequiresKeyField {
        binding: String,
        query: String,
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
    /// Standing currentness without a standing holder (or the reverse), a
    /// standing grant that may be delegated or activated, or a self-grant
    /// rule on a delegable workflow-tenure grant.
    InvalidApplicationCapabilityTenure {
        capability: String,
    },
    /// A `require_active` binding names the grant kind, repeats a kind,
    /// exceeds the installed ceiling, or names a kind the resource path
    /// never crosses.
    InvalidApplicationCapabilityExtentStatus {
        capability: String,
    },
    /// An open grant validity bound is a standing grant's: a workflow-tenure
    /// grant may be delegated, and a child's window is proven inside its
    /// parent's only when both are closed.
    OpenApplicationCapabilityValidity {
        capability: String,
    },
    /// Governed elevation adds durations to its bounds, so it needs a
    /// Unix-epoch timeline; UTC-timestamp and local-date grants cannot elevate.
    UnsupportedApplicationCapabilityElevationTimeline {
        capability: String,
    },
    /// A declarative body links its own admitted principal as the grantee of
    /// a grant kind declared `forbid_self_grant`.
    SelfGrantingOperationBody {
        operation: String,
    },
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
    /// A ReBAC rule could not lower; the cause names the authored path or condition.
    InvalidAuthorizationRule(super::ApplicationAuthorizationRuleDenial),
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
