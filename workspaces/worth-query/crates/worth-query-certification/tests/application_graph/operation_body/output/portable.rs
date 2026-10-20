//! Descriptions cannot substitute for their exact installed roles or target slots.
use super::*;
use worth_query_host::facade::{
    application_installation::{
        in_memory_program_from_checkpoint, WorthQueryCheckpointCapturePolicy,
    },
    declaration::application_schema::*,
};

#[test]
fn portable_outputs_preserve_meaning_and_reject_mismatched_contract_or_target() {
    let schema = program::schema::<31>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(portable.clone()).unwrap(),
        *schema.erased()
    );
    for defect in [
        "binding",
        "role",
        "entity",
        "posture",
        "cardinality",
        "slot",
        "mandatory",
        "duplicate",
    ] {
        let mut parts = portable.clone().into_parts();
        for member in &mut parts.members {
            if let ApplicationSchemaMember::OperationBody { operation, body } = member {
                if operation != "CreateDocument" {
                    continue;
                }
                let mut outputs = body.outputs().to_vec();
                match defect {
                    "binding" => outputs[0].binding = "worth.query.foreign.v1".to_owned(),
                    "role" => outputs[0].role.name = "other".to_owned(),
                    "entity" => outputs[0].role.entity = "Tenant".to_owned(),
                    "posture" => {
                        outputs[0].role.posture = ApplicationMutationOutputPosture::Preserve
                    }
                    "cardinality" => {
                        outputs[0].role.cardinality =
                            ApplicationMutationOutputRoleCardinality::ExactlyOne
                    }
                    "slot" => {
                        outputs[0].target = ApplicationOperationBodyOutputTarget::Creation {
                            key: "foreign".to_owned(),
                        }
                    }
                    "mandatory" => outputs.clear(),
                    "duplicate" => outputs.push(outputs[0].clone()),
                    _ => unreachable!(),
                }
                *body = body.clone().with_untrusted_outputs(outputs);
            }
        }
        if defect == "mandatory" {
            for member in &mut parts.members {
                if let ApplicationSchemaMember::ApplicationMutation { description } = member {
                    if description.operation() == "CreateDocument" {
                        let mut binding = description.clone().into_parts();
                        binding.output_roles[0].cardinality =
                            ApplicationMutationOutputRoleCardinality::ExactlyOne;
                        *description =
                            ApplicationMutationDescription::from_untrusted_parts(binding);
                    }
                }
            }
        }
        parts.members.sort();
        assert!(
            validate_portable_application_schema_freshly(
                WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
            )
            .is_err(),
            "{defect} must not readmit"
        );
    }
}

#[test]
fn output_only_body_requires_a_real_entity_decision_read() {
    let schema = program::schema::<31>();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(schema.erased()).into_parts();
    let member = parts.members.iter_mut().find(|member| matches!(member,
        ApplicationSchemaMember::OperationDecisionRead { operation, target: ApplicationOperationDecisionReadTarget::Entity { entity } }
            if operation == "SetRetention" && entity == "Document"
    )).expect("output-only declaration derives scope Entity membership");
    if let ApplicationSchemaMember::OperationDecisionRead { target, .. } = member {
        *target = ApplicationOperationDecisionReadTarget::Entity {
            entity: "Principal".to_owned(),
        };
    }
    parts.members.sort();
    assert_eq!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency
    );
}

#[test]
fn output_bodies_reconstruct_and_new_receipts_rebind_exact_entity_types() {
    let old = install::<31>();
    assert!(matches!(create(&old, 312), Outcome::Committed { .. }));
    let checkpoint = old
        .with_host_advancement(|phase| {
            old.runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    drop(old);
    let schema = program::schema::<31>();
    let restored = in_memory_program_from_checkpoint(
        program::validated::<31>(&schema),
        schema,
        (program::Installation::Native,),
        host_limits(),
        checkpoint,
    )
    .unwrap();
    assert!(matches!(
        create(&restored, 312),
        Outcome::PreviouslyCommitted(_)
    ));
    let Outcome::Committed { receipt, .. } = invoke(&restored, 313) else {
        panic!("restored preserve")
    };
    let preserved = receipt
        .outputs_of::<ScopeOutputs<31>>()
        .unwrap()
        .entity::<ScopeOutput<31>>()
        .unwrap()
        .expect("restored Preserve output is present");
    assert!(receipt.outputs_of::<CreatedOutputs<31>>().is_err());
    assert_eq!(read_retention(&restored), 7);
    let Outcome::AlreadyCommitted(replayed) = invoke(&restored, 313) else {
        panic!("restored replay")
    };
    assert_eq!(
        replayed
            .outputs_of::<ScopeOutputs<31>>()
            .unwrap()
            .entity::<ScopeOutput<31>>()
            .unwrap()
            .expect("replayed Preserve output is present")
            .entity_id(),
        preserved.entity_id()
    );
}
