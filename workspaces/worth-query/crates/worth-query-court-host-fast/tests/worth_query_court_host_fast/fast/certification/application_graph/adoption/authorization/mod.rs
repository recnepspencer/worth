pub(crate) use worth_query_worlds::facade::certification::application_graph::adoption::authorization::*;

pub(crate) mod branch_set;

pub(crate) mod lifecycle;

pub(crate) mod progression_expiry;

pub(crate) mod selected_program;

#[test]
fn authorized_single_branch_adopts_the_real_rostered_revision() {
    let host = publish_on_first_program();
    let source = *host.owned_revision();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host.runtime().request(&principal, &scope).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    assert_eq!(prepared.source(), &source);
    match host
        .runtime()
        .request(&principal, &scope)
        .programs()
        .publish(prepared)
    {
        WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) => {
            assert_eq!(performed.target(), &target)
        }
        _ => panic!("authorized native publication must perform"),
    }
    let inspection = host
        .runtime()
        .request(&principal, &scope)
        .programs()
        .inspect()
        .unwrap();
    assert_eq!(inspection.revision(), &target);
    assert_eq!(read_retention(host.runtime(), host.current_world()), 7);
}

#[test]
fn revoked_grant_denies_compare_adopt_prepare_and_publication_without_effects() {
    let host = publish_on_first_program();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let requirements = host
        .runtime()
        .request(&principal, &scope)
        .programs()
        .compare(&target)
        .unwrap();
    let to_prepare = host
        .runtime()
        .request(&principal, &scope)
        .programs()
        .adopt(&requirements)
        .unwrap();
    let prepared = host
        .runtime()
        .request(&principal, &scope)
        .programs()
        .adopt(&requirements)
        .unwrap()
        .prepare(64)
        .unwrap();
    revoke(&host);
    let before = host.current_world();
    let programs = host.runtime().request(&principal, &scope).programs();
    for denial in [
        programs.compare(&target).err().unwrap(),
        programs.adopt(&requirements).err().unwrap(),
    ] {
        match denial {
            WorthQueryApplicationProgramAdoptionPreparationDenial::Authorization(cause) => {
                permission_denied(cause)
            }
            other => panic!("fresh entrance must deny capability: {other:?}"),
        }
    }
    match to_prepare.prepare(64).err().unwrap() {
        WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial::Authorization(cause)) => permission_denied(cause),
        other => panic!("preparation must refuse the capability: {other:?}"),
    }
    match prepared.publish(host.runtime(), &principal, &scope) {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, prepared } => {
            permission_denied(cause);
            assert_eq!(prepared.target(), &target);
            drop(prepared);
        }
        _ => panic!("publication must recheck the revoked grant"),
    }
    assert_eq!(host.current_world(), before);
    assert_eq!(
        host.runtime()
            .request(&principal, &scope)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        host.owned_revision()
    );
    assert_eq!(read_retention(host.runtime(), before), 7);
}

#[test]
fn foreign_request_branch_preserves_the_exact_prepared_owner_for_correct_context() {
    let host = publish_on_first_program();
    let original = host.current_world();
    let sibling = host
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .fork(original)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(original)
        .programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    let prepared = match host
        .runtime()
        .request(&principal, &scope)
        .on_branch(sibling)
        .programs()
        .publish(prepared)
    {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied {
            cause:
                WorthQueryProgramAdoptionAuthorizationDenial::ForeignBranch {
                    expected,
                    requested,
                },
            prepared,
        } => {
            assert_eq!(expected, original);
            assert_eq!(requested, sibling);
            prepared
        }
        _ => panic!("a different request branch must refuse without consuming the native owner"),
    };
    assert!(matches!(
        host.runtime()
            .request(&principal, &scope)
            .on_branch(original)
            .programs()
            .publish(prepared),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    assert_eq!(
        host.runtime()
            .request(&principal, &scope)
            .on_branch(sibling)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        host.owned_revision()
    );
}
