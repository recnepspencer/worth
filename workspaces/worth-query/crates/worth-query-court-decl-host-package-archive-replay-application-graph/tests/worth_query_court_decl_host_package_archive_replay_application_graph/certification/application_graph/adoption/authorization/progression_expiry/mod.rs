pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::adoption::authorization::progression_expiry::*;

#[test]
fn expiry_during_capability_progression_preserves_the_native_prepared_owner() {
    let clock = ExpiryClock::default();
    let host = worth_query_testkit_document_retention::facade::document_retention_model::host::publish_on_first_program_with_trusted_time(
        clock.clone(),
    );
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let current = authenticate_operator(host.installed_schema(), &scope);
    let programs = host.runtime().request(&current, &scope).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    let before = host
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(branch)
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap();
    let expiring = authenticate_live_expiring(&host, &scope);
    clock.expire_during_progression(expiring.valid_until());
    let prepared = match programs_publication(&host, prepared, &expiring, &scope) {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, prepared } => {
            expired(&cause);
            prepared
        }
        _ => panic!("lapsed admitted authentication must not publish"),
    };
    clock.assert_serviced();
    let after = host
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(branch)
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap();
    assert_eq!(before, after);
    assert_eq!(prepared.target(), &target);
    assert_eq!(read_retention(host.runtime(), branch), 7);
    assert!(matches!(
        host.runtime()
            .request(&current, &scope)
            .programs()
            .publish(prepared),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}

#[test]
fn expiry_during_progression_preserves_branch_set_prefix_and_pending_owner() {
    let clock = ExpiryClock::default();
    let host = worth_query_testkit_document_retention::facade::document_retention_model::host::publish_on_first_program_with_trusted_time(
        clock.clone(),
    );
    let a = host.current_world();
    let b = super::branch_set::fork(&host);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let current = authenticate_operator(host.installed_schema(), &scope);
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b], NonZeroUsize::new(2).unwrap())
        .unwrap();
    let mut adoption = host
        .runtime()
        .request(&current, &scope)
        .on_branches(coverage, &[a, b])
        .unwrap()
        .programs()
        .adopt(&target)
        .unwrap()
        .prepare(64)
        .unwrap();
    assert!(matches!(
        adoption
            .advance(host.runtime(), &current, &scope)
            .unwrap()
            .unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { .. }
    ));
    let expiring = authenticate_live_expiring(&host, &scope);
    clock.expire_during_progression(expiring.valid_until());
    match adoption
        .advance(host.runtime(), &expiring, &scope)
        .err()
        .unwrap()
    {
        WorthQueryBranchSetAdoptionAdvanceDenial::Authorization { branch, cause } => {
            assert_eq!(branch, b);
            expired(&cause);
        }
        other => panic!("expired next-branch admission must preserve pending custody: {other:?}"),
    }
    clock.assert_serviced();
    assert_eq!(adoption.progress().len(), 1);
    assert_eq!(
        host.runtime()
            .request(&current, &scope)
            .on_branch(a)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        &target
    );
    assert_eq!(
        host.runtime()
            .request(&current, &scope)
            .on_branch(b)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        host.owned_revision()
    );
    assert!(matches!(
        adoption
            .advance(host.runtime(), &current, &scope)
            .unwrap()
            .unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { .. }
    ));
    assert!(adoption.close().is_ok());
}
