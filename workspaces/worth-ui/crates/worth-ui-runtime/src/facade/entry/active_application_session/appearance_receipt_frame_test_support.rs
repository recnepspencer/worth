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
        .unwrap_or_else(|stop| {
            use crate::facade::entry::WorthUiMountedFrameExecutionStop as Stop;
            match stop {
                Stop::Preparation(denial) => {
                    panic!("receipt frame preparation at {now}: {denial:?}")
                }
                Stop::PublicationLease(denial) => {
                    panic!("receipt frame lease at {now}: {denial:?}")
                }
                Stop::HostMeasurement(denial) => {
                    panic!("receipt frame measurement at {now}: {denial:?}")
                }
                Stop::HostMeasurementTransition(denial) => {
                    panic!("receipt frame measurement transition at {now}: {denial:?}")
                }
                Stop::OccurrenceGeometry(denial) => {
                    panic!("receipt frame occurrence geometry at {now}: {denial:?}")
                }
                Stop::FrameworkTransition(_) => {
                    panic!("receipt frame framework transition at {now}")
                }
            }
        });
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(_) => {}
        crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {
            panic!("receipt frame was unchanged at {now}")
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            panic!("receipt frame admission at {now}: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(rejection) => {
            panic!("receipt frame retention at {now}: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            panic!("receipt frame completion at {now}: {denial:?}")
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejection) => {
            panic!(
                "receipt frame rejection at {now}: {:?}",
                rejection.rejections()
            )
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => {
            panic!("receipt frame remained in flight at {now}")
        }
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("receipt frame became indeterminate at {now}")
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => {
            panic!("receipt frame was superseded at {now}")
        }
        crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
            panic!("receipt frame reconciled at {now}")
        }
    }
}
