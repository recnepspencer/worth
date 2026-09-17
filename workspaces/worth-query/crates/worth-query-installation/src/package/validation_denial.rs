use super::WorthQueryPortableDefinitionKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPortablePackageValidationDenialKind {
    EmptyDomainOwner,
    EmptyDefinitionSlot,
    EmptyDefinitionSemantics,
    EmptyRequirement,
    DuplicateContributionCategory,
    DuplicateDefinition,
    ConflictingDefinition,
    InvalidDomainOperation,
    DuplicateArtifactContract,
    ConflictingArtifactContract,
    ApplicationSchemaIdentityMismatch,
    DuplicateApplicationSchema,
    ConflictingApplicationSchema,
    ApplicationContractDuplicateAspectIdentity,
    ApplicationContractDuplicateAspectLocus,
    ApplicationContractDuplicateFieldLocus,
    ApplicationContractRevisionZero,
    ApplicationContractMissingAspectFieldClosure,
    ApplicationContractFieldWithoutAspect,
    ApplicationContractInvalidAspectKey,
    ApplicationContractInvalidFieldKey,
    ApplicationContractInvalidAspectShape,
    ApplicationContractProjectionMaskRejected,
    ApplicationOperationContractMissingNativeAspect,
    ApplicationOperationContractMissingNativeField,
    ApplicationOperationContractInvalidProjectionMask,
    ApplicationOperationContractAmbiguousExternalEffect,
    ApplicationOperationContractAmbiguousAftermath,
    DuplicateConditionalApplicationOperation,
    ConflictingConditionalApplicationOperation,
    ConditionalApplicationSchemaMissing,
    ConditionalApplicationOperationMissing,
    ConditionalApplicationOperationChanged,
    ConditionalDomainOperationMissing,
    ConditionalDomainOperationChanged,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortablePackageValidationDenial {
    kind: WorthQueryPortablePackageValidationDenialKind,
    definition_kind: Option<WorthQueryPortableDefinitionKind>,
    slot: String,
    maximum_canonical_bytes: Option<usize>,
    attempted_canonical_bytes: Option<usize>,
}

impl WorthQueryPortablePackageValidationDenial {
    pub(super) fn empty_domain_owner() -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::EmptyDomainOwner,
            None,
            "domain-owner",
        )
    }

    pub(super) fn empty_definition_slot(kind: WorthQueryPortableDefinitionKind) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::EmptyDefinitionSlot,
            Some(kind),
            "definition-slot",
        )
    }

    pub(super) fn empty_definition_semantics(
        kind: WorthQueryPortableDefinitionKind,
        slot: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::EmptyDefinitionSemantics,
            Some(kind),
            slot,
        )
    }

    pub(super) fn empty_requirement(slot: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::EmptyRequirement,
            None,
            slot,
        )
    }

    pub(super) fn duplicate_contribution_category() -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::DuplicateContributionCategory,
            None,
            "contribution",
        )
    }

    pub(super) fn duplicate_definition(
        kind: WorthQueryPortableDefinitionKind,
        slot: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::DuplicateDefinition,
            Some(kind),
            slot,
        )
    }

    pub(super) fn conflicting_definition(
        kind: WorthQueryPortableDefinitionKind,
        slot: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ConflictingDefinition,
            Some(kind),
            slot,
        )
    }

    pub(super) fn invalid_domain_operation(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::InvalidDomainOperation,
            Some(WorthQueryPortableDefinitionKind::DomainOperation),
            subject,
        )
    }

    pub(super) fn duplicate_artifact_contract(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::DuplicateArtifactContract,
            None,
            subject,
        )
    }

    pub(super) fn conflicting_artifact_contract(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ConflictingArtifactContract,
            None,
            subject,
        )
    }

    pub(super) fn application_schema_identity_mismatch(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ApplicationSchemaIdentityMismatch,
            None,
            subject,
        )
    }

    pub(super) fn duplicate_application_schema(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::DuplicateApplicationSchema,
            None,
            subject,
        )
    }

    pub(super) fn conflicting_application_schema(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ConflictingApplicationSchema,
            None,
            subject,
        )
    }

    pub(super) fn invalid_application_contract_spine(
        kind: WorthQueryPortablePackageValidationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self::new(kind, None, subject)
    }

    pub(super) fn duplicate_conditional_application_operation(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::DuplicateConditionalApplicationOperation,
            None,
            subject,
        )
    }

    pub(super) fn conflicting_conditional_application_operation(
        subject: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ConflictingConditionalApplicationOperation,
            None,
            subject,
        )
    }

    pub(super) fn conditional_application_schema_missing(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ConditionalApplicationSchemaMissing,
            None,
            subject,
        )
    }

    pub(super) fn conditional_application_operation_missing(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ConditionalApplicationOperationMissing,
            None,
            subject,
        )
    }

    pub(super) fn conditional_application_operation_changed(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ConditionalApplicationOperationChanged,
            None,
            subject,
        )
    }

    pub(super) fn conditional_domain_operation_missing(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ConditionalDomainOperationMissing,
            Some(WorthQueryPortableDefinitionKind::DomainOperation),
            subject,
        )
    }

    pub(super) fn conditional_domain_operation_changed(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::ConditionalDomainOperationChanged,
            Some(WorthQueryPortableDefinitionKind::DomainOperation),
            subject,
        )
    }

    pub(super) fn canonical_entry_budget_exceeded() -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::CanonicalEntryBudgetExceeded,
            None,
            "package-canonical-entry-budget",
        )
    }

    pub(super) fn canonical_encoded_byte_budget_exceeded(maximum: usize, attempted: usize) -> Self {
        let mut denial = Self::new(
            WorthQueryPortablePackageValidationDenialKind::CanonicalEncodedByteBudgetExceeded,
            None,
            "package-canonical-encoded-byte-budget",
        );
        denial.maximum_canonical_bytes = Some(maximum);
        denial.attempted_canonical_bytes = Some(attempted);
        denial
    }

    pub(super) fn canonical_digest_slot_rejected() -> Self {
        Self::new(
            WorthQueryPortablePackageValidationDenialKind::CanonicalDigestSlotRejected,
            None,
            "package-canonical-digest-slot",
        )
    }

    fn new(
        kind: WorthQueryPortablePackageValidationDenialKind,
        definition_kind: Option<WorthQueryPortableDefinitionKind>,
        slot: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            definition_kind,
            slot: slot.into(),
            maximum_canonical_bytes: None,
            attempted_canonical_bytes: None,
        }
    }

    pub fn kind(&self) -> WorthQueryPortablePackageValidationDenialKind {
        self.kind
    }

    pub fn definition_kind(&self) -> Option<WorthQueryPortableDefinitionKind> {
        self.definition_kind
    }

    pub fn slot(&self) -> &str {
        &self.slot
    }

    pub const fn maximum_canonical_bytes(&self) -> Option<usize> {
        self.maximum_canonical_bytes
    }

    pub const fn attempted_canonical_bytes(&self) -> Option<usize> {
        self.attempted_canonical_bytes
    }
}

impl std::fmt::Display for WorthQueryPortablePackageValidationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "portable package validation denied: {:?} ({}){}",
            self.kind,
            self.slot,
            match (self.maximum_canonical_bytes, self.attempted_canonical_bytes) {
                (Some(maximum), Some(attempted)) => {
                    format!("; canonical bytes {attempted} exceed {maximum}")
                }
                _ => String::new(),
            }
        )
    }
}

impl std::error::Error for WorthQueryPortablePackageValidationDenial {}
