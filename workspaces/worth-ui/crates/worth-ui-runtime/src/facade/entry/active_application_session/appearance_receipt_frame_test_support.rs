pub(super) fn publish_validation_class(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    graph_node: crate::graph::UiGraphNodeIdentity,
    class: crate::runtime::intent::UiValidationAppearanceClass,
    expected_revision: Option<u64>,
) -> u64 {
    let identity = session.inspect_mounted_identity();
    let row = identity
        .mounted_instances()
        .iter()
        .find(|row| row.graph_node_identity() == graph_node)
        .expect("validation transition has a mounted appearance instance");
    let instance = row.identity();
    let receipt = session
        .inspect_mounted_identity()
        .frame_receipts()
        .iter()
        .find(|row| row.mounted_instance_identity() == instance)
        .expect("validation transition has a current node receipt")
        .node_receipt_identity();
    let target = crate::runtime::intent::UiAdmittedValidationAppearanceTarget::admit(
        session, graph_node, instance, receipt,
    )
    .unwrap();
    session
        .intent_application_facts
        .publish_validation_appearance_fact(target, expected_revision, class)
        .unwrap();
    session
        .intent_application_facts
        .validation_appearance_snapshot()
        .and_then(|snapshot| snapshot.fact_basis_for(graph_node, instance))
        .expect("validation publication should retain its fact revision")
        .1
}

pub(super) fn publish_frame(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    now: u64,
) {
    let outcome = session
        .execute_mounted_frame(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            now,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("receipt frame should publish"));
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
}
