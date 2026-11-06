#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledApplicationSchemaDenialKind {
    DomainNotInstalled,
    SchemaNotInstalled,
    SchemaMeaningChanged,
    ForeignRuntime,
    StaleGeneration,
    PackageIdentityChanged,
    AdmissionIdentityChanged,
    AuthorityMismatch,
    CapabilityInstallationDenied,
    QueryInstallationDenied,
    InvalidInspectionDefinition,
    InspectionBindingMeaningChanged,
    InspectionUnavailable,
    InvalidManagedIndexDefinition,
    ManagedIndexBindingMeaningChanged,
    ManagedIndexSourceUnsupported,
    ManagedIndexParentUnsupported,
    ManagedIndexUnavailable,
    ResourceInstallationDenied,
    HistoryInstallationDenied,

    OperationInstallationDenied,
    PreImageDemandNotCoveredByAuthoredAnchors,
    DerivedSourceUnavailable,
    ContentBindingMissing,
    ContentFormatDuplicate,
    ContentFormatMismatch,
    FieldBindingMissing,
    FieldBindingUnexpected,
    FieldBindingContractMismatch,
    FieldBindingNativeContractMissing,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledApplicationSchemaDenial {
    kind: WorthQueryInstalledApplicationSchemaDenialKind,
    subject: String,
    canonical_denial: Option<worth_foundational::facade::CanonicalDigestDerivationDenial>,
}

impl WorthQueryInstalledApplicationSchemaDenial {
    pub(crate) fn new(
        kind: WorthQueryInstalledApplicationSchemaDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
            canonical_denial: None,
        }
    }

    pub const fn kind(&self) -> WorthQueryInstalledApplicationSchemaDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub const fn canonical_denial(
        &self,
    ) -> Option<&worth_foundational::facade::CanonicalDigestDerivationDenial> {
        self.canonical_denial.as_ref()
    }

    pub(crate) fn with_canonical_denial(
        mut self,
        cause: worth_foundational::facade::CanonicalDigestDerivationDenial,
    ) -> Self {
        self.canonical_denial = Some(cause);
        self
    }
}

impl std::fmt::Display for WorthQueryInstalledApplicationSchemaDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "installed application schema denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryInstalledApplicationSchemaDenial {}
