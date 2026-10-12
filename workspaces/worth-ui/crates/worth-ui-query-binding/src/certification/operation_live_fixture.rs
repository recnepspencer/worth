use worth_query::facade::{
    consumer_kit::{
        in_memory_test_runtime, WorthQueryInMemoryTestRuntimeBuilder, WorthQueryTestBackendSchema,
    },
    domain,
    foundation::WorthQueryEntityIdentity,
    runtime::{WorthQueryAspectTouch, WorthQueryAuthoredAspectValue, WorthQueryWorkspace},
};

mod explicit_window;

use crate::{
    worth_ui_domain_package, worth_ui_native_aspect_contracts, WorthUiCollectionAllocationPolicy,
    WorthUiInstalledLiveQueryView, WorthUiInstalledQueryBindingReference,
    WorthUiOperationLiveCloseOutcome, WorthUiOperationLiveOpenRequest,
    WorthUiOperationLiveRefreshRequest, WorthUiOperationLiveResource,
    WorthUiOperationLiveRetirement, WorthUiOperationLiveRetirementCloseOutcome,
    WorthUiQueryAllocationDetail, WorthUiQueryBindingPlan, WorthUiQueryConsumerRequirements,
    WorthUiQueryDenialPresentation, WorthUiQueryInspectionRelevance, WorthUiQueryViewShape,
    WorthUiQueryWorkspaceExt,
};

/// Real Query-backed operation-native fixture for downstream framework owners.
///
/// Query workspace and entity authority remain private. Consumers receive only
/// WUI resources, plans, references, refresh requests, and close outcomes.
pub struct WorthUiOperationLiveTestFixture {
    workspace: WorthQueryWorkspace,
    entities: std::collections::BTreeMap<String, WorthQueryEntityIdentity>,
    view: WorthUiInstalledLiveQueryView,
    plan: WorthUiQueryBindingPlan,
    reference: WorthUiInstalledQueryBindingReference,
    breadth: u32,
    next_measurement_value: f32,
}

struct OperationLiveFixtureConfig<'a> {
    label: &'a str,
    identities: &'a [&'a str],
    breadth: u32,
    entity_lookup: bool,
    failed_closes: usize,
}

impl WorthUiOperationLiveTestFixture {
    pub fn new(label: &str) -> Self {
        Self::build(OperationLiveFixtureConfig {
            label,
            identities: &["measurement"],
            breadth: 1,
            entity_lookup: true,
            failed_closes: 0,
        })
    }

    pub fn with_rows(label: &str, identities: &[&str], breadth: u32) -> Self {
        Self::build(OperationLiveFixtureConfig {
            label,
            identities,
            breadth,
            entity_lookup: true,
            failed_closes: 0,
        })
    }

    pub fn without_collection_entity_lookup(label: &str) -> Self {
        Self::build(OperationLiveFixtureConfig {
            label,
            identities: &["measurement"],
            breadth: 1,
            entity_lookup: false,
            failed_closes: 0,
        })
    }

    pub fn with_failed_close(label: &str) -> Self {
        Self::build(OperationLiveFixtureConfig {
            label,
            identities: &["measurement"],
            breadth: 1,
            entity_lookup: true,
            failed_closes: 1,
        })
    }

