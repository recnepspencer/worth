use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::domain::{
    WorthQueryConsumerSupportDimension, WorthQueryConsumerSupportPosture,
};
use worth_query::facade::runtime::{WorthQueryAspectTouch, WorthQueryAuthoredAspectValue};

use crate::{
    install_worth_ui_test_operation_executors, worth_ui_domain_package,
    worth_ui_native_aspect_contracts,
};

pub(crate) fn projection_workspace(
    supports_async_lifecycle: bool,
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        projection_runtime_builder(supports_async_lifecycle)
            .workspace(execution, "worth-ui-scalar-text-projection")
            .expect("scalar text Query workspace")
    })
}

pub(crate) fn remasked_projection_workspace() -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        projection_runtime_builder(true)
            .remask_projection(
                worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection::remasked(
                    worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
                    "policy:drifted",
                    "tenant-truth:stable",
                    "tenant-schema:stable",
                    "relationship-proof:verified",
                    "schema-context:worth-ui-projection",
                ),
            )
            .workspace(execution, "worth-ui-remasked-scalar-text-projection")
            .expect("remasked scalar text Query workspace")
    })
}

pub(crate) fn collection_projection_workspace() -> worth_query::facade::runtime::WorthQueryWorkspace
{
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        collection_projection_workspace_builder(true)
            .workspace(execution, "worth-ui-collection-text-projection")
            .expect("collection text Query workspace")
    })
}

pub(crate) fn collection_projection_workspace_without_entity_lookup(
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        collection_projection_workspace_builder(true)
            .without_collection_entity_lookup()
            .workspace(
                execution,
                "worth-ui-collection-text-projection-without-entity-lookup",
            )
            .expect("collection text Query workspace without entity lookup")
    })
}

pub(crate) fn collection_projection_workspace_without_dependency_impact(
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        collection_projection_workspace_builder(false)
            .workspace(
                execution,
                "worth-ui-collection-text-projection-without-dependency-impact",
            )
            .expect("collection text Query workspace without dependency-impact support")
    })
}

pub(crate) fn partial_collection_projection_workspace(
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    let builder = collection_projection_support(projection_runtime_base_builder(), true);
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        crate::install_worth_ui_partial_collection_test_operation_executors(builder)
            .workspace(execution, "worth-ui-partial-collection-text-projection")
            .expect("partial collection text Query workspace")
    })
}

pub(crate) fn seeded_collection_projection_workspace(
    rows: Vec<(String, String)>,
    partial: bool,
    entity_lookup: bool,
    supports_async_lifecycle: bool,
) -> (
    worth_query::facade::runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestSeedReceipt,
) {
    let rows = rows
        .into_iter()
        .enumerate()
        .map(|(index, (identity, status))| (identity, status, index as u64 + 1))
        .collect();
    seeded_collection_projection_workspace_with_item_keys(
        rows,
        partial,
        entity_lookup,
        supports_async_lifecycle,
    )
}

pub(crate) fn seeded_collection_projection_workspace_with_item_keys(
    rows: Vec<(String, String, u64)>,
    partial: bool,
    entity_lookup: bool,
    supports_async_lifecycle: bool,
) -> (
    worth_query::facade::runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestSeedReceipt,
) {
    use worth_query::facade::{
        consumer_kit::WorthQueryTestSeedRow,
        runtime::{WorthQueryAspectTouch, WorthQueryAuthoredAspectValue},
    };

    let identity_touch =
        WorthQueryAspectTouch::from_authoring_ingress_text("identity.id").expect("identity touch");
    let status_touch = WorthQueryAspectTouch::from_authoring_ingress_text("collection_item.status")
        .expect("status touch");
    let item_key_touch = WorthQueryAspectTouch::from_authoring_ingress_text("collection_item.key")
        .expect("application item-key touch");
    let seed_rows = rows
        .into_iter()
        .map(|(identity, status, item_key)| {
            WorthQueryTestSeedRow::new(identity.clone(), "WorthUiProjectionText", |entity| {
                entity
                    .set_aspect(
                        identity_touch.clone(),
                        WorthQueryAuthoredAspectValue::string(identity),
                    )
                    .set_aspect(
                        WorthQueryAspectTouch::from_authoring_ingress_text("query_text.status")
                            .expect("scalar status touch"),
                        WorthQueryAuthoredAspectValue::string(status.clone()),
                    )
                    .set_aspect(
                        status_touch.clone(),
                        WorthQueryAuthoredAspectValue::string(status),
                    )
                    .set_aspect(
                        item_key_touch.clone(),
                        WorthQueryAuthoredAspectValue::native(
                            worth_foundational::AspectValue::UInt64(item_key),
                        ),
                    )
            })
            .expect("projection seed row")
        })
        .collect();
    let builder = if supports_async_lifecycle {
        projection_runtime_builder(true)
    } else {
        projection_runtime_base_builder()
    };
    let builder = collection_projection_support(builder, true);
    let builder = if partial {
        crate::install_worth_ui_partial_collection_test_operation_executors(builder)
    } else if supports_async_lifecycle {
        builder
    } else {
        install_worth_ui_test_operation_executors(builder)
    };
    let builder = if entity_lookup {
        builder
    } else {
        builder.without_collection_entity_lookup()
    };
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        builder
            .seed_collection_rows(identity_touch, seed_rows)
            .expect("projection seed identities")
            .workspace_with_seed_receipt(execution, "worth-ui-seeded-collection-text-projection")
            .expect("seeded collection text Query workspace")
    })
}

