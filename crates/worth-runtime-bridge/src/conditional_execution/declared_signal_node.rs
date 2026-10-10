use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind,
    BridgeInstalledConditionalLoweringCounters,
};

pub(super) fn declared_signal_node(
    registrations: &[crate::correspondence::BridgeSemanticCorrespondenceRegistration],
    counters: &mut BridgeInstalledConditionalLoweringCounters,
) -> Result<(u64, worth_signal::facade::NodeId), BridgeConditionalDenial> {
    let mut targets = registrations
        .iter()
        .flat_map(|registration| registration.targets.iter());
    let first_target = targets.next().ok_or_else(|| {
        BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::EmptyCorrespondenceSet,
            "conditional registrations retained no Signal target",
        )
        .with_lowering_counters(*counters)
    })?;
    counters.correspondence_targets_inspected += 1;
    let graph = first_target.graph_instance_id();
    let first = first_target.node;
    for target in targets {
        counters.correspondence_targets_inspected += 1;
        if target.graph_instance_id() != graph || target.node != first {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::MixedSignalNodes,
                "one conditional declaration cannot lower across multiple Signal nodes",
            )
            .with_lowering_counters(*counters));
        }
    }
    Ok((graph, first))
}
