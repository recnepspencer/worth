use worth_query_host::facade::domain::WorthQueryApplicationOperationInstallationDenialKind as QueryKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankOperationInstallationDenialKind {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankOperationInstallationDenial {
    kind: BankOperationInstallationDenialKind,
}

impl BankOperationInstallationDenial {
    pub const fn kind(self) -> BankOperationInstallationDenialKind {
        self.kind
    }

    pub const fn code(self) -> &'static str {
        use BankOperationInstallationDenialKind as Bank;
        match self.kind {
            Bank::MutationBindingIdentityCollision => "mutation-binding-identity-collision",
            Bank::MutationBindingNotInstalled => "mutation-binding-not-installed",
            Bank::MutationBindingMeaningChanged => "mutation-binding-meaning-changed",
            Bank::MutationBindingOperationNotInstalled => {
                "mutation-binding-operation-not-installed"
            }
            Bank::MutationBindingOperationMeaningChanged => {
                "mutation-binding-operation-meaning-changed"
            }
            Bank::MutationBindingPrincipalMeaningChanged => {
                "mutation-binding-principal-meaning-changed"
            }
            Bank::MutationBindingResultMeaningChanged => "mutation-binding-result-meaning-changed",
            Bank::MutationBindingScopeMeaningChanged => "mutation-binding-scope-meaning-changed",
            Bank::MutationBindingCandidateMeaningChanged => {
                "mutation-binding-candidate-meaning-changed"
            }
            Bank::MutationBindingHandlerMeaningChanged => {
                "mutation-binding-handler-meaning-changed"
            }
            Bank::MutationBindingIdempotencyMeaningChanged => {
                "mutation-binding-idempotency-meaning-changed"
            }
            Bank::MutationBindingOutputMeaningChanged => "mutation-binding-output-meaning-changed",
            Bank::InvalidMutationBindingIdentity => "invalid-mutation-binding-identity",
            Bank::InvalidMutationHandlerIdentity => "invalid-mutation-handler-identity",
            Bank::InvalidMutationIdempotencyIdentity => "invalid-mutation-idempotency-identity",
            Bank::InvalidMutationOutputRole => "invalid-mutation-output-role",
            Bank::MutationOutputEntityNotInstalled => "mutation-output-entity-not-installed",
            Bank::MutationOutputCandidateMismatch => "mutation-output-candidate-mismatch",
            Bank::OperationNotInstalled => "operation-not-installed",
            Bank::OperationMeaningChanged => "operation-meaning-changed",
            Bank::MissingAbilityPolicy => "missing-ability-policy",
            Bank::MissingProgram => "missing-program",
            Bank::MissingDecisionFactBudget => "missing-decision-fact-budget",
            Bank::MissingProjectionWorkBudget => "missing-projection-work-budget",
            Bank::ConflictingAuthorizationContract => "conflicting-authorization-contract",
            Bank::InvalidMutationPreconditionContract => "invalid-mutation-precondition-contract",
            Bank::CanonicalEntryBudgetExceeded => "canonical-entry-budget-exceeded",
            Bank::CanonicalEncodedByteBudgetExceeded => "canonical-byte-budget-exceeded",
            Bank::CanonicalDigestSlotRejected => "canonical-digest-slot-rejected",
            Bank::InvalidGraphObligationContract => "invalid-graph-obligation-contract",
            Bank::AftermathInstallationDenied => "aftermath-installation-denied",
            Bank::AmbiguousExternalEffectContract => "ambiguous-external-effect-contract",
            Bank::AmbiguousAftermathContract => "ambiguous-aftermath-contract",
            Bank::ForeignRuntime => "foreign-runtime",
            Bank::StaleGeneration => "stale-generation",
            Bank::SchemaMeaningChanged => "schema-meaning-changed",
            Bank::PackageIdentityChanged => "package-identity-changed",
            Bank::AuthorityMismatch => "authority-mismatch",
        }
    }

    pub(crate) const fn from_query(kind: QueryKind) -> Self {
        use BankOperationInstallationDenialKind as Bank;
        let kind = match kind {
            QueryKind::MutationBindingIdentityCollision => Bank::MutationBindingIdentityCollision,
            QueryKind::MutationBindingNotInstalled => Bank::MutationBindingNotInstalled,
            QueryKind::MutationBindingMeaningChanged => Bank::MutationBindingMeaningChanged,
            QueryKind::MutationBindingOperationNotInstalled => {
                Bank::MutationBindingOperationNotInstalled
            }
            QueryKind::MutationBindingOperationMeaningChanged => {
                Bank::MutationBindingOperationMeaningChanged
            }
            QueryKind::MutationBindingPrincipalMeaningChanged => {
                Bank::MutationBindingPrincipalMeaningChanged
            }
            QueryKind::MutationBindingResultMeaningChanged => {
                Bank::MutationBindingResultMeaningChanged
            }
            QueryKind::MutationBindingScopeMeaningChanged => {
                Bank::MutationBindingScopeMeaningChanged
            }
            QueryKind::MutationBindingCandidateMeaningChanged => {
                Bank::MutationBindingCandidateMeaningChanged
            }
            QueryKind::MutationBindingHandlerMeaningChanged => {
                Bank::MutationBindingHandlerMeaningChanged
            }
            QueryKind::MutationBindingIdempotencyMeaningChanged => {
                Bank::MutationBindingIdempotencyMeaningChanged
            }
            QueryKind::MutationBindingOutputMeaningChanged => {
                Bank::MutationBindingOutputMeaningChanged
            }
            QueryKind::InvalidMutationBindingIdentity => Bank::InvalidMutationBindingIdentity,
            QueryKind::InvalidMutationHandlerIdentity => Bank::InvalidMutationHandlerIdentity,
            QueryKind::InvalidMutationIdempotencyIdentity => {
                Bank::InvalidMutationIdempotencyIdentity
            }
            QueryKind::InvalidMutationOutputRole => Bank::InvalidMutationOutputRole,
            QueryKind::MutationOutputEntityNotInstalled => Bank::MutationOutputEntityNotInstalled,
            QueryKind::MutationOutputCandidateMismatch => Bank::MutationOutputCandidateMismatch,
            QueryKind::OperationNotInstalled => Bank::OperationNotInstalled,
            QueryKind::OperationMeaningChanged => Bank::OperationMeaningChanged,
            QueryKind::MissingAbilityPolicy => Bank::MissingAbilityPolicy,
            QueryKind::MissingProgram => Bank::MissingProgram,
            QueryKind::MissingDecisionFactBudget => Bank::MissingDecisionFactBudget,
            QueryKind::MissingProjectionWorkBudget => Bank::MissingProjectionWorkBudget,
            QueryKind::ConflictingAuthorizationContract => Bank::ConflictingAuthorizationContract,
            QueryKind::InvalidMutationPreconditionContract => {
                Bank::InvalidMutationPreconditionContract
            }
            QueryKind::CanonicalEntryBudgetExceeded => Bank::CanonicalEntryBudgetExceeded,
            QueryKind::CanonicalEncodedByteBudgetExceeded => {
                Bank::CanonicalEncodedByteBudgetExceeded
            }
            QueryKind::CanonicalDigestSlotRejected => Bank::CanonicalDigestSlotRejected,
            QueryKind::InvalidGraphObligationContract => Bank::InvalidGraphObligationContract,
            QueryKind::AftermathInstallationDenied => Bank::AftermathInstallationDenied,
            QueryKind::AmbiguousExternalEffectContract => Bank::AmbiguousExternalEffectContract,
            QueryKind::AmbiguousAftermathContract => Bank::AmbiguousAftermathContract,
            QueryKind::ForeignRuntime => Bank::ForeignRuntime,
            QueryKind::StaleGeneration => Bank::StaleGeneration,
            QueryKind::SchemaMeaningChanged => Bank::SchemaMeaningChanged,
            QueryKind::PackageIdentityChanged => Bank::PackageIdentityChanged,
            QueryKind::AuthorityMismatch => Bank::AuthorityMismatch,
        };
        Self { kind }
    }
}

impl std::fmt::Display for BankOperationInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}