fn collection_projection_workspace_builder(
    supports_dependency_impact: bool,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    collection_projection_support(
        projection_runtime_builder(false),
        supports_dependency_impact,
    )
}

fn collection_projection_support(
    builder: worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder,
    supports_dependency_impact: bool,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let builder = [
        WorthQueryConsumerSupportDimension::Live,
        WorthQueryConsumerSupportDimension::Continuation,
        WorthQueryConsumerSupportDimension::ProjectionConsumption,
        WorthQueryConsumerSupportDimension::Sharing,
        WorthQueryConsumerSupportDimension::Invalidation,
        WorthQueryConsumerSupportDimension::CollectionDelivery,
    ]
    .into_iter()
    .fold(builder, |builder, dimension| {
        builder.consumer_support_posture(dimension, WorthQueryConsumerSupportPosture::Supported)
    });
    if supports_dependency_impact {
        builder.consumer_support_posture(
            WorthQueryConsumerSupportDimension::DependencyImpact,
            WorthQueryConsumerSupportPosture::Supported,
        )
    } else {
        builder
    }
}

fn projection_runtime_builder(
    supports_async_lifecycle: bool,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let builder = install_worth_ui_test_operation_executors(projection_runtime_base_builder());
    if supports_async_lifecycle {
        [
            WorthQueryConsumerSupportDimension::AsyncResultState,
            WorthQueryConsumerSupportDimension::Recovery,
        ]
        .into_iter()
        .fold(builder, |builder, dimension| {
            builder.consumer_support_posture(dimension, WorthQueryConsumerSupportPosture::Supported)
        })
    } else {
        builder
    }
}

fn projection_runtime_base_builder(
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiProjectionText")
        .aspect_contracts(worth_ui_native_aspect_contracts())
        .expect("Worth UI native aspect contracts")
        .aspect("identity.id", "identity.id")
        .expect("identity aspect")
        .aspect("query_text.status", "query_text.status")
        .expect("projection status aspect")
        .aspect("query_revision.value", "query_revision.value")
        .expect("query revision aspect")
        .aspect("collection_item.status", "collection_item.status")
        .expect("collection item status aspect")
        .aspect("collection_item.key", "collection_item.key")
        .expect("collection item key aspect");
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(worth_ui_domain_package())
}

#[cfg(test)]
pub(crate) fn insert_status(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    status: &str,
) -> worth_query::facade::foundation::WorthQueryEntityIdentity {
    insert_collection_status(workspace, "platform.pulse.status", status)
}

pub(crate) fn insert_collection_status(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    identity: &str,
    status: &str,
) -> worth_query::facade::foundation::WorthQueryEntityIdentity {
    let receipt = workspace
        .with_execution_request_mut(|execution, mut workspace| {
            workspace
                .insert(execution, "WorthUiProjectionText", |entity| {
                    entity
                        .set_aspect(
                            WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")
                                .expect("identity touch"),
                            WorthQueryAuthoredAspectValue::string(identity),
                        )
                        .set_aspect(
                            WorthQueryAspectTouch::from_authoring_ingress_text("query_text.status")
                                .expect("projection status touch"),
                            WorthQueryAuthoredAspectValue::string(status),
                        )
                        .set_aspect(
                            WorthQueryAspectTouch::from_authoring_ingress_text(
                                "collection_item.status",
                            )
                            .expect("collection status touch"),
                            WorthQueryAuthoredAspectValue::string(status),
                        )
                        .set_aspect(
                            WorthQueryAspectTouch::from_authoring_ingress_text(
                                "collection_item.key",
                            )
                            .expect("application item-key touch"),
                            WorthQueryAuthoredAspectValue::native(
                                worth_foundational::AspectValue::UInt64(fixture_item_key(identity)),
                            ),
                        )
                })
                .expect("projection text insertion")
        })
        .expect("workspace host call opens");
    receipt.deltas()[0].entity_identity().clone()
}

fn fixture_item_key(identity: &str) -> u64 {
    let value = identity.bytes().fold(0xcbf2_9ce4_8422_2325, |value, byte| {
        (value ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    });
    value.max(1)
}

pub(crate) fn update_status(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    entity: worth_query::facade::foundation::WorthQueryEntityIdentity,
    status: &str,
) {
    workspace
        .with_execution_request_mut(|execution, mut workspace| {
            workspace
                .update(execution, entity, |record| {
                    record
                        .set_aspect(
                            WorthQueryAspectTouch::from_authoring_ingress_text("query_text.status")
                                .expect("projection status touch"),
                            WorthQueryAuthoredAspectValue::string(status),
                        )
                        .set_aspect(
                            WorthQueryAspectTouch::from_authoring_ingress_text(
                                "collection_item.status",
                            )
                            .expect("collection status touch"),
                            WorthQueryAuthoredAspectValue::string(status),
                        )
                })
                .expect("projection text update")
        })
        .expect("workspace host call opens");
}

pub(crate) fn update_identity(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    entity: worth_query::facade::foundation::WorthQueryEntityIdentity,
    identity: &str,
) {
    workspace
        .with_execution_request_mut(|execution, mut workspace| {
            workspace
                .update(execution, entity, |record| {
                    record.set_aspect(
                        WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")
                            .expect("identity touch"),
                        WorthQueryAuthoredAspectValue::string(identity),
                    )
                })
                .expect("projection identity update")
        })
        .expect("workspace host call opens");
}
