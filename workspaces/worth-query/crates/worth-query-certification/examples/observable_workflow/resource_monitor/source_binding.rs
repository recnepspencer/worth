//! Actual adoption preparation perturbs Native issuance before a fork-backed Resource.
use super::{resource::Monitor, Configuration};
use crate::document_retention_model::{schema::DocumentRetentionSchema, MixedApplication};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*,
    application_installation::WorthQueryProgramOwner, product::WorthQueryProductBranch,
};

pub(super) struct PreparedSource {
    fork: WorthQueryProductBranch,
    original: WorthQueryApplicationReadObservation,
}

pub(super) fn prepare(
    application: &MixedApplication,
    actor: &WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    scope: &WorthQueryRequestScope,
) -> PreparedSource {
    let runtime = application.runtime();
    let fork = runtime
        .with_host_advancement(|phase| {
            runtime
                .branches()
                .fork(runtime.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let original = runtime
        .request(actor, scope)
        .on_branch(fork)
        .retain_current_read()
        .unwrap();
    let target = *application
        .supported_program::<crate::program::MixedProgramP1>()
        .expect("the actual alternate program is rostered")
        .owned_revision();
    let programs = runtime.request(actor, scope).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    assert_eq!(prepared.selected_entity_count(), 2);
    // This is the real Native prepared transaction, not an unstaged EffectProgram.
    drop(prepared);
    PreparedSource { fork, original }
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
        "dropping the real prepared adoption burns Native issuance before sibling publication"
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