    fn build(config: OperationLiveFixtureConfig<'_>) -> Self {
        let mut builder = live_builder();
        if !config.entity_lookup {
            builder = builder.without_collection_entity_lookup();
        }
        if config.failed_closes > 0 {
            builder = builder.fail_next_live_closes(config.failed_closes);
        }
        let builder = crate::install_worth_ui_test_operation_executors(builder);
        let mut workspace =
            crate::certification::request_custody::with_bootstrap_request(|execution| {
                builder
                    .workspace(execution, config.label)
                    .expect("operation-live fixture workspace")
            });
        let entities = config
            .identities
            .iter()
            .enumerate()
            .map(|(index, identity)| {
                (
                    (*identity).to_owned(),
                    insert_measurement(&mut workspace, identity, 240.0 + index as f32),
                )
            })
            .collect();
        let view = workspace
            .worth_ui()
            .expect("Worth UI domain installed")
            .live_measurement_view("certification.live.measurements")
            .expect("live measurement view");
        let plan = WorthUiQueryBindingPlan::default()
            .register_view(view.clone())
            .expect("live view registration");
        let reference = plan
            .resolve_definition(
                view.definition().identity(),
                WorthUiQueryViewShape::Collection,
            )
            .expect("live view reference");
        Self {
            workspace,
            entities,
            view,
            plan,
            reference,
            breadth: config.breadth,
            next_measurement_value: 320.0,
        }
    }

    pub fn binding_plan(&self) -> WorthUiQueryBindingPlan {
        self.plan.clone()
    }

    pub fn view(&self) -> WorthUiInstalledLiveQueryView {
        self.view.clone()
    }

    pub fn reference(&self) -> &WorthUiInstalledQueryBindingReference {
        &self.reference
    }

    pub fn open_resource(&mut self) -> WorthUiOperationLiveResource {
        self.view
            .open_operation(operation_live_request(self.breadth), &mut self.workspace)
            .expect("operation-native live resource opens")
    }

    pub fn update_measurement(&mut self) {
        let identity = self
            .entities
            .first_key_value()
            .map(|(identity, _)| identity.clone())
            .expect("fixture has one measurement");
        self.update_named_measurement(&identity);
    }

    pub fn update_named_measurement(&mut self, identity: &str) {
        let value = self.next_measurement_value;
        self.next_measurement_value += 80.0;
        let entity = self
            .entities
            .get(identity)
            .expect("named fixture measurement exists")
            .clone();
        self.workspace
            .with_execution_request_mut(|execution, workspace| {
                workspace
                    .update(execution, entity, |measurement| {
                        measurement.set_aspect(
                            aspect_touch("measurement.value"),
                            native_measurement(value),
                        )
                    })
                    .expect("operation-live fixture update")
            })
            .expect("workspace host call opens");
    }

    pub fn insert_named_measurement(&mut self, identity: &str) {
        let entity = insert_measurement(&mut self.workspace, identity, self.next_measurement_value);
        self.next_measurement_value += 80.0;
        assert!(
            self.entities.insert(identity.to_owned(), entity).is_none(),
            "fixture identity must be unique"
        );
    }

    pub fn remove_named_measurement(&mut self, identity: &str) {
        let entity = self
            .entities
            .remove(identity)
            .expect("removed fixture measurement exists");
        self.workspace
            .with_execution_request_mut(|execution, workspace| {
                workspace
                    .delete(execution, entity)
                    .expect("operation-live fixture deletion")
            })
            .expect("workspace host call opens");
    }

    pub fn rename_measurement(&mut self, from: &str, to: &str) {
        let entity = self
            .entities
            .remove(from)
            .expect("renamed fixture measurement exists");
        self.workspace
            .with_execution_request_mut(|execution, workspace| {
                workspace
                    .update(execution, entity.clone(), |measurement| {
                        measurement.set_aspect(
                            aspect_touch("identity.id"),
                            WorthQueryAuthoredAspectValue::string(to),
                        )
                    })
                    .expect("operation-live fixture rename")
            })
            .expect("workspace host call opens");
        assert!(
            self.entities.insert(to.to_owned(), entity).is_none(),
            "fixture rename target must be unique"
        );
    }

