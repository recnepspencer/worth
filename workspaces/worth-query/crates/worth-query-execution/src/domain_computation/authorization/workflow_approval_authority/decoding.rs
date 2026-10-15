//! Owned standalone approval descriptions retain their original parse admission.
use super::super::WorkflowApprovalRequestDecodingDenial as Denial;
use super::*;
use worth_execution::ExecutionRequest;

pub(in crate::domain_computation) struct DecodedApprovalPrincipal {
    value: WorthQueryDurablePrincipalCurrentness,
    _funding: Arc<ActorCodecFunding>,
}

impl DecodedApprovalPrincipal {
    pub(in crate::domain_computation) fn value(&self) -> &WorthQueryDurablePrincipalCurrentness {
        &self.value
    }
}

pub(in crate::domain_computation) struct DecodedApprovalLineage {
    value: super::super::WorthQueryDurableCapabilityLineage,
    _text_funding: Arc<ActorCodecFunding>,
    _node_funding: Arc<ActorCodecFunding>,
}

impl DecodedApprovalLineage {
    pub(in crate::domain_computation) fn value(
        &self,
    ) -> &super::super::WorthQueryDurableCapabilityLineage {
        &self.value
    }
}

pub(in crate::domain_computation) fn decode_principal(
    execution: &ExecutionRequest<'_, '_>,
    encoded: &str,
) -> Result<DecodedApprovalPrincipal, Denial> {
    if encoded.len() > MAXIMUM_PORTABLE_BYTES {
        return Err(().into());
    }
    let funding = fund_actor_codec(execution, encoded.len().checked_mul(3).ok_or(())?)?;
    let value = serde_json::from_str(encoded).map_err(|_| ())?;
    crate::domain_computation::primary_graph::workflow::actor::encoding::check_live(execution)?;
    Ok(DecodedApprovalPrincipal {
        value,
        _funding: funding,
    })
}

pub(in crate::domain_computation) fn decode_lineage(
    execution: &ExecutionRequest<'_, '_>,
    encoded: &str,
) -> Result<DecodedApprovalLineage, Denial> {
    if encoded.len() > MAXIMUM_PORTABLE_BYTES {
        return Err(().into());
    }
    let text_funding = fund_actor_codec(execution, encoded.len().checked_mul(3).ok_or(())?)?;
    let inventory: lineage_inventory::LineageCount =
        serde_json::from_str(encoded).map_err(|_| ())?;
    let node_funding = fund_actor_codec(
        execution,
        inventory
            .0
            .checked_mul(std::mem::size_of::<
                super::super::WorthQueryDurableCapabilityLineage,
            >())
            .ok_or(())?,
    )?;
    let value = serde_json::from_str(encoded).map_err(|_| ())?;
    crate::domain_computation::primary_graph::workflow::actor::encoding::check_live(execution)?;
    Ok(DecodedApprovalLineage {
        value,
        _text_funding: text_funding,
        _node_funding: node_funding,
    })
}
