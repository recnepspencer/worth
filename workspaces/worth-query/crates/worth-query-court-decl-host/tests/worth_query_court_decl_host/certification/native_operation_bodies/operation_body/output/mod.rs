pub(crate) use worth_query_testkit_certification_operation::facade::certification::native_operation_bodies::operation_body::output::*;

#[test]
fn created_and_scope_outputs_name_the_same_real_row_and_replay_keeps_correspondence() {
    let app = install::<31>();
    let Outcome::Committed { receipt, .. } = create(&app, 310) else {
        panic!("ordinary genesis")
    };
    let created = receipt
        .outputs_of::<CreatedOutputs<31>>()
        .unwrap()
        .entity::<CreatedOutput<31>>()
        .unwrap()
        .expect("authored Create output is present")
        .entity_id();
    assert!(receipt.outputs_of::<ScopeOutputs<31>>().is_err());
    assert!(receipt
        .outputs_of::<CreatedOutputs<31>>()
        .unwrap()
        .entity::<AbsentOutput<31>>()
        .unwrap()
        .is_none());
    let Outcome::AlreadyCommitted(replayed) = create(&app, 310) else {
        panic!("replay")
    };
    assert_eq!(
        replayed
            .outputs_of::<CreatedOutputs<31>>()
            .unwrap()
            .entity::<CreatedOutput<31>>()
            .unwrap()
            .expect("replayed Create output is present")
            .entity_id(),
        created
    );
    let Outcome::Committed { receipt, .. } = invoke(&app, 311) else {
        panic!("output-only preserve")
    };
    assert_eq!(
        receipt
            .outputs_of::<ScopeOutputs<31>>()
            .unwrap()
            .entity::<ScopeOutput<31>>()
            .unwrap()
            .expect("authored Preserve output is present")
            .entity_id(),
        created
    );
    assert_eq!(
        read_retention(&app),
        7,
        "preservation must not fabricate a business write"
    );
    let body = program::schema::<31>().erased().members().iter().find_map(|member| match member {
        worth_query_host::facade::declaration::application_schema::ApplicationSchemaMember::OperationBody { operation, body } if operation == "SetRetention" => Some(body.clone()), _ => None,
    }).unwrap();
    assert!(body.scope_fields().is_empty());
    assert!(body.creation().is_none());
    assert_eq!(body.outputs().len(), 1);
}

pub(crate) mod portable;

pub(crate) mod related;
