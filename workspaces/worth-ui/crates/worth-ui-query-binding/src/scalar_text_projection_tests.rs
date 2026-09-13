use worth_foundational::facade::{AspectValue, InternedString};
use worth_query::facade::domain::{
    WorthQueryConsumerProjectionContractDenial, WorthQueryConsumerSupportDimension,
};
use worth_query::facade::{
    installed, read,
    runtime::{WorthQueryAspectTouch, WorthQueryAuthoredAspectValue},
};

use crate::{
    scalar_text_projection_fixture::projection_workspace, WorthUiQueryWorkspaceExt,
    WorthUiScalarTextProjection, WorthUiScalarTextProjectionFamily,
};

#[test]
fn installed_scalar_text_meaning_executes_and_consumes_native_status() {
    let mut workspace = projection_workspace(true);
    let write = workspace
        .insert("WorthUiProjectionText", |entity| {
            entity
                .set_aspect(
                    WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")
                        .expect("identity touch"),
                    WorthQueryAuthoredAspectValue::string("platform.pulse.status"),
                )
                .set_aspect(
                    WorthQueryAspectTouch::from_authoring_ingress_text("query_text.status")
                        .expect("projection status touch"),
                    WorthQueryAuthoredAspectValue::string("Ready"),
                )
        })
        .expect("projection text insertion");
    assert_eq!(write.declared_aspect_operations().len(), 2);
    assert!(write
        .declared_aspect_operations()
        .iter()
        .any(|operation| operation.aspect_touch().native_aspect_key().as_str() == "query_text"));
    assert_independent_detail_read(&mut workspace);
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .expect("observation world")
        .family(WorthUiScalarTextProjectionFamily)
        .bind(installed.handle(), WorthUiScalarTextProjection)
        .expect("scalar text operation binds");
    let consumer = bound
        .consumer_projection_contract()
        .expect("scalar text consumer contract");
    let mut request = consumer.projection_request();
    let selection = request
        .select_display_native_field_name("status")
        .expect("declared status field");
    let request = request.build().expect("bound native request");
    let key = request
        .resolve_native_key(&selection)
        .expect("Query resolves the selected native key")
        .into_key();
    let settled = bound
        .admit_execution_resources(
            (),
            crate::installed_domain::execution_resources::operation_execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();

    assert_eq!(settled.counters().primary_read_contacts, 1);
    assert_eq!(
        settled
            .native_value(&key, 0)
            .expect("exact Query key accesses the settled native value")
            .value()
            .scalar(),
        Some(&AspectValue::String(InternedString::Raw("Ready".into())))
    );
}

fn assert_independent_detail_read(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) {
    let declaration = read::declare(|builder| {
        builder.local_detail(
            "WorthUiProjectionText",
            read::QuerySchemaView::new(
                "test-owned-scalar-text-projection",
                [
                    read::SchemaFieldView::new(
                        read::AspectName::new("identity").unwrap(),
                        read::FieldName::new("id").unwrap(),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                    read::SchemaFieldView::new(
                        read::AspectName::new("query_text").unwrap(),
                        read::FieldName::new("status").unwrap(),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                ],
                [],
            ),
            |query| {
                query
                    .project(read::AspectFieldSelector::new("identity", "id").unwrap())
                    .project(read::AspectFieldSelector::new("query_text", "status").unwrap())
                    .where_equal(
                        read::EqualityPredicate::new(
                            "identity",
                            "id",
                            read::WorthQueryPredicateOperand::string(
                                "platform.pulse.status".to_owned(),
                            ),
                        )
                        .unwrap(),
                    )
            },
            |shape| {
                shape
                    .field(read::AuthoredResultShapeField::new("identity", "id", "id").unwrap())
                    .field(
                        read::AuthoredResultShapeField::new(
                            "query_text",
                            "status",
                            "query_text.status",
                        )
                        .unwrap(),
                    )
            },
        )
    })
    .expect("test-owned detail declaration");
    let completion = declaration
        .using(read::current())
        .run(workspace)
        .into_result()
        .expect("test-owned detail read");
    let rows = completion.result().rows();
    assert_eq!(
        completion
            .result()
            .receipt()
            .breadth()
            .execution_query_projection_count(),
        2
    );
    assert_eq!(rows.len(), 1);
    let fields = rows[0].terminal_field_value_projection();
    assert_eq!(
        fields.get("query_text.status"),
        Some(&AspectValue::String(InternedString::Raw("Ready".into()))),
        "returned fields: {fields:?}"
    );
}

#[test]
fn scalar_text_consumer_contract_requires_query_async_and_recovery_support() {
    let workspace = projection_workspace(false);
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .expect("observation world")
        .family(WorthUiScalarTextProjectionFamily)
        .bind(installed.handle(), WorthUiScalarTextProjection)
        .expect("scalar text operation binds");
    let denial = match bound.consumer_projection_contract() {
        Ok(_) => panic!("unsupported async state must deny consumer authority"),
        Err(denial) => denial,
    };

    let WorthQueryConsumerProjectionContractDenial::Compatibility(denial) = denial else {
        panic!("unsupported async state must return Query's compatibility denial");
    };
    assert_eq!(
        denial.dimension(),
        WorthQueryConsumerSupportDimension::AsyncResultState
    );
}

#[test]
fn missing_exact_scalar_entity_fails_consumption_without_a_native_value() {
    let mut workspace = projection_workspace(true);
    let installed_domain = workspace.worth_ui().expect("Worth UI domain installed");
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .expect("observation world")
        .family(WorthUiScalarTextProjectionFamily)
        .bind(installed_domain.handle(), WorthUiScalarTextProjection)
        .expect("scalar text operation binds");
    let mut builder = bound
        .consumer_projection_contract()
        .expect("supported consumer contract")
        .projection_request();
    builder
        .select_display_native_field_name("status")
        .expect("declared status field");
    let request = builder.build().expect("bound native request");
    let published = bound
        .admit_execution_resources(
            (),
            crate::installed_domain::execution_resources::operation_execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap();

    assert!(matches!(
        installed::transition::consumption(published.consume_bound(request)),
        installed::transition::WorthQueryConsumptionTransition::Failed(_)
    ));
}
