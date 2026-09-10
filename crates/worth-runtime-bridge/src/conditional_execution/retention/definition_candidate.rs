use super::{arc_charge, arc_slice_charge, array_charge, sum, BridgeRetentionDenial as Denial};

pub(in crate::conditional_execution) fn retained_bytes(
    signal_branch_identity: &worth_signal::facade::branch::SignalBranchIdentity,
    contract: &super::super::BridgeConditionalContract,
    location: &super::super::BridgeConditionalLocation,
    providers: &super::super::BridgeConditionalProviderSet,
    projection_identity_bytes: usize,
    correspondences: usize,
    observation_plan: Option<
        &super::super::semantic_observation_plan::BridgeConditionalSemanticObservationPlan,
    >,
) -> Result<u64, Denial> {
    sum(&[
        arc_charge::<super::super::BridgeInstalledConditionalLowering>()?,
        arc_charge::<super::super::liveness::BridgeConditionalLoweringLease>()?,
        arc_charge::<super::BridgeRetentionReservation>()?,
        arc_charge::<worth_signal::facade::branch::SignalBranchIdentity>()?,
        array_charge::<u8>(signal_branch_identity.as_str().len())?,
        arc_charge::<
            std::sync::OnceLock<
                worth_signal::facade::branch::SignalConditionalExecutionPort<(), (), ()>,
            >,
        >()?,
        arc_slice_charge::<crate::correspondence::BridgeInstalledSemanticCorrespondence>(
            correspondences,
        )?,
        super::btree_charge::<
            super::super::lowering_registry::BridgeConditionalLoweringKey,
            super::super::lowering_registry::BridgeConditionalLoweringSlot,
        >(1)?,
        super::btree_charge::<
            super::super::lowering_registry::BridgeExactConditionalBasisKey,
            std::sync::Arc<super::super::BridgeInstalledConditionalLowering>,
        >(1)?,
        1,
        providers.retained_arc_bytes()?,
        arc_slice_charge::<u8>(projection_identity_bytes)?,
        contract.retained_heap_bytes()?,
        location.retained_heap_bytes()?,
        observation_plan
            .map(|plan| plan.retained_heap_bytes())
            .transpose()?
            .unwrap_or(0),
    ])
}
