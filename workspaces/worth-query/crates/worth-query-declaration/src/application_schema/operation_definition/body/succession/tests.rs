//! Cold descriptions must retain exactly the existing scope correspondence.
use super::*;
use crate::application_operation::ApplicationMutationOutputRoleDescription;
use crate::application_schema::{
    ApplicationOperationBodyOutput, ApplicationOperationBodyOutputTarget,
};

#[test]
fn succession_requires_exact_preserved_scope_and_canonical_target_identity() {
    let output = ApplicationOperationBodyOutput {
        binding: "renew".into(),
        role: ApplicationMutationOutputRoleDescription {
            name: "root".into(),
            entity: "Root".into(),
            posture: Posture::Preserve,
            cardinality: Cardinality::ExactlyOne,
        },
        target: ApplicationOperationBodyOutputTarget::Scope,
    };
    let attachment = ApplicationOperationBodySuccession::from_untrusted(
        "workflow".into(),
        "renew".into(),
        "root".into(),
    );
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Root".into(),
        vec![],
        None,
        vec![],
    )
    .with_untrusted_outputs(vec![output.clone()])
    .with_untrusted_succession(Some(attachment.clone()));
    assert!(valid(&body));
    for changed in [
        ApplicationOperationBodySuccession::from_untrusted(
            "workflow".into(),
            "other-binding".into(),
            "root".into(),
        ),
        ApplicationOperationBodySuccession::from_untrusted(
            "workflow".into(),
            "renew".into(),
            "other-role".into(),
        ),
        ApplicationOperationBodySuccession::from_untrusted(
            " workflow".into(),
            "renew".into(),
            "root".into(),
        ),
    ] {
        assert_ne!(attachment.canonical_record(), changed.canonical_record());
        assert!(!valid(
            &body.clone().with_untrusted_succession(Some(changed))
        ));
    }
    for posture in [Posture::Create, Posture::Retire] {
        let mut changed = output.clone();
        changed.role.posture = posture;
        assert!(!valid(&body.clone().with_untrusted_outputs(vec![changed])));
    }
    let mut changed = output;
    changed.role.entity = "Other".into();
    assert!(!valid(&body.clone().with_untrusted_outputs(vec![changed])));
    assert!(!valid(&body.with_untrusted_outputs(vec![])));
}
