pub(crate) use worth_query_testkit_certification_operation::facade::certification::native_operation_bodies::operation_body::requirement::*;

#[test]
fn changed_authored_requirement_changes_admission_with_the_same_consumer() {
    use worth_query_host::facade::declaration::application_schema::ApplicationSchemaMember;
    let required = program::schema::<20>();
    let removed = program::schema::<24>();
    let without_body = |members: &[ApplicationSchemaMember]| {
        members
            .iter()
            .filter(|member| {
                !matches!(member, ApplicationSchemaMember::OperationBody { .. })
                    && !matches!(
                        member,
                        ApplicationSchemaMember::OperationDecisionRead { operation, .. }
                            if operation == "RequireDraftNote"
                    )
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(
        without_body(required.erased().members()),
        without_body(removed.erased().members())
    );
    let (first_with_projection, first_projection) = super::install_with_projection::<20>();
    let (second, second_projection) = super::install_with_projection::<24>();
    assert_ne!(
        first_with_projection.installed_program().revision(),
        second.installed_program().revision()
    );
    assert!(matches!(
        invoke_requirement(&first_with_projection, 408),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::RequirementUnmet { .. })
    ));
    assert_eq!(
        guidance(&first_with_projection, &first_projection),
        vec![Status::Unmet]
    );
    assert!(guidance(&second, &second_projection).is_empty());
    assert!(matches!(
        invoke_requirement(&second, 408),
        Outcome::Committed { .. }
    ));
}

#[test]
fn one_body_requirement_guides_and_enforces_without_rejecting_draft_save() {
    let (app, projection) = super::install_with_projection::<20>();
    assert_eq!(guidance(&app, &projection), vec![Status::Unmet]);
    assert!(matches!(
        invoke::<RequireDraftNote>(&app, 401, ""),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::RequirementUnmet { .. })
    ));
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 402, ""),
        Outcome::Committed { .. }
    ));
    assert_eq!(super::draft_query::read_note(&app), Some(String::new()));
    assert_eq!(guidance(&app, &projection), vec![Status::Met]);
    assert!(matches!(
        invoke::<RequireDraftNote>(&app, 403, ""),
        Outcome::Committed { .. }
    ));
    assert!(matches!(
        invoke::<ClearDraftNote>(&app, 404, ""),
        Outcome::Committed { .. }
    ));
    assert_eq!(guidance(&app, &projection), vec![Status::Unmet]);
}

#[test]
fn clearing_after_preparation_stales_the_requirement_without_effect() {
    let (app, projection) = super::install_with_projection::<20>();
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 405, ""),
        Outcome::Committed { .. }
    ));
    assert_eq!(guidance(&app, &projection), vec![Status::Met]);
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let key = 406_u64;
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(DraftIntent::<20, RequireDraftNote>::new(""))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh requirement must prepare"),
    };
    assert!(matches!(
        invoke::<ClearDraftNote>(&app, 407, ""),
        Outcome::Committed { .. }
    ));
    match candidate.commit(ExecutionAllocationPolicy::SystemAllocation) {
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(stale)) => {
            assert!(stale.stale_fact_count() > 0)
        }
        other => panic!("cleared required field must stale: {other:?}"),
    }
    assert_eq!(super::draft_query::read_audit_source(&app), 7);
    assert_eq!(guidance(&app, &projection), vec![Status::Unmet]);
    assert_eq!(super::draft_query::read_note(&app), None);
}
