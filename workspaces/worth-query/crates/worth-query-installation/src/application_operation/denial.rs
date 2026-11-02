#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationOperationInstallationDenialKind {
    MutationBindingIdentityCollision,
    MutationBindingNotInstalled,
    MutationBindingMeaningChanged,
    MutationBindingOperationNotInstalled,
    MutationBindingOperationMeaningChanged,
    MutationBindingPrincipalMeaningChanged,
    MutationBindingResultMeaningChanged,
    MutationBindingScopeMeaningChanged,
    MutationBindingHandlerMeaningChanged,
    MutationBindingIdempotencyMeaningChanged,
    MutationBindingOutputMeaningChanged,
    MutationSourceQueryNotInstalled,
    InvalidSourceDisclosureBinding,
    SourceDisclosureQueryNotInstalled,
    SourceDisclosureMeaningChanged,
    InvalidMutationBindingIdentity,
    InvalidMutationHandlerIdentity,
    InvalidMutationIdempotencyIdentity,
    InvalidMutationOutputRole,
    MutationOutputEntityNotInstalled,
    OperationNotInstalled,
    OperationMeaningChanged,
    MissingAbilityPolicy,
    MissingProgram,
    ConflictingAuthorizationContract,
    InvalidMutationPreconditionContract,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
    InvalidGraphObligationContract,
    AftermathInstallationDenied,
    AmbiguousExternalEffectContract,
    AmbiguousAftermathContract,
    ForeignRuntime,
    StaleGeneration,
    SchemaMeaningChanged,
    PackageIdentityChanged,
    AuthorityMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationOperationInstallationDenial {
    kind: WorthQueryApplicationOperationInstallationDenialKind,
    operation: String,
    canonical_denial: Option<worth_foundational::facade::CanonicalDigestDerivationDenial>,
}

impl WorthQueryApplicationOperationInstallationDenial {
    pub(crate) fn new(
        kind: WorthQueryApplicationOperationInstallationDenialKind,
        operation: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            operation: operation.into(),
            canonical_denial: None,
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationOperationInstallationDenialKind {
        self.kind
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    /// Exact canonical refusal, including requested and maximum entry/byte counts.
    /// The cause type is available through `worth_query_host::facade::domain`.
    pub const fn canonical_denial(
        &self,
    ) -> Option<&worth_foundational::facade::CanonicalDigestDerivationDenial> {
        self.canonical_denial.as_ref()
    }

    pub(super) fn with_canonical_denial(
        mut self,
        denial: worth_foundational::facade::CanonicalDigestDerivationDenial,
    ) -> Self {
        self.canonical_denial = Some(denial);
        self
    }
}

impl std::fmt::Display for WorthQueryApplicationOperationInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "installed application operation denied: {:?} ({})",
            self.kind, self.operation
        )?;
        if let Some(denial) = self.canonical_denial {
            write!(formatter, ": {denial:?}")?;
        }
        Ok(())
    }
}

impl std::error::Error for WorthQueryApplicationOperationInstallationDenial {}
