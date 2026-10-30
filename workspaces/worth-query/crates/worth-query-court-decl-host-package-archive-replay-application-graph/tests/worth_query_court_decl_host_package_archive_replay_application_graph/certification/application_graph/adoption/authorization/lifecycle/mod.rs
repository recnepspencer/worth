pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::adoption::authorization::lifecycle::*;

#[test]
fn expired_authentication_retains_prepared_and_pending_owners_for_fresh_publication() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = super::branch_set::fork(&host);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let current = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&current, &scope)
        .on_branch(a)
        .programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[b], NonZeroUsize::new(1).unwrap())
        .unwrap();
    let mut pending = host
        .runtime()
        .request(&current, &scope)
        .on_branches(coverage, &[b])
        .unwrap()
        .programs()
        .adopt(&target)
        .unwrap()
        .prepare(64)
        .unwrap();
    let stale = expired_operator(&host, &scope);
    let prepared = match prepared.publish(host.runtime(), &stale, &scope) {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, prepared } => {
            expired(cause);
            prepared
        }
        _ => panic!("expiry must retain the reservation"),
    };
    match pending
        .advance(host.runtime(), &stale, &scope)
        .err()
        .unwrap()
    {
        WorthQueryBranchSetAdoptionAdvanceDenial::Authorization { cause, .. } => expired(cause),
        other => panic!("expiry must retain pending native custody: {other:?}"),
    }
    assert_eq!(host.current_world(), a);
    assert_eq!(read_retention(host.runtime(), a), 7);
    assert!(matches!(
        prepared.publish(host.runtime(), &current, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    assert!(matches!(
        pending
            .advance(host.runtime(), &current, &scope)
            .unwrap()
            .unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { .. }
    ));
    assert!(pending.close().is_ok());
}
