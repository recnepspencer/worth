//! Home-retirement denial vocabulary.
use worth_query_execution_contracts::facade::{
    WorthQueryHomeRetirementOwner, WorthQueryOperationHomeRetirementDenial,
};

#[derive(Debug)]
pub enum WorthQueryAttemptHomeRetirementDenial {
    RecoveryCustody(crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial),
    CorrectionRetention(super::super::WorthQueryCorrectionRetentionDenial),
    ManagedCleanup(super::super::query_obligations::QueryObligationDenial),
    ConditionalClock(worth_runtime_bridge::facade::BridgeManagedTemporalDenialKind),
    ConditionalBinding(worth_runtime_bridge::facade::BridgeConditionalDenialKind),
    CorrectionMaterial(worth_query_execution_obligations::facade::aftermath::correction_material::QueryCorrectionMaterialDenial),
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
    /// The dispatch journal still holds an executing attempt or an unsettled record.
    DispatchCustody(
        crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial,
    ),
    Execution(worth_execution::WorkCeilingDenial),
}




impl From<worth_query_execution_runtime::facade::ports::RuntimeHomeRetirementDenial>
    for WorthQueryAttemptHomeRetirementDenial
{
    fn from(value: worth_query_execution_runtime::facade::ports::RuntimeHomeRetirementDenial) -> Self {
        use worth_query_execution_runtime::facade::ports::RuntimeHomeRetirementDenial as R;
        match value {
            R::Busy(owner) => Self::Busy(owner),
            R::Unsupported(owner) => Self::Unsupported(owner),
            R::NativeAlias(cause) => Self::NativeAlias(cause),
            R::NativeObservation(cause) => Self::NativeObservation(cause),
            R::BridgeBasis(cause) => Self::BridgeBasis(cause),
            R::CanonicalSubscription(cause) => Self::CanonicalSubscription(cause),
            R::Bridge(cause) => Self::Bridge(cause),
            R::Publication(cause) => Self::Publication(cause),
            R::Execution(cause) => Self::Execution(cause),
        }
    }
}

impl From<worth_query_execution_lineage::facade::ports::LineageHomeRetirementDenial>
    for WorthQueryAttemptHomeRetirementDenial
{
    fn from(value: worth_query_execution_lineage::facade::ports::LineageHomeRetirementDenial) -> Self {
        use worth_query_execution_lineage::facade::ports::LineageHomeRetirementDenial as L;
        match value {
            L::Busy(owner) => Self::Busy(owner),
            L::Unsupported(owner) => Self::Unsupported(owner),
            L::Handler { binding, cause } => Self::Handler { binding, cause },
            L::NativeAlias(cause) => Self::NativeAlias(cause),
            L::NativeObservation(cause) => Self::NativeObservation(cause),
            L::BridgeBasis(cause) => Self::BridgeBasis(cause),
            L::CanonicalSubscription(cause) => Self::CanonicalSubscription(cause),
            L::Bridge(cause) => Self::Bridge(cause),
            L::Publication(cause) => Self::Publication(cause),
            L::Execution(cause) => Self::Execution(cause),
        }
    }
}

impl From<crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial>
    for WorthQueryAttemptHomeRetirementDenial
{
    fn from(value: crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial) -> Self {
        Self::DispatchCustody(value)
    }
}

impl From<worth_execution::WorkCeilingDenial> for WorthQueryAttemptHomeRetirementDenial {
    fn from(value: worth_execution::WorkCeilingDenial) -> Self {
        Self::Execution(value)
    }
}

impl From<super::super::WorthQueryCorrectionRetentionDenial> for WorthQueryAttemptHomeRetirementDenial {
    fn from(value: super::super::WorthQueryCorrectionRetentionDenial) -> Self {
        Self::CorrectionRetention(value)
    }
}

impl From<super::super::query_obligations::QueryObligationDenial> for WorthQueryAttemptHomeRetirementDenial {
    fn from(value: super::super::query_obligations::QueryObligationDenial) -> Self {
        Self::ManagedCleanup(value)
    }
}

impl From<crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial>
    for WorthQueryAttemptHomeRetirementDenial
{
    fn from(value: crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial) -> Self {
        Self::RecoveryCustody(value)
    }
}

impl From<worth_query_execution_obligations::facade::aftermath::correction_material::QueryCorrectionMaterialDenial>
    for WorthQueryAttemptHomeRetirementDenial
{
    fn from(value: worth_query_execution_obligations::facade::aftermath::correction_material::QueryCorrectionMaterialDenial) -> Self {
        Self::CorrectionMaterial(value)
    }
}

pub type WorthQueryHomeRetirementDenial = WorthQueryAttemptHomeRetirementDenial;
