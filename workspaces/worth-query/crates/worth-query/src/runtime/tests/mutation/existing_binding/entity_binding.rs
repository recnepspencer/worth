use super::*;

#[test]
fn update_existing_preserves_authoritative_binding_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.update-existing")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.update-existing-table", |q| {
            q.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
                .schema_basis("tasks-update-existing-table")
        })
        .expect("live view should declare");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Before existing update"),
            )
        })
        .expect("seed insert should execute");
    let binding_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity");
    let binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                binding_authority.clone(),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding should build");

    let receipt = workspace
        .update_existing(request_execution, binding, |task| {
            task.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("After existing update"),
            )
        })
        .expect("existing-target update should execute");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("existing-target receipt should inspect");

    let evidence = receipt
        .existing_truth_binding_evidence()
        .expect("receipt should retain existing-truth evidence");
    assert_eq!(
        evidence.family(),
        WorthQueryExistingTruthBindingFamily::DirectEntityIdentity
    );
    assert_eq!(
        evidence.outcome(),
        WorthQueryExistingTruthBindingOutcome::ExistingAuthoritativeTarget
    );
    assert_eq!(
        evidence.authoritative_identity().as_str(),
        "authority:task-1"
    );
    assert_eq!(
        evidence.resolved_entity_identity(),
        &seed.deltas()[0].entity_identity
    );
    assert_eq!(
        evidence
            .target_collection()
            .map(|collection| collection.as_str()),
        Some("Task")
    );
    assert!(!evidence.binding_digest().is_empty());

    match inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            let evidence = inspection
                .existing_truth_binding_evidence()
                .expect("inspection should retain existing-truth evidence");
            assert_eq!(
                evidence.authoritative_identity().as_str(),
                "authority:task-1"
            );
            assert_eq!(
                evidence.resolved_entity_identity(),
                &seed.deltas()[0].entity_identity
            );
            assert_eq!(
                evidence
                    .target_collection()
                    .map(|collection| collection.as_str()),
                Some("Task")
            );
            assert_eq!(
                evidence.binding_digest(),
                receipt
                    .existing_truth_binding_evidence()
                    .expect("receipt should retain existing-truth evidence")
                    .binding_digest()
            );
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}

#[test]
fn update_existing_denies_missing_target_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.update-existing-denial")
        .expect("task runtime should open a named workspace");
    let binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                test_entity_identity("task:missing"),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding should build");

    let error = workspace
        .update_existing(request_execution, binding, |task| {
            task.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("No target"),
            )
        })
        .expect_err("missing existing target should deny early");

    match error {
        WorthQueryRuntimeError::MutationBindingDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthBindingDenialKind::ResolvedTargetMissing
            );
            assert!(!denial.denial_digest().is_empty());
        }
        other => panic!("expected typed mutation binding denial, got {other:?}"),
    }
}
