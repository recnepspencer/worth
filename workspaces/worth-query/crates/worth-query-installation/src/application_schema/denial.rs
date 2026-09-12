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
    OperationInstallationDenied,
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
}

impl WorthQueryInstalledApplicationSchemaDenial {
    pub(crate) fn new(
        kind: WorthQueryInstalledApplicationSchemaDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryInstalledApplicationSchemaDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
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
