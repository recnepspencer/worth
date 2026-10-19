//! Portable enrollment closure is checked again, not trusted from transport.
use super::{valid, ApplicationOperationBodyEnrollment};
use crate::application_operation::{
    ApplicationMutationOutputPosture, ApplicationMutationOutputRoleCardinality,
    ApplicationMutationOutputRoleDescription,
};
use crate::application_schema::{
    ApplicationOperationBodyCreation, ApplicationOperationBodyOutput,
    ApplicationOperationBodyOutputTarget, ApplicationOperationBodyProgram,
};

#[test]
fn decoded_enrollment_requires_exact_create_slot_binding_role_entity_and_cardinality() {
    let body = |binding: &str, role: &str, entity: &str, key: &str, posture, cardinality| {
        ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Container".into(),
            vec![],
            Some(
                ApplicationOperationBodyCreation::from_untrusted(
                    "Subject".into(),
                    "subject".into(),
                    vec![],
                )
                .with_untrusted_enrollment(Some(
                    ApplicationOperationBodyEnrollment::from_untrusted(
                        "workflow".into(),
                        "binding".into(),
                        "role".into(),
                    ),
                )),
            ),
            vec![],
        )
        .with_untrusted_outputs(vec![ApplicationOperationBodyOutput {
            binding: binding.into(),
            role: ApplicationMutationOutputRoleDescription {
                name: role.into(),
                entity: entity.into(),
                posture,
                cardinality,
            },
            target: ApplicationOperationBodyOutputTarget::Creation { key: key.into() },
        }])
    };
    let create = ApplicationMutationOutputPosture::Create;
    let one = ApplicationMutationOutputRoleCardinality::ExactlyOne;
    assert!(valid(&body(
        "binding", "role", "Subject", "subject", create, one
    )));
    for corrupted in [
        body("other", "role", "Subject", "subject", create, one),
        body("binding", "other", "Subject", "subject", create, one),
        body("binding", "role", "Other", "subject", create, one),
        body("binding", "role", "Subject", "other", create, one),
        body(
            "binding",
            "role",
            "Subject",
            "subject",
            ApplicationMutationOutputPosture::Preserve,
            one,
        ),
        body(
            "binding",
            "role",
            "Subject",
            "subject",
            create,
            ApplicationMutationOutputRoleCardinality::AtMostOne,
        ),
    ] {
        assert!(!valid(&corrupted));
    }
}
