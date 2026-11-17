//! Actual adoption preparation perturbs Native issuance before a fork-backed Resource.
use super::{resource::Monitor, Configuration};
use crate::document_retention_model::{schema::DocumentRetentionSchema, MixedApplication};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*,
    application_installation::WorthQueryProgramOwner,
    primary_graph::WorthQueryPreparedBranchAdoption, product::WorthQueryProductBranch,
};

pub(super) struct PreparedSource {
    fork: WorthQueryProductBranch,
    original: WorthQueryApplicationReadObservation,
    adoption: WorthQueryPreparedBranchAdoption,
}

pub(super) fn prepare(
    application: &MixedApplication,
    actor: &WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    scope: &WorthQueryRequestScope,
) -> PreparedSource {
    let runtime = application.runtime();
    let fork = fork(application, scope);
    let original = runtime
        .request(actor, scope)
        .on_branch(fork)
        .retain_current_read()
        .unwrap();
    let target = *application
        .runtime()
        .supported_program::<crate::program::MixedProgramP1>()
        .expect("the actual alternate program is rostered")
        .owned_revision();
    let programs = runtime.request(actor, scope).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    assert_eq!(prepared.selected_entity_count(), 2);
    // Keep Native preparation until the intervening publication makes its
    // expected root stale; release its exact losing attempt before Home close.
    PreparedSource {
        fork,
        original,
        adoption: prepared,
    }
}

pub(super) fn run(
    application: &MixedApplication,
    configuration: &Configuration,
    actor: &WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    scope: &WorthQueryRequestScope,
    prepared: PreparedSource,
) {
    let runtime = application.runtime();
    let sibling = runtime.request(actor, scope).retain_current_read().unwrap();
    match runtime
        .request(actor, scope)
        .programs()
        .publish(prepared.adoption)
    {
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
            assert_eq!(
                no_effect.cause(),
                worth_query_host::facade::runtime::NoEffectCause::StaleExpectedProductHead
            );
        }
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, .. } => {
            panic!("adoption authorization: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::AdmissionDenied { cause, .. } => {
            panic!("adoption admission: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
            panic!("adoption execution: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ObligationDischargePending {
            cause, ..
        } => panic!("adoption discharge: {cause:?}"),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {
            panic!("adoption performed after monitor mutation")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            assert_eq!(
                unpublished.cause(),
                worth_query_host::facade::runtime::ProductUnpublishedCause::ProductPublicationLost
            );
            runtime
                .release_branch_adoption_recovery(unpublished.into_recovery(), 0)
                .expect("the stale prepared adoption releases its exact retained owner");
        }
    }
    let after = runtime.request(actor, scope).retain_current_read().unwrap();
    assert_eq!(
        sibling.selected_commit(),
        after.selected_commit(),
        "stale adoption must not publish another root or change its program"
    );
    assert_ne!(
        prepared.original.selected_commit(),
        sibling.selected_commit()
    );
    let reissue = |read: &WorthQueryApplicationReadObservation| {
        runtime
            .with_host_advancement(|phase| {
                read.select_on(&phase, runtime)
                    .unwrap()
                    .retain_application_read()
            })
            .unwrap()
    };
    let original = reissue(&prepared.original);
    let sibling = reissue(&sibling);
    let request = runtime.request(actor, scope).on_branch(prepared.fork);
    let resources = request.resources();
    let index = configuration.state.counts().0;
    let mut handle = resources
        .start::<Monitor>(61)
        .unwrap_or_else(|stop| panic!("source Resource: {:?}", stop.into_parts().1));
    let (same, original_native, version) = resources
        .plain_source_matches_read_for_test(&handle.receipt(), &original)
        .unwrap();
    assert!(
        same,
        "the actual Resource source is the independently retained fork"
    );
    assert_eq!(
        version, 0,
        "the populated fork resets only its branch version"
    );
    assert_ne!(original_native, version);
    let (same, sibling_native, _) = resources
        .plain_source_matches_read_for_test(&handle.receipt(), &sibling)
        .unwrap();
    assert!(!same, "the genuine sibling is not the Resource source");
    assert!(
        sibling_native > original_native + 1,
        "the real prepared adoption burns Native issuance before sibling publication"
    );
    configuration.state.send(index, 6161).unwrap();
    resources.advance(&mut handle).unwrap();
    assert_eq!(
        resources
            .with_result(&handle.receipt(), |value| *value)
            .unwrap(),
        Some(6161)
    );
    resources.close(&mut handle).unwrap();
}

// Keep the original unpublished fork owner until World completes its publication.
fn fork(application: &MixedApplication, scope: &WorthQueryRequestScope) -> WorthQueryProductBranch {
    use worth_query_host::facade::product::{
        WorthQueryProductBranchCreateError as Created,
        WorthQueryProductBranchCreationContinuation as Continued,
        WorthQueryProductBranchRecoveryDenial,
    };
    let runtime = application.runtime();
    let created = runtime
        .with_application_advancement(scope, |phase| {
            runtime
                .branches()
                .fork(runtime.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap();
    let mut original = match created {
        Ok(branch) => return branch,
        Err(Created::ProductUnpublished(original)) => original,
        Err(cause) => panic!("resource source fork: {cause:?}"),
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "original resource source fork must settle"
        );
        match runtime
            .with_application_advancement(scope, |phase| original.continue_in_advancement(&phase))
            .unwrap()
        {
            Ok(Continued::Performed(branch)) => return branch,
            Ok(Continued::NoEffect(cause)) => panic!("resource source fork no effect: {cause:?}"),
            Ok(Continued::ReleasedOriginal(cause)) => {
                panic!("resource source fork released: {cause:?}")
            }
            Err(failure) => {
                assert_eq!(
                    failure.denial(),
                    WorthQueryProductBranchRecoveryDenial::Busy
                );
                original = failure.into_recovery();
                std::thread::yield_now();
            }
        }
    }
}
