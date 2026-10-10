use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};
use worth_query::facade::consumer_kit::in_memory_test_runtime;
use worth_query::facade::runtime::WorthQueryWorkspace;

use crate::support;

mod live_journey {
    use worth_query::facade::live::{
        current, declare, AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName,
        QuerySchemaView, ScalarAspectType, SchemaFieldView, WorthQueryAuthorityLane,
        WorthQueryManagedLiveCloseOutcome, WorthQueryManagedLiveDeliveryCauseKind,
        WorthQueryManagedLiveLifecyclePosture, WorthQueryManagedLiveSubscriptionFamily,
        WorthQueryOrdinaryRuntimePostureKind,
    };

    #[test]
    fn facade_live_completes_declare_context_open_observe_delivery_and_close() {
        let host_request = crate::test_execution_policy::serial_request();
        let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let mut workspace = super::live_workspace(request_execution);
        let opened = declare("tasks.public", |read| {
            read.local_collection(
                "Task",
                QuerySchemaView::new(
                    "public-live-task",
                    [SchemaFieldView::new(
                        AspectName::new("identity").expect("aspect should build"),
                        FieldName::new("id").expect("field should build"),
                        ScalarAspectType::String,
                    )],
                    [],
                ),
                |query| {
                    query.project(
                        AspectFieldSelector::new("identity", "id").expect("selector should build"),
                    )
                },
                |shape| {
                    shape.field(
                        AuthoredResultShapeField::new("identity", "id", "identity.id")
                            .expect("result field should build"),
                    )
                },
            )
        })
        .expect("live declaration should author")
        .using(current())
        .open(request_execution, &mut workspace);
        let handle = match opened {
            worth_query::facade::live::WorthQueryLiveOpenOutcome::Opened(completion) => {
                completion.into_handle()
            }
            worth_query::facade::live::WorthQueryLiveOpenOutcome::Stopped(stop) => {
                panic!("public live open unexpectedly stopped: {:?}", stop.source())
            }
        };

        let observation = handle
            .observe(&mut workspace)
            .expect("public live lifecycle should be observable");
        assert_eq!(
            observation.posture(),
            WorthQueryManagedLiveLifecyclePosture::Active
        );
        assert_eq!(
            observation.authority_lane(),
            WorthQueryAuthorityLane::AuthoritativeTruth
        );
        assert_eq!(
            observation.runtime_posture().kind(),
            WorthQueryOrdinaryRuntimePostureKind::Current
        );
        assert_eq!(
            observation.subscription_family(),
            WorthQueryManagedLiveSubscriptionFamily::CollectionMembership
        );
        assert_eq!(observation.activation_work().family_selection_count(), 1);
        assert_eq!(observation.activation_work().declaration_count(), 1);
        assert_eq!(observation.activation_work().admission_count(), 1);
        assert_eq!(observation.activation_work().activation_input_count(), 1);
        assert_eq!(
            observation.activation_work().active_lane_creation_count(),
            1
        );
        assert_eq!(observation.activation_work().active_lane_join_count(), 0);
        assert_eq!(observation.activation_work().consumer_attachment_count(), 1);
        super::insert_task(request_execution, &mut workspace);
        let delivery = handle
            .drain(&mut workspace)
            .expect("public typed delivery should be observable");
        assert_eq!(delivery.batches().len(), 1);
        assert_eq!(
            delivery.batches()[0].cause_kind(),
            WorthQueryManagedLiveDeliveryCauseKind::RelationalChange
        );
        assert_eq!(delivery.batches()[0].patch_group_width(), 1);
        assert_ne!(
            delivery.batches()[0].delivery_batch_identity(),
            delivery.batches()[0].cause_identity()
        );
        let maintenance_work = delivery.batches()[0]
            .maintenance_work()
            .expect("relational delivery should disclose bounded maintenance work");
        assert_eq!(maintenance_work.mutation_delta_count(), 1);
        assert_eq!(maintenance_work.index_update_count(), 1);
        assert_eq!(maintenance_work.live_view_update_count(), 1);

        match handle.close(&mut workspace) {
            WorthQueryManagedLiveCloseOutcome::Closed(receipt) => {
                assert_eq!(receipt.resource_name(), "tasks.public");
                assert!(receipt.lane_terminal());
                assert_eq!(receipt.disposal_work().consumer_attachment_close_count(), 1);
                assert_eq!(receipt.disposal_work().active_lane_close_count(), 1);
                assert_eq!(receipt.disposal_work().lifecycle_closeout_count(), 1);
                assert_eq!(receipt.disposal_work().budget_consumption_width(), 2);
                assert_eq!(receipt.disposal_work().budget_remaining_width(), 0);
            }
            WorthQueryManagedLiveCloseOutcome::Stopped(stop) => {
                panic!("public live close unexpectedly stopped: {:?}", stop.error())
            }
        }
    }
}

fn insert_task(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
) {
    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                worth_query::facade::runtime::WorthQueryAspectTouch::aspect_field_path(
                    AspectKey::new("identity").expect("identity aspect should build"),
                    CanonicalFieldPath::new([
                        FieldKey::new("id").expect("identity field should build")
                    ])
                    .expect("identity path should build"),
                ),
                worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string("task-1"),
            )
        })
        .expect("public live delivery fixture should write through the real runtime");
}

fn live_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkspace {
    in_memory_test_runtime()
        .with_schema(support::task_backend_schema::task_backend_schema())
        .workspace(request_execution, "declarative-live-public-dx")
        .expect("public live test workspace should build")
}
