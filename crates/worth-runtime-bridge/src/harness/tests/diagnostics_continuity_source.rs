use crate::facade::{
    BridgeHistoricalLineageAuthority, BridgeHistoricalLineageRequest,
    BridgeHistoricalResolvedLineageIdentity, BridgeHistoricalResolvedRecordIdentity,
    BridgeLineageSourceError, ContinuityLineageSource,
};

#[derive(Debug, Clone, Default)]
pub(super) struct DiagnosticsContinuityLineageSource;

impl ContinuityLineageSource for DiagnosticsContinuityLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                "lineage:diagnostics-successor",
            )],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "entity:0:4:2",
            )],
            vec![7],
        )
    }
}
