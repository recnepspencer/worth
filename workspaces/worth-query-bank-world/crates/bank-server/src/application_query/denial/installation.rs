//! Bank kinds for application query and capability installation denial.

use worth_query_host::facade::domain::{
    WorthQueryApplicationCapabilityInstallationDenialKind as QueryCapability,
    WorthQueryApplicationQueryInstallationDenialKind as QueryInstallation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationQueryInstallationDenialKind {
    BindingIdentityCollision,
    BindingMeaningChanged,
    BindingPrincipalMeaningChanged,
    BindingQueryMeaningChanged,
    BindingQueryNotInstalled,
    BindingResultLimitIsZero,
    BindingScopeMeaningChanged,
    BindingWorkLimitIsZero,
    InvalidBindingIdentity,
    QueryNotInstalled,
    QueryMeaningChanged,
    AuthorizationNotInstalled,
    LiveEffectNotInstalled,
    LiveScopeIdentityNotInstalled,
    LiveTargetIdentityNotInstalled,
    RootNotInstalled,
    ProjectionNotInstalled,
    RelationNotInstalled,
    PredicateNotInstalled,
    OrderingNotInstalled,
    ResultShapeDisconnected,
    DependencyCeilingExceeded,
    ForeignRuntime,
    StaleGeneration,
    SchemaMeaningChanged,
    PackageIdentityChanged,
    AuthorityMismatch,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
    InvalidGraphObligationContract,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationCapabilityInstallationDenialKind {
    CapabilityNotInstalled,
    CapabilityMeaningChanged,
    ForeignRuntime,
    StaleGeneration,
    PackageIdentityChanged,
    SchemaMeaningChanged,
    CanonicalEntryLimitExceeded,
    CanonicalByteLimitExceeded,
    CanonicalDigestSlotRejected,
    AuthorityMismatch,
}

impl BankApplicationCapabilityInstallationDenialKind {
    pub(crate) const fn from_query(kind: QueryCapability) -> Self {
        match kind {
            QueryCapability::CapabilityNotInstalled => Self::CapabilityNotInstalled,
            QueryCapability::CapabilityMeaningChanged => Self::CapabilityMeaningChanged,
            QueryCapability::ForeignRuntime => Self::ForeignRuntime,
            QueryCapability::StaleGeneration => Self::StaleGeneration,
            QueryCapability::PackageIdentityChanged => Self::PackageIdentityChanged,
            QueryCapability::SchemaMeaningChanged => Self::SchemaMeaningChanged,
            QueryCapability::CanonicalEntryLimitExceeded => Self::CanonicalEntryLimitExceeded,
            QueryCapability::CanonicalByteLimitExceeded => Self::CanonicalByteLimitExceeded,
            QueryCapability::CanonicalDigestSlotRejected => Self::CanonicalDigestSlotRejected,
            QueryCapability::AuthorityMismatch => Self::AuthorityMismatch,
        }
    }
}

pub(super) const fn query_installation(
    kind: QueryInstallation,
) -> BankApplicationQueryInstallationDenialKind {
    use BankApplicationQueryInstallationDenialKind as Bank;
    match kind {
        QueryInstallation::BindingIdentityCollision => Bank::BindingIdentityCollision,
        QueryInstallation::BindingMeaningChanged => Bank::BindingMeaningChanged,
        QueryInstallation::BindingPrincipalMeaningChanged => Bank::BindingPrincipalMeaningChanged,
        QueryInstallation::BindingQueryMeaningChanged => Bank::BindingQueryMeaningChanged,
        QueryInstallation::BindingQueryNotInstalled => Bank::BindingQueryNotInstalled,
        QueryInstallation::BindingResultLimitIsZero => Bank::BindingResultLimitIsZero,
        QueryInstallation::BindingScopeMeaningChanged => Bank::BindingScopeMeaningChanged,
        QueryInstallation::BindingWorkLimitIsZero => Bank::BindingWorkLimitIsZero,
        QueryInstallation::InvalidBindingIdentity => Bank::InvalidBindingIdentity,
        QueryInstallation::QueryNotInstalled => Bank::QueryNotInstalled,
        QueryInstallation::QueryMeaningChanged => Bank::QueryMeaningChanged,
        QueryInstallation::AuthorizationNotInstalled => Bank::AuthorizationNotInstalled,
        QueryInstallation::LiveEffectNotInstalled => Bank::LiveEffectNotInstalled,
        QueryInstallation::LiveScopeIdentityNotInstalled => Bank::LiveScopeIdentityNotInstalled,
        QueryInstallation::LiveTargetIdentityNotInstalled => Bank::LiveTargetIdentityNotInstalled,
        QueryInstallation::RootNotInstalled => Bank::RootNotInstalled,
        QueryInstallation::ProjectionNotInstalled => Bank::ProjectionNotInstalled,
        QueryInstallation::RelationNotInstalled => Bank::RelationNotInstalled,
        QueryInstallation::PredicateNotInstalled => Bank::PredicateNotInstalled,
        QueryInstallation::OrderingNotInstalled => Bank::OrderingNotInstalled,
        QueryInstallation::ResultShapeDisconnected => Bank::ResultShapeDisconnected,
        QueryInstallation::DependencyCeilingExceeded => Bank::DependencyCeilingExceeded,
        QueryInstallation::ForeignRuntime => Bank::ForeignRuntime,
        QueryInstallation::StaleGeneration => Bank::StaleGeneration,
        QueryInstallation::SchemaMeaningChanged => Bank::SchemaMeaningChanged,
        QueryInstallation::PackageIdentityChanged => Bank::PackageIdentityChanged,
        QueryInstallation::AuthorityMismatch => Bank::AuthorityMismatch,
        QueryInstallation::CanonicalEntryBudgetExceeded => Bank::CanonicalEntryBudgetExceeded,
        QueryInstallation::CanonicalEncodedByteBudgetExceeded => {
            Bank::CanonicalEncodedByteBudgetExceeded
        }
        QueryInstallation::CanonicalDigestSlotRejected => Bank::CanonicalDigestSlotRejected,
        QueryInstallation::InvalidGraphObligationContract => Bank::InvalidGraphObligationContract,
    }
}

pub(super) const fn capability_installation(
    kind: QueryCapability,
) -> BankApplicationCapabilityInstallationDenialKind {
    BankApplicationCapabilityInstallationDenialKind::from_query(kind)
}
