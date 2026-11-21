//! Home-retirement denial vocabulary.
use worth_query_execution_contracts::facade::{
    WorthQueryHomeRetirementOwner, WorthQueryOperationHomeRetirementDenial,
};

#[derive(Debug)]
pub enum WorthQueryHomeRetirementDenial {
    RecoveryCustody(crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial),
    CorrectionRetention(crate::domain_computation::primary_graph::WorthQueryCorrectionRetentionDenial),
    ManagedCleanup(crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial),
    Resource(crate::domain_computation::primary_graph::application_resource::WorthQueryResourceActionDenial),
    ConditionalClock(worth_runtime_bridge::facade::BridgeManagedTemporalDenialKind),
    ConditionalBinding(worth_runtime_bridge::facade::BridgeConditionalDenialKind),
    CorrectionMaterial(crate::facade::correction_material::QueryCorrectionMaterialDenial),
    Busy(WorthQueryHomeRetirementOwner),
    Unsupported(WorthQueryHomeRetirementOwner),
    Handler {
        binding: &'static str,
        cause: WorthQueryOperationHomeRetirementDenial,
    },
    NativeAlias(
        worth_relational::facade::change_source::RelationalRuntimeHandleAliasRetirementDenial,
    ),
    NativeObservation(
        worth_relational::facade::change_source::RelationalRetainedObservationRetirementDenial,
    ),
    BridgeBasis(worth_runtime_bridge::facade::RelationalBridgeBasisRetirementDenial),
    CanonicalSubscription(worth_relational::facade::mvcc::PublicationCompanionRetirementDenial),
    Bridge(worth_runtime_bridge::facade::BridgeHomeRetirementDenial),
    Publication(worth_runtime_world::facade::ConsumedPublicationRetirementDenial),
    DispatchCustody(
        crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial,
    ),
    Execution(worth_execution::WorkCeilingDenial),
}

impl From<worth_query_execution_attempt::facade::domain_computation::primary_graph::WorthQueryAttemptHomeRetirementDenial>
    for WorthQueryHomeRetirementDenial
{
    fn from(
        value: worth_query_execution_attempt::facade::domain_computation::primary_graph::WorthQueryAttemptHomeRetirementDenial,
    ) -> Self {
        use worth_query_execution_attempt::facade::domain_computation::primary_graph::WorthQueryAttemptHomeRetirementDenial as A;
        match value {
            A::RecoveryCustody(cause) => Self::RecoveryCustody(cause),
            A::CorrectionRetention(cause) => Self::CorrectionRetention(cause),
            A::ManagedCleanup(cause) => Self::ManagedCleanup(cause),
            A::ConditionalClock(cause) => Self::ConditionalClock(cause),
            A::ConditionalBinding(cause) => Self::ConditionalBinding(cause),
            A::CorrectionMaterial(cause) => Self::CorrectionMaterial(cause),
            A::Busy(owner) => Self::Busy(owner),
            A::Unsupported(owner) => Self::Unsupported(owner),
            A::Handler { binding, cause } => Self::Handler { binding, cause },
            A::NativeAlias(cause) => Self::NativeAlias(cause),
            A::NativeObservation(cause) => Self::NativeObservation(cause),
            A::BridgeBasis(cause) => Self::BridgeBasis(cause),
            A::CanonicalSubscription(cause) => Self::CanonicalSubscription(cause),
            A::Bridge(cause) => Self::Bridge(cause),
            A::Publication(cause) => Self::Publication(cause),
            A::DispatchCustody(cause) => Self::DispatchCustody(cause),
            A::Execution(cause) => Self::Execution(cause),
        }
    }
}