    pub fn refresh_request(&mut self) -> WorthUiOperationLiveRefreshRequest<'_> {
        WorthUiOperationLiveRefreshRequest::new(&self.reference, &mut self.workspace)
    }

    pub fn refresh_request_for(
        &mut self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> WorthUiOperationLiveRefreshRequest<'_> {
        WorthUiOperationLiveRefreshRequest::new(reference, &mut self.workspace)
    }

    pub fn close_retirement(
        &mut self,
        retirement: WorthUiOperationLiveRetirement,
    ) -> WorthUiOperationLiveRetirementCloseOutcome {
        retirement.close(&mut self.workspace)
    }

    pub fn close_resource(
        &mut self,
        resource: WorthUiOperationLiveResource,
    ) -> WorthUiOperationLiveCloseOutcome {
        resource.close(&mut self.workspace)
    }
}

fn supported_live_dimensions() -> impl Iterator<Item = domain::WorthQueryConsumerSupportDimension> {
    [
        domain::WorthQueryConsumerSupportDimension::Live,
        domain::WorthQueryConsumerSupportDimension::Sharing,
        domain::WorthQueryConsumerSupportDimension::Invalidation,
        domain::WorthQueryConsumerSupportDimension::DependencyImpact,
        domain::WorthQueryConsumerSupportDimension::CollectionDelivery,
    ]
    .into_iter()
}

fn live_builder() -> WorthQueryInMemoryTestRuntimeBuilder {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(worth_ui_native_aspect_contracts())
        .expect("Worth UI aspect contracts")
        .aspect("identity.id", "identity.id")
        .expect("identity aspect")
        .aspect("measurement.value", "measurement.value")
        .expect("measurement aspect");
    supported_live_dimensions().fold(
        in_memory_test_runtime()
            .with_schema(schema)
            .domain_package(worth_ui_domain_package()),
        |builder, dimension| {
            builder.consumer_support_posture(
                dimension,
                domain::WorthQueryConsumerSupportPosture::Supported,
            )
        },
    )
}

fn insert_measurement(
    workspace: &mut WorthQueryWorkspace,
    identity: &str,
    value: f32,
) -> WorthQueryEntityIdentity {
    workspace
        .with_execution_request_mut(|execution, workspace| {
            workspace
                .insert(execution, "WorthUiMeasurement", |measurement| {
                    measurement
                        .set_aspect(
                            aspect_touch("identity.id"),
                            WorthQueryAuthoredAspectValue::string(identity),
                        )
                        .set_aspect(aspect_touch("measurement.value"), native_measurement(value))
                })
                .expect("operation-live fixture insertion")
                .deltas()[0]
                .entity_identity()
                .clone()
        })
        .expect("workspace host call opens")
}

fn aspect_touch(path: &str) -> WorthQueryAspectTouch {
    WorthQueryAspectTouch::from_authoring_ingress_text(path).expect("static aspect touch")
}

fn native_measurement(value: f32) -> WorthQueryAuthoredAspectValue {
    WorthQueryAuthoredAspectValue::native(worth_foundational::AspectValue::Float32(
        worth_foundational::CanonicalF32::from_f32(value),
    ))
}

fn operation_live_request(breadth: u32) -> WorthUiOperationLiveOpenRequest {
    WorthUiOperationLiveOpenRequest::new(
        operation_live_requirements(),
        collection_breadth(breadth),
        WorthUiCollectionAllocationPolicy::PreserveAdmittedRows,
    )
}

fn operation_live_requirements() -> WorthUiQueryConsumerRequirements {
    WorthUiQueryConsumerRequirements::new(
        domain::WorthQueryConsumerBoundaryRequirements {
            presentation: domain::WorthQueryConsumerPresentationPosture::Interactive,
            allocation: domain::WorthQueryConsumerAllocationPosture::Borrowed,
        },
        WorthUiQueryAllocationDetail::BorrowedFactSlice,
        WorthUiQueryViewShape::Collection,
        WorthUiQueryDenialPresentation::StructuredStatus,
        WorthUiQueryInspectionRelevance::Relevant,
    )
}

fn collection_breadth(breadth: u32) -> domain::WorthQueryCollectionWindowBreadth {
    domain::WorthQueryCollectionWindowBreadth::new(breadth, 0, 0, breadth)
        .expect("static collection breadth")
}
