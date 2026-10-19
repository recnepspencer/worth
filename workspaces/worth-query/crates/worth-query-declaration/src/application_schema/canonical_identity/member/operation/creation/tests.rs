//! Enrollment is canonical meaning, while absence preserves historical bytes.
use crate::application_schema::canonical_identity::{
    canonical_identity, ApplicationSchemaCanonicalHeader,
};
use crate::application_schema::{
    ApplicationOperationBodyCreation, ApplicationOperationBodyEnrollment,
    ApplicationOperationBodyProgram, ApplicationSchemaMember,
};

#[test]
fn workflow_enrollment_and_each_target_axis_change_schema_identity() {
    let identity = |enrollment| {
        let creation = ApplicationOperationBodyCreation::from_untrusted(
            "Subject".into(),
            "subject".into(),
            vec![],
        )
        .with_untrusted_enrollment(enrollment);
        let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Container".into(),
            vec![],
            Some(creation),
            vec![],
        );
        canonical_identity(
            ApplicationSchemaCanonicalHeader {
                owner: "test",
                name: "birth",
                major: 1,
                minor: 0,
            },
            &[ApplicationSchemaMember::OperationBody {
                operation: "Create".into(),
                body,
            }],
            &[],
        )
    };
    let enrolled = |workflow: &str, binding: &str, role: &str| {
        Some(ApplicationOperationBodyEnrollment::from_untrusted(
            workflow.into(),
            binding.into(),
            role.into(),
        ))
    };
    let absent = identity(None);
    let base = identity(enrolled("workflow", "binding", "role"));
    assert_ne!(absent, base);
    for target in [
        enrolled("other-workflow", "binding", "role"),
        enrolled("workflow", "other-binding", "role"),
        enrolled("workflow", "binding", "other-role"),
    ] {
        assert_ne!(base, identity(target));
    }
    assert_eq!(
        absent,
        identity(None),
        "unused enrollment adds no schema entries"
    );
}
