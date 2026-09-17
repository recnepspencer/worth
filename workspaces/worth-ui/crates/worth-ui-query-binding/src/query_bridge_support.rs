use worth_runtime_bridge::facade::{
    BridgeDeliveryReceipt, BridgeWritebackEffectClass, BridgeWritebackFamilyKind,
    BridgeWritebackOutcomeClass, InvalidationSink, SignalBridgeSinkError, TruthWritebackAuthority,
    TruthWritebackAuthorityError, TruthWritebackReceipt, TruthWritebackRequest,
};

#[derive(Clone, Copy)]
pub(crate) struct UiBridgeSignalSink;

impl InvalidationSink for UiBridgeSignalSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct UiBridgeWritebackAuthority;

impl TruthWritebackAuthority for UiBridgeWritebackAuthority {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, TruthWritebackAuthorityError> {
        if request.family_kind() != BridgeWritebackFamilyKind::AspectReconciliation
            || request.effect_class() != BridgeWritebackEffectClass::AspectReconciliation
        {
            return Err(TruthWritebackAuthorityError::new(
                "Worth UI permits only aspect-reconciliation writeback",
            ));
        }
        Ok(TruthWritebackReceipt::new(
            BridgeWritebackOutcomeClass::AuthoritativeCommit,
            &request,
        ))
    }
}
