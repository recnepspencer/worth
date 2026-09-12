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
    MutationBindingCandidateMeaningChanged,
    MutationBindingHandlerMeaningChanged,
    MutationBindingIdempotencyMeaningChanged,
    MutationBindingOutputMeaningChanged,
    InvalidMutationBindingIdentity,
    InvalidMutationHandlerIdentity,
    InvalidMutationIdempotencyIdentity,
    InvalidMutationOutputRole,
    MutationOutputEntityNotInstalled,
    MutationOutputCandidateMismatch,
    OperationNotInstalled,
    OperationMeaningChanged,
    MissingAbilityPolicy,
    MissingProgram,
    MissingDecisionFactBudget,
    MissingProjectionWorkBudget,
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
}

impl WorthQueryApplicationOperationInstallationDenial {
    pub(crate) fn new(
        kind: WorthQueryApplicationOperationInstallationDenialKind,
        operation: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            operation: operation.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationOperationInstallationDenialKind {
        self.kind
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }
}

impl std::fmt::Display for WorthQueryApplicationOperationInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "installed application operation denied: {:?} ({})",
            self.kind, self.operation
        )
    }
}

impl std::error::Error for WorthQueryApplicationOperationInstallationDenial {}
