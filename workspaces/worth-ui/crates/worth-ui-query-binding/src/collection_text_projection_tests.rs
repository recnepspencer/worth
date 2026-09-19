use worth_foundational::facade::{AspectValue, InternedString};
use worth_query::facade::{
    domain::{
        WorthQueryOperationCollectionContract, WorthQueryOperationCollectionField,
        WorthQueryOperationContinuationPosture, WorthQueryOperationWindowPolicy,
    },
    installed::{collection, operation},
    runtime::{WorthQueryAspectTouch, WorthQueryAuthoredAspectValue},
};

use crate::{
    installed_domain::{
        collection_text_projection::{
            WorthUiCollectionTextProjection, WorthUiCollectionTextProjectionFamily,
        },
        execution_resources::operation_execution_resource_request,
    },
    scalar_text_projection_fixture::collection_projection_workspace,
    WorthUiQueryWorkspaceExt,
};

#[test]
fn installed_collection_text_meaning_exposes_native_rows_and_exact_contract() {
    let mut workspace = collection_projection_workspace();
    insert_status(&mut workspace, "pulse.alpha", "Alpha");
    insert_status(&mut workspace, "pulse.bravo", "Bravo");
    let (collection, key) = prepare_collection(&mut workspace, 8);
    let actual = collection
        .rows()
        .iter()
        .map(|row| {
            collection
                .native_value(row, &key)
                .expect("own row and key")
                .native_value()
                .scalar()
                .cloned()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        actual,
        [
            Some(AspectValue::String(InternedString::Raw("Alpha".into()))),
            Some(AspectValue::String(InternedString::Raw("Bravo".into()))),
        ]
    );
}

#[test]
fn collection_native_access_rejects_foreign_row_and_key_with_indexed_cost() {
    let mut owner = collection_projection_workspace();
    let mut foreign = collection_projection_workspace();
    insert_status(&mut owner, "pulse.owner", "Owner");
    insert_status(&mut foreign, "pulse.foreign", "Foreign");
    let (owner_collection, owner_key) = prepare_collection(&mut owner, 8);
    let (foreign_collection, foreign_key) = prepare_collection(&mut foreign, 8);
    let owner_row = &owner_collection.rows()[0];
    let foreign_row = &foreign_collection.rows()[0];
    let access = owner_collection
        .native_value(owner_row, &owner_key)
        .expect("own row and selected key");

    assert_eq!(access.row_identity(), owner_row.entity_identity());
    assert_eq!(access.counters().capability_checks, 1);
    assert_eq!(access.counters().window_row_checks, 1);
    assert_eq!(access.counters().selected_key_checks, 1);
    assert_eq!(access.counters().indexed_row_lookups, 1);
    assert_eq!(access.counters().native_facts_materialized, 1);
    assert!(matches!(
        owner_collection.native_value(foreign_row, &owner_key),
        Err(collection::WorthQueryCollectionRowAccessDenial::ForeignRowHandle)
    ));
    assert!(matches!(
        owner_collection.native_value(owner_row, &foreign_key),
        Err(collection::WorthQueryCollectionRowAccessDenial::ForeignNativeAccessKey)
    ));
}

fn prepare_collection(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    breadth: u32,
) -> (
    collection::WorthQueryCollectionConsumerWindow,
    operation::WorthQueryNativeAccessKey,
) {
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .expect("observation world")
        .family(WorthUiCollectionTextProjectionFamily)
        .bind(installed.handle(), WorthUiCollectionTextProjection)
        .expect("collection text operation binds");
    let consumer = bound
        .consumer_projection_contract()
        .expect("collection text consumer contract");
    assert_collection_contract(consumer.collection());
    let mut builder = consumer.projection_request();
    let selection = builder
        .select_display_native_field_name("status")
        .expect("declared status field");
    let request = builder.build().expect("bound native request");
    let key = request
        .resolve_native_key(&selection)
        .expect("Query resolves selected status")
        .into_key();
    let settled = bound
        .admit_execution_resources((), operation_execution_resource_request(), workspace)
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();
    let collection = settled
        .prepare_collection_consumer(collection_breadth(breadth))
        .expect("Query prepares exact collection window");
    (collection, key)
}

fn assert_collection_contract(contract: &WorthQueryOperationCollectionContract) {
    let WorthQueryOperationCollectionContract::Collection {
        row_identity_field,
        window,
        continuation,
        ..
    } = contract
    else {
        panic!("collection text meaning cannot degrade to scalar");
    };
    assert_eq!(
        row_identity_field,
        &WorthQueryOperationCollectionField::from_dotted("identity.id").unwrap()
    );
    assert_eq!(
        *window,
        WorthQueryOperationWindowPolicy::ContinuationBounded
    );
    assert_eq!(
        *continuation,
        WorthQueryOperationContinuationPosture::LiveCursor
    );
}

fn insert_status(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    identity: &str,
    status: &str,
) {
    workspace
        .insert("WorthUiProjectionText", |entity| {
            entity
                .set_aspect(
                    WorthQueryAspectTouch::from_authoring_ingress_text("identity.id").unwrap(),
                    WorthQueryAuthoredAspectValue::string(identity),
                )
                .set_aspect(
                    WorthQueryAspectTouch::from_authoring_ingress_text("collection_item.status")
                        .unwrap(),
                    WorthQueryAuthoredAspectValue::string(status),
                )
                .set_aspect(
                    WorthQueryAspectTouch::from_authoring_ingress_text("collection_item.key")
                        .unwrap(),
                    WorthQueryAuthoredAspectValue::native(AspectValue::UInt64(
                        identity
                            .bytes()
                            .fold(1_u64, |key, byte| key.wrapping_mul(31) + u64::from(byte))
                            .max(1),
                    )),
                )
        })
        .expect("projection text insertion");
}

fn collection_breadth(
    width: u32,
) -> worth_query::facade::domain::WorthQueryCollectionWindowBreadth {
    worth_query::facade::domain::WorthQueryCollectionWindowBreadth::new(width, 0, 0, width)
        .expect("test breadth")
}
