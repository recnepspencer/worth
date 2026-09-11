use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use super::{authenticated_principal, live_scope, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, AccountStatus, ExactStatusRetentionOperation,
    RetainedStatusEffect, RetainedStatusNotice,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;

#[test]
fn structured_effect_validation_precedes_retention_and_external_projection() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountStatus::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();

    let retained = effects
        .emit(
            RetainedStatusEffect::reference(),
            RetainedStatusNotice(String::new()),
        )
        .unwrap_err();
    assert_eq!(
        retained.kind(),
        WorthQueryApplicationAttemptDenialKind::InvalidEffectValue
    );
    let external = effects
        .emit_external(
            RetainedStatusEffect::reference(),
            RetainedStatusNotice(String::new()),
        )
        .unwrap_err();
    assert_eq!(
        external.kind(),
        WorthQueryApplicationAttemptDenialKind::InvalidEffectValue
    );
}
