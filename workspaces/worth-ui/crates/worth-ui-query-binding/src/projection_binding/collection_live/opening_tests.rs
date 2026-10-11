use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::{
    installed_domain::collection_text_projection::{
        WorthUiCollectionTextProjection, WorthUiCollectionTextProjectionExecutor,
        WorthUiCollectionTextProjectionFamily,
    },
    WorthUiQueryWorkspaceExt,
};
use worth_query::facade::{consumer_kit, domain, read};

#[test]
fn nested_collection_opening_is_denied_without_handler_contact() {
    let (mut workspace, binding, contacts) = collection_opening_fixture();
    let budget = crate::UiCollectionProjectionBudget::new(1, 64, 1, 65_536).unwrap();
    let outcome = workspace
        .with_advancement_mut(|_, workspace| binding.open(budget, workspace))
        .unwrap();
    let super::UiCollectionProjectionOpenOutcome::Stopped(stop) = outcome else {
        panic!("nested opening must stop");
    };
    assert_eq!(
        stop.kind(),
        super::UiCollectionProjectionOpenStopKind::ExecutionDenied
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
}

#[test]
fn admitted_collection_opening_succeeds_with_handler_contact() {
    let (mut workspace, binding, contacts) = collection_opening_fixture();
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    let budget = crate::UiCollectionProjectionBudget::new(1, 64, 1, 65_536).unwrap();
    let outcome = binding.open(budget, &mut workspace);
    let super::UiCollectionProjectionOpenOutcome::Opened(receipt) = outcome else {
        panic!("the admitted control must open: {outcome:?}");
    };
    assert!(contacts.load(Ordering::SeqCst) > 0);
    let (live, _) = receipt.into_parts();
    assert!(matches!(
        live.close(&mut workspace),
        super::UiLiveCollectionProjectionCloseOutcome::Closed(_)
    ));
}

fn collection_opening_fixture() -> (
    worth_query::facade::runtime::WorthQueryWorkspace,
    crate::UiCollectionProjectionBinding,
    Arc<AtomicUsize>,
) {
    let contacts = Arc::new(AtomicUsize::new(0));
    let schema =
        consumer_kit::WorthQueryTestBackendSchema::single_collection("WorthUiProjectionText")
            .aspect_contracts(crate::worth_ui_native_aspect_contracts())
            .unwrap()
            .aspect("identity.id", "identity.id")
            .unwrap()
            .aspect("query_text.status", "query_text.status")
            .unwrap()
            .aspect("query_revision.value", "query_revision.value")
            .unwrap()
            .aspect("collection_item.status", "collection_item.status")
            .unwrap()
            .aspect("collection_item.key", "collection_item.key")
            .unwrap();
    let mut builder = consumer_kit::in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(crate::worth_ui_domain_package())
        .domain_operation_executor(
            crate::WorthUiDomainEntry,
            crate::installed_domain::snapshot_measurement::WorthUiSnapshotMeasurement,
            crate::installed_domain::snapshot_measurement::WorthUiSnapshotMeasurementFamily,
            crate::installed_domain::snapshot_measurement::WorthUiSnapshotMeasurementExecutor,
        )
        .domain_operation_executor(
            crate::WorthUiDomainEntry,
            crate::installed_domain::scalar_text_projection::WorthUiScalarTextProjection,
            crate::installed_domain::scalar_text_projection::WorthUiScalarTextProjectionFamily,
            crate::installed_domain::scalar_text_projection::WorthUiScalarTextProjectionExecutor,
        )
        .workflow_stage_executor(
            crate::WorthUiDomainEntry,
            crate::installed_domain::measurement_recording::WorthUiMeasurementRecording,
            crate::installed_domain::measurement_recording::WorthUiMeasurementRecordingFamily,
            crate::installed_domain::measurement_recording::WorthUiMeasurementRecordingExecutor,
        )
        .domain_operation_executor(
            crate::WorthUiDomainEntry,
            WorthUiCollectionTextProjection,
            WorthUiCollectionTextProjectionFamily,
            CountedExecutor(contacts.clone()),
        );
    for dimension in [
        domain::WorthQueryConsumerSupportDimension::Live,
        domain::WorthQueryConsumerSupportDimension::Continuation,
        domain::WorthQueryConsumerSupportDimension::ProjectionConsumption,
        domain::WorthQueryConsumerSupportDimension::Sharing,
        domain::WorthQueryConsumerSupportDimension::Invalidation,
        domain::WorthQueryConsumerSupportDimension::CollectionDelivery,
        domain::WorthQueryConsumerSupportDimension::DependencyImpact,
    ] {
        builder = builder.consumer_support_posture(
            dimension,
            domain::WorthQueryConsumerSupportPosture::Supported,
        );
    }
    let workspace = crate::certification::request_custody::with_bootstrap_request(|execution| {
        builder
            .workspace(execution, "nested-collection-opening")
            .unwrap()
    });
    let registration = crate::UiCollectionProjectionRegistration::text(
        workspace
            .worth_ui()
            .unwrap()
            .projection_view("support.collection.status")
            .unwrap(),
        crate::UiProjectionFieldRequirement::declared("identity.id").unwrap(),
        [crate::UiProjectionFieldRequirement::declared("status").unwrap()],
        false,
        true,
    )
    .unwrap();
    let crate::UiCollectionProjectionBindingAdmission::Ready(binding) =
        registration.admit(&workspace)
    else {
        panic!("collection registration must admit before the hostile opening");
    };
    (workspace, binding, contacts)
}

struct CountedExecutor(Arc<AtomicUsize>);

impl
    domain::WorthQueryDomainOperationExecutor<
        crate::WorthUiDomainEntry,
        WorthUiCollectionTextProjection,
        WorthUiCollectionTextProjectionFamily,
    > for CountedExecutor
{
    const LOWERING_FAMILY: &'static str = "worth-ui-collection-text-projection-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        WorthUiCollectionTextProjectionExecutor.installed_read_declaration()
    }
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        WorthUiCollectionTextProjectionExecutor.execution_resource_support()
    }
    fn execute(
        &self,
        (): (),
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        self.0.fetch_add(1, Ordering::SeqCst);
        WorthUiCollectionTextProjectionExecutor.execute((), context, workspace)
    }
}
